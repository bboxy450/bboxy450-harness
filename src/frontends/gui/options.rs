// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

use std::path::PathBuf;
#[derive(Debug)]
pub struct Options {
    pub replay: Option<PathBuf>,
    pub profile: String,
    pub cwd: PathBuf,
    pub headless: bool,
    pub screenshot: PathBuf,
    /// Render-only capture size (parity): no fixture assertions, transcript top.
    pub capture: Option<[u32; 2]>,
    /// `--watch [claude:|codex:]PATH`: render another harness's native session live.
    pub watch: Option<(Option<crate::import::Native>, PathBuf)>,
}
impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut out = Self {
            replay: None,
            profile: "nemotron".into(),
            cwd: std::env::current_dir().map_err(|e| e.to_string())?,
            headless: false,
            screenshot: ".bboxy/runs/gui-smoke.png".into(),
            capture: None,
            watch: None,
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--replay" => out.replay = Some(args.next().ok_or("missing replay path")?.into()),
                "--profile" => out.profile = args.next().ok_or("missing profile")?,
                "--cwd" => out.cwd = args.next().ok_or("missing cwd")?.into(),
                "--headless" => out.headless = true,
                "--screenshot" => {
                    out.screenshot = args.next().ok_or("missing screenshot path")?.into()
                }
                "--watch" => {
                    let spec = args.next().ok_or("missing watch path")?;
                    out.watch = Some(match spec.split_once(':') {
                        Some((kind, path)) if crate::import::Native::parse(kind).is_some() => {
                            (crate::import::Native::parse(kind), path.into())
                        }
                        _ => (None, spec.into()),
                    });
                }
                "--capture" => {
                    let size = args.next().ok_or("missing capture size")?;
                    let (w, h) = size.split_once('x').ok_or("capture size is WxH")?;
                    out.capture = Some([
                        w.parse().map_err(|_| "invalid capture width")?,
                        h.parse().map_err(|_| "invalid capture height")?,
                    ]);
                    out.headless = true;
                }
                _ => return Err(format!("unknown gui option: {arg}")),
            }
        }
        if out.headless && out.replay.is_none() {
            return Err("--headless requires --replay".into());
        }
        Ok(out)
    }
}
