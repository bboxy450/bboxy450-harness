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
