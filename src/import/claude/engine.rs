// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::{ImportError, Metrics, Options};
use crate::protocol::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    io::{BufRead, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};
pub const MAX_LINE_BYTES: usize = 1024 * 1024;
pub const MAX_PENDING_TOOLS: usize = 32;
pub const MAX_PENDING_BYTES: usize = 4 * 1024 * 1024;
// Leave room for JSON escaping, two tool fields and protocol identity headers.
const TEXT_CHUNK: usize = 24 * 1024;
struct Pending {
    id: ItemId,
    call_id: String,
    source_id: String,
    name: String,
    input: String,
    terminal: Option<(ItemId, String)>,
}
impl Pending {
    fn bytes(&self) -> usize {
        self.source_id.len()
            + self.call_id.len()
            + self.name.len()
            + self.input.len()
            + self.terminal.as_ref().map_or(0, |(_, s)| s.len())
    }
}
pub(super) struct Engine<R> {
    reader: R,
    options: Options,
    startup: Vec<String>,
    queue: VecDeque<Result<Envelope, ImportError>>,
    pending: VecDeque<Pending>,
    session: Option<SessionId>,
    turn: Option<TurnId>,
    usage: Option<Usage>,
    seq: u64,
    ordinal: u64,
    ts: u64,
    frontier: Seq,
    done: bool,
    pub metrics: Metrics,
}
impl<R: BufRead> Engine<R> {
    pub fn new(reader: R, options: Options, startup: Vec<String>) -> Self {
        Self {
            reader,
            options,
            startup,
            queue: VecDeque::new(),
            pending: VecDeque::new(),
            session: None,
            turn: None,
            usage: None,
            seq: 0,
            ordinal: 0,
            ts: 0,
            frontier: Seq(0),
            done: false,
            metrics: Metrics::default(),
        }
    }
    fn stable(&self, domain: &str, value: &[u8]) -> ulid::Ulid {
        let mut hash = Sha256::new();
        hash.update(b"bboxy450-claude-import-v1\0");
        if let Some(session) = self.session {
            hash.update(session.0.to_bytes());
        }
        hash.update(domain.as_bytes());
        hash.update([0]);
        hash.update(value);
        let digest = hash.finalize();
        let mut bytes = [0; 16];
        bytes.copy_from_slice(&digest[..16]);
        ulid::Ulid::from_bytes(bytes)
    }
    fn item_id(&mut self) -> ItemId {
        self.ordinal += 1;
        ItemId(self.stable("item", &self.ordinal.to_be_bytes()))
    }
    fn emit(&mut self, event: Event) {
        let seq = if event.is_durable() {
            self.seq += 1;
            Some(Seq(self.seq))
        } else {
            None
        };
        self.queue.push_back(Ok(Envelope {
            v: PROTOCOL,
            session: self.session.expect("initialized import session"),
            epoch: Epoch(0),
            seq,
            at: Seq(self.seq),
            ts_ms: self.ts,
            cause: None,
            event,
        }));
        self.metrics.envelopes += 1;
        self.metrics.peak_queued_envelopes =
            self.metrics.peak_queued_envelopes.max(self.queue.len());
    }
    fn initialize(&mut self, record: &Value) {
        if self.session.is_some() {
            return;
        }
        let seed = record
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or("unidentified-claude-session");
        self.session = Some(SessionId(self.stable("session", seed.as_bytes())));
        self.emit(Event::SessionStarted {
            meta: SessionMeta {
                session: self.session.unwrap(),
                cwd: record
                    .get("cwd")
                    .and_then(Value::as_str)
                    .map(|s| prefix(s, 4096))
                    .unwrap_or(".")
                    .into(),
                profile: Some("Claude Code import".into()),
                fork: None,
                protocol: PROTOCOL,
                harness: env!("CARGO_PKG_VERSION").into(),
                created_ms: self.ts,
            },
        });
        for text in std::mem::take(&mut self.startup) {
            self.notice(&text);
        }
    }
    fn start(&mut self, body: &Item, parent: Option<ItemId>) -> ItemId {
        let item = self.item_id();
        self.emit(Event::ItemStarted {
            turn: self.turn,
            item,
            kind: body.kind(),
            parent,
            head: body.head(),
        });
        item
    }
    fn item(&mut self, body: Item, parent: Option<ItemId>) {
        let item = self.start(&body, parent);
        self.emit(Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body,
        });
    }
    fn notice(&mut self, text: &str) {
        self.item(
            Item::Notice {
                level: Level::Warn,
                text: text.into(),
            },
            None,
        );
    }
    fn unknown(&mut self, label: &str) {
        self.metrics.unknown += 1;
        self.notice(&format!(
            "Unrecognised Claude {label} at line {}",
            self.metrics.lines
        ));
    }
    fn begin_turn(&mut self) {
        if self.turn.is_none() {
            let turn = TurnId(self.stable("turn", &self.metrics.lines.to_be_bytes()));
            self.turn = Some(turn);
            self.usage = None;
            self.emit(Event::TurnQueued { turn });
            self.emit(Event::TurnStarted { turn });
        }
    }
    fn end_turn(&mut self, reason: StopReason) {
        if let Some(turn) = self.turn {
            while let Some(p) = self.pending.pop_front() {
                self.complete(
                    p,
                    "No tool result in imported log",
                    true,
                    &Value::Null,
                    true,
                );
            }
            let usage = self.usage.take();
            self.emit(Event::TurnEnded {
                turn,
                reason,
                usage,
            });
            self.frontier = Seq(self.seq);
            self.turn = None;
        }
    }
    fn blob(&mut self, text: &str, mime: &str) -> BlobRef {
        let bytes = text.as_bytes();
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        if let Some(root) = &self.options.blob_root {
            let dir = root.join(self.session.unwrap().to_string()).join("blobs");
            let result = (|| -> std::io::Result<()> {
                std::fs::create_dir_all(&dir)?;
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(dir.join(&sha256))
                {
                    Ok(mut file) => file.write_all(bytes),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
                    Err(error) => Err(error),
                }
            })();
            if result.is_err() {
                self.queue.push_back(Err(ImportError::Blob {
                    line: self.metrics.lines,
                }));
            }
        }
        BlobRef {
            sha256,
            len: bytes.len() as u64,
            mime: mime.into(),
        }
    }
    fn text(&mut self, text: &str, user: bool, thinking: bool) {
        if text.is_empty() {
            self.notice("Empty Claude text block");
            return;
        }
        let mut rest = text;
        while !rest.is_empty() {
            let part = prefix(rest, TEXT_CHUNK);
            let body = if user {
                Item::UserMessage {
                    blocks: vec![InputBlock::Text { text: part.into() }],
                }
            } else if thinking {
                Item::Reasoning {
                    summary: part.into(),
                    raw: None,
                    provider_opaque: None,
                }
            } else {
                Item::AgentMessage {
                    markdown: part.into(),
                }
            };
            self.item(body, None);
            rest = &rest[part.len()..];
        }
    }
    fn subagent(&mut self, value: &str, parent: Option<ItemId>) {
        self.item(
            Item::Subagent {
                session: SessionId(self.stable("subagent", value.as_bytes())),
            },
            parent,
        );
    }
    fn image(&mut self, block: &Value) {
        use base64::Engine;
        let source = &block["source"];
        let mime = prefix(
            source["media_type"]
                .as_str()
                .or_else(|| block["mime_type"].as_str())
                .unwrap_or("image/unknown"),
            256,
        )
        .to_owned();
        let bytes = source["data"]
            .as_str()
            .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok());
        if bytes.is_none() {
            self.notice("Image metadata only; source byte length unavailable");
        }
        let (sha256, len) = bytes
            .map(|b| (format!("{:x}", Sha256::digest(&b)), b.len() as u64))
            .unwrap_or_else(|| {
                (
                    format!("{:x}", Sha256::digest(b"claude-image-metadata-only")),
                    0,
                )
            });
        self.item(
            Item::Image {
                mime: mime.clone(),
                blob: BlobRef { sha256, len, mime },
            },
            None,
        );
    }
    fn tool_use(&mut self, block: &Value) {
        self.begin_turn();
        let source = block["id"].as_str().unwrap_or("missing-tool-id").to_owned();
        if let Some(index) = self.pending.iter().position(|p| p.source_id == source) {
            let p = self.pending.remove(index).unwrap();
            self.complete(
                p,
                "Duplicate tool identity in import",
                true,
                &Value::Null,
                true,
            );
            self.notice("Duplicate Claude tool identity repaired");
        }
        let name = prefix(block["name"].as_str().unwrap_or("Unknown tool"), 1024).to_owned();
        let raw = canonical(&block["input"]);
        let input = prefix(&raw, TEXT_CHUNK).to_owned();
        if input.len() != raw.len() {
            self.notice("Oversized tool input represented by an excerpt");
        }
        while self.pending.len() >= MAX_PENDING_TOOLS
            || self.pending.iter().map(Pending::bytes).sum::<usize>()
                + input.len()
                + source.len()
                + name.len()
                + TEXT_CHUNK
                > MAX_PENDING_BYTES
        {
            let Some(p) = self.pending.pop_front() else {
                break;
            };
            self.complete(
                p,
                "Import pending-tool budget exceeded",
                true,
                &Value::Null,
                true,
            );
            self.notice("Pending Claude tool budget exceeded; oldest call closed");
        }
        let body = Item::ToolCall {
            call_id: format!("import-{}", self.ordinal + 1),
            name: name.clone(),
            tool_kind: tool_kind(&name),
            input_raw: input.clone(),
            output: vec![OutputBlock::Text {
                text: "Pending imported result".into(),
            }],
            locations: vec![],
            spill: None,
        };
        let id = self.start(&body, None);
        let terminal = if name.eq_ignore_ascii_case("bash") {
            let command =
                prefix(block["input"]["command"].as_str().unwrap_or("Bash"), 8192).to_owned();
            let body = Item::Terminal {
                terminal: id.to_string(),
                command: command.clone(),
                exit: None,
                signal: None,
                output: None,
                live: Some(String::new()),
            };
            let child = self.start(&body, Some(id));
            Some((child, command))
        } else {
            None
        };
        // The durable tool head owns the stable call_id used on completion.
        let Item::ToolCall { call_id, .. } = body else {
            unreachable!("tool body constructed above")
        };
        self.pending.push_back(Pending {
            id,
            call_id,
            source_id: source,
            name,
            input,
            terminal,
        });
        self.metrics.peak_pending_tools = self.metrics.peak_pending_tools.max(self.pending.len());
        self.metrics.peak_pending_bytes = self
            .metrics
            .peak_pending_bytes
            .max(self.pending.iter().map(Pending::bytes).sum());
    }
    fn complete(&mut self, p: Pending, text: &str, error: bool, meta: &Value, interrupted: bool) {
        let status = if interrupted {
            ItemStatus::Interrupted
        } else if error {
            ItemStatus::Failed {
                reason: "Claude tool reported an error".into(),
            }
        } else {
            ItemStatus::Completed
        };
        let input: Value = serde_json::from_str(&p.input).unwrap_or(Value::Null);
        if let Some((id, command)) = &p.terminal {
            let blob = self.blob(text, "text/plain");
            let exit = meta["exitCode"]
                .as_i64()
                .or_else(|| meta["exit_code"].as_i64())
                .or_else(|| meta["returnCode"].as_i64())
                .and_then(|v| i32::try_from(v).ok())
                .or(if interrupted {
                    None
                } else if error {
                    Some(1)
                } else {
                    Some(0)
                });
            self.emit(Event::ItemSnapshot {
                item: *id,
                partial: Item::Terminal {
                    terminal: p.id.to_string(),
                    command: command.clone(),
                    exit: None,
                    signal: None,
                    output: None,
                    live: Some(suffix(text, TEXT_CHUNK).into()),
                },
            });
            self.emit(Event::ItemCompleted {
                item: *id,
                status: status.clone(),
                body: Item::Terminal {
                    terminal: p.id.to_string(),
                    command: command.clone(),
                    exit,
                    signal: None,
                    output: Some(blob),
                    live: None,
                },
            });
        }
        if !error {
            self.tool_extras(&p, &input, meta);
        }
        let spill = if text.len() > TEXT_CHUNK {
            Some(self.blob(text, "text/plain"))
        } else {
            None
        };
        if text.len() > TEXT_CHUNK {
            let mut rest = &text[prefix(text, TEXT_CHUNK).len()..];
            while !rest.is_empty() {
                let part = prefix(rest, TEXT_CHUNK);
                self.item(
                    Item::Notice {
                        level: Level::Info,
                        text: format!("Imported tool output continuation:\n{part}"),
                    },
                    Some(p.id),
                );
                rest = &rest[part.len()..];
            }
        }
        self.emit(Event::ItemCompleted {
            item: p.id,
            status,
            body: Item::ToolCall {
                call_id: p.call_id,
                name: p.name.clone(),
                tool_kind: tool_kind(&p.name),
                input_raw: p.input,
                output: vec![if error {
                    OutputBlock::Error {
                        text: prefix(text, TEXT_CHUNK).into(),
                    }
                } else {
                    OutputBlock::Text {
                        text: if text.is_empty() {
                            "(empty tool result)".into()
                        } else {
                            prefix(text, TEXT_CHUNK).into()
                        },
                    }
                }],
                locations: vec![],
                spill,
            },
        });
    }
    fn tool_extras(&mut self, p: &Pending, input: &Value, meta: &Value) {
        match p.name.to_ascii_lowercase().as_str() {
            "edit" | "write" | "multiedit" => {
                let path = prefix(
                    input["file_path"]
                        .as_str()
                        .or_else(|| input["path"].as_str())
                        .unwrap_or("unknown-file"),
                    4096,
                );
                let mut diffs = String::new();
                let edits = if p.name.eq_ignore_ascii_case("multiedit") {
                    input["edits"].as_array().cloned().unwrap_or_default()
                } else {
                    vec![input.clone()]
                };
                for edit in edits {
                    let old = edit["old_string"].as_str().unwrap_or("");
                    let new = edit["new_string"]
                        .as_str()
                        .or_else(|| edit["content"].as_str())
                        .unwrap_or("");
                    diffs.push_str(&format!(
                        "--- a/{path}\n+++ b/{path}\n@@ -1,{} +1,{} @@\n",
                        old.lines().count(),
                        new.lines().count()
                    ));
                    for line in old.lines() {
                        diffs.push('-');
                        diffs.push_str(line);
                        diffs.push('\n');
                    }
                    for line in new.lines() {
                        diffs.push('+');
                        diffs.push_str(line);
                        diffs.push('\n');
                    }
                }
                if diffs.len() > TEXT_CHUNK {
                    self.notice("Large imported diff represented by an excerpt");
                }
                let unified = prefix(&diffs, TEXT_CHUNK).to_owned();
                self.item(
                    Item::FileChange {
                        path: PathBuf::from(path),
                        change: if p.name.eq_ignore_ascii_case("write")
                            && meta["originalFile"].is_null()
                        {
                            ChangeKind::Add
                        } else {
                            ChangeKind::Modify
                        },
                        unified: unified.clone(),
                    },
                    Some(p.id),
                );
                self.item(Item::TurnDiff { unified }, Some(p.id));
            }
            "todowrite" => {
                let entries = input["todos"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|v| PlanEntry {
                        text: v["content"]
                            .as_str()
                            .or_else(|| v["text"].as_str())
                            .unwrap_or("Untitled plan entry")
                            .into(),
                        status: match v["status"].as_str() {
                            Some("pending") => PlanStatus::Pending,
                            Some("in_progress") => PlanStatus::InProgress,
                            Some("completed") => PlanStatus::Completed,
                            Some("cancelled") => PlanStatus::Cancelled,
                            _ => PlanStatus::Unknown,
                        },
                        priority: match v["priority"].as_str() {
                            Some("high") => Some(Priority::High),
                            Some("medium") => Some(Priority::Medium),
                            Some("low") => Some(Priority::Low),
                            _ => None,
                        },
                    })
                    .collect();
                self.item(Item::Plan { entries }, Some(p.id));
            }
            "task" | "agent" => {
                self.subagent(meta["agentId"].as_str().unwrap_or(&p.source_id), Some(p.id))
            }
            _ => {}
        }
    }
    fn block(&mut self, block: &Value, user: bool, meta: &Value) {
        match block["type"].as_str().unwrap_or("missing") {
            "text" => self.text(block["text"].as_str().unwrap_or(""), user, false),
            "thinking" => self.text(block["thinking"].as_str().unwrap_or(""), false, true),
            "redacted_thinking" => self.notice("Claude thinking was withheld by the source"),
            "image" => self.image(block),
            "tool_use" => self.tool_use(block),
            "tool_result" => {
                if !block["content"].is_string() && !block["content"].is_array() {
                    self.unknown("tool result content");
                }
                let source = block["tool_use_id"].as_str().unwrap_or("");
                let image_result = meta["isImage"].as_bool().unwrap_or(false);
                let text = if image_result {
                    "(image result, metadata only)".into()
                } else {
                    content_text(&block["content"])
                };
                let interrupted = meta["interrupted"].as_bool().unwrap_or(false);
                let error = block["is_error"].as_bool().unwrap_or(false) || interrupted;
                if let Some(index) = self.pending.iter().position(|p| p.source_id == source) {
                    let p = self.pending.remove(index).unwrap();
                    self.complete(p, &text, error, meta, interrupted);
                } else {
                    self.notice("Orphan Claude tool result; synthesized call identity");
                    self.tool_use(
                        &serde_json::json!({"id":source,"name":"Unknown tool","input":{}}),
                    );
                    let p = self.pending.pop_back().unwrap();
                    self.complete(p, &text, error, meta, interrupted);
                }
                if image_result && block["content"].is_string() {
                    self.image(&serde_json::json!({"source":{"media_type":"image/unknown","data":block["content"]}}));
                }
                if let Some(blocks) = block["content"].as_array() {
                    for unknown in blocks
                        .iter()
                        .filter(|b| b["type"] != "image" && b["type"] != "text")
                    {
                        self.unknown(&format!(
                            "tool result block {}",
                            prefix(unknown["type"].as_str().unwrap_or("missing"), 128)
                        ));
                    }
                    for image in blocks.iter().filter(|b| b["type"] == "image") {
                        self.image(image);
                    }
                }
            }
            kind => self.unknown(&format!("content block {}", prefix(kind, 128))),
        }
    }
    fn record(&mut self, record: Value) {
        let record = self.options.redactor.import_value(record);
        if let Some(timestamp) = record.get("timestamp") {
            if let Some(ts) = timestamp
                .as_u64()
                .or_else(|| timestamp.as_str().and_then(timestamp_ms))
            {
                self.ts = ts;
            } else {
                self.initialize(&record);
                self.notice("Invalid Claude timestamp; retained preceding instant");
            }
        }
        self.initialize(&record);
        let kind = record["type"].as_str().unwrap_or("missing");
        let compact = record["isCompactSummary"].as_bool() == Some(true)
            || kind == "summary"
            || (kind == "system" && record["subtype"] == "compact_boundary");
        if compact {
            self.end_turn(StopReason::Interrupted);
            let summary = record["summary"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| content_text(&record["message"]["content"]));
            let summary = if summary.is_empty() {
                "Claude compaction boundary (summary absent)".into()
            } else {
                summary
            };
            let blob = self.blob(&summary, "text/markdown");
            self.item(
                Item::Compaction {
                    summary: blob,
                    replaced_through: self.frontier,
                },
                None,
            );
            let mut rest = summary.as_str();
            while !rest.is_empty() {
                let part = prefix(rest, TEXT_CHUNK);
                self.item(
                    Item::Notice {
                        level: Level::Info,
                        text: format!("Imported compaction summary:\n{part}"),
                    },
                    None,
                );
                rest = &rest[part.len()..];
            }
            return;
        }
        let sidechain = record["isSidechain"].as_bool().unwrap_or(false);
        if sidechain {
            self.subagent(
                record["agentId"]
                    .as_str()
                    .or_else(|| record["sessionId"].as_str())
                    .unwrap_or("sidechain"),
                None,
            );
        }
        match kind {
            "user" | "assistant" => {
                let user = kind == "user";
                let content = &record["message"]["content"];
                let is_request = user
                    && (content.is_string()
                        || content
                            .as_array()
                            .is_some_and(|v| v.iter().any(|b| b["type"] != "tool_result")));
                if is_request && !sidechain {
                    // A mixed user record can deliver the preceding tool's
                    // result and then introduce a new request. Pair first.
                    if let Some(blocks) = content.as_array() {
                        for result in blocks.iter().filter(|b| b["type"] == "tool_result") {
                            self.block(result, user, &record["toolUseResult"]);
                        }
                    }
                    self.end_turn(StopReason::Interrupted);
                }
                self.begin_turn();
                if let Some(text) = content.as_str() {
                    self.text(text, user, false);
                } else if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        if is_request && !sidechain && block["type"] == "tool_result" {
                            continue;
                        }
                        self.block(block, user, &record["toolUseResult"]);
                    }
                } else {
                    self.unknown("message content");
                }
                if let Some(value) = record["message"].get("usage").filter(|v| v.is_object()) {
                    let usage = Usage {
                        input_tokens: value["input_tokens"].as_u64().unwrap_or(0),
                        output_tokens: value["output_tokens"].as_u64().unwrap_or(0),
                        cached_input_tokens: value["cache_read_input_tokens"].as_u64(),
                        reasoning_tokens: value["reasoning_tokens"].as_u64(),
                        cost_micros_usd: None,
                    };
                    self.usage = Some(usage.clone());
                    self.item(Item::Usage { usage }, None);
                }
                if !sidechain {
                    match record["message"]["stop_reason"].as_str() {
                        Some("end_turn" | "stop_sequence") => self.end_turn(StopReason::EndTurn),
                        Some("max_tokens") => self.end_turn(StopReason::MaxTokens),
                        _ => {}
                    }
                }
            }
            _ => self.unknown(&format!("record {}", prefix(kind, 128))),
        }
    }
    fn read_line(&mut self) -> Result<Option<(Vec<u8>, bool, bool)>, ImportError> {
        let mut line = Vec::new();
        let mut oversized = false;
        let mut any = false;
        loop {
            let buf = self.reader.fill_buf().map_err(|_| ImportError::Input {
                line: self.metrics.lines + 1,
            })?;
            if buf.is_empty() {
                return Ok(any.then_some((line, oversized, true)));
            }
            let end = buf.iter().position(|b| *b == b'\n');
            let count = end.map_or(buf.len(), |n| n + 1);
            any = true;
            let take = count.min(MAX_LINE_BYTES.saturating_sub(line.len()));
            line.extend_from_slice(&buf[..take]);
            if take < count {
                oversized = true;
            }
            self.reader.consume(count);
            self.metrics.input_bytes += count as u64;
            self.metrics.peak_line_bytes = self.metrics.peak_line_bytes.max(line.len());
            if end.is_some() {
                return Ok(Some((line, oversized, false)));
            }
        }
    }
    fn finish(&mut self) {
        self.initialize(&Value::Null);
        self.end_turn(StopReason::Interrupted);
        self.emit(Event::SessionClosed);
        self.done = true;
    }
}
impl<R: BufRead> Iterator for Engine<R> {
    type Item = Result<Envelope, ImportError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next) = self.queue.pop_front() {
                return Some(next);
            }
            if self.done {
                return None;
            }
            match self.read_line() {
                Ok(Some((line, oversized, truncated))) => {
                    self.metrics.lines += 1;
                    if oversized {
                        self.metrics.oversized += 1;
                        self.initialize(&Value::Null);
                        self.notice("Oversized Claude line skipped at import bound");
                    } else {
                        match serde_json::from_slice::<Value>(&line) {
                            Ok(value) => self.record(value),
                            Err(_) => {
                                self.metrics.malformed += 1;
                                self.initialize(&Value::Null);
                                self.notice(&format!(
                                    "Malformed Claude JSON at line {}",
                                    self.metrics.lines
                                ));
                            }
                        }
                    }
                    if truncated {
                        self.metrics.truncated += 1;
                        self.notice("Truncated final Claude line at EOF");
                    }
                }
                Ok(None) => self.finish(),
                Err(error) => {
                    self.queue.push_back(Err(error));
                    self.finish();
                }
            }
        }
    }
}
fn prefix(text: &str, limit: usize) -> &str {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}
fn suffix(text: &str, limit: usize) -> &str {
    let mut start = text.len().saturating_sub(limit);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    &text[start..]
}
fn content_text(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.into();
    }
    if let Some(blocks) = value.as_array() {
        return blocks
            .iter()
            .filter_map(|v| v["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n");
    }
    String::new()
}
fn canonical(value: &Value) -> String {
    let mut value = value.clone();
    value.sort_all_objects();
    serde_json::to_string(&value).unwrap_or_else(|_| "null".into())
}
fn tool_kind(name: &str) -> ToolKind {
    match name.to_ascii_lowercase().as_str() {
        "bash" => ToolKind::Execute,
        "edit" | "write" | "multiedit" => ToolKind::Edit,
        "read" => ToolKind::Read,
        "glob" | "grep" => ToolKind::Search,
        "todowrite" => ToolKind::Think,
        _ => ToolKind::Other,
    }
}
fn timestamp_ms(value: &str) -> Option<u64> {
    let parsed =
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()?;
    u64::try_from(parsed.unix_timestamp_nanos() / 1_000_000).ok()
}
