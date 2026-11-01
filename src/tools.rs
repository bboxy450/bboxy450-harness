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
    runtime::blocking,
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
            readers: Arc::new(Semaphore::new(4)),
            exclusive: Arc::new(RwLock::new(())),
            paths: Arc::default(),
        }
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
            if matches!(name, "exec" | "bash") {
                return self.exec(name, &args, &ctx).await;
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
    async fn exec(&self, name: &str, args: &Value, ctx: &Context) -> ToolOutput {
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
        match crate::process::run(req, ctx.cancel.clone()).await {
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
    let path = ctx.cwd.join(args["path"].as_str().unwrap_or("."));
    let io = |_: std::io::Error| "filesystem operation failed".to_string();
    match name {
        "read" => {
            text(args, "path")?;
            if path.is_dir() {
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
