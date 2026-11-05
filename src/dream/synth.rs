// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Strict D1 candidates become proposals only; no producer approval or adoption.
use super::{
    deny,
    entry::{self, Identity, Invalid, Record},
    prompt::{self, Prompt},
    resolve::Resolver,
};
use crate::{protocol::*, redact::Set, store::Store};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const OUTPUT_BYTES: usize = 32 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    outcome: Outcome,
    candidates: Vec<Candidate>,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Outcome {
    Noop,
    Candidates,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Candidate {
    kind: String,
    title: String,
    description: String,
    why: String,
    how_to_apply: String,
    evidence: Vec<String>,
    links: Vec<Link>,
    scope_level: String,
    keywords: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub to: String,
    pub rel: String,
}
#[derive(Clone, Debug)]
pub struct Proposal {
    identity: Identity,
    scope: Scope,
    description: String,
    links: Vec<Link>,
    keywords: Vec<String>,
    evidence: Vec<EvidenceRef>,
}
fn nfc(text: &str) -> String {
    icu_normalizer::ComposingNormalizer::new_nfc()
        .normalize(text)
        .into_owned()
}
fn safe_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    !text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        && ![
            "ignore previous instructions",
            "ignore all instructions",
            "<|im_",
            "[inst]",
            "</episode",
            "</cluster",
            "system prompt:",
            "developer message:",
        ]
        .iter()
        .any(|s| lower.contains(s))
        && !lower
            .split(|c: char| {
                c.is_whitespace()
                    || matches!(c, '`' | '"' | '\'' | '(' | ')' | '<' | '>' | ',' | ';')
            })
            .any(|word| {
                deny::instruction_sink(word.trim_end_matches(['.', ':', '!', '?', ']', '}']))
            })
}
/// Parse atomically before any proposal is emitted. No model-provided digest,
/// scope repo, lifecycle, record path, identity, origin or trust bit is accepted.
pub async fn parse(
    output: &str,
    input: &Prompt,
    resolver: &mut Resolver,
    set: &Set,
) -> Result<Vec<Proposal>, Invalid> {
    if output.len() > OUTPUT_BYTES {
        return Err(Invalid("synthesis output byte bound"));
    }
    let raw: Value =
        serde_json::from_str(output).map_err(|_| Invalid("synthesis requires strict JSON"))?;
    deny::value(&raw, set).map_err(|_| Invalid("unsafe synthesis output"))?;
    // Deserialize the original bytes: struct decoding rejects duplicate known keys.
    let output: Output = serde_json::from_str(output)
        .map_err(|_| Invalid("unknown/duplicate synthesis field or outcome"))?;
    match output.outcome {
        Outcome::Noop if output.candidates.is_empty() => return Ok(vec![]),
        Outcome::Noop => return Err(Invalid("noop contains candidates")),
        Outcome::Candidates if output.candidates.is_empty() || output.candidates.len() > 3 => {
            return Err(Invalid("synthesis candidate bound"));
        }
        _ => {}
    }
    let mut proposals = BTreeMap::new();
    for mut candidate in output.candidates {
        candidate.title = nfc(&candidate.title);
        candidate.description = nfc(&candidate.description);
        candidate.why = nfc(&candidate.why);
        candidate.how_to_apply = nfc(&candidate.how_to_apply);
        if candidate.description.is_empty()
            || candidate.description.chars().count() > 1024
            || candidate.description.contains(['\n', '\r'])
            || candidate.title.contains(['\n', '\r'])
            || candidate.links.len() > 32
            || candidate.keywords.len() > 32
            || candidate
                .keywords
                .iter()
                .any(|k| k.is_empty() || k.chars().count() > 16)
            || [
                &candidate.kind,
                &candidate.title,
                &candidate.description,
                &candidate.why,
                &candidate.how_to_apply,
            ]
            .into_iter()
            .chain(candidate.keywords.iter())
            .any(|s| !safe_text(s))
        {
            return Err(Invalid("unsafe synthesis text/bounds"));
        }
        let level = match candidate.scope_level.as_str() {
            "repo" => ScopeLevel::Repo,
            "system" => ScopeLevel::System,
            _ => return Err(Invalid("unknown candidate scope")),
        };
        if level == ScopeLevel::System && input.scope.level != ScopeLevel::System {
            return Err(Invalid("candidate widens scope"));
        }
        let scope = Scope {
            level,
            repos: input.scope.repos.clone(),
        };
        if candidate.evidence.is_empty() || candidate.evidence.len() > 32 {
            return Err(Invalid("candidate evidence bound"));
        }
        let mut evidence = BTreeMap::new();
        for reference in candidate.evidence {
            let pin = input
                .refs
                .get(&reference)
                .ok_or(Invalid("invented candidate evidence"))?
                .clone();
            if evidence.insert(reference, pin.clone()).is_some() {
                return Err(Invalid("duplicate candidate evidence"));
            }
            Box::pin(resolver.evidence(&pin)).await?;
        }
        let evidence = evidence.into_values().collect::<Vec<_>>();
        Box::pin(resolver.scope(&scope, &evidence)).await?;
        let mut links = BTreeMap::new();
        for mut link in candidate.links {
            link.to = nfc(&link.to);
            if (!input.index.contains(&link.to) && !prompt::safe_slug(&link.to))
                || !matches!(
                    link.rel.as_str(),
                    "refines"
                        | "supports"
                        | "contradicts"
                        | "supersedes"
                        | "related"
                        | "applies_to"
                )
            {
                return Err(Invalid("unsafe candidate link"));
            }
            if input.index.contains(&link.to) {
                links.insert((link.to.clone(), link.rel.clone()), link);
            }
        }
        let identity = Identity {
            kind: candidate.kind,
            title: candidate.title,
            why: candidate.why,
            how_to_apply: candidate.how_to_apply,
            evidence: evidence
                .iter()
                .map(|p| serde_json::to_value(p).unwrap())
                .collect(),
            derived_from: vec![],
            revival: 0,
        };
        // Canonicalization normalizes evidence strings only for hashing. Keep
        // the actual pin spelling: Git paths are byte identities, including NFD.
        identity.validate()?;
        let id = identity.id()?;
        // Every Recall basis excludes exposed episode sessions, including when
        // the candidate cites their committed Task anchor rather than an Event.
        if input.source_sessions.iter().any(|session| resolver.logs.get(session).is_some_and(|events|events.iter().any(|e|matches!(&e.event,Event::ItemCompleted{body:Item::Recall{entry,..},..} if entry==&id)))) {
            return Err(Invalid("self-reinforcing candidate evidence"));
        }
        let mut keywords = candidate
            .keywords
            .iter()
            .map(|s| nfc(s))
            .collect::<Vec<_>>();
        keywords.sort();
        keywords.dedup();
        let proposal = Proposal {
            identity,
            scope,
            description: nfc(&candidate.description),
            links: links.into_values().collect(),
            keywords,
            evidence,
        };
        // The complete body and EntryId use the shared validator, never a D1 copy.
        let draft = proposal.draft(SessionId(ulid::Ulid::from(0u128)), Seq(1))?;
        draft.validate_identity()?;
        deny::value(&json!({"frontmatter":draft.front,"body":draft.body}), set)
            .map_err(|_| Invalid("unsafe normalized draft"))?;
        proposals.entry(id).or_insert(proposal);
    }
    Ok(proposals.into_values().collect())
}
impl Proposal {
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn links(&self) -> &[Link] {
        &self.links
    }
    pub fn draft(&self, session: SessionId, seq: Seq) -> Result<Record, Invalid> {
        let id = self.identity.id()?;
        let body = format!(
            "# {}\n\n## Why\n{}\n\n## How to apply\n{}\n\n## Evidence\n{}\n",
            self.identity.title,
            self.identity.why,
            self.identity.how_to_apply,
            self.evidence
                .iter()
                .map(|p| prompt::reference(p).map(|s| format!("- [[{s}]]")))
                .collect::<Result<Vec<_>, _>>()?
                .join("\n")
        );
        let front = json!({"type":"memory","kind":self.identity.kind,"title":self.identity.title,"description":self.description,"why":self.identity.why,"how_to_apply":self.identity.how_to_apply,"entry":id,"evidence":self.identity.evidence,"derived_from":[],"revival":0,"scope":self.scope,"status":"proposed","harnesses":[],"origin":{"kind":"dream","session":session,"seq":seq},"links":self.links,"keywords":self.keywords,"content_sha256":entry::digest(&body)});
        Record::draft(&json!({"frontmatter":front,"body":body}))
    }
}
// A provisional title slug carries an identity suffix; only confirmed MCP flush
// can assign a final record path. This never creates or replaces a KB file.
fn pending_slug(title: &str, id: &str) -> String {
    let mut slug = String::new();
    for ch in title.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            if slug.len() >= 50 {
                break;
            }
            slug.push(ch);
        } else if !slug.is_empty() && !slug.ends_with('-') && slug.len() < 50 {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    format!(
        "{}-{}",
        if slug.is_empty() { "memory" } else { slug },
        &id[7..19]
    )
}
/// D1 seam for WP9: append to the caller's existing owned, open dream Store.
/// Validated output grants no KB write, trial/adoption, sink or receipt capability.
pub async fn append_pending(
    store: &mut Store,
    proposals: &[Proposal],
    root: &str,
    set: &Set,
) -> Result<Vec<String>, Invalid> {
    if proposals.len()>3 || root.is_empty() || store.closed || !store.events.first().is_some_and(|e|matches!(&e.event,Event::SessionStarted{meta} if meta.harness=="dream" && meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Synthesis))) { return Err(Invalid("proposal requires owned synthesis dream session")); }
    let mut items = Vec::new();
    for (n, proposal) in proposals.iter().enumerate() {
        let seq = Seq(store
            .head()
            .0
            .checked_add((n as u64) * 4 + 2)
            .ok_or(Invalid("proposal sequence exhausted"))?);
        let draft = proposal.draft(store.session, seq)?;
        draft.validate_identity()?;
        let id = proposal.identity.id()?;
        let value = json!({"frontmatter":draft.front,"body":draft.body});
        deny::value(&value, set).map_err(|_| Invalid("unsafe pending draft"))?;
        let pin = RecordPin {
            root: if proposal.scope.level == ScopeLevel::System {
                "agents".into()
            } else {
                root.into()
            },
            id: None,
            path: format!(
                "memories/{}.md",
                pending_slug(&proposal.identity.title, &id)
            ),
            digest: entry::digest(&draft.bytes),
            commit: None,
        };
        items.push((
            id.clone(),
            Item::MemoryWrite {
                entry: id.clone(),
                op: MemoryOp::Pending,
                record: pin,
                evidence: proposal.evidence.clone(),
                draft: Some(value),
            },
            Item::MemoryState {
                entry: id,
                state: EntryState::Proposed,
                harness: None,
                reason: "D1 proposal; trust, trial and semantic audit not established".into(),
                refs: proposal.evidence.clone(),
            },
        ));
    }
    let mut ids = Vec::new();
    for (entry, write, state) in items {
        for body in [write, state] {
            let item = ItemId::new();
            store
                .append(
                    Event::ItemStarted {
                        turn: None,
                        item,
                        kind: body.kind(),
                        parent: None,
                        head: None,
                    },
                    None,
                )
                .await
                .map_err(|_| Invalid("proposal append failed"))?;
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
                .map_err(|_| Invalid("proposal append failed"))?;
        }
        ids.push(entry);
    }
    store
        .sync()
        .await
        .map_err(|_| Invalid("proposal sync failed"))?;
    Ok(ids)
}

/// Reuse the ordinary provider lifecycle while giving D1 exactly two
/// messages and no tools. Buffering keeps rejected model secrets out of the log.
#[derive(Clone)]
pub struct TextOnlyProvider {
    inner: std::sync::Arc<dyn crate::provider::Provider>,
    input: Prompt,
    set: Set,
}
impl TextOnlyProvider {
    pub fn new(
        inner: std::sync::Arc<dyn crate::provider::Provider>,
        input: Prompt,
        set: Set,
    ) -> Self {
        Self { inner, input, set }
    }
}
impl crate::provider::Provider for TextOnlyProvider {
    fn id(&self) -> &str {
        self.inner.id()
    }
    fn caps(&self) -> crate::provider::Caps {
        self.inner.caps()
    }
    fn stream(
        &self,
        mut request: crate::provider::ModelRequest,
        cancel: tokio_util::sync::CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = crate::provider::ModelEvent> + Send>>
    {
        use crate::provider::{Finish, ModelEvent as M, ProviderError, ToolChoice};
        use futures_util::StreamExt;
        let this = self.clone();
        Box::pin(async_stream::stream! {
            if prompt::input_bytes(&this.input.user)>prompt::INPUT_BYTES {yield M::Failed(ProviderError::new(ErrorClass::BadRequest,"D1 input bound"));return;}
            request.messages=vec![json!({"role":"system","content":prompt::SYSTEM}),json!({"role":"user","content":this.input.user})];
            request.tools.clear();request.tool_choice=ToolChoice::None;request.stable_prefix=1;request.max_tokens=Some(8192);
            let deadline=tokio::time::Instant::now()+std::time::Duration::from_secs(1800);
            let mut stream=this.inner.stream(request,cancel.clone());let mut text=String::new();let mut bytes=0usize;let mut output=false;
            loop {
                let event=tokio::select!{biased;_=cancel.cancelled()=>{yield M::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},event=tokio::time::timeout_at(deadline,stream.next())=>match event {Ok(event)=>event,Err(_)=>{yield M::Failed(ProviderError::new(ErrorClass::Cancelled,"D1 deadline"));return;}}};
                match event {
                    Some(M::Text(part))=>{output=true;bytes=bytes.saturating_add(part.len());if bytes>OUTPUT_BYTES{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 output bound"));return;}text.push_str(&part);},
                    Some(M::Reasoning(part))=>{output=true;bytes=bytes.saturating_add(part.len());if bytes>OUTPUT_BYTES{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 output bound"));return;}},
                    Some(M::ReasoningOpaque(value))=>{
                        let size=match serde_json::to_vec(&value){Ok(bytes)=>bytes.len(),Err(_)=>{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 reasoning bound"));return;}};
                        bytes=bytes.saturating_add(size);output=true;
                        if bytes>OUTPUT_BYTES{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 output bound"));return;}
                    },
                    Some(M::ToolCallDelta{..}|M::ToolCallReady{..})=>{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 requires text only"));return;},
                    Some(M::Retrying{..}) if output=>{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 retry after output"));return;},
                    Some(event@M::Retrying{..})=>yield event,
                    Some(M::Usage(usage))=>{
                        if deny::value(&serde_json::to_value(&usage).unwrap(),&this.set).is_err(){yield M::Failed(ProviderError::new(ErrorClass::Malformed,"unsafe D1 usage"));return;}
                        let over_budget=usage.input_tokens.saturating_add(usage.output_tokens).saturating_add(usage.reasoning_tokens.unwrap_or(0))>200_000;
                        yield M::Usage(usage);
                        if over_budget {yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 spend bound"));return;}
                    },
                    Some(M::Finish(Finish::Stop))=>{
                        let tail=tokio::select!{biased;_=cancel.cancelled()=>{yield M::Failed(ProviderError::new(ErrorClass::Cancelled,"cancelled"));return;},tail=tokio::time::timeout_at(deadline,stream.next())=>tail};
                        if !matches!(tail,Ok(None)){yield M::Failed(ProviderError::new(ErrorClass::Malformed,"D1 response after finish/incomplete stream"));return;}
                        if deny::value(&json!(text),&this.set).is_err(){yield M::Failed(ProviderError::new(ErrorClass::Malformed,"unsafe D1 output"));return;}
                        yield M::Text(text);yield M::Finish(Finish::Stop);return;
                    },
                    Some(M::Failed(error))=>{yield M::Failed(ProviderError::new(error.class,"D1 provider failed"));return;},
                    _=>{yield M::Failed(ProviderError::new(ErrorClass::Malformed,"incomplete D1 response"));return;}
                }
            }
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Synthesis {
    pub outcome: &'static str,
    pub entries: Vec<String>,
    pub estimated_tokens_upper_bound: u64,
    pub semantic_audit: &'static str,
}
async fn item(store: &mut Store, turn: TurnId, body: Item) -> Result<(), Invalid> {
    let item = ItemId::new();
    let mut check = vec![
        store
            .events
            .first()
            .ok_or(Invalid("D1 missing header"))?
            .clone(),
    ];
    for event in [
        Event::ItemStarted {
            turn: None,
            item,
            kind: body.kind(),
            parent: None,
            head: None,
        },
        Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: body.clone(),
        },
        Event::SessionClosed,
    ] {
        let seq = Seq(check.len() as u64 + 1);
        check.push(Envelope {
            v: PROTOCOL,
            session: store.session,
            epoch: store.epoch,
            seq: Some(seq),
            at: seq,
            ts_ms: 0,
            cause: None,
            native: None,
            raw: None,
            ext: Default::default(),
            event,
        });
    }
    crate::protocol::laws::check_authored(&check)
        .map_err(|_| Invalid("D1 item violates authored laws"))?;
    store
        .append(
            Event::ItemStarted {
                turn: Some(turn),
                item,
                kind: body.kind(),
                parent: None,
                head: None,
            },
            None,
        )
        .await
        .map_err(|_| Invalid("D1 item start failed"))?;
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
        .map_err(|_| Invalid("D1 item completion failed"))?;
    Ok(())
}
/// One bounded provider call on the caller's ordinary single-writer Store.
/// WP9 retains this SAME writer for its shared final receipt and close sequence.
pub async fn run(
    store: &mut Store,
    provider: std::sync::Arc<dyn crate::provider::Provider>,
    model: String,
    input: Prompt,
    resolver: &mut Resolver,
    set: Set,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Synthesis, Invalid> {
    use crate::provider::{Finish, ModelEvent as M, ModelRequest, Provider, ToolChoice};
    use futures_util::StreamExt;
    if store.closed || input.refs.values().any(|pin|matches!(pin,EvidenceRef::Event{session,..} if session==&store.session))
        || !store.events.first().is_some_and(|e|matches!(&e.event,Event::SessionStarted{meta} if meta.harness=="dream" && meta.dream.as_ref().is_some_and(|d|d.phase==DreamPhase::Synthesis && !matches!(d.trigger,Trigger::Unknown(_)) && d.scope==input.scope && d.prompt_version.as_deref()==Some(prompt::VERSION) && d.thresholds.as_deref()==Some(prompt::THRESHOLDS)))) {return Err(Invalid("D1 requires matching owned dream session"));}
    if prompt::input_bytes(&input.user) > prompt::INPUT_BYTES
        || model.is_empty()
        || model.len() > 1024
    {
        return Err(Invalid("D1 input/model bound"));
    }
    deny::value(&json!(model), &set).map_err(|_| Invalid("unsafe D1 model"))?;
    Box::pin(resolver.scope(&input.scope, &[])).await?;
    deny::value(&json!(input.user), &set).map_err(|_| Invalid("unsafe D1 input"))?;
    let turn = TurnId::new();
    store
        .append(Event::TurnQueued { turn }, None)
        .await
        .map_err(|_| Invalid("D1 turn append failed"))?;
    store
        .append(
            Event::TurnStarted {
                turn,
                model: Some(model.clone()),
                origin: None,
            },
            None,
        )
        .await
        .map_err(|_| Invalid("D1 turn append failed"))?;
    item(
        store,
        turn,
        Item::UserMessage {
            blocks: vec![InputBlock::Text {
                text: input.user.clone(),
            }],
        },
    )
    .await?;
    let backend = Backend {
        kind: BackendKind::Engine,
        name: provider.id().into(),
        version: Some(env!("CARGO_PKG_VERSION").into()),
    };
    let usage_model = model.clone();
    let mut usage = None;
    let response = async {
        let guarded = TextOnlyProvider::new(provider, input.clone(), set.clone());
        let mut stream = guarded.stream(
            ModelRequest {
                model,
                messages: vec![],
                tools: vec![],
                tool_choice: ToolChoice::None,
                max_tokens: Some(8192),
                stable_prefix: 1,
            },
            cancel.clone(),
        );
        let mut output = String::new();
        let mut usage_events = 0usize;
        let mut finished = false;
        while let Some(event) = stream.next().await {
            match event {
                M::Text(text) => output.push_str(&text),
                M::Usage(value) => {
                    usage = Some(value);
                    usage_events += 1;
                    if usage_events > 16 {
                        return Err(Invalid("D1 usage event bound"));
                    }
                }
                M::Retrying {
                    attempt,
                    wait_ms,
                    class,
                } => {
                    if matches!(class, ErrorClass::Unknown(_)) {
                        return Err(Invalid("unknown provider retry class"));
                    }
                    item(
                        store,
                        turn,
                        Item::Notice {
                            level: Level::Info,
                            text: format!(
                                "D1 provider retry {attempt}, wait {wait_ms}ms ({class:?})"
                            ),
                        },
                    )
                    .await?;
                }
                M::Finish(Finish::Stop) => {
                    finished = true;
                }
                M::Failed(_) => return Err(Invalid("D1 provider failed or cancelled")),
                _ => return Err(Invalid("D1 text response required")),
            }
        }
        if !finished {
            return Err(Invalid("incomplete D1 response"));
        }
        Ok(output)
    }
    .await;
    let mut turn_usage = None;
    let result = async {
        // Retain cost even when the response fails, is cancelled, or is rejected.
        // Bound the entire response, including suppressed reasoning/opaque
        // content. Missing Usage never becomes a zero-cost or text-only arm.
        let estimated_tokens_upper_bound =
            prompt::input_bytes(&input.user).saturating_add(OUTPUT_BYTES) as u64;
        let estimated = Usage {
            input_tokens: prompt::input_bytes(&input.user) as u64,
            output_tokens: OUTPUT_BYTES as u64,
            source: Some(NumberSource::Estimated),
            ..Default::default()
        };
        let usage =
            usage
                .unwrap_or(estimated)
                .authored(UsageScope::Call, backend, Some(usage_model));
        turn_usage = Usage::sum_calls(std::slice::from_ref(&usage)).map_err(Invalid)?;
        item(store, turn, Item::Usage { usage }).await?;
        let output = response?;
        let proposals = Box::pin(parse(&output, &input, resolver, &set)).await?;
        item(store, turn, Item::AgentMessage { markdown: output }).await?;
        let root = resolver
            .roots
            .values()
            .find(|r| r.repo == input.scope.repos[0])
            .ok_or(Invalid("proposal root unavailable"))?
            .name
            .clone();
        if cancel.is_cancelled() {
            return Err(Invalid("D1 cancelled before proposals"));
        }
        let entries = append_pending(store, &proposals, &root, &set).await?;
        store
            .append(
                Event::TurnEnded {
                    turn,
                    reason: StopReason::EndTurn,
                    usage: turn_usage.clone(),
                },
                None,
            )
            .await
            .map_err(|_| Invalid("D1 ending append failed"))?;
        store.sync().await.map_err(|_| Invalid("D1 sync failed"))?;
        Ok(Synthesis {
            outcome: if entries.is_empty() {
                "noop"
            } else {
                "proposed"
            },
            entries,
            estimated_tokens_upper_bound,
            semantic_audit: "UNPROVEN: caller final run receipt required; no auto-adoption",
        })
    }
    .await;
    if result.is_err() {
        let reason = if cancel.is_cancelled() {
            StopReason::Cancelled
        } else {
            StopReason::Error {
                class: ErrorClass::Malformed,
                message: "D1 response rejected; no proposal authority".into(),
            }
        };
        // A failed append may leave partial/unproven IR. Never recover or reopen it here.
        if !store
            .events
            .iter()
            .any(|e| matches!(e.event,Event::TurnEnded{turn:t,..} if t==turn))
        {
            let _ = store
                .append(
                    Event::TurnEnded {
                        turn,
                        reason,
                        usage: turn_usage,
                    },
                    None,
                )
                .await;
        }
    }
    result
}

pub async fn cli(args: impl Iterator<Item = String>) -> Result<i32, String> {
    match Box::pin(cli_inner(args)).await {
        Ok(code) => Ok(code),
        Err(error) => {
            eprintln!("dream synthesize: {error}");
            Ok(2)
        }
    }
}
// Only static, non-source diagnostics may cross the CLI error boundary.
async fn cli_inner(args: impl Iterator<Item = String>) -> Result<i32, &'static str> {
    use super::{d0, resolve::Root, trust::Trust};
    use std::{path::PathBuf, sync::Arc};
    let mut args = args;
    let mut sessions = None;
    let mut fingerprint = None;
    let mut scripted = None;
    let mut roots = Vec::new();
    let mut policy = None;
    let mut json_output = false;
    let mut profile = "scripted".to_owned();
    let mut level = ScopeLevel::Repo;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sessions" => sessions = Some(PathBuf::from(args.next().ok_or("missing sessions")?)),
            "--cluster" => fingerprint = Some(args.next().ok_or("missing cluster")?),
            "--scripted" => {
                scripted = Some(PathBuf::from(
                    args.next().ok_or("missing scripted fixture")?,
                ))
            }
            "--root" => {
                let root = args.next().ok_or("missing root")?;
                let (name, path) = root.split_once('=').ok_or("root requires NAME=PATH")?;
                roots.push(
                    Root::open(name.into(), path.into())
                        .await
                        .map_err(|_| "invalid explicit root")?,
                );
            }
            "--trust-policy" => policy = Some(PathBuf::from(args.next().ok_or("missing policy")?)),
            "--profile" => profile = args.next().ok_or("missing profile")?,
            "--scope" => {
                level = match args.next().as_deref() {
                    Some("repo") => ScopeLevel::Repo,
                    Some("system") => ScopeLevel::System,
                    _ => return Ok(2),
                }
            }
            "--json" => json_output = true,
            "--help" | "-h" => {
                println!(
                    "dream synthesize [--sessions DIR] [--cluster FP] [--root NAME=PATH] [--trust-policy PATH] [--scripted SSE] [--profile NAME] [--scope repo|system] [--json]"
                );
                return Ok(0);
            }
            _ => return Ok(2),
        }
    }
    let directory = super::directory().map_err(|_| "HOME unavailable")?;
    let sessions = sessions.unwrap_or_else(|| directory.with_file_name("sessions"));
    let set = Set::default();
    crate::capability::register(&set).await;
    if profile.len() > 128
        || crate::provider::profile::Profile::file_in(std::path::Path::new("."), &profile).is_err()
        || deny::value(&json!(profile), &set).is_err()
    {
        return Ok(2);
    }
    if roots.is_empty() {
        let cwd = std::env::current_dir().map_err(|_| "cwd unavailable")?;
        let repo = crate::vcs::repo_key(&cwd)
            .await
            .ok_or("repository unavailable")?;
        roots.push(
            Root::open(repo, cwd)
                .await
                .map_err(|_| "repository root unavailable")?,
        );
    }
    let scope = Scope {
        level,
        repos: roots.iter().map(|r| r.repo.clone()).collect(),
    };
    let forbidden = roots
        .iter()
        .map(|r| r.path.clone())
        .chain(std::iter::once(sessions.clone()))
        .collect();
    let mut resolver = Box::pin(Resolver::open(sessions.clone(), roots))
        .await
        .map_err(|_| "retained sessions unavailable")?;
    let policy = policy.unwrap_or_else(|| directory.join("trust.json"));
    if let Ok(trust) = Box::pin(Trust::load(policy, forbidden)).await {
        resolver = resolver.with_trust(trust);
    }
    let replay = Box::pin(d0::replay_with_sessions(
        vec![sessions.clone()],
        directory.clone(),
        sessions.clone(),
        false,
        set.clone(),
    ))
    .await
    .map_err(|error| {
        // Classification only: raw IO text may contain quoted source paths/data.
        eprintln!(
            "D0 replay rejected: kind={:?}, errno={:?}",
            error.kind(),
            error.raw_os_error()
        );
        match error.to_string().as_str() {
            "D0 exceeded 1ms CPU between awaits" => "D0 exceeded 1ms CPU between awaits",
            "invalid source transcript" => "D0 invalid source transcript",
            "cache fingerprint mismatch" => "D0 cache fingerprint mismatch",
            "unknown cache pin" => "D0 unknown cache pin",
            _ => "D0 evidence unavailable",
        }
    })?;
    let clusters = if let Some(fp) = fingerprint {
        vec![fp]
    } else {
        replay
            .episodes
            .iter()
            .flat_map(|e| e.fingerprints.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    let mut selected = None;
    for fp in clusters {
        if let Some(input) = Box::pin(prompt::prepare(
            &mut resolver,
            &replay.episodes,
            &fp,
            scope.clone(),
            &set,
        ))
        .await
        .map_err(|_| "invalid synthesis evidence/scope/bounds")?
        {
            selected = Some(input);
            break;
        }
    }
    let Some(input) = selected else {
        println!(
            "{}",
            json!({"outcome":"noop","reason":"insufficient episodes or independently trusted verdict anchor"})
        );
        return Ok(0);
    };
    let Some(scripted) = scripted else {
        println!(
            "{}",
            json!({"outcome":"skipped","reason":"provider/profile/quota scheduling belongs to run caller; no inference started"})
        );
        return Ok(0);
    };
    let fp = input.fingerprint.clone();
    let claim = crate::runtime::blocking(move || d0::Claim::acquire(&directory, &fp))
        .await
        .map_err(|_| "cluster claim unavailable")?;
    let Some(_claim) = claim else {
        println!("{}", json!({"outcome":"skipped","reason":"cluster busy"}));
        return Ok(0);
    };
    let bytes = crate::runtime::blocking(move || {
        use std::io::Read;
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(scripted)?;
        if !file.metadata()?.is_file() {
            return Err(std::io::Error::other("script must be a regular file"));
        }
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 1024 * 1024 {
            return Err(std::io::Error::other("script bound"));
        }
        Ok::<_, std::io::Error>(bytes)
    })
    .await
    .map_err(|_| "script unavailable/bound")?;
    let session = SessionId::new();
    let cwd = std::env::current_dir().map_err(|_| "cwd unavailable")?;
    let meta = SessionMeta {
        session,
        cwd,
        profile: Some(profile),
        fork: None,
        protocol: PROTOCOL,
        harness: "dream".into(),
        created_ms: crate::store::now_ms(),
        vcs: None,
        source: None,
        dream: Some(DreamRun {
            phase: DreamPhase::Synthesis,
            trigger: Trigger::Operator { message: None },
            scope: input.scope.clone(),
            parent: None,
            prompt_version: Some(prompt::VERSION.into()),
            thresholds: Some(prompt::THRESHOLDS.into()),
        }),
    };
    let mut store = Store::open(sessions, meta, None, set.clone())
        .await
        .map_err(|_| "synthesis store unavailable")?;
    let result = Box::pin(run(
        &mut store,
        Arc::new(crate::provider::ScriptedProvider::new(vec![bytes])),
        "scripted".into(),
        input,
        &mut resolver,
        set,
        tokio_util::sync::CancellationToken::new(),
    ))
    .await;
    store
        .append(Event::SessionClosed, None)
        .await
        .map_err(|_| "synthesis close failed")?;
    match result {
        Ok(result) => {
            if json_output {
                println!("{}", json!({"session":session,"synthesis":result}));
            } else {
                println!(
                    "D1 {}: {} proposals; semantic audit UNPROVEN (no final run receipt)",
                    session,
                    result.entries.len()
                );
            }
            Ok(0)
        }
        Err(_) => {
            eprintln!("D1 rejected: no proposal authority; semantic audit UNPROVEN");
            Ok(1)
        }
    }
}
