// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{
    import::claude::{self, MAX_LINE_BYTES, MAX_PENDING_BYTES, MAX_PENDING_TOOLS, Options},
    protocol::*,
    redact::Set,
};
use futures_util::StreamExt;
use std::io::{BufRead, Read, Write};
fn secrets() -> Vec<(&'static str, String)> {
    vec![
        (
            "{{OPENAI}}",
            ["sk", "SYNTHETIC_OPENAI_CREDENTIAL_12345678"].join("-"),
        ),
        (
            "{{ANTHROPIC}}",
            ["sk", "ant", "SYNTHETIC_ANTHROPIC_CREDENTIAL_12345678"].join("-"),
        ),
        (
            "{{GITHUB}}",
            ["ghp", "SYNTHETICGITHUBCREDENTIAL123456789"].join("_"),
        ),
        ("{{AWS}}", ["AKIA", &"S".repeat(16)].concat()),
        (
            "{{SLACK}}",
            ["xoxb", "SYNTHETIC", "CREDENTIAL12345678"].join("-"),
        ),
        (
            "{{GOOGLE}}",
            ["AI", "zaSYNTHETICGOOGLECREDENTIAL123456789012345"].concat(),
        ),
        ("{{CONFIG}}", "synthetic-configured-credential".into()),
        ("{{CAPABILITY}}", "ab".repeat(32)),
    ]
}
fn source() -> Vec<u8> {
    let mut text = include_str!("fixtures/import/claude-synthetic.jsonl").to_owned();
    for (placeholder, value) in secrets() {
        text = text.replace(placeholder, &value);
    }
    text.into_bytes()
}
fn options() -> Options {
    let set = Set::default();
    for (name, value) in secrets()
        .into_iter()
        .filter(|(name, _)| matches!(*name, "{{CONFIG}}" | "{{CAPABILITY}}"))
    {
        let _ = name;
        set.register(&value);
    }
    Options {
        redactor: set,
        supplied_redactor_only: true,
        ..Default::default()
    }
}
fn canonical(events: &[Envelope]) -> String {
    events
        .iter()
        .map(|e| {
            let mut value = serde_json::to_value(e).unwrap();
            value.sort_all_objects();
            serde_json::to_string(&value).unwrap() + "\n"
        })
        .collect()
}
async fn imported(bytes: Vec<u8>) -> Vec<Envelope> {
    let mut stream = claude::convert_with(std::io::Cursor::new(bytes), options());
    let mut events = vec![];
    while let Some(next) = stream.next().await {
        events.push(next.unwrap());
    }
    laws::check_transcript(&events).unwrap();
    events
}
#[tokio::test]
async fn synthetic_golden_is_canonical_stable_and_covers_every_mapping() {
    let events = imported(source()).await;
    let jsonl = canonical(&events);
    assert_eq!(jsonl, canonical(&imported(source()).await));
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/import/claude-synthetic.protocol.jsonl");
    if std::env::var_os("UPDATE_IMPORT_GOLDEN").is_some() {
        std::fs::write(&path, &jsonl).unwrap();
    }
    assert_eq!(jsonl, std::fs::read_to_string(path).unwrap());
    for kind in [
        ItemKind::UserMessage,
        ItemKind::AgentMessage,
        ItemKind::Reasoning,
        ItemKind::ToolCall,
        ItemKind::Terminal,
        ItemKind::FileChange,
        ItemKind::TurnDiff,
        ItemKind::Plan,
        ItemKind::Image,
        ItemKind::Subagent,
        ItemKind::Compaction,
        ItemKind::Usage,
        ItemKind::Notice,
    ] {
        assert!(
            events
                .iter()
                .any(|e| matches!(&e.event,Event::ItemCompleted {body,..} if body.kind()==kind)),
            "mapping kind missing: {kind:?}"
        );
    }
    assert_eq!(events[0].ts_ms, 1_790_856_000_123);
    assert_eq!(events.last().unwrap().ts_ms, 1_790_856_018_123);
    assert!(events.iter().any(|e| matches!(&e.event,Event::ItemStarted {head:Some(ItemHead::Terminal {command,..}),..} if command.contains("synthetic terminal")) && e.ts_ms==1_790_856_001_123));
    assert!(events.iter().any(|e| matches!(&e.event,Event::ItemCompleted {body:Item::Terminal {command,..},..} if command.contains("synthetic terminal")) && e.ts_ms==1_790_856_002_123));
    assert!(events.iter().any(|e| matches!(
        e.event,
        Event::ItemCompleted {
            status: ItemStatus::Failed { .. },
            ..
        }
    )));
    assert!(
        events
            .iter()
            .all(|e| serde_json::to_vec(e).unwrap().len() <= MAX_ENVELOPE_BYTES)
    );
}
#[tokio::test]
async fn planted_shapes_and_registered_values_never_reach_envelopes_or_blobs() {
    let overlap = Set::default();
    overlap.register("synthetic");
    overlap.register("synthetic-long-credential");
    assert_eq!(
        overlap.scrub_import("synthetic-long-credential"),
        "<redacted>"
    );
    let dir = tempfile::tempdir().unwrap();
    let mut opts = options();
    opts.blob_root = Some(dir.path().into());
    let mut stream = claude::convert_with(std::io::Cursor::new(source()), opts);
    let mut events = vec![];
    while let Some(next) = stream.next().await {
        events.push(next.unwrap());
    }
    laws::check_transcript(&events).unwrap();
    let text = canonical(&events);
    for (_, secret) in secrets() {
        assert!(
            !text.contains(&secret),
            "ImportSecretShapesRedacted: output contains a planted value"
        );
    }
    let mut pending = vec![dir.path().to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = std::fs::read(path).unwrap();
                for (_, secret) in secrets() {
                    assert!(
                        !bytes.windows(secret.len()).any(|v| v == secret.as_bytes()),
                        "ImportSecretShapesRedacted: blob contains a planted value"
                    );
                }
            }
        }
    }
}
#[tokio::test]
async fn unknown_malformed_and_truncated_input_never_panics_and_finishes_lawfully() {
    let mut stream = claude::convert_with(std::io::Cursor::new(source()), options());
    let mut events = vec![];
    while let Some(next) = stream.next().await {
        assert!(
            next.is_ok(),
            "ImportUnknownNeverPanics: conversion worker failed"
        );
        events.push(next.unwrap());
    }
    let stats = stream.metrics();
    assert!(stats.unknown >= 2 && stats.malformed == 2 && stats.truncated == 1);
    laws::check_transcript(&events).unwrap();
    for fragment in [
        "Unrecognised Claude",
        "Malformed Claude JSON",
        "Truncated final Claude line",
        "Duplicate Claude tool",
        "Orphan Claude tool",
    ] {
        assert!(events.iter().any(|e|matches!(&e.event,Event::ItemCompleted {body:Item::Notice {text,..},..} if text.contains(fragment))));
    }
    for input in [
        vec![],
        b"not json".to_vec(),
        b"null\n".to_vec(),
        b"[]\n".to_vec(),
        b"42\n".to_vec(),
        b"\"synthetic string\"\n".to_vec(),
        vec![b'x'; MAX_LINE_BYTES + 100],
    ] {
        laws::check_transcript(&imported(input).await).unwrap();
    }
}
struct Observed<R> {
    inner: R,
    consumed: std::sync::Arc<std::sync::atomic::AtomicU64>,
}
impl<R: Read> Read for Observed<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.consumed
            .fetch_add(n as u64, std::sync::atomic::Ordering::SeqCst);
        Ok(n)
    }
}
impl<R: BufRead> BufRead for Observed<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.inner.consume(n);
        self.consumed
            .fetch_add(n as u64, std::sync::atomic::Ordering::SeqCst);
    }
}
#[tokio::test]
async fn hundred_megabyte_file_is_incremental_with_bounded_live_state() {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic-100mb.jsonl");
    let line = serde_json::json!({"type":"synthetic-future-record","padding":"p".repeat(8192)})
        .to_string()
        + "\n";
    let count = 100_000_000usize.div_ceil(line.len());
    {
        let mut file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
        for _ in 0..count {
            file.write_all(line.as_bytes()).unwrap();
        }
        file.flush().unwrap();
    }
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(size >= 100_000_000);
    let consumed = Arc::new(AtomicU64::new(0));
    let reader = Observed {
        inner: std::io::BufReader::new(std::fs::File::open(path).unwrap()),
        consumed: consumed.clone(),
    };
    let mut stream = claude::convert_with(reader, options());
    let first = stream.next().await.unwrap().unwrap();
    let first_consumed = consumed.load(Ordering::SeqCst);
    assert!(
        consumed.load(Ordering::SeqCst) <= (line.len() * claude::PREPARE_BATCH) as u64,
        "ImportDoesNotReadAheadWholeFile"
    );
    let mut events = vec![first];
    while let Some(next) = stream.next().await {
        events.push(next.unwrap());
    }
    let stats = stream.metrics();
    assert_eq!(stats.input_bytes, size);
    assert_eq!(stats.lines, count as u64);
    assert!(
        stats.peak_delivery_batch <= claude::PREPARE_BATCH
            && stats.peak_line_bytes <= line.len()
            && stats.peak_queued_envelopes <= 3
            && stats.peak_pending_tools == 0
            && stats.peak_pending_bytes == 0,
        "ImportBoundedState"
    );
    assert_eq!(consumed.load(Ordering::SeqCst), size);
    laws::check_transcript(&events).unwrap();
    println!(
        "synthetic_bytes={size} peak_line={} peak_queue={} pending_bytes={}",
        stats.peak_line_bytes, stats.peak_queued_envelopes, stats.peak_pending_bytes
    );
    let record = serde_json::json!({"synthetic_only":true,"input_bytes":size,"lines":stats.lines,"observed_first_consumed_bytes":first_consumed,"peak_line_bytes":stats.peak_line_bytes,"peak_pending_tools":stats.peak_pending_tools,"peak_pending_bytes":stats.peak_pending_bytes,"peak_queued_envelopes":stats.peak_queued_envelopes,"peak_delivery_batch":stats.peak_delivery_batch,"line_limit":MAX_LINE_BYTES,"pending_tool_limit":MAX_PENDING_TOOLS,"pending_byte_limit":MAX_PENDING_BYTES,"delivery_batch_limit":claude::PREPARE_BATCH,"wall_clock_gate":false});
    std::fs::write(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".bboxy/runs/import-claude-budget.json"),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
}
#[tokio::test]
async fn missing_results_duplicate_provider_ids_and_pending_limits_preserve_pairing() {
    let mut input = String::new();
    for i in 0..1000 {
        input.push_str(&serde_json::json!({"type":"assistant","sessionId":"synthetic-pending","message":{"content":[{"type":"tool_use","id":format!("c{i}"),"name":"Bash","input":{"command":"printf synthetic"}}]}}).to_string());
        input.push('\n');
    }
    let mut stream = claude::convert_with(std::io::Cursor::new(input.into_bytes()), options());
    let mut events = vec![];
    while let Some(next) = stream.next().await {
        events.push(next.unwrap());
    }
    let stats = stream.metrics();
    assert_eq!(stats.peak_pending_tools, MAX_PENDING_TOOLS);
    assert!(stats.peak_pending_bytes <= MAX_PENDING_BYTES);
    laws::check_transcript(&events).unwrap();
}

#[tokio::test]
async fn image_results_interrupts_and_timestamp_offsets_are_preserved() {
    let records = [
        serde_json::json!({"type":"user","timestamp":"2026-10-01T14:00:00.123+02:00","message":{"content":"synthetic image request"}}),
        serde_json::json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"image","name":"Bash","input":{"command":"synthetic-image-command"}}]}}),
        serde_json::json!({"type":"user","toolUseResult":{"isImage":true,"interrupted":true},"message":{"content":[{"type":"tool_result","tool_use_id":"image","content":"AAECAw=="}]}}),
        serde_json::json!({"type":"assistant","timestamp":"invalid-synthetic-date","message":{"content":"done","stop_reason":"end_turn"}}),
    ];
    let input = records
        .iter()
        .map(|r| r.to_string() + "\n")
        .collect::<String>()
        .into_bytes();
    let events = imported(input).await;
    assert!(events.iter().all(|e| e.ts_ms == 1_790_856_000_123));
    assert!(events.iter().any(
        |e| matches!(&e.event,Event::ItemCompleted {body:Item::Image {blob,..},..} if blob.len==4)
    ));
    assert!(events.iter().any(|e| matches!(
        &e.event,
        Event::ItemCompleted {
            status: ItemStatus::Interrupted,
            body: Item::Terminal { exit: None, .. },
            ..
        }
    )));
    assert!(!canonical(&events).contains("AAECAw=="));
}

#[tokio::test]
async fn mixed_user_result_and_text_pairs_before_starting_the_new_turn() {
    let records = [
        serde_json::json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"mixed","name":"Read","input":{"file_path":"synthetic.txt"}}]}}),
        serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"mixed","content":"synthetic paired result"},{"type":"text","text":"synthetic next request"}]}}),
    ];
    let events = imported(
        records
            .iter()
            .map(|r| r.to_string() + "\n")
            .collect::<String>()
            .into_bytes(),
    )
    .await;
    let calls = events
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::ToolCall { output, .. },
                status: ItemStatus::Completed,
                ..
            } => Some(output),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 1);
    assert!(matches!(&calls[0][0],OutputBlock::Text {text} if text=="synthetic paired result"));
    assert!(!canonical(&events).contains("Orphan Claude tool result"));
}
#[test]
fn cli_reads_only_private_files_resolves_command_keys_and_writes_replayable_jsonl() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let source = root.join("synthetic.jsonl");
    std::fs::write(&source, self::source()).unwrap();
    let cap = root.join("capability");
    std::fs::write(&cap, format!("{}\n", "ab".repeat(32))).unwrap();
    std::fs::set_permissions(&cap, std::fs::Permissions::from_mode(0o600)).unwrap();
    let profiles = root.join(".bboxy450-harness/profiles");
    std::fs::create_dir_all(&profiles).unwrap();
    std::fs::write(profiles.join("synthetic.md"),"---\nname: synthetic\nmodel: synthetic\nendpoint: https://example.invalid/v1\napi_key: '!printf synthetic-configured-credential'\n---\n").unwrap();
    let output = root.join("converted.jsonl");
    let command = |out: bool| {
        let mut c = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
        c.env_clear()
            .env("HOME", root)
            .env("PATH", "/usr/bin:/bin")
            .env("BBOXY_CAPABILITY_FILE", &cap)
            .args(["import", "claude"])
            .arg(&source);
        if out {
            c.arg("-o").arg(&output);
        }
        c
    };
    let result = command(true).output().unwrap();
    assert!(result.status.success(), "private import CLI failed");
    assert!(result.stdout.is_empty());
    let bytes = std::fs::read(&output).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    for (_, secret) in secrets() {
        assert!(!text.contains(&secret), "ImportConfiguredSecretsRedacted");
    }
    let events = text
        .lines()
        .map(|l| serde_json::from_str::<Envelope>(l).unwrap())
        .collect::<Vec<_>>();
    laws::check_transcript(&events).unwrap();
    let stdout = command(false).output().unwrap();
    assert!(stdout.status.success());
    assert_eq!(stdout.stdout, bytes);
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"))
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin")
        .args(["check-transcript"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(status.status.success());
    let blobs = root.join(events[0].session.to_string()).join("blobs");
    assert!(std::fs::read_dir(blobs).unwrap().count() >= 2);
    assert_eq!(std::fs::read(&source).unwrap(), self::source());
}

#[tokio::test]
async fn paused_consumers_release_preparation_workers_and_drop_their_readers() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Owned {
        inner: std::io::Cursor<Vec<u8>>,
        dropped: Arc<AtomicUsize>,
    }
    impl Read for Owned {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(b)
        }
    }
    impl BufRead for Owned {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            self.inner.fill_buf()
        }
        fn consume(&mut self, n: usize) {
            self.inner.consume(n);
        }
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            self.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut streams = vec![];
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        for _ in 0..6 {
            let reader = Owned {
                inner: std::io::Cursor::new(source()),
                dropped: dropped.clone(),
            };
            let mut stream = claude::convert_with(reader, options());
            assert!(stream.next().await.unwrap().is_ok());
            streams.push(stream);
        }
        assert_eq!(bboxy450_harness::runtime::blocking(|| 7).await, 7);
    })
    .await
    .expect("ImportsDoNotHoldPreparationSlotsWhileBackpressured");
    drop(streams);
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        while dropped.load(Ordering::SeqCst) != 6 {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("DroppingImportReleasesOwnedReader");
}

#[tokio::test]
async fn large_text_and_escaped_tool_output_remain_readable_inside_envelope_bounds() {
    let text = "hello\n".repeat(12000);
    let output = "\u{1}".repeat(70000);
    let input = [
        serde_json::json!({"type":"user","message":{"content":"synthetic"}}),
        serde_json::json!({"type":"assistant","message":{"content":[{"type":"text","text":text},{"type":"tool_use","id":"x","name":"Bash","input":{"command":"printf synthetic"}}]}}),
        serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"x","content":output}]}}),
    ];
    let bytes = input
        .iter()
        .map(|v| v.to_string() + "\n")
        .collect::<String>()
        .into_bytes();
    let events = imported(bytes).await;
    let actual = events
        .iter()
        .filter_map(|e| match &e.event {
            Event::ItemCompleted {
                body: Item::AgentMessage { markdown },
                ..
            } => Some(markdown.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert_eq!(actual, text);
    let mut visible = String::new();
    for e in &events {
        match &e.event {
            Event::ItemCompleted {
                body: Item::ToolCall { output, .. },
                ..
            } => {
                for b in output {
                    if let OutputBlock::Text { text } = b {
                        visible.push_str(text);
                    }
                }
            }
            Event::ItemCompleted {
                body: Item::Notice { text, .. },
                ..
            } => {
                if let Some(part) = text.strip_prefix("Imported tool output continuation:\n") {
                    visible.push_str(part);
                }
            }
            _ => {}
        }
    }
    // Continuations complete before their owning tool card; all bytes are kept.
    assert_eq!(visible.len(), output.len());
    assert!(visible.chars().all(|c| c == '\u{1}'));
    assert!(
        events
            .iter()
            .all(|e| serde_json::to_vec(e).unwrap().len() <= MAX_ENVELOPE_BYTES)
    );
}
