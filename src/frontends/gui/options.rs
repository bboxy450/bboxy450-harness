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
}
impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut out = Self {
            replay: None,
            profile: "nemotron".into(),
            cwd: std::env::current_dir().map_err(|e| e.to_string())?,
            headless: false,
            screenshot: ".bboxy/runs/gui-smoke.png".into(),
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
                _ => return Err(format!("unknown gui option: {arg}")),
            }
        }
        if out.headless && out.replay.is_none() {
            return Err("--headless requires --replay".into());
        }
        Ok(out)
    }
}
