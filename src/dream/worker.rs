// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Narrow shared owner facade. The accepted C1 scheduler remains the sole scheduler.
//! Independent inspiration credit requested; retain the absolute policy above.
pub(crate) use super::c1::{
    Budget, Completed, Control, Counters, Dir, Frame, Job, Paced, cleanup_unit, cpu_ns, disposal,
    framed, heaps, read_bounded, rss, run, storage,
};

use super::{c1_wire as c, read_wire::BundleFile};
use disposal::Drain;
use std::{
    collections::BTreeMap,
    fs::{File, Metadata},
    mem::ManuallyDrop,
    os::{
        fd::{FromRawFd, IntoRawFd},
        unix::fs::MetadataExt,
    },
};

const INVENTORY_FILES: usize = 2048;
const INVENTORY_DIRS: usize = 2048;
const INVENTORY_PATH: usize = 4096;

#[derive(Clone, Copy)]
enum ExpectedKind {
    Directory,
    File(u64),
    Manifest,
}
#[derive(Clone, Copy)]
struct Expected {
    kind: ExpectedKind,
    seen: bool,
    files: usize,
}
struct InventoryPlan<'a> {
    entries: BTreeMap<&'a str, Expected>,
    directories: Vec<&'a str>,
}
impl Drain for InventoryPlan<'_> {
    fn drain_step(&mut self) -> bool {
        self.entries.pop_first().is_some() || self.directories.pop().is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CustodyStamp {
    device: u64,
    inode: u64,
    len: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
}
impl From<&Metadata> for CustodyStamp {
    fn from(m: &Metadata) -> Self {
        Self {
            device: m.dev(),
            inode: m.ino(),
            len: m.len(),
            modified: m.mtime(),
            modified_ns: m.mtime_nsec(),
            changed: m.ctime(),
            changed_ns: m.ctime_nsec(),
        }
    }
}
struct CustodyFile {
    name: String,
    stamp: CustodyStamp,
}
struct CustodyDirectory {
    path: String,
    stamp: CustodyStamp,
    files: Vec<CustodyFile>,
}
impl Drain for CustodyDirectory {
    fn drain_step(&mut self) -> bool {
        self.files.pop().is_some()
    }
}
/// Private read-custody continuity, never an authority/proof-of-content flag.
/// Own only admitted names and their original inode/time identities; retain the
/// associated storage credit through the complete proof and final revalidation.
pub(crate) struct CustodySeal {
    directories: Vec<CustodyDirectory>,
}
impl Drain for CustodySeal {
    fn drain_step(&mut self) -> bool {
        self.directories.drain_step()
    }
}
fn owned_path(path: &str) -> Result<String, c::Code> {
    let mut value = String::new();
    value
        .try_reserve_exact(path.len())
        .map_err(|_| c::Code::ResourceLimit)?;
    value.push_str(path);
    Ok(value)
}

/// The only filesystem names visited are the directories derived from the
/// supplied inventory. Discovered names are compared against that closed set;
/// they never become traversal instructions, evidence or an enlarged allowlist.
#[cfg(test)]
pub(crate) fn inventory_closed(
    input: &Dir,
    files: &[BundleFile],
    b: &mut Budget,
) -> Result<(), c::Code> {
    let seal = capture_custody(input, files, b)?;
    let _seal = Paced::new(seal, &b.control, Drain::drain_step);
    Ok(())
}

pub(crate) fn capture_custody(
    input: &Dir,
    files: &[BundleFile],
    b: &mut Budget,
) -> Result<storage::Tracked<CustodySeal>, c::Code> {
    if files.len() > INVENTORY_FILES {
        return Err(c::Code::ResourceLimit);
    }
    // Manifest strings already own the borrowed keys. Reserve the bounded
    // directory index and conservative declared BTree-entry slots before growth;
    // libc's one directory-stream buffer remains existing helper headroom.
    let bytes = ((INVENTORY_FILES + INVENTORY_DIRS + 1) * 256
        + INVENTORY_DIRS * std::mem::size_of::<&str>()) as u64;
    let plan = b.track(bytes, || {
        let mut directories = Vec::new();
        directories
            .try_reserve_exact(INVENTORY_DIRS)
            .map_err(|_| c::Code::ResourceLimit)?;
        directories.push("");
        let mut entries = BTreeMap::new();
        entries.insert(
            "",
            Expected {
                kind: ExpectedKind::Directory,
                seen: true,
                files: 1,
            },
        );
        entries.insert(
            "manifest.json",
            Expected {
                kind: ExpectedKind::Manifest,
                seen: false,
                files: 0,
            },
        );
        Ok(InventoryPlan {
            entries,
            directories,
        })
    })?;
    let mut plan = Paced::new(plan, &b.control, Drain::drain_step);
    let mut previous = "";
    let mut slot = 0;
    let mut component = 0;
    let mut validated = false;
    while slot < files.len() {
        b.unit(|| {
            let at = cpu_ns();
            for _ in 0..64 {
                let Some(file) = files.get(slot) else {
                    break;
                };
                let path = file.path.0.as_str();
                let relative = path.strip_prefix("input/").ok_or(c::Code::BadRequest)?;
                if !validated {
                    if file.slot as usize != slot
                        || path <= previous
                        || path.len() > INVENTORY_PATH
                        || relative.is_empty()
                        || relative == "manifest.json"
                        || relative.contains(['\\', '\0'])
                        || relative
                            .split('/')
                            .any(|p| p.is_empty() || matches!(p, "." | ".."))
                    {
                        return Err(c::Code::BadRequest);
                    }
                    previous = path;
                    validated = true;
                }
                if let Some(slash) = relative[component..].find('/') {
                    let end = component + slash;
                    let directory = &relative[..end];
                    if let Some(expected) = plan.entries.get(directory) {
                        if !matches!(expected.kind, ExpectedKind::Directory) {
                            return Err(c::Code::BadRequest);
                        }
                    } else {
                        if plan.directories.len() == INVENTORY_DIRS {
                            return Err(c::Code::ResourceLimit);
                        }
                        plan.entries.insert(
                            directory,
                            Expected {
                                kind: ExpectedKind::Directory,
                                seen: false,
                                files: 0,
                            },
                        );
                        plan.directories.push(directory);
                    }
                    component = end + 1;
                } else {
                    if plan
                        .entries
                        .insert(
                            relative,
                            Expected {
                                kind: ExpectedKind::File(file.len.0),
                                seen: false,
                                files: 0,
                            },
                        )
                        .is_some()
                    {
                        return Err(c::Code::BadRequest);
                    }
                    let parent = relative.rsplit_once('/').map_or("", |(parent, _)| parent);
                    plan.entries
                        .get_mut(parent)
                        .ok_or(c::Code::BadRequest)?
                        .files += 1;
                    slot += 1;
                    component = 0;
                    validated = false;
                }
                if cpu_ns().saturating_sub(at) >= 400_000 {
                    break;
                }
            }
            Ok(())
        })?;
    }
    // Exact requested Vec slots and owned UTF-8 name lengths, before allocation.
    // All names are fixed by the admitted plan; no found directory entry sizes
    // this reservation or enlarges the selected set.
    let mut seal_bytes = plan
        .directories
        .len()
        .checked_mul(std::mem::size_of::<CustodyDirectory>())
        .ok_or(c::Code::ResourceLimit)?;
    let mut expected = plan.entries.iter().peekable();
    while expected.peek().is_some() {
        b.unit(|| {
            let at = cpu_ns();
            for _ in 0..64 {
                let Some((path, value)) = expected.next() else {
                    break;
                };
                let size = if matches!(value.kind, ExpectedKind::Directory) {
                    path.len()
                } else {
                    std::mem::size_of::<CustodyFile>() + path.rsplit('/').next().unwrap().len()
                };
                seal_bytes = seal_bytes.checked_add(size).ok_or(c::Code::ResourceLimit)?;
                if cpu_ns().saturating_sub(at) >= 400_000 {
                    break;
                }
            }
            Ok(())
        })?;
    }
    let seal = b.track(seal_bytes as u64, || {
        let mut directories = Vec::new();
        directories
            .try_reserve_exact(plan.directories.len())
            .map_err(|_| c::Code::ResourceLimit)?;
        Ok(CustodySeal { directories })
    })?;
    let mut seal = Paced::new(seal, &b.control, Drain::drain_step);
    // Copy only an admitted borrowed name; one stream and at most three transient
    // directory descriptions are live, regardless of selected path depth/count.
    for index in 0..plan.directories.len() {
        let relative = plan.directories[index];
        let held = expected_directory(input, relative, b)?;
        let directory = held.as_ref().unwrap_or(input);
        let stream = b.unit(|| DirectoryStream::open(directory.inventory_fd()?))?;
        let mut stream = Paced::new(stream, &b.control, Drain::drain_step);
        let captured = b.unit(|| {
            let mut files = Vec::new();
            files
                .try_reserve_exact(plan.entries[relative].files)
                .map_err(|_| c::Code::ResourceLimit)?;
            Ok(CustodyDirectory {
                path: owned_path(relative)?,
                stamp: CustodyStamp::from(&stream.before),
                files,
            })
        })?;
        let mut captured = Paced::new(captured, &b.control, Drain::drain_step);
        let mut ended = false;
        while !ended {
            // Each bounded action consumes exactly one readdir result. At most
            // 64 simple actions share a unit, stopping at 400us of thread CPU;
            // the unchanged outer Budget still rejects any whole unit over 1ms.
            b.unit(|| {
                let at = cpu_ns();
                for _ in 0..64 {
                    let Some(name) = stream.next()? else {
                        ended = true;
                        break;
                    };
                    let name = name.as_str()?;
                    if !matches!(name, "." | "..") {
                        let mut path = [0u8; INVENTORY_PATH];
                        let prefix = if relative.is_empty() {
                            0
                        } else {
                            path[..relative.len()].copy_from_slice(relative.as_bytes());
                            path[relative.len()] = b'/';
                            relative.len() + 1
                        };
                        let len = prefix
                            .checked_add(name.len())
                            .filter(|n| *n <= path.len())
                            .ok_or(c::Code::SourceChanged)?;
                        path[prefix..len].copy_from_slice(name.as_bytes());
                        let key = std::str::from_utf8(&path[..len])
                            .map_err(|_| c::Code::SourceChanged)?;
                        let admitted = *plan
                            .entries
                            .get_key_value(key)
                            .ok_or(c::Code::SourceChanged)?
                            .0;
                        let expected = plan
                            .entries
                            .get_mut(admitted)
                            .ok_or(c::Code::SourceChanged)?;
                        if expected.seen {
                            return Err(c::Code::SourceChanged);
                        }
                        match expected.kind {
                            ExpectedKind::Directory => {
                                directory.dir(name)?;
                            }
                            ExpectedKind::File(len) => {
                                let (_, metadata) = directory.file(name, len)?;
                                if metadata.len() != len {
                                    return Err(c::Code::SourceChanged);
                                }
                                captured.files.push(CustodyFile {
                                    name: owned_path(admitted.rsplit('/').next().unwrap())?,
                                    stamp: CustodyStamp::from(&metadata),
                                });
                            }
                            ExpectedKind::Manifest => {
                                let (_, metadata) = directory.file(name, c::MAX_FRAME as u64)?;
                                captured.files.push(CustodyFile {
                                    name: owned_path(admitted)?,
                                    stamp: CustodyStamp::from(&metadata),
                                });
                            }
                        }
                        expected.seen = true;
                    }
                    if cpu_ns().saturating_sub(at) >= 400_000 {
                        break;
                    }
                }
                Ok(())
            })?;
        }
        // Reopen only this admitted directory from the original input fd. This
        // detects pathname replacement as well as mutation of the held stream.
        let current = expected_directory(input, relative, b)?;
        b.unit(|| {
            let file = current.as_ref().unwrap_or(input).inventory_fd()?;
            stream.unchanged(&file)
        })?;
        b.unit(|| {
            seal.directories.push(captured.take_value());
            Ok(())
        })?;
    }
    let mut expected = plan.entries.values().peekable();
    while expected.peek().is_some() {
        b.unit(|| {
            let at = cpu_ns();
            for _ in 0..64 {
                let Some(value) = expected.next() else {
                    break;
                };
                if !value.seen {
                    return Err(c::Code::SourceChanged);
                }
                if cpu_ns().saturating_sub(at) >= 400_000 {
                    break;
                }
            }
            Ok(())
        })?;
    }
    Ok(seal.take_value())
}

/// Compare both held and reopened descriptor-relative paths with the initial
/// custody cut. Identical bytes do not forgive a rename/ref swap or a mutation
/// followed by restoration. This performs no content lookup or authority claim.
pub(crate) fn validate_custody(
    input: &Dir,
    seal: &CustodySeal,
    b: &mut Budget,
) -> Result<(), c::Code> {
    for captured in &seal.directories {
        let held = expected_directory(input, &captured.path, b).map_err(custody_change)?;
        let directory = held.as_ref().unwrap_or(input);
        b.unit(|| {
            let file = directory
                .inventory_fd()
                .map_err(|_| c::Code::SourceChanged)?;
            let metadata = file.metadata().map_err(|_| c::Code::SourceChanged)?;
            if CustodyStamp::from(&metadata) != captured.stamp {
                return Err(c::Code::SourceChanged);
            }
            Ok(())
        })?;
        let mut files = captured.files.iter().peekable();
        while files.peek().is_some() {
            b.unit(|| {
                let at = cpu_ns();
                for _ in 0..64 {
                    let Some(captured) = files.next() else {
                        break;
                    };
                    let (_, metadata) = directory
                        .file(&captured.name, captured.stamp.len)
                        .map_err(|_| c::Code::SourceChanged)?;
                    if CustodyStamp::from(&metadata) != captured.stamp {
                        return Err(c::Code::SourceChanged);
                    }
                    if cpu_ns().saturating_sub(at) >= 400_000 {
                        break;
                    }
                }
                Ok(())
            })?;
        }
        let current = expected_directory(input, &captured.path, b).map_err(custody_change)?;
        b.unit(|| {
            let file = current
                .as_ref()
                .unwrap_or(input)
                .inventory_fd()
                .map_err(|_| c::Code::SourceChanged)?;
            let metadata = file.metadata().map_err(|_| c::Code::SourceChanged)?;
            if CustodyStamp::from(&metadata) != captured.stamp {
                return Err(c::Code::SourceChanged);
            }
            Ok(())
        })?;
    }
    Ok(())
}
fn custody_change(code: c::Code) -> c::Code {
    match code {
        c::Code::StorageUnavailable | c::Code::SourceUnavailable => c::Code::SourceChanged,
        code => code,
    }
}

fn expected_directory(
    input: &Dir,
    relative: &str,
    b: &mut Budget,
) -> Result<Paced<Option<Dir>>, c::Code> {
    let mut current = Paced::new(None, &b.control, Drain::drain_step);
    if !relative.is_empty() {
        // This lexical path was already admitted by the finite inventory (or
        // retained seal). Keep its <=4096-byte traversal in one measured unit;
        // old/new directory fds overlap only until the next assignment. The
        // unchanged Budget rejects the complete traversal if it exceeds 1ms.
        b.unit(|| {
            for name in relative.split('/') {
                let next = current.as_ref().unwrap_or(input).dir(name)?;
                *current = Some(next);
            }
            Ok(())
        })?;
    }
    Ok(current)
}

struct EntryName {
    bytes: [u8; 1024],
    len: usize,
}
impl EntryName {
    fn as_str(&self) -> Result<&str, c::Code> {
        std::str::from_utf8(&self.bytes[..self.len]).map_err(|_| c::Code::SourceChanged)
    }
}
struct DirectoryStream {
    stream: *mut libc::DIR,
    before: Metadata,
}
impl DirectoryStream {
    fn open(file: File) -> Result<Self, c::Code> {
        let before = file.metadata().map_err(|_| c::Code::StorageUnavailable)?;
        let fd = file.into_raw_fd();
        let stream = unsafe { libc::fdopendir(fd) };
        if stream.is_null() {
            unsafe {
                libc::close(fd);
            }
            return Err(c::Code::StorageUnavailable);
        }
        Ok(Self { stream, before })
    }
    fn next(&mut self) -> Result<Option<EntryName>, c::Code> {
        #[cfg(target_os = "macos")]
        let errno = unsafe { libc::__error() };
        #[cfg(target_os = "linux")]
        let errno = unsafe { libc::__errno_location() };
        unsafe {
            *errno = 0;
        }
        let entry = unsafe { libc::readdir(self.stream) };
        if entry.is_null() {
            return if unsafe { *errno } == 0 {
                Ok(None)
            } else {
                Err(c::Code::StorageUnavailable)
            };
        }
        let raw = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
        if raw.is_empty() || raw.len() > 1023 || raw.contains(&b'/') {
            return Err(c::Code::SourceChanged);
        }
        let mut name = EntryName {
            bytes: [0; 1024],
            len: raw.len(),
        };
        name.bytes[..raw.len()].copy_from_slice(raw);
        Ok(Some(name))
    }
    fn unchanged(&self, current: &File) -> Result<(), c::Code> {
        let held = ManuallyDrop::new(unsafe { File::from_raw_fd(libc::dirfd(self.stream)) });
        let held = held.metadata().map_err(|_| c::Code::SourceChanged)?;
        let current = current.metadata().map_err(|_| c::Code::SourceChanged)?;
        let identity = |m: &Metadata| {
            (
                m.dev(),
                m.ino(),
                m.len(),
                m.mtime(),
                m.mtime_nsec(),
                m.ctime(),
                m.ctime_nsec(),
            )
        };
        if identity(&self.before) == identity(&held) && identity(&held) == identity(&current) {
            Ok(())
        } else {
            Err(c::Code::SourceChanged)
        }
    }
}
impl Drain for DirectoryStream {
    fn drain_step(&mut self) -> bool {
        false
    }
}
impl Drop for DirectoryStream {
    fn drop(&mut self) {
        unsafe {
            libc::closedir(self.stream);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::PermissionsExt,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU8, AtomicU64},
        },
        time::{Duration, Instant},
    };
    fn budget() -> Budget {
        let control = Arc::new(Control {
            cancel: AtomicU8::new(0),
            foreground: AtomicBool::new(false),
            deadline: Instant::now() + Duration::from_secs(30),
            reactor: Arc::new(AtomicU64::new(0)),
            memory: storage::Ledger::new(),
            cleanup_cpu: AtomicU64::new(0),
            cleanup_max: AtomicU64::new(0),
            cleanup_overruns: AtomicU64::new(0),
            helper: std::sync::Mutex::new(None),
        });
        Budget {
            control,
            progress: Arc::new(crossbeam_queue::ArrayQueue::new(32)),
            counters: Counters {
                phase: c::Phase::Validate,
                bytes: 0,
                envelopes: 0,
                cpu: 0,
            },
            max_unit: 0,
            heap_peak: 0,
            overruns: 0,
            last_end: None,
            paused: false,
            normalization_ready: false,
            normalization_units: [0; 4],
        }
    }
    fn fixture() -> (tempfile::TempDir, Dir, Vec<BundleFile>) {
        fixture_at("evidence/data")
    }
    fn fixture_at(relative: &str) -> (tempfile::TempDir, Dir, Vec<BundleFile>) {
        let temp = tempfile::Builder::new()
            .prefix("dream-read-closure-")
            .tempdir_in("/private/tmp")
            .unwrap();
        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut parent = temp.path().to_path_buf();
        for name in relative
            .rsplit_once('/')
            .map_or("", |(parent, _)| parent)
            .split('/')
            .filter(|p| !p.is_empty())
        {
            parent.push(name);
            std::fs::create_dir(&parent).unwrap();
            std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        for (path, bytes) in [
            (temp.path().join("manifest.json"), b"{}".as_slice()),
            (temp.path().join(relative), b"pin".as_slice()),
        ] {
            std::fs::write(&path, bytes).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let files = vec![BundleFile {
            slot: 0,
            path: format!("input/{relative}").try_into().unwrap(),
            len: c::Number(3),
            sha256: c::hash(b"pin"),
        }];
        let dir = Dir::open(temp.path()).unwrap();
        (temp, dir, files)
    }
    #[test]
    fn inventory_closed_accepts_exact_custody_on_fresh_streams() {
        let (_temp, dir, files) = fixture();
        inventory_closed(&dir, &files, &mut budget()).unwrap();
        inventory_closed(&dir, &files, &mut budget()).unwrap();
    }
    #[test]
    fn inventory_closed_refuses_unknown_fifo_without_opening_data() {
        let (temp, dir, files) = fixture();
        let path =
            std::ffi::CString::new(temp.path().join("unlisted").as_os_str().as_encoded_bytes())
                .unwrap();
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        assert_eq!(
            inventory_closed(&dir, &files, &mut budget()),
            Err(c::Code::SourceChanged)
        );
    }
    #[test]
    fn inventory_closed_refuses_unlisted_directory_missing_file_and_symlink() {
        let (temp, dir, files) = fixture();
        let unexpected = temp.path().join("unexpected");
        std::fs::create_dir(&unexpected).unwrap();
        assert_eq!(
            inventory_closed(&dir, &files, &mut budget()),
            Err(c::Code::SourceChanged)
        );
        std::fs::remove_dir(unexpected).unwrap();
        let data = temp.path().join("evidence/data");
        std::fs::remove_file(&data).unwrap();
        assert_eq!(
            inventory_closed(&dir, &files, &mut budget()),
            Err(c::Code::SourceChanged)
        );
        std::os::unix::fs::symlink(temp.path().join("manifest.json"), data).unwrap();
        assert_eq!(
            inventory_closed(&dir, &files, &mut budget()),
            Err(c::Code::StorageUnavailable)
        );
    }
    #[test]
    fn custody_seal_retains_credit_and_exact_stamps_until_stepped_disposal() {
        let (_temp, dir, files) = fixture();
        let mut b = budget();
        let captured = capture_custody(&dir, &files, &mut b).unwrap();
        let captured = Paced::new(captured, &b.control, Drain::drain_step);
        assert!(
            b.control.memory.snapshot().0 > 0,
            "owned custody names retain their credit"
        );
        validate_custody(&dir, &captured, &mut b).unwrap();
        drop(captured);
        assert_eq!(
            b.control.memory.snapshot().0,
            0,
            "credit follows actual stepped disposal"
        );
    }
    #[test]
    fn custody_seal_refuses_directory_aba_ref_swap_and_identical_byte_rewrite() {
        for mutation in 0..3 {
            let relative = ".git/refs/remotes/origin/main";
            let (temp, dir, files) = fixture_at(relative);
            let mut b = budget();
            let captured = capture_custody(&dir, &files, &mut b).unwrap();
            let captured = Paced::new(captured, &b.control, Drain::drain_step);
            let path = temp.path().join(relative);
            match mutation {
                0 => {
                    let original = temp.path().join(".git/refs");
                    let moved = temp.path().join(".git/refs-aba");
                    std::fs::rename(&original, &moved).unwrap();
                    std::fs::rename(&moved, &original).unwrap();
                }
                1 => {
                    let replacement = path.with_file_name("ref-replacement");
                    std::fs::write(&replacement, b"pin").unwrap();
                    std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                    std::fs::rename(replacement, &path).unwrap();
                }
                _ => {
                    std::fs::write(&path, b"pin").unwrap();
                }
            }
            assert_eq!(c::hash(std::fs::read(&path).unwrap()), files[0].sha256);
            // A fresh exact-membership/length cut accepts these same bytes. Only
            // the retained initial identities detect the intervening mutation.
            inventory_closed(&dir, &files, &mut budget()).unwrap();
            assert_eq!(
                validate_custody(&dir, &captured, &mut b),
                Err(c::Code::SourceChanged),
                "mutation {mutation}"
            );
        }
    }
}
