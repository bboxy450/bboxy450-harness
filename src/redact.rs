// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use std::{
    fmt,
    sync::{Arc, RwLock},
};
use zeroize::Zeroizing;

pub struct Secret(Zeroizing<String>);
impl Secret {
    pub fn new(value: String, set: &Set) -> Self {
        set.register(&value);
        Self(Zeroizing::new(value))
    }
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}
#[derive(Clone, Default)]
pub struct Set(Arc<RwLock<Vec<Zeroizing<String>>>>);
impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Set(<redacted>)")
    }
}
impl Set {
    pub fn register(&self, value: &str) {
        if !value.is_empty() {
            self.0.write().unwrap().push(Zeroizing::new(value.into()));
        }
    }
    pub fn scrub(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for s in self.0.read().unwrap().iter() {
            out = out.replace(s.as_str(), "<redacted>");
        }
        out
    }
    pub fn value(&self, mut value: serde_json::Value) -> serde_json::Value {
        fn walk(set: &Set, v: &mut serde_json::Value) {
            match v {
                serde_json::Value::String(s) => *s = set.scrub(s),
                serde_json::Value::Array(v) => v.iter_mut().for_each(|v| walk(set, v)),
                serde_json::Value::Object(o) => {
                    let old = std::mem::take(o);
                    for (k, mut v) in old {
                        walk(set, &mut v);
                        o.insert(set.scrub(&k), v);
                    }
                }
                _ => {}
            }
        }
        walk(self, &mut value);
        value
    }
    pub fn sanitize<T: serde::Serialize + serde::de::DeserializeOwned + Clone>(
        &self,
        v: &T,
    ) -> Result<T, serde_json::Error> {
        if self.0.read().unwrap().is_empty() {
            return Ok(v.clone());
        }
        serde_json::from_value(self.value(serde_json::to_value(v)?))
    }
}
impl Set {
    pub fn bytes(&self, bytes: &[u8]) -> Vec<u8> {
        let mut out = bytes.to_vec();
        for secret in self.0.read().unwrap().iter() {
            let needle = secret.as_bytes();
            let mut i = 0;
            while i + needle.len() <= out.len() {
                if &out[i..i + needle.len()] == needle {
                    out.splice(i..i + needle.len(), b"<redacted>".iter().copied());
                    i += 10;
                } else {
                    i += 1;
                }
            }
        }
        out
    }
}
/// Retains a raw tail in memory so a key split between pipe/SSE chunks is scrubbed.
pub struct StreamScrubber {
    set: Set,
    pending: Vec<u8>,
}
impl StreamScrubber {
    pub fn new(set: Set) -> Self {
        Self {
            set,
            pending: vec![],
        }
    }
    pub fn push(&mut self, bytes: &[u8], finish: bool) -> Vec<u8> {
        self.pending.extend_from_slice(bytes);
        let secrets = self.set.0.read().unwrap();
        let max = secrets.iter().map(|s| s.len()).max().unwrap_or(1);
        let mut cut = if finish {
            self.pending.len()
        } else {
            self.pending.len().saturating_sub(max)
        };
        for secret in secrets.iter() {
            let needle = secret.as_bytes();
            for start in 0..cut {
                if start + needle.len() > cut
                    && self.pending.get(start..start + needle.len()) == Some(needle)
                {
                    cut = start;
                    break;
                }
            }
        }
        drop(secrets);
        let out = self.set.bytes(&self.pending[..cut]);
        self.pending.drain(..cut);
        out
    }
}

/// Ordered live output retains a single lookbehind across stdout/stderr reads.
/// A match spanning streams is attributed to the stream where it began.
pub(crate) struct OrderedScrubber {
    set: Set,
    pending: Vec<(crate::protocol::Stream, u8)>,
}
impl OrderedScrubber {
    pub fn new(set: Set) -> Self {
        Self {
            set,
            pending: vec![],
        }
    }
    pub fn push(
        &mut self,
        stream: crate::protocol::Stream,
        bytes: &[u8],
        finish: bool,
    ) -> Vec<(crate::protocol::Stream, Vec<u8>)> {
        self.pending.extend(bytes.iter().map(|b| (stream, *b)));
        let secrets = self.set.0.read().unwrap();
        for key in secrets.iter() {
            let needle = key.as_bytes();
            let mut i = 0;
            while i + needle.len() <= self.pending.len() {
                if self.pending[i..i + needle.len()]
                    .iter()
                    .map(|(_, b)| *b)
                    .eq(needle.iter().copied())
                {
                    let tag = self.pending[i].0;
                    self.pending
                        .splice(i..i + needle.len(), b"<redacted>".iter().map(|b| (tag, *b)));
                    i += 10;
                } else {
                    i += 1;
                }
            }
        }
        let retain = if finish {
            0
        } else {
            secrets
                .iter()
                .map(|k| k.len().saturating_sub(1))
                .max()
                .unwrap_or(0)
        };
        let n = self.pending.len().saturating_sub(retain);
        let mut out: Vec<(crate::protocol::Stream, Vec<u8>)> = vec![];
        for (tag, b) in self.pending.drain(..n) {
            if let Some((last, bytes)) = out.last_mut()
                && *last == tag
            {
                bytes.push(b);
                continue;
            }
            out.push((tag, vec![b]));
        }
        out
    }
}
#[cfg(test)]
mod ordered_tests {
    use super::*;
    use crate::protocol::Stream::{Stderr, Stdout};
    #[test]
    fn held_tails_keep_arrival_order_and_cross_stream_secrets_are_scrubbed() {
        let set = Set::default();
        set.register("a-long-unrelated-secret");
        let mut s = OrderedScrubber::new(set);
        let mut out = s.push(Stdout, b"A", false);
        out.extend(s.push(Stderr, b"B", false));
        out.extend(s.push(Stdout, b"C", true));
        assert_eq!(
            out,
            vec![
                (Stdout, b"A".to_vec()),
                (Stderr, b"B".to_vec()),
                (Stdout, b"C".to_vec())
            ]
        );
        let set = Set::default();
        set.register("ABCD");
        let mut s = OrderedScrubber::new(set);
        let mut out = s.push(Stdout, b"AB", false);
        out.extend(s.push(Stderr, b"CD", true));
        assert_eq!(
            out.into_iter().flat_map(|(_, b)| b).collect::<Vec<_>>(),
            b"<redacted>"
        );
    }
}
