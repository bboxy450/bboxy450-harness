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
        Item::Native {
            blob: Some(blob), ..
        } => add(out, blob),
        Item::UserMessage { blocks } => {
            for b in blocks {
                if let InputBlock::Image { blob }
                | InputBlock::Audio { blob }
                | InputBlock::Resource {
                    blob: Some(blob), ..
                } = b
                {
                    add(out, blob);
                }
            }
        }
        Item::ToolCall { output, spill, .. } => {
            if let Some(b) = spill {
                add(out, b);
            }
            for b in output {
                if let OutputBlock::Image { blob }
                | OutputBlock::Audio { blob }
                | OutputBlock::Resource {
                    blob: Some(blob), ..
                } = b
                {
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
pub(crate) fn publish(temp: &Path, path: &Path) -> io::Result<()> {
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
fn copy_prefix(writer: &mut impl Write, lines: &[&[u8]]) -> io::Result<()> {
    for line in lines.iter().skip(1) {
        writer.write_all(line)?;
    }
    Ok(())
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
    let mut lines = vec![];
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        if prefix.len() as u64 == at.0 {
            break;
        }
        if !line.ends_with(b"\n") {
            return Err(invalid());
        }
        let e: Envelope = serde_json::from_slice(line).map_err(|_| invalid())?;
        if e.seq.is_none() || matches!(e.event, Event::SessionClosed) {
            return Err(invalid());
        }
        prefix.push(e);
        lines.push(line);
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
            raw: None,
            ext: Default::default(),
            native: None,
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
        copy_prefix(&mut file, &lines)?;
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

#[cfg(test)]
mod controls {
    use super::*;
    fn copy_with_control(lines: &[&[u8]], reserialize: bool) -> Vec<u8> {
        let mut bytes = vec![];
        if reserialize {
            for line in lines.iter().skip(1) {
                let e: Envelope = serde_json::from_slice(line).unwrap();
                serde_json::to_writer(&mut bytes, &e).unwrap();
                bytes.push(b'\n');
            }
        } else {
            copy_prefix(&mut bytes, lines).unwrap();
        }
        bytes
    }
    #[test]
    fn reserializing_fork_fails_byte_identity() {
        let line = concat!(
            r#"{"v":{"major":1,"minor":99},"session":"00000000000000000000000001","epoch":1,"seq":2,"at":2,"ts_ms":0,"cause":null,"event":{"type":"future_event","future_field":42},"future_envelope_field":true}"#,
            "\n"
        );
        let lines = [b"header\n".as_slice(), line.as_bytes()];
        // Deliberately broken old path, selected only in the unit-test build.
        let broken = copy_with_control(&lines, true);
        assert!(std::panic::catch_unwind(|| assert_eq!(broken, line.as_bytes())).is_err());
        assert_eq!(copy_with_control(&lines, false), line.as_bytes());
        println!("broken reserializing fork: byte-identity assertion rejected");
    }
}
