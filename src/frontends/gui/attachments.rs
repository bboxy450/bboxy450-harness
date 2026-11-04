// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Workspace-only file selection and worker-owned immutable image snapshots.
use crate::{protocol::*, store::intake::ImageCustody};
use bbt_render::chrome::icons::Image;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
pub const MAX_FILE_BYTES: usize = 64 * 1024;
#[derive(Debug)]
pub enum Intake {
    Png(Vec<u8>),
    Files(Vec<PathBuf>),
    /// A fixed native converter supplied by the host, executed on the intake worker.
    EncodedImage {
        bytes: Vec<u8>,
        png: fn(&[u8]) -> Result<Vec<u8>, String>,
    },
}
#[derive(Debug)]
pub struct Attachment {
    pub id: String,
    pub label: String,
    pub block: InputBlock,
    pub preview: Option<Arc<Image>>,
    pub custody: Option<Arc<ImageCustody>>,
}
fn admitted(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|_| "Workspace unavailable")?;
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let path = path
        .canonicalize()
        .map_err(|_| "Selected file is unavailable")?;
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| "File is outside the admitted workspace")?;
    if relative
        .components()
        .any(|c| c.as_os_str().to_str().is_none_or(|s| s.starts_with('.')))
    {
        return Err("Hidden files and directories are excluded from attachment intake".into());
    }
    Ok(path)
}
pub fn candidates(root: &Path, query: &str) -> Result<Vec<String>, String> {
    if query.starts_with('/') || query.contains('\0') {
        return Err("Use a path inside the admitted workspace".into());
    }
    let (dir, prefix) = query.rsplit_once('/').map_or(("", query), |(d, p)| (d, p));
    let path = if dir.is_empty() {
        root.canonicalize().map_err(|_| "Workspace unavailable")?
    } else {
        admitted(root, Path::new(dir))?
    };
    let mut found = vec![];
    // One directory, no recursive walk, no home/config scan. Bound enumeration too.
    for entry in std::fs::read_dir(path)
        .map_err(|_| "Directory unavailable")?
        .take(4096)
    {
        let entry = entry.map_err(|_| "Directory entry unavailable")?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with('.') || !name.starts_with(prefix) {
            continue;
        }
        let Ok(path) = admitted(root, &entry.path()) else {
            continue;
        };
        let meta = path.metadata().map_err(|_| "File unavailable")?;
        if !meta.is_dir() && !meta.is_file() {
            continue;
        }
        let relative = if dir.is_empty() {
            name
        } else {
            format!("{dir}/{name}")
        };
        found.push(format!(
            "@{relative}{}",
            if meta.is_dir() { "/" } else { "" }
        ));
    }
    found.sort();
    found.truncate(64);
    Ok(found)
}
pub fn read_file(root: &Path, path: &Path) -> Result<Attachment, String> {
    let path = admitted(root, path)?;
    // O_NOFOLLOW closes the final-component replacement race. Canonical ancestors
    // must remain stable under this local, same-user workspace contract.
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&path)
        .map_err(|_| "Selected file unavailable")?;
    if !file.metadata().map_err(|_| "File unavailable")?.is_file() {
        return Err("Select a regular file".into());
    }
    let png = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));
    let limit = if png {
        crate::store::intake::MAX_IMAGE_BYTES
    } else {
        MAX_FILE_BYTES
    };
    let mut bytes = vec![];
    file.by_ref()
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read selected file")?;
    if bytes.len() > limit {
        return Err(format!("Attachment exceeds the {} KiB limit", limit / 1024));
    }
    let label = path
        .strip_prefix(root.canonicalize().map_err(|_| "Workspace unavailable")?)
        .unwrap_or(&path)
        .display()
        .to_string();
    if png {
        return image(bytes, label);
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| "Only UTF-8 text files and PNG images are supported")?;
    if text.contains('\0') {
        return Err("Binary files are unsupported; select UTF-8 text or PNG".into());
    }
    Ok(Attachment {
        id: OpId::new().to_string(),
        label: label.clone(),
        block: InputBlock::Text {
            text: format!("Attached workspace file: {label}\n{text}"),
        },
        preview: None,
        custody: None,
    })
}
pub fn image(bytes: Vec<u8>, label: String) -> Result<Attachment, String> {
    if bytes.len() > crate::store::intake::MAX_IMAGE_BYTES {
        return Err("PNG image exceeds 16 MiB".into());
    }
    let mut decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder.set_limits(png::Limits {
        bytes: 128 * 1024 * 1024,
    });
    let mut reader = decoder.read_info().map_err(|_| "Invalid PNG image")?;
    let info = reader.info();
    if u64::from(info.width) * u64::from(info.height) > 32 * 1024 * 1024 {
        return Err("PNG exceeds 32 megapixels".into());
    }
    let mut pixels = vec![0; reader.output_buffer_size().ok_or("PNG image too large")?];
    let info = reader
        .next_frame(&mut pixels)
        .map_err(|_| "Invalid PNG image data")?;
    let (w, h) = (info.width, info.height);
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return Err("Unsupported PNG color type".into()),
    };
    let width = w.min(96);
    let height = (u64::from(h) * u64::from(width) / u64::from(w)).clamp(1, 96) as u32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let at = ((u64::from(y) * u64::from(h) / u64::from(height) * u64::from(w)
                + u64::from(x) * u64::from(w) / u64::from(width)) as usize)
                * channels;
            let p = &pixels[at..at + channels];
            rgba.extend_from_slice(&match channels {
                4 => [p[0], p[1], p[2], p[3]],
                3 => [p[0], p[1], p[2], 255],
                2 => [p[0], p[0], p[0], p[1]],
                _ => [p[0], p[0], p[0], 255],
            });
        }
    }
    let custody = Arc::new(
        ImageCustody::create(&bytes, "image/png")
            .map_err(|_| "Private image custody unavailable")?,
    );
    let id = OpId::new().to_string();
    use sha2::Digest as _;
    let image_key = format!("harness-attachment/{:x}", sha2::Sha256::digest(&bytes));
    Ok(Attachment {
        id: id.clone(),
        label,
        block: custody.block.clone(),
        preview: Some(Arc::new(Image {
            name: image_key,
            width,
            height,
            rgba,
        })),
        custody: Some(custody),
    })
}
