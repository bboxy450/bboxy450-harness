// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Content identity is independent of status, wall clocks and arrival order.
use crate::protocol::Scope;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invalid(pub &'static str);
impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for Invalid {}

/// These seven fields alone select an EntryId. Evidence remains fully pinned.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub kind: String,
    pub title: String,
    pub why: String,
    pub how_to_apply: String,
    pub evidence: Vec<Value>,
    #[serde(default)]
    pub derived_from: Vec<String>,
    #[serde(default)]
    pub revival: u64,
}

pub fn digest(bytes: impl AsRef<[u8]>) -> String {
    format!("sha256:{}", super::fingerprint::digest(bytes))
}
pub fn valid_digest(digest: &str) -> bool {
    digest.strip_prefix("sha256:").is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}
pub fn valid_commit(commit: &str) -> bool {
    matches!(commit.len(), 40 | 64) && commit.bytes().all(|c| c.is_ascii_hexdigit())
}

impl Identity {
    pub fn validate(&self) -> Result<(), Invalid> {
        if !matches!(
            self.kind.as_str(),
            "lesson"
                | "workflow"
                | "pitfall"
                | "preference"
                | "fact"
                | "brief_template"
                | "invariant_candidate"
        ) {
            return Err(Invalid("unknown memory kind"));
        }
        for (text, limit) in [
            (&self.title, 120),
            (&self.why, 1500),
            (&self.how_to_apply, 1500),
        ] {
            if text.trim().is_empty() || text.chars().count() > limit {
                return Err(Invalid("memory text bound"));
            }
        }
        if self.evidence.is_empty()
            || self.evidence.len() > 32
            || self.evidence.iter().any(|r| {
                !r.is_object()
                    || !(r.get("ref").is_some_and(Value::is_string)
                        || matches!(
                            r.get("kind").and_then(Value::as_str),
                            Some("event" | "record")
                        ))
            })
        {
            return Err(Invalid("memory requires bounded known evidence"));
        }
        if self.derived_from.iter().any(|id| !valid_digest(id)) {
            return Err(Invalid("invalid predecessor identity"));
        }
        if self.revival > 0 && self.derived_from.is_empty() {
            return Err(Invalid("revival requires a terminal predecessor"));
        }
        Ok(())
    }
    /// Unicode NFC is applied to keys and values before deterministic JSON sorting.
    pub fn canonical(&self) -> Result<Vec<u8>, Invalid> {
        self.validate()?;
        let mut value = serde_json::to_value(self).map_err(|_| Invalid("identity encoding"))?;
        normalize(&mut value)?;
        for key in ["evidence", "derived_from"] {
            let values = value[key].as_array_mut().ok_or(Invalid("identity array"))?;
            values.sort_by_cached_key(|v| serde_json::to_string(v).expect("JSON value"));
        }
        serde_json::to_vec(&value).map_err(|_| Invalid("identity encoding"))
    }
    pub fn id(&self) -> Result<String, Invalid> {
        Ok(digest(self.canonical()?))
    }
}

fn normalize(value: &mut Value) -> Result<(), Invalid> {
    match value {
        Value::String(text) => {
            *text = icu_normalizer::ComposingNormalizer::new_nfc()
                .normalize(text)
                .into_owned()
        }
        Value::Array(values) => {
            for value in values {
                normalize(value)?;
            }
        }
        Value::Object(object) => {
            let mut sorted = BTreeMap::new();
            for (key, mut value) in std::mem::take(object) {
                let key = icu_normalizer::ComposingNormalizer::new_nfc()
                    .normalize(&key)
                    .into_owned();
                normalize(&mut value)?;
                if sorted.insert(key, value).is_some() {
                    return Err(Invalid("NFC object key collision"));
                }
            }
            object.extend(sorted);
        }
        _ => {}
    }
    Ok(())
}

/// Preserve exact body bytes; projection metadata never authorizes body repair.
#[derive(Clone, Debug)]
pub struct Record {
    pub front: Value,
    pub yaml: serde_yaml::Value,
    pub body: String,
    pub bytes: Vec<u8>,
}
impl Record {
    pub fn draft(value: &Value) -> Result<Self, Invalid> {
        if let Some(text) = value.as_str() {
            return Self::parse(text.as_bytes());
        }
        let front = value
            .get("frontmatter")
            .filter(|v| v.is_object())
            .ok_or(Invalid("draft requires full frontmatter"))?;
        let body = value
            .get("body")
            .and_then(Value::as_str)
            .ok_or(Invalid("draft requires full body"))?;
        let text = format!(
            "---\n{}---\n{body}",
            serde_yaml::to_string(front).map_err(|_| Invalid("draft frontmatter encoding"))?
        );
        Self::parse(text.as_bytes())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, Invalid> {
        let text = std::str::from_utf8(bytes).map_err(|_| Invalid("record is not UTF-8"))?;
        let source = text.strip_prefix('\u{feff}').unwrap_or(text);
        let after = source
            .strip_prefix("---\n")
            .or_else(|| source.strip_prefix("---\r\n"))
            .ok_or(Invalid("missing record frontmatter"))?;
        let mut offset = 0;
        let (end, body_start) = after
            .split_inclusive('\n')
            .find_map(|line| {
                let start = offset;
                offset += line.len();
                matches!(line.trim_end_matches(['\r', '\n']), "---" | "...")
                    .then_some((start, offset))
            })
            .ok_or(Invalid("incomplete record frontmatter"))?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&after[..end])
            .map_err(|_| Invalid("invalid record frontmatter"))?;
        let front =
            serde_json::to_value(&yaml).map_err(|_| Invalid("non-JSON record frontmatter"))?;
        if !front.is_object() {
            return Err(Invalid("record frontmatter is not a mapping"));
        }
        Ok(Self {
            front,
            yaml,
            body: after[body_start..].into(),
            bytes: bytes.into(),
        })
    }
    pub fn identity(&self) -> Result<Identity, Invalid> {
        let section = |title: &str| -> Option<String> {
            let header = format!("## {title}");
            let mut active = false;
            let mut lines = Vec::new();
            for line in self.body.lines() {
                if line == header {
                    active = true;
                    continue;
                }
                if active && line.starts_with("## ") {
                    break;
                }
                if active {
                    lines.push(line);
                }
            }
            active.then(|| lines.join("\n").trim().to_owned())
        };
        let string = |key: &str| {
            self.front
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(Invalid("missing memory identity field"))
        };
        let why = self
            .front
            .get("why")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| section("Why"))
            .ok_or(Invalid("missing memory why"))?;
        let how = self
            .front
            .get("how_to_apply")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| section("How to apply"))
            .ok_or(Invalid("missing memory application"))?;
        let evidence = serde_json::from_value(
            self.front
                .get("evidence")
                .cloned()
                .ok_or(Invalid("missing memory evidence"))?,
        )
        .map_err(|_| Invalid("invalid pinned memory evidence"))?;
        let identity = Identity {
            kind: string("kind")?,
            title: string("title")?,
            why,
            how_to_apply: how,
            evidence,
            derived_from: serde_json::from_value(
                self.front.get("derived_from").cloned().unwrap_or(json!([])),
            )
            .map_err(|_| Invalid("invalid memory predecessors"))?,
            revival: self
                .front
                .get("revival")
                .map_or(Some(0), Value::as_u64)
                .ok_or(Invalid("invalid revival"))?,
        };
        identity.validate()?;
        Ok(identity)
    }
    pub fn scope(&self) -> Result<Scope, Invalid> {
        serde_json::from_value(
            self.front
                .get("scope")
                .cloned()
                .ok_or(Invalid("missing memory scope"))?,
        )
        .map_err(|_| Invalid("invalid memory scope"))
    }
    pub fn content_changed(&self) -> bool {
        self.front.get("content_sha256").and_then(Value::as_str)
            != Some(digest(&self.body).as_str())
    }
    pub fn validate_identity(&self) -> Result<Identity, Invalid> {
        if self.front.get("type").and_then(Value::as_str) != Some("memory")
            || self.body.len() > 8192
        {
            return Err(Invalid("invalid memory record type/body bound"));
        }
        let identity = self.identity()?;
        if self.front.get("entry").and_then(Value::as_str) != Some(identity.id()?.as_str()) {
            return Err(Invalid("entry identity mismatch"));
        }
        if self.content_changed() {
            return Err(Invalid("changed memory content; preserve as new proposal"));
        }
        Ok(identity)
    }
}
