// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub model: String,
    pub endpoint: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub retry_deadline: Option<u64>,
    #[serde(default)]
    pub on_unattended: Unattended,
    #[serde(default)]
    pub policy: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unattended {
    #[default]
    Deny,
    Allow,
}
impl fmt::Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Profile")
            .field("name", &self.name)
            .field("model", &self.model)
            .field("endpoint", &"<redacted>")
            .field("api_key", &"<redacted>")
            .finish()
    }
}
pub fn validate_endpoint(endpoint: &str) -> Result<(), ProviderError> {
    let u = reqwest::Url::parse(endpoint)
        .map_err(|_| ProviderError::new(ErrorClass::Config, "invalid endpoint"))?;
    let local = matches!(
        u.host_str(),
        Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
    );
    if !(u.scheme() == "https" || (u.scheme() == "http" && local))
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err(ProviderError::new(
            ErrorClass::Config,
            "endpoint must be HTTPS, without credentials or query",
        ));
    }
    Ok(())
}
impl Profile {
    pub fn parse(text: &str) -> Result<Self, ProviderError> {
        let yaml = text
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---"))
            .map(|(s, _)| s)
            .ok_or_else(|| ProviderError::new(ErrorClass::Config, "missing profile frontmatter"))?;
        let p: Self = serde_yaml::from_str(yaml)
            .map_err(|_| ProviderError::new(ErrorClass::Config, "invalid profile frontmatter"))?;
        validate_endpoint(&p.endpoint)?;
        Ok(p)
    }
    pub async fn load(path: &Path) -> Result<Self, ProviderError> {
        let text = tokio::fs::read_to_string(path)
            .await
            .map_err(|_| ProviderError::new(ErrorClass::Config, "profile unavailable"))?;
        Self::parse(&text)
    }
    pub fn path(name: &str) -> Result<PathBuf, ProviderError> {
        if name.is_empty()
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
        {
            return Err(ProviderError::new(
                ErrorClass::Config,
                "invalid profile name",
            ));
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| ProviderError::new(ErrorClass::Config, "HOME unavailable"))?;
        Ok(PathBuf::from(home)
            .join(".bboxy450-harness/profiles")
            .join(format!("{name}.md")))
    }
    pub async fn key(&self, set: &Set, cancel: CancellationToken) -> Result<Secret, ProviderError> {
        let value = if let Some(source) = &self.api_key {
            if let Some(cmd) = source.strip_prefix('!') {
                resolve_command(cmd, cancel).await?
            } else {
                source.clone()
            }
        } else if let Some(name) = &self.api_key_env {
            std::env::var(name).map_err(|_| {
                ProviderError::new(ErrorClass::Auth, "key environment is unavailable")
            })?
        } else {
            return Err(ProviderError::new(ErrorClass::Auth, "no key configured"));
        };
        if value.trim().is_empty() {
            return Err(ProviderError::new(ErrorClass::Auth, "empty key"));
        }
        Ok(Secret::new(value.trim().into(), set))
    }
    pub fn policy(&self) -> crate::policy::Policy {
        if self.policy.as_deref() == Some("yolo") {
            crate::policy::Policy::allow_all(self.on_unattended)
        } else {
            crate::policy::Policy::new(self.on_unattended)
        }
    }
}
struct Wake(tokio::sync::Notify);
impl bbt_rt::Waker for Wake {
    fn wake(&self) {
        self.0.notify_one();
    }
}
async fn resolve_command(cmd: &str, cancel: CancellationToken) -> Result<String, ProviderError> {
    // Keychain lookup can show a dialog before a timeout. Never launch it in an
    // unattended harness; the same command source is retained in the profile.
    if cmd.contains("security ")
        || cmd.contains("osascript")
        || cmd.to_ascii_lowercase().contains("keychain")
    {
        return Err(ProviderError::new(
            ErrorClass::Auth,
            "key command requires an attended key source",
        ));
    }
    let wake = Arc::new(Wake(tokio::sync::Notify::new()));
    let dirty = bbt_rt::DirtySet::new(1, wake.clone());
    let (ui, rx) = bbt_rt::ui_channel(32, dirty.source(0));
    let output = bbt_rt::ViewRing::new(32, dirty.source(0));
    let runner = bbt_rt::CommandRunner::new(ui, output.clone());
    let task = runner.spawn_command(
        vec!["sh".into(), "-c".into(), cmd.into()],
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        vec![],
    );
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    let mut overflow = false;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if output.dropped() > 0 {
            overflow = true;
            bbt_rt::cancel(&task);
        }
        while let Some(chunk) = output.pop() {
            if matches!(chunk.stream, bbt_rt::Stream::Stdout) {
                if bytes.len() + chunk.bytes.len() > 8192 {
                    overflow = true;
                    bbt_rt::cancel(&task);
                } else {
                    bytes.extend_from_slice(&chunk.bytes);
                }
            }
        }
        while rx.try_recv().is_some() {}
        rx.complete_drain();
        output.complete_drain();
        if let Some(status) = task.status() {
            if overflow || status.code != Some(0) || status.cancelled {
                return Err(ProviderError::new(
                    ErrorClass::Auth,
                    format!("key command failed (exit {})", status.code.unwrap_or(-1)),
                ));
            }
            return String::from_utf8(bytes.to_vec())
                .map_err(|_| ProviderError::new(ErrorClass::Auth, "key command failed (exit -1)"));
        }
        if tokio::time::Instant::now() >= deadline || cancel.is_cancelled() {
            bbt_rt::cancel(&task);
            while task.status().is_none() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
            return Err(ProviderError::new(
                ErrorClass::Auth,
                "key command failed (exit -1)",
            ));
        }
        tokio::select! {_=wake.0.notified()=>{},_=tokio::time::sleep(Duration::from_millis(2))=>{}}
    }
}
