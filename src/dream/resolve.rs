// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Exact byte custody and explicit mounted-root boundaries; no filesystem root discovery.
use super::entry::{self, Invalid, Record};
use crate::protocol::*;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, hash_map::RandomState},
    hash::BuildHasher,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

/// Closed host-admitted proof material. Keys retain the exact logical spelling
/// carried by signed records and receipts; values never select another path.
#[derive(Clone, Default)]
pub(crate) struct SelectedBindings {
    pub records: BTreeMap<(String, String, String), String>,
    pub blobs: BTreeMap<(SessionId, String), Vec<u8>>,
    pub artifacts: BTreeMap<PathBuf, Vec<u8>>,
    pub keys: BTreeMap<PathBuf, PathBuf>,
    pub key_bytes: BTreeMap<PathBuf, Vec<u8>>,
    pub prepared_tools: Option<PreparedTools>,
    pub scratch: Option<PathBuf>,
}
#[derive(Clone, Debug)]
pub(crate) struct HelperIdentity {
    pub sequence: u16, pub pid: i32, pub pgid: i32, pub start_identity: u64,
    pub expected_members: u8, pub image: PathBuf, pub descendant_image: Option<PathBuf>,
}
#[derive(Clone, Debug)]
pub(crate) struct NativeCreation {
    pub identity: Option<HelperIdentity>, pub rawerrno: i32, pub sequence: u16, pub cpu_ns: u64, pub wall_ns: u64,
    pub inclusive_cpu_ns: u64, pub inclusive_wall_ns: u64, pub app_cpu_ns: u64, pub credits: u16,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedExecutable { pub absolute_path: PathBuf, pub sha256: String, pub build: String, pub features: Vec<String> }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedOs { pub release: String, pub arch: String }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityReceipt { pub probe_source_sha256: String, pub probe_exe_sha256: String, pub evidence_sha256: String, pub features: Vec<String> }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedTools { pub schema: String, pub source_sha256: String, pub os: PreparedOs, pub git: PreparedExecutable, pub gpg: PreparedExecutable, pub capability: CapabilityReceipt }
/// Exact compiled adapter sources; host provisioning binds this implementation.
pub fn provision_source_sha256() -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new(); hash.update(b"bboxy.dream.proof-tools/source/1\0");
    for (name, bytes) in [("resolve.rs",include_bytes!("resolve.rs").as_slice()),("trust.rs",include_bytes!("trust.rs").as_slice()),("c1.rs",include_bytes!("c1.rs").as_slice()),("read.rs",include_bytes!("read.rs").as_slice())] {
        hash.update(name.as_bytes()); hash.update([0]); hash.update((bytes.len() as u64).to_le_bytes());
        for chunk in bytes.chunks(16*1024) { hash.update(chunk); }
    }
    format!("{:x}",hash.finalize())
}
fn valid_hex(value: &str) -> bool { value.len()==64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) }
fn tool_features(actual: &[String], required: &[&str]) -> bool { actual.len()<=32 && actual.iter().all(|v|!v.is_empty()&&v.len()<=128&&!v.contains('\0')) && required.iter().all(|v|actual.iter().any(|s|s==v)) }
fn current_release() -> Result<String,Invalid> {
    let mut value=std::mem::MaybeUninit::<libc::utsname>::zeroed();
    if unsafe{libc::uname(value.as_mut_ptr())}!=0 {return Err(Invalid("helper host unavailable"));}
    let value=unsafe{value.assume_init()};
    unsafe{std::ffi::CStr::from_ptr(value.release.as_ptr())}.to_str().map(str::to_owned).map_err(|_|Invalid("helper host unavailable"))
}
fn hash_prepared_image(path:&Path)->Result<String,Invalid> {
    use std::{io::Read,os::unix::fs::{MetadataExt,OpenOptionsExt}};
    use sha2::{Digest,Sha256};
    if !path.is_absolute() || path.as_os_str().as_encoded_bytes().len()>4096 || path.components().any(|c|!matches!(c,Component::RootDir|Component::Normal(_))) || path.as_os_str().as_encoded_bytes().iter().any(|b| b.is_ascii_whitespace() || b"'\"`$;|&<>".contains(b)) {return Err(Invalid("helper image path unavailable"));}
    checkpoint()?;
    let mut file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC).open(path).map_err(|_|Invalid("helper image unavailable"))?;
    let before=file.metadata().map_err(|_|Invalid("helper image unavailable"))?;
    if !before.is_file() || before.mode()&0o022!=0 || before.mode()&0o111==0 || before.nlink()!=1 || before.len()>64*1024*1024 || ![0,unsafe{libc::geteuid()}].contains(&before.uid()) {return Err(Invalid("helper image custody unavailable"));}
    let mut hash=Sha256::new();let mut chunk=[0;16*1024];
    loop {checkpoint()?;let n=file.read(&mut chunk).map_err(|_|Invalid("helper image read unavailable"))?;if n==0{break;}hash.update(&chunk[..n]);}
    let after=file.metadata().map_err(|_|Invalid("helper image unavailable"))?;
    if before.dev()!=after.dev()||before.ino()!=after.ino()||before.len()!=after.len()||before.mtime()!=after.mtime()||before.mtime_nsec()!=after.mtime_nsec(){return Err(Invalid("helper image changed"));}
    let digest=format!("{:x}",hash.finalize());
    SELECTED.with(|slot|{let mut slot=slot.borrow_mut();if let Some(owner)=slot.as_mut(){if !owner.tool_guards.iter().any(|g|g.path==path){owner.tool_guards.push(ToolGuard{file,path:path.to_owned(),metadata:after,expected:digest.clone()});}}});
    Ok(digest)
}
pub(crate) fn helper_refusal(reason:&'static str){SELECTED.with(|slot|{if let Some(owner)=slot.borrow_mut().as_mut(){owner.pace.helper_refused(reason);}});}
pub(crate) fn prepared_tools() -> Result<PreparedTools,Invalid> {
    let result=load_prepared_tools();if let Err(error)=&result{helper_refusal(error.0);}result
}
fn load_prepared_tools() -> Result<PreparedTools,Invalid> {
    if let Some(tools)=SELECTED.with(|slot|slot.borrow().as_ref().and_then(|o|o.tools.clone())) {return Ok(tools);}
    use std::{io::Read,os::unix::fs::{MetadataExt,OpenOptionsExt}};
    let path=std::env::var_os("BBOXY_DREAM_HELPER_PROVISION").map(PathBuf::from).ok_or(Invalid("helper provision unavailable"))?;
    let expected=std::env::var("BBOXY_DREAM_HELPER_PROVISION_SHA256").map_err(|_|Invalid("helper provision unavailable"))?;
    if !path.is_absolute()||!valid_hex(&expected){return Err(Invalid("helper provision unavailable"));}
    checkpoint()?;
    let mut file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW|libc::O_NONBLOCK|libc::O_CLOEXEC).open(&path).map_err(|_|Invalid("helper provision unavailable"))?;
    let meta=file.metadata().map_err(|_|Invalid("helper provision unavailable"))?;
    if !meta.is_file()||meta.uid()!=unsafe{libc::geteuid()}||meta.mode()&0o077!=0||meta.nlink()!=1||meta.len()>32*1024{return Err(Invalid("helper provision custody unavailable"));}
    let mut raw=Vec::new();file.by_ref().take(32*1024+1).read_to_end(&mut raw).map_err(|_|Invalid("helper provision unavailable"))?;
    if super::fingerprint::digest(&raw)!=expected{return Err(Invalid("helper provision changed"));}
    let shape=super::c1::storage::preflight(&raw,super::c1::storage::WORK).map_err(|_|Invalid("resource_limit: helper provision bound"))?;
    let _ = shape; // typed strict32KiB DTO is covered by the precharged metadata arena
    SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().tool_guards.push(ToolGuard{file,path:path.clone(),metadata:meta,expected:expected.clone()}));
    let tools:PreparedTools=serde_json::from_slice(&raw).map_err(|_|Invalid("helper provision schema unavailable"))?;
    if tools.schema!="bboxy.dream.proof-tools/1"||tools.source_sha256!=provision_source_sha256()||tools.os.release!=current_release()?||tools.os.arch!=std::env::consts::ARCH||tools.capability.probe_source_sha256!=super::fingerprint::digest(include_bytes!("../../tests/fixtures/dream-read/proof-helper-probe.c"))||![&tools.capability.probe_exe_sha256,&tools.capability.evidence_sha256].into_iter().all(|v|valid_hex(v))||!tool_features(&tools.capability.features,&["darwin-start-suspended","setpgroup","cloexec-default","birth-identity","group-members-2","descendant-cpu-rollup","git-gpg-fixed-argv"]) {return Err(Invalid("helper capability proof unavailable"));}
    for (tool,features) in [(&tools.git,&["git-readonly","openpgp-verify-absolute-gpg"][..]),(&tools.gpg,&["public-key-stdin","no-autostart"][..])] {
        if !valid_hex(&tool.sha256)||tool.build.is_empty()||tool.build.len()>4096||!tool_features(&tool.features,features)||hash_prepared_image(&tool.absolute_path)?!=tool.sha256{return Err(Invalid("helper image proof unavailable"));}
    }
    SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().tools=Some(tools.clone()));
    Ok(tools)
}
/// The caller owns the single low-priority worker and its existing C1 budget.
/// Both edges of every primitive are measured; callback errors stop proof work.
pub(crate) trait ProofPace {
    fn before(&mut self) -> Result<(), Invalid>;
    fn boundary(&mut self) -> Result<(), Invalid> {
        self.after()?;
        self.before()
    }
    fn after(&mut self) -> Result<(), Invalid>;
    fn reserve(&mut self, bytes: u64) -> Result<(), Invalid>;
    /// Register the exact group created by this owner. The existing independent
    /// control owner applies foreground/duty/cancel/deadline while before waits.
    fn helper_attach(&mut self, identity: HelperIdentity) -> Result<(), Invalid>;
    fn native_created(&mut self, span: NativeCreation) -> Result<(), Invalid>;
    fn helper_refused(&mut self, _reason: &'static str) {}
    fn helper_poll(&mut self, pgid: i32) -> Result<(), Invalid>;
    /// Stop controller signals before the pinned leader is reaped, then perform
    /// the last group kill while that leader still prevents pgid reuse.
    fn helper_retire(&mut self, pgid: i32) -> Result<(), Invalid>;
    fn helper_detach(&mut self, pgid: i32) -> Result<(), Invalid>;
    fn helper_usage(&mut self, pgid: i32, cpu_ns: u64, rss_bytes: u64) -> Result<(), Invalid>;
    /// Drain with the C1 cleanup lane even after cancellation or a failed clock.
    fn cleanup(&mut self, step: &mut dyn FnMut() -> bool);
}
struct ToolGuard { file:std::fs::File, path:PathBuf, metadata:std::fs::Metadata, expected:String }
fn same_tool(a:&std::fs::Metadata,b:&std::fs::Metadata)->bool { use std::os::unix::fs::MetadataExt; a.dev()==b.dev()&&a.ino()==b.ino()&&a.len()==b.len()&&a.mtime()==b.mtime()&&a.mtime_nsec()==b.mtime_nsec()&&a.ctime()==b.ctime()&&a.ctime_nsec()==b.ctime_nsec()&&a.nlink()==b.nlink() }
fn check_tool_guards()->Result<(),Invalid> {
    SELECTED.with(|slot|{let slot=slot.borrow();let owner=slot.as_ref().ok_or(Invalid("helper owner unavailable"))?;for guard in &owner.tool_guards {let file=guard.file.metadata().map_err(|_|Invalid("source_changed: helper image"))?;let path=std::fs::symlink_metadata(&guard.path).map_err(|_|Invalid("source_changed: helper image"))?;if !same_tool(&guard.metadata,&file)||!same_tool(&file,&path){return Err(Invalid("source_changed: helper image/provision"));}}Ok(())})
}
pub(crate) fn revalidate_prepared_tools()->Result<(),Invalid> {
    check_tool_guards()?;
    let tools=SELECTED.with(|slot|slot.borrow().as_ref().and_then(|owner|owner.tools.clone()));
    if let Some(tools)=tools {
        for image in [&tools.git,&tools.gpg] { if hash_prepared_image(&image.absolute_path)?!=image.sha256{return Err(Invalid("source_changed: helper image"));} }
        let provision=SELECTED.with(|slot|slot.borrow().as_ref().and_then(|owner|owner.tool_guards.first().map(|g|(g.path.clone(),g.expected.clone()))));
        if let Some((path,expected))=provision {use std::io::Read;let mut file=std::fs::File::open(path).map_err(|_|Invalid("source_changed: helper provision"))?;let mut bytes=Vec::new();file.take(32*1024+1).read_to_end(&mut bytes).map_err(|_|Invalid("source_changed: helper provision"))?;if super::fingerprint::digest(&bytes)!=expected{return Err(Invalid("source_changed: helper provision"));}}
    }
    check_tool_guards()
}
pub(crate) fn admit_policy_branches(branches:impl IntoIterator<Item=String>)->Result<(),Invalid>{SELECTED.with(|slot|{let mut slot=slot.borrow_mut();let owner=slot.as_mut().ok_or(Invalid("helper owner unavailable"))?;for branch in branches{if owner.policy_branches.len()>=32{return Err(Invalid("helper policy branch bound"));}owner.policy_branches.insert(branch);}Ok(())})}
struct SelectedOwner {
    bindings: Arc<SelectedBindings>,
    pace: Box<dyn ProofPace>,
    git_cache: BTreeMap<(PathBuf, Vec<String>), Vec<u8>>,
    git_cache_bytes: usize,
    trust_cache: BTreeSet<String>,
    failed_trace: bool,
    helper_profile: HelperProfile,
    tools: Option<PreparedTools>,
    attempts: u16,
    tool_guards: Vec<ToolGuard>,
    policy_branches: BTreeSet<String>,
    helper_active: bool,
}
thread_local! {
    static SELECTED: std::cell::RefCell<Option<SelectedOwner>> = const { std::cell::RefCell::new(None) };
}
pub(crate) struct SelectedScope;
impl SelectedScope {
    pub(crate) fn enter(
        bindings: Arc<SelectedBindings>,
        pace: Box<dyn ProofPace>,
    ) -> Result<Self, Invalid> {
        SELECTED.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err(Invalid("selected proof owner already installed"));
            }
            *slot = Some(SelectedOwner {
                bindings,
                pace,
                git_cache: BTreeMap::new(),
                git_cache_bytes: 0,
                trust_cache: BTreeSet::new(),
                failed_trace: false,
                helper_profile: HelperProfile::default(),
                tools: None,
                attempts: 0,
                tool_guards: vec![],
                policy_branches: BTreeSet::new(),
                helper_active: false,
            });
            Ok(Self)
        })
    }
}
impl Drop for SelectedScope {
    fn drop(&mut self) {
        if let Err(_) = super::trust::cleanup_selected_scratch() { eprintln!("dream-read helper_scratch_cleanup=unproven"); }
        let owner = SELECTED.with(|slot| slot.borrow_mut().take());
        if let Some(mut owner) = owner {
            owner.pace.cleanup(&mut || {
                owner.git_cache.pop_first().is_some() || owner.trust_cache.pop_first().is_some() || owner.tool_guards.pop().is_some() || owner.policy_branches.pop_first().is_some()
            });
            owner.helper_profile.emit();
        }
    }
}
pub(crate) fn helper_active()->bool { SELECTED.with(|slot|slot.borrow().as_ref().is_some_and(|o|o.helper_active)) }
pub(crate) fn selected() -> bool {
    SELECTED.with(|slot| slot.borrow().is_some())
}
pub(crate) fn bindings() -> Option<Arc<SelectedBindings>> {
    SELECTED.with(|slot| slot.borrow().as_ref().map(|owner| owner.bindings.clone()))
}
#[track_caller]
pub(crate) fn checkpoint() -> Result<(), Invalid> {
    let location = std::panic::Location::caller();
    SELECTED.with(|slot| {
        if let Some(owner) = slot.borrow_mut().as_mut() {
            let result = owner.pace.after().and_then(|()| owner.pace.before());
            if result.is_err() && !owner.failed_trace {
                owner.failed_trace = true;
                eprintln!(
                    "dream-read proof_stopped source={}:{}",
                    location.file(),
                    location.line()
                );
            }
            result
        } else {
            Ok(())
        }
    })
}

/// Reserve declared storage before YAML, JSON-front or graph owners are built.
/// The caller retains these credits beyond scope through returned lesson disposal.
pub(crate) fn reserve_record(bytes: &[u8]) -> Result<(), Invalid> {
    if !selected() {
        return Ok(());
    }
    checkpoint()?;
    let source = std::str::from_utf8(bytes).map_err(|_| Invalid("record is not UTF-8"))?;
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let after = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
        .ok_or(Invalid("missing record frontmatter"))?;
    let mut length = 0usize;
    let mut lines = 0usize;
    let mut flow = 0usize;
    let mut closed = false;
    for line in after.split_inclusive('\n') {
        checkpoint()?;
        if matches!(line.trim_end_matches(['\r', '\n']), "---" | "...") {
            closed = true;
            break;
        }
        length = length
            .checked_add(line.len())
            .ok_or(Invalid("selected record storage bound"))?;
        lines += 1;
        if length > 64 * 1024
            || lines > 2048
            || line.bytes().take_while(|b| *b == b' ').count() > 64
        {
            return Err(Invalid("selected YAML bound"));
        }
        let mut quoted = None;
        let mut escaped = false;
        for byte in line.bytes() {
            if let Some(quote) = quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' && quote == b'"' {
                    escaped = true;
                } else if byte == quote {
                    quoted = None;
                }
                continue;
            }
            match byte {
                b'"' | b'\'' => quoted = Some(byte),
                b'#' => break,
                b'&' | b'*' | b'!' => {
                    return Err(Invalid("selected YAML alias or tag unsupported"));
                }
                b'[' | b'{' => {
                    flow += 1;
                    if flow > 32 {
                        return Err(Invalid("selected YAML depth bound"));
                    }
                }
                b']' | b'}' => flow = flow.saturating_sub(1),
                _ => {}
            }
        }
    }
    if !closed {
        return Err(Invalid("incomplete record frontmatter"));
    }
    let declared = (length as u64)
        .checked_mul(512)
        .and_then(|n| n.checked_add((bytes.len() as u64).checked_mul(8)?))
        .ok_or(Invalid("selected record storage bound"))?;
    SELECTED.with(|slot| {
        slot.borrow_mut()
            .as_mut()
            .ok_or(Invalid("selected proof owner unavailable"))?
            .pace
            .reserve(declared)
    })
}
pub(crate) fn assert_proof_owner() {
    if !selected() {
        crate::runtime::assert_off_rt_worker();
    }
}
pub(crate) async fn proof_blocking<T: super::c1::disposal::Drain + Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, Invalid> {
    if selected() {
        SELECTED.with(|slot| {
            slot.borrow_mut()
                .as_mut()
                .ok_or(Invalid("selected proof owner unavailable"))?
                .pace
                .boundary()
        })?;
        let result = f();
        if let Err(error) = checkpoint() {
            drop(ProofOwned::new(result));
            return Err(error);
        }
        Ok(result)
    } else {
        Ok(crate::runtime::blocking(f).await)
    }
}
/// Hidden temporary proof owners use the same stepped cleanup lane as results.
pub(crate) struct ProofOwned<T: super::c1::disposal::Drain>(Option<T>);
impl<T: super::c1::disposal::Drain> ProofOwned<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Some(value))
    }
    pub(crate) fn take_value(&mut self) -> T {
        self.0.take().unwrap()
    }
}
impl<T: super::c1::disposal::Drain> std::ops::Deref for ProofOwned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}
impl<T: super::c1::disposal::Drain> std::ops::DerefMut for ProofOwned<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}
impl<T: super::c1::disposal::Drain> Drop for ProofOwned<T> {
    fn drop(&mut self) {
        if self.0.is_none() {
            return;
        }
        SELECTED.with(|slot| {
            let mut slot = slot.borrow_mut();
            if let Some(owner) = slot.as_mut() {
                owner.pace.cleanup(&mut || {
                    let Some(value) = self.0.as_mut() else {
                        return false;
                    };
                    if !value.drain_step() {
                        self.0.take();
                        return false;
                    }
                    true
                });
            }
        });
    }
}
pub(crate) fn cleanup_steps(step: &mut dyn FnMut() -> bool) {
    SELECTED.with(|slot| {
        if let Some(owner) = slot.borrow_mut().as_mut() {
            owner.pace.cleanup(step);
        } else {
            while step() {}
        }
    });
}
fn helper_hook(pgid: i32, action: u8) -> Result<(), Invalid> {
    SELECTED.with(|slot| {
        let mut slot = slot.borrow_mut();
        let owner = slot
            .as_mut()
            .ok_or(Invalid("selected helper owner unavailable"))?;
        match action {
            0 => Err(Invalid("helper identity required")),
            1 => owner.pace.helper_poll(pgid),
            2 => owner.pace.helper_retire(pgid),
            _ => owner.pace.helper_detach(pgid),
        }
    })
}
/// Observe completion without reaping. The direct leader pins its process-group
/// identity until controller retirement and the final group signal have finished.
fn helper_exited(pid: i32) -> Result<bool, Invalid> {
    let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
    let code = unsafe {
        libc::waitid(
            libc::P_PID,
            pid as libc::id_t,
            info.as_mut_ptr(),
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    };
    if code != 0 {
        if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            return Ok(false);
        }
        return Err(Invalid("selected helper observation unavailable"));
    }
    let info = unsafe { info.assume_init() };
    #[cfg(target_os = "macos")]
    let observed = info.si_pid;
    #[cfg(not(target_os = "macos"))]
    let observed = unsafe { info.si_pid() };
    Ok(observed == pid
        && matches!(
            info.si_code,
            libc::CLD_EXITED | libc::CLD_KILLED | libc::CLD_DUMPED
        ))
}
pub(crate) fn trust_cache_has(key: &str) -> bool {
    SELECTED.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|owner| owner.trust_cache.contains(key))
    })
}
pub(crate) fn trust_cache_put(key: String) {
    SELECTED.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(owner) = slot.as_mut() else {
            return;
        };
        if key.len() <= 8192
            && owner.trust_cache.len() < 640
            && owner.git_cache_bytes.saturating_add(key.len()) <= 8 * 1024 * 1024
        {
            owner.git_cache_bytes += key.len();
            owner.trust_cache.insert(key);
        }
    });
}
fn pure_git(program: &str, args: &[String], home: Option<&Path>) -> bool {
    if program != "git" || home.is_some() {
        return false;
    }
    let mut at = 0;
    while let Some(arg) = args.get(at) {
        if arg == "-c" {
            at += 2;
        } else if arg.starts_with('-') {
            at += 1;
        } else {
            break;
        }
    }
    match args.get(at).map(String::as_str) {
        Some("rev-parse" | "cat-file" | "ls-tree" | "show" | "merge-base" | "rev-list") => true,
        Some("config") => {
            args.get(at + 1).map(String::as_str) == Some("--get") && args.len() == at + 3
        }
        _ => false,
    }
}
fn git_cache_get(cwd: &Path, args: &[String]) -> Option<Vec<u8>> {
    SELECTED.with(|slot| {
        slot.borrow()
            .as_ref()?
            .git_cache
            .get(&(cwd.to_owned(), args.to_vec()))
            .cloned()
    })
}
fn git_cache_put(cwd: &Path, args: &[String], bytes: &[u8]) {
    const CACHE: usize = 8 * 1024 * 1024;
    SELECTED.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(owner) = slot.as_mut() else {
            return;
        };
        let key_bytes = cwd
            .as_os_str()
            .as_encoded_bytes()
            .len()
            .saturating_add(args.iter().map(String::len).sum::<usize>());
        let size = bytes.len().saturating_add(key_bytes);
        if owner.git_cache.len() >= 256 || owner.git_cache_bytes.saturating_add(size) > CACHE {
            return;
        }
        owner
            .git_cache
            .insert((cwd.to_owned(), args.to_vec()), bytes.to_vec());
        owner.git_cache_bytes += size;
    });
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
enum HelperPhase { Prepare, Spawn, RetainStop, Postspawn, FirstPoll, RawInclusive }
const HELPER_PHASE_NAMES: [&str; 6] = ["prepare", "exact_spawn", "retain_stop", "postspawn", "first_poll", "raw_inclusive"];
#[derive(Clone, Copy, Default)]
struct HelperSample { count: u64, cpu_total: u64, cpu_max: u64, wall_total: u64, wall_max: u64, unknown: u64 }
#[derive(Default)]
struct HelperProfile { phases: [HelperSample; 6] }
#[derive(Clone, Copy)]
struct HelperStamp { cpu: u64, wall: std::time::Instant }
#[cfg(test)]
thread_local! { static FAIL_HELPER_CLOCK: std::cell::Cell<Option<HelperPhase>> = const { std::cell::Cell::new(None) }; }
fn helper_stamp(phase: HelperPhase, end: bool) -> HelperStamp {
    let cpu = super::c1::cpu_ns();
    #[cfg(test)]
    let cpu = FAIL_HELPER_CLOCK.with(|failure| {
        if end && failure.get() == Some(phase) { failure.set(None); u64::MAX } else { cpu }
    });
    #[cfg(not(test))]
    let _ = (phase, end);
    HelperStamp { cpu, wall: std::time::Instant::now() }
}
impl HelperProfile {
    fn record(&mut self, phase: HelperPhase, start: HelperStamp, end: HelperStamp) -> Result<(), Invalid> {
        let sample = &mut self.phases[phase as usize];
        sample.count = sample.count.saturating_add(1);
        let wall = u64::try_from(end.wall.saturating_duration_since(start.wall).as_nanos()).unwrap_or(u64::MAX);
        sample.wall_total = sample.wall_total.saturating_add(wall); sample.wall_max = sample.wall_max.max(wall);
        let cpu = if start.cpu == u64::MAX || end.cpu == u64::MAX { None } else { end.cpu.checked_sub(start.cpu) };
        let Some(cpu) = cpu else { sample.unknown = sample.unknown.saturating_add(1); return Err(Invalid("selected helper phase clock unavailable")); };
        sample.cpu_total = sample.cpu_total.saturating_add(cpu); sample.cpu_max = sample.cpu_max.max(cpu);
        // Raw inclusive is an observation across explicit waits, never a unit
        // exemption. Every actual ordinary phase and combined spawn+stop still
        // passes the unchanged1ms C1 clock boundary.
        let limit = if phase == HelperPhase::Spawn { 2_000_000 } else { 1_000_000 };
        if phase != HelperPhase::RawInclusive && cpu > limit { return Err(Invalid("resource_limit: selected helper phase CPU bound")); }
        if phase == HelperPhase::Spawn && wall > 25_000_000 { return Err(Invalid("resource_limit: selected helper native wall bound")); }
        Ok(())
    }
    fn emit(&self) {
        for (index, sample) in self.phases.iter().enumerate() {
            if sample.count == 0 { continue; }
            eprintln!("dream-read helper_phase={} count={} cpu_total_ns={} cpu_max_ns={} wall_total_ns={} wall_max_ns={} unknown_clocks={}", HELPER_PHASE_NAMES[index], sample.count, sample.cpu_total, sample.cpu_max, sample.wall_total, sample.wall_max, sample.unknown);
        }
    }
}
fn helper_record(phase: HelperPhase, start: HelperStamp, end: HelperStamp) -> Result<(), Invalid> {
    SELECTED.with(|slot| slot.borrow_mut().as_mut().ok_or(Invalid("selected helper owner unavailable"))?.helper_profile.record(phase, start, end))
}
fn helper_boundary() -> Result<(), Invalid> {
    SELECTED.with(|slot| slot.borrow_mut().as_mut().ok_or(Invalid("selected helper owner unavailable"))?.pace.boundary())
}
fn helper_after() -> Result<(), Invalid> {
    SELECTED.with(|slot| slot.borrow_mut().as_mut().ok_or(Invalid("selected helper owner unavailable"))?.pace.after())
}
struct ToolRequest { image: PathBuf, args: Vec<String>, expected_members: u8, descendant: Option<PathBuf> }
fn git_object(value:&str)->bool { entry::valid_commit(value) || value.strip_suffix("^{tree}").is_some_and(entry::valid_commit) || value.split_once(':').is_some_and(|(commit,path)|entry::valid_commit(commit)&&repository_path(path).is_ok()) }
fn git_ref(value:&str)->bool { value=="refs/remotes/origin/main" || SELECTED.with(|slot|slot.borrow().as_ref().is_some_and(|owner|owner.policy_branches.contains(value))) }

fn admitted_request(program:&str,args:&[String],home:Option<&Path>,input:Option<&[u8]>)->Result<ToolRequest,Invalid> {
    #[cfg(test)]
    if program=="/bin/sh" {return Ok(ToolRequest{image:PathBuf::from(program),args:args.to_vec(),expected_members:1,descendant:None});}
    let tools=prepared_tools()?;
    if args.len()>63 || args.iter().try_fold(0usize,|n,v|n.checked_add(v.len()+1)).is_none_or(|n|n>32*1024){return Err(Invalid("resource_limit: helper argv bound"));}
    let mut args=args.to_vec();
    if program=="gpg" || Path::new(program)==tools.gpg.absolute_path {
        let action=args.iter().position(|v|matches!(v.as_str(),"--show-keys"|"--import")).ok_or(Invalid("helper argument unavailable"))?;
        if ["--no-options","--no-autostart","--batch","--no-tty"].iter().any(|required|args[..action].iter().filter(|v|v==required).count()!=1)||input.is_none()||home.is_none()||action+2!=args.len()||args.last().map(String::as_str)!=Some("-")||args[..action].iter().any(|v|!matches!(v.as_str(),"--no-options"|"--no-autostart"|"--batch"|"--no-tty"|"--with-colons")){return Err(Invalid("helper argument unavailable"));}
        return Ok(ToolRequest{image:tools.gpg.absolute_path,args,expected_members:1,descendant:None});
    }
    if program!="git"&&Path::new(program)!=tools.git.absolute_path||input.is_some(){return Err(Invalid("helper image not approved"));}
    let mut at=0;
    while let Some(arg)=args.get(at) {
        if matches!(arg.as_str(),"--no-pager"|"--no-optional-locks"){at+=1;continue;}
        if arg!="-c"{break;}
        let value=args.get_mut(at+1).ok_or(Invalid("helper argument unavailable"))?;
        if value=="gpg.program=gpg"||value=="gpg.openpgp.program=gpg" {
            let key=value.split('=').next().unwrap();*value=format!("{key}={}",tools.gpg.absolute_path.display());
        } else if !matches!(value.as_str(),"core.fsmonitor=false"|"core.hooksPath=/dev/null"|"credential.helper="|"protocol.allow=never"|"gpg.format=openpgp") && *value!=format!("gpg.program={}",tools.gpg.absolute_path.display()) && *value!=format!("gpg.openpgp.program={}",tools.gpg.absolute_path.display()) {return Err(Invalid("helper Git configuration unavailable"));}
        at+=2;
    }
    if ["--no-pager","--no-optional-locks","core.fsmonitor=false","core.hooksPath=/dev/null","credential.helper=","protocol.allow=never"].iter().any(|required|args[..at].iter().filter(|v|v==required).count()!=1){return Err(Invalid("helper isolation flags unavailable"));}
    let rest=&args[at..];let words=rest.iter().map(String::as_str).collect::<Vec<_>>();
    let signature=matches!(words.as_slice(),["log","-1","--format=%G?%x00%GF%x00%GP",commit] if entry::valid_commit(commit));
    let accepted=signature || match words.as_slice() {
        ["rev-parse","--show-toplevel"|"--is-bare-repository"]=>true,
        ["rev-parse","--verify",reference]=>git_ref(reference),
        ["rev-parse",object]=>git_object(object),
        ["cat-file","-t",commit]=>entry::valid_commit(commit),
        ["merge-base","--is-ancestor",commit,main]=>entry::valid_commit(commit)&&entry::valid_commit(main),
        ["ls-tree","-z",commit,"--",path]=>entry::valid_commit(commit)&&repository_path(path.trim_end_matches('/')).is_ok(),
        ["show",object]=>git_object(object)&&object.contains(':'),
        ["rev-list","-1",commit,"--",path]=>entry::valid_commit(commit)&&repository_path(path).is_ok(),
        ["config","--get","remote.origin.url"]=>true,
        _=>false,
    };
    if !accepted||signature&&home.is_none(){return Err(Invalid("helper Git command unavailable"));}
    if signature {
        for key in ["gpg.program","gpg.openpgp.program"] {
            if !args[..at].iter().any(|v|v==&format!("{key}={}",tools.gpg.absolute_path.display())){return Err(Invalid("helper fixed signature verifier unavailable"));}
        }
    }
    Ok(ToolRequest{image:tools.git.absolute_path,args,expected_members:if signature{2}else{1},descendant:signature.then_some(tools.gpg.absolute_path)})
}
pub(crate) fn helper_creation_reserve(cost:u16)->Result<u16,Invalid> {
    SELECTED.with(|slot| {
        let mut slot=slot.borrow_mut();let owner=slot.as_mut().ok_or(Invalid("helper owner unavailable"))?;
        let next=owner.attempts.checked_add(cost).filter(|next|*next<=256).ok_or(Invalid("resource_limit: helper creation cap"))?;
        owner.attempts=next;Ok(next)
    })
}
#[cfg(target_os="macos")]
struct NativeSpawn {
    image:std::ffi::CString, args:Vec<std::ffi::CString>, env:Vec<std::ffi::CString>,
    argv:Vec<*mut libc::c_char>, envp:Vec<*mut libc::c_char>,
    actions:libc::posix_spawn_file_actions_t, attr:libc::posix_spawnattr_t,
    actions_live:bool, attr_live:bool, input:Option<(std::fs::File,std::fs::File)>, output:Option<(std::fs::File,std::fs::File)>,
}
#[cfg(target_os="macos")]
impl Drop for NativeSpawn {fn drop(&mut self){unsafe{if self.actions_live{libc::posix_spawn_file_actions_destroy(&mut self.actions);}if self.attr_live{libc::posix_spawnattr_destroy(&mut self.attr);}}}}
#[cfg(target_os="macos")]
fn native_pipe()->Result<(std::fs::File,std::fs::File),Invalid>{
    use std::os::fd::{FromRawFd,AsRawFd};let mut descriptors=[-1;2];
    if unsafe{libc::pipe(descriptors.as_mut_ptr())}!=0{return Err(Invalid("helper pipe unavailable"));}
    let a=unsafe{std::fs::File::from_raw_fd(descriptors[0])};let b=unsafe{std::fs::File::from_raw_fd(descriptors[1])};
    for file in [&a,&b]{if unsafe{libc::fcntl(file.as_raw_fd(),libc::F_SETFD,libc::FD_CLOEXEC)}<0{return Err(Invalid("helper pipe unavailable"));}}
    Ok((a,b))
}
#[cfg(target_os="macos")]
impl NativeSpawn {
    fn prepare(request:&ToolRequest,cwd:&Path,home:Option<&Path>,input:bool)->Result<Self,Invalid>{
        use std::os::fd::AsRawFd;
        let mut environment=vec!["GIT_CONFIG_NOSYSTEM=1".to_owned(),"GIT_CONFIG_GLOBAL=/dev/null".into(),"GIT_TERMINAL_PROMPT=0".into(),"GIT_ASKPASS=/usr/bin/false".into(),"SSH_ASKPASS=/usr/bin/false".into(),"LC_ALL=C".into(),"LANG=C".into()];
        let scratch=bindings().and_then(|bindings|bindings.scratch.clone()).ok_or(Invalid("helper private scratch unavailable"))?;
        let private=&scratch;
        environment.push(format!("HOME={}",private.display()));environment.push(format!("TMPDIR={}",private.display()));
        if let Some(home)=home{environment.push(format!("GNUPGHOME={}",home.display()));}
        if request.args.len()+1>64||environment.len()>16||request.args.iter().map(String::len).sum::<usize>()+environment.iter().map(String::len).sum::<usize>()+request.image.as_os_str().as_encoded_bytes().len()>32*1024{return Err(Invalid("resource_limit: helper argv environment bound"));}
        let image=std::ffi::CString::new(request.image.as_os_str().as_encoded_bytes()).map_err(|_|Invalid("helper image unavailable"))?;
        let mut args=vec![image.clone()];for value in &request.args{args.push(std::ffi::CString::new(value.as_bytes()).map_err(|_|Invalid("helper argv unavailable"))?);}
        let env=environment.iter().map(|s|std::ffi::CString::new(s.as_bytes()).map_err(|_|Invalid("helper environment unavailable"))).collect::<Result<Vec<_>,_>>()?;
        let mut prepared=Self{image,args,env,argv:vec![],envp:vec![],actions:unsafe{std::mem::zeroed()},attr:unsafe{std::mem::zeroed()},actions_live:false,attr_live:false,input:if input{Some(native_pipe()?)}else{None},output:Some(native_pipe()?)};
        if unsafe{libc::posix_spawn_file_actions_init(&mut prepared.actions)}!=0{return Err(Invalid("helper spawn preparation unavailable"));}prepared.actions_live=true;
        if unsafe{libc::posix_spawnattr_init(&mut prepared.attr)}!=0{return Err(Invalid("helper spawn preparation unavailable"));}prepared.attr_live=true;
        let cwd=std::ffi::CString::new(cwd.as_os_str().as_encoded_bytes()).map_err(|_|Invalid("helper cwd unavailable"))?;
        let null=c"/dev/null";
        let mut codes=vec![unsafe{libc::posix_spawnattr_setpgroup(&mut prepared.attr,0)},unsafe{libc::posix_spawnattr_setflags(&mut prepared.attr,(libc::POSIX_SPAWN_START_SUSPENDED|libc::POSIX_SPAWN_SETPGROUP|libc::POSIX_SPAWN_CLOEXEC_DEFAULT) as i16)},unsafe{libc::posix_spawn_file_actions_addchdir_np(&mut prepared.actions,cwd.as_ptr())},unsafe{libc::posix_spawn_file_actions_adddup2(&mut prepared.actions,prepared.output.as_ref().unwrap().1.as_raw_fd(),1)},unsafe{libc::posix_spawn_file_actions_addopen(&mut prepared.actions,2,null.as_ptr(),libc::O_WRONLY,0)}];
        if let Some((read,_))=&prepared.input{codes.push(unsafe{libc::posix_spawn_file_actions_adddup2(&mut prepared.actions,read.as_raw_fd(),0)});}else{codes.push(unsafe{libc::posix_spawn_file_actions_addopen(&mut prepared.actions,0,null.as_ptr(),libc::O_RDONLY,0)});}
        if codes.iter().any(|code|*code!=0){return Err(Invalid("helper spawn preparation unavailable"));}
        prepared.argv=prepared.args.iter().map(|s|s.as_ptr().cast_mut()).chain(std::iter::once(std::ptr::null_mut())).collect();
        prepared.envp=prepared.env.iter().map(|s|s.as_ptr().cast_mut()).chain(std::iter::once(std::ptr::null_mut())).collect();
        Ok(prepared)
    }
    fn create(&self,pid:&mut i32)->i32 {unsafe{libc::posix_spawn(pid,self.image.as_ptr(),&self.actions,&self.attr,self.argv.as_ptr(),self.envp.as_ptr())}}
}
#[cfg(target_os="macos")]
impl super::c1::disposal::Drain for NativeSpawn {
    fn drain_step(&mut self)->bool { self.args.pop().is_some() || self.env.pop().is_some() || self.argv.pop().is_some() || self.envp.pop().is_some() }
}
fn helper_birth(pid:i32)->Result<u64,Invalid>{
    #[cfg(target_os="macos")]{let mut info=std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();let size=std::mem::size_of::<libc::proc_bsdinfo>();if unsafe{libc::proc_pidinfo(pid,libc::PROC_PIDTBSDINFO,0,info.as_mut_ptr().cast(),size as i32)}!=size as i32{return Err(Invalid("helper birth unavailable"));}let info=unsafe{info.assume_init()};if info.pbi_pid!=pid as u32||info.pbi_pgid!=pid as u32{return Err(Invalid("helper group unavailable"));}info.pbi_start_tvsec.checked_mul(1_000_000).and_then(|v|v.checked_add(info.pbi_start_tvusec)).ok_or(Invalid("helper birth unavailable"))}
    #[cfg(not(target_os="macos"))]{let _=pid;Err(Invalid("prepared helper unavailable on host"))}
}
fn span_cpu(start:HelperStamp,end:HelperStamp)->u64{if start.cpu==u64::MAX||end.cpu==u64::MAX{u64::MAX}else{end.cpu.checked_sub(start.cpu).unwrap_or(u64::MAX)}}
fn span_wall(start:HelperStamp,end:HelperStamp)->u64{u64::try_from(end.wall.saturating_duration_since(start.wall).as_nanos()).unwrap_or(u64::MAX)}
fn helper_catch<T>(f: impl FnOnce() -> Result<T, Invalid>) -> Result<T, Invalid> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or(Err(Invalid("selected helper owner panicked")))
}
/// One exact owned process group, finite pre-reserved pipe storage and actual
/// direct+descendant settlement on every path. No runtime or general helper pool.
pub(crate) fn selected_command(
    cwd: &Path,
    program: &str,
    args: &[String],
    home: Option<&Path>,
) -> Result<(i32, Vec<u8>), Invalid> {
    selected_command_input(cwd, program, args, home, None)
}
pub(crate) fn selected_command_input(
    cwd: &Path,
    program: &str,
    args: &[String],
    home: Option<&Path>,
    input: Option<&[u8]>,
) -> Result<(i32, Vec<u8>), Invalid> {
    let result=selected_command_input_owned(cwd,program,args,home,input);
    if let Err(error)=&result { helper_refusal(error.0); }
    result
}
fn selected_command_input_owned(
    cwd:&Path,program:&str,args:&[String],home:Option<&Path>,input:Option<&[u8]>,
)->Result<(i32,Vec<u8>),Invalid>{
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    #[cfg(not(target_os="macos"))]
    { let _=(cwd,program,args,home,input);return Err(Invalid("prepared native helpers unavailable on host")); }
    #[cfg(target_os="macos")]
    let (mut prepared,request,sequence,cost,inclusive,spawn,spawn_end,pid,native_code) = {
        if input.is_some_and(|v|v.len()>2*1024*1024){return Err(Invalid("resource_limit: helper input bound"));}
        let request=admitted_request(program,args,home,input)?;
        helper_boundary()?;
        let inclusive=helper_stamp(HelperPhase::RawInclusive,false);
        let prepare=helper_stamp(HelperPhase::Prepare,false);
        let prepared=ProofOwned::new(NativeSpawn::prepare(&request,cwd,home,input.is_some())?);
        let cost=request.expected_members as u16;
        let sequence=helper_creation_reserve(cost)?;
        check_tool_guards()?;
        helper_record(HelperPhase::Prepare,prepare,helper_stamp(HelperPhase::Prepare,true))?;
        helper_after()?; // ordinary proof clock PAUSED before native purpose span
        let sequence=sequence; // admitted before effect, never refunded
        let inclusive=HelperStamp{cpu:super::c1::cpu_ns(),wall:std::time::Instant::now()};
        let spawn=helper_stamp(HelperPhase::Spawn,false);
        let mut pid=0;
        let native_code=prepared.create(&mut pid); // exact prepared native call; no application wrapper
        let spawn_end=helper_stamp(HelperPhase::Spawn,true);
        (prepared,request,sequence,cost,inclusive,spawn,spawn_end,pid,native_code)
    };
    #[cfg(target_os="macos")]
    let pgid=pid; // kernel START_SUSPENDED retains the typed positive effect before verdict
    #[cfg(target_os="macos")]
    if pid<=0 {
        let raw=helper_stamp(HelperPhase::RawInclusive,true);
        let cpu=span_cpu(spawn,spawn_end);let total=span_cpu(inclusive,raw);
        let span=NativeCreation{identity:None,rawerrno:native_code,sequence,cpu_ns:cpu,wall_ns:span_wall(spawn,spawn_end),inclusive_cpu_ns:total,inclusive_wall_ns:span_wall(inclusive,raw),app_cpu_ns:total.checked_sub(cpu).unwrap_or(u64::MAX),credits:cost};
        let _=helper_record(HelperPhase::Spawn,spawn,spawn_end);let _=helper_record(HelperPhase::RawInclusive,inclusive,raw);
        SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().pace.native_created(span))?;
        return Err(Invalid("prepared helper native creation failed"));
    }
    SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().helper_active=true);
    const OUTPUT:usize=2*1024*1024;
    let mut bytes=Vec::new(); // capture capacity was admitted before creation by host
    let mut pipe=None;
    let mut stdin=None;
    let mut sent=0usize;
    let mut chunk=[0;4096];
    let mut inclusive_recorded=false;
    let mut first_poll=true;
    let observed=helper_catch(|| {
        let app=spawn_end;
        let birth=helper_birth(pgid); // PID remains retained and kernel-suspended even on failure
        let identity=HelperIdentity{sequence,pid,pgid,start_identity:birth.clone().unwrap_or(0),expected_members:request.expected_members,image:request.image.clone(),descendant_image:request.descendant.clone()};
        let attached=SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().pace.helper_attach(identity.clone()));
        if let Some((read,write))=prepared.output.take(){pipe=Some(read);drop(write);}
        if let Some((read,write))=prepared.input.take(){stdin=Some(write);drop(read);}
        for fd in pipe.iter().chain(stdin.iter()) {
            let flags=unsafe{libc::fcntl(fd.as_raw_fd(),libc::F_GETFL)};
            if flags<0||unsafe{libc::fcntl(fd.as_raw_fd(),libc::F_SETFL,flags|libc::O_NONBLOCK)}<0{return Err(Invalid("helper pipe setup unavailable"));}
        }
        bytes.try_reserve_exact(OUTPUT).map_err(|_|Invalid("resource_limit: helper capture reservation"))?;
        let end=helper_stamp(HelperPhase::RawInclusive,true);
        let native_cpu=span_cpu(spawn,spawn_end);let total=span_cpu(inclusive,end);
        let native=NativeCreation{identity:Some(identity),rawerrno:native_code,sequence,cpu_ns:native_cpu,wall_ns:span_wall(spawn,spawn_end),inclusive_cpu_ns:total,inclusive_wall_ns:span_wall(inclusive,end),app_cpu_ns:total.checked_sub(native_cpu).unwrap_or(u64::MAX),credits:cost};
        let exact=helper_record(HelperPhase::Spawn,spawn,spawn_end);
        let app_profile=helper_record(HelperPhase::Postspawn,app,end);
        let raw=helper_record(HelperPhase::RawInclusive,inclusive,end);inclusive_recorded=true;
        let admitted=SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().pace.native_created(native));
        birth?;attached?;exact?;app_profile?;raw?;admitted?;if native_code!=0{return Err(Invalid("helper creation returned failure with effect"));}
        // Only the controller's helper_poll charges pipe CPU and waits on group
        // stop/debt. No ordinary proof clock/parser work runs while helper active.
        let pipe=pipe.as_mut().ok_or(Invalid("helper output pipe unavailable"))?;
        loop {
            helper_hook(pgid,1)?;
            let poll=helper_stamp(HelperPhase::FirstPoll,false);
            if let Some(input)=input {
                if sent==input.len(){stdin.take();}
                else if let Some(stdin)=stdin.as_mut(){let end=(sent+4096).min(input.len());match stdin.write(&input[sent..end]){Ok(0)=>return Err(Invalid("helper stdin stalled")),Ok(n)=>sent+=n,Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},Err(_)=>return Err(Invalid("helper stdin unavailable")),}}
                if sent==input.len(){stdin.take();}
            }
            match pipe.read(&mut chunk){Ok(n)=>{if bytes.len()+n>OUTPUT{return Err(Invalid("resource_limit: helper output bound"));}bytes.extend_from_slice(&chunk[..n]);},Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::Interrupted)=>{},Err(_)=>return Err(Invalid("helper output unavailable")),}
            let exited=helper_exited(pgid)?;
            if first_poll{first_poll=false;helper_record(HelperPhase::FirstPoll,poll,helper_stamp(HelperPhase::FirstPoll,true))?;}
            if exited{return Ok(());}
        }
    });
    if !inclusive_recorded {let _=helper_record(HelperPhase::RawInclusive,inclusive,helper_stamp(HelperPhase::RawInclusive,true));}
    stdin.take(); // close the owned writer on success, refusal or unwind
    // Never use Child::try_wait here: it could reap the leader while the control
    // owner still holds permission to signal this pgid. Retire is serialized by
    // that owner with its independent stop/continue/kill controller.
    let mut retired = helper_catch(|| helper_hook(pgid, 2));
    if retired.is_err() {
        // Retirement is idempotent for this still-pinned exact leader; retry a
        // one-shot hook panic before any reap. Original failure still refuses.
        let _ = helper_catch(|| helper_hook(pgid, 2))
            .map(|_| retired = Err(Invalid("selected helper retirement recovered")));
    }
    unsafe {
        libc::kill(-pgid, libc::SIGKILL);
    }
    let mut status = None;
    let mut usage = unsafe { std::mem::zeroed::<libc::rusage>() };
    let mut settlement_error = None;
    let mut group_gone = false;
    let mut settle = || {
        if status.is_none() {
            match helper_exited(pgid) {
                Ok(false) => return true,
                Ok(true) => {
                    let mut raw = 0;
                    let reaped = unsafe { libc::wait4(pgid, &mut raw, libc::WNOHANG, &mut usage) };
                    if reaped == 0 {
                        return true;
                    }
                    if reaped != pgid {
                        settlement_error = Some(Invalid("selected helper direct join unavailable"));
                        return false;
                    }
                    status = Some(if libc::WIFEXITED(raw) {
                        libc::WEXITSTATUS(raw)
                    } else {
                        -1
                    });
                    return true;
                }
                Err(error) => {
                    settlement_error = Some(error);
                    return false;
                }
            }
        }
        // A descendant retaining this exact group cannot outlive certification.
        // No signals are sent after the leader's reap, so pid reuse cannot turn
        // a delayed control tick into an effect on an unrelated group.
        let exists = unsafe { libc::kill(-pgid, 0) };
        if exists != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            group_gone = true;
            return false;
        }
        true
    };
    let mut cleanup_panic = false;
    loop {
        let mut exhausted = false;
        let mut called = false;
        let pacing = helper_catch(|| {
            cleanup_steps(&mut || {
                called = true;
                let more = settle();
                if !more {
                    exhausted = true;
                }
                more
            });
            Ok(())
        });
        if pacing.is_err() {
            cleanup_panic = true;
        }
        if exhausted {
            break;
        }
        // A panicking or prematurely returning cleanup hook cannot fabricate
        // settlement. Continue one small physical step per existing9ms wait;
        // this recovery always refuses certification.
        if pacing.is_err() || !called {
            cleanup_panic = true;
            std::thread::sleep(std::time::Duration::from_millis(9));
            if !settle() {
                break;
            }
        }
    }
    drop(settle);
    let time_ns = |time: libc::timeval| {
        (time.tv_sec as u64)
            .saturating_mul(1_000_000_000)
            .saturating_add((time.tv_usec as u64).saturating_mul(1000))
    };
    let cpu = time_ns(usage.ru_utime).saturating_add(time_ns(usage.ru_stime));
    #[cfg(target_os = "macos")]
    let rss = usage.ru_maxrss as u64;
    #[cfg(not(target_os = "macos"))]
    let rss = (usage.ru_maxrss as u64).saturating_mul(1024);
    let measured = if status.is_some() {
        helper_catch(|| {
            SELECTED.with(|slot| {
                slot.borrow_mut()
                    .as_mut()
                    .ok_or(Invalid("selected helper owner unavailable"))?
                    .pace
                    .helper_usage(pgid, cpu, rss)
            })
        })
    } else {
        Err(Invalid("selected helper usage unavailable"))
    };
    if group_gone && status.is_some() { SELECTED.with(|slot|slot.borrow_mut().as_mut().unwrap().helper_active=false); }
    let detached = if group_gone && status.is_some() {
        helper_catch(|| helper_hook(pgid, 3))
    } else {
        Err(Invalid("selected helper settlement unproven"))
    };
    let resumed = helper_catch(helper_boundary);
    observed?;
    retired?;
    if let Some(error) = settlement_error {
        return Err(error);
    }
    if !group_gone || cleanup_panic {
        return Err(Invalid("selected helper settlement unproven"));
    }
    detached?;
    measured?;
    resumed?;
    if rss > super::c1::storage::HELPERS {
        return Err(Invalid("selected helper RSS bound"));
    }
    // All writers have actually settled. Drain the remaining finite pipe bytes
    // using the same owner budget; no full capture or child result is trusted early.
    let pipe = pipe
        .as_mut()
        .ok_or(Invalid("selected proof pipe unavailable"))?;
    loop {
        checkpoint()?;
        match pipe.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if bytes.len() + n > OUTPUT {
                    return Err(Invalid("selected proof command output bound"));
                }
                bytes.extend_from_slice(&chunk[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Invalid("selected settled pipe unavailable")),
        }
    }
    bytes.shrink_to_fit();
    helper_boundary()?;
    let code = status.ok_or(Invalid("selected helper direct join unavailable"))?;

    Ok((code, bytes))
}

#[derive(Clone, Debug)]
pub struct Root {
    pub name: String,
    pub repo: String,
    pub path: PathBuf,
    pub main: String,
    pub cross_repo: bool,
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path != ".bboxy"
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && !path.contains(['\0', '\\'])
}
/// MCP RecordPins are KB-relative; D0's git-object pins are repository-relative.
/// Normalize only for lookup/permission checks, never rewrite their signed/identity spelling.
pub fn repository_path(path: &str) -> Result<String, Invalid> {
    if !safe_path(path) {
        return Err(Invalid("record path escapes mounted KB"));
    }
    Ok(if path.starts_with(".bboxy/") {
        path.into()
    } else {
        format!(".bboxy/{path}")
    })
}

// Adapted from bboxy450-harness WP2/WP3 owned Git capture isolation in src/vcs.rs
// and src/dream/join.rs. Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
async fn git(
    path: &Path,
    args: &[&str],
    deadline: tokio::time::Instant,
) -> Result<(i32, Vec<u8>), Invalid> {
    if selected() {
        let mut command = vec![
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        command.extend(args.iter().map(|arg| (*arg).to_owned()));
        return selected_command(path, "git", &command, None);
    }
    let timeout = deadline
        .checked_duration_since(tokio::time::Instant::now())
        .ok_or(Invalid("audit git deadline"))?;
    let mut command = Vec::new();
    for name in std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with("GIT_"))
    {
        command.extend(["-u".into(), name]);
    }
    command.extend(
        [
            "GIT_CONFIG_NOSYSTEM=1",
            "GIT_CONFIG_GLOBAL=/dev/null",
            "GIT_TERMINAL_PROMPT=0",
            "GIT_ASKPASS=/usr/bin/false",
            "SSH_ASKPASS=/usr/bin/false",
            "git",
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]
        .map(str::to_owned),
    );
    command.extend(args.iter().map(|a| (*a).to_owned()));
    // The existing KB provider's MAX_DOCUMENT is 2MiB. Captured bytes never become diagnostics.
    let cap = 2 * 1024 * 1024;
    let output = Box::pin(crate::process::capture_stdout(
        crate::process::Request {
            program: "/usr/bin/env".into(),
            args: command,
            stdin: None,
            cwd: path.into(),
            timeout,
            spill_dir: Default::default(),
            cap,
            redactor: Default::default(),
        },
        tokio_util::sync::CancellationToken::new(),
    ))
    .await
    .map_err(|_| Invalid("audit git unavailable"))?;
    if output.cancelled || output.bytes.len() >= cap {
        return Err(Invalid("audit git deadline/output bound"));
    }
    Ok((
        output.code.ok_or(Invalid("audit git did not exit"))?,
        output.bytes,
    ))
}
fn deadline() -> tokio::time::Instant {
    tokio::time::Instant::now() + crate::vcs::CAPTURE_BUDGET
}
impl Root {
    /// Only call for an explicitly authorized mounted root. A name is never interpreted as a path.
    pub async fn open(name: String, path: PathBuf) -> Result<Self, Invalid> {
        if name.is_empty() || name.contains(['\0', '\n']) {
            return Err(Invalid("invalid mounted root name"));
        }
        let repo = crate::vcs::repo_key(&path)
            .await
            .ok_or(Invalid("mounted root repository unavailable"))?;
        let until = deadline();
        let (code, bytes) = git(&path, &["rev-parse", "--show-toplevel"], until).await?;
        if code != 0 {
            return Err(Invalid("mounted root repository unavailable"));
        }
        let path = PathBuf::from(
            std::str::from_utf8(&bytes)
                .map_err(|_| Invalid("invalid git root"))?
                .trim_end_matches(['\r', '\n']),
        );
        let (code, bytes) = git(&path, &["rev-parse", "--verify", "origin/main"], until).await?;
        let main = std::str::from_utf8(&bytes)
            .map_err(|_| Invalid("invalid main pin"))?
            .trim()
            .to_owned();
        if code != 0 || !entry::valid_commit(&main) {
            return Err(Invalid("mounted origin/main unavailable"));
        }
        let object = format!("{main}:.bboxy/dream.toml");
        let (code, bytes) = git(&path, &["show", &object], until).await?;
        let cross_repo = code == 0 && std::str::from_utf8(&bytes).is_ok_and(cross_repo_opt_in);
        Ok(Self {
            name,
            repo,
            path,
            main,
            cross_repo,
        })
    }
    pub async fn paths(&self, prefix: &str) -> Result<Vec<String>, Invalid> {
        self.paths_at(prefix, &self.main).await
    }
    pub async fn paths_at(&self, prefix: &str, commit: &str) -> Result<Vec<String>, Invalid> {
        let normalized = repository_path(prefix)?;
        if let Some(bindings) = bindings() {
            checkpoint()?;
            let mut paths = bindings
                .records
                .keys()
                .filter(|(root, at, path)| {
                    root == &self.name && at == commit && path.starts_with(&normalized)
                })
                .map(|(_, _, path)| path.clone())
                .collect::<Vec<_>>();
            paths.sort();
            paths.dedup();
            return Ok(paths);
        }
        let prefix = normalized.as_str();
        if !entry::valid_commit(commit) {
            return Err(Invalid("invalid catalogue commit"));
        }
        let until = deadline();
        let (code, kind) = git(&self.path, &["cat-file", "-t", commit], until).await?;
        if code != 0 || kind != b"commit\n" {
            return Err(Invalid("record pin is not a commit object"));
        }
        let (member, _) = git(
            &self.path,
            &["merge-base", "--is-ancestor", commit, &self.main],
            until,
        )
        .await?;
        if member != 0 {
            return Err(Invalid("catalogue commit is outside mounted origin/main"));
        }
        let (code, bytes) = git(
            &self.path,
            &["ls-tree", "-r", "-z", "--name-only", commit, "--", prefix],
            until,
        )
        .await?;
        if code != 0 {
            return Err(Invalid("committed record catalogue unavailable"));
        }
        super::resolve::proof_blocking(move || {
            super::resolve::assert_proof_owner();
            bytes
                .split(|b| *b == 0)
                .filter(|b| !b.is_empty())
                .map(|b| {
                    std::str::from_utf8(b)
                        .map(str::to_owned)
                        .map_err(|_| Invalid("non-UTF8 record path"))
                })
                .collect()
        })
        .await?
    }
    pub async fn read(&self, path: &str, commit: &str) -> Result<Vec<u8>, Invalid> {
        if !entry::valid_commit(commit) {
            return Err(Invalid("invalid committed record pin"));
        }
        let normalized = repository_path(path)?;
        let admitted = if let Some(bindings) = bindings() {
            Some(
                bindings
                    .records
                    .get(&(self.name.clone(), commit.to_owned(), normalized.clone()))
                    .cloned()
                    .ok_or(Invalid("selected record proof unavailable"))?,
            )
        } else {
            None
        };
        let path = normalized.as_str();
        let until = deadline();
        let (code, kind) = git(&self.path, &["cat-file", "-t", commit], until).await?;
        if code != 0 || kind != b"commit\n" {
            return Err(Invalid("record pin is not a commit object"));
        }
        let (member, _) = git(
            &self.path,
            &["merge-base", "--is-ancestor", commit, &self.main],
            until,
        )
        .await?;
        if member != 0 {
            return Err(Invalid("record commit is not on mounted origin/main"));
        }
        let (code, mode) = git(&self.path, &["ls-tree", "-z", commit, "--", path], until).await?;
        let regular = mode.split(|b| *b == 0).any(|line| {
            line.iter().position(|b| *b == b'\t').is_some_and(|at| {
                let (head, name) = (&line[..at], &line[at + 1..]);
                name == path.as_bytes()
                    && (head.starts_with(b"100644 blob ") || head.starts_with(b"100755 blob "))
            })
        });
        if code != 0 || !regular {
            return Err(Invalid("record is not a committed regular file"));
        }
        let object = format!("{commit}:{path}");
        let (code, bytes) = git(&self.path, &["show", &object], until).await?;
        if code != 0 {
            return Err(Invalid("committed record unavailable"));
        }
        if admitted
            .as_ref()
            .is_some_and(|expected| expected != &entry::digest(&bytes))
        {
            return Err(Invalid("selected committed record byte mismatch"));
        }
        Ok(bytes)
    }
}
fn cross_repo_opt_in(text: &str) -> bool {
    let mut seen = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            break;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "cross_repo"
        {
            if seen.is_some() {
                return false;
            }
            seen = Some(value.trim() == "true");
        }
    }
    seen == Some(true)
}

/// Match the actual KB writer's DSSE payload, retaining YAML key order.
/// Adapted from bboxy450-term b2f4b0cea9d58c58d44f6c5746e1fd286e2317c8,
/// crates/bbt-mux/src/index.rs:1044-1073 and provenance/author.rs.
/// Upstream policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
/// This checks bytes only. A self-signature never establishes signer authority.
pub fn signature_bytes(record: &Record) -> bool {
    let Some(id) = record.front.get("id").and_then(Value::as_str) else {
        return false;
    };
    super::trust::verify_bytes(&record.bytes, id, &["seat".into()])
        .is_ok_and(|v| v.front == record.front && v.body == record.body && v.yaml == record.yaml)
}

#[derive(Clone)]
pub struct ResolvedRecord {
    pub pin: RecordPin,
    pub record: Record,
    pub trusted: bool,
    pub signature_valid: bool,
}
// Safe API use of hashbrown (MIT OR Apache-2.0), https://github.com/rust-lang/hashbrown.
// Store only the Arc: the immutable record already owns the complete pin.
// No cache iteration escapes into canonical output. Production uses RandomState;
// the private generic only permits collision/rehash tests without weakening it.
#[derive(Default)]
struct RecordCache<S = RandomState> {
    table: hashbrown::HashTable<Arc<ResolvedRecord>>,
    hash_builder: S,
}
fn record_hash<S: BuildHasher>(builder: &S, pin: &RecordPin) -> u64 {
    // Hash's tuple/string/Option encodings preserve field boundaries and None/Some.
    builder.hash_one((&pin.root, &pin.path, &pin.id, &pin.commit, &pin.digest))
}
impl<S: BuildHasher> RecordCache<S> {
    fn get(&self, pin: &RecordPin) -> Option<&Arc<ResolvedRecord>> {
        self.table
            .find(record_hash(&self.hash_builder, pin), |record| {
                record.pin == *pin
            })
    }
    fn insert(&mut self, record: Arc<ResolvedRecord>) -> Arc<ResolvedRecord> {
        let builder = &self.hash_builder;
        self.table
            .entry(
                record_hash(builder, &record.pin),
                |cached| cached.pin == record.pin,
                |cached| record_hash(builder, &cached.pin),
            )
            .or_insert(record)
            .get()
            .clone()
    }
    fn clear(&mut self) {
        self.table.clear();
    }
}

pub struct Resolver {
    pub roots: BTreeMap<String, Root>,
    pub sessions: PathBuf,
    pub logs: Arc<BTreeMap<SessionId, Vec<Envelope>>>,
    records: RecordCache,
    trust: Option<Arc<super::trust::Trust>>,
    receipt_domains: Option<super::receipt::Domains>,
    catalogues: BTreeMap<(String, String), Vec<String>>,
}
impl Resolver {
    /// No directory enumeration, HOME, live record or implicit Git branch lookup.
    /// Install SelectedScope on this dedicated owner before proof operations.
    pub(crate) fn from_selected(
        sessions: PathBuf,
        roots: Vec<Root>,
        logs: BTreeMap<SessionId, Vec<Envelope>>,
        domains: super::receipt::Domains,
    ) -> Result<Self, Invalid> {
        if !selected() || domains.sessions != sessions {
            return Err(Invalid("selected custody unavailable"));
        }
        let mut mounted = BTreeMap::new();
        for root in roots {
            checkpoint()?;
            if root.name.is_empty()
                || !entry::valid_commit(&root.main)
                || !root.path.is_absolute()
                || mounted.insert(root.name.clone(), root).is_some()
            {
                return Err(Invalid("invalid selected mounted root"));
            }
        }
        for (session, log) in &logs {
            checkpoint()?;
            if !log
                .last()
                .is_some_and(|e| matches!(e.event, Event::SessionClosed))
                || log.iter().any(|e| e.session != *session)
            {
                return Err(Invalid("selected session closure unavailable"));
            }
        }
        let mut catalogues: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
        for (root, commit, path) in bindings().unwrap().records.keys() {
            checkpoint()?;
            catalogues
                .entry((root.clone(), commit.clone()))
                .or_default()
                .push(path.clone());
        }
        Ok(Self {
            roots: mounted,
            sessions,
            logs: Arc::new(logs),
            records: RecordCache::default(),
            trust: None,
            receipt_domains: Some(domains),
            catalogues,
        })
    }
    pub async fn open(sessions: PathBuf, roots: Vec<Root>) -> Result<Self, Invalid> {
        let input = sessions.clone();
        let logs = super::resolve::proof_blocking(move || {
            use std::io::Read;
            use std::os::unix::fs::OpenOptionsExt;
            super::resolve::assert_proof_owner();
            let mut logs = BTreeMap::new();
            for file in
                std::fs::read_dir(input).map_err(|_| Invalid("sessions root unavailable"))?
            {
                let file = file.map_err(|_| Invalid("session directory unavailable"))?;
                let path = file.path();
                if path.extension().is_none_or(|ext| ext != "jsonl") {
                    continue;
                }
                let Some(session) = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.parse::<ulid::Ulid>().ok())
                    .map(SessionId)
                else {
                    return Err(Invalid("sessions root contains non-custody JSONL input"));
                };
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
                    .open(&path)
                    .map_err(|_| Invalid("unsafe session input"))?;
                if !file
                    .metadata()
                    .map_err(|_| Invalid("session metadata unavailable"))?
                    .is_file()
                {
                    return Err(Invalid("unsafe session input"));
                }
                let mut text = String::new();
                file.read_to_string(&mut text)
                    .map_err(|_| Invalid("session bytes unavailable"))?;
                let events = text
                    .split_inclusive('\n')
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| {
                        Envelope::from_line(line)
                            .map_err(|_| Invalid("malformed retained envelope"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                logs.insert(session, events);
            }
            Ok::<_, Invalid>(logs)
        })
        .await??;
        let mut mounted = BTreeMap::new();
        for root in roots {
            if mounted.insert(root.name.clone(), root).is_some() {
                return Err(Invalid("duplicate mounted root name"));
            }
        }
        Ok(Self {
            roots: mounted,
            sessions,
            logs: Arc::new(logs),
            records: RecordCache::default(),
            trust: None,
            receipt_domains: None,
            catalogues: BTreeMap::new(),
        })
    }
    /// Supply only independently loaded local policy; records never select policy contents.
    pub fn with_receipt_domains(mut self, domains: super::receipt::Domains) -> Self {
        self.receipt_domains = Some(domains);
        self
    }
    pub fn receipt_domains(&self) -> super::receipt::Domains {
        self.receipt_domains.clone().unwrap_or_else(|| {
            super::receipt::Domains::configured(
                self.sessions.clone(),
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default(),
            )
        })
    }
    pub fn trust(&self) -> Option<Arc<super::trust::Trust>> {
        self.trust.clone()
    }
    pub fn with_trust(mut self, trust: super::trust::Trust) -> Self {
        self.records.clear();
        self.trust = Some(Arc::new(trust));
        self
    }
    /// Accepted verdicts used for graph strength must pass T, not just D0's locator join.
    pub async fn accepted_tasks(&self) -> Result<Vec<super::join::Task>, Invalid> {
        let Some(trust) = &self.trust else {
            return Ok(vec![]);
        };
        let mut accepted = Vec::new();
        for root in self.roots.values() {
            super::resolve::checkpoint()?;
            if root.name != root.repo {
                continue;
            }
            for path in root
                .paths(".bboxy/tasks")
                .await?
                .into_iter()
                .filter(|p| p.ends_with(".md"))
            {
                let bytes = root.read(&path, &root.main).await?;
                let repo = root.repo.clone();
                let commit = root.main.clone();
                let task = super::resolve::proof_blocking(move || {
                    super::join::Task::from_committed(&repo, &path, &commit, &bytes)
                })
                .await?;
                if let Ok(task) = task
                    && task.verdict == super::join::Verdict::Accepted
                    && trust.anchor(&task.pin).await.is_ok()
                {
                    accepted.push(task);
                }
            }
        }
        Ok(accepted)
    }
    pub fn event(&self, pin: &EvidenceRef) -> Result<Envelope, Invalid> {
        Self::event_from(&self.logs, pin)
    }
    fn event_from(
        logs: &BTreeMap<SessionId, Vec<Envelope>>,
        pin: &EvidenceRef,
    ) -> Result<Envelope, Invalid> {
        super::resolve::assert_proof_owner();
        let EvidenceRef::Event {
            session,
            seq,
            digest,
        } = pin
        else {
            return Err(Invalid("not an event pin"));
        };
        let event = logs
            .get(session)
            .and_then(|events| {
                events
                    .iter()
                    .find(|e| e.session == *session && e.seq == Some(*seq))
            })
            .ok_or(Invalid("event identity does not resolve"))?;
        let bytes = event
            .wire_bytes()
            .map_err(|_| Invalid("event bytes unavailable"))?;
        if !entry::valid_digest(digest)
            || entry::digest(bytes.strip_suffix(b"\n").unwrap_or(&bytes)) != *digest
        {
            return Err(Invalid("event line digest mismatch"));
        }
        Ok(event.clone())
    }
    pub async fn event_async(&self, pin: &EvidenceRef) -> Result<Envelope, Invalid> {
        let logs = self.logs.clone();
        let pin = pin.clone();
        super::resolve::proof_blocking(move || Self::event_from(&logs, &pin)).await?
    }
    pub async fn record(&mut self, pin: &RecordPin) -> Result<Arc<ResolvedRecord>, Invalid> {
        if !entry::valid_digest(&pin.digest) || !safe_path(&pin.path) {
            return Err(Invalid("invalid memory record pin"));
        }
        if let Some(record) = self.records.get(pin) {
            return Ok(record.clone());
        }
        let root = self
            .roots
            .get(&pin.root)
            .ok_or(Invalid("record root is not mounted"))?;
        if selected() && pin.commit.is_none() {
            return Err(Invalid("selected committed record required"));
        }
        let bytes = if let Some(commit) = &pin.commit {
            root.read(&pin.path, commit).await?
        } else {
            let root_path = root.path.clone();
            let path = repository_path(&pin.path)?;
            super::resolve::proof_blocking(move || {
                use std::{
                    io::Read,
                    os::{
                        fd::{AsRawFd, FromRawFd},
                        unix::fs::OpenOptionsExt,
                    },
                };
                super::resolve::assert_proof_owner();
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
                    .open(root_path)
                    .map_err(|_| Invalid("live record unavailable"))?;
                let components = path.split('/').collect::<Vec<_>>();
                for (index, component) in components.iter().enumerate() {
                    let name = std::ffi::CString::new(*component)
                        .map_err(|_| Invalid("invalid live record path"))?;
                    let flags = libc::O_RDONLY
                        | libc::O_NOFOLLOW
                        | libc::O_NONBLOCK
                        | libc::O_CLOEXEC
                        | if index + 1 < components.len() {
                            libc::O_DIRECTORY
                        } else {
                            0
                        };
                    let descriptor =
                        unsafe { libc::openat(file.as_raw_fd(), name.as_ptr(), flags) };
                    if descriptor < 0 {
                        return Err(Invalid("unsafe live record path"));
                    }
                    file = unsafe { std::fs::File::from_raw_fd(descriptor) };
                }
                if !file
                    .metadata()
                    .map_err(|_| Invalid("live record unavailable"))?
                    .is_file()
                {
                    return Err(Invalid("unsafe live record"));
                }
                let mut bytes = Vec::new();
                file.by_ref()
                    .take(2 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| Invalid("live record unavailable"))?;
                if bytes.len() > 2 * 1024 * 1024 {
                    return Err(Invalid("live record byte bound"));
                }
                Ok(bytes)
            })
            .await??
        };
        reserve_record(&bytes)?;
        let expected = pin.digest.clone();
        let (record, signature_valid) = super::resolve::proof_blocking(move || {
            super::resolve::assert_proof_owner();
            if entry::digest(&bytes) != expected {
                return Err(Invalid("record byte digest mismatch"));
            }
            let record = Record::parse(&bytes)?;
            let signature_valid = signature_bytes(&record);
            Ok::<_, Invalid>((record, signature_valid))
        })
        .await??;
        if pin
            .id
            .as_deref()
            .is_some_and(|id| record.front.get("id").and_then(Value::as_str) != Some(id))
        {
            return Err(Invalid("record id mismatch"));
        }
        let trusted =
            if let (Some(trust), Some(id), Some(commit)) = (&self.trust, &pin.id, &pin.commit) {
                trust
                    .anchor(&EvidenceRef::Record {
                        root: pin.root.clone(),
                        id: id.clone(),
                        path: pin.path.clone(),
                        commit: commit.clone(),
                        digest: pin.digest.clone(),
                    })
                    .await
                    .is_ok()
            } else {
                false
            };
        let result = Arc::new(ResolvedRecord {
            pin: pin.clone(),
            record,
            trusted,
            signature_valid,
        });
        Ok(self.records.insert(result))
    }
    pub async fn evidence(&mut self, pin: &EvidenceRef) -> Result<bool, Invalid> {
        match pin {
            EvidenceRef::Event { .. } => {
                self.event_async(pin).await?;
                Ok(false)
            }
            EvidenceRef::Record {
                root,
                id,
                path,
                commit,
                digest,
            } => {
                if id.is_empty() || !entry::valid_commit(commit) {
                    return Err(Invalid("incomplete record evidence pin"));
                }
                Ok(self
                    .record(&RecordPin {
                        root: root.clone(),
                        id: Some(id.clone()),
                        path: path.clone(),
                        commit: Some(commit.clone()),
                        digest: digest.clone(),
                    })
                    .await?
                    .trusted)
            }
            _ => Err(Invalid("unknown evidence pin")),
        }
    }
    /// Locator spelling stays in EntryId; resolution returns observed pins without rewriting it.
    pub async fn evidence_value(
        &mut self,
        local_root: &str,
        value: &Value,
    ) -> Result<EvidenceRef, Invalid> {
        if value.get("kind").is_some() {
            let pin: EvidenceRef = serde_json::from_value(value.clone())
                .map_err(|_| Invalid("invalid evidence shape"))?;
            self.evidence(&pin).await?;
            return Ok(pin);
        }
        let reference = value
            .get("ref")
            .and_then(Value::as_str)
            .ok_or(Invalid("missing evidence locator"))?;
        let reference = reference
            .strip_prefix("[[")
            .and_then(|r| r.strip_suffix("]]"))
            .unwrap_or(reference);
        let digest = value
            .get("digest")
            .and_then(Value::as_str)
            .filter(|d| entry::valid_digest(d))
            .ok_or(Invalid("missing exact evidence digest"))?;
        if let Some(event) = reference.strip_prefix("ir:") {
            let (session, seq) = event
                .split_once('#')
                .ok_or(Invalid("invalid event locator"))?;
            let session = SessionId(
                session
                    .parse()
                    .map_err(|_| Invalid("invalid event session"))?,
            );
            let seq = Seq(seq.parse().map_err(|_| Invalid("invalid event sequence"))?);
            let pin = EvidenceRef::Event {
                session,
                seq,
                digest: digest.into(),
            };
            self.event_async(&pin).await?;
            return Ok(pin);
        }
        let (root, locator) = if let Some(cross) = reference.strip_prefix("kb:") {
            let name = self
                .roots
                .keys()
                .filter(|name| cross.starts_with(&format!("{name}/")))
                .max_by_key(|name| name.len())
                .ok_or(Invalid("cross-root evidence is not mounted"))?
                .clone();
            let locator = cross
                .strip_prefix(&format!("{name}/"))
                .ok_or(Invalid("invalid root prefix"))?
                .to_owned();
            (name, locator)
        } else {
            (
                value
                    .get("root")
                    .and_then(Value::as_str)
                    .unwrap_or(local_root)
                    .to_owned(),
                reference.to_owned(),
            )
        };
        if value
            .get("root")
            .and_then(Value::as_str)
            .is_some_and(|r| r != root)
        {
            return Err(Invalid("conflicting evidence root"));
        }
        let commit = value
            .get("commit")
            .and_then(Value::as_str)
            .filter(|c| entry::valid_commit(c))
            .ok_or(Invalid("record evidence requires an exact commit"))?
            .to_owned();
        let key = (root.clone(), commit.clone());
        if selected() && !self.catalogues.contains_key(&key) {
            return Err(Invalid("selected record catalogue unavailable"));
        }
        if !self.catalogues.contains_key(&key) {
            let paths = self
                .roots
                .get(&root)
                .ok_or(Invalid("evidence root is not mounted"))?
                .paths_at(".bboxy/", &commit)
                .await?;
            self.catalogues.insert(key.clone(), paths);
        }
        let paths = self.catalogues[&key]
            .iter()
            .filter(|path| {
                let relative = path.trim_start_matches(".bboxy/").trim_end_matches(".md");
                locator == **path
                    || locator == relative
                    || relative.rsplit('/').next() == Some(locator.as_str())
            })
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return Err(Invalid("ambiguous or missing committed evidence locator"));
        }
        let path = paths[0].to_owned();
        if value
            .get("path")
            .and_then(Value::as_str)
            .is_some_and(|p| !repository_path(p).is_ok_and(|p| p == path))
        {
            return Err(Invalid("conflicting evidence path"));
        }
        let result = self
            .record(&RecordPin {
                root: root.clone(),
                id: value.get("id").and_then(Value::as_str).map(str::to_owned),
                path: path.clone(),
                commit: Some(commit.clone()),
                digest: digest.into(),
            })
            .await?;
        let id = result
            .record
            .front
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(Invalid("evidence record has no id"))?;
        Ok(EvidenceRef::Record {
            root,
            id: id.into(),
            path,
            commit,
            digest: digest.into(),
        })
    }
    pub async fn scope(&mut self, scope: &Scope, evidence: &[EvidenceRef]) -> Result<(), Invalid> {
        let repos: BTreeSet<_> = scope.repos.iter().collect();
        if repos.is_empty()
            || repos.len() != scope.repos.len()
            || matches!(scope.level, ScopeLevel::Unknown(_))
        {
            return Err(Invalid("invalid memory scope"));
        }
        for pin in evidence {
            super::resolve::checkpoint()?;
            let repo = match pin {
                EvidenceRef::Event { session, .. } => {
                    self.event_async(pin).await?;
                    let meta = self
                        .logs
                        .get(session)
                        .and_then(|events| {
                            events.iter().find_map(|e| {
                                if let Event::SessionStarted { meta } = &e.event {
                                    Some(meta)
                                } else {
                                    None
                                }
                            })
                        })
                        .ok_or(Invalid("evidence session header unavailable"))?;
                    match meta.vcs.as_ref().and_then(|v| v.repo.clone()) {
                        Some(repo) => repo,
                        None if selected() => {
                            return Err(Invalid("selected source repository unavailable"));
                        }
                        None => crate::vcs::repo_key(&meta.cwd)
                            .await
                            .ok_or(Invalid("evidence session repository unavailable"))?,
                    }
                }
                EvidenceRef::Record { root, .. } => {
                    self.evidence(pin).await?;
                    self.roots
                        .get(root)
                        .ok_or(Invalid("evidence root unavailable"))?
                        .repo
                        .clone()
                }
                _ => return Err(Invalid("unknown scope evidence")),
            };
            if !repos.contains(&repo) {
                return Err(Invalid("evidence repository is outside memory scope"));
            }
        }
        if matches!(scope.level, ScopeLevel::System) || repos.len() > 1 {
            for repo in repos {
                if !self
                    .roots
                    .values()
                    .any(|root| &root.repo == repo && root.cross_repo)
                {
                    return Err(Invalid("cross-repository scope lacks committed opt-in"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::hash::{BuildHasherDefault, Hasher};

    fn pin(n: usize) -> RecordPin {
        RecordPin {
            root: format!("fixture.invalid/dream/repository-{}", n % 16),
            path: format!(".bboxy/memories/lesson-{n}.md"),
            id: Some(format!("record-{n}")),
            commit: Some(format!("{n:040x}")),
            digest: format!("sha256:{n:064x}"),
        }
    }
    fn resolved(pin: RecordPin, marker: usize, trusted: bool) -> Arc<ResolvedRecord> {
        Arc::new(ResolvedRecord {
            pin,
            record: Record {
                front: json!({"marker":marker}),
                yaml: serde_yaml::Value::Null,
                body: format!("synthetic record {marker}"),
                bytes: format!("synthetic bytes {marker}").into_bytes(),
            },
            signature_valid: trusted,
            trusted,
        })
    }

    #[test]
    fn record_cache_matches_all_fields_and_exact_option_identity() {
        let mut base = pin(0);
        base.id = None;
        base.commit = None;
        let mut pins = vec![base.clone()];
        for field in 0..5 {
            let mut different = base.clone();
            match field {
                0 => different.root.push_str("-other"),
                1 => different.path.push_str(".other"),
                2 => different.id = Some(String::new()),
                3 => different.commit = Some(String::new()),
                _ => different.digest.push('1'),
            }
            pins.push(different);
        }
        for value in ["one", "two"] {
            let mut id = base.clone();
            id.id = Some(value.into());
            pins.push(id);
            let mut commit = base.clone();
            commit.commit = Some(value.into());
            pins.push(commit);
        }
        let mut cache = RecordCache {
            table: hashbrown::HashTable::new(),
            hash_builder: BuildHasherDefault::<Collide>::default(),
        };
        let records = pins
            .into_iter()
            .enumerate()
            .map(|(i, p)| resolved(p, i, false))
            .collect::<Vec<_>>();
        for record in &records {
            cache.insert(record.clone());
        }
        assert_eq!(cache.table.len(), records.len());
        for record in records {
            let hit = cache.get(&record.pin).unwrap();
            assert!(Arc::ptr_eq(hit, &record));
            assert_eq!(hit.record.bytes, record.record.bytes);
        }
        assert!(cache.get(&pin(99)).is_none());
    }

    // Deliberate collision control, used only in this private test module.
    #[derive(Default)]
    struct Collide;
    impl Hasher for Collide {
        fn finish(&self) -> u64 {
            7
        }
        fn write(&mut self, _: &[u8]) {}
    }
    fn exercise_growth<S: BuildHasher>(hash_builder: S) -> Vec<String> {
        let mut cache = RecordCache {
            table: hashbrown::HashTable::new(),
            hash_builder,
        };
        let records = (0..512)
            .map(|i| resolved(pin(i), i, true))
            .collect::<Vec<_>>();
        let mut growths = 0;
        let mut capacity = cache.table.capacity();
        for record in &records {
            assert!(Arc::ptr_eq(&cache.insert(record.clone()), record));
            if cache.table.capacity() != capacity {
                growths += 1;
                capacity = cache.table.capacity();
            }
        }
        assert!(growths > 1, "exercise actual resize/rehash callbacks");
        let mut canonical = vec![];
        for i in (0..512).map(|n| (n * 37) % 512) {
            let record = cache.get(&records[i].pin).unwrap();
            assert!(Arc::ptr_eq(record, &records[i]));
            canonical.push(
                serde_json::to_string(&(
                    &record.pin,
                    &record.record.front,
                    &record.record.bytes,
                    record.trusted,
                    record.signature_valid,
                ))
                .unwrap(),
            );
        }
        let contradictory = resolved(records[0].pin.clone(), 9999, false);
        assert!(Arc::ptr_eq(
            &cache.insert(contradictory.clone()),
            &records[0]
        ));
        assert_eq!(cache.table.len(), records.len());
        assert!(cache.get(&records[0].pin).unwrap().trusted);
        cache.clear();
        assert_eq!(cache.table.len(), 0);
        assert!(records.iter().all(|r| cache.get(&r.pin).is_none()));
        assert!(Arc::ptr_eq(
            &cache.insert(contradictory.clone()),
            &contradictory
        ));
        assert!(!cache.get(&contradictory.pin).unwrap().trusted);
        canonical.sort();
        canonical
    }
    #[test]
    fn record_cache_collisions_resize_duplicates_and_clear() {
        let colliding = exercise_growth(BuildHasherDefault::<Collide>::default());
        let seeded = exercise_growth(RandomState::new());
        assert_eq!(
            colliding, seeded,
            "collisions never choose the wrong complete pin"
        );
    }
    #[test]
    fn record_cache_random_seeds_preserve_canonical_observable_records() {
        assert_eq!(
            exercise_growth(RandomState::new()),
            exercise_growth(RandomState::new())
        );
    }

    /// Bounded lookup kernel only: no filesystem, signatures, resolver validation or IO.
    #[test]
    #[ignore = "explicit release-mode measurement, never a CI timing threshold"]
    fn hashbrown_record_cache_bench() {
        use std::{hint::black_box, time::Instant};
        const RECORDS: usize = 4096;
        const LOOKUPS: usize = 262144;
        const SAMPLES: usize = 7;
        let pins = (0..RECORDS).map(pin).collect::<Vec<_>>();
        let mut old = BTreeMap::new();
        let mut new: RecordCache = RecordCache::default();
        for (i, pin) in pins.iter().enumerate() {
            let record = resolved(pin.clone(), i, false);
            old.insert(serde_json::to_string(pin).unwrap(), record.clone());
            new.insert(record);
        }
        let payload = old.keys().map(String::len).sum::<usize>();
        let old_sample = |count: usize| {
            let start = Instant::now();
            for n in 0..count {
                let key = serde_json::to_string(black_box(&pins[(n * 17) % RECORDS])).unwrap();
                black_box(old.get(&key).unwrap().clone());
            }
            start.elapsed().as_nanos()
        };
        let new_sample = |count: usize| {
            let start = Instant::now();
            for n in 0..count {
                black_box(
                    new.get(black_box(&pins[(n * 17) % RECORDS]))
                        .unwrap()
                        .clone(),
                );
            }
            start.elapsed().as_nanos()
        };
        old_sample(16384);
        new_sample(16384);
        let mut old_ns = vec![];
        let mut new_ns = vec![];
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                old_ns.push(old_sample(LOOKUPS));
                new_ns.push(new_sample(LOOKUPS));
            } else {
                new_ns.push(new_sample(LOOKUPS));
                old_ns.push(old_sample(LOOKUPS));
            }
        }
        let median = |values: &[u128]| {
            let mut v = values.to_vec();
            v.sort();
            v[v.len() / 2]
        };
        println!(
            "hashbrown_record_cache_bench {}",
            json!({
                "records":RECORDS,"debug_assertions":cfg!(debug_assertions),"lookups_per_sample_per_path":LOOKUPS,"samples":SAMPLES,
                "warmup_lookups_per_path":16384,"sample_order":"alternating old/new",
                "old_serde_btree_ns":old_ns,"new_hash_table_ns":new_ns,
                "old_median_ns_per_lookup":median(&old_ns) as f64 / LOOKUPS as f64,
                "new_median_ns_per_lookup":median(&new_ns) as f64 / LOOKUPS as f64,
                "old_over_new_median_ratio":median(&old_ns) as f64 / median(&new_ns) as f64,
                "old_duplicate_serialized_key_payload_bytes":payload,
                "new_duplicate_key_payload_bytes":0,
                "old_serialized_key_bytes_per_sample":payload*(LOOKUPS/RECORDS),
                "limits":"cache kernel; payload lengths exclude node/bucket/capacity/allocator overhead; not end-to-end or process memory"
            })
        );
    }
}

impl super::c1::disposal::Drain for SelectedBindings {
    fn drain_step(&mut self) -> bool {
        // Byte buffers have no nested ownership; remove one finite bound entry.
        self.records.pop_first().is_some()
            || self.blobs.pop_first().is_some()
            || self.artifacts.pop_first().is_some()
            || self.keys.pop_first().is_some()
            || self.key_bytes.pop_first().is_some()
    }
}
impl super::c1::disposal::Drain for super::entry::Identity {
    fn drain_step(&mut self) -> bool {
        self.evidence.drain_step() || self.derived_from.pop().is_some()
    }
}
impl super::c1::disposal::Drain for super::entry::Record {
    fn drain_step(&mut self) -> bool {
        self.front.drain_step() || self.yaml.drain_step()
    }
}
impl super::c1::disposal::Drain for Resolver {
    fn drain_step(&mut self) -> bool {
        if let Some(logs) = Arc::get_mut(&mut self.logs) {
            if let Some(mut first) = logs.first_entry() {
                if !first.get_mut().drain_step() {
                    first.remove_entry();
                }
                return true;
            }
        }
        if let Some(record) = self.records.table.iter_mut().next() {
            if let Some(record) = Arc::get_mut(record) {
                if record.record.drain_step() {
                    return true;
                }
            }
            self.records.table.extract_if(|_| true).next();
            return true;
        }
        if let Some(trust) = self.trust.as_mut() {
            if let Some(trust) = Arc::get_mut(trust) {
                if trust.drain_step() {
                    return true;
                }
            }
            self.trust.take();
            return true;
        }
        if let Some(mut first) = self.catalogues.first_entry() {
            if first.get_mut().pop().is_none() {
                first.remove_entry();
            }
            return true;
        }
        if let Some(domains) = self.receipt_domains.as_mut() {
            if domains.caches.pop().is_some()
                || domains.sink_roots.pop().is_some()
                || domains.native_repos.pop_first().is_some()
            {
                return true;
            }
        }
        self.roots.pop_first().is_some()
    }
}

impl<T: super::c1::disposal::Drain> super::c1::disposal::Drain for BTreeMap<SessionId, T> {
    fn drain_step(&mut self) -> bool {
        let Some(mut first) = self.first_entry() else {
            return false;
        };
        if !first.get_mut().drain_step() {
            first.remove_entry();
        }
        true
    }
}

impl<T: super::c1::disposal::Drain> super::c1::disposal::Drain for Result<T, Invalid> {
    fn drain_step(&mut self) -> bool {
        self.as_mut().is_ok_and(|v| v.drain_step())
    }
}
impl<T: super::c1::disposal::Drain> super::c1::disposal::Drain
    for Result<T, super::seal::ProofError>
{
    fn drain_step(&mut self) -> bool {
        self.as_mut().is_ok_and(|v| v.drain_step())
    }
}
impl<T: super::c1::disposal::Drain> super::c1::disposal::Drain for Arc<T> {
    fn drain_step(&mut self) -> bool {
        Arc::get_mut(self).is_some_and(|v| v.drain_step())
    }
}
impl super::c1::disposal::Drain for (Record, bool) {
    fn drain_step(&mut self) -> bool {
        self.0.drain_step()
    }
}
impl super::c1::disposal::Drain for (String, super::entry::Identity) {
    fn drain_step(&mut self) -> bool {
        self.1.drain_step()
    }
}
impl super::c1::disposal::Drain for (Result<(String, super::entry::Identity), Invalid>, bool) {
    fn drain_step(&mut self) -> bool {
        self.0.drain_step()
    }
}
impl super::c1::disposal::Drain for (Result<bool, Invalid>, bool) {
    fn drain_step(&mut self) -> bool {
        false
    }
}
impl super::c1::disposal::Drain
    for (
        Result<Vec<Envelope>, Invalid>,
        BTreeMap<SessionId, SessionMeta>,
        bool,
    )
{
    fn drain_step(&mut self) -> bool {
        self.0.drain_step() || self.1.drain_step()
    }
}
impl super::c1::disposal::Drain for (super::trust::Policy, PathBuf) {
    fn drain_step(&mut self) -> bool {
        self.0.drain_step()
    }
}
impl super::c1::disposal::Drain for (Vec<super::projection::Fact>, super::projection::Projection) {
    fn drain_step(&mut self) -> bool {
        self.0.drain_step() || self.1.drain_step()
    }
}
impl super::c1::disposal::Drain for (Vec<u8>, BTreeMap<PathBuf, Vec<u8>>) {
    fn drain_step(&mut self) -> bool {
        self.1.pop_first().is_some()
    }
}
impl super::c1::disposal::Drain for BTreeSet<SessionId> {
    fn drain_step(&mut self) -> bool {
        self.pop_first().is_some()
    }
}
impl super::c1::disposal::Drain for super::projection::RecordView {
    fn drain_step(&mut self) -> bool {
        match self {
            Self::StaleMetadata { harnesses, .. } => harnesses.pop().is_some(),
            _ => false,
        }
    }
}

impl super::c1::disposal::Drain for Result<super::join::Task, String> {
    fn drain_step(&mut self) -> bool {
        self.as_mut().is_ok_and(|v| v.pin.drain_step())
    }
}

impl super::c1::disposal::Drain for serde_yaml::Value {
    fn drain_step(&mut self) -> bool {
        use serde_yaml::Value;
        match self {
            Value::Sequence(values) => values.drain_step(),
            Value::Mapping(map) => {
                let Some((key, value)) = map.iter_mut().next() else {
                    return false;
                };
                if value.drain_step() {
                    return true;
                }
                if let Value::String(key) = key {
                    // Selected YAML preflight bounds the complete front to64KiB.
                    // Borrow a stack key so removing a leaf requires no heap copy.
                    let mut bytes = [0u8; 64 * 1024];
                    if key.len() > bytes.len() {
                        return false;
                    }
                    let length = key.len();
                    bytes[..length].copy_from_slice(key.as_bytes());
                    let key = std::str::from_utf8(&bytes[..length]).unwrap();
                    map.remove_entry(key);
                } else {
                    let key = key.clone();
                    map.remove_entry(&key);
                }
                true
            }
            Value::Tagged(tagged) => {
                if tagged.value.drain_step() {
                    return true;
                }
                *self = Value::Null;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod selected_helper_process_tests {
    // Project-owned private process/API witnesses. Keep the policy with reuse:
    // https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
    use super::*;
    use crate::dream::c1::{self, Control, HelperGroup, storage};
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
        mpsc,
    };
    use std::time::{Duration, Instant};

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum PanicAt {
        None,
        Attach,
        Before,
        Cleanup,
    }
    struct State {
        control: Arc<Control>,
        groups: Mutex<Vec<i32>>,
        paused: AtomicBool,
        stale_rejected: AtomicBool,
        panic_at: Mutex<PanicAt>,
        marker: PathBuf,
        pause_on_attach: bool,
    }
    struct Pace {
        state: Arc<State>,
        _credit: storage::Credit,
    }
    fn stopped() -> Invalid {
        Invalid("private helper control stopped")
    }
    impl Pace {
        fn take_panic(&self, point: PanicAt) -> bool {
            let mut selected = self
                .state
                .panic_at
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if *selected == point {
                *selected = PanicAt::None;
                true
            } else {
                false
            }
        }
    }
    impl ProofPace for Pace {
        fn reserve(&mut self, _: u64) -> Result<(), Invalid> {
            Ok(())
        }
        fn before(&mut self) -> Result<(), Invalid> {
            if self.state.marker.exists() && self.take_panic(PanicAt::Before) {
                panic!("private proof checkpoint panic");
            }
            loop {
                if self.state.control.error().is_some() {
                    return Err(stopped());
                }
                if !self.state.control.foreground.load(Ordering::Acquire) {
                    return Ok(());
                }
                self.state.paused.store(true, Ordering::Release);
                std::thread::sleep(Duration::from_micros(100));
            }
        }
        fn after(&mut self) -> Result<(), Invalid> {
            if self.state.control.error().is_some() {
                Err(stopped())
            } else {
                Ok(())
            }
        }
        fn helper_attach(&mut self, pgid: i32) -> Result<(), Invalid> {
            let mut group = self
                .state
                .control
                .helper
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if pgid <= 0 || group.is_some() {
                return Err(stopped());
            }
            unsafe {
                libc::kill(-pgid, libc::SIGSTOP);
            }
            *group = Some(HelperGroup {
                pgid,
                running: false,
                retiring: false,
                changed: Instant::now(),
            });
            drop(group);
            if self.state.pause_on_attach {
                self.state.control.foreground.store(true, Ordering::Release);
            }
            let previous = self.state.groups.lock().unwrap().last().copied();
            if let Some(previous) = previous {
                assert_ne!(previous, pgid, "distinct owned group generation");
                assert!(self.helper_poll(previous).is_err());
                assert!(self.helper_retire(previous).is_err());
                assert!(self.helper_detach(previous).is_err());
                assert_eq!(
                    self.state
                        .control
                        .helper
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .pgid,
                    pgid
                );
                self.state.stale_rejected.store(true, Ordering::Release);
            }
            self.state.groups.lock().unwrap().push(pgid);
            if self.take_panic(PanicAt::Attach) {
                panic!("private helper registration panic");
            }
            Ok(())
        }
        fn helper_poll(&mut self, pgid: i32) -> Result<(), Invalid> {
            let group = self
                .state
                .control
                .helper
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if group.as_ref().is_none_or(|g| g.pgid != pgid) || self.state.control.error().is_some()
            {
                Err(stopped())
            } else {
                Ok(())
            }
        }
        fn helper_retire(&mut self, pgid: i32) -> Result<(), Invalid> {
            let mut group = self
                .state
                .control
                .helper
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let group = group
                .as_mut()
                .filter(|g| g.pgid == pgid)
                .ok_or_else(stopped)?;
            group.retiring = true;
            Ok(())
        }
        fn helper_detach(&mut self, pgid: i32) -> Result<(), Invalid> {
            let mut group = self
                .state
                .control
                .helper
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if group.as_ref().is_none_or(|g| g.pgid != pgid || !g.retiring) {
                return Err(stopped());
            }
            if unsafe { libc::kill(-pgid, 0) } == 0
                || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                return Err(stopped());
            }
            group.take();
            Ok(())
        }
        fn helper_usage(&mut self, _: i32, _: u64, rss: u64) -> Result<(), Invalid> {
            if rss > storage::HELPERS {
                Err(stopped())
            } else {
                Ok(())
            }
        }
        fn cleanup(&mut self, step: &mut dyn FnMut() -> bool) {
            let active = self
                .state
                .control
                .helper
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_some();
            if active && self.take_panic(PanicAt::Cleanup) {
                panic!("private helper cleanup panic");
            }
            let mut more = true;
            while more {
                c1::cleanup_unit(&self.state.control, || {
                    let at = c1::cpu_ns();
                    for _ in 0..64 {
                        more = step();
                        if !more || c1::cpu_ns().saturating_sub(at) >= 200_000 {
                            break;
                        }
                    }
                });
            }
        }
    }
    struct Run {
        state: Arc<State>,
        stop: Arc<AtomicBool>,
        controller: Option<std::thread::JoinHandle<()>>,
        owner: Option<std::thread::JoinHandle<()>>,
        result: mpsc::Receiver<Result<Vec<Vec<u8>>, Invalid>>,
    }
    impl Run {
        fn start(
            cwd: &Path,
            scripts: Vec<&str>,
            pause: bool,
            panic_at: PanicAt,
            deadline: Duration,
        ) -> Self {
            let scripts = scripts.into_iter().map(str::to_owned).collect::<Vec<_>>();
            Self::start_with(
                cwd,
                SelectedBindings::default(),
                pause,
                panic_at,
                deadline,
                move |cwd| {
                    scripts
                        .into_iter()
                        .map(|script| {
                            selected_command(cwd, "/bin/sh", &["-c".into(), script], None).and_then(
                                |(code, bytes)| {
                                    if code == 0 {
                                        Ok(bytes)
                                    } else {
                                        Err(Invalid("private helper nonzero exit"))
                                    }
                                },
                            )
                        })
                        .collect()
                },
            )
        }
        fn start_with(
            cwd: &Path,
            bindings: SelectedBindings,
            pause: bool,
            panic_at: PanicAt,
            deadline: Duration,
            job: impl FnOnce(&Path) -> Result<Vec<Vec<u8>>, Invalid> + Send + 'static,
        ) -> Self {
            let control = Arc::new(Control {
                cancel: AtomicU8::new(0),
                foreground: AtomicBool::new(false),
                deadline: Instant::now() + deadline,
                reactor: Arc::new(AtomicU64::new(0)),
                memory: storage::Ledger::new(),
                cleanup_cpu: AtomicU64::new(0),
                cleanup_max: AtomicU64::new(0),
                cleanup_overruns: AtomicU64::new(0),
                helper: Mutex::new(None),
            });
            let state = Arc::new(State {
                control,
                groups: Mutex::new(vec![]),
                paused: AtomicBool::new(false),
                stale_rejected: AtomicBool::new(false),
                panic_at: Mutex::new(panic_at),
                marker: cwd.join("ready"),
                pause_on_attach: pause,
            });
            let stop = Arc::new(AtomicBool::new(false));
            let tick_state = state.clone();
            let tick_stop = stop.clone();
            let controller = std::thread::spawn(move || {
                while !tick_stop.load(Ordering::Acquire) {
                    tick_state.control.helper_tick();
                    let runnable = tick_state
                        .control
                        .helper
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .as_ref()
                        .is_some_and(|group| group.running && !group.retiring);
                    // Match the real control owner: active helper quantum is
                    // polled directly; timer sleeps occur only while stopped.
                    if !runnable {
                        std::thread::sleep(Duration::from_micros(100));
                    }
                }
            });
            let (tx, result) = mpsc::channel();
            let worker_state = state.clone();
            let cwd = cwd.to_owned();
            let owner = std::thread::spawn(move || {
                let credit = worker_state
                    .control
                    .memory
                    .reserve(10 * 1024 * 1024)
                    .unwrap();
                let _scope = SelectedScope::enter(
                    Arc::new(bindings),
                    Box::new(Pace {
                        state: worker_state,
                        _credit: credit,
                    }),
                )
                .unwrap();
                let result = job(&cwd);
                tx.send(result).unwrap();
            });
            Self {
                state,
                stop,
                controller: Some(controller),
                owner: Some(owner),
                result,
            }
        }
        #[track_caller]
        fn wait(&self, predicate: impl Fn() -> bool) {
            let until = Instant::now() + Duration::from_secs(2);
            while !predicate() {
                assert!(Instant::now() < until, "private helper witness timed out");
                std::thread::sleep(Duration::from_micros(100));
            }
        }
        fn finish(mut self) -> Result<Vec<Vec<u8>>, Invalid> {
            let result = self
                .result
                .recv_timeout(Duration::from_secs(2))
                .expect("private helper result must settle unattended");
            self.owner.take().unwrap().join().unwrap();
            self.stop.store(true, Ordering::Release);
            self.controller.take().unwrap().join().unwrap();
            assert!(
                self.state
                    .control
                    .helper
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .is_none(),
                "no phantom group settlement"
            );
            for pgid in self.state.groups.lock().unwrap().iter().copied() {
                assert_eq!(unsafe { libc::kill(-pgid, 0) }, -1);
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
            }
            result
        }
    }
    impl Drop for Run {
        fn drop(&mut self) {
            if let Some(owner) = self.owner.take() {
                self.state.control.cancel.store(1, Ordering::Release);
                let _ = owner.join();
            }
            self.stop.store(true, Ordering::Release);
            if let Some(controller) = self.controller.take() {
                let _ = controller.join();
            }
        }
    }
    #[cfg(target_os = "macos")]
    fn stopped_group(pgid: i32) -> bool {
        let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
        let size = std::mem::size_of::<libc::proc_bsdinfo>();
        let got = unsafe { libc::proc_pidinfo(pgid, libc::PROC_PIDTBSDINFO, 0, info.as_mut_ptr().cast(), size as i32) };
        if got != size as i32 { return false; }
        let info = unsafe { info.assume_init() };
        info.pbi_pid == pgid as u32 && info.pbi_pgid == pgid as u32 && info.pbi_status == libc::SSTOP
    }
    #[cfg(not(target_os = "macos"))]
    fn stopped_group(pgid: i32) -> bool {
        let mut info = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
        let code = unsafe {
            libc::waitid(
                libc::P_PID,
                pgid as libc::id_t,
                info.as_mut_ptr(),
                libc::WSTOPPED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        code == 0 && unsafe { info.assume_init() }.si_code == libc::CLD_STOPPED
    }
    #[test]
    fn foreground_pause_and_cancel_reach_real_group_while_proof_before_waits() {
        let home = tempfile::tempdir().unwrap();
        let run = Run::start(
            home.path(),
            vec!["printf ready > ready; while :; do printf x >> ticks; sleep 0.001; done"],
            true,
            PanicAt::None,
            Duration::from_secs(5),
        );
        run.wait(|| run.state.paused.load(Ordering::Acquire));
        let pgid = run.state.groups.lock().unwrap()[0];
        run.wait(|| stopped_group(pgid));
        assert!(
            !home.path().join("ready").exists(),
            "stopped real helper cannot execute while proof before is paused"
        );
        run.state.control.foreground.store(false, Ordering::Release);
        run.wait(|| home.path().join("ticks").exists());
        let at = Instant::now();
        run.state.control.foreground.store(true, Ordering::Release);
        run.wait(|| stopped_group(pgid));
        assert!(at.elapsed() < Duration::from_millis(50));
        let size = std::fs::metadata(home.path().join("ticks")).unwrap().len();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(
            std::fs::metadata(home.path().join("ticks")).unwrap().len(),
            size
        );
        let at = Instant::now();
        run.state.control.cancel.store(1, Ordering::Release);
        assert!(run.finish().is_err());
        assert!(
            at.elapsed() < Duration::from_millis(60),
            "shared cancel settlement bound"
        );
    }
    #[test]
    fn deadline_settles_real_group_while_foreground_before_stays_paused() {
        let home = tempfile::tempdir().unwrap();
        let run = Run::start(
            home.path(),
            vec!["printf unexpected > ready; sleep 30"],
            true,
            PanicAt::None,
            Duration::from_millis(150),
        );
        run.wait(|| run.state.paused.load(Ordering::Acquire));
        assert!(run.finish().is_err());
        assert!(!home.path().join("ready").exists());
    }
    #[test]
    fn current_group_rejects_old_hooks_after_distinct_real_group_settled() {
        let home = tempfile::tempdir().unwrap();
        let run = Run::start(
            home.path(),
            vec!["printf first", "printf second"],
            false,
            PanicAt::None,
            Duration::from_secs(5),
        );
        let state = run.state.clone();
        assert_eq!(
            run.finish().unwrap(),
            vec![b"first".to_vec(), b"second".to_vec()]
        );
        assert!(state.stale_rejected.load(Ordering::Acquire));
    }
    #[test]
    fn descendant_and_panicking_proof_owner_settle_before_refusal() {
        for point in [PanicAt::Attach, PanicAt::Before, PanicAt::Cleanup] {
            let home = tempfile::tempdir().unwrap();
            let run = Run::start(
                home.path(),
                vec![
                    "sleep 30 & printf '%s' $! > descendant; printf ready > ready; while :; do sleep 1; done",
                ],
                false,
                point,
                Duration::from_secs(5),
            );
            let descendant_pid = || std::fs::read_to_string(home.path().join("descendant"))
                .ok().and_then(|text| text.trim().parse::<i32>().ok()).filter(|pid| *pid > 0);
            if point == PanicAt::Cleanup {
                run.wait(|| descendant_pid().is_some());
                run.state.control.cancel.store(1, Ordering::Release);
            }
            let state = run.state.clone();
            assert!(run.finish().is_err());
            assert!(
                *state.panic_at.lock().unwrap_or_else(|e| e.into_inner()) == PanicAt::None,
                "panic witness must execute its injected fault"
            );
            if point != PanicAt::Attach {
                assert!(
                    descendant_pid().is_some(),
                    "descendant witness must actually launch"
                );
            }
            // Attach panic may precede the script's PID publication; the exact
            // owned group ESRCH assertion in finish remains mandatory in all arms.
            if let Some(pid) = descendant_pid() {
                assert_eq!(
                    unsafe { libc::kill(pid, 0) },
                    -1,
                    "owned descendant actually settled"
                );
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
            }
        }
    }
    #[test]
    fn admitted_public_key_uses_exact_stdin_after_physical_file_deleted() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();
        // Complete public-only certificate generated once in a throwaway
        // project fixture keyring; tests require no private key or account.
        let packet = br#"-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEauzM2xYJKwYBBAHaRw8BAQdABoel4ot13JPSQPieD2kERGLP8SsFK0R3sVeV
XU8ECiC0NEJveHkgcHJpdmF0ZSBEcmVhbSByZWFkIGZpeHR1cmUgPGJib3h5NDUw
QGdtYWlsLmNvbT6IrwQTFgoAVxYhBBsSNJ4SabNvVk2kSBPUo1wcshmXBQJq7Mzb
GxSAAAAAAAQADm1hbnUyLDIuNSsxLjEyLDAsMwIbAwULCQgHAgIiAgYVCgkICwIE
FgIDAQIeBwIXgAAKCRAT1KNcHLIZl9vrAP9EwwIr9QFUc936g0varowwXs/OSKj7
ex+0H79O/1fH+gEAutvxjuOWPgOixgmpMN4FZgi8rDsaoZ5XKYDx1uUJHA0=
=/6hw
-----END PGP PUBLIC KEY BLOCK-----
"#
        .to_vec();
        let fingerprint = "1B12349E1269B36F564DA44813D4A35C1CB21997";
        let physical = home.path().join("deleted-public-key.bin");
        std::fs::write(&physical, b"this pathname must never be reopened").unwrap();
        std::fs::remove_file(&physical).unwrap();
        let gpg_home = home.path().join("public-gpg");
        std::fs::create_dir(&gpg_home).unwrap();
        std::fs::set_permissions(&gpg_home, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut admitted = SelectedBindings::default();
        admitted.keys.insert(
            PathBuf::from("/logical/authorized-public-key.bin"),
            physical.clone(),
        );
        admitted.key_bytes.insert(physical.clone(), packet);
        let run = Run::start_with(
            home.path(),
            admitted,
            false,
            PanicAt::None,
            Duration::from_secs(5),
            move |cwd| {
                let mut args = [
                    "--no-options",
                    "--no-autostart",
                    "--batch",
                    "--no-tty",
                    "--with-colons",
                    "--show-keys",
                ]
                .map(str::to_owned)
                .to_vec();
                args.push(physical.to_string_lossy().into_owned());
                let (code, output) =
                    crate::dream::trust::selected_key_command(cwd, &args, Some(&gpg_home))?;
                if code != 0 {
                    return Err(Invalid("private public key inspection failed"));
                }
                let text = std::str::from_utf8(&output).unwrap();
                assert!(text.lines().any(|line| line.starts_with("pub:")));
                assert!(text.contains(&fingerprint));
                assert!(!physical.exists());
                args.last_mut().unwrap().push_str(".unlisted");
                assert!(
                    crate::dream::trust::selected_key_command(cwd, &args, Some(&gpg_home)).is_err()
                );
                Ok(vec![output])
            },
        );
        assert_eq!(run.finish().unwrap().len(), 1);
    }
    #[test]
    fn owned_stdin_streams_multiple_chunks_and_closes_eof() {
        let home = tempfile::tempdir().unwrap();
        let input = (0..64 * 1024).map(|n| (n % 251) as u8).collect::<Vec<_>>();
        let expected = input.clone();
        let run = Run::start_with(
            home.path(),
            SelectedBindings::default(),
            false,
            PanicAt::None,
            Duration::from_secs(5),
            move |cwd| {
                let (code, output) = selected_command_input(
                    cwd,
                    "/bin/sh",
                    &["-c".into(), "/bin/cat".into()],
                    None,
                    Some(&input),
                )?;
                if code != 0 {
                    return Err(Invalid("private stdin child failed"));
                }
                Ok(vec![output])
            },
        );
        assert_eq!(run.finish().unwrap(), vec![expected]);
    }

    #[test]
    fn backpressured_stdin_does_not_block_foreground_or_cancel() {
        let home = tempfile::tempdir().unwrap();
        let input = vec![b'x'; 1024 * 1024];
        let run = Run::start_with(
            home.path(),
            SelectedBindings::default(),
            false,
            PanicAt::None,
            Duration::from_secs(5),
            move |cwd| {
                selected_command_input(
                    cwd,
                    "/bin/sh",
                    &["-c".into(), "printf ready > ready; /bin/sleep 30".into()],
                    None,
                    Some(&input),
                )
                .map(|(_, bytes)| vec![bytes])
            },
        );
        run.wait(|| home.path().join("ready").exists());
        std::thread::sleep(Duration::from_millis(20));
        let at = Instant::now();
        run.state.control.foreground.store(true, Ordering::Release);
        run.wait(|| run.state.paused.load(Ordering::Acquire));
        assert!(
            at.elapsed() < Duration::from_millis(50),
            "backpressured stdin must yield foreground pause"
        );
        let at = Instant::now();
        run.state.control.cancel.store(1, Ordering::Release);
        assert!(run.finish().is_err());
        assert!(
            at.elapsed() < Duration::from_millis(60),
            "backpressured stdin must retain cancel settlement bound"
        );
    }
    #[test]
    fn unknown_spawn_clock_refuses_after_retained_effect_actually_settles() {
        let home = tempfile::tempdir().unwrap();
        let run = Run::start_with(home.path(), SelectedBindings::default(), false, PanicAt::None, Duration::from_secs(5), move |cwd| {
            FAIL_HELPER_CLOCK.with(|failure| failure.set(Some(HelperPhase::Spawn)));
            selected_command(cwd, "/bin/sh", &["-c".into(), "/bin/sleep 30".into()], None).map(|(_, bytes)| vec![bytes])
        });
        let state = run.state.clone();
        assert!(run.finish().is_err());
        assert_eq!(state.groups.lock().unwrap().len(), 1, "failed clock retains and registers the real spawned effect for cleanup");
    }

}
