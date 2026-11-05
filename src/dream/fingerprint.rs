// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Stable D0 cluster keys. Only digests, never source text, enter cache names.
use regex::{Captures, Regex};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

pub fn digest(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

/// Lowercase; mask quotes, absolute paths, long hexadecimal runs and digits.
pub fn normalize(text: &str) -> String {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(
            r#"(?x)
        (?P<quote> "(?:\\.|[^"\\])*" | '(?:\\.|[^'\\])*' | `[^`]*` )
        | (?P<path> (?: [a-z]:[\\/] | / ) [^\s"'`<>]+ )
        | (?P<hex> \b[0-9a-f]{7,}\b )
        | (?P<number> [0-9]+ )
    "#,
        )
        .expect("fixed fingerprint pattern")
    });
    let lower = text.to_lowercase();
    let masked = pattern.replace_all(&lower, |c: &Captures<'_>| {
        if c.name("quote").is_some() {
            "<quote>"
        } else if c.name("path").is_some() {
            "<path>"
        } else if c.name("hex").is_some() {
            "<hex>"
        } else {
            "<n>"
        }
    });
    masked.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn tool_fail(name: &str, output: &str) -> String {
    digest(format!(
        "tool-fail:{name}:{}",
        normalize(output.lines().next().unwrap_or_default())
    ))
}
pub fn retry(name: &str, args: &str) -> String {
    digest(format!("retry:{name}:{}", normalize(args)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalization_masks_unstable_values_before_hashing() {
        assert_eq!(
            normalize(r#"ERROR 123 ABCDEF123 /tmp/one/file "private value" 'other' `cmd`"#),
            "error <n> <hex> <path> <quote> <quote> <quote>"
        );
        assert_eq!(normalize(r"C:\private\file:456"), "<path>");
        assert_eq!(
            tool_fail("Read", "Missing /tmp/a at 12\nignored"),
            tool_fail("Read", "MISSING /tmp/b at 99\ndifferent")
        );
        assert_ne!(tool_fail("Read", "missing"), tool_fail("Write", "missing"));
        assert_ne!(tool_fail("Read", "missing"), tool_fail("Read", "denied"));
    }
    #[test]
    fn digest_is_the_exact_sha256_string_vector() {
        assert_eq!(
            digest("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
