// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! A fork publishes an immutable, self-contained prefix; the parent is never opened for writing.
use super::*;
use std::collections::BTreeMap;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "invalid fork point or parent")
}
fn references(body: &Item, out: &mut BTreeMap<String, BlobRef>) {
    fn add(out: &mut BTreeMap<String, BlobRef>, b: &BlobRef) {
        out.insert(b.sha256.clone(), b.clone());
    }
    match body {
        Item::UserMessage { blocks } => {
            for b in blocks {
                if let InputBlock::Image { blob } = b {
                    add(out, blob);
                }
            }
        }
        Item::ToolCall { output, spill, .. } => {
            if let Some(b) = spill {
                add(out, b);
            }
            for b in output {
                if let OutputBlock::Image { blob } = b {
                    add(out, blob);
                }
            }
        }
        Item::Terminal {
            output: Some(b), ..
        }
        | Item::Image { blob: b, .. }
        | Item::Compaction { summary: b, .. } => add(out, b),
        _ => {}
    }
}
fn valid_blob(path: &Path, blob: &BlobRef) -> io::Result<bool> {
    use std::io::Read;
    let mut file = File::open(path)?;
    if file.metadata()?.len() != blob.len {
        return Ok(false);
    }
    let mut hash = Sha256::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        hash.update(&chunk[..n]);
    }
    Ok(format!("{:x}", hash.finalize()) == blob.sha256)
}
fn publish(temp: &Path, path: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let from = std::ffi::CString::new(temp.as_os_str().as_bytes()).map_err(io::Error::other)?;
    let to = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)?;
    #[cfg(target_os = "macos")]
    let result = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
    #[cfg(target_os = "linux")]
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        std::fs::hard_link(temp, path)?;
        return std::fs::remove_file(temp);
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
pub(super) fn materialize(
    root: &Path,
    path: &Path,
    meta: &SessionMeta,
    cause: Option<OpId>,
) -> io::Result<()> {
    let (parent, at) = meta.fork.ok_or_else(invalid)?;
    if at.0 < 2 {
        return Err(invalid());
    }
    let bytes = std::fs::read(log_path(root, parent)).map_err(|_| invalid())?;
    let mut prefix = vec![];
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        if prefix.len() as u64 == at.0 {
            break;
        }
        if !line.ends_with(b"\n") {
            return Err(invalid());
        }
        let e: Envelope = serde_json::from_slice(line).map_err(|_| invalid())?;
        if e.session != parent || !e.event.is_durable() || matches!(e.event, Event::SessionClosed) {
            return Err(invalid());
        }
        prefix.push(e);
    }
    if prefix.len() as u64 != at.0
        || !matches!(&prefix.first().ok_or_else(invalid)?.event,Event::SessionStarted{meta} if meta.session==parent)
    {
        return Err(invalid());
    }
    crate::protocol::laws::check_transcript(&prefix).map_err(|_| invalid())?;
    let mut blobs = BTreeMap::new();
    for e in &prefix {
        if let Event::ItemCompleted { body, .. } = &e.event {
            references(body, &mut blobs);
        }
    }
    let directory = root.join(meta.session.to_string()).join("blobs");
    std::fs::create_dir_all(&directory)?;
    for b in blobs.values() {
        if b.sha256.len() != 64 || !b.sha256.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(invalid());
        }
        let from = root.join(parent.to_string()).join("blobs").join(&b.sha256);
        let to = directory.join(&b.sha256);
        if !valid_blob(&from, b).map_err(|_| invalid())? {
            return Err(invalid());
        }
        if !to.exists() && std::fs::hard_link(&from, &to).is_err() {
            let mut source = File::open(&from)?;
            let mut dest = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&to)?;
            std::io::copy(&mut source, &mut dest)?;
            crate::runtime::sync_all(&dest)?;
        }
        if !valid_blob(&to, b).map_err(|_| invalid())? {
            return Err(invalid());
        }
    }
    crate::runtime::sync_all(&File::open(&directory)?)?;
    let temp = root.join(format!(".fork-{}-{}.tmp", meta.session, OpId::new()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&temp)?;
        let first = Envelope {
            v: PROTOCOL,
            session: meta.session,
            epoch: Epoch(1),
            seq: Some(Seq(1)),
            at: Seq(1),
            ts_ms: meta.created_ms,
            cause,
            event: Event::SessionStarted { meta: meta.clone() },
        };
        serde_json::to_writer(&mut file, &first)?;
        file.write_all(b"\n")?;
        for mut e in prefix.into_iter().skip(1) {
            e.session = meta.session;
            e.epoch = Epoch(1);
            serde_json::to_writer(&mut file, &e)?;
            file.write_all(b"\n")?;
        }
        crate::runtime::sync_all(&file)?;
        write_epoch(&root.join(meta.session.to_string()), 1)?;
        publish(&temp, path)?;
        crate::runtime::sync_all(&File::open(root)?)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(temp);
    }
    result
}
