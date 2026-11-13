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
    /// An ACP agent profile may omit model and endpoint: the agent brings its own.
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub retry_deadline: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
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
    /// `provider: acp-agent`: the ACP agent to spawn. Keys never live here; the
    /// agent authenticates itself (API key, console login, Bedrock or Vertex).
    #[serde(default)]
    pub agent: Option<AgentSpec>,
}
/// The command line of an ACP agent, run without a shell in the session cwd.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AgentSpec {
    pub command: Vec<String>,
    /// Environment for the agent process. A value starting with `!` is a command whose
    /// output is the value (like `api_key`), resolved at spawn and registered for
    /// redaction; the profile never holds the secret itself.
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}
impl Profile {
    /// True when this profile drives an external ACP agent instead of a model.
    pub fn is_acp_agent(&self) -> bool {
        self.provider.as_deref() == Some("acp-agent")
    }
    /// The profile named `remote`, when the host put an agent command in
    /// `BBOXY450_HARNESS_AGENT` (a JSON `AgentSpec`): a seat on a machine runs
    /// this GUI here and its agent through that command. No file is written.
    pub fn from_agent_env(name: &str) -> Option<Self> {
        if name != REMOTE_PROFILE {
            return None;
        }
        let agent: AgentSpec =
            serde_json::from_str(&std::env::var("BBOXY450_HARNESS_AGENT").ok()?).ok()?;
        if agent.command.is_empty() {
            return None;
        }
        Some(Self {
            name: REMOTE_PROFILE.into(),
            model: String::new(),
            endpoint: String::new(),
            provider: Some("acp-agent".into()),
            api_key: None,
            api_key_env: None,
            retry_deadline: None,
            max_tokens: None,
            on_unattended: Unattended::Allow,
            policy: Some("yolo".into()),
            auto_compact: true,
            context_tokens: None,
            mcp_servers: Default::default(),
            agent: Some(agent),
        })
    }
}
/// The profile a machine seat runs on; see [`Profile::from_agent_env`].
pub const REMOTE_PROFILE: &str = "remote";
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
        let mut p: Self = serde_yaml::from_str(yaml)
            .map_err(|_| ProviderError::new(ErrorClass::Config, "invalid profile frontmatter"))?;
        if p.is_acp_agent() {
            // Existing discovery profiles predate the explicit yolo field. ACP
            // seats use full access by default, including those saved profiles.
            if p.policy.is_none() {
                p.policy = Some("yolo".into());
            }
            // the agent brings its own model and transport; it needs a command instead
            if p.agent.as_ref().is_none_or(|a| a.command.is_empty()) {
                return Err(ProviderError::new(
                    ErrorClass::Config,
                    "acp-agent profile needs agent.command",
                ));
            }
        } else {
            if p.model.is_empty() {
                return Err(ProviderError::new(
                    ErrorClass::Config,
                    "profile needs model",
                ));
            }
            if p.provider.as_deref() == Some("anthropic") && p.endpoint.is_empty() {
                p.endpoint = super::anthropic::DEFAULT_ENDPOINT.into();
            }
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
    /// Every face uses this dispatch, including idle profile changes and ACP factories.
    pub fn build_provider(
        &self,
        key: Secret,
        redactor: Set,
        options: Options,
    ) -> Result<Arc<dyn Provider>, ProviderError> {
        if self.provider.as_deref() == Some("anthropic") {
            let mut client = Anthropic::new(&self.endpoint, key, redactor, options)?;
            client.max_tokens = self.max_tokens;
            Ok(Arc::new(client))
        } else {
            let mut client = OpenAiCompat::new(&self.endpoint, key, redactor, options)?;
            if let Some(provider) = &self.provider {
                client.provider_id = provider.clone();
            }
            Ok(Arc::new(client))
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

#[cfg(test)]
mod remote_profile_tests {
    use super::*;
    // State: the host put an agent command in the environment. Safety: only
    // the `remote` name yields the synthesized acp-agent profile, with the
    // command verbatim and a yolo policy; anything else keeps loading files.
    #[test]
    fn remote_profile_comes_from_the_environment_only_by_name() {
        // Environment is process-wide; this test owns the variable.
        unsafe {
            std::env::set_var(
                "BBOXY450_HARNESS_AGENT",
                r#"{"command":["/app/bboxy450-term-remote","attach","--via","ssh box"]}"#,
            );
        }
        let profile = Profile::from_agent_env(REMOTE_PROFILE).expect("remote profile");
        assert!(profile.is_acp_agent());
        assert_eq!(profile.policy.as_deref(), Some("yolo"));
        assert_eq!(
            profile.agent.as_ref().unwrap().command,
            ["/app/bboxy450-term-remote", "attach", "--via", "ssh box"]
        );
        assert!(Profile::from_agent_env("codex").is_none());
        unsafe {
            std::env::set_var("BBOXY450_HARNESS_AGENT", "{\"command\":[]}");
        }
        assert!(Profile::from_agent_env(REMOTE_PROFILE).is_none());
        unsafe {
            std::env::remove_var("BBOXY450_HARNESS_AGENT");
        }
        assert!(Profile::from_agent_env(REMOTE_PROFILE).is_none());
    }
}
