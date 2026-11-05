// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Deterministic, derived dream observations; never a second source of truth.
pub mod audit;
pub mod d0;
pub mod deny;
pub mod entry;
pub mod fingerprint;
pub mod flush;
pub mod graph;
pub mod join;
pub mod lower;
pub mod projection;
pub mod prompt;
pub mod recall;
pub mod receipt;
pub mod resolve;
pub mod seal;
pub mod synth;
pub mod trial;
pub mod trust;

pub fn directory() -> std::io::Result<std::path::PathBuf> {
    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join(".bboxy450-harness/dream"))
        .ok_or_else(|| std::io::Error::other("HOME unavailable"))
}
pub async fn cli(args: impl Iterator<Item = String>) -> Result<i32, String> {
    let mut args = args;
    let subcommand = args.next();
    if subcommand.as_deref() == Some("synthesize") {
        return Box::pin(synth::cli(args)).await;
    }
    if let Some(subcommand @ ("audit" | "graph" | "why")) = subcommand.as_deref() {
        return audit::cli(subcommand, args).await;
    }
    if subcommand.as_deref() != Some("replay") {
        eprintln!(
            "usage: bboxy450-harness dream replay [--sessions DIR] [--json] [--rebuild] PATH…"
        );
        return Ok(2);
    }
    let mut json = false;
    let mut rebuild = false;
    let mut paths = vec![];
    let mut sessions = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--rebuild" => rebuild = true,
            "--sessions" => {
                let Some(path) = args.next() else {
                    return Ok(2);
                };
                sessions = Some(std::path::PathBuf::from(path));
            }
            "--help" | "-h" => {
                println!("dream replay [--sessions DIR] [--json] [--rebuild] PATH…");
                return Ok(0);
            }
            _ if arg.starts_with('-') => return Ok(2),
            _ => paths.push(std::path::PathBuf::from(arg)),
        }
    }
    let sessions =
        sessions.unwrap_or_else(|| directory().unwrap_or_default().with_file_name("sessions"));
    if paths.is_empty() {
        paths.push(sessions.clone());
    }
    let set = crate::redact::Set::default();
    crate::capability::register(&set).await;
    let result = match directory() {
        Ok(root) => d0::replay_with_sessions(paths, root, sessions, rebuild, set.clone()).await,
        Err(e) => Err(e),
    };
    match result {
        Ok(result) => {
            let bytes = crate::runtime::blocking(move || {
                crate::runtime::assert_off_rt_worker();
                if json {
                    serde_json::to_value(&result)
                        .and_then(|value| serde_json::to_vec(&set.value(value)))
                } else {
                    let mut text = format!(
                        "D0: {} episodes, {} updated shards, {} busy, {} skipped",
                        result.episodes.len(),
                        result.updated,
                        result.busy,
                        result.skipped
                    );
                    for notice in result.notices {
                        text.push('\n');
                        text.push_str(&notice);
                    }
                    Ok(text.into_bytes())
                }
            })
            .await
            .map_err(|_| "dream output unavailable")?;
            use tokio::io::AsyncWriteExt;
            let mut out = tokio::io::stdout();
            out.write_all(&bytes)
                .await
                .map_err(|_| "dream output unavailable")?;
            out.write_all(b"\n")
                .await
                .map_err(|_| "dream output unavailable")?;
            Ok(0)
        }
        Err(_) => {
            eprintln!("dream replay: invalid or unavailable input/cache");
            Ok(2)
        }
    }
}
