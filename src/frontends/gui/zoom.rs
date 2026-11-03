// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Global text zoom (harness-spec.md "Standalone window"): Cmd+= / Cmd++ zoom in, Cmd+-
//! zooms out, Cmd+0 resets. The scale runs 90 % to 160 % in 10 % steps, applies to every
//! text size and line height in the face, and persists in the harness settings
//! (`$HOME/.bboxy450-harness/settings.json`, key `zoom`). The formal contract is
//! HarnessTextZoom (tests/formal/models.rs); scripts/test-chrome-control.py breaks the clamp.
use std::path::{Path, PathBuf};

pub const MIN: u8 = 90;
pub const MAX: u8 = 160;
pub const STEP: u8 = 10;
pub const DEFAULT: u8 = 100;

/// A zoom percentage on the 10 % grid between 90 and 160.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Zoom(u8);
impl Default for Zoom {
    fn default() -> Self {
        Self(DEFAULT)
    }
}
impl Zoom {
    /// Any percentage, clamped to the range and snapped to the nearest step.
    pub fn from_percent(percent: i32) -> Self {
        let clamped = percent.clamp(i32::from(MIN), i32::from(MAX));
        let steps = (clamped - i32::from(MIN) + i32::from(STEP) / 2) / i32::from(STEP);
        Self((i32::from(MIN) + steps * i32::from(STEP)) as u8)
    }
    pub fn percent(self) -> u8 {
        self.0
    }
    /// The multiplier for every text size and line height in the face.
    pub fn scale(self) -> f32 {
        f32::from(self.0) / 100.
    }
    /// One step in (`+1`) or out (`-1`), held at the range's ends.
    pub fn step(self, delta: i8) -> Self {
        Self::from_percent(i32::from(self.0) + i32::from(delta) * i32::from(STEP))
    }
    /// The pill's text: `110%`.
    pub fn label(self) -> String {
        format!("{}%", self.0)
    }
}

const FILE: &str = "settings.json";
const KEY: &str = "zoom";

fn settings_path(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// The saved zoom, or the default when the settings file is missing or unreadable. Other
/// keys in the file are left alone by `save`.
pub async fn load(root: &Path) -> Zoom {
    let Ok(text) = tokio::fs::read_to_string(settings_path(root)).await else {
        return Zoom::default();
    };
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| v.get(KEY)?.as_i64())
        .and_then(|p| i32::try_from(p).ok())
        .map_or_else(Zoom::default, Zoom::from_percent)
}

/// Persist `zoom`, keeping every other setting: the file is rewritten whole through a
/// sibling temporary and a rename, so a crash leaves either version intact.
pub async fn save(root: &Path, zoom: Zoom) -> Result<(), String> {
    let path = settings_path(root);
    let mut settings = match tokio::fs::read_to_string(&path).await {
        Ok(text) => serde_json::from_str::<serde_json::Value>(&text).unwrap_or_default(),
        Err(_) => serde_json::Value::default(),
    };
    if !settings.is_object() {
        settings = serde_json::json!({});
    }
    settings[KEY] = serde_json::json!(zoom.percent());
    let text = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())? + "\n";
    tokio::fs::create_dir_all(root)
        .await
        .map_err(|e| e.to_string())?;
    let tmp = root.join(format!("{FILE}.{}.tmp", std::process::id()));
    tokio::fs::write(&tmp, text)
        .await
        .map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // keeps: harness-spec.md Standalone window — 90..160 in 10 % steps
    #[test]
    fn steps_clamp_and_snap() {
        let z = Zoom::default();
        assert_eq!(z.percent(), 100);
        assert_eq!(z.step(1).percent(), 110);
        assert_eq!(z.step(-1).percent(), 90);
        assert_eq!(z.step(-1).step(-1).percent(), 90, "held at the floor");
        let mut up = z;
        for expected in [110, 120, 130, 140, 150, 160, 160, 160] {
            up = up.step(1);
            assert_eq!(up.percent(), expected);
        }
        assert_eq!(Zoom::from_percent(0).percent(), 90);
        assert_eq!(Zoom::from_percent(1000).percent(), 160);
        assert_eq!(Zoom::from_percent(104).percent(), 100, "snaps down");
        assert_eq!(Zoom::from_percent(105).percent(), 110, "snaps up");
        assert_eq!(Zoom::from_percent(-7).percent(), 90);
        assert_eq!(Zoom::from_percent(133).label(), "130%");
        assert!((Zoom::from_percent(150).scale() - 1.5).abs() < 1e-6);
    }

    #[tokio::test]
    async fn settings_round_trip_and_keep_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".bboxy450-harness");
        assert_eq!(load(&root).await, Zoom::default(), "no file: the default");
        tokio::fs::create_dir_all(&root).await.unwrap();
        tokio::fs::write(root.join(FILE), "{\"theme\":\"dark\",\"zoom\":120}\n")
            .await
            .unwrap();
        assert_eq!(load(&root).await.percent(), 120);
        save(&root, Zoom::from_percent(140)).await.unwrap();
        assert_eq!(load(&root).await.percent(), 140);
        let text = tokio::fs::read_to_string(root.join(FILE)).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["theme"], "dark", "other settings survive a save");
        tokio::fs::write(root.join(FILE), "not json").await.unwrap();
        assert_eq!(load(&root).await, Zoom::default(), "garbage: the default");
        tokio::fs::write(root.join(FILE), "{\"zoom\": 999}")
            .await
            .unwrap();
        assert_eq!(load(&root).await.percent(), 160, "a saved value is clamped");
    }
}
