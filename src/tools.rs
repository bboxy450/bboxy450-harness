// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    policy::{EffectClass, Permit},
    protocol::*,
    provider::ToolDef,
    redact::Set,
    runtime::{BlockingWork, assert_off_rt_worker, blocking, blocking_hook},
    store::Blobs,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio_util::sync::CancellationToken;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Concurrency {
    ReadOnly,
    PathWrite,
    Exclusive,
}
#[derive(Clone, Debug)]
pub struct Definition {
    pub wire: ToolDef,
    pub effect: EffectClass,
    pub concurrency: Concurrency,
    pub kind: ToolKind,
}
#[derive(Clone)]
pub struct Registry {
    defs: Arc<Vec<Definition>>,
    trial: Option<TrialCapability>,
    mcp: Arc<BTreeMap<String, crate::mcp::Binding>>,
    readers: Arc<Semaphore>,
    exclusive: Arc<RwLock<()>>,
    paths: Arc<Mutex<BTreeMap<PathBuf, Weak<Mutex<()>>>>>,
}
#[derive(Clone)]
pub struct Context {
    pub cwd: PathBuf,
    pub blobs: Blobs,
    pub redactor: Set,
    pub generation: u64,
    pub current_generation: Arc<AtomicU64>,
    pub cancel: CancellationToken,
}
#[derive(Clone, Debug)]
pub struct ToolOutput {
    pub output: Vec<OutputBlock>,
    pub changes: Vec<Item>,
    pub spill: Option<BlobRef>,
    pub terminal: Option<Item>,
    pub pgid: Option<u32>,
    pub failed: bool,
}
impl ToolOutput {
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            output: vec![OutputBlock::Error { text: text.into() }],
            changes: vec![],
            spill: None,
            terminal: None,
            pgid: None,
            failed: true,
        }
    }
}
impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
impl Registry {
    pub fn new() -> Self {
        let mut defs = vec![];
        for (name, description, effect, concurrency, kind, properties, required) in [
            (
                "read",
                "Read UTF-8 lines or list a directory",
                EffectClass::ReadOnly,
                Concurrency::ReadOnly,
                ToolKind::Read,
                json!({"path":{"type":"string"},"offset":{"type":"integer","minimum":1},"limit":{"type":"integer","minimum":1}}),
                vec!["path"],
            ),
            (
                "write",
                "Write an entire file",
                EffectClass::Write,
                Concurrency::PathWrite,
                ToolKind::Edit,
                json!({"path":{"type":"string"},"content":{"type":"string"}}),
                vec!["path", "content"],
            ),
            (
                "edit",
                "Replace exactly one occurrence; misses leave files unchanged",
                EffectClass::Write,
                Concurrency::PathWrite,
                ToolKind::Edit,
                json!({"path":{"type":"string"},"oldText":{"type":"string"},"newText":{"type":"string"}}),
                vec!["path", "oldText", "newText"],
            ),
            (
                "apply_patch",
                "Stage and validate every exact edit before writing",
                EffectClass::Write,
                Concurrency::PathWrite,
                ToolKind::Edit,
                json!({"edits":{"type":"array","minItems":1,"maxItems":64,"items":{"type":"object","additionalProperties":false,"properties":{"path":{"type":"string"},"oldText":{"type":"string"},"newText":{"type":"string"}},"required":["path","oldText","newText"]}}}),
                vec!["edits"],
            ),
            (
                "bash",
                "Execute a shell command in an owned process group",
                EffectClass::Exec,
                Concurrency::Exclusive,
                ToolKind::Execute,
                json!({"command":{"type":"string"},"timeout":{"type":"integer","minimum":1}}),
                vec!["command"],
            ),
            (
                "exec",
                "Execute a program with literal argv",
                EffectClass::Exec,
                Concurrency::Exclusive,
                ToolKind::Execute,
                json!({"program":{"type":"string"},"args":{"type":"array","maxItems":256,"items":{"type":"string"}},"stdin":{"type":"string"},"timeout":{"type":"integer","minimum":1}}),
                vec!["program"],
            ),
            (
                "grep",
                "Search file contents using a regular expression",
                EffectClass::ReadOnly,
                Concurrency::ReadOnly,
                ToolKind::Search,
                json!({"pattern":{"type":"string"},"path":{"type":"string"}}),
                vec!["pattern"],
            ),
            (
                "glob",
                "Find paths matching a glob",
                EffectClass::ReadOnly,
                Concurrency::ReadOnly,
                ToolKind::Search,
                json!({"pattern":{"type":"string"},"path":{"type":"string"}}),
                vec!["pattern"],
            ),
        ] {
            defs.push(Definition{wire:ToolDef{name:name.into(),description:description.into(),parameters:json!({"type":"object","additionalProperties":false,"properties":properties,"required":required})},effect,concurrency,kind});
        }
        Self {
            defs: Arc::new(defs),
            trial: None,
            mcp: Arc::default(),
            readers: Arc::new(Semaphore::new(4)),
            exclusive: Arc::new(RwLock::new(())),
            paths: Arc::default(),
        }
    }
    pub(crate) fn trial(capability: TrialCapability) -> Self {
        let mut registry = Self::new();
        registry.defs = Arc::new(registry.defs.iter().filter(|d| matches!(d.wire.name.as_str(), "read" | "write")).cloned().map(|mut d| { if d.wire.name == "read" { d.wire.description="Read a confined UTF-8 regular file (50 KiB preview)".into();d.wire.parameters=json!({"type":"object","additionalProperties":false,"properties":{"path":{"type":"string"}},"required":["path"]}); } d }).collect());
        registry.trial = Some(capability);
        registry
    }
    pub(crate) fn is_trial(&self) -> bool {
        self.trial.is_some()
    }
    pub(crate) fn with_mcp(
        &self,
        defs: Vec<Definition>,
        bindings: BTreeMap<String, crate::mcp::Binding>,
    ) -> Self {
        if self.trial.is_some() {
            return self.clone();
        }
        let mut registry = self.clone();
        let mut all = self.defs.as_ref().clone();
        all.extend(defs);
        registry.defs = Arc::new(all);
        registry.mcp = Arc::new(bindings);
        registry
    }
    pub fn definitions(&self) -> &[Definition] {
        &self.defs
    }
    pub fn get(&self, name: &str) -> Option<&Definition> {
        self.defs.iter().find(|d| d.wire.name == name)
    }
    pub async fn run(
        &self,
        name: &str,
        args: Value,
        ctx: Context,
        permit: Option<Permit>,
    ) -> ToolOutput {
        self.run_stream(name, args, ctx, permit, None).await
    }
    pub async fn run_stream(
        &self,
        name: &str,
        args: Value,
        ctx: Context,
        permit: Option<Permit>,
        output: Option<crate::process::OutputSink>,
    ) -> ToolOutput {
        if let Some(capability) = &self.trial {
            return capability.run(name, args, ctx, permit).await;
        }
        let Some(def) = self.get(name) else {
            return ToolOutput::error(format!("unknown tool {name}"));
        };
        if ctx.cancel.is_cancelled()
            || ctx.current_generation.load(Ordering::Acquire) != ctx.generation
        {
            return ToolOutput::error("cancelled or stale tool generation");
        }
        if def.effect != EffectClass::ReadOnly
            && !permit.is_some_and(|p| p.tool == name && p.generation == ctx.generation)
        {
            return ToolOutput::error("policy denied this tool");
        }
        let run = async {
            let _reader = if def.concurrency == Concurrency::ReadOnly {
                Some(self.readers.acquire().await.unwrap())
            } else {
                None
            };
            let _shared = if def.concurrency != Concurrency::Exclusive {
                Some(self.exclusive.read().await)
            } else {
                None
            };
            let _exclusive = if def.concurrency == Concurrency::Exclusive {
                Some(self.exclusive.write().await)
            } else {
                None
            };
            if ctx.cancel.is_cancelled() {
                return ToolOutput::error("cancelled");
            }
            let mut path_guards = vec![];
            if def.concurrency == Concurrency::PathWrite {
                let mut paths = if name == "apply_patch" {
                    args["edits"]
                        .as_array()
                        .map(|v| {
                            v.iter()
                                .filter_map(|e| e["path"].as_str())
                                .map(|p| ctx.cwd.join(p))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                } else {
                    args["path"]
                        .as_str()
                        .map(|p| vec![ctx.cwd.join(p)])
                        .unwrap_or_default()
                };
                paths = blocking(move || {
                    paths
                        .into_iter()
                        .map(|p| canonical_lock_path(&p))
                        .collect::<Vec<_>>()
                })
                .await;
                paths.sort();
                paths.dedup();
                for path in paths {
                    let lock = {
                        let mut map = self.paths.lock().await;
                        map.retain(|_, v| v.strong_count() > 0);
                        if let Some(l) = map.get(&path).and_then(Weak::upgrade) {
                            l
                        } else {
                            let l = Arc::new(Mutex::new(()));
                            map.insert(path, Arc::downgrade(&l));
                            l
                        }
                    };
                    path_guards.push(lock.lock_owned().await);
                }
            }
            if let Some(binding) = self.mcp.get(name) {
                return binding.call(args, &ctx).await;
            }
            if matches!(name, "exec" | "bash") {
                return self.exec(name, &args, &ctx, output).await;
            }
            let name = name.to_owned();
            let a = args.clone();
            let c = ctx.clone();
            let result = blocking(move || file_tool(&name, &a, &c)).await;
            match result {
                Err(e) => ToolOutput::error(ctx.redactor.scrub(&e)),
                Ok((text, changes)) => {
                    let text = ctx.redactor.scrub(&text);
                    let spill = if text.len() > 50 * 1024 {
                        ctx.blobs
                            .put(text.as_bytes().to_vec(), "text/plain")
                            .await
                            .ok()
                    } else {
                        None
                    };
                    let preview = truncate(&text, 50 * 1024);
                    ToolOutput {
                        output: vec![OutputBlock::Text { text: preview }],
                        changes,
                        spill,
                        terminal: None,
                        pgid: None,
                        failed: false,
                    }
                }
            }
        };
        let output = run.await;
        if ctx.current_generation.load(Ordering::Acquire) != ctx.generation {
            ToolOutput::error("stale completion retired")
        } else {
            output
        }
    }
    async fn exec(
        &self,
        name: &str,
        args: &Value,
        ctx: &Context,
        output: Option<crate::process::OutputSink>,
    ) -> ToolOutput {
        let (program, argv) = if name == "bash" {
            let Some(command) = args["command"].as_str() else {
                return ToolOutput::error("command is required");
            };
            ("bash".to_owned(), vec!["-c".into(), command.into()])
        } else {
            let Some(program) = args["program"].as_str() else {
                return ToolOutput::error("program is required");
            };
            let argv = args["args"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .map(|v| v.as_str().map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                })
                .unwrap_or(Some(vec![]));
            let Some(argv) = argv.filter(|v| v.len() <= 256) else {
                return ToolOutput::error("args must be at most 256 strings");
            };
            (program.into(), argv)
        };
        let command = ctx
            .redactor
            .scrub(&format!("{} {}", program, argv.join(" ")));
        let req = crate::process::Request {
            program,
            args: argv,
            stdin: args["stdin"].as_str().map(str::to_owned),
            cwd: ctx.cwd.clone(),
            timeout: Duration::from_secs(args["timeout"].as_u64().unwrap_or(120).min(3600)),
            spill_dir: ctx.blobs.dir.parent().unwrap().join("tmp"),
            cap: 50 * 1024,
            redactor: ctx.redactor.clone(),
        };
        match crate::process::run_stream(req, ctx.cancel.clone(), output).await {
            Err(_) => ToolOutput::error("process could not run"),
            Ok(out) => {
                let spill = if let Some(path) = out.spill {
                    ctx.blobs.ingest(path, "text/plain").await.ok()
                } else {
                    None
                };
                let output_blob = if let Some(b) = &spill {
                    Some(b.clone())
                } else {
                    ctx.blobs.put(out.bytes.clone(), "text/plain").await.ok()
                };
                let text = String::from_utf8_lossy(&out.bytes).into_owned();
                let failed = out.code != Some(0) || out.cancelled;
                ToolOutput {
                    output: vec![if failed {
                        OutputBlock::Error { text }
                    } else {
                        OutputBlock::Text { text }
                    }],
                    changes: vec![],
                    spill,
                    terminal: Some(Item::Terminal {
                        terminal: out.pgid.to_string(),
                        command,
                        exit: out.code,
                        signal: out.signal,
                        output: output_blob,
                        live: None,
                    }),
                    pgid: Some(out.pgid),
                    failed,
                }
            }
        }
    }
}
fn canonical_lock_path(path: &Path) -> PathBuf {
    if let Ok(p) = path.canonicalize() {
        return p;
    }
    let mut tail = vec![];
    let mut cursor = path;
    while !cursor.exists() {
        if let Some(n) = cursor.file_name() {
            tail.push(n.to_owned());
        }
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent;
    }
    let mut p = cursor
        .canonicalize()
        .unwrap_or_else(|_| cursor.to_path_buf());
    for n in tail.into_iter().rev() {
        p.push(n);
    }
    p
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str, String> {
    v[k].as_str().ok_or_else(|| format!("{k} must be a string"))
}
fn truncate(s: &str, cap: usize) -> String {
    let mut end = s.len().min(cap);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].into()
}
fn exact(old: &str, needle: &str, replacement: &str) -> Result<String, String> {
    if needle.is_empty() {
        return Err("oldText must not be empty".into());
    }
    let count = old.matches(needle).count();
    if count != 1 {
        let hint = old
            .lines()
            .filter(|line| {
                line.trim()
                    .to_lowercase()
                    .contains(&needle.trim().to_lowercase())
            })
            .take(3)
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "edit requires exactly one match; found {count}. did you mean:\n{}",
            if hint.is_empty() {
                truncate(old, 512)
            } else {
                hint
            }
        ));
    }
    Ok(old.replacen(needle, replacement, 1))
}
fn diff(path: &Path, old: &str, new: &str) -> String {
    blocking_hook(BlockingWork::Diff);
    let mut out = format!(
        "--- a/{}\n+++ b/{}\n@@ -1,{} +1,{} @@\n",
        path.display(),
        path.display(),
        old.lines().count(),
        new.lines().count()
    );
    for l in old.lines() {
        out.push('-');
        out.push_str(l);
        out.push('\n');
    }
    for l in new.lines() {
        out.push('+');
        out.push_str(l);
        out.push('\n');
    }
    out
}
fn file_tool(name: &str, args: &Value, ctx: &Context) -> Result<(String, Vec<Item>), String> {
    assert_off_rt_worker();
    let path = ctx.cwd.join(args["path"].as_str().unwrap_or("."));
    let io = |_: std::io::Error| "filesystem operation failed".to_string();
    match name {
        "read" => {
            text(args, "path")?;
            if path.is_dir() {
                blocking_hook(BlockingWork::DirectoryRead);
                let mut names = std::fs::read_dir(path)
                    .map_err(io)?
                    .map(|e| e.map(|e| e.file_name().to_string_lossy().to_string()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(io)?;
                names.sort();
                return Ok((
                    names.into_iter().take(2000).collect::<Vec<_>>().join("\n"),
                    vec![],
                ));
            }
            let content = std::fs::read_to_string(path).map_err(io)?;
            let offset = args["offset"].as_u64().unwrap_or(1);
            if offset == 0 {
                return Err("offset starts at 1".into());
            }
            let limit = args["limit"].as_u64().unwrap_or(2000);
            Ok((
                content
                    .lines()
                    .skip((offset - 1) as usize)
                    .take(limit as usize)
                    .enumerate()
                    .map(|(i, l)| format!("{}: {l}\n", offset + i as u64))
                    .collect(),
                vec![],
            ))
        }
        "write" | "edit" | "apply_patch" => {
            let edits = if name == "apply_patch" {
                args["edits"]
                    .as_array()
                    .filter(|v| !v.is_empty() && v.len() <= 64)
                    .ok_or("edits must contain 1..64 changes")?
                    .clone()
            } else {
                vec![args.clone()]
            };
            let mut staged: BTreeMap<PathBuf, (String, String, bool)> = BTreeMap::new();
            for e in edits {
                if ctx.cancel.is_cancelled() {
                    return Err("cancelled".into());
                }
                let path = ctx.cwd.join(text(&e, "path")?);
                let exists = path.exists();
                let before = if let Some((_, next, _)) = staged.get(&path) {
                    next.clone()
                } else if exists {
                    std::fs::read_to_string(&path).map_err(io)?
                } else if name == "write" {
                    String::new()
                } else {
                    return Err("file does not exist".into());
                };
                let after = if name == "write" {
                    text(&e, "content")?.to_string()
                } else {
                    exact(&before, text(&e, "oldText")?, text(&e, "newText")?)?
                };
                if let Some((_, next, _)) = staged.get_mut(&path) {
                    *next = after;
                } else {
                    staged.insert(path, (before, after, exists));
                }
            }
            let mut changes = vec![];
            let mut output = String::new();
            for (path, (before, after, existed)) in staged {
                if ctx.cancel.is_cancelled() {
                    return Err("cancelled before write".into());
                }
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(io)?;
                }
                std::fs::write(&path, &after).map_err(io)?;
                let unified = ctx.redactor.scrub(&diff(&path, &before, &after));
                output.push_str(&unified);
                changes.push(Item::FileChange {
                    path,
                    change: if existed {
                        ChangeKind::Modify
                    } else {
                        ChangeKind::Add
                    },
                    unified: truncate(&unified, 50 * 1024),
                });
            }
            Ok((output, changes))
        }
        "glob" | "grep" => {
            blocking_hook(if name == "grep" {
                BlockingWork::Grep
            } else {
                BlockingWork::Glob
            });
            let pattern = text(args, "pattern")?;
            let regex = if name == "grep" {
                Some(regex::Regex::new(pattern).map_err(|_| "invalid regex")?)
            } else {
                None
            };
            let glob = if name == "glob" {
                Some(glob::Pattern::new(pattern).map_err(|_| "invalid glob")?)
            } else {
                None
            };
            let mut stack = vec![path.clone()];
            let mut output = String::new();
            let mut hits = 0;
            blocking_hook(BlockingWork::Walk);
            while let Some(p) = stack.pop() {
                if ctx.cancel.is_cancelled() {
                    return Err("cancelled".into());
                }
                let meta = std::fs::symlink_metadata(&p).map_err(io)?;
                if meta.file_type().is_symlink() {
                    continue;
                }
                if p.is_dir() {
                    for e in std::fs::read_dir(&p).map_err(io)? {
                        stack.push(e.map_err(io)?.path());
                    }
                } else if let Some(glob) = &glob {
                    let relative = p.strip_prefix(&path).unwrap_or(&p);
                    if glob.matches_path(relative) || glob.matches_path(&p) {
                        output.push_str(&format!("{}\n", p.display()));
                        hits += 1;
                    }
                } else if let Ok(content) = std::fs::read_to_string(&p) {
                    for (i, line) in content.lines().enumerate() {
                        if regex.as_ref().unwrap().is_match(line) {
                            output.push_str(&format!("{}:{}:{line}\n", p.display(), i + 1));
                            hits += 1;
                            if hits >= 10000 {
                                break;
                            }
                        }
                    }
                }
                if hits >= 10000 {
                    break;
                }
            }
            Ok((output, vec![]))
        }
        _ => Err("unknown file tool".into()),
    }
}

// Trial-only descriptor capability. Application confinement of the advertised
// read/write builtins, not an OS sandbox for malicious Rust providers or same-uid
// external root relocation/mount injection. Minted by trusted trial orchestration.
#[derive(Clone)]
pub struct TrialCapability(Arc<TrialRoot>);
struct TrialRoot {
    fd: std::fs::File,
    cwd: PathBuf,
    sessions: PathBuf,
    profiles: PathBuf,
    home: PathBuf,
    gate: Arc<Mutex<()>>,
    closed: std::sync::atomic::AtomicBool,
    #[cfg(test)]
    after_parent: std::sync::Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
const TRIAL_FILE_LIMIT: usize = 1024 * 1024;
impl TrialCapability {
    pub(crate) fn mint(
        cwd: PathBuf,
        sessions: PathBuf,
        profiles: PathBuf,
        home: PathBuf,
    ) -> std::io::Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        assert_off_rt_worker();
        for path in [&cwd, &sessions, &profiles, &home] {
            private_trial_dir(path)?;
        }
        let fd = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .open(&cwd)?;
        Ok(Self(Arc::new(TrialRoot {
            fd,
            cwd,
            sessions,
            profiles,
            home,
            gate: Arc::new(Mutex::new(())),
            closed: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            after_parent: std::sync::Mutex::new(None),
        })))
    }
    pub(crate) fn validate_launch(
        &self,
        cwd: &Path,
        sessions: &Path,
        profiles: &Path,
    ) -> std::io::Result<()> {
        use std::os::unix::fs::MetadataExt;
        assert_off_rt_worker();
        if self.0.closed.load(Ordering::Acquire)
            || cwd != self.0.cwd
            || sessions != self.0.sessions
            || profiles != self.0.profiles
        {
            return Err(std::io::Error::other("trial launch roots mismatch"));
        }
        for path in [
            &self.0.cwd,
            &self.0.sessions,
            &self.0.profiles,
            &self.0.home,
        ] {
            private_trial_dir(path)?;
        }
        let a = self.0.fd.metadata()?;
        let b = std::fs::symlink_metadata(cwd)?;
        if a.dev() != b.dev() || a.ino() != b.ino() {
            return Err(std::io::Error::other("trial root identity changed"));
        }
        Ok(())
    }
    /// Prevent new admissions, then join the serial descriptor work, including a
    /// blocking closure whose original future was dropped. No child is spawned.
    pub(crate) async fn finish(&self) {
        self.0.closed.store(true, Ordering::Release);
        let _joined = self.0.gate.lock().await;
    }
    pub(crate) async fn verifier_preflight(&self) -> Result<bool, String> {
        let guard = self.0.gate.clone().lock_owned().await;
        let this = self.clone();
        blocking(move || {
            let _guard = guard;
            this.open_file("fixed", false, false).map(|f| f.is_some())
        })
        .await
    }
    async fn run(
        &self,
        name: &str,
        args: Value,
        ctx: Context,
        permit: Option<Permit>,
    ) -> ToolOutput {
        if !matches!(name, "read" | "write") {
            return ToolOutput::error("trial denies exec, MCP, network and unsupported tools");
        }
        if args.as_object().is_none_or(|o| {
            o.keys()
                .any(|k| k != "path" && !(name == "write" && k == "content"))
        }) {
            return ToolOutput::error("unsupported trial tool arguments");
        }
        if name == "write"
            && !permit.is_some_and(|p| p.tool == name && p.generation == ctx.generation)
        {
            return ToolOutput::error("policy denied this tool");
        }
        if ctx.cwd != self.0.cwd {
            return ToolOutput::error("trial cwd mismatch");
        }
        let cap = self.clone();
        let name = name.to_owned();
        let guard = self.0.gate.clone().lock_owned().await;
        let redactor = ctx.redactor.clone();
        let result = blocking(move || -> Result<(String, Vec<Item>), String> {
            let _guard = guard;
            assert_off_rt_worker();
            let fence = || {
                if cap.0.closed.load(Ordering::Acquire)
                    || ctx.cancel.is_cancelled()
                    || ctx.current_generation.load(Ordering::Acquire) != ctx.generation
                {
                    Err("cancelled or stale trial tool".to_string())
                } else {
                    Ok(())
                }
            };
            fence()?;
            let path = text(&args, "path")?;
            if crate::dream::deny::value(&json!(path), &ctx.redactor).is_err() {
                return Err("unsafe trial path".into());
            }
            if name == "read" {
                let file = cap
                    .open_file(path, false, false)?
                    .ok_or("file does not exist")?;
                let content = trial_read(&file)?;
                fence()?;
                let preview = if content.len() > 50 * 1024 {
                    format!(
                        "{}\n[trial read truncated at 50 KiB]",
                        truncate(&content, 50 * 1024)
                    )
                } else {
                    content
                };
                return Ok((preview, vec![]));
            }
            let content = text(&args, "content")?;
            if content.len() > TRIAL_FILE_LIMIT {
                return Err("trial file exceeds bound".into());
            }
            // Existing file opened non-destructively and checked before mutation.
            let (mut file, existed) = cap.write_file(path, fence)?;
            let before = trial_read(&file)?;
            fence()?;
            use std::io::{Seek, Write};
            file.rewind().map_err(|_| "trial seek failed")?;
            file.write_all(content.as_bytes())
                .map_err(|_| "trial write failed")?;
            file.set_len(content.len() as u64)
                .map_err(|_| "trial truncate failed")?;
            let unified = trial_safe_text(&diff(Path::new(path), &before, content), &ctx.redactor);
            Ok((
                truncate(&unified, 50 * 1024),
                vec![Item::FileChange {
                    path: PathBuf::from(path),
                    change: if existed {
                        ChangeKind::Modify
                    } else {
                        ChangeKind::Add
                    },
                    unified: truncate(&unified, 50 * 1024),
                }],
            ))
        })
        .await;
        match result {
            Ok((text, changes)) => ToolOutput {
                output: vec![OutputBlock::Text {
                    text: trial_safe_text(&text, &redactor),
                }],
                changes,
                spill: None,
                terminal: None,
                pgid: None,
                failed: false,
            },
            Err(e) => ToolOutput::error(redactor.scrub(&e)),
        }
    }
    fn parent(&self, path: &str) -> Result<(std::fs::File, std::ffi::CString), String> {
        use std::path::Component;
        let components = Path::new(path)
            .components()
            .map(|c| match c {
                Component::Normal(n) if !n.as_encoded_bytes().eq_ignore_ascii_case(b".git") => {
                    std::ffi::CString::new(n.as_encoded_bytes())
                        .map_err(|_| "invalid trial path".to_string())
                }
                _ => Err("trial path must be relative normal components".into()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if components.is_empty()
            || path.as_bytes().contains(&0)
            || path.split('/').any(|p| p == "." || p.is_empty())
        {
            return Err("invalid trial path".into());
        }
        let mut fd = self
            .0
            .fd
            .try_clone()
            .map_err(|_| "trial root unavailable")?;
        for component in &components[..components.len() - 1] {
            fd = trial_openat(&fd, component, libc::O_RDONLY | libc::O_DIRECTORY, 0)
                .map_err(|_| "trial component refused")?;
        }
        #[cfg(test)]
        if let Some(hook) = self.0.after_parent.lock().unwrap().take() {
            hook();
        }
        Ok((fd, components.last().unwrap().clone()))
    }
    fn write_file(
        &self,
        path: &str,
        before_create: impl FnOnce() -> Result<(), String>,
    ) -> Result<(std::fs::File, bool), String> {
        let (parent, name) = self.parent(path)?;
        let (file, existed) = match trial_openat(&parent, &name, libc::O_RDWR, 0) {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                before_create()?;
                (
                    trial_openat(
                        &parent,
                        &name,
                        libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                        0o600,
                    )
                    .map_err(|_| "trial exclusive create refused")?,
                    false,
                )
            }
            Err(_) => return Err("trial file refused".into()),
        };
        trial_regular(&file)?;
        Ok((file, existed))
    }
    pub(crate) fn validate_store(&self, cwd: &Path, sessions: &Path) -> std::io::Result<()> {
        self.validate_launch(cwd, sessions, &self.0.profiles)
    }
    fn open_file(
        &self,
        path: &str,
        write: bool,
        create: bool,
    ) -> Result<Option<std::fs::File>, String> {
        let (parent, name) = self.parent(path)?;
        let flags = if create {
            libc::O_CREAT | libc::O_EXCL | libc::O_RDWR
        } else if write {
            libc::O_RDWR
        } else {
            libc::O_RDONLY
        };
        let file = match trial_openat(&parent, &name, flags, 0o600) {
            Ok(file) => file,
            Err(e) if !create && e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("trial file refused".into()),
        };
        trial_regular(&file)?;
        Ok(Some(file))
    }
}
fn trial_safe_text(text: &str, set: &Set) -> String {
    match crate::dream::deny::model_input(json!(text), set) {
        Value::String(text) => text,
        _ => unreachable!("model_input preserves string shape"),
    }
}
fn private_trial_dir(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let m = std::fs::symlink_metadata(path)?;
    if !m.is_dir()
        || m.file_type().is_symlink()
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o077 != 0
    {
        return Err(std::io::Error::other(
            "trial directory is not private and owned",
        ));
    }
    Ok(())
}
fn trial_openat(
    parent: &std::fs::File,
    name: &std::ffi::CStr,
    flags: i32,
    mode: libc::mode_t,
) -> std::io::Result<std::fs::File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    // SAFETY: owned parent remains live; NUL-terminated component, no-follow,
    // nonblocking and close-on-exec on EVERY opened descriptor. No path reopen.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            mode as libc::c_uint,
        )
    };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { std::fs::File::from_raw_fd(fd) })
    }
}
fn trial_regular(file: &std::fs::File) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let m = file.metadata().map_err(|_| "trial metadata failed")?;
    if !m.is_file() || m.nlink() != 1 || m.len() > TRIAL_FILE_LIMIT as u64 {
        return Err("trial requires bounded single-link regular file".into());
    }
    Ok(())
}
fn trial_read(file: &std::fs::File) -> Result<String, String> {
    use std::io::Read;
    trial_regular(file)?;
    let mut bytes = Vec::new();
    file.take((TRIAL_FILE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "trial read failed")?;
    if bytes.len() > TRIAL_FILE_LIMIT {
        return Err("trial file exceeds bound".into());
    }
    String::from_utf8(bytes).map_err(|_| "trial file is not UTF-8".into())
}

#[cfg(test)]
mod trial_tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    async fn fixture() -> (tempfile::TempDir, TrialCapability, Context) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        for name in ["repo", "sessions", "profiles", "home"] {
            std::fs::create_dir(root.join(name)).unwrap();
            std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        let cap = blocking(move || {
            TrialCapability::mint(
                root.join("repo"),
                root.join("sessions"),
                root.join("profiles"),
                root.join("home"),
            )
        })
        .await
        .unwrap();
        let context = Context {
            cwd: temp.path().join("repo"),
            blobs: Blobs {
                dir: temp.path().join("blobs"),
                redactor: Set::default(),
            },
            redactor: Set::default(),
            generation: 1,
            current_generation: Arc::new(AtomicU64::new(1)),
            cancel: CancellationToken::new(),
        };
        (temp, cap, context)
    }
    #[tokio::test]
    async fn confined_registry_rejects_escapes_links_special_files_and_forged_allow() {
        let (temp, cap, ctx) = fixture().await;
        let registry = Registry::trial(cap.clone());
        let outside = temp.path().join("sentinel");
        std::fs::write(&outside, "private").unwrap();
        std::fs::create_dir(ctx.cwd.join("nested")).unwrap();
        symlink(temp.path(), ctx.cwd.join("link")).unwrap();
        symlink(&outside, ctx.cwd.join("leaf")).unwrap();
        std::fs::hard_link(&outside, ctx.cwd.join("hard")).unwrap();
        let fifo =
            std::ffi::CString::new(ctx.cwd.join("fifo").as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let permit = || Some(Permit::approval("write".into(), 1));
        for path in [
            "../sentinel",
            outside.to_str().unwrap(),
            "link/sentinel",
            "leaf",
            "hard",
            "fifo",
            ".git/config",
            "nested/../x",
        ] {
            assert!(
                registry
                    .run("read", json!({"path":path}), ctx.clone(), None)
                    .await
                    .failed,
                "read {path}"
            );
            assert!(
                registry
                    .run(
                        "write",
                        json!({"path":path,"content":"escape"}),
                        ctx.clone(),
                        permit()
                    )
                    .await
                    .failed,
                "write {path}"
            );
        }
        for name in [
            "exec",
            "bash",
            "network",
            "install",
            "git push",
            "cargo publish",
            "gh",
            "apply_patch",
            "mcp_fake",
        ] {
            assert!(registry.run(name,json!({"program":"sh","command":"touch escaped","edits":[{"path":"safe","oldText":"","newText":"yes"},{"path":"../sentinel","oldText":"private","newText":"escape"}]}),ctx.clone(),Some(Permit::approval(name.into(),1))).await.failed);
        }
        assert!(!ctx.cwd.join("safe").exists());
        let mut wrong = ctx.clone();
        wrong.cwd = temp.path().to_path_buf();
        assert!(
            registry
                .run(
                    "write",
                    json!({"path":"sentinel","content":"escape"}),
                    wrong,
                    permit()
                )
                .await
                .failed
        );
        ctx.current_generation.store(2, Ordering::Release);
        assert!(
            registry
                .run(
                    "write",
                    json!({"path":"fixed","content":"yes"}),
                    ctx.clone(),
                    permit()
                )
                .await
                .failed
        );
        ctx.current_generation.store(1, Ordering::Release);
        ctx.cancel.cancel();
        assert!(
            registry
                .run(
                    "write",
                    json!({"path":"fixed","content":"yes"}),
                    ctx.clone(),
                    permit()
                )
                .await
                .failed
        );
        assert_eq!(std::fs::read_to_string(outside).unwrap(), "private");
        assert!(!ctx.cwd.join("fixed").exists());
        cap.finish().await;
    }
    #[tokio::test]
    async fn confined_real_write_read_and_descriptor_substitution_preserve_external_bytes() {
        let (temp, cap, ctx) = fixture().await;
        let registry = Registry::trial(cap.clone());
        assert_eq!(
            registry
                .definitions()
                .iter()
                .map(|d| d.wire.name.as_str())
                .collect::<Vec<_>>(),
            ["read", "write"]
        );
        assert!(
            !registry
                .run(
                    "write",
                    json!({"path":"fixed","content":"private result"}),
                    ctx.clone(),
                    Some(Permit::approval("write".into(), 1))
                )
                .await
                .failed
        );
        let read = registry
            .run("read", json!({"path":"fixed"}), ctx.clone(), None)
            .await;
        assert!(!read.failed);
        assert!(matches!(&read.output[0],OutputBlock::Text{text} if text=="private result"));
        // Actual openat-parent descriptor survives replacing its pathname with a
        // symlink; a reopened cwd.join would write the external private sentinel.
        std::fs::create_dir(ctx.cwd.join("nested")).unwrap();
        let outside = temp.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("victim"), "private").unwrap();
        let root = ctx.cwd.clone();
        let outside_copy = outside.clone();
        *cap.0.after_parent.lock().unwrap() = Some(Box::new(move || {
            std::fs::rename(root.join("nested"), root.join("retained")).unwrap();
            symlink(outside_copy, root.join("nested")).unwrap();
        }));
        let write = registry
            .run(
                "write",
                json!({"path":"nested/victim","content":"owned"}),
                ctx.clone(),
                Some(Permit::approval("write".into(), 1)),
            )
            .await;
        assert!(!write.failed);
        assert_eq!(
            std::fs::read_to_string(outside.join("victim")).unwrap(),
            "private"
        );
        assert_eq!(
            std::fs::read_to_string(ctx.cwd.join("retained/victim")).unwrap(),
            "owned"
        );
        let generation = ctx.current_generation.clone();
        *cap.0.after_parent.lock().unwrap() = Some(Box::new(move || {
            generation.store(2, Ordering::Release);
        }));
        assert!(
            registry
                .run(
                    "write",
                    json!({"path":"stale-create","content":"must not exist"}),
                    ctx.clone(),
                    Some(Permit::approval("write".into(), 1))
                )
                .await
                .failed
        );
        assert!(!ctx.cwd.join("stale-create").exists());
        ctx.current_generation.store(1, Ordering::Release);
        cap.finish().await;
        assert!(
            registry
                .run(
                    "write",
                    json!({"path":"fixed","content":"late"}),
                    ctx.clone(),
                    Some(Permit::approval("write".into(), 1))
                )
                .await
                .failed
        );
        assert_eq!(
            std::fs::read_to_string(ctx.cwd.join("fixed")).unwrap(),
            "private result"
        );
    }
    #[tokio::test]
    async fn trial_mcp_rebinding_cannot_widen_definitions_or_entry_point() {
        let (_temp, cap, ctx) = fixture().await;
        let base = Registry::trial(cap);
        let rebound = base.with_mcp(Registry::new().definitions().to_vec(), BTreeMap::new());
        assert_eq!(rebound.definitions().len(), 2);
        assert!(
            rebound
                .run(
                    "exec",
                    json!({"program":"touch","args":["escape"]}),
                    ctx,
                    Some(Permit::approval("exec".into(), 1))
                )
                .await
                .failed
        );
    }
}
