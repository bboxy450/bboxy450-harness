// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Offline, read-only adapters into the current harness protocol.
pub mod claude;

pub async fn cli(args: impl IntoIterator<Item = String>) -> i32 {
    use futures_util::StreamExt;
    use std::io::Write;
    let mut args = args.into_iter();
    if args.next().as_deref() != Some("claude") {
        eprintln!("usage: bboxy450-harness import claude SESSION.jsonl [-o OUTPUT.jsonl]");
        return 2;
    }
    let Some(source) = args.next() else {
        return 2;
    };
    let mut output = None;
    while let Some(arg) = args.next() {
        if arg != "-o" || output.is_some() {
            return 2;
        }
        output = args.next().map(std::path::PathBuf::from);
        if output.is_none() {
            return 2;
        }
    }
    let source = match std::fs::File::open(source) {
        Ok(file) => std::io::BufReader::new(file),
        Err(_) => {
            eprintln!("import: source unavailable");
            return 1;
        }
    };
    // Output sidecars use the same session/blobs layout as gui --replay.
    let options = claude::Options {
        blob_root: output.as_ref().map(|p| {
            p.parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf()
        }),
        ..Default::default()
    };
    let mut writer: Box<dyn Write> = if let Some(path) = output {
        use std::os::unix::fs::OpenOptionsExt;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(file) => Box::new(std::io::BufWriter::new(file)),
            Err(_) => {
                eprintln!("import: output unavailable or already exists");
                return 1;
            }
        }
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };
    let mut stream = claude::convert_with(source, options);
    while let Some(next) = stream.next().await {
        let envelope = match next {
            Ok(envelope) => envelope,
            Err(error) => {
                eprintln!("import: {error}");
                return 1;
            }
        };
        let mut value = match serde_json::to_value(envelope) {
            Ok(v) => v,
            Err(_) => return 1,
        };
        value.sort_all_objects();
        if serde_json::to_writer(&mut writer, &value).is_err() || writer.write_all(b"\n").is_err() {
            eprintln!("import: output write failed");
            return 1;
        }
    }
    if writer.flush().is_err() {
        return 1;
    }
    0
}
