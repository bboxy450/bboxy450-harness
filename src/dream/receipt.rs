// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Pure pre-seal effect interpreter. This is not the still-unbaselined receipt seal/R.
//! The final carrier integration and constructive sealing interface remain required.
use super::entry::Invalid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Generated {
    pub path: PathBuf,
    pub sha256: String,
    pub len: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultCode {
    Ok,
    Eexist,
    Error,
    Absent,
    Present,
    Symlink,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Read {
        path: PathBuf,
        result: ResultCode,
        sha256: Option<String>,
        len: Option<u64>,
    },
    Mkdir {
        path: PathBuf,
        result: ResultCode,
    },
    CreateTemp {
        path: PathBuf,
        result: ResultCode,
    },
    WriteTemp {
        path: PathBuf,
        sha256: String,
        len: u64,
        result: ResultCode,
    },
    Fsync {
        path: PathBuf,
        result: ResultCode,
    },
    LinkNoreplace {
        from: PathBuf,
        to: PathBuf,
        result: ResultCode,
    },
    FsyncDir {
        path: PathBuf,
        result: ResultCode,
    },
    UnlinkOwnTemp {
        path: PathBuf,
        result: ResultCode,
    },
    AppendOwnLog {
        session: crate::protocol::SessionId,
        seq_from: u64,
        seq_to: u64,
        result: ResultCode,
    },
    WriteOwnBlob {
        sha256: String,
        result: ResultCode,
    },
    CacheWrite {
        path: PathBuf,
        result: ResultCode,
    },
}
#[derive(Clone, Debug, Serialize)]
pub struct Effect {
    pub i: u64,
    #[serde(flatten)]
    pub operation: Operation,
}
/// Parsing unknown effects fails closed; this interface does not invent a receipt carrier.
pub fn parse_effects(values: &[Value]) -> Result<Vec<Effect>, Invalid> {
    values
        .iter()
        .map(|v| {
            if v.get("result").is_none() {
                return Err(Invalid("malformed_effect"));
            }
            if !matches!(
                v.get("op").and_then(Value::as_str),
                Some(
                    "read"
                        | "mkdir"
                        | "create_temp"
                        | "write_temp"
                        | "fsync"
                        | "link_noreplace"
                        | "fsync_dir"
                        | "unlink_own_temp"
                        | "append_own_log"
                        | "write_own_blob"
                        | "cache_write"
                )
            ) {
                return Err(Invalid("trace:clobber"));
            }
            let mut fields = v.clone();
            let i = fields
                .as_object_mut()
                .and_then(|o| o.remove("i"))
                .and_then(|i| i.as_u64())
                .ok_or(Invalid("malformed_effect"))?;
            let operation =
                serde_json::from_value(fields).map_err(|_| Invalid("malformed_effect"))?;
            Ok(Effect { i, operation })
        })
        .collect()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Published,
    Idempotent,
    Skipped,
}
#[derive(Clone, Debug)]
pub struct Interpretation {
    pub outcome: Outcome,
    pub temp_leaks: Vec<PathBuf>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    Absent,
    Seen(String, u64),
    Unsafe,
    Dir,
    Created,
    Written(String, u64),
    Synced(String, u64),
    Linked(String, u64),
    Published,
    Conflict,
    Idempotent,
    Failed,
    Removed,
}
fn absolute(path: &Path) -> bool {
    path.is_absolute()
        && path.components().all(|c| {
            matches!(
                c,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )
        })
}
fn sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn result(result: &ResultCode, allowed: &[ResultCode]) -> Result<(), Invalid> {
    if allowed.contains(result) {
        Ok(())
    } else {
        Err(Invalid("malformed_effect"))
    }
}
/// The caller supplies authorized cache locations. No policy, path or source authority
/// is taken from a receipt. This validates only the agreed pre-seal vocabulary E.
pub fn interpret(
    effects: &[Effect],
    generated: &[Generated],
    session: crate::protocol::SessionId,
    caches: &[PathBuf],
    declared: Option<Outcome>,
) -> Result<Interpretation, Invalid> {
    let mut targets = BTreeMap::new();
    for g in generated {
        if !absolute(&g.path) || !sha(&g.sha256) || targets.insert(g.path.clone(), g).is_some() {
            return Err(Invalid("malformed_generated"));
        }
    }
    let mut states = BTreeMap::new();
    let mut observed_present = BTreeSet::new();
    let mut linked = BTreeSet::new();
    let mut attempted = BTreeSet::new();
    for (index, effect) in effects.iter().enumerate() {
        if effect.i != index as u64 {
            return Err(Invalid("truncated_witness"));
        }
        use Operation::*;
        use ResultCode as R;
        match &effect.operation {
            Read {
                path,
                result: r,
                sha256,
                len,
            } => {
                result(r, &[R::Absent, R::Present, R::Symlink, R::Error])?;
                if !absolute(path) {
                    return Err(Invalid("malformed_effect"));
                }
                let state = match r {
                    R::Absent => {
                        if sha256.is_some() || len.is_some() {
                            return Err(Invalid("malformed_effect"));
                        }
                        State::Absent
                    }
                    R::Present => {
                        let hash = sha256
                            .as_ref()
                            .filter(|h| sha(h))
                            .ok_or(Invalid("malformed_effect"))?;
                        let len = len.ok_or(Invalid("malformed_effect"))?;
                        observed_present.insert(path.clone());
                        let eligible =
                            !attempted.contains(path) || states.get(path) == Some(&State::Conflict);
                        if eligible
                            && targets
                                .get(path)
                                .is_some_and(|g| g.sha256 == *hash && g.len == len)
                        {
                            State::Idempotent
                        } else {
                            State::Seen(hash.clone(), len)
                        }
                    }
                    _ => {
                        if sha256.is_some() || len.is_some() {
                            return Err(Invalid("malformed_effect"));
                        }
                        State::Unsafe
                    }
                };
                // Observation does not erase an already completed publication or temp ownership.
                if !matches!(
                    states.get(path),
                    Some(
                        State::Published
                            | State::Linked(..)
                            | State::Created
                            | State::Written(..)
                            | State::Synced(..)
                    )
                ) {
                    states.insert(path.clone(), state);
                }
            }
            CreateTemp { path, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                let name = path
                    .file_name()
                    .and_then(|p| p.to_str())
                    .and_then(|p| p.strip_prefix(".dream-tmp-"));
                if !absolute(path)
                    || states.contains_key(path)
                    || !name.is_some_and(|s| s.parse::<ulid::Ulid>().is_ok())
                    || !generated.iter().any(|g| g.path.parent() == path.parent())
                {
                    return Err(Invalid("trace:unowned_source"));
                }
                states.insert(
                    path.clone(),
                    if *r == R::Ok {
                        State::Created
                    } else {
                        State::Failed
                    },
                );
            }
            WriteTemp {
                path,
                sha256,
                len,
                result: r,
            } => {
                result(r, &[R::Ok, R::Error])?;
                if observed_present.contains(path) || states.get(path) != Some(&State::Created) {
                    return Err(Invalid("trace:clobber"));
                }
                if !sha(sha256) {
                    return Err(Invalid("malformed_effect"));
                }
                states.insert(
                    path.clone(),
                    if *r == R::Ok {
                        State::Written(sha256.clone(), *len)
                    } else {
                        State::Failed
                    },
                );
            }
            Fsync { path, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                let Some(State::Written(hash, len)) = states.get(path) else {
                    return Err(Invalid("trace:unowned_source"));
                };
                let state = if *r == R::Ok {
                    State::Synced(hash.clone(), *len)
                } else {
                    State::Failed
                };
                states.insert(path.clone(), state);
            }
            LinkNoreplace {
                from,
                to,
                result: r,
            } => {
                result(r, &[R::Ok, R::Eexist, R::Error])?;
                let (hash, len) = match states.get(from) {
                    Some(State::Synced(h, l)) => (h, l),
                    Some(State::Created | State::Written(..)) => {
                        return Err(Invalid("trace:unsynced"));
                    }
                    _ => return Err(Invalid("trace:unowned_source")),
                };
                if !targets
                    .get(to)
                    .is_some_and(|g| g.sha256 == *hash && g.len == *len)
                    || from.parent() != to.parent()
                {
                    return Err(Invalid("trace:temp_digest"));
                }
                if observed_present.contains(to) && *r != R::Eexist {
                    return Err(Invalid("trace:inconsistent"));
                }
                if linked.contains(to)
                    || matches!(
                        states.get(to),
                        Some(State::Unsafe | State::Linked(..) | State::Published)
                    )
                {
                    return Err(Invalid("trace:clobber"));
                }
                attempted.insert(to.clone());
                let state = match r {
                    R::Ok => {
                        linked.insert(to.clone());
                        State::Linked(hash.clone(), *len)
                    }
                    R::Eexist => State::Conflict,
                    _ => State::Failed,
                };
                states.insert(to.clone(), state);
            }
            FsyncDir { path, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                if !absolute(path) {
                    return Err(Invalid("malformed_effect"));
                }
                if *r == R::Ok {
                    for (target, state) in &mut states {
                        if target.parent() == Some(path) && matches!(state, State::Linked(..)) {
                            *state = State::Published;
                        }
                    }
                }
            }
            UnlinkOwnTemp { path, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                if !matches!(
                    states.get(path),
                    Some(State::Created | State::Written(..) | State::Synced(..))
                ) {
                    return Err(Invalid("trace:clobber"));
                }
                if *r == R::Ok {
                    states.insert(path.clone(), State::Removed);
                }
            }
            Mkdir { path, result: r } => {
                result(r, &[R::Ok, R::Eexist, R::Error])?;
                if !absolute(path) || states.get(path) != Some(&State::Absent) {
                    return Err(Invalid("trace:clobber"));
                }
                states.insert(
                    path.clone(),
                    if *r == R::Ok {
                        State::Dir
                    } else {
                        State::Failed
                    },
                );
            }
            AppendOwnLog {
                session: owner,
                seq_from,
                seq_to,
                result: r,
            } => {
                result(r, &[R::Ok, R::Error])?;
                if *owner != session || seq_from > seq_to || *seq_from == 0 {
                    return Err(Invalid("writer_touched_source"));
                }
            }
            WriteOwnBlob { sha256, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                if !sha(sha256) {
                    return Err(Invalid("malformed_effect"));
                }
                // Only the containing session's blob store is expressible by this op.
                // The pending seal schema must prevent self-referential own-blob traces.
            }
            CacheWrite { path, result: r } => {
                result(r, &[R::Ok, R::Error])?;
                if !absolute(path)
                    || !caches
                        .iter()
                        .any(|c| absolute(c) && path.starts_with(c) && path != c)
                {
                    return Err(Invalid("writer_touched_source"));
                }
            }
        }
    }
    let complete = generated.iter().all(|g| {
        matches!(
            states.get(&g.path),
            Some(State::Published | State::Idempotent)
        )
    });
    let published = generated
        .iter()
        .any(|g| states.get(&g.path) == Some(&State::Published));
    let outcome = if complete && published {
        Outcome::Published
    } else if complete && !generated.is_empty() {
        Outcome::Idempotent
    } else {
        Outcome::Skipped
    };
    if matches!(declared, Some(Outcome::Published | Outcome::Idempotent))
        && generated
            .iter()
            .any(|g| matches!(states.get(&g.path), Some(State::Linked(..))))
    {
        return Err(Invalid("trace:no_dir_sync"));
    }
    if declared.as_ref().is_some_and(|d| *d != outcome) {
        return Err(Invalid("outcome_mismatch"));
    }
    let temp_leaks = states
        .into_iter()
        .filter_map(|(path, state)| {
            matches!(
                state,
                State::Created | State::Written(..) | State::Synced(..)
            )
            .then_some(path)
        })
        .collect();
    Ok(Interpretation {
        outcome,
        temp_leaks,
    })
}
