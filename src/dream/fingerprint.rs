// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Stable D0 cluster keys. Only digests, never source text, enter cache names.
use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

pub fn digest(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

/// The same alternatives and priority as the original combined mask expression.
/// Independent compilation/search setup lets C1 charge cold startup in four units.
pub const NORMALIZE_MASKS: usize = 4;
const MASKS: [(&str, &str); NORMALIZE_MASKS] = [
    (
        r#"(?x) "(?:\\.|[^"\\])*" | '(?:\\.|[^'\\])*' | `[^`]*` "#,
        "<quote>",
    ),
    (r#"(?x) (?: [a-z]:[\\/] | / ) [^\s"'`<>]+ "#, "<path>"),
    (r"\b[0-9a-f]{7,}\b", "<hex>"),
    (r"[0-9]+", "<n>"),
];
fn mask(index: usize) -> &'static Regex {
    static MASKS_COMPILED: [OnceLock<Regex>; NORMALIZE_MASKS] =
        [const { OnceLock::new() }; NORMALIZE_MASKS];
    MASKS_COMPILED[index]
        .get_or_init(|| Regex::new(MASKS[index].0).expect("fixed fingerprint pattern"))
}
/// Includes first-search setup, so cold regex work does not hide in a record unit.
pub fn warm_normalize_mask(index: usize) {
    let re = mask(index);
    for sample in [
        r#""/tmp/path 123 abcdef123" 'quote' `tick`"#,
        r"c:\private\file /tmp/name abcdef0123 12345",
        "éabcdef123 é /路徑/42 \n\t text",
    ] {
        for m in re.find_iter(sample) {
            std::hint::black_box((m.start(), m.end()));
        }
    }
}
/// Lowercase; mask quotes, absolute paths, long hexadecimal runs and digits.
pub fn normalize(text: &str) -> String {
    let lower = text.to_lowercase();
    let patterns: [&Regex; NORMALIZE_MASKS] = std::array::from_fn(mask);
    let mut next = patterns.map(|p| p.find(&lower));
    let mut masked = String::with_capacity(lower.len());
    let mut cursor = 0;
    loop {
        let choice = next
            .iter()
            .enumerate()
            .filter_map(|(i, m)| m.map(|m| (i, m)))
            .min_by_key(|(i, m)| (m.start(), *i));
        let Some((index, found)) = choice else {
            masked.push_str(&lower[cursor..]);
            break;
        };
        masked.push_str(&lower[cursor..found.start()]);
        masked.push_str(MASKS[index].1);
        cursor = found.end();
        // Re-search only candidates consumed by this match. This preserves original
        // input boundaries and overlaps; it never performs sequential replacements.
        for (i, candidate) in next.iter_mut().enumerate() {
            if candidate.is_some_and(|m| m.start() < cursor) {
                *candidate = patterns[i].find_at(&lower, cursor);
            }
        }
    }
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

#[cfg(test)]
mod equivalence {
    use super::*;
    use proptest::prelude::*;
    fn original(text: &str) -> String {
        static OLD: OnceLock<Regex> = OnceLock::new();
        let old = OLD.get_or_init(|| {
            Regex::new(
                r#"(?x)
        (?P<quote> "(?:\\.|[^"\\])*" | '(?:\\.|[^'\\])*' | `[^`]*` )
        | (?P<path> (?: [a-z]:[\\/] | / ) [^\s"'`<>]+ )
        | (?P<hex> \b[0-9a-f]{7,}\b )
        | (?P<number> [0-9]+ )
    "#,
            )
            .unwrap()
        });
        old.replace_all(&text.to_lowercase(), |c: &regex::Captures<'_>| {
            if c.name("quote").is_some() {
                "<quote>"
            } else if c.name("path").is_some() {
                "<path>"
            } else if c.name("hex").is_some() {
                "<hex>"
            } else {
                "<n>"
            }
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    }
    #[test]
    fn old_priority_overlap_escaping_and_unicode_vectors_are_identical() {
        for text in [
            "\"quoted /tmp/123 abcdef123\" 123abcdef",
            "c:\\private\\x123 /tmp/name abcdef123",
            "'unclosed /path 123 `tick 456`",
            "éabcdef123 123abcdefé 0000000 /路徑/123",
            "quote \\\" escaped\n digits 42",
            "\"\\\"\\\n 123 /path",
            "`\"123\" /path` abcdef012345",
            "aa123abc 1234567 123456 / 42",
            "line\n \t\r '/42' abcdef0123",
        ] {
            assert_eq!(normalize(text), original(text), "{text:?}");
        }
    }
    proptest! {
    #[test]
    fn split_masks_equal_original_for_mixed_inputs(chars in prop::collection::vec(prop::sample::select("abcd0123456789/\\: \t\n\r\"'`<>é漢".chars().collect::<Vec<_>>()),0..256)){
    let text:String=chars.into_iter().collect();prop_assert_eq!(normalize(&text),original(&text));
    }
    }
}
