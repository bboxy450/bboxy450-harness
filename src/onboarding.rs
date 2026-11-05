// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Bounded local discovery and explicit, fixed-argv adapter installation.
//! Registry facts are reviewed source constants, never downloaded executable recipes.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Duration,
};
use tokio::{io::AsyncReadExt, sync::watch};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Agent {
    Codex,
    Claude,
    Pi,
    Grok,
    Gemini,
    OpenCode,
    Omp,
    Goose,
}
pub const AGENTS: [Agent; 8] = [
    Agent::Codex,
    Agent::Claude,
    Agent::Pi,
    Agent::Grok,
    Agent::Gemini,
    Agent::OpenCode,
    Agent::Omp,
    Agent::Goose,
];
impl Agent {
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude Code",
            Self::Pi => "Pi",
            Self::Grok => "Grok Build",
            Self::Gemini => "Gemini CLI",
            Self::OpenCode => "OpenCode",
            Self::Omp => "Oh My Pi",
            Self::Goose => "Goose",
        }
    }
    pub fn cli(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Pi => "pi",
            Self::Grok => "grok",
            Self::Gemini => "gemini",
            Self::OpenCode => "opencode",
            Self::Omp => "omp",
            Self::Goose => "goose",
        }
    }
    pub fn adapter(self) -> Option<(&'static str, &'static [&'static str])> {
        Some(match self {
            Self::Codex => ("codex-acp", &[]),
            Self::Claude => ("claude-agent-acp", &[]),
            Self::Pi => ("pi-acp", &[]),
            // Verified ACP registry grok-build, Gemini and OpenCode contracts (2026-11-04).
            Self::Grok => ("grok", &["agent", "stdio"]),
            Self::Gemini => ("gemini", &["--acp"]),
            Self::OpenCode => ("opencode", &["acp"]),
            // Official ACP registry: https://github.com/agentclientprotocol/registry/blob/main/goose/agent.json
            // Command documented at https://block.github.io/goose/docs/gdk/acp/
            Self::Goose => ("goose", &["acp"]),
            Self::Omp => return None,
        })
    }
    pub fn instructions(self) -> &'static str {
        match self {
            Self::Codex => "https://github.com/agentclientprotocol/codex-acp",
            Self::Claude => "https://github.com/agentclientprotocol/claude-agent-acp",
            Self::Pi => "https://github.com/svkozak/pi-acp",
            Self::Grok => "https://docs.x.ai/build/overview",
            Self::Gemini => "https://github.com/google-gemini/gemini-cli",
            Self::OpenCode => "https://opencode.ai/docs/acp/",
            Self::Omp => "https://github.com/can1357/oh-my-pi",
            Self::Goose => "https://block.github.io/goose/docs/gdk/acp/",
        }
    }
    fn package(self) -> Option<&'static str> {
        Some(match self {
            Self::Codex => "@agentclientprotocol/codex-acp@2.5.0",
            Self::Claude => "@agentclientprotocol/claude-agent-acp@0.100.0",
            Self::Pi => "pi-acp@0.0.34",
            Self::Grok => "@xai-official/grok@1.0.74",
            Self::Gemini => "@google/gemini-cli@0.66.0",
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Manager {
    Npm,
    Brew,
    Mise,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub agent: Agent,
    pub manager: Manager,
    pub native: bool,
}
impl Recipe {
    pub fn argv(&self) -> Vec<String> {
        match self.manager {
            Manager::Mise => vec![
                "mise".into(),
                "use".into(),
                "--global".into(),
                format!(
                    "npm:{}",
                    if self.native {
                        "@earendil-works/pi-coding-agent@0.81.0"
                    } else {
                        self.agent.package().unwrap_or("")
                    }
                ),
            ],
            Manager::Npm => vec![
                "npm".into(),
                "install".into(),
                "--global".into(),
                if self.native {
                    "@earendil-works/pi-coding-agent@0.81.0".into()
                } else {
                    self.agent.package().unwrap_or("").into()
                },
            ],
            Manager::Brew => vec!["brew".into(), "install".into(), "codex-acp".into()],
        }
    }
    pub fn display(&self) -> String {
        self.argv().join(" ")
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub agent: Agent,
    pub native: bool,
    pub adapter: bool,
    pub profile: bool,
    pub recipes: Vec<Recipe>,
}
impl Entry {
    pub fn status(&self) -> &'static str {
        if self.agent.adapter().is_none() {
            "ACP client tool support unavailable"
        } else if self.adapter {
            if self.agent == Agent::Pi && !self.native {
                "Pi CLI missing"
            } else {
                "ACP adapter available"
            }
        } else if self.native {
            "CLI installed · ACP adapter missing"
        } else {
            "CLI and ACP adapter missing"
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub entries: Vec<Entry>,
    pub installs: BTreeMap<Agent, Install>,
    pub error: Option<String>,
    pub catalogue_revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Install {
    pub generation: u64,
    pub running: bool,
    pub output: String,
    pub result: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Environment {
    pub profiles: PathBuf,
    pub home: PathBuf,
    pub path: OsString,
    pub os: String,
}
impl Environment {
    pub fn host(profiles: PathBuf) -> Option<Self> {
        Some(Self {
            profiles,
            home: std::env::var_os("HOME")?.into(),
            path: std::env::var_os("PATH").unwrap_or_default(),
            os: std::env::consts::OS.into(),
        })
    }
    pub fn executable(&self, name: &str) -> Option<PathBuf> {
        std::env::split_paths(&self.path)
            .filter(|p| p.is_absolute())
            .take(64)
            .map(|p| p.join(name))
            .find(|p| {
                std::fs::metadata(p)
                    .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            })
    }
}
/// Publish a complete file atomically with a create-new hard link. Existing files,
/// directories and symlinks always win; crashes cannot expose a half-written profile.
fn create_profile(env: &Environment, agent: Agent) -> io::Result<()> {
    let Some((cmd, args)) = agent.adapter() else {
        return Ok(());
    };
    std::fs::create_dir_all(&env.profiles)?;
    let dest = env.profiles.join(format!("{}.md", agent.cli()));
    if dest.symlink_metadata().is_ok() {
        return Ok(());
    }
    let temp = env
        .profiles
        .join(format!(".discover-{}", ulid::Ulid::new()));
    let mut command = vec![cmd];
    command.extend_from_slice(args);
    let body = format!(
        "---\nname: {}\nprovider: acp-agent\npolicy: yolo\nagent:\n  command: {}\n---\nEditable local ACP profile. Authentication stays with the agent.\nOfficial instructions: {}\nOrigin: https://github.com/bboxy450/bboxy450-harness\nPolicy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md\n",
        agent.name(),
        serde_json::to_string(&command)?,
        agent.instructions()
    );
    let result = (|| {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all()?;
        match std::fs::hard_link(&temp, &dest) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e),
        }
    })();
    let _ = std::fs::remove_file(temp);
    result
}
pub fn discover(env: &Environment) -> Snapshot {
    let npm = env.executable("npm").is_some();
    let brew = matches!(env.os.as_str(), "macos" | "linux") && env.executable("brew").is_some();
    let mut snapshot = Snapshot::default();
    for agent in AGENTS {
        let native = env.executable(agent.cli()).is_some();
        let adapter = agent
            .adapter()
            .is_some_and(|(cmd, _)| env.executable(cmd).is_some());
        if (native || adapter)
            && let Err(e) = create_profile(env, agent)
        {
            snapshot.error = Some(format!("Could not create profile: {}", e.kind()));
        }
        let mut recipes = vec![];
        if brew && agent == Agent::Codex && !adapter {
            recipes.push(Recipe {
                agent,
                manager: Manager::Brew,
                native: false,
            });
        }
        if npm && agent.package().is_some() && (!adapter || (agent == Agent::Pi && !native)) {
            recipes.push(Recipe {
                agent,
                manager: Manager::Npm,
                native: agent == Agent::Pi && !native,
            });
        }
        // mise's documented npm backend; require its runtime and active shims.
        // No implicit Node.js or package-manager bootstrap.
        let shims = env.home.join(".local/share/mise/shims");
        if env.executable("mise").is_some()
            && env.executable("node").is_some()
            && std::env::split_paths(&env.path).any(|p| p == shims)
            && agent.package().is_some()
            && (!adapter || (agent == Agent::Pi && !native))
        {
            recipes.insert(
                0,
                Recipe {
                    agent,
                    manager: Manager::Mise,
                    native: agent == Agent::Pi && !native,
                },
            );
        }
        snapshot.entries.push(Entry {
            agent,
            native,
            adapter,
            profile: env.profiles.join(format!("{}.md", agent.cli())).is_file(),
            recipes,
        });
    }
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    if let Ok(entries) = std::fs::read_dir(&env.profiles) {
        let mut stamps: Vec<_> = entries
            .take(256)
            .filter_map(Result::ok)
            .filter_map(|e| {
                let path = e.path();
                if path.extension().is_none_or(|s| s != "md") {
                    return None;
                }
                let metadata = e.metadata().ok()?;
                Some((e.file_name(), metadata.len(), metadata.modified().ok()))
            })
            .collect();
        stamps.sort();
        stamps.hash(&mut hash);
    }
    snapshot.catalogue_revision = hash.finish();
    snapshot
}
/// Discover locally before selecting a new session's profile. Runs only on a
/// blocking worker, never a GUI callback. Missing saved preferences stay missing:
/// first-run Codex selection needs no new persistence/ownership protocol.
pub async fn bootstrap(
    profiles: PathBuf,
    explicit: Option<String>,
    discover_profiles: bool,
) -> String {
    let environment = std::env::var("BBOXY450_HARNESS_PROFILE").ok();
    let env = Environment::host(profiles.clone());
    crate::runtime::blocking(move || {
        if discover_profiles && let Some(env) = env {
            discover(&env);
        }
        explicit
            .or_else(|| environment.filter(|p| !p.trim().is_empty()))
            .or_else(|| {
                let path = profiles.parent()?.join("default-profile");
                std::fs::read_to_string(path)
                    .ok()
                    .map(|p| p.trim().to_owned())
                    .filter(|p| !p.is_empty())
            })
            .unwrap_or_else(|| "codex".into())
    })
    .await
}
/// Local setup guidance; no credential commands, installers or vendor calls.
pub fn setup_guidance(profile: &str) -> String {
    format!(
        "Profile {profile:?} is unavailable or its ACP adapter is missing. Open bboxy450-harness gui for setup, install the required adapter on PATH (Codex uses codex-acp), or configure ~/.bboxy450-harness/profiles and select --profile NAME."
    )
}
struct Running {
    generation: u64,
    cancel: CancellationToken,
}
struct State {
    next: u64,
    running: BTreeMap<Agent, Running>,
}
pub struct Controller {
    env: Environment,
    tx: watch::Sender<Arc<Snapshot>>,
    state: Mutex<State>,
    refresh: tokio::sync::Notify,
}
impl Controller {
    pub fn new(env: Environment) -> Arc<Self> {
        let (tx, _) = watch::channel(Arc::new(Snapshot::default()));
        Arc::new(Self {
            env,
            tx,
            state: Mutex::new(State {
                next: 0,
                running: BTreeMap::new(),
            }),
            refresh: tokio::sync::Notify::new(),
        })
    }
    pub fn shared(env: Environment) -> Arc<Self> {
        static VIEWS: OnceLock<Mutex<BTreeMap<PathBuf, Weak<Controller>>>> = OnceLock::new();
        let mut views = VIEWS.get_or_init(Default::default).lock().unwrap();
        views.retain(|_, view| view.strong_count() > 0);
        if let Some(view) = views.get(&env.profiles).and_then(Weak::upgrade) {
            return view;
        }
        let view = Self::new(env);
        views.insert(view.env.profiles.clone(), Arc::downgrade(&view));
        let weak = Arc::downgrade(&view);
        bbt_rt::handle().spawn(async move {
            loop {
                let Some(view) = weak.upgrade() else { break };
                view.refresh_now().await;
                let notify = view.refresh.notified();
                tokio::select! { _=notify=>{}, _=tokio::time::sleep(Duration::from_secs(5))=>{} }
                drop(view);
                if weak.strong_count() == 0 {
                    break;
                }
            }
        });
        view
    }
    pub fn subscribe(&self) -> watch::Receiver<Arc<Snapshot>> {
        self.tx.subscribe()
    }
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.tx.borrow().clone()
    }
    pub async fn refresh_now(&self) {
        let env = self.env.clone();
        let snapshot = crate::runtime::blocking(move || discover(&env)).await;
        self.tx.send_if_modified(|old| {
            let mut next = snapshot.clone();
            next.installs = old.installs.clone();
            if **old == next {
                return false;
            }
            *old = Arc::new(next);
            true
        });
    }
    pub fn cancel(&self, agent: Agent) {
        if let Some(r) = self.state.lock().unwrap().running.get(&agent) {
            r.cancel.cancel();
        }
    }
    /// Only called by a concrete UI action. The snapshot is revalidated against the
    /// static allowlist; strings from profiles, output or the registry cannot execute.
    pub fn install(self: &Arc<Self>, recipe: Recipe) -> Result<(), &'static str> {
        if !self
            .snapshot()
            .entries
            .iter()
            .any(|e| e.agent == recipe.agent && e.recipes.contains(&recipe))
        {
            return Err("Install choice is no longer available");
        }
        let mut state = self.state.lock().unwrap();
        if state.running.contains_key(&recipe.agent) {
            return Err("Installation already in progress");
        }
        state.next += 1;
        let generation = state.next;
        let cancel = CancellationToken::new();
        state.running.insert(
            recipe.agent,
            Running {
                generation,
                cancel: cancel.clone(),
            },
        );
        self.tx.send_modify(|s| {
            Arc::make_mut(s).installs.insert(
                recipe.agent,
                Install {
                    generation,
                    running: true,
                    output: String::new(),
                    result: None,
                },
            );
        });
        let controller = self.clone();
        bbt_rt::handle().spawn(async move {
            let result = controller
                .run_install(&recipe, generation, cancel, Duration::from_secs(180))
                .await;
            controller.finish(recipe.agent, generation, result);
            controller.refresh.notify_one();
        });
        Ok(())
    }
    fn finish(&self, agent: Agent, generation: u64, result: String) {
        let mut state = self.state.lock().unwrap();
        if !state
            .running
            .get(&agent)
            .is_some_and(|r| r.generation == generation)
        {
            return;
        }
        state.running.remove(&agent);
        self.tx.send_modify(|s| {
            if let Some(i) = Arc::make_mut(s).installs.get_mut(&agent)
                && i.generation == generation
            {
                i.running = false;
                i.result = Some(result.clone());
            }
        });
    }
    async fn run_install(
        &self,
        recipe: &Recipe,
        generation: u64,
        cancel: CancellationToken,
        timeout: Duration,
    ) -> String {
        let argv = recipe.argv();
        let env = self.env.clone();
        let name = argv[0].clone();
        let Some(program) = crate::runtime::blocking(move || env.executable(&name)).await else {
            return "Package manager is unavailable; refresh and retry".into();
        };
        if cancel.is_cancelled() {
            return "Installation cancelled".into();
        }
        let mut command = tokio::process::Command::new(program);
        command
            .args(&argv[1..])
            .env_clear()
            .env("HOME", &self.env.home)
            .env("PATH", &self.env.path)
            .env("CI", "1")
            .env("HOMEBREW_NO_AUTO_UPDATE", "1")
            .env("HOMEBREW_NO_INSTALL_CLEANUP", "1")
            .current_dir(&self.env.home)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = match crate::process::OwnedChild::spawn(command) {
            Ok(c) => c,
            Err(e) => return format!("Could not start installer: {}", e.kind()),
        };
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(8);
        async fn drain(
            mut reader: impl tokio::io::AsyncRead + Unpin,
            tx: tokio::sync::mpsc::Sender<Vec<u8>>,
        ) {
            let mut buf = [0; 1024];
            while let Ok(n) = reader.read(&mut buf).await {
                if n == 0 || tx.send(buf[..n].to_vec()).await.is_err() {
                    break;
                }
            }
        }
        let stdout = tokio::spawn(drain(child.stdout().unwrap(), tx.clone()));
        let stderr = tokio::spawn(drain(child.stderr().unwrap(), tx));
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);
        let reason = loop {
            tokio::select! { biased;
                _=cancel.cancelled()=>break Some("Installation cancelled"),
                _=&mut deadline=>break Some("Installation timed out after 180 seconds"),
                Some(bytes)=rx.recv()=>self.tx.send_modify(|s| { if let Some(i)=Arc::make_mut(s).installs.get_mut(&recipe.agent) && i.generation==generation {
                    i.output.extend(String::from_utf8_lossy(&bytes).chars().filter(|c| !c.is_control() || matches!(c,'\n'|'\t')));
                    if i.output.len()>8192 { let mut end=i.output.len()-8192; while !i.output.is_char_boundary(end) { end+=1; } i.output.drain(..end); }
                } }),
                _=tokio::time::sleep(Duration::from_millis(20))=>match child.exited() { Ok(true)=>break None, Err(_)=>break Some("Installer process state unavailable"), _=>{} },
            }
        };
        let status = child.wait().await;
        stdout.abort();
        stderr.abort();
        let _ = stdout.await;
        let _ = stderr.await;
        if let Some(reason) = reason {
            return reason.into();
        }
        match status {
            Ok(status) if status.success() => {
                "Installation complete · refreshing local catalogue".into()
            }
            Ok(status) => format!("Installation failed ({status}); review output and retry"),
            Err(e) => format!("Installer failed: {}", e.kind()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn onboarding_installer_deadline_kills_owned_private_process() {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let manager = bin.join("npm");
        std::fs::write(
            &manager,
            "#!/bin/sh\necho $$ > \"$HOME/pid\"\n/bin/sleep 60 & wait\n",
        )
        .unwrap();
        std::fs::set_permissions(&manager, std::fs::Permissions::from_mode(0o700)).unwrap();
        let c = Controller::new(Environment {
            profiles: root.path().join("profiles"),
            home: root.path().into(),
            path: bin.into_os_string(),
            os: "macos".into(),
        });
        let result = c
            .run_install(
                &Recipe {
                    agent: Agent::Codex,
                    manager: Manager::Npm,
                    native: false,
                },
                1,
                CancellationToken::new(),
                Duration::from_secs(2),
            )
            .await;
        assert!(result.contains("timed out"));
        let pid: i32 = std::fs::read_to_string(root.path().join("pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(unsafe { libc::kill(-pid, 0) }, -1);
    }
    #[test]
    fn onboarding_stale_completion_cannot_settle_replacement() {
        let root = tempfile::tempdir().unwrap();
        let c = Controller::new(Environment {
            profiles: root.path().join("profiles"),
            home: root.path().into(),
            path: OsString::new(),
            os: "macos".into(),
        });
        c.state.lock().unwrap().running.insert(
            Agent::Codex,
            Running {
                generation: 3,
                cancel: CancellationToken::new(),
            },
        );
        c.tx.send_modify(|s| {
            Arc::make_mut(s).installs.insert(
                Agent::Codex,
                Install {
                    generation: 3,
                    running: true,
                    output: String::new(),
                    result: None,
                },
            );
        });
        c.finish(Agent::Codex, 1, "old A completion".into());
        assert!(c.snapshot().installs[&Agent::Codex].running);
        c.finish(Agent::Codex, 3, "current A completion".into());
        assert!(!c.snapshot().installs[&Agent::Codex].running);
    }
}
