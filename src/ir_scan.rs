// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Observations, never a runtime schema mutation. Parsing and catalogue IO run on
//! blocking workers; the close caller only enqueues a detached bbt-rt job.
#[path = "ir_scan/proposal.rs"]
mod proposal;

use crate::redact::Set;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

pub const SAMPLE_BYTES: usize = 4096;
pub const CPU_BUDGET_NS: u64 = 1_000_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub fingerprint: String,
    pub carrier: String,
    pub provider: String,
    pub record_type: String,
    pub definition: String,
    pub variant: Option<String>,
    pub shape: Vec<String>,
    pub count: u64,
    pub sessions: BTreeSet<String>,
    pub first_seen: u64,
    pub last_seen: u64,
    pub sample: Value,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Scan {
    pub entries: BTreeMap<String, Entry>,
    pub bytes: u64,
    pub lines: u64,
    pub new_fingerprints: BTreeSet<String>,
    pub max_poll_cpu_ns: u64,
}

/// Shape ignores scalar values and array length/order, retaining all key paths
/// and the kinds present at each path. JSON Pointer escaping prevents collisions.
pub fn shape(value: &Value) -> Vec<String> {
    fn walk(v: &Value, path: &str, out: &mut BTreeSet<String>) {
        let kind = match v {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        out.insert(format!("{path}:{kind}"));
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    walk(
                        v,
                        &format!("{path}/{}", k.replace('~', "~0").replace('/', "~1")),
                        out,
                    );
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v, &format!("{path}/*"), out);
                }
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    walk(value, "", &mut out);
    out.into_iter().collect()
}

fn bounded_sample(value: &Value) -> Value {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    if bytes.len() <= SAMPLE_BYTES {
        return value.clone();
    }
    // Truncate by encoded JSON size, so escaping cannot exceed the sample cap.
    let text = String::from_utf8_lossy(&bytes);
    let mut prefix: String = text.chars().take(900).collect();
    loop {
        let sample = json!({"truncated": true, "bytes": bytes.len(), "prefix": prefix});
        if serde_json::to_vec(&sample).unwrap().len() <= SAMPLE_BYTES {
            return sample;
        }
        prefix.pop();
    }
}

impl Scan {
    fn observe(
        &mut self,
        carrier: &str,
        provider: &str,
        record_type: &str,
        definition: &str,
        variant: Option<&str>,
        value: &Value,
        session: &str,
        at: u64,
    ) {
        let shape = shape(value);
        let identity = json!([carrier, provider, record_type, definition, variant, shape]);
        let fingerprint = format!("{:x}", Sha256::digest(identity.to_string().as_bytes()));
        let e = self
            .entries
            .entry(fingerprint.clone())
            .or_insert_with(|| Entry {
                fingerprint,
                carrier: carrier.into(),
                provider: provider.into(),
                record_type: record_type.into(),
                definition: definition.into(),
                variant: variant.map(str::to_owned),
                shape,
                count: 0,
                sessions: BTreeSet::new(),
                first_seen: at,
                last_seen: at,
                sample: bounded_sample(value),
            });
        e.count += 1;
        e.sessions.insert(session.into());
        e.first_seen = e.first_seen.min(at);
        e.last_seen = e.last_seen.max(at);
    }

    pub fn inspect(&mut self, raw: &Value, schema: &Value, redactor: &Set, source: &str) {
        let raw = redact_sample(raw.clone(), redactor);
        if raw.get("event").is_none() || raw.get("session").is_none() {
            return;
        }
        let session = raw["session"].as_str().unwrap_or(source);
        let at = raw["ts_ms"].as_u64().unwrap_or(0);
        if let Some(ext) = raw.get("ext").and_then(Value::as_object) {
            for (provider, value) in ext {
                self.observe(
                    "ext",
                    provider,
                    "extension",
                    "Item",
                    None,
                    value,
                    session,
                    at,
                );
            }
        }
        self.walk(
            &raw,
            &schema["$defs"]["Envelope"],
            schema,
            "Envelope",
            None,
            session,
            at,
        );
    }

    fn walk(
        &mut self,
        raw: &Value,
        node: &Value,
        schema: &Value,
        name: &str,
        variant: Option<&str>,
        session: &str,
        at: u64,
    ) {
        if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
            if let Some(def) = reference.strip_prefix("#/$defs/") {
                self.walk(raw, &schema["$defs"][def], schema, def, None, session, at);
            }
            return;
        }
        if let Some(choices) = node
            .get("oneOf")
            .or_else(|| node.get("anyOf"))
            .and_then(Value::as_array)
        {
            for choice in choices {
                if let Some((tag, kind)) = tag(choice) {
                    if raw.get(tag).and_then(Value::as_str) == Some(kind) {
                        if kind == "unknown" {
                            self.observe("unknown", name, kind, name, None, raw, session, at);
                        } else if name == "Item" && kind == "native" {
                            self.observe(
                                "native",
                                raw["provider"].as_str().unwrap_or("native"),
                                raw["record_type"].as_str().unwrap_or("record"),
                                name,
                                None,
                                raw.get("raw").unwrap_or(raw),
                                session,
                                at,
                            );
                        } else {
                            self.walk(raw, choice, schema, name, Some(kind), session, at);
                        }
                        return;
                    }
                } else if choice.get("$ref").is_some() || type_matches(raw, choice) {
                    // Nullable references and plain string enum branches.
                    self.walk(raw, choice, schema, name, variant, session, at);
                    return;
                }
            }
            self.observe(
                "unknown",
                name,
                raw.get("kind")
                    .or_else(|| raw.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("value"),
                name,
                None,
                raw,
                session,
                at,
            );
            return;
        }
        if let Some(values) = node.get("enum").and_then(Value::as_array) {
            if !values.contains(raw) || raw == "unknown" {
                self.observe(
                    "unknown",
                    name,
                    raw.as_str().unwrap_or("value"),
                    name,
                    None,
                    raw,
                    session,
                    at,
                );
            }
            return;
        }
        if let (Some(object), Some(properties)) = (
            raw.as_object(),
            node.get("properties").and_then(Value::as_object),
        ) {
            let mut extra = serde_json::Map::new();
            for (key, value) in object {
                if name == "Envelope" && key == "ext" {
                    continue;
                }
                if let Some(field) = properties.get(key) {
                    self.walk(value, field, schema, name, variant, session, at);
                } else {
                    extra.insert(key.clone(), value.clone());
                }
            }
            if !extra.is_empty() {
                self.observe(
                    "field",
                    name,
                    variant.unwrap_or(name),
                    name,
                    variant,
                    &Value::Object(extra),
                    session,
                    at,
                );
            }
        } else if let (Some(array), Some(items)) = (raw.as_array(), node.get("items")) {
            for value in array {
                self.walk(value, items, schema, name, variant, session, at);
            }
        }
    }
}

pub(super) fn tag(node: &Value) -> Option<(&str, &str)> {
    for (key, value) in node.get("properties")?.as_object()? {
        if let Some(tag) = value.get("const").and_then(Value::as_str) {
            return Some((key, tag));
        }
    }
    None
}
fn type_matches(raw: &Value, node: &Value) -> bool {
    if let Some(value) = node.get("const") {
        return value == raw;
    }
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return values.contains(raw);
    }
    match node.get("type").and_then(Value::as_str) {
        Some("null") => raw.is_null(),
        Some("string") => raw.is_string(),
        Some("object") => raw.is_object() && node.get("properties").is_some(),
        Some("array") => raw.is_array(),
        _ => false,
    }
}

/// Native logs can hold credentials not registered by this process. In addition
/// to the shared exact-value redactor, mask credential-labelled fields.
fn redact_sample(mut raw: Value, set: &Set) -> Value {
    fn walk(v: &mut Value) {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    let key = k.to_ascii_lowercase();
                    if [
                        "api_key",
                        "apikey",
                        "authorization",
                        "password",
                        "secret",
                        "access_token",
                        "capability",
                    ]
                    .iter()
                    .any(|s| key.contains(s))
                    {
                        *v = Value::String("<redacted>".into());
                    } else {
                        walk(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v);
                }
            }
            _ => {}
        }
    }
    walk(&mut raw);
    set.value(raw)
}

async fn files(paths: Vec<PathBuf>) -> io::Result<Vec<PathBuf>> {
    bbt_rt::handle()
        .spawn_blocking(move || {
            let mut todo = paths;
            let mut result = BTreeSet::new();
            while let Some(path) = todo.pop() {
                let meta = std::fs::symlink_metadata(&path)?;
                if meta.is_dir() {
                    for entry in std::fs::read_dir(path)? {
                        todo.push(entry?.path());
                    }
                } else if meta.is_file()
                    && path
                        .extension()
                        .is_some_and(|e| e == "jsonl" || e == "json")
                {
                    result.insert(path);
                }
            }
            Ok(result.into_iter().collect())
        })
        .await
        .map_err(io::Error::other)?
}

pub fn catalogue_dir() -> io::Result<PathBuf> {
    Ok(
        PathBuf::from(
            std::env::var_os("HOME").ok_or_else(|| io::Error::other("HOME unavailable"))?,
        )
        .join(".bboxy450-harness/ir-unknowns"),
    )
}

/// All expensive work is separated from the async driver by an await. Each file
/// is capped at its initial length, so concurrent appends cannot prolong a scan.
pub async fn scan_paths(
    paths: Vec<PathBuf>,
    schema: Value,
    redactor: Set,
    catalogue: Option<PathBuf>,
) -> io::Result<Scan> {
    scan_inner(paths, schema, redactor, catalogue, false).await
}

/// Deliberately broken control: parsing is put back on the runtime worker. The
/// same budget assertion must reject it; this is never selected by normal IO.
#[doc(hidden)]
pub async fn unbounded_control(paths: Vec<PathBuf>, schema: Value) -> io::Result<Scan> {
    scan_inner(paths, schema, Set::default(), None, true).await
}
async fn scan_inner(
    paths: Vec<PathBuf>,
    schema: Value,
    redactor: Set,
    catalogue: Option<PathBuf>,
    broken: bool,
) -> io::Result<Scan> {
    let schema = Arc::new(schema);
    let mut scan = Scan::default();
    for path in files(paths).await? {
        let file = tokio::fs::File::open(&path).await?;
        let len = file.metadata().await?.len();
        let mut lines = tokio::io::BufReader::new(file.take(len)).lines();
        while let Some(line) = lines.next_line().await? {
            let start = cpu_ns();
            let bytes = line.len() as u64 + 1;
            let schema = schema.clone();
            let redactor = redactor.clone();
            if broken {
                if let Ok(raw) = serde_json::from_str(&line) {
                    scan.inspect(&raw, &schema, &redactor, "log");
                }
                scan.max_poll_cpu_ns = scan.max_poll_cpu_ns.max(cpu_ns().saturating_sub(start));
                scan.bytes += bytes;
                scan.lines += 1;
                assert_budget(scan.max_poll_cpu_ns)?;
                continue;
            }
            scan.max_poll_cpu_ns = scan.max_poll_cpu_ns.max(cpu_ns().saturating_sub(start));
            scan = bbt_rt::handle()
                .spawn_blocking(move || {
                    if let Ok(raw) = serde_json::from_str(&line) {
                        scan.inspect(&raw, &schema, &redactor, "log");
                    }
                    scan.bytes += bytes;
                    scan.lines += 1;
                    scan
                })
                .await
                .map_err(io::Error::other)?;
            tokio::task::yield_now().await;
        }
    }
    if let Some(dir) = catalogue {
        scan = bbt_rt::handle()
            .spawn_blocking(move || merge_catalogue(scan, &dir))
            .await
            .map_err(io::Error::other)??;
    }
    assert_budget(scan.max_poll_cpu_ns)?;
    Ok(scan)
}

pub fn assert_budget(cpu_ns: u64) -> io::Result<()> {
    if cpu_ns > CPU_BUDGET_NS {
        Err(io::Error::other("ir-scan exceeded 1 ms CPU between awaits"))
    } else {
        Ok(())
    }
}
fn cpu_ns() -> u64 {
    let mut time = std::mem::MaybeUninit::<libc::timespec>::uninit();
    // A thread CPU clock excludes scheduler delays and blocking worker work.
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, time.as_mut_ptr()) } != 0 {
        return 0;
    }
    let time = unsafe { time.assume_init() };
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}
fn merge_catalogue(mut scan: Scan, dir: &Path) -> io::Result<Scan> {
    std::fs::create_dir_all(dir)?;
    for entry in scan.entries.values() {
        let path = dir.join(format!("{}.json", entry.fingerprint));
        // A per-fingerprint advisory lock prevents concurrent close scans losing
        // counts. It is held only on a blocking worker, never by a close caller.
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join(format!("{}.lock", entry.fingerprint)))?;
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut merged = entry.clone();
        match std::fs::read(&path) {
            Ok(bytes) => {
                let old: Entry = serde_json::from_slice(&bytes)?;
                merged.count += old.count;
                merged.sessions.extend(old.sessions);
                merged.first_seen = merged.first_seen.min(old.first_seen);
                merged.last_seen = merged.last_seen.max(old.last_seen);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                scan.new_fingerprints.insert(entry.fingerprint.clone());
            }
            Err(e) => return Err(e),
        }
        let tmp = dir.join(format!("{}.{}.tmp", entry.fingerprint, ulid::Ulid::new()));
        std::fs::write(&tmp, serde_json::to_vec_pretty(&merged)?)?;
        std::fs::rename(tmp, path)?;
    }
    Ok(scan)
}

/// Returns immediately; ownership of scan failures stays with the detached job.
/// No session task, UI callback, store lease or shutdown join awaits this job.
pub fn on_close(
    path: PathBuf,
    redactor: Set,
    notice: impl Fn(String) + Send + 'static,
) -> tokio::task::JoinHandle<io::Result<Scan>> {
    let catalogue = catalogue_dir();
    bbt_rt::handle().spawn(async move {
        on_close_in(path, redactor, catalogue?, notice)
            .await
            .map_err(io::Error::other)?
    })
}

pub fn on_close_in(
    path: PathBuf,
    redactor: Set,
    catalogue: PathBuf,
    notice: impl Fn(String) + Send + 'static,
) -> tokio::task::JoinHandle<io::Result<Scan>> {
    bbt_rt::handle().spawn(async move {
        let schema = bbt_rt::handle()
            .spawn_blocking(crate::protocol::schema_value)
            .await
            .map_err(io::Error::other)?;
        let scan = scan_paths(vec![path], schema, redactor, Some(catalogue)).await?;
        let mut providers: BTreeMap<&str, usize> = BTreeMap::new();
        for key in &scan.new_fingerprints {
            *providers.entry(&scan.entries[key].provider).or_default() += 1;
        }
        for (provider, n) in providers {
            notice(format!(
                "{n} unmapped kinds from {provider}; run ir-scan --propose"
            ));
        }
        Ok(scan)
    })
}

pub(crate) fn session_closed(
    sessions: &Path,
    closed: &crate::protocol::Envelope,
    redactor: Set,
    hub: Arc<crate::gateway::Hub>,
) {
    use crate::protocol::*;
    let path = sessions.join(format!("{}.jsonl", closed.session));
    let mut template = closed.clone();
    template.seq = None;
    template.cause = None;
    // Ephemeral notices preserve the already closed source. Observers requesting
    // ephemeral events may consume them after SessionClosed; no writer is opened.
    drop(on_close(path, redactor, move |text| {
        let item = ItemId::new();
        let mut started = template.clone();
        started.event = Event::ItemStarted {
            turn: None,
            item,
            kind: ItemKind::Notice,
            parent: None,
            head: None,
        };
        hub.publish(started);
        let mut completed = template.clone();
        completed.event = Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body: Item::Notice {
                level: Level::Info,
                text,
            },
        };
        hub.publish(completed);
    }));
}

pub async fn cli(args: impl Iterator<Item = String>) -> Result<i32, String> {
    let mut paths = vec![];
    let mut json_output = false;
    let mut propose = None;
    let mut pending = None;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json_output = true,
            "--propose" => {
                propose = Some(PathBuf::from(
                    args.next().ok_or("missing proposal directory")?,
                ))
            }
            "--check-pending" => {
                pending = Some(PathBuf::from(
                    args.next().ok_or("missing pending directory")?,
                ))
            }
            "--help" | "-h" => {
                println!("bboxy450-harness ir-scan [--json] [--propose DIR] PATH…");
                return Ok(0);
            }
            a if a.starts_with('-') => return Err(format!("unknown ir-scan option {a}")),
            _ => paths.push(PathBuf::from(arg)),
        }
    }
    if paths.is_empty() {
        return Err("ir-scan requires a log or directory".into());
    }
    let redactor = Set::default();
    crate::capability::register(&redactor).await;
    let schema = crate::protocol::schema_value();
    let scan = scan_paths(
        paths,
        schema.clone(),
        redactor,
        Some(catalogue_dir().map_err(|e| e.to_string())?),
    )
    .await
    .map_err(|e| e.to_string())?;
    if let Some(dir) = propose {
        proposal::write(&scan, &schema, &dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Some(dir) = pending {
        check_pending(&scan, &dir)
            .await
            .map_err(|e| e.to_string())?;
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&scan).unwrap());
    } else {
        for e in scan.entries.values() {
            println!(
                "{} {} {} · {} ({})",
                e.fingerprint,
                e.count,
                e.provider,
                e.record_type,
                e.sessions.len()
            );
        }
    }
    Ok(0)
}

pub async fn propose(scan: &Scan, schema: &Value, dir: &Path) -> io::Result<()> {
    proposal::write(scan, schema, dir).await
}
pub async fn check_pending(scan: &Scan, dir: &Path) -> io::Result<()> {
    for fingerprint in scan.entries.keys() {
        let path = dir.join(format!("{fingerprint}.jsonl"));
        let content = tokio::fs::read(&path).await.unwrap_or_default();
        let fixture = serde_json::from_slice::<Value>(&content).ok();
        if fixture
            .as_ref()
            .and_then(|v| v.get("fingerprint"))
            .and_then(Value::as_str)
            != Some(fingerprint)
        {
            return Err(io::Error::other(format!(
                "unknown {fingerprint} has no pending proposal"
            )));
        }
    }
    Ok(())
}
