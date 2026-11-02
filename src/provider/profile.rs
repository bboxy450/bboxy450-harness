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
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub permission_mode: Option<String>,
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
    #[serde(default = "enabled")]
    pub auto_compact: bool,
    #[serde(default)]
    pub context_tokens: Option<u64>,
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: crate::mcp::Servers,
}
fn enabled() -> bool {
    true
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
        if p.is_claude_code() {
            if p.model.starts_with('-') || p.model.len() > 256 {
                return Err(ProviderError::new(
                    ErrorClass::Config,
                    "invalid Claude Code model",
                ));
            }
            if p.api_key.is_some() || p.api_key_env.is_some() {
                return Err(ProviderError::new(
                    ErrorClass::Config,
                    "claude-code uses its own login; remove api_key and api_key_env",
                ));
            }
            if let Some(mode) = &p.permission_mode
                && ![
                    "default",
                    "acceptEdits",
                    "plan",
                    "dontAsk",
                    "bypassPermissions",
                    "auto",
                ]
                .contains(&mode.as_str())
            {
                return Err(ProviderError::new(
                    ErrorClass::Config,
                    "invalid Claude Code permission mode",
                ));
            }
        } else {
            validate_endpoint(&p.endpoint)?;
        }
        Ok(p)
    }
    pub async fn load(path: &Path) -> Result<Self, ProviderError> {
        let text = tokio::fs::read_to_string(path)
            .await
            .map_err(|_| ProviderError::new(ErrorClass::Config, "profile unavailable"))?;
        Self::parse(&text)
    }
    pub fn file_in(root: &Path, name: &str) -> Result<PathBuf, ProviderError> {
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
        Ok(root.join(format!("{name}.md")))
    }
    pub fn path(name: &str) -> Result<PathBuf, ProviderError> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| ProviderError::new(ErrorClass::Config, "HOME unavailable"))?;
        Self::file_in(
            &PathBuf::from(home).join(".bboxy450-harness/profiles"),
            name,
        )
    }
    pub fn is_claude_code(&self) -> bool {
        self.provider.as_deref() == Some("claude-code")
    }
    /// Select the agent before touching any credential source or HTTP client.
    pub async fn build_provider(
        &self,
        set: &Set,
        cancel: CancellationToken,
    ) -> Result<Arc<dyn Provider>, ProviderError> {
        if self.is_claude_code() {
            return Ok(Arc::new(super::claude_code::AgentProvider(
                super::claude_code::Options {
                    model: self.model.clone(),
                    permission_mode: self.permission_mode.clone(),
                },
            )));
        }
        let key = self.key(set, cancel).await?;
        Ok(Arc::new(OpenAiCompat::new(
            &self.endpoint,
            key,
            set.clone(),
            super::Options {
                retry_deadline: self.retry_deadline.map(Duration::from_secs),
                ..Default::default()
            },
        )?))
    }
    pub async fn key(&self, set: &Set, cancel: CancellationToken) -> Result<Secret, ProviderError> {
        if self.is_claude_code() {
            return Err(ProviderError::new(
                ErrorClass::Auth,
                "Claude Code owns authentication; use claude /login or claude setup-token",
            ));
        }
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
pub(crate) async fn resolve_command(
    cmd: &str,
    cancel: CancellationToken,
) -> Result<String, ProviderError> {
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
    let output = crate::process::capture_stdout(
        crate::process::Request {
            program: "sh".into(),
            args: vec!["-c".into(), cmd.into()],
            stdin: None,
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            timeout: Duration::from_secs(10),
            cap: 8192,
            spill_dir: PathBuf::new(),
            redactor: Set::default(),
        },
        cancel,
    )
    .await
    .map_err(|_| ProviderError::new(ErrorClass::Auth, "key command failed (exit -1)"))?;
    let bytes = zeroize::Zeroizing::new(output.bytes);
    if output.cancelled || output.code != Some(0) {
        return Err(ProviderError::new(
            ErrorClass::Auth,
            "key command failed (exit -1)",
        ));
    }
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProviderError::new(ErrorClass::Auth, "key command failed (exit -1)"))
}
