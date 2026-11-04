// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use bboxy450_harness::{protocol::*, vcs};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

struct Repo {
    home: tempfile::TempDir,
    path: PathBuf,
    head: String,
}
fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
impl Repo {
    fn new() -> Self {
        let home = tempfile::tempdir_in("/tmp").unwrap();
        let path = home.path().join("checkout");
        std::fs::create_dir(&path).unwrap();
        git(&path, &["init", "-b", "vcs-fixture"]);
        // A disposable SSH signing identity needs neither the operator's key
        // nor an agent/Keychain. Even these private fixture commits are signed.
        let key = home.path().join("signing");
        assert!(
            Command::new("/usr/bin/ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-f"])
                .arg(&key)
                .status()
                .unwrap()
                .success()
        );
        for (key, value) in [
            ("user.name", "Boxy"),
            ("user.email", "bboxy450@gmail.com"),
            ("gpg.format", "ssh"),
            ("commit.gpgsign", "true"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git(&path, &["config", key, value]);
        }
        git(&path, &["config", "user.signingkey", key.to_str().unwrap()]);
        std::fs::write(path.join("tracked.txt"), "clean\n").unwrap();
        git(&path, &["add", "tracked.txt"]);
        git(&path, &["commit", "-m", "private VCS fixture"]);
        let head = git(&path, &["rev-parse", "HEAD"]);
        assert!(
            git(&path, &["cat-file", "-p", "HEAD"])
                .contains("gpgsig -----BEGIN SSH SIGNATURE-----")
        );
        Self { home, path, head }
    }
    fn remote(&self) {
        git(
            &self.path,
            &[
                "remote",
                "add",
                "origin",
                "https://fixture-user:fixture-password@GitHub.COM/bboxy450/vcs-fixture.git?token=private-query#private-fragment",
            ],
        );
    }
}
fn executable(path: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
fn run(home: &Path, cwd: &Path, acp: bool, configure: impl FnOnce(&mut Command)) -> Output {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    command
        .args(["--json", "-p", "metadata fixture", "--cwd"])
        .arg(cwd)
        .arg("--sessions")
        .arg(home.join("sessions"))
        .env("HOME", home)
        .env_remove("BBOXY_CAPABILITY")
        .env_remove("BBOXY_CAPABILITY_FILE")
        .env_remove("BBOXY_SEAT_ID")
        .env_remove("BBOXY_SOCK");
    if acp {
        let profiles = home.join(".bboxy450-harness/profiles");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::write(profiles.join("vcs-acp.md"), format!("---\nname: vcs-acp\nprovider: acp-agent\nagent:\n  command: [/usr/bin/python3, -I, {}, text]\non_unattended: deny\n---\n", manifest.join("tests/fixtures/acp-agent/fake_agent.py").display())).unwrap();
        command.args(["--profile", "vcs-acp"]);
    } else {
        command
            .arg("--scripted")
            .arg(manifest.join("tests/fixtures/sse/text.sse"));
    }
    configure(&mut command);
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn events(output: &Output) -> Vec<Envelope> {
    let events: Vec<_> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    laws::check_authored(&events).unwrap();
    events
}
fn meta(events: &[Envelope]) -> &SessionMeta {
    let Event::SessionStarted { meta } = &events[0].event else {
        panic!("no session header")
    };
    meta
}

#[test]
fn normalization_is_shared_and_discards_remote_authentication() {
    for remote in [
        "https://GitHub.COM/bboxy450/vcs-fixture.git",
        "git@GitHub.COM:bboxy450/vcs-fixture.git",
        "ssh://git@GitHub.COM/bboxy450/vcs-fixture.git",
        "git://github.com/bboxy450/vcs-fixture.git",
        "https://fixture-user:fixture-password@github.com/bboxy450/vcs-fixture.git?token=private-query#private-fragment",
    ] {
        assert_eq!(
            vcs::normalize_repo_key(remote).as_deref(),
            Some("github.com/bboxy450/vcs-fixture")
        );
    }
    let oversized = format!(
        "https://fixture-user:fixture-password@host.example/group/{}",
        "x".repeat(20_000)
    );
    assert_eq!(vcs::normalize_repo_key(&oversized), None);
    assert_eq!(vcs::public_remote(&oversized), "<redacted>");
    assert_eq!(
        vcs::normalize_repo_key("ssh://git@host.example:2222/group/repo.git").as_deref(),
        Some("host.example:2222/group/repo")
    );
    for remote in [
        "",
        "relative/repo.git",
        "/absolute/repo.git",
        "file:///tmp/repo.git",
        "https://host.example/repo",
        "ssh://host/owner/repo\nmalformed",
    ] {
        assert_eq!(vcs::normalize_repo_key(remote), None, "{remote}");
    }
}

#[tokio::test]
async fn real_git_capture_records_origin_branch_commit_clean_and_dirty() {
    let repo = Repo::new();
    repo.remote();
    let clean = vcs::capture(&repo.path).await.unwrap();
    assert_eq!(
        clean,
        Vcs {
            repo: Some("github.com/bboxy450/vcs-fixture".into()),
            branch: Some("vcs-fixture".into()),
            commit: Some(repo.head.clone()),
            dirty: Some(false)
        }
    );
    assert_eq!(vcs::repo_key(&repo.path).await, clean.repo);
    std::fs::write(repo.path.join("tracked.txt"), "modified\n").unwrap();
    std::fs::write(repo.path.join("untracked.txt"), "new\n").unwrap();
    assert_eq!(vcs::capture(&repo.path).await.unwrap().dirty, Some(true));
}

#[tokio::test]
async fn missing_origin_uses_real_toplevel_and_no_repository_stays_absent() {
    let repo = Repo::new();
    assert_eq!(
        vcs::capture(&repo.path).await.unwrap().repo,
        Some(format!(
            "path:{}",
            std::fs::canonicalize(&repo.path).unwrap().display()
        ))
    );
    let plain = tempfile::tempdir_in("/tmp").unwrap();
    assert_eq!(vcs::capture(plain.path()).await, None);
    assert_eq!(vcs::repo_key(plain.path()).await, None);
}

#[test]
fn engine_and_acp_headers_capture_without_persisting_remote_credentials() {
    for acp in [false, true] {
        let repo = Repo::new();
        repo.remote();
        let trace = repo.home.path().join("git-trace.json");
        let output = run(repo.home.path(), &repo.path, acp, |command| {
            command
                .env("GIT_TRACE2_EVENT", &trace)
                .env("GIT_TRACE2_ENV_VARS", "VCS_FIXTURE_SECRET")
                .env("VCS_FIXTURE_SECRET", "fixture-private-env-value");
        });
        assert!(
            !trace.exists(),
            "VcsMetadataTraceDisabled: inherited Git tracing wrote outside the capture"
        );
        let events = events(&output);
        let vcs = meta(&events).vcs.as_ref().unwrap();
        assert_eq!(vcs.repo.as_deref(), Some("github.com/bboxy450/vcs-fixture"));
        assert_eq!(vcs.branch.as_deref(), Some("vcs-fixture"));
        assert_eq!(vcs.commit.as_deref(), Some(repo.head.as_str()));
        assert_eq!(vcs.dirty, Some(false));
        let log = std::fs::read(
            repo.home
                .path()
                .join("sessions")
                .join(format!("{}.jsonl", meta(&events).session)),
        )
        .unwrap();
        for forbidden in [
            "fixture-user",
            "fixture-password",
            "private-query",
            "private-fragment",
            "fixture-private-env-value",
        ] {
            assert!(!String::from_utf8_lossy(&log).contains(forbidden));
            assert!(!String::from_utf8_lossy(&output.stdout).contains(forbidden));
            assert!(!String::from_utf8_lossy(&output.stderr).contains(forbidden));
        }
    }
}

#[test]
fn missing_git_and_git_errors_degrade_without_failing_a_turn() {
    for failure in [false, true] {
        let home = tempfile::tempdir_in("/tmp").unwrap();
        let bin = home.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        if failure {
            executable(
                &bin.join("git"),
                "#!/bin/sh\nprintf '%s' 'fixture-password' >&2\nexit 72\n",
            );
        }
        let output = run(home.path(), home.path(), false, |c| {
            c.env("PATH", &bin);
        });
        assert!(meta(&events(&output)).vcs.is_none());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-password"));
    }
}

#[tokio::test]
async fn stalled_git_is_bounded_and_its_owned_children_are_reaped() {
    let home = tempfile::tempdir_in("/tmp").unwrap();
    let bin = home.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    executable(
        &bin.join("git"),
        "#!/bin/sh\ntrap '' TERM\nprintf '%s' \"$$\" > \"$VCS_PROBE_PID\"\n/bin/sleep 600 &\nprintf '%s' \"$!\" > \"$VCS_MEMBER_PID\"\nwait\n",
    );
    let pid = home.path().join("leader");
    let member = home.path().join("member");
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_bboxy450-harness"));
    child
        .args(["--json", "-p", "fixture", "--cwd"])
        .arg(home.path())
        .arg("--sessions")
        .arg(home.path().join("sessions"))
        .arg("--scripted")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sse/text.sse"))
        .env("HOME", home.path())
        .env("PATH", &bin)
        .env("VCS_PROBE_PID", &pid)
        .env("VCS_MEMBER_PID", &member)
        .env_remove("BBOXY_CAPABILITY")
        .env_remove("BBOXY_CAPABILITY_FILE")
        .env_remove("BBOXY_SEAT_ID")
        .kill_on_drop(true);
    use tokio::io::AsyncReadExt;
    child
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = child.spawn().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let status = match tokio::time::timeout(Duration::from_secs(15), child.wait()).await {
        Ok(status) => status.unwrap(),
        Err(_) => {
            // Retain the Child/PID lease while requesting the app's synchronous
            // owned-group shutdown; even a failing test leaves no Git member.
            if let Some(pid) = child.id() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            if tokio::time::timeout(Duration::from_secs(3), child.wait())
                .await
                .is_err()
            {
                child.kill().await.unwrap();
            }
            panic!("Git metadata capture did not finish within its bounded startup");
        }
    };
    let mut out = vec![];
    let mut err = vec![];
    stdout.read_to_end(&mut out).await.unwrap();
    stderr.read_to_end(&mut err).await.unwrap();
    let output = Output {
        status,
        stdout: out,
        stderr: err,
    };
    assert!(output.status.success());
    assert!(meta(&events(&output)).vcs.is_none());
    for file in [pid, member] {
        let pid: i32 = std::fs::read_to_string(file).unwrap().parse().unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            while unsafe { libc::kill(pid, 0) } != -1 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("owned Git process survived bounded capture");
    }
}
