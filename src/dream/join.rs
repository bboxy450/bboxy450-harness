// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Task joins use IR tool inputs and exact committed record bytes, never a seat map.
use super::fingerprint;
use crate::protocol::EvidenceRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accepted,
    Rejected,
    Failed,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub pin: EvidenceRef,
    pub number: Option<u64>,
    pub verdict: Verdict,
    pub verify: Option<String>,
    pub verify_exit: Option<i64>,
    pub rejection: Option<String>,
}
impl Task {
    pub fn from_committed(
        root: &str,
        path: &str,
        commit: &str,
        bytes: &[u8],
    ) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|_| "task is not UTF-8")?;
        let yaml = text
            .strip_prefix("---\n")
            .and_then(|s| s.split_once("\n---").map(|(fm, _)| fm))
            .ok_or("task has no frontmatter")?;
        let value: Value = serde_yaml::from_str(yaml).map_err(|_| "invalid task frontmatter")?;
        if value.get("type").and_then(Value::as_str) != Some("task") {
            return Err("not a task record".into());
        }
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or("task has no id")?;
        if root.is_empty()
            || !path.starts_with(".bboxy/tasks/")
            || path.split('/').any(|part| part == "..")
            || !matches!(commit.len(), 40 | 64)
            || !commit.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid committed task pin".into());
        }
        let exit = value
            .pointer("/verify_result/exit_code")
            .and_then(Value::as_i64);
        let rejection = value.get("rejection").filter(|r| !r.is_null());
        let verdict = if matches!(
            value.get("status").and_then(Value::as_str),
            Some("accepted" | "done")
        ) && exit == Some(0)
        {
            Verdict::Accepted
        } else if rejection.is_some() {
            Verdict::Rejected
        } else if exit.is_some_and(|v| v != 0) {
            Verdict::Failed
        } else {
            Verdict::None
        };
        Ok(Self {
            pin: EvidenceRef::Record {
                root: root.into(),
                id: id.into(),
                path: path.into(),
                commit: commit.into(),
                digest: format!("sha256:{}", fingerprint::digest(bytes)),
            },
            number: value
                .get("number")
                .and_then(|v| v.as_u64().or_else(|| v.as_str()?.parse().ok())),
            verdict,
            verify_exit: exit,
            verify: value
                .get("verify")
                .and_then(Value::as_str)
                .map(str::to_owned),
            rejection: rejection
                .and_then(|r| r.get("reason"))
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
    }
    /// Only these three task tools establish an association; ordinary mentions do not.
    pub fn matches(&self, name: &str, input: &Value) -> bool {
        if !["task_report", "task_accept", "task_reject"]
            .iter()
            .any(|end| name.ends_with(end))
        {
            return false;
        }
        let EvidenceRef::Record { id, path, .. } = &self.pin else {
            return false;
        };
        let number = self.number.map(|n| format!("#{n}"));
        fn contains(v: &Value, needles: &[&str]) -> bool {
            match v {
                Value::String(s) => needles.iter().any(|needle| {
                    s.match_indices(needle).any(|(at, _)| {
                        let token =
                            |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '/' | '.');
                        !s[..at].chars().next_back().is_some_and(token)
                            && !s[at + needle.len()..].chars().next().is_some_and(token)
                    })
                }),
                Value::Array(a) => a.iter().any(|v| contains(v, needles)),
                Value::Object(o) => o.values().any(|v| contains(v, needles)),
                _ => false,
            }
        }
        let mut needles = vec![id.as_str(), path.as_str()];
        if let Some(path) = path.strip_prefix(".bboxy/") {
            needles.push(path);
        }
        if let Some(number) = &number {
            needles.push(number);
        }
        contains(input, &needles)
    }
}

/// Resolve origin/main once, enumerate literal object paths, then read that pin.
/// A moving ref never changes a task halfway through this catalogue.
pub async fn committed_tasks(cwd: &std::path::Path, root: &str) -> std::io::Result<Vec<Task>> {
    use tokio::time::Instant;
    let deadline = Instant::now() + crate::vcs::CAPTURE_BUDGET;
    let toplevel = git_object(cwd, &["rev-parse", "--show-toplevel"], deadline).await?;
    let toplevel = std::path::PathBuf::from(
        std::str::from_utf8(&toplevel)
            .map_err(std::io::Error::other)?
            .trim_end_matches(['\r', '\n']),
    );
    let cwd = toplevel.as_path();
    let commit = git_object(cwd, &["rev-parse", "--verify", "origin/main"], deadline).await?;
    let commit = std::str::from_utf8(&commit)
        .map_err(std::io::Error::other)?
        .trim()
        .to_owned();
    if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(std::io::Error::other("invalid task commit"));
    }
    let paths = git_object(
        cwd,
        &[
            "ls-tree",
            "-r",
            "-z",
            "--name-only",
            &commit,
            "--",
            ".bboxy/tasks",
        ],
        deadline,
    )
    .await?;
    let mut tasks = Vec::new();
    for path in paths.split(|b| *b == 0).filter(|p| p.ends_with(b".md")) {
        let path = std::str::from_utf8(path).map_err(std::io::Error::other)?;
        let object = format!("{commit}:{path}");
        let bytes = git_object(cwd, &["show", &object], deadline).await?;
        let root = root.to_owned();
        let path = path.to_owned();
        let pin = commit.clone();
        if let Ok(task) = crate::runtime::blocking(move || {
            crate::runtime::assert_off_rt_worker();
            Task::from_committed(&root, &path, &pin, &bytes)
        })
        .await
        {
            tasks.push(task);
        }
        tokio::task::yield_now().await;
    }
    Ok(tasks)
}
async fn git_object(
    cwd: &std::path::Path,
    args: &[&str],
    deadline: tokio::time::Instant,
) -> std::io::Result<Vec<u8>> {
    use std::io;
    let timeout = deadline
        .checked_duration_since(tokio::time::Instant::now())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "task pin read deadline"))?;
    // Adapted from this project's WP2 metadata child isolation in src/vcs.rs;
    // credit/policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
    let mut command = Vec::new();
    for name in std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with("GIT_"))
    {
        command.extend(["-u".into(), name]);
    }
    command.extend(
        [
            "GIT_CONFIG_NOSYSTEM=1",
            "GIT_CONFIG_GLOBAL=/dev/null",
            "GIT_TERMINAL_PROMPT=0",
            "GIT_ASKPASS=/usr/bin/false",
            "SSH_ASKPASS=/usr/bin/false",
            "git",
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]
        .map(String::from),
    );
    command.extend(args.iter().map(|s| (*s).to_owned()));
    let cap = 1024 * 1024;
    let output = crate::process::capture_stdout(
        crate::process::Request {
            program: "/usr/bin/env".into(),
            args: command,
            stdin: None,
            cwd: cwd.into(),
            timeout,
            spill_dir: Default::default(),
            cap,
            redactor: Default::default(),
        },
        tokio_util::sync::CancellationToken::new(),
    )
    .await?;
    if output.cancelled || output.code != Some(0) || output.bytes.len() >= cap {
        return Err(io::Error::other("committed task catalogue unavailable"));
    }
    Ok(output.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn task(extra: &str) -> Task {
        Task::from_committed(
            "repo",
            ".bboxy/tasks/example.md",
            &"a".repeat(40),
            format!("---\ntype: task\nid: TASK-ID\nnumber: 42\n{extra}\n---\nbody\n").as_bytes(),
        )
        .unwrap()
    }
    #[test]
    fn verdicts_use_recorded_results_and_keep_exact_pins() {
        assert_eq!(
            task("status: accepted\nverify_result: {exit_code: 0}").verdict,
            Verdict::Accepted
        );
        assert_eq!(task("status: accepted").verdict, Verdict::None);
        assert_eq!(
            task("verify_result: {exit_code: 1}").verdict,
            Verdict::Failed
        );
        assert_eq!(
            task("verify_result: {exit_code: 1}\nrejection: {reason: bad}").verdict,
            Verdict::Rejected
        );
        let a = task("status: claimed");
        let b = task("status: done\nverify_result: {exit_code: 0}");
        assert_ne!(a.pin, b.pin);
    }
    #[test]
    fn joins_require_task_tools_and_bounded_id_number_or_path_tokens() {
        let t = task("status: claimed");
        for value in [
            "TASK-ID",
            "[[TASK-ID]]",
            "#42",
            ".bboxy/tasks/example.md",
            "tasks/example.md",
        ] {
            assert!(t.matches(
                "mcp__bboxy__task_report",
                &serde_json::json!({"task_path": value})
            ));
            assert!(!t.matches("Read", &serde_json::json!({"task_path": value})));
        }
        for value in [
            "#420",
            "TASK-ID-extra",
            ".bboxy/tasks/example.md.backup",
            "other",
        ] {
            assert!(!t.matches("task_accept", &serde_json::json!({"task": value})));
        }
    }
}
