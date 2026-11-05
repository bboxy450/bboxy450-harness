// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Confirmed flush through a caller-authenticated route, never a shell KB writer.
//! A caller retains the affine Ticket before polling an effect. Cancellation or
//! an unknown reply leaves that original operation queryable; it is not replayed.
use super::{
    deny,
    entry::{self, Invalid, Record},
    resolve::Resolver,
};
use crate::{protocol::*, redact::Set, store::Store};
use serde_json::{Value, json};
use std::{future::Future, pin::Pin};

pub type Reply<'a> = Pin<Box<dyn Future<Output = Result<Value, Invalid>> + Send + 'a>>;
/// Trusted program interface to an ALREADY authenticated route. This is neither
/// deserialized authority nor a discovery/bootstrap mechanism. An adapter must
/// honor original operation IDs across write/commit/status, or refuse the route.
/// Live wiring and durable ticket retention across process restarts belong to WP9.
pub trait Route: Send + Sync {
    fn root(&self) -> Option<&str>;
    fn call<'a>(&'a self, method: &'static str, args: Value) -> Reply<'a>;
}
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    NoRoute,
    Pending(&'static str),
    Failed(&'static str),
    Committed(RecordPin),
}
#[derive(Clone, Debug)]
enum Phase {
    Write,
    QueryWrite,
    Commit(RecordPin),
    QueryCommit(RecordPin),
    Done(RecordPin),
    Failed(&'static str),
}
/// One logical Pending item. No Clone/Deserialize or automatic restart redispatch.
/// A lost ticket is unavailable, never evidence authorizing a fresh write.
pub struct Ticket {
    entry: String,
    draft: Value,
    target: RecordPin,
    evidence: Vec<EvidenceRef>,
    write: String,
    commit: String,
    phase: Phase,
}
impl Ticket {
    pub fn prepare(item: &Item, set: &Set) -> Result<Self, Invalid> {
        let Item::MemoryWrite {
            entry,
            op: MemoryOp::Pending,
            record,
            evidence,
            draft: Some(draft),
        } = item
        else {
            return Err(Invalid("flush requires a complete Pending item"));
        };
        let r = Record::draft(draft)?;
        if r.validate_identity()?.id()? != *entry
            || record.commit.is_some()
            || record.root.is_empty()
            || entry::digest(&r.bytes) != record.digest
            || record.id.is_some()
        {
            return Err(Invalid("flush pending identity/pin mismatch"));
        }
        super::resolve::repository_path(&record.path)?;
        deny::value(draft, set).map_err(|_| Invalid("denied pending memory"))?;
        Ok(Self {
            entry: entry.clone(),
            draft: draft.clone(),
            target: record.clone(),
            evidence: evidence.clone(),
            write: ulid::Ulid::new().to_string(),
            commit: ulid::Ulid::new().to_string(),
            phase: Phase::Write,
        })
    }
    pub fn original_operations(&self) -> (&str, &str) {
        (&self.write, &self.commit)
    }
    fn matching(&self, pin: &RecordPin, committed: bool) -> bool {
        pin.root == self.target.root
            && super::resolve::repository_path(&pin.path).ok()
                == super::resolve::repository_path(&self.target.path).ok()
            && pin
                .id
                .as_deref()
                .is_some_and(|s| s.parse::<ulid::Ulid>().is_ok())
            && entry::valid_digest(&pin.digest)
            && if committed {
                pin.commit.as_deref().is_some_and(entry::valid_commit)
            } else {
                pin.commit.is_none()
            }
    }
    /// The effect state is advanced BEFORE awaiting transport. A dropped future
    /// therefore resumes by operation_status, even if the remote effect completed.
    /// One query is made per poll invocation; unavailable status never spins.
    pub async fn advance(
        &mut self,
        route: Option<&dyn Route>,
        resolver: &mut Resolver,
        set: &Set,
    ) -> Outcome {
        let Some(route) = route.filter(|r| r.root() == Some(self.target.root.as_str())) else {
            return Outcome::NoRoute;
        };
        loop {
            let (method, args, writing) = match self.phase.clone() {
                Phase::Write => {
                    let logs = resolver.logs.clone();
                    let all = match super::projection::union(logs.values().flatten().cloned()) {
                        Ok(v) => v,
                        Err(_) => return Outcome::Failed("flush projection unavailable"),
                    };
                    let projected = match super::projection::facts_with_logs(&all, &logs)
                        .and_then(|f| super::projection::Projection::build(&f))
                    {
                        Ok(v) => v,
                        Err(_) => return Outcome::Failed("flush projection unavailable"),
                    };
                    if let Some(state) = projected.entries.get(&self.entry) {
                        self.draft["frontmatter"]["status"] = json!(state.status());
                        self.draft["frontmatter"]["harnesses"] = json!(state.harnesses());
                        self.target.digest = match Record::draft(&self.draft) {
                            Ok(r) => entry::digest(r.bytes),
                            Err(_) => return Outcome::Failed("flush projected draft unavailable"),
                        };
                    }
                    self.phase = Phase::QueryWrite;
                    (
                        "knowledge_write",
                        json!({"op_id":self.write,"path":self.target.path,"frontmatter":self.draft["frontmatter"],"body":self.draft["body"]}),
                        true,
                    )
                }
                Phase::QueryWrite => (
                    "operation_status",
                    json!({"original_op_id":self.write}),
                    true,
                ),
                Phase::Commit(pin) => {
                    self.phase = Phase::QueryCommit(pin.clone());
                    (
                        "knowledge_commit",
                        json!({"op_id":self.commit,"record":pin}),
                        false,
                    )
                }
                Phase::QueryCommit(_) => (
                    "operation_status",
                    json!({"original_op_id":self.commit}),
                    false,
                ),
                Phase::Done(pin) => return Outcome::Committed(pin),
                Phase::Failed(reason) => return Outcome::Failed(reason),
            };
            let reply = match route.call(method, args).await {
                Ok(r) => r,
                Err(_) => return Outcome::Pending("flush outcome unknown"),
            };
            // Missing/unknown/incomplete is not a negative acknowledgement.
            if reply["status"] == "failed" {
                self.phase = Phase::Failed(if writing {
                    "write failed"
                } else {
                    "commit failed"
                });
                continue;
            }
            if reply["status"] != "completed" {
                return Outcome::Pending("flush outcome unknown");
            }
            let Ok(pin) = serde_json::from_value::<RecordPin>(reply["record"].clone()) else {
                return Outcome::Pending("flush pin unavailable");
            };
            if !self.matching(&pin, !writing) {
                self.phase = Phase::Failed("flush route/pin mismatch");
                continue;
            }
            if writing {
                self.phase = Phase::Commit(pin);
                continue;
            }
            let Phase::QueryCommit(written) = &self.phase else {
                return Outcome::Failed("flush operation mismatch");
            };
            if pin.root != written.root
                || pin.id != written.id
                || pin.path != written.path
                || pin.digest != written.digest
            {
                self.phase = Phase::Failed("commit changed written revision");
                continue;
            }
            // Refresh only the already mounted root at its unchanged canonical
            // path/name. A confirmed commit outside its actual origin/main is
            // still unavailable to the common resolver, never an eligibility pin.
            let Some(mounted) = resolver.roots.get(&pin.root).cloned() else {
                return Outcome::Pending("committed root unavailable");
            };
            let Ok(mut root) =
                super::resolve::Root::open(mounted.name.clone(), mounted.path.clone()).await
            else {
                return Outcome::Pending("committed root unavailable");
            };
            if root.repo != mounted.repo || root.path != mounted.path {
                return Outcome::Pending("committed root changed");
            }
            root.cross_repo = mounted.cross_repo;
            resolver.roots.insert(pin.root.clone(), root);
            let Ok(committed) = resolver.record(&pin).await else {
                return Outcome::Pending("committed revision unavailable");
            };
            if self.draft["frontmatter"].as_object().is_none_or(|front| {
                front
                    .iter()
                    .any(|(key, value)| committed.record.front.get(key) != Some(value))
            }) || !committed.signature_valid
                || committed
                    .record
                    .validate_identity()
                    .and_then(|i| i.id())
                    .ok()
                    .as_ref()
                    != Some(&self.entry)
                || committed.record.body
                    != Record::draft(&self.draft)
                        .map(|r| r.body)
                        .unwrap_or_default()
                || committed.record.scope().ok()
                    != Record::draft(&self.draft).ok().and_then(|r| r.scope().ok())
                || deny::value(&committed.record.front, set).is_err()
                || deny::text(&committed.record.body).is_err()
            {
                self.phase = Phase::Failed("committed memory changed identity/body");
                continue;
            }
            self.phase = Phase::Done(pin.clone());
            return Outcome::Committed(pin);
        }
    }
    /// Only a confirmed committed ticket can append Created. A retry verifies the
    /// exact existing own-log item rather than producing a duplicate record.
    pub async fn append(&self, store: &mut Store) -> Result<RecordPin, Invalid> {
        let Phase::Done(pin) = &self.phase else {
            return Err(Invalid("flush remains Pending"));
        };
        if store.closed || !store.events.first().is_some_and(|e|matches!(&e.event,Event::SessionStarted{meta} if meta.harness=="dream"&&meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Flush))) {
            return Err(Invalid("flush requires owned Flush session"));
        }
        let body = Item::MemoryWrite {
            entry: self.entry.clone(),
            op: MemoryOp::Created,
            record: pin.clone(),
            evidence: self.evidence.clone(),
            draft: None,
        };
        if store.events.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:b,..} if serde_json::to_value(b).ok()==serde_json::to_value(&body).ok())) {return Ok(pin.clone());}
        let item = ItemId::new();
        store
            .append(
                Event::ItemStarted {
                    turn: None,
                    item,
                    kind: ItemKind::MemoryWrite,
                    parent: None,
                    head: None,
                },
                None,
            )
            .await
            .map_err(|_| Invalid("flush log append failed"))?;
        store
            .append(
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body,
                },
                None,
            )
            .await
            .map_err(|_| Invalid("flush log append failed"))?;
        store
            .sync()
            .await
            .map_err(|_| Invalid("flush log sync failed"))?;
        Ok(pin.clone())
    }
}
