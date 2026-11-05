// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Derived pin-set cache. Source logs and task objects are opened read-only.
use super::{
    fingerprint as fp,
    join::{self, Task, Verdict},
};
use crate::{protocol::*, redact::Set};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{self, Write},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cluster {
    pub fingerprint: String,
    pub count: usize,
    pub episodes: Vec<EvidenceRef>,
    pub first: u64,
    pub last: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub origin: EvidenceRef,
    pub repo: String,
    pub session: SessionId,
    pub turns: Vec<TurnId>,
    pub members: BTreeMap<SessionId, BTreeSet<TurnId>>,
    pub task: Option<Task>,
    pub fingerprints: BTreeSet<String>,
    pub first: u64,
    pub last: u64,
}
#[derive(Default, Debug, Serialize)]
pub struct Replay {
    pub episodes: Vec<Episode>,
    pub updated: usize,
    pub busy: usize,
    pub skipped: usize,
    pub notices: BTreeSet<String>,
    pub max_poll_cpu_ns: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Merge {
    Updated,
    Busy,
}

fn directory(path: &Path) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    if !std::fs::symlink_metadata(path)?.is_dir() {
        return Err(io::Error::other("unsafe dream directory"));
    }
    Ok(())
}
/// A lease stays on this inode; never unlink a claim file during release.
pub struct Claim {
    _file: File,
}
impl Claim {
    pub fn acquire(root: &Path, fingerprint: &str) -> io::Result<Option<Self>> {
        crate::runtime::assert_off_rt_worker();
        if fingerprint.len() != 64 || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(io::Error::other("invalid fingerprint"));
        }
        directory(root)?;
        let claims = root.join("claims");
        directory(&claims)?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(claims.join(fingerprint))?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = io::Error::last_os_error();
            return if error.kind() == io::ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(error)
            };
        }
        let seat = std::env::var("BBOXY_SEAT_ID")
            .ok()
            .filter(|v| v.parse::<ulid::Ulid>().is_ok());
        let content = serde_json::to_vec(
            &serde_json::json!({"pid":std::process::id(),"seat":seat,"started":crate::store::now_ms()}),
        )?;
        file.set_len(0)?;
        file.write_all(&content)?;
        Ok(Some(Self { _file: file }))
    }
}
/// Count is derived from canonical unique pins, never from competing counts.
pub fn union(mut a: Cluster, b: Cluster) -> io::Result<Cluster> {
    if a.fingerprint != b.fingerprint {
        return Err(io::Error::other("cache fingerprint mismatch"));
    }
    let mut pins = BTreeMap::new();
    for pin in a.episodes.into_iter().chain(b.episodes) {
        if matches!(pin, EvidenceRef::Unknown(_)) {
            return Err(io::Error::other("unknown cache pin"));
        }
        pins.insert(serde_json::to_string(&pin)?, pin);
    }
    a.episodes = pins.into_values().collect();
    a.count = a.episodes.len();
    a.first = a.first.min(b.first);
    a.last = a.last.max(b.last);
    Ok(a)
}
pub async fn merge_cluster(
    root: PathBuf,
    repo: String,
    incoming: Cluster,
    _rebuild: bool,
) -> io::Result<Merge> {
    crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let Some(_claim) = Claim::acquire(&root, &incoming.fingerprint)? else {
            return Ok(Merge::Busy);
        };
        let base = root.join("d0");
        directory(&base)?;
        let dir = base.join(fp::digest(repo.as_bytes()));
        directory(&dir)?;
        let path = dir.join(format!("{}.json", incoming.fingerprint));
        let mut merged = union(
            Cluster {
                fingerprint: incoming.fingerprint.clone(),
                count: 0,
                episodes: vec![],
                first: incoming.first,
                last: incoming.last,
            },
            incoming,
        )?;
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if !meta.is_file() => return Err(io::Error::other("unsafe cache file")),
            Ok(_) => {
                // Rebuild rescans input/derives counts; retained committed pins
                // still union under the same claim, never overwrite competing counts.
                merged = union(serde_json::from_slice(&std::fs::read(&path)?)?, merged)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let temp = dir.join(format!(".{}.{}.tmp", merged.fingerprint, ulid::Ulid::new()));
        let result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&temp)?;
            file.write_all(&serde_json::to_vec(&merged)?)?;
            crate::runtime::sync_all(&file)?;
            std::fs::rename(&temp, &path)?;
            crate::runtime::sync_all(&File::open(&dir)?)?;
            Ok(Merge::Updated)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temp);
        }
        result
    })
    .await
}

struct Turn {
    origin: EvidenceRef,
    id: TurnId,
    first: u64,
    last: u64,
    patterns: BTreeSet<String>,
    calls: Vec<(String, String, bool)>,
    denied: Vec<ItemId>,
    tools: BTreeMap<ItemId, String>,
    usage: Option<Usage>,
}
struct Log {
    meta: SessionMeta,
    turns: Vec<Turn>,
    calls: Vec<(String, serde_json::Value)>,
}
fn event_pin(e: &Envelope, line: &str) -> io::Result<EvidenceRef> {
    Ok(EvidenceRef::Event {
        session: e.session,
        seq: e
            .seq
            .ok_or_else(|| io::Error::other("missing durable sequence"))?,
        digest: format!(
            "sha256:{}",
            fp::digest(line.strip_suffix('\n').unwrap_or(line))
        ),
    })
}
fn read_log(path: PathBuf, redactor: Set) -> io::Result<Option<Log>> {
    crate::runtime::assert_off_rt_worker();
    let text = std::fs::read_to_string(path)?;
    let mut events = Vec::new();
    for line in text.split_inclusive('\n').filter(|l| !l.trim().is_empty()) {
        let e = Envelope::from_line(line).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "not a durable IR log; import native input first",
            )
        })?;
        if e.seq.is_some() {
            events.push(e);
        }
    }
    if !matches!(events.last().map(|e| &e.event), Some(Event::SessionClosed)) {
        return Ok(None);
    }
    let Some(Envelope {
        event: Event::SessionStarted { meta },
        ..
    }) = events.first()
    else {
        return Err(io::Error::other("missing session header"));
    };
    let meta = meta.clone();
    let projected: Vec<_> = events
        .iter()
        .cloned()
        .map(|mut e| {
            e.project_into(&meta);
            e
        })
        .collect();
    laws::check_transcript(&projected)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid source transcript"))?;
    drop(projected);
    let mut observed = ClosedLog::new(meta);
    for e in events {
        observed.observe(&e, &redactor, true)?;
    }
    Ok(Some(observed.finish()))
}
/// Incremental D0 event accumulator. C1 never enters path discovery/task/cache replay.
pub struct ClosedLog {
    meta: SessionMeta,
    turns: Vec<Turn>,
    current: Option<usize>,
    items: BTreeMap<ItemId, usize>,
    calls: Vec<(String, serde_json::Value)>,
}
impl ClosedLog {
    pub fn new(meta: SessionMeta) -> Self {
        Self {
            meta,
            turns: vec![],
            current: None,
            items: BTreeMap::new(),
            calls: vec![],
        }
    }
    pub fn observe(&mut self, e: &Envelope, redactor: &Set, retain_calls: bool) -> io::Result<()> {
        let line = e
            .raw
            .as_deref()
            .ok_or_else(|| io::Error::other("missing retained line"))?;
        match &e.event {
            Event::TurnStarted { turn, .. } => {
                self.current = Some(self.turns.len());
                self.turns.push(Turn {
                    origin: event_pin(e, line)?,
                    id: *turn,
                    first: e.ts_ms,
                    last: e.ts_ms,
                    patterns: BTreeSet::new(),
                    calls: vec![],
                    denied: vec![],
                    tools: BTreeMap::new(),
                    usage: None,
                });
            }
            Event::ItemStarted {
                item,
                turn: Some(turn),
                ..
            } => {
                if let Some(i) = self.turns.iter().position(|t| t.id == *turn) {
                    self.items.insert(*item, i);
                }
            }
            Event::ItemCompleted { item, status, body } => {
                let Some(i) = self.items.remove(item).or(self.current) else {
                    return Ok(());
                };
                let t = &mut self.turns[i];
                t.last = t.last.max(e.ts_ms);
                match body {
                    Item::ToolCall {
                        name,
                        input_raw,
                        output,
                        ..
                    } => {
                        let input = serde_json::from_str::<serde_json::Value>(input_raw).ok();
                        if let Some(input) = &input
                            && retain_calls
                        {
                            self.calls.push((name.clone(), input.clone()));
                        }
                        let args = redactor.scrub(
                            &input
                                .as_ref()
                                .map(serde_json::to_string)
                                .transpose()?
                                .unwrap_or_else(|| input_raw.clone()),
                        );
                        let success = matches!(status, ItemStatus::Completed);
                        if matches!(status, ItemStatus::Failed { .. }) {
                            let output = output
                                .iter()
                                .find_map(|b| match b {
                                    OutputBlock::Text { text } | OutputBlock::Error { text } => {
                                        Some(text.clone())
                                    }
                                    OutputBlock::Json { value } => {
                                        serde_json::to_string(value).ok()
                                    }
                                    _ => None,
                                })
                                .unwrap_or_default();
                            t.patterns
                                .insert(fp::tool_fail(name, &redactor.scrub(&output)));
                        }
                        if success && t.calls.iter().any(|(n, a, _)| n == name && a == &args) {
                            t.patterns.insert(fp::retry(name, &args));
                        }
                        t.calls.push((name.clone(), args, success));
                        t.tools.insert(*item, name.clone());
                    }
                    Item::Approval {
                        subject,
                        decision: Some(ApprovalDecision::Deny { .. }),
                        ..
                    } => {
                        t.denied.push(subject.tool_call);
                    }
                    _ => {}
                }
            }
            Event::TurnEnded {
                turn,
                reason,
                usage,
            } => {
                if let Some(t) = self.turns.iter_mut().find(|t| t.id == *turn) {
                    t.last = t.last.max(e.ts_ms);
                    t.usage = usage.clone();
                    if matches!(reason, StopReason::Quiet { .. }) {
                        t.patterns.insert(fp::digest("quiet-stop"));
                    }
                    self.current = None;
                }
            }
            _ => {}
        }

        if !retain_calls
            && (self.turns.len() > 4096 || self.turns.last().is_some_and(|t| t.patterns.len() > 32))
        {
            return Err(io::Error::other("resource limit"));
        }
        Ok(())
    }
    fn finish(mut self) -> Log {
        for t in &mut self.turns {
            for id in &t.denied {
                if let Some(name) = t.tools.get(id) {
                    t.patterns
                        .insert(fp::digest(format!("approval-denied:{name}")));
                }
            }
        }
        Log {
            meta: self.meta,
            turns: self.turns,
            calls: self.calls,
        }
    }
}

fn reported_cost(usage: &Usage) -> Option<(String, u64)> {
    let backend = usage.backend.as_ref()?;
    let model = usage.model.as_ref()?;
    let cost = if let Some(cost) = &usage.cost {
        (cost.source == NumberSource::Reported && cost.currency.eq_ignore_ascii_case("USD"))
            .then_some(cost.amount_micros)?
    } else {
        (usage.source == Some(NumberSource::Reported)).then_some(usage.cost_micros_usd)??
    };
    Some((format!("{}/{model}", backend.name), cost))
}
fn paths(root: PathBuf, out: &mut BTreeSet<PathBuf>) -> io::Result<()> {
    if root
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with(".native-stage-") || n == ".native-custody")
    {
        return Ok(());
    }
    let meta = std::fs::symlink_metadata(&root)?;
    if meta.is_symlink() {
        return Ok(());
    }
    if meta.is_file() {
        out.insert(root);
    } else if meta.is_dir() {
        for p in std::fs::read_dir(root)? {
            let p = p?.path();
            if p.is_dir() || p.extension().is_some_and(|e| e == "jsonl") {
                paths(p, out)?;
            }
        }
    }
    Ok(())
}
async fn replay_inner(
    inputs: Vec<PathBuf>,
    root: PathBuf,
    rebuild: bool,
    redactor: Set,
    sessions: PathBuf,
) -> io::Result<Replay> {
    let inputs = crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let mut out = BTreeSet::new();
        for input in inputs {
            paths(input, &mut out)?;
        }
        Ok::<_, io::Error>(out)
    })
    .await?;
    let mut result = Replay::default();
    let mut logs = Vec::new();
    for input in inputs {
        // Ready IO completions may otherwise chain across the entire corpus in
        // one caller poll. Bound each read phase without changing its contents.
        tokio::task::yield_now().await;
        let file = input.clone();
        let ir = crate::runtime::blocking(move || is_ir(&file)).await?;
        let input = if ir {
            input
        } else {
            match crate::import::materialize_native(input, sessions.clone(), redactor.clone())
                .await?
            {
                Some(path) => path,
                None => {
                    result.busy += 1;
                    result
                        .notices
                        .insert("native materializer busy; replay later".into());
                    continue;
                }
            }
        };
        let set = redactor.clone();
        match crate::runtime::blocking(move || read_log(input, set)).await? {
            Some(log) => logs.push(log),
            None => result.skipped += 1,
        };
    }
    let mut costs: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    // All corpus-wide CPU work is off the two shared IO workers.
    logs = crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let mut observed: BTreeMap<(String, String), BTreeSet<u64>> = BTreeMap::new();
        for log in &logs {
            for turn in &log.turns {
                if let Some((pair, cost)) = turn.usage.as_ref().and_then(reported_cost)
                    && let Ok(pin) = serde_json::to_string(&turn.origin)
                {
                    observed.entry((pair, pin)).or_default().insert(cost);
                }
            }
        }
        for ((pair, _), values) in &observed {
            if values.len() == 1 {
                costs
                    .entry(pair.clone())
                    .or_default()
                    .push(*values.first().unwrap());
            }
        }
        for values in costs.values_mut() {
            values.sort_unstable();
        }
        for log in &mut logs {
            for turn in &mut log.turns {
                if let Some((pair, cost)) = turn.usage.as_ref().and_then(reported_cost)
                    && let Ok(pin) = serde_json::to_string(&turn.origin)
                    && observed
                        .get(&(pair.clone(), pin))
                        .is_some_and(|v| v.len() == 1)
                {
                    let values = &costs[&pair];
                    let rank = (values.len() * 95).div_ceil(100);
                    if cost > values[rank - 1] {
                        turn.patterns
                            .insert(fp::digest(format!("cost-outlier:{pair}")));
                    }
                }
            }
        }
        logs
    })
    .await;
    let mut clusters: BTreeMap<(String, String), Cluster> = BTreeMap::new();
    for log in logs {
        // Includes early-continue paths, which never reach the existing final
        // per-log yield. Claims/cache publication remain in their owned jobs.
        tokio::task::yield_now().await;
        let repo = match log.meta.vcs.as_ref().and_then(|v| v.repo.clone()) {
            Some(repo) => Some(repo),
            None => crate::vcs::repo_key(&log.meta.cwd).await,
        };
        let Some(repo) = repo else {
            result.skipped += 1;
            continue;
        };
        let repo = if repo.contains("://") || (!repo.starts_with("path:") && repo.contains('@')) {
            crate::vcs::normalize_repo_key(&repo)
        } else {
            Some(repo)
        };
        let Some(repo) = repo.filter(|r| redactor.scrub(r) == *r) else {
            result.skipped += 1;
            continue;
        };
        let tasks = match join::committed_tasks(&log.meta.cwd, &repo).await {
            Ok(tasks) => tasks,
            Err(_) => {
                result
                    .notices
                    .insert("committed task catalogue unavailable; event-only episodes".into());
                vec![]
            }
        };
        let set = redactor.clone();
        let new = crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            let joined: Vec<_> = tasks
                .into_iter()
                .filter(|t| log.calls.iter().any(|(n, v)| t.matches(n, v)))
                .filter(|t| serde_json::to_value(&t.pin).is_ok_and(|v| set.value(v.clone()) == v))
                .map(|mut t| {
                    t.verify = t.verify.map(|s| set.scrub(&s));
                    t.rejection = t.rejection.map(|s| set.scrub(&s));
                    t
                })
                .collect();
            let make = |turns: &[&Turn], task: Option<Task>| {
                let first = turns
                    .iter()
                    .map(|t| t.first)
                    .min()
                    .unwrap_or(log.meta.created_ms);
                let last = turns.iter().map(|t| t.last).max().unwrap_or(first);
                let mut fingerprints = turns
                    .iter()
                    .flat_map(|t| t.patterns.clone())
                    .collect::<BTreeSet<_>>();
                if let Some(task) = &task {
                    if task.verdict == Verdict::Rejected {
                        fingerprints.insert(fp::digest(format!(
                            "rejected:{}",
                            fp::normalize(task.rejection.as_deref().unwrap_or_default())
                        )));
                    }
                    if task.verify_exit.is_some_and(|exit| exit != 0)
                        && let Some(verify) = &task.verify
                    {
                        fingerprints.insert(fp::digest(format!("verify-fail:{verify}")));
                    }
                }
                Episode {
                    origin: task
                        .as_ref()
                        .map(|t| t.pin.clone())
                        .unwrap_or_else(|| turns[0].origin.clone()),
                    repo: repo.clone(),
                    session: log.meta.session,
                    turns: turns.iter().map(|t| t.id).collect(),
                    members: BTreeMap::from([(
                        log.meta.session,
                        turns.iter().map(|t| t.id).collect(),
                    )]),
                    task,
                    fingerprints,
                    first,
                    last,
                }
            };
            if joined.is_empty() {
                log.turns
                    .iter()
                    .map(|t| make(&[t], None))
                    .collect::<Vec<_>>()
            } else {
                let turns = log.turns.iter().collect::<Vec<_>>();
                joined
                    .into_iter()
                    .filter(|_| !turns.is_empty())
                    .map(|task| make(&turns, Some(task)))
                    .collect()
            }
        })
        .await;
        // Move union/sorting into the shared blocking pool as well.
        let (next, episodes) = crate::runtime::blocking(move || -> io::Result<_> {
            crate::runtime::assert_off_rt_worker();
            let mut clusters = clusters;
            for e in &new {
                for fingerprint in &e.fingerprints {
                    let key = (e.repo.clone(), fingerprint.clone());
                    let incoming = Cluster {
                        fingerprint: fingerprint.clone(),
                        count: 1,
                        episodes: vec![e.origin.clone()],
                        first: e.first,
                        last: e.last,
                    };
                    if let Some(prior) = clusters.remove(&key) {
                        clusters.insert(key, union(prior, incoming)?);
                    } else {
                        clusters.insert(key, incoming);
                    }
                }
            }
            Ok((clusters, new))
        })
        .await?;
        clusters = next;
        result = crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            result.episodes.extend(episodes);
            result
        })
        .await;
        tokio::task::yield_now().await;
    }
    for ((repo, _), cluster) in clusters {
        match merge_cluster(root.clone(), repo, cluster, rebuild).await? {
            Merge::Updated => result.updated += 1,
            Merge::Busy => result.busy += 1,
        };
        tokio::task::yield_now().await;
    }
    crate::runtime::blocking(move || {
        crate::runtime::assert_off_rt_worker();
        let mut episodes: BTreeMap<(String, String), Episode> = BTreeMap::new();
        for episode in std::mem::take(&mut result.episodes) {
            let key = (
                episode.repo.clone(),
                serde_json::to_string(&episode.origin)?,
            );
            if let Some(prior) = episodes.get_mut(&key) {
                prior.first = prior.first.min(episode.first);
                prior.last = prior.last.max(episode.last);
                prior.fingerprints.extend(episode.fingerprints);
                if episode.session < prior.session {
                    prior.session = episode.session;
                    prior.turns = episode.turns;
                }
                for (session, turns) in episode.members {
                    prior.members.entry(session).or_default().extend(turns);
                }
            } else {
                episodes.insert(key, episode);
            }
        }
        result.episodes = episodes.into_values().collect();
        Ok(result)
    })
    .await
}
/// Only enqueues a detached job: source/session close never joins cache IO.
pub fn on_close(
    path: PathBuf,
    root: PathBuf,
    redactor: Set,
) -> tokio::task::JoinHandle<io::Result<Replay>> {
    bbt_rt::handle().spawn(async move { replay(vec![path], root, false, redactor).await })
}
pub fn session_closed(sessions: &Path, closed: &Envelope, redactor: Set) {
    let path = sessions.join(format!("{}.jsonl", closed.session));
    let root = super::directory();
    drop(bbt_rt::handle().spawn(async move {
        if let Ok(root) = root {
            let _ = on_close(path, root, redactor).await;
        }
    }));
}

fn cpu_ns() -> u64 {
    let mut time = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, time.as_mut_ptr()) } != 0 {
        return u64::MAX;
    }
    let time = unsafe { time.assume_init() };
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}
async fn budgeted(
    future: impl std::future::Future<Output = io::Result<Replay>>,
) -> io::Result<Replay> {
    let mut future = std::pin::pin!(future);
    let mut maximum = 0;
    std::future::poll_fn(move |cx| {
        let start = cpu_ns();
        let poll = future.as_mut().poll(cx);
        let used = cpu_ns().saturating_sub(start);
        maximum = maximum.max(used);
        if used > 1_000_000 {
            return std::task::Poll::Ready(Err(io::Error::other(format!(
                "D0 exceeded 1ms CPU between awaits: {used}ns in replay"
            ))));
        }
        poll.map(|result| {
            result.map(|mut r| {
                r.max_poll_cpu_ns = maximum;
                r
            })
        })
    })
    .await
}
pub async fn replay(
    inputs: Vec<PathBuf>,
    root: PathBuf,
    rebuild: bool,
    redactor: Set,
) -> io::Result<Replay> {
    let sessions = root.with_file_name("sessions");
    replay_with_sessions(inputs, root, sessions, rebuild, redactor).await
}
pub async fn replay_with_sessions(
    inputs: Vec<PathBuf>,
    root: PathBuf,
    sessions: PathBuf,
    rebuild: bool,
    redactor: Set,
) -> io::Result<Replay> {
    budgeted(replay_inner(inputs, root, rebuild, redactor, sessions)).await
}
fn is_ir(path: &Path) -> io::Result<bool> {
    use std::io::{BufRead, BufReader, Read};
    crate::runtime::assert_off_rt_worker();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file.take(crate::import::MAX_NATIVE_SNAPSHOT_BYTES + 1));
    let mut line = zeroize::Zeroizing::new(Vec::new());
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            return Ok(false);
        }
        if line.len() as u64 > crate::import::MAX_NATIVE_SNAPSHOT_BYTES {
            return Err(io::Error::other("source header bound exceeded"));
        }
        if line.iter().any(|c| !c.is_ascii_whitespace()) {
            return Ok(serde_json::from_slice::<Envelope>(&line).is_ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn inline_cpu_broken_control_is_rejected_by_the_actual_poll_budget() {
        let result = budgeted(async {
            let start = cpu_ns();
            while cpu_ns().saturating_sub(start) < 2_000_000 {
                std::hint::spin_loop();
            }
            Ok(Replay::default())
        })
        .await;
        assert!(result.unwrap_err().to_string().contains("1ms CPU"));
    }
}

/// Inspect only already validated immutable selected inputs. No filesystem/task/cache access.
pub fn inspect_closed(
    logs: Vec<ClosedLog>,
    mut unit: impl FnMut(&mut dyn FnMut() -> io::Result<()>) -> io::Result<()>,
) -> io::Result<Vec<super::c1_wire::Row>> {
    use super::c1_wire as w;
    let mut rows = Vec::new();
    let mut costs: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut logs = logs;
    for log in &logs {
        for t in &log.turns {
            if t.usage.is_some() {
                unit(&mut || {
                    if let Some((key, cost)) = t.usage.as_ref().and_then(reported_cost) {
                        costs.entry(key).or_default().push(cost);
                    }
                    Ok(())
                })?;
            }
        }
    }
    for values in costs.values_mut() {
        unit(&mut || {
            values.sort_unstable();
            Ok(())
        })?;
    }
    let mut clusters: BTreeMap<(String, String), Cluster> = BTreeMap::new();
    for log in &mut logs {
        let repo = log
            .meta
            .vcs
            .as_ref()
            .and_then(|v| v.repo.as_ref())
            .ok_or_else(|| io::Error::other("missing selected repo"))?;
        while let Some(t) = log.turns.pop() {
            let mut owned = Some(t);
            unit(&mut || {
                let mut t = owned.take().unwrap();
                for id in &t.denied {
                    if let Some(name) = t.tools.get(id) {
                        t.patterns
                            .insert(fp::digest(format!("approval-denied:{name}")));
                    }
                }
                if let Some((key, cost)) = t.usage.as_ref().and_then(reported_cost) {
                    let values = &costs[&key];
                    if cost > values[(values.len() * 95).div_ceil(100) - 1] {
                        t.patterns.insert(fp::digest(format!("cost-outlier:{key}")));
                    }
                }
                if t.patterns.len() > 32 || rows.len() >= 4096 {
                    return Err(io::Error::other("resource limit"));
                }
                let EvidenceRef::Event {
                    session,
                    seq,
                    digest,
                } = &t.origin
                else {
                    return Err(io::Error::other("invalid pin"));
                };
                rows.push(w::Row::Episode(w::Episode {
                    session: w::Ulid(log.meta.session.to_string()),
                    turn: w::Ulid(t.id.to_string()),
                    origin: w::Origin {
                        session: w::Ulid(session.to_string()),
                        seq: w::Number(seq.0),
                        digest: digest.clone(),
                    },
                    fingerprints: t.patterns.iter().cloned().map(w::Hex).collect(),
                    first_ms: w::Number(t.first),
                    last_ms: w::Number(t.last),
                }));
                for fingerprint in &t.patterns {
                    let key = (repo.clone(), fingerprint.clone());
                    let c = clusters.entry(key).or_insert_with(|| Cluster {
                        fingerprint: fingerprint.clone(),
                        count: 0,
                        episodes: vec![],
                        first: t.first,
                        last: t.last,
                    });
                    // One row per unique TurnStarted; validated sources have distinct sessions.
                    c.count += 1;
                    c.first = c.first.min(t.first);
                    c.last = c.last.max(t.last);
                }
                Ok(())
            })?;
        }
    }
    unit(&mut || {
        rows.sort_by(|a, b| match (a, b) {
            (w::Row::Episode(a), w::Row::Episode(b)) => (
                &a.session,
                a.origin.seq,
                &a.origin.digest,
            )
                .cmp(&(&b.session, b.origin.seq, &b.origin.digest)),
            _ => std::cmp::Ordering::Equal,
        });
        Ok(())
    })?;
    for ((repo, fingerprint), c) in clusters {
        let mut owned = Some((repo, fingerprint, c));
        unit(&mut || {
            let (repo, fingerprint, c) = owned.take().unwrap();
            if rows.len() >= 4096 {
                return Err(io::Error::other("resource limit"));
            }
            rows.push(w::Row::Cluster(w::Cluster {
                repo: w::Repo(repo),
                fingerprint: w::Hex(fingerprint),
                count: c.count as u32,
                first_ms: w::Number(c.first),
                last_ms: w::Number(c.last),
            }));
            Ok(())
        })?;
    }
    // Remaining retained metadata/cost tables are bounded but still measured.
    unit(&mut || {
        logs.clear();
        costs.clear();
        Ok(())
    })?;
    Ok(rows)
}
