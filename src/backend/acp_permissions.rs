// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Provider-specific full-access launch settings; no unsupported vendor flags.
use crate::provider::profile::AgentSpec;
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn prepare(spec: &mut AgentSpec) {
    let program = spec
        .command
        .first()
        .and_then(|p| Path::new(p).file_name())
        .and_then(|p| p.to_str())
        .map(str::to_owned);
    match program.as_deref() {
        // codex-acp 2.x selects its own policies for each turn. A codex CLI
        // --yolo flag is not an ACP flag: INITIAL_AGENT_MODE is its equivalent.
        Some("codex-acp") => {
            spec.env
                .insert("INITIAL_AGENT_MODE".into(), "agent-full-access".into());
        }
        // Claude's SDK permission mode is negotiated over ACP, below.
        Some("gemini" | "omp") => {
            if !spec.command.iter().any(|arg| arg == "--yolo") {
                spec.command.push("--yolo".into());
            }
            if program.as_deref() == Some("gemini") {
                spec.env.insert("GEMINI_SANDBOX".into(), "false".into());
            }
        }
        Some("grok") => {
            if !spec.command.iter().any(|arg| arg == "--always-approve") {
                spec.command.insert(1, "--always-approve".into());
            }
            spec.env.insert("GROK_SANDBOX".into(), "off".into());
        }
        Some("opencode") => {
            spec.env.insert(
                "OPENCODE_PERMISSION".into(),
                json!({"*":"allow"}).to_string(),
            );
        }
        Some("goose") => {
            spec.env.insert("GOOSE_MODE".into(), "auto".into());
        }
        // Pi's RPC tools already run without a native approval/sandbox gate.
        // Its extension permission requests still pass through our yolo rule.
        _ => {}
    }
}

pub(super) fn session_options(params: &mut Value, name: &str) {
    if name.to_ascii_lowercase().contains("claude") {
        params["_meta"] =
            json!({"claudeCode":{"options":{"allowDangerouslySkipPermissions":true}}});
    }
}

/// Select only advertised full-access identifiers. Pi's thinking-level modes,
/// for example, must never be mistaken for permission controls.
pub(super) fn selection(result: &Value) -> Option<(Option<String>, String)> {
    const MODES: &[&str] = &[
        "agent-full-access",
        "bypassPermissions",
        "yolo",
        "full-access",
        "fullAccess",
        "danger-full-access",
        "auto-approve",
        "always-approve",
    ];
    for wanted in MODES {
        if result
            .pointer("/modes/availableModes")
            .and_then(Value::as_array)
            .is_some_and(|modes| {
                modes
                    .iter()
                    .any(|m| m.get("id").and_then(Value::as_str) == Some(wanted))
            })
        {
            return Some((None, (*wanted).into()));
        }
        for option in result
            .get("configOptions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(id) = option.get("id").and_then(Value::as_str) else {
                continue;
            };
            let category = option.get("category").and_then(Value::as_str).unwrap_or(id);
            if !matches!(
                category,
                "mode" | "permission_mode" | "permissions" | "approval_mode"
            ) {
                continue;
            }
            if option
                .get("options")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .flat_map(|entry| {
                    entry
                        .get("options")
                        .and_then(Value::as_array)
                        .map(Vec::as_slice)
                        .unwrap_or(std::slice::from_ref(entry))
                })
                .any(|entry| entry.get("value").and_then(Value::as_str) == Some(wanted))
            {
                return Some((Some(id.into()), (*wanted).into()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acp_full_access_is_the_default_for_saved_profiles() {
        use crate::provider::profile::Profile;
        for program in [
            "claude-agent-acp",
            "codex-acp",
            "pi-acp",
            "omp",
            "custom-agent",
        ] {
            let text = format!(
                "---\nname: legacy\nprovider: acp-agent\nagent:\n  command: [{program}]\n---\n"
            );
            assert_eq!(
                Profile::parse(&text).unwrap().policy.as_deref(),
                Some("yolo")
            );
            let ask = text.replace("provider: acp-agent", "provider: acp-agent\npolicy: ask");
            assert_eq!(Profile::parse(&ask).unwrap().policy.as_deref(), Some("ask"));
        }
    }

    #[test]
    fn acp_full_access_launches_are_vendor_specific_and_idempotent() {
        for (program, flag, env) in [
            (
                "codex-acp",
                None,
                Some(("INITIAL_AGENT_MODE", "agent-full-access")),
            ),
            ("claude-agent-acp", None, None),
            ("pi-acp", None, None),
            ("omp", Some("--yolo"), None),
            ("gemini", Some("--yolo"), Some(("GEMINI_SANDBOX", "false"))),
            (
                "grok",
                Some("--always-approve"),
                Some(("GROK_SANDBOX", "off")),
            ),
            (
                "opencode",
                None,
                Some(("OPENCODE_PERMISSION", "{\"*\":\"allow\"}")),
            ),
            ("goose", None, Some(("GOOSE_MODE", "auto"))),
        ] {
            let mut spec = AgentSpec {
                command: vec![format!("/private/bin/{program}")],
                ..Default::default()
            };
            prepare(&mut spec);
            if let Some(flag) = flag {
                assert!(spec.command.iter().any(|arg| arg == flag), "{program}");
            }
            if let Some((name, value)) = env {
                assert_eq!(spec.env.get(name).map(String::as_str), Some(value));
            }
            let once = spec.clone();
            prepare(&mut spec);
            assert_eq!(spec, once);
        }
        let mut unknown = AgentSpec {
            command: vec!["private-wrapper".into(), "literal --yolo".into()],
            ..Default::default()
        };
        let before = unknown.clone();
        prepare(&mut unknown);
        assert_eq!(unknown, before);
        let mut params = json!({"cwd":"/private"});
        session_options(&mut params, "claude-agent-acp");
        assert_eq!(
            params.pointer("/_meta/claudeCode/options/allowDangerouslySkipPermissions"),
            Some(&json!(true))
        );
    }

    #[test]
    fn acp_full_access_uses_only_permission_modes_and_config_options() {
        assert_eq!(
            selection(&json!({"modes":{"availableModes":[{"id":"high"}]}})),
            None
        );
        assert_eq!(
            selection(&json!({"configOptions":[{"id":"model","options":[{"value":"yolo"}]}]})),
            None
        );
        assert_eq!(
            selection(
                &json!({"configOptions":[{"id":"permission","category":"mode","options":[{"name":"Modes","options":[{"value":"bypassPermissions"}]}]}]})
            ),
            Some((Some("permission".into()), "bypassPermissions".into()))
        );
    }
}
