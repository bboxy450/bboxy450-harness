// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Local, bounded image custody. Only unguessable files in our private intake
//! directory are admitted; resource payloads never authorize arbitrary file reads.
use super::*;
use std::os::unix::fs::MetadataExt;
pub const MAX_IMAGE_BYTES: usize = 16 * 1024 * 1024;
const PREFIX: &str = "bboxy-intake:";
fn root() -> PathBuf {
    // GUI and supervised seat may have different TMPDIR environments.
    PathBuf::from("/tmp").join(format!("bboxy450-harness-intake-{}", unsafe {
        libc::geteuid()
    }))
}
#[derive(Debug, PartialEq)]
pub struct ImageCustody {
    path: PathBuf,
    pub block: InputBlock,
}
impl Drop for ImageCustody {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
impl ImageCustody {
    /// Caller runs on a blocking worker. Nothing from the general clipboard is read here.
    pub fn create(bytes: &[u8], mime: &str) -> io::Result<Self> {
        if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES || mime != "image/png" {
            return Err(io::Error::other(
                "Only PNG images up to 16 MiB are supported",
            ));
        }
        let dir = root();
        match std::fs::DirBuilder::new().mode(0o700).create(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        let dir = checked_root()?;
        let path = dir.join(OpId::new().to_string());
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(bytes)?;
        let block = InputBlock::Resource {
            uri: format!("{PREFIX}{}", path.display()),
            mime: mime.into(),
            blob: Some(BlobRef {
                sha256: format!("{:x}", Sha256::digest(bytes)),
                len: bytes.len() as u64,
                mime: mime.into(),
            }),
        };
        Ok(Self { path, block })
    }
}
use std::os::unix::fs::DirBuilderExt;
fn checked_root() -> io::Result<PathBuf> {
    let path = root();
    let meta = std::fs::symlink_metadata(&path)?;
    if !meta.is_dir() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
        return Err(io::Error::other(
            "Private image intake directory is unavailable",
        ));
    }
    path.canonicalize()
}
impl Blobs {
    /// Snapshot before queueing. A queued/durable image refers to the session blob,
    /// never to the temporary file. Rejected input leaves the sender's custody intact.
    pub async fn admit_images(&self, blocks: Vec<InputBlock>) -> io::Result<Vec<InputBlock>> {
        let mut out = Vec::with_capacity(blocks.len());
        for block in blocks {
            if let InputBlock::Resource {
                uri,
                mime,
                blob: Some(expected),
            } = &block
                && let Some(raw) = uri.strip_prefix(PREFIX)
            {
                let path = PathBuf::from(raw);
                let expected = expected.clone();
                let mime = mime.clone();
                let bytes = blocking(move || {
                    use std::io::Read;
                    let dir = checked_root()?;
                    if path.parent() != Some(dir.as_path())
                        || path
                            .file_name()
                            .and_then(|p| p.to_str())
                            .and_then(|n| n.parse::<ulid::Ulid>().ok())
                            .is_none()
                        || mime != "image/png"
                        || expected.mime != mime
                        || expected.len > MAX_IMAGE_BYTES as u64
                    {
                        return Err(io::Error::other("Invalid image custody reference"));
                    }
                    let mut file = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                        .open(path)?;
                    let meta = file.metadata()?;
                    if !meta.is_file()
                        || meta.uid() != unsafe { libc::geteuid() }
                        || meta.mode() & 0o077 != 0
                        || meta.len() != expected.len
                    {
                        return Err(io::Error::other("Invalid image custody file"));
                    }
                    let mut bytes = Vec::new();
                    std::io::Read::by_ref(&mut file)
                        .take(MAX_IMAGE_BYTES as u64 + 1)
                        .read_to_end(&mut bytes)?;
                    if bytes.len() as u64 != expected.len
                        || format!("{:x}", Sha256::digest(&bytes)) != expected.sha256
                        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
                    {
                        return Err(io::Error::other("Image custody content changed"));
                    }
                    Ok(bytes)
                })
                .await?;
                // Binary payloads must remain byte-exact: secret redaction is for text.
                let blob = self.put_binary(bytes, "image/png").await?;
                out.push(InputBlock::Image { blob });
            } else {
                out.push(block);
            }
        }
        Ok(out)
    }
    pub async fn put_binary(&self, bytes: Vec<u8>, mime: &str) -> io::Result<BlobRef> {
        let unredacted = Self {
            dir: self.dir.clone(),
            redactor: Set::default(),
        };
        unredacted.put(bytes, mime).await
    }
}
