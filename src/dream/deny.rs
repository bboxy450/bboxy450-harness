// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! One secret boundary for synthesis input, memory records and rendered sinks.
use crate::redact::Set;
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

fn patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            r"sk-[A-Za-z0-9_-]{20,}",
            r"sk-ant-",
            r"gh[pousr]_[A-Za-z0-9]{20,}",
            r"github_pat_",
            r"AKIA[0-9A-Z]{16}",
            r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
            r"xox[baprs]-",
            r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.",
            r"(?i)(api[_-]?key|token|secret|password)\s*[:=]\s*\S{8,}",
            r"https?://[^/\s]*:[^@\s]*@",
        ]
        .into_iter()
        .map(|p| Regex::new(p).expect("fixed dream deny pattern"))
        .collect()
    })
}

/// Diagnostics identify a rule, never include the matching bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Denied {
    pub rule: usize,
}
impl std::fmt::Display for Denied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dream secret check {}", self.rule)
    }
}
impl std::error::Error for Denied {}

pub fn text(text: &str) -> Result<(), Denied> {
    patterns()
        .iter()
        .position(|p| p.is_match(text))
        .map_or(Ok(()), |rule| Err(Denied { rule: rule + 1 }))
}
pub fn value(value: &Value, set: &Set) -> Result<(), Denied> {
    if set.value(value.clone()) != *value {
        return Err(Denied { rule: 0 });
    }
    fn visit(value: &Value) -> Result<(), Denied> {
        match value {
            Value::String(s) => text(s),
            Value::Array(values) => values.iter().try_for_each(visit),
            Value::Object(values) => values.iter().try_for_each(|(key, value)| {
                text(key)?;
                visit(value)
            }),
            _ => Ok(()),
        }
    }
    visit(value)
}

/// Input is quoted data. Matching text is removed before it reaches a model.
pub fn model_input(value: Value, set: &Set) -> Value {
    fn string(mut s: String) -> String {
        for pattern in patterns() {
            s = pattern.replace_all(&s, "<redacted>").into_owned();
        }
        s
    }
    fn scrub(value: Value) -> Value {
        match value {
            Value::String(s) => Value::String(string(s)),
            Value::Array(values) => Value::Array(values.into_iter().map(scrub).collect()),
            Value::Object(values) => Value::Object(
                values
                    .into_iter()
                    .map(|(k, v)| (string(k), scrub(v)))
                    .collect(),
            ),
            other => other,
        }
    }
    scrub(set.value(value))
}

/// These targets only accept proposals, regardless of a memory's own kind.
pub fn instruction_sink(path: &str) -> bool {
    let path = path.replace('\\', "/").to_lowercase();
    let parts: Vec<_> = path
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    parts.last().is_some_and(|name| {
        matches!(
            *name,
            "claude.md" | "agents.md" | "agents.override.md" | "gemini.md"
        )
    }) || parts.windows(2).any(|p| {
        matches!(
            (p[0], p[1]),
            (".claude", "rules") | (".grok", "rules") | (".agents", "skills")
        )
    })
}

/// Escape data which could otherwise terminate a harness's quotation wrapper.
pub fn neutralize(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace("MEMORY_SUMMARY", "MEMORY&#95;SUMMARY")
}
