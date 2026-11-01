// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Claude Code JSONL -> current protocol, without a provider or executing tools.
//!
//! | Claude record/block | Harness representation |
//! | --- | --- |
//! | user text | UserMessage |
//! | assistant text | AgentMessage (large text split without loss) |
//! | thinking / redacted_thinking | Reasoning / safe Notice |
//! | tool_use + tool_result | paired ToolCall, failed on tool error |
//! | Bash | Terminal child, command/exit + output blob |
//! | Edit / Write / MultiEdit | FileChange + TurnDiff, unified remove/add hunks |
//! | TodoWrite | Plan with entry status |
//! | image | Image metadata only; no image bytes persisted |
//! | sidechain / Task / Agent | Subagent with deterministic child session |
//! | summary / compact_boundary / isCompactSummary | Compaction + summary blob |
//! | usage | Usage; final reported usage on TurnEnded |
//! | user request / end_turn / EOF | queued/start/end; unfinished calls synthesized |
//! | timestamp | original instant in ts_ms (missing: preceding instant or zero) |
//! | unknown / malformed / oversized / truncated | explicit safe Notice |
//!
//! `convert(reader)` returns a backpressured Stream. Setup registers the seat
//! file and available harness/kittens key profiles before any source projection.
//! Parsing and optional blob writes run on a preparation worker, never the UI.
//! Consume the stream; dropping it closes the one-envelope channel and stops
//! production. IDs derive from session identity + source line + local ordinal;
//! no randomness, wall clock, local path lookup or real-log fixtures are used.
//!
//! Memory, independent of session length, is bounded by the configured secret
//! set, MAX_LINE_BYTES, MAX_PENDING_BYTES/MAX_PENDING_TOOLS,
//! one source line's emitted queue, a fixed delivery batch and a one-envelope channel. Excess
//! pending calls are explicitly failed; oversized lines are drained with Notice.
//! Blob roots are optional: without one, ToolCall text remains readable and
//! Terminal/Compaction blob references are metadata; with one (CLI -o), all
//! sanitized text blobs are materialized under <root>/<session>/blobs/<sha256>.
//! Tool-output continuations and compaction summary text also remain in Notice
//! items, so a transcript without sidecars retains its readable text.
use crate::{protocol::Envelope, redact::Set};
use futures_core::Stream;
use std::{
    io::BufRead,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tokio::sync::mpsc;
mod engine;
pub use engine::{MAX_LINE_BYTES, MAX_PENDING_BYTES, MAX_PENDING_TOOLS};
pub const PREPARE_BATCH: usize = 16;

#[derive(Default, Clone)]
pub struct Options {
    pub redactor: Set,
    pub blob_root: Option<PathBuf>,
    /// Default false: resolve user key profiles. Tests/callers supplying a
    /// complete Set can explicitly avoid profile command execution.
    pub supplied_redactor_only: bool,
}
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub input_bytes: u64,
    pub lines: u64,
    pub envelopes: u64,
    pub unknown: u64,
    pub malformed: u64,
    pub truncated: u64,
    pub oversized: u64,
    pub peak_line_bytes: usize,
    pub peak_pending_tools: usize,
    pub peak_pending_bytes: usize,
    pub peak_queued_envelopes: usize,
    pub peak_delivery_batch: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    Input { line: u64 },
    Blob { line: u64 },
    Worker,
}
impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input { line } => write!(f, "input IO failed at line {line}"),
            Self::Blob { line } => write!(f, "blob IO failed at line {line}"),
            Self::Worker => f.write_str("conversion worker failed"),
        }
    }
}
impl std::error::Error for ImportError {}
pub struct Conversion {
    receiver: mpsc::Receiver<Result<Envelope, ImportError>>,
    producer: tokio::task::JoinHandle<()>,
    metrics: Arc<Mutex<Metrics>>,
    ended: bool,
}
impl Conversion {
    pub fn metrics(&self) -> Metrics {
        self.metrics.lock().unwrap().clone()
    }
}
impl Stream for Conversion {
    type Item = Result<Envelope, ImportError>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.ended {
            return Poll::Ready(None);
        }
        match self.receiver.poll_recv(cx) {
            Poll::Ready(None) => {
                match std::future::Future::poll(Pin::new(&mut self.producer), cx) {
                    Poll::Ready(result) => {
                        self.ended = true;
                        Poll::Ready(result.err().map(|_| Err(ImportError::Worker)))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            next => next,
        }
    }
}
impl Drop for Conversion {
    fn drop(&mut self) {
        self.receiver.close();
        self.producer.abort();
    }
}
pub fn convert<R: BufRead + Send + 'static>(reader: R) -> Conversion {
    convert_with(reader, Options::default())
}
pub fn convert_with<R: BufRead + Send + 'static>(reader: R, options: Options) -> Conversion {
    let (send, receiver) = mpsc::channel(1);
    let metrics = Arc::new(Mutex::new(Metrics::default()));
    let observed = metrics.clone();
    let producer = bbt_rt::handle().spawn(async move {
        let notices = configure(&options).await;
        let mut engine = engine::Engine::new(reader, options, notices);
        loop {
            let (next, mut returned) = crate::runtime::blocking(move || {
                let batch = engine.by_ref().take(PREPARE_BATCH).collect::<Vec<_>>();
                (batch, engine)
            })
            .await;
            returned.metrics.peak_delivery_batch =
                returned.metrics.peak_delivery_batch.max(next.len());
            *observed.lock().unwrap() = returned.metrics.clone();
            engine = returned;
            if next.is_empty() {
                break;
            }
            // No preparation permit is held across consumer backpressure.
            for envelope in next {
                if send.send(envelope).await.is_err() {
                    return;
                }
            }
        }
    });
    Conversion {
        receiver,
        producer,
        metrics,
        ended: false,
    }
}

async fn configure(options: &Options) -> Vec<String> {
    use crate::provider::profile::Profile;
    let mut notices = vec![];
    crate::capability::register(&options.redactor).await;
    if let Some(notice) = options.redactor.capability_notice() {
        notices.push(notice);
    }
    if options.supplied_redactor_only {
        return notices;
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let Some(home) = home else {
        return notices;
    };
    let profiles = crate::runtime::blocking(move || {
        let mut found = vec![];
        for directory in [
            home.join(".bboxy450-harness/profiles"),
            home.join(".kittens/profiles"),
        ] {
            if let Ok(entries) = std::fs::read_dir(directory) {
                let mut paths = std::collections::BTreeSet::new();
                for path in entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|s| s == "md"))
                {
                    paths.insert(path);
                }
                for path in paths {
                    use std::io::Read;
                    if let Ok(file) = std::fs::File::open(path) {
                        let mut bytes = Vec::new();
                        if file.take(65537).read_to_end(&mut bytes).is_ok() {
                            // Parse frontmatter even when the documentation
                            // body is longer than the bounded prefix read.
                            if let Ok(profile) = Profile::parse(&String::from_utf8_lossy(&bytes)) {
                                found.push(profile);
                            }
                        }
                    }
                }
            }
        }
        found
    })
    .await;
    for profile in profiles {
        if profile
            .key(
                &options.redactor,
                tokio_util::sync::CancellationToken::new(),
            )
            .await
            .is_err()
        {
            notices.push("Import redaction profile unavailable".into());
        }
    }
    notices
}
