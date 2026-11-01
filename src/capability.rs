// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! One face-start admission read, before providers, tools or MCP start.
// Adapted from bboxy450-term's descriptor-validated admission consumer:
// https://github.com/bboxy450/bboxy450-term/blob/main/crates/bboxy450-harness-wake-up/src/capability.rs
// Its origin and reuse policy: https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
use std::{
    fs::{Metadata, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
use zeroize::Zeroizing;

/// Only fixed codes cross the face boundary: never a path, value or OS message.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    MissingFile,
    Symlink,
    NotRegularFile,
    WrongOwner,
    WrongMode,
    Malformed,
    Unavailable,
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingFile => "seat_admission_file_missing",
            Self::Symlink => "seat_admission_file_symlink",
            Self::NotRegularFile => "seat_admission_file_not_regular",
            Self::WrongOwner => "seat_admission_file_wrong_owner",
            Self::WrongMode => "seat_admission_file_wrong_mode",
            Self::Malformed => "seat_admission_file_malformed",
            Self::Unavailable => "seat_admission_file_unavailable",
        })
    }
}
impl std::fmt::Debug for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for Refusal {}
fn io_refusal(error: std::io::Error) -> Refusal {
    match error.raw_os_error() {
        Some(libc::ELOOP) => Refusal::Symlink,
        Some(libc::ENOENT) => Refusal::MissingFile,
        _ => Refusal::Unavailable,
    }
}
fn validate(meta: &Metadata) -> Result<(), Refusal> {
    if meta.file_type().is_symlink() {
        return Err(Refusal::Symlink);
    }
    if !meta.is_file() {
        return Err(Refusal::NotRegularFile);
    }
    // SAFETY: geteuid has no arguments or borrowed memory.
    if meta.uid() != unsafe { libc::geteuid() } {
        return Err(Refusal::WrongOwner);
    }
    if meta.mode() & 0o7777 != 0o600 {
        return Err(Refusal::WrongMode);
    }
    Ok(())
}
fn read(path: &Path) -> Result<Zeroizing<String>, Refusal> {
    validate(&std::fs::symlink_metadata(path).map_err(io_refusal)?)?;
    let file = OpenOptions::new()
        .read(true)
        // A replacement FIFO cannot hang open before descriptor validation.
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(io_refusal)?;
    validate(&file.metadata().map_err(io_refusal)?)?; // fstat, not another path lookup.
    let mut bytes = Zeroizing::new(Vec::with_capacity(66));
    file.take(66).read_to_end(&mut bytes).map_err(io_refusal)?;
    if bytes.len() != 65
        || bytes[64] != b'\n'
        || !bytes[..64]
            .iter()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
    {
        return Err(Refusal::Malformed);
    }
    Ok(Zeroizing::new(
        String::from_utf8(bytes[..64].to_vec()).map_err(|_| Refusal::Malformed)?,
    ))
}

/// Called by each face before it creates a provider or a session. File IO runs
/// on a preparation worker; refusal records a safe Notice and never aborts it.
pub async fn register(set: &crate::redact::Set) {
    if set.capability_initialized() {
        return;
    }
    let path = std::env::var_os("BBOXY_CAPABILITY_FILE");
    let result = crate::runtime::blocking(move || {
        path.ok_or(Refusal::MissingFile)
            .and_then(|path| read(Path::new(&path)))
    })
    .await;
    match result {
        Ok(value) => set.register_capability(&value),
        Err(reason) => set.refuse_capability(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn file(root: &Path, text: &[u8]) -> std::path::PathBuf {
        let path = root.join("private-admission");
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        path
    }
    #[test]
    fn canonical_file_is_bounded_and_reading_preserves_it() {
        let own = tempfile::tempdir().unwrap();
        let text = format!("{}\n", "ab".repeat(32));
        let path = file(own.path(), text.as_bytes());
        assert!(read(&path).is_ok_and(|value| value.as_str() == text.trim_end()));
        assert!(read(&path).is_ok());
        assert!(path.is_file());
        for invalid in [
            "ab".repeat(32),
            format!("{}\n", "AB".repeat(32)),
            format!("{text}x"),
            "a".repeat(65536),
        ] {
            std::fs::write(&path, invalid).unwrap();
            assert_eq!(read(&path).err(), Some(Refusal::Malformed));
        }
    }
    #[test]
    fn unsafe_shape_refusals_never_reveal_the_path_or_value() {
        let own = tempfile::tempdir().unwrap();
        let path = file(own.path(), format!("{}\n", "ab".repeat(32)).as_bytes());
        let link = own.path().join("secret-path-marker");
        symlink(&path, &link).unwrap();
        assert_eq!(read(&link).err(), Some(Refusal::Symlink));
        assert_eq!(read(own.path()).err(), Some(Refusal::NotRegularFile));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();
        assert_eq!(read(&path).err(), Some(Refusal::WrongMode));
        assert_eq!(
            read(&own.path().join("absent")).err(),
            Some(Refusal::MissingFile)
        );
        for reason in [
            Refusal::Symlink,
            Refusal::NotRegularFile,
            Refusal::WrongOwner,
            Refusal::WrongMode,
            Refusal::Malformed,
            Refusal::MissingFile,
            Refusal::Unavailable,
        ] {
            assert!(reason.to_string().starts_with("seat_admission_file_"));
            assert_eq!(reason.to_string(), format!("{reason:?}"));
            assert!(!reason.to_string().contains("secret-path-marker"));
        }
    }
    #[test]
    fn foreign_owned_file_is_refused_before_read() {
        // Metadata only: never read the system file's contents or require chown.
        let path = Path::new("/etc/passwd");
        let meta = std::fs::metadata(path).unwrap();
        if meta.uid() != unsafe { libc::geteuid() } {
            assert_eq!(read(path).err(), Some(Refusal::WrongOwner));
        } else {
            let own = tempfile::tempdir().unwrap();
            let path = file(own.path(), b"unused");
            let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
            assert_eq!(unsafe { libc::chown(name.as_ptr(), 1, 1) }, 0);
            assert_eq!(read(&path).err(), Some(Refusal::WrongOwner));
        }
    }
}
