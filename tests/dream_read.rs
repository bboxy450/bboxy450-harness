// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Private fixture-parent and actual sibling-runner witnesses. No operator inputs.
#![allow(dead_code)]
#[path = "fixtures/dream/trust/fixture.rs"]
mod fixture;
#[path = "fixtures/dream/good/receipt_support.rs"]
mod r;
#[path = "fixtures/dream-read/root_fixture.rs"]
mod root_fixture;
#[path = "fixtures/dream/sinks/support.rs"]
mod sinks;
#[path = "fixtures/dream/good/audit_fixture.rs"]
mod sufficient;
#[path = "fixtures/dream/good/support.rs"]
mod support;
use bboxy450_harness::{
    dream::{bootstrap as boot, entry, inspect, lower, read_wire as w, receipt, seal},
    protocol::*,
    redact::Set,
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
            process::{CommandExt, ExitStatusExt},
        },
    },
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

fn private_dir(path: &Path) {
    std::fs::create_dir(path).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
fn private_file(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
fn add_file(files: &mut BTreeMap<String, Vec<u8>>, path: String, bytes: Vec<u8>) {
    if let Some(old) = files.insert(path, bytes.clone()) {
        assert_eq!(old, bytes);
    }
}
fn copy_tree(files: &mut BTreeMap<String, Vec<u8>>, from: &Path, prefix: &str) {
    for item in std::fs::read_dir(from).unwrap() {
        let item = item.unwrap();
        let name = item.file_name().into_string().unwrap();
        let dest = format!("{prefix}/{name}");
        let metadata = item.file_type().unwrap();
        assert!(!metadata.is_symlink());
        if metadata.is_dir() {
            copy_tree(files, &item.path(), &dest);
        } else {
            assert!(metadata.is_file());
            add_file(files, dest, std::fs::read(item.path()).unwrap());
        }
    }
}
fn git(f: &sufficient::AuditFixture, args: &[&str]) -> Vec<u8> {
    let output = Command::new("/usr/bin/git")
        .args([
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .current_dir(&f.fixture.repo.path)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/opt/homebrew/bin")
        .env("HOME", f.fixture.repo.home.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "private fixture git: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
fn text_git(f: &sufficient::AuditFixture, args: &[&str]) -> String {
    String::from_utf8(git(f, args)).unwrap().trim().to_owned()
}
fn event_pin(event: &Envelope) -> w::EventPin {
    let bytes = event.wire_bytes().unwrap();
    w::EventPin {
        session: w::Ulid(event.session.to_string()),
        seq: w::Number(event.seq.unwrap().0),
        digest: w::Digest(entry::digest(bytes.strip_suffix(b"\n").unwrap())),
    }
}
fn rebind_inventory(mut input: w::Input, files: &BTreeMap<String, Vec<u8>>) -> w::Input {
    let old = input
        .evidence
        .files
        .iter()
        .map(|f| (f.slot, f.path.0.clone()))
        .collect::<BTreeMap<_, _>>();
    let inventory = files
        .iter()
        .enumerate()
        .map(|(slot, (path, bytes))| w::BundleFile {
            slot: slot as u32,
            path: w::RelativePath(path.clone()),
            len: w::Number(bytes.len() as u64),
            sha256: w::hash(bytes),
        })
        .collect::<Vec<_>>();
    let slots = inventory
        .iter()
        .map(|f| (f.path.0.as_str(), f.slot))
        .collect::<BTreeMap<_, _>>();
    input.evidence.policy = slots[old[&input.evidence.policy].as_str()];
    input
        .evidence
        .public_keys
        .retain_mut(|key| match slots.get(old[&key.file].as_str()) {
            Some(slot) => {
                key.file = *slot;
                true
            }
            None => false,
        });
    input
        .evidence
        .artifacts
        .retain_mut(|a| match slots.get(old[&a.file].as_str()) {
            Some(slot) => {
                a.file = *slot;
                true
            }
            None => false,
        });
    input.evidence.files = inventory;
    input
}
struct HelperProvision {
    home: tempfile::TempDir,
    receipt: PathBuf,
    digest: String,
    evidence: serde_json::Value,
    probe: PathBuf,
    git: PathBuf,
    gpg: PathBuf,
    parent: Option<Arc<HelperProvision>>,
}
fn provision_command(
    program: &Path,
    args: &[&str],
    home: &Path,
    input: Option<&[u8]>,
) -> std::process::Output {
    let mut child = Command::new(program)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/opt/homebrew/bin")
        .env("HOME", home)
        .env("GNUPGHOME", home)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(bytes) = input {
        child.stdin.take().unwrap().write_all(bytes).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.stdout.len() <= 128 * 1024 && output.stderr.len() <= 128 * 1024,
        "private provisioning output bound"
    );
    assert!(
        output.status.success(),
        "private provisioning failed {} {:?}: {}",
        program.display(),
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn os_release() -> String {
    let mut value = std::mem::MaybeUninit::<libc::utsname>::zeroed();
    assert_eq!(unsafe { libc::uname(value.as_mut_ptr()) }, 0);
    let value = unsafe { value.assume_init() };
    unsafe { std::ffi::CStr::from_ptr(value.release.as_ptr()) }
        .to_str()
        .unwrap()
        .to_owned()
}
fn capability_command(
    probe: &Path,
    args: &[&str],
    home: &Path,
    label: &str,
) -> std::process::Output {
    let output = Command::new(probe)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", home)
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.stdout.len() <= 128 * 1024 && output.stderr.len() <= 128 * 1024);
    private_file(&home.join(format!("{label}.stdout")), &output.stdout);
    private_file(&home.join(format!("{label}.stderr")), &output.stderr);
    private_file(
        &home.join(format!("{label}.status")),
        output
            .status
            .code()
            .map_or_else(|| "signal".into(), |v| v.to_string())
            .as_bytes(),
    );
    if !output.status.success() {
        let retained = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "dream-read-capability-failed-{}",
                ulid::Ulid::new()
            ));
        private_dir(&retained);
        for name in [
            format!("{label}.stdout"),
            format!("{label}.stderr"),
            format!("{label}.status"),
        ] {
            private_file(
                &retained.join(&name),
                &std::fs::read(home.join(name)).unwrap(),
            );
        }
        for name in [
            "native-probe.stdout",
            "native-probe.stderr",
            "native-probe.status",
        ] {
            if let Ok(bytes) = std::fs::read(home.join(name)) {
                private_file(&retained.join(name), &bytes);
            }
        }
        private_file(&retained.join("probe"), &std::fs::read(probe).unwrap());
        private_file(
            &retained.join("proof-helper-probe.c"),
            include_bytes!("fixtures/dream-read/proof-helper-probe.c"),
        );
        private_file(&retained.join("command.json"), &w::canonical(&serde_json::json!({"program":probe,"args":args,"scope":"private native/tool component; no managed Dream worker"})).unwrap());
        panic!(
            "capability witness failed {:?}; retained {}: {}",
            output.status.code(),
            retained.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}
impl HelperProvision {
    fn build(f: &sufficient::AuditFixture) -> Arc<Self> {
        let home = tempfile::Builder::new()
            .prefix("dream-read-provision-")
            .tempdir_in("/private/tmp")
            .unwrap();
        let root = home.path();
        let public = root.join("public");
        let native = root.join("native");
        let scratch = root.join("signature-tmp");
        for path in [&public, &native, &scratch] {
            private_dir(path);
        }
        private_file(&public.join("gpg.conf"),b"batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\n");
        let found = provision_command(
            Path::new("/usr/bin/xcrun"),
            &["--find", "git"],
            &public,
            None,
        );
        let git =
            std::fs::canonicalize(std::str::from_utf8(&found.stdout).unwrap().trim()).unwrap();
        let gpg = std::fs::canonicalize("/opt/homebrew/bin/gpg").unwrap();
        assert!(git.is_absolute() && gpg.is_absolute());
        let git_build =
            String::from_utf8(provision_command(&git, &["--version"], &public, None).stdout)
                .unwrap()
                .trim()
                .to_owned();
        let gpg_build = String::from_utf8(
            provision_command(
                &gpg,
                &[
                    "--no-options",
                    "--no-autostart",
                    "--batch",
                    "--no-tty",
                    "--version",
                ],
                &public,
                None,
            )
            .stdout,
        )
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_owned();
        let policy: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&f.fixture.policy).unwrap()).unwrap();
        let key = std::fs::read(
            policy["git_signers"][0]["public_key_file"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        provision_command(
            &gpg,
            &[
                "--no-options",
                "--no-autostart",
                "--batch",
                "--no-tty",
                "--import",
                "-",
            ],
            &public,
            Some(&key),
        );
        let probe = root.join("probe");
        let source = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/dream-read/proof-helper-probe.c"
        ));
        provision_command(
            Path::new("/usr/bin/clang"),
            &[
                "-O2",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                source.to_str().unwrap(),
                "-o",
                probe.to_str().unwrap(),
            ],
            root,
            None,
        );
        std::fs::set_permissions(&probe, std::fs::Permissions::from_mode(0o700)).unwrap();
        let native_output = capability_command(
            &probe,
            &["native", native.to_str().unwrap()],
            root,
            "native-probe",
        );
        let native_evidence: serde_json::Value =
            serde_json::from_slice(&native_output.stdout).unwrap();
        assert_eq!(native_evidence["created_suspended"], true);
        assert_eq!(native_evidence["cloexec_default"], true);
        assert_eq!(native_evidence["audit_token_signals"], true);
        assert_eq!(native_evidence["running_child_stop_applied"], true);
        assert!(
            matches!(native_evidence["stale_audit_error"].as_i64(),Some(v) if v==libc::ESRCH as i64||v==libc::EINVAL as i64)
        );
        assert_eq!(native_evidence["stale_pid_version_control"], true);
        assert_eq!(native_evidence["members"], 2);
        assert_eq!(native_evidence["reaped"], true);
        assert_eq!(native_evidence["threads"], 1);
        assert_eq!(native_evidence["confirmed_runnable"], 0);
        assert_eq!(native_evidence["held_user_cpu_delta_ns"], 0);
        assert_eq!(
            native_evidence["held_kernel_paid_debt_ns"]
                .as_u64()
                .unwrap(),
            9 * native_evidence["held_kernel_cpu_ns"].as_u64().unwrap()
        );
        assert_eq!(
            native_evidence["native_bootstrap_reconciliation_ns"]
                .as_u64()
                .unwrap(),
            native_evidence["confirmed_native_cpu_ns"].as_u64().unwrap()
                - native_evidence["initial_native_cpu_ns"].as_u64().unwrap()
        );
        assert_eq!(
            native_evidence["sampled_child_cpu_ns"].as_u64().unwrap() as u128,
            native_evidence["sampled_child_cpu_raw"].as_u64().unwrap() as u128
                * native_evidence["mach_timebase_numer"].as_u64().unwrap() as u128
                / native_evidence["mach_timebase_denom"].as_u64().unwrap() as u128
        );
        assert!(native_evidence["waited_child_cpu_ns"].as_u64().unwrap() >= 12_000_000);
        assert!(
            native_evidence["leader_wait4_cpu_ns"].as_u64().unwrap()
                >= native_evidence["waited_child_cpu_ns"].as_u64().unwrap()
        );
        let trace = root.join("git-trace.jsonl");
        let output = root.join("git-signature-result");
        let commit = text_git(f, &["rev-parse", "refs/remotes/origin/main"]);
        let git_output = capability_command(
            &probe,
            &[
                "git",
                git.to_str().unwrap(),
                gpg.to_str().unwrap(),
                f.fixture.repo.path.to_str().unwrap(),
                public.to_str().unwrap(),
                &commit,
                scratch.to_str().unwrap(),
                trace.to_str().unwrap(),
                output.to_str().unwrap(),
            ],
            root,
            "git-probe",
        );
        let git_evidence: serde_json::Value = serde_json::from_slice(&git_output.stdout).unwrap();
        assert_eq!(git_evidence["max_members"], 2);
        assert_eq!(git_evidence["max_threads_per_process"], 1);
        assert_eq!(git_evidence["exact_pgid_membership"], true);
        assert_eq!(git_evidence["git_fork_before_gpg_exec"], true);
        assert!((1..=16).contains(&git_evidence["preexec_observations"].as_u64().unwrap()));
        assert!(git_evidence["preexec_peak_cpu_ns"].as_u64().unwrap() <= 1_000_000);
        assert!(git_evidence["max_full_grant_charge_ns"].as_u64().unwrap() <= 1_000_000);
        assert!(git_evidence["confirmed_stopped_rests"].as_u64().unwrap() > 0);
        assert_eq!(git_evidence["stopped_rest_user_cpu_delta_ns"], 0);
        assert_eq!(
            git_evidence["late_kernel_paid_debt_ns"].as_u64().unwrap(),
            9 * git_evidence["late_kernel_cpu_ns"].as_u64().unwrap()
        );
        assert!(
            git_evidence["shared_reconciliation_paid_debt_ns"]
                .as_u64()
                .unwrap()
                >= git_evidence["late_kernel_paid_debt_ns"].as_u64().unwrap()
        );
        assert_eq!(git_evidence["reaped"], true);
        let git_cpu = git_evidence["sampled_git_cpu_ns"].as_u64().unwrap();
        let gpg_cpu = git_evidence["sampled_gpg_cpu_ns"].as_u64().unwrap();
        assert!(
            gpg_cpu > 0 && git_evidence["git_wait4_cpu_ns"].as_u64().unwrap() >= git_cpu + gpg_cpu
        );
        assert!(
            git_evidence["rolled_gpg_cpu_ns"].as_u64().unwrap() >= gpg_cpu,
            "actual Git kernel child-rollup must retain observed GPG CPU"
        );
        assert!(git_evidence["group_peak_rss_bytes"].as_u64().unwrap() <= 16 * 1024 * 1024);
        let signature = std::fs::read(&output).unwrap();
        assert!(
            matches!(signature.first(), Some(b'G' | b'U')),
            "actual Git signature status"
        );
        let raw_trace = std::fs::read(&trace).unwrap();
        assert!(raw_trace.len() <= 128 * 1024);
        let trace_events = raw_trace
            .split(|b| *b == b'\n')
            .filter(|b| !b.is_empty())
            .map(|line| serde_json::from_slice::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        let starts = trace_events
            .iter()
            .filter(|v| v["event"] == "child_start")
            .collect::<Vec<_>>();
        assert_eq!(
            starts.len(),
            1,
            "actual Git must create only one implicit verification child"
        );
        assert_eq!(starts[0]["use_shell"], false);
        let argv = starts[0]["argv"].as_array().unwrap();
        assert_eq!(argv.len(), 6);
        assert_eq!(argv[0], gpg.to_str().unwrap());
        assert_eq!(argv[1], "--keyid-format=long");
        assert_eq!(argv[2], "--status-fd=1");
        assert_eq!(argv[3], "--verify");
        assert_eq!(argv[5], "-");
        let signature_file = Path::new(argv[4].as_str().unwrap());
        let resolved = if signature_file.is_absolute() {
            signature_file.to_owned()
        } else {
            f.fixture.repo.path.join(signature_file)
        };
        assert!(
            (resolved.starts_with(&scratch) || resolved.starts_with(&f.fixture.repo.path))
                && resolved
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(".git_vtag_tmp"),
            "bounded actual Git signature pathname"
        );
        assert!(
            !resolved.exists(),
            "Git must retire its private signature file"
        );
        let exit = trace_events
            .iter()
            .find(|v| v["event"] == "child_exit" && v["child_id"] == starts[0]["child_id"])
            .unwrap();
        assert_eq!(exit["code"], 0);
        assert_eq!(exit["pid"], git_evidence["gpg_pid"]);
        let evidence = serde_json::json!({"native":native_evidence,"git":git_evidence,"git_trace":trace_events,"git_trace_sha256":w::hash(&raw_trace).0,"git_signature_result_sha256":w::hash(&signature).0});
        let evidence_bytes = w::canonical(&evidence).unwrap();
        assert!(evidence_bytes.len() <= 128 * 1024);
        private_file(&root.join("capability-evidence.json"), &evidence_bytes);
        let receipt_value = serde_json::json!({"schema":"bboxy.dream.proof-tools/1","source_sha256":bboxy450_harness::dream::resolve::provision_source_sha256(),
            "os":{"release":os_release(),"arch":std::env::consts::ARCH},
            "git":{"absolute_path":git,"sha256":w::hash(std::fs::read(&git).unwrap()).0,"build":git_build,"features":["git-readonly","openpgp-verify-absolute-gpg"]},
            "gpg":{"absolute_path":gpg,"sha256":w::hash(std::fs::read(&gpg).unwrap()).0,"build":gpg_build,"features":["public-key-stdin","no-autostart"]},
            "capability":{"probe_source_sha256":w::hash(std::fs::read(source).unwrap()).0,"probe_exe_sha256":w::hash(std::fs::read(&probe).unwrap()).0,"evidence_sha256":w::hash(&evidence_bytes).0,
                "features":["darwin-start-suspended","setpgroup","cloexec-default","audit-token-signals","birth-identity","group-members-2","single-threaded-helpers","descendant-cpu-rollup","git-gpg-fixed-argv","git-fork-before-gpg-exec","mach-timebase-cpu-ns"]}});
        let bytes = w::canonical(&receipt_value).unwrap();
        assert!(bytes.len() <= 32 * 1024);
        let receipt = root.join("helper-provision.json");
        private_file(&receipt, &bytes);
        Arc::new(Self {
            home,
            receipt,
            digest: w::hash(&bytes).0,
            evidence,
            probe,
            git,
            gpg,
            parent: None,
        })
    }
    fn altered(
        self: &Arc<Self>,
        edit: impl FnOnce(&mut serde_json::Value),
        bind_new_digest: bool,
    ) -> Arc<Self> {
        let home = tempfile::Builder::new()
            .prefix("dream-read-provision-negative-")
            .tempdir_in("/private/tmp")
            .unwrap();
        let receipt = home.path().join("helper-provision.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&self.receipt).unwrap()).unwrap();
        edit(&mut value);
        let bytes = w::canonical(&value).unwrap();
        private_file(&receipt, &bytes);
        Arc::new(Self {
            home,
            receipt,
            digest: if bind_new_digest {
                w::hash(&bytes).0
            } else {
                self.digest.clone()
            },
            evidence: self.evidence.clone(),
            probe: self.probe.clone(),
            git: self.git.clone(),
            gpg: self.gpg.clone(),
            parent: Some(self.clone()),
        })
    }
    fn retain_evidence(&self) -> PathBuf {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        let retained = target.join(format!("dream-read-capability-{}", ulid::Ulid::new()));
        private_dir(&retained);
        for name in [
            "helper-provision.json",
            "capability-evidence.json",
            "git-trace.jsonl",
            "git-signature-result",
            "native-probe.stdout",
            "native-probe.stderr",
            "native-probe.status",
            "git-probe.stdout",
            "git-probe.stderr",
            "git-probe.status",
            "probe",
        ] {
            let bytes = std::fs::read(self.home.path().join(name)).unwrap();
            private_file(&retained.join(name), &bytes);
        }
        std::fs::set_permissions(
            retained.join("probe"),
            std::fs::Permissions::from_mode(0o700),
        )
        .unwrap();
        private_file(
            &retained.join("proof-helper-probe.c"),
            include_bytes!("fixtures/dream-read/proof-helper-probe.c"),
        );
        let recipe = serde_json::json!({"program":"/usr/bin/clang","args":["-O2","-std=c11","-Wall","-Wextra","-Werror","proof-helper-probe.c","-o","probe"],
            "scope":"private component capability only; no managed Dream worker invocation","origin":"https://github.com/bboxy450/bboxy450-harness","policy":"https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md"});
        private_file(
            &retained.join("probe-build.json"),
            &w::canonical(&recipe).unwrap(),
        );
        retained
    }
}
struct Template {
    fixture: sufficient::AuditFixture,
    input: w::Input,
    files: BTreeMap<String, Vec<u8>>,
    entry: String,
    publication: w::EventPin,
    provision: Arc<HelperProvision>,
}
impl Template {
    async fn publication() -> Self {
        Self::publication_root("fixture.invalid/dream/repo").await
    }
    async fn publication_root(root: &str) -> Self {
        let (f, logs, publication) = if root == "fixture.invalid/dream/repo" {
            // Actual construction from dream_recall::publication_kind: completed lower,
            // exact accepted publication/adoption suffix and final constructive outer seal.
            let f = sinks::lower_fixture();
            let mut resolver = f.resolver().await;
            let store = sinks::lower_store(&f).await;
            let outer = seal::Owner::default();
            let plan = lower::prepare_at(
                &store,
                &mut resolver,
                sinks::context_sink(&f),
                "engine",
                None,
                &Set::default(),
                4005,
            )
            .await
            .unwrap();
            let (store, published) = lower::publish(store, plan, outer.clone(), Set::default())
                .unwrap()
                .settle()
                .await;
            let published = published.unwrap();
            let item = published.accepted.item;
            let (store, _) = sinks::finish_run(store, &f, &outer).await;
            let mut logs = (*resolver.logs).clone();
            logs.insert(store.session, store.events.clone());
            let publication = event_pin(logs[&store.session].iter().find(|event| {
            matches!(&event.event, Event::ItemCompleted {item: found,..} if *found == item)
        }).unwrap());
            (f, logs, publication)
        } else {
            let built = root_fixture::completed_publication(root).await;
            let publication = event_pin(&built.publication);
            (built.fixture, built.logs, publication)
        };
        let provision = HelperProvision::build(&f);
        assert!(
            logs.values().flatten().all(|event| !matches!(
                &event.event,
                Event::ItemCompleted {
                    body: Item::Recall { .. },
                    ..
                }
            )),
            "preparation fixture must not invent Recall"
        );
        assert!(
            logs.len() <= 32,
            "fixture exceeds admitted finite source count"
        );
        let mut files = BTreeMap::new();
        let mut sources = vec![];
        let mut artifacts = BTreeMap::<String, String>::new();
        for (session, events) in &logs {
            let logical = f.sessions.join(format!("{session}.jsonl"));
            let bytes = std::fs::read(&logical).unwrap();
            assert_eq!(bytes.last(), Some(&b'\n'));
            let final_line = bytes[..bytes.len() - 1]
                .rsplit(|b| *b == b'\n')
                .next()
                .unwrap();
            let mut blobs = BTreeMap::new();
            for event in events {
                for blob in inspect::blobs(&event.event).unwrap() {
                    blobs.insert(blob.sha256.clone(), blob.clone());
                }
            }
            let mut pinned_blobs = vec![];
            for blob in blobs.values() {
                let logical = f
                    .sessions
                    .join(session.to_string())
                    .join("blobs")
                    .join(&blob.sha256);
                let physical = format!("input/{session}/blobs/{}", blob.sha256);
                let bytes = std::fs::read(&logical).unwrap();
                assert_eq!(bytes.len() as u64, blob.len);
                assert_eq!(w::hash(&bytes).0, blob.sha256);
                artifacts.insert(logical.to_str().unwrap().to_owned(), physical.clone());
                add_file(&mut files, physical, bytes.clone());
                pinned_blobs.push(bboxy450_harness::dream::c1_wire::Blob {
                    sha256: w::Hex(blob.sha256.clone()),
                    len: w::Number(blob.len),
                    mime: bboxy450_harness::dream::c1_wire::Mime(blob.mime.clone()),
                });
                if blob.mime == receipt::MIME {
                    let body: receipt::Body = serde_json::from_slice(&bytes).unwrap();
                    if let Some(selection) = body.selection {
                        for generated in selection.generated {
                            let logical = generated.path.to_str().unwrap().to_owned();
                            if !artifacts.contains_key(&logical) {
                                let physical =
                                    format!("input/evidence/artifacts/{:06}", artifacts.len());
                                let bytes = std::fs::read(&generated.path).unwrap();
                                assert_eq!(w::hash(&bytes).0, generated.sha256);
                                artifacts.insert(logical, physical.clone());
                                add_file(&mut files, physical, bytes);
                            }
                        }
                    }
                }
            }
            let Event::SessionStarted { meta } = &events[0].event else {
                panic!("closed source header")
            };
            sources.push(w::Source {
                session: w::Ulid(session.to_string()),
                source_generation: w::Number(1),
                repo: bboxy450_harness::dream::c1_wire::Repo(
                    meta.vcs.as_ref().unwrap().repo.clone().unwrap(),
                ),
                len: w::Number(bytes.len() as u64),
                sha256: w::hash(&bytes),
                closed_seq: w::Number(events.last().unwrap().seq.unwrap().0),
                closed_line_sha256: w::hash(final_line),
                blobs: pinned_blobs,
            });
            add_file(&mut files, format!("input/{session}.jsonl"), bytes);
        }
        let main = text_git(&f, &["rev-parse", "refs/remotes/origin/main"]);
        let root_dir = "input/evidence/roots/0";
        copy_tree(
            &mut files,
            &f.fixture.repo.path.join(".git/objects"),
            &format!("{root_dir}/.git/objects"),
        );
        copy_tree(
            &mut files,
            &f.fixture.repo.path.join(".bboxy"),
            &format!("{root_dir}/.bboxy"),
        );
        add_file(
            &mut files,
            format!("{root_dir}/.git/HEAD"),
            b"ref: refs/heads/fixture\n".to_vec(),
        );
        add_file(
            &mut files,
            format!("{root_dir}/.git/refs/heads/fixture"),
            format!("{main}\n").into_bytes(),
        );
        add_file(
            &mut files,
            format!("{root_dir}/.git/refs/remotes/origin/main"),
            format!("{main}\n").into_bytes(),
        );
        add_file(&mut files,format!("{root_dir}/.git/config"),
            format!("[core]\nrepositoryformatversion = 0\nbare = false\n[remote \"origin\"]\nurl = https://{root}.git\n").into_bytes());
        let mut catalogue = BTreeMap::new();
        for commit in text_git(&f, &["rev-list", "--all"]).lines() {
            let names = git(
                &f,
                &["ls-tree", "-rz", "--name-only", commit, "--", ".bboxy"],
            );
            for path in names.split(|b| *b == 0).filter(|b| !b.is_empty()) {
                let path = std::str::from_utf8(path).unwrap();
                if !path.ends_with(".md") {
                    continue;
                }
                let bytes = git(&f, &["show", &format!("{commit}:{path}")]);
                let record = entry::Record::parse(&bytes).unwrap();
                let id = record.front["id"].as_str().unwrap().to_owned();
                catalogue.insert(
                    (path.to_owned(), commit.to_owned()),
                    w::RecordPin {
                        root: w::Id(root.into()),
                        id: w::Id(id),
                        path: w::PathText(path.into()),
                        commit: w::Commit(commit.into()),
                        digest: w::Digest(entry::digest(&bytes)),
                    },
                );
            }
        }
        let policy_bytes = std::fs::read(&f.fixture.policy).unwrap();
        let policy_value: serde_json::Value = serde_json::from_slice(&policy_bytes).unwrap();
        add_file(
            &mut files,
            "input/evidence/policy.json".into(),
            policy_bytes.clone(),
        );
        let mut key_paths = vec![];
        for signer in policy_value["git_signers"].as_array().unwrap() {
            let physical = format!("input/evidence/keys/{:06}.asc", key_paths.len());
            let logical = signer["public_key_file"].as_str().unwrap();
            add_file(
                &mut files,
                physical.clone(),
                std::fs::read(logical).unwrap(),
            );
            key_paths.push((signer["fingerprint"].as_str().unwrap().to_owned(), physical));
        }
        let inventory: Vec<_> = files
            .iter()
            .enumerate()
            .map(|(slot, (path, bytes))| w::BundleFile {
                slot: slot as u32,
                path: w::RelativePath(path.clone()),
                len: w::Number(bytes.len() as u64),
                sha256: w::hash(bytes),
            })
            .collect();
        let slots: BTreeMap<_, _> = inventory
            .iter()
            .map(|f| (f.path.0.clone(), f.slot))
            .collect();
        let input = w::Input {
            schema: w::InputSchema::V1,
            nonce: w::Ulid(ulid::Ulid::new().to_string()),
            context: w::Context {
                account: w::Id("fixture".into()),
                account_epoch: w::Number(1),
                seat: w::Ulid(ulid::Ulid::new().to_string()),
                generation: w::Number(1),
                grant_epoch: w::Number(1),
                swarm: w::Ulid(ulid::Ulid::new().to_string()),
                swarm_revision: w::Number(1),
                workspace: w::Id("fixture".into()),
                workspace_revision: w::Number(1),
                root: w::Id(root.into()),
                root_revision: w::Number(1),
                provider: w::Id("git".into()),
                provider_epoch: w::Number(1),
            },
            closed_sources: sources,
            records: vec![w::RecordPin::try_from(f.record.clone()).unwrap()],
            publications: vec![publication.clone()],
            trust_policy: w::hash(&policy_bytes),
            provider_revision: w::Id(main.clone()),
            evidence: w::EvidenceBundle {
                files: inventory,
                roots: vec![w::RootBinding {
                    root: w::Id(root.into()),
                    repo: w::Id(root.into()),
                    main: w::Commit(main),
                    dir: w::RelativePath(root_dir.into()),
                }],
                policy: slots["input/evidence/policy.json"],
                public_keys: key_paths
                    .into_iter()
                    .map(|(fingerprint, path)| w::KeyBinding {
                        fingerprint: w::Id(fingerprint),
                        file: slots[&path],
                    })
                    .collect(),
                artifacts: artifacts
                    .into_iter()
                    .map(|(logical, path)| w::ArtifactBinding {
                        logical: w::PathText(logical),
                        file: slots[&path],
                    })
                    .collect(),
                catalogue: catalogue.into_values().collect(),
                selected_at_ms: w::Number(bboxy450_harness::store::now_ms()),
            },
        };
        let entry = f.entry.clone();
        eprintln!(
            "read-fixture selected_sources={} selected_envelopes={} inventory_files={} inventory_bytes={} catalogue_pins={} selected_blobs={}",
            input.closed_sources.len(),
            logs.values().map(Vec::len).sum::<usize>(),
            input.evidence.files.len(),
            input.evidence.files.iter().map(|f| f.len.0).sum::<u64>(),
            input.evidence.catalogue.len(),
            input
                .closed_sources
                .iter()
                .map(|s| s.blobs.len())
                .sum::<usize>()
        );
        Self {
            fixture: f,
            input,
            files,
            entry,
            publication,
            provision,
        }
    }
    fn hide_originals(&self) {
        let home = self.fixture.fixture.repo.home.path();
        for (from, to) in [
            ("repo", "offline-repo"),
            ("sessions", "offline-sessions"),
            (".bboxy450-harness", "offline-sinks"),
            ("trust.json", "offline-policy.json"),
            ("public.asc", "offline-public.asc"),
        ] {
            if home.join(from).exists() {
                std::fs::rename(home.join(from), home.join(to)).unwrap();
            }
        }
    }
    fn job(&self) -> Job {
        Job::materialize(self.input.clone(), &self.files).with_provision(self.provision.clone())
    }
    fn changed_committed_record(&self, revival: bool) -> Job {
        // This is an independently signed, committed current head, not invalid
        // record bytes. A new revival or terminal status cannot inherit adoption.
        let f = &self.fixture;
        let original = self.input.evidence.roots[0].main.0.clone();
        let path = f.fixture.repo.path.join(".bboxy/memories/lesson.md");
        let record = entry::Record::parse(&std::fs::read(&path).unwrap()).unwrap();
        let mut front = record.front.clone();
        if revival {
            let mut identity = record.validate_identity().unwrap();
            identity.revival += 1;
            identity.derived_from = vec![self.entry.clone()];
            for (key, value) in serde_json::to_value(&identity)
                .unwrap()
                .as_object()
                .unwrap()
            {
                front[key] = value.clone();
            }
            front["entry"] = identity.id().unwrap().into();
        } else {
            front["status"] = "expired".into();
            front["harnesses"] = serde_json::json!([]);
        }
        let changed = support::sign_record(front, &record.body, &f.fixture.repo.key);
        std::fs::write(&path, &changed.bytes).unwrap();
        f.fixture.repo.git(&["add", ".bboxy/memories/lesson.md"]);
        f.fixture.repo.git(&[
            "commit",
            "-qm",
            if revival {
                "private distinct revival"
            } else {
                "private current terminal record"
            },
        ]);
        let main = text_git(f, &["rev-parse", "HEAD"]);
        f.fixture
            .repo
            .git(&["update-ref", "refs/remotes/origin/main", &main]);
        let mut files = self.files.clone();
        let root_dir = &self.input.evidence.roots[0].dir.0;
        files.retain(|p, _| {
            !p.starts_with(&format!("{root_dir}/.git/objects/"))
                && !p.starts_with(&format!("{root_dir}/.bboxy/"))
        });
        copy_tree(
            &mut files,
            &f.fixture.repo.path.join(".git/objects"),
            &format!("{root_dir}/.git/objects"),
        );
        copy_tree(
            &mut files,
            &f.fixture.repo.path.join(".bboxy"),
            &format!("{root_dir}/.bboxy"),
        );
        files.insert(
            format!("{root_dir}/.git/refs/heads/fixture"),
            format!("{main}\n").into_bytes(),
        );
        files.insert(
            format!("{root_dir}/.git/refs/remotes/origin/main"),
            format!("{main}\n").into_bytes(),
        );
        let mut input = self.input.clone();
        input.evidence.roots[0].main = w::Commit(main.clone());
        input.provider_revision = w::Id(main.clone());
        let names = git(f, &["ls-tree", "-rz", "--name-only", &main, "--", ".bboxy"]);
        for path in names.split(|b| *b == 0).filter(|p| !p.is_empty()) {
            let path = std::str::from_utf8(path).unwrap();
            if !path.ends_with(".md") {
                continue;
            }
            let bytes = git(f, &["show", &format!("{main}:{path}")]);
            let record = entry::Record::parse(&bytes).unwrap();
            input.evidence.catalogue.push(w::RecordPin {
                root: self.input.context.root.clone(),
                id: w::Id(record.front["id"].as_str().unwrap().into()),
                path: w::PathText(path.into()),
                commit: w::Commit(main.clone()),
                digest: w::Digest(entry::digest(&bytes)),
            });
        }
        f.fixture.repo.git(&["reset", "--hard", &original]);
        f.fixture
            .repo
            .git(&["update-ref", "refs/remotes/origin/main", &original]);
        Job::materialize(rebind_inventory(input, &files), &files)
            .with_provision(self.provision.clone())
    }
}
// Adapted from this project's private seat fixture; retain failed owned inputs
// without changing successful fixture cleanup or touching another test's HOME.
struct ReadFailureHome(tempfile::TempDir);
impl ReadFailureHome {
    fn path(&self) -> &Path {
        self.0.path()
    }
}
impl Drop for ReadFailureHome {
    fn drop(&mut self) {
        if thread::panicking() {
            self.0.disable_cleanup(true);
            eprintln!(
                "read-private-parent preserved_home={}",
                self.0.path().display()
            );
        }
    }
}
struct Job {
    home: ReadFailureHome,
    path: PathBuf,
    input: w::Input,
    start: w::Start,
    provision: Option<Arc<HelperProvision>>,
    captured_context: w::Context,
}
impl Job {
    fn materialize(mut input: w::Input, files: &BTreeMap<String, Vec<u8>>) -> Self {
        let home = tempfile::Builder::new()
            .prefix("dream-read-")
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in("/private/tmp")
            .unwrap();
        let run = w::Ulid(ulid::Ulid::new().to_string());
        input.nonce = w::Ulid(ulid::Ulid::new().to_string());
        let path = home.path().join(&run.0);
        private_dir(&path);
        private_dir(&path.join("input"));
        private_dir(&path.join("output"));
        for (relative, bytes) in files {
            let relative = Path::new(relative);
            let mut parent = path.clone();
            for component in relative.parent().unwrap().components() {
                parent.push(component);
                if !parent.exists() {
                    private_dir(&parent);
                }
            }
            private_file(&path.join(relative), bytes);
        }
        let bytes = w::canonical(&input).unwrap();
        assert!(bytes.len() <= w::MAX_FRAME);
        private_file(&path.join("input/manifest.json"), &bytes);
        let start = w::Start {
            protocol: w::Protocol::V1,
            run_id: run,
            nonce: input.nonce.clone(),
            input_sha256: w::hash(&bytes),
            timeout_ms: 30000,
            activity_revision: w::Number(1),
            foreground: false,
        };
        Self {
            home: ReadFailureHome(home),
            path,
            captured_context: input.context.clone(),
            input,
            start,
            provision: None,
        }
    }
    fn with_provision(mut self, provision: Arc<HelperProvision>) -> Self {
        self.provision = Some(provision);
        self
    }
    fn rewrite_input(&mut self) {
        let bytes = w::canonical(&self.input).unwrap();
        self.start.input_sha256 = w::hash(&bytes);
        private_file(&self.path.join("input/manifest.json"), &bytes);
    }
    fn rebind_empty_scripted_root(&mut self, root: &str) {
        assert!(
            self.input.records.is_empty()
                && self.input.publications.is_empty()
                && self.input.evidence.roots.is_empty(),
            "signed proof roots require construction before signing"
        );
        self.input.context.root = w::Id(root.into());
        self.captured_context = self.input.context.clone();
        self.rewrite_input();
    }
    fn empty() -> Self {
        let mut input: w::Input =
            w::decode(include_bytes!("fixtures/dream-read/valid/input.json")).unwrap();
        input.closed_sources.clear();
        input.records.clear();
        input.publications.clear();
        input.evidence.roots.clear();
        input.evidence.catalogue.clear();
        input.evidence.public_keys.clear();
        input.evidence.artifacts.clear();
        let bytes=br#"{"schema":"bboxy.dream.trust/2","roots":[],"git_signers":[],"kb_signer_kinds":["seat"],"producers":[]}"#.to_vec();
        input.trust_policy = w::hash(&bytes);
        input.evidence.policy = 0;
        input.evidence.files = vec![w::BundleFile {
            slot: 0,
            path: w::RelativePath("input/evidence/policy.json".into()),
            len: w::Number(bytes.len() as u64),
            sha256: w::hash(&bytes),
        }];
        Self::materialize(
            input,
            &BTreeMap::from([("input/evidence/policy.json".into(), bytes)]),
        )
    }
    fn large(items: usize, padding: usize) -> Self {
        let empty = Self::empty();
        let mut input = empty.input.clone();
        let mut files = BTreeMap::from([(
            "input/evidence/policy.json".to_owned(),
            std::fs::read(empty.path.join("input/evidence/policy.json")).unwrap(),
        )]);
        let session = r::sid(10001);
        let mut log = r::start(session, Path::new("/private-scripted-source"), false);
        for index in 0..items {
            let item = r::item(10002 + index as u128);
            r::begin(&mut log, session, item, ItemKind::AgentMessage);
            r::completed(
                &mut log,
                session,
                item,
                Item::AgentMessage {
                    markdown: "x".repeat(padding),
                },
            );
        }
        r::push(&mut log, session, Event::SessionClosed);
        let bytes = log
            .iter()
            .flat_map(|e| e.wire_bytes().unwrap())
            .collect::<Vec<_>>();
        let last = bytes[..bytes.len() - 1]
            .rsplit(|b| *b == b'\n')
            .next()
            .unwrap();
        input.closed_sources = vec![w::Source {
            session: w::Ulid(session.to_string()),
            source_generation: w::Number(1),
            repo: bboxy450_harness::dream::c1_wire::Repo("fixture.invalid/dream/repo".into()),
            len: w::Number(bytes.len() as u64),
            sha256: w::hash(&bytes),
            closed_seq: w::Number(log.len() as u64),
            closed_line_sha256: w::hash(last),
            blobs: vec![],
        }];
        files.insert(format!("input/{session}.jsonl"), bytes);
        input.evidence.files = files
            .iter()
            .enumerate()
            .map(|(slot, (path, bytes))| w::BundleFile {
                slot: slot as u32,
                path: w::RelativePath(path.clone()),
                len: w::Number(bytes.len() as u64),
                sha256: w::hash(bytes),
            })
            .collect();
        input.evidence.policy = input
            .evidence
            .files
            .iter()
            .find(|f| f.path.0 == "input/evidence/policy.json")
            .unwrap()
            .slot;
        Self::materialize(input, &files)
    }
    fn source_path(&self) -> PathBuf {
        self.path.join(format!(
            "input/{}.jsonl",
            self.input.closed_sources[0].session.0
        ))
    }
    fn replace_source(&mut self, bytes: &[u8]) {
        private_file(&self.source_path(), bytes);
        let source = &mut self.input.closed_sources[0];
        source.len = w::Number(bytes.len() as u64);
        source.sha256 = w::hash(bytes);
        let last = bytes[..bytes.len() - 1]
            .rsplit(|b| *b == b'\n')
            .next()
            .unwrap();
        source.closed_line_sha256 = w::hash(last);
        let path = format!("input/{}.jsonl", source.session.0);
        let file = self
            .input
            .evidence
            .files
            .iter_mut()
            .find(|f| f.path.0 == path)
            .unwrap();
        file.len = source.len;
        file.sha256 = source.sha256.clone();
        self.rewrite_input();
    }
    fn rehash_source(&mut self, session: &w::Ulid, edit: impl FnOnce(&mut Vec<serde_json::Value>)) {
        let path = format!("input/{}.jsonl", session.0);
        let bytes = std::fs::read(self.path.join(&path)).unwrap();
        let raw = bytes.split_inclusive(|b| *b == b'\n').collect::<Vec<_>>();
        let mut values = raw
            .iter()
            .map(|line| serde_json::from_slice::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        let original = values.clone();
        edit(&mut values);
        assert_eq!(original.len(), values.len());
        let mut result = vec![];
        for ((before, after), raw) in original.iter().zip(&values).zip(raw) {
            if before == after {
                result.extend_from_slice(raw);
            } else {
                result.extend(serde_json::to_vec(after).unwrap());
                result.push(b'\n');
            }
        }
        private_file(&self.path.join(&path), &result);
        let source = self
            .input
            .closed_sources
            .iter_mut()
            .find(|s| s.session == *session)
            .unwrap();
        source.len = w::Number(result.len() as u64);
        source.sha256 = w::hash(&result);
        source.closed_line_sha256 = w::hash(
            result[..result.len() - 1]
                .rsplit(|b| *b == b'\n')
                .next()
                .unwrap(),
        );
        let file = self
            .input
            .evidence
            .files
            .iter_mut()
            .find(|f| f.path.0 == path)
            .unwrap();
        file.len = source.len;
        file.sha256 = source.sha256.clone();
        for pin in &mut self.input.publications {
            if pin.session == *session {
                let event = &values[(pin.seq.0 - 1) as usize];
                let bytes = if event == &original[(pin.seq.0 - 1) as usize] {
                    None
                } else {
                    Some(serde_json::to_vec(event).unwrap())
                };
                if let Some(bytes) = bytes {
                    pin.digest = w::Digest(entry::digest(bytes));
                }
            }
        }
        self.rewrite_input();
    }
    fn changed_policy(&mut self, edit: impl FnOnce(&mut serde_json::Value)) {
        let file = &mut self.input.evidence.files[self.input.evidence.policy as usize];
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(self.path.join(&file.path.0)).unwrap()).unwrap();
        edit(&mut value);
        let bytes = w::canonical(&value).unwrap();
        private_file(&self.path.join(&file.path.0), &bytes);
        file.len = w::Number(bytes.len() as u64);
        file.sha256 = w::hash(&bytes);
        self.input.trust_policy = file.sha256.clone();
        self.rewrite_input();
    }
    fn missing_source(self, session: &w::Ulid) -> Self {
        let mut input = self.input.clone();
        input.closed_sources.retain(|s| s.session != *session);
        let mut files = self
            .input
            .evidence
            .files
            .iter()
            .filter(|f| {
                f.path.0 != format!("input/{}.jsonl", session.0)
                    && !f.path.0.starts_with(&format!("input/{}/", session.0))
            })
            .map(|f| {
                (
                    f.path.0.clone(),
                    std::fs::read(self.path.join(&f.path.0)).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let input = rebind_inventory(input, &files);
        let mut next = Self::materialize(input, &files);
        next.provision = self.provision.clone();
        files.clear();
        next
    }
    fn changed_outer(mut self, session: &w::Ulid) -> Self {
        let log = std::fs::read(self.path.join(format!("input/{}.jsonl", session.0))).unwrap();
        let values = log
            .split(|b| *b == b'\n')
            .filter(|b| !b.is_empty())
            .map(|line| serde_json::from_slice::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        let outer = values
            .iter()
            .find(|v| {
                v["event"]["body"]["kind"] == "receipt"
                    && v["event"]["body"]["receipt_kind"] == "run"
            })
            .unwrap();
        let seq = outer["seq"].as_u64().unwrap();
        let old_hash = outer["event"]["body"]["blob"]["sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        let old_path = format!("input/{}/blobs/{old_hash}", session.0);
        let mut body: serde_json::Value =
            serde_json::from_slice(&std::fs::read(self.path.join(&old_path)).unwrap()).unwrap();
        body["seal"]["own_prefix"]["prefix_digest"] = format!("sha256:{}", "0".repeat(64)).into();
        let bytes = w::canonical(&body).unwrap();
        let hash = w::hash(&bytes);
        let new_path = format!("input/{}/blobs/{}", session.0, hash.0);
        private_file(&self.path.join(&new_path), &bytes);
        std::fs::remove_file(self.path.join(&old_path)).unwrap();
        let file = self
            .input
            .evidence
            .files
            .iter_mut()
            .find(|f| f.path.0 == old_path)
            .unwrap();
        file.path = w::RelativePath(new_path);
        file.len = w::Number(bytes.len() as u64);
        file.sha256 = hash.clone();
        let slot = file.slot;
        for a in &mut self.input.evidence.artifacts {
            if a.file == slot {
                a.logical.0 = a.logical.0.strip_suffix(&old_hash).unwrap().to_owned() + &hash.0;
            }
        }
        let blob = self
            .input
            .closed_sources
            .iter_mut()
            .find(|s| s.session == *session)
            .unwrap()
            .blobs
            .iter_mut()
            .find(|b| b.sha256.0 == old_hash)
            .unwrap();
        blob.sha256 = hash.clone();
        blob.len = w::Number(bytes.len() as u64);
        self.input
            .closed_sources
            .iter_mut()
            .find(|s| s.session == *session)
            .unwrap()
            .blobs
            .sort_by(|a, b| a.sha256.cmp(&b.sha256));
        self.rehash_source(session, |events| {
            let blob = &mut events[(seq - 1) as usize]["event"]["body"]["blob"];
            blob["sha256"] = hash.0.into();
            blob["len"] = (bytes.len() as u64).into();
        });
        let files = self
            .input
            .evidence
            .files
            .iter()
            .map(|f| {
                (
                    f.path.0.clone(),
                    std::fs::read(self.path.join(&f.path.0)).unwrap(),
                )
            })
            .collect();
        let mut next = Self::materialize(rebind_inventory(self.input.clone(), &files), &files);
        next.provision = self.provision.clone();
        next
    }
}
#[derive(Debug, PartialEq, Eq)]
enum ReaderExit {
    Eof,
    Stopped,
    Invalid(&'static str),
}
fn read_stdout(
    mut stdout: std::process::ChildStdout,
    sender: mpsc::SyncSender<Vec<u8>>,
    stop: Arc<AtomicBool>,
) -> ReaderExit {
    let fd = stdout.as_raw_fd();
    let mut chunk = [0u8; 4096];
    let mut line = Vec::with_capacity(4096);
    loop {
        if stop.load(Ordering::Acquire) {
            return ReaderExit::Stopped;
        }
        let mut ready = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let polled = unsafe { libc::poll(&mut ready, 1, 5) };
        if polled < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return ReaderExit::Invalid("stdout poll failed");
        }
        if polled == 0 {
            continue;
        }
        let n = match stdout.read(&mut chunk) {
            Ok(0) => {
                return if line.is_empty() {
                    ReaderExit::Eof
                } else {
                    ReaderExit::Invalid("partial frame at EOF")
                };
            }
            Ok(n) => n,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                continue;
            }
            Err(_) => return ReaderExit::Invalid("stdout read failed"),
        };
        for byte in &chunk[..n] {
            if line.len() == w::MAX_FRAME {
                return ReaderExit::Invalid("frame exceeds byte limit");
            }
            line.push(*byte);
            if *byte == b'\n' {
                if sender.send(std::mem::take(&mut line)).is_err() {
                    return ReaderExit::Stopped;
                }
                line = Vec::with_capacity(4096);
            }
        }
    }
}
#[derive(serde::Serialize)]
struct HelloPollFailure {
    stage: &'static str,
    pid: u32,
    run: String,
    original_deadline_ns: u64,
    poll_started_ns: u64,
    post_poll_ns: Option<u64>,
    poll_result: i32,
    poll_revents: i16,
    poll_errno: Option<i32>,
    prior_status_known: bool,
    prior_status_raw: Option<i32>,
    prior_status_errno: Option<i32>,
}
impl HelloPollFailure {
    fn observe(
        child: &mut Child,
        run: &w::Ulid,
        deadline_ns: u64,
        poll_started_ns: u64,
        post_poll_ns: Option<u64>,
        raw_poll: (libc::pollfd, i32, Option<i32>),
    ) -> Self {
        let (poll, result, errno) = raw_poll;
        let prior_status = child.try_wait();
        Self {
            stage: "hello_poll",
            pid: child.id(),
            run: run.0.clone(),
            original_deadline_ns: deadline_ns,
            poll_started_ns,
            post_poll_ns,
            poll_result: result,
            poll_revents: poll.revents,
            poll_errno: errno,
            prior_status_known: prior_status.is_ok(),
            prior_status_raw: prior_status
                .as_ref()
                .ok()
                .and_then(|s| s.map(|s| s.into_raw())),
            prior_status_errno: prior_status.err().and_then(|e| e.raw_os_error()),
        }
    }
}
struct Runner {
    child: Child,
    input: Option<ChildStdin>,
    receiver: mpsc::Receiver<Vec<u8>>,
    reader: Option<thread::JoinHandle<ReaderExit>>,
    reader_stop: Arc<AtomicBool>,
    stderr: PathBuf,
    stderr_pipe: Option<std::process::ChildStderr>,
    stderr_bytes: Vec<u8>,
    reaped: bool,
    reap_status: Option<i32>,
    reap_usage: Option<libc::rusage>,
    hello_failure: Option<HelloPollFailure>,
    lease: Option<File>,
    authority: PrivateAuthority,
    job_deadline: Instant,
    run_id: w::Ulid,
    nonce: w::Ulid,
    probe: bool,
    binding_json: Vec<u8>,
    launch_path: PathBuf,
}
type Stamp = (u64, u64, u64, i64, i64, i64, i64);
fn stamp(path: &Path) -> Stamp {
    let m = std::fs::symlink_metadata(path).unwrap();
    owned_stamp(&m)
}
fn owned_stamp(m: &std::fs::Metadata) -> Stamp {
    assert!(m.is_file() && !m.file_type().is_symlink());
    assert_eq!(m.uid(), unsafe { libc::geteuid() });
    assert_eq!(m.nlink(), 1, "private artifact custody requires one link");
    (
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    )
}
fn observed_parent_cpu() -> Option<u64> {
    let mut t = std::mem::MaybeUninit::<libc::timespec>::uninit();
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, t.as_mut_ptr()) } != 0 {
        return None;
    }
    let t = unsafe { t.assume_init() };
    if t.tv_sec < 0 || !(0..1_000_000_000).contains(&t.tv_nsec) {
        return None;
    }
    (t.tv_sec as u64)
        .checked_mul(1_000_000_000)?
        .checked_add(t.tv_nsec as u64)
}
fn parent_cpu() -> u64 {
    observed_parent_cpu().expect("known nonnegative private parent CPU clock")
}
fn parent_rest(cpu: u64, end: Instant) {
    if let Some(left) = Duration::from_nanos(9 * cpu.max(1_000_000)).checked_sub(end.elapsed()) {
        thread::sleep(left);
    }
}
#[track_caller]
fn parent_paid(at: u64) {
    parent_paid_with_rest(at, parent_rest);
}
#[track_caller]
fn parent_paid_with_rest(at: u64, rest: impl FnOnce(u64, Instant)) {
    let used = parent_cpu().checked_sub(at).unwrap();
    let end = Instant::now();
    // A failed budget verdict still owes this entire measured unit. The
    // callback lets the causal control observe the same real payment path.
    rest(used, end);
    assert!(
        used <= 1_000_000,
        "ordinary private parent unit exceeds1ms: {used}"
    );
}
#[test]
fn ordinary_parent_overrun_pays_fresh_measured_debt_before_refusal() {
    use std::cell::Cell;
    let old_idle = Instant::now();
    thread::sleep(Duration::from_millis(25));
    let at = parent_cpu();
    while parent_cpu().checked_sub(at).unwrap() < 2_000_000 {
        std::hint::spin_loop();
    }
    let call_started = Instant::now();
    let paid = Cell::new(false);
    let used_ns = Cell::new(0);
    let rest_ns = Cell::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parent_paid_with_rest(at, |used, end| {
            assert!(used >= 2_000_000, "control must consume real thread CPU");
            assert!(end >= call_started, "old idle became a payment endpoint");
            used_ns.set(used);
            parent_rest(used, end);
            let observed = end.elapsed();
            assert!(observed >= Duration::from_nanos(9 * used));
            rest_ns.set(observed.as_nanos());
            paid.set(true);
        });
    }));
    let refusal = result.expect_err("over1ms unit must still refuse after payment");
    let message = refusal
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| refusal.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        message.contains("ordinary private parent unit exceeds1ms"),
        "unexpected refusal: {message}"
    );
    assert!(paid.get(), "budget assertion bypassed its measured debt");
    eprintln!(
        "ordinary-parent-failure-debt real_cpu_ns={} owed_ns={} fresh_rest_ns={} prior_idle_and_control_ns={} paid_before_refusal=true fixed_limit_ns=1000000",
        used_ns.get(),
        9 * used_ns.get(),
        rest_ns.get(),
        old_idle.elapsed().as_nanos()
    );
}
fn bounded_parent_text(path: &Path, limit: usize) -> String {
    let at = parent_cpu();
    let before = stamp(path);
    assert!(before.2 <= limit as u64, "receipt exceeds byte limit");
    let mut file = File::options()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .unwrap();
    assert_eq!(owned_stamp(&file.metadata().unwrap()), before);
    let mut bytes = Vec::with_capacity(limit);
    let mut chunk = [0u8; 4096];
    parent_paid(at);
    loop {
        let at = parent_cpu();
        let n = file.read(&mut chunk).unwrap();
        assert!(n <= limit - bytes.len(), "receipt grew past byte limit");
        bytes.extend_from_slice(&chunk[..n]);
        parent_paid(at);
        if n == 0 {
            break;
        }
    }
    let at = parent_cpu();
    assert_eq!(owned_stamp(&file.metadata().unwrap()), before);
    assert_eq!(stamp(path), before);
    drop(file);
    let text = String::from_utf8(bytes).unwrap();
    parent_paid(at);
    text
}
struct ApprovedRunner {
    path: PathBuf,
    hash: w::Hex,
    guards: Vec<(PathBuf, Stamp)>,
}
fn bounded_parent_hash(path: &Path) -> w::Hex {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path).unwrap();
    let mut digest = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let at = parent_cpu();
        let n = file.read(&mut buf).unwrap();
        digest.update(&buf[..n]);
        parent_paid(at);
        if n == 0 {
            break;
        }
    }
    w::Hex(format!("{:x}", digest.finalize()))
}
fn approved_runner() -> &'static ApprovedRunner {
    static APPROVED: std::sync::OnceLock<ApprovedRunner> = std::sync::OnceLock::new();
    let approved = APPROVED.get_or_init(|| {
        // Adapted from https://github.com/bboxy450/bboxy450-harness/blob/main/tests/dream_c1.rs.
        // Credit policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
        // This is an
        // explicit ordinary artifact receipt, never an executable probe or fallback.
        let path = PathBuf::from(
            std::env::var_os("DREAM_READ_RUNNER")
                .expect("explicit preapproved ordinary read worker required"),
        );
        assert!(path.is_absolute());
        assert_eq!(std::fs::canonicalize(&path).unwrap(), path);
        let folder = path.parent().unwrap();
        assert_eq!(
            bounded_parent_text(&folder.join("bootstrap-support.txt"), 18),
            "dream-bootstrap/1\n"
        );
        let source = bounded_parent_text(&folder.join("source-commit.txt"), 16 * 1024);
        assert!(source.trim().len() == 40 && source.trim().bytes().all(|b| b.is_ascii_hexdigit()));
        let compiler = bounded_parent_text(&folder.join("compiler.txt"), 16 * 1024);
        assert!(
            compiler.len() <= 16 * 1024
                && compiler.starts_with("rustc ")
                && compiler.contains("host: ")
        );
        let receipts = bounded_parent_text(&folder.join("hashes.txt"), 16 * 1024);
        assert!(receipts.len() <= 16 * 1024);
        let declared_worker_hash = receipts
            .lines()
            .find_map(|line| {
                let mut parts = line.split_whitespace();
                let hash = parts.next()?;
                let name = parts.next()?;
                (!name.starts_with("src/")
                    && !name.starts_with("tests/")
                    && !matches!(name, "Makefile" | "Cargo.toml" | "Cargo.lock"))
                .then_some(hash)
            })
            .unwrap();
        assert!(
            declared_worker_hash.len() == 64
                && declared_worker_hash.bytes().all(|b| b.is_ascii_hexdigit())
        );
        let frozen_source_id = bounded_parent_text(&folder.join("source-snapshot.txt"), 65);
        assert!(frozen_source_id.len() == 65 && frozen_source_id.ends_with('\n'));
        let frozen_source_id = frozen_source_id.trim_end();
        assert!(frozen_source_id.bytes().all(|b| b.is_ascii_hexdigit()));
        let frozen_sources = folder.join(format!("source-{frozen_source_id}"));
        let frozen_metadata = std::fs::symlink_metadata(&frozen_sources).unwrap();
        assert!(frozen_metadata.is_dir() && !frozen_metadata.file_type().is_symlink());
        let allowed = [
            "Makefile",
            "src/main.rs",
            "src/frontends/headless.rs",
            "src/dream/mod.rs",
            "src/dream/read.rs",
            "src/dream/read_wire.rs",
            "src/dream/c1_wire.rs",
            "src/dream/fingerprint.rs",
            "src/dream/entry.rs",
            "src/dream/worker.rs",
            "src/dream/c1.rs",
            "src/dream/bootstrap.rs",
            "src/gateway/mod.rs",
            "src/gateway/unix.rs",
            "src/engine/mod.rs",
            "tests/fixtures/dream-read/closure-codec-controls.c",
            "tests/fixtures/dream-read/check-closure-vectors.py",
            "tests/fixtures/dream-read/prepare-c-probe.py",
            "tests/fixtures/dream-proof-tools/closure-boundary.schema.json",
            "tests/fixtures/dream-proof-tools/closure-boundary-vectors.json",
            "src/dream/resolve.rs",
            "src/dream/trust.rs",
            "src/dream/audit.rs",
            "src/dream/graph.rs",
            "src/dream/recall.rs",
            "src/dream/receipt.rs",
            "src/dream/projection.rs",
            "src/dream/seal.rs",
            "tests/fixtures/dream-bootstrap/vectors.json",
            "tests/fixtures/dream-read/proof-helper-probe.c",
            "tests/fixtures/dream-read/signature-boundary-controls.c",
            "tests/fixtures/dream-read/private-input-controls.c",
            "tests/fixtures/dream-read/private-engine-controls.c",
            "tests/fixtures/dream-read/check-private-engine-controls.py",
            "tests/fixtures/dream-read/io-helper.c",
            "tests/fixtures/dream-read/root_fixture.rs",
            "tests/dream_read.rs",
            "Cargo.toml",
            "Cargo.lock",
        ];
        let mut seen = std::collections::BTreeSet::new();
        let mut guards = Vec::new();
        let mut worker_hash = None;
        let mut source_rows = Vec::new();
        for line in receipts.lines() {
            let mut parts = line.split_whitespace();
            let expected = parts.next().unwrap();
            let name = parts.next().unwrap();
            assert!(
                parts.next().is_none()
                    && expected.len() == 64
                    && expected.bytes().all(|b| b.is_ascii_hexdigit())
            );
            assert!(seen.insert(name.to_owned()), "duplicate artifact receipt");
            let target = if allowed.contains(&name) {
                source_rows.extend_from_slice(line.as_bytes());
                source_rows.push(b'\n');
                // Immutable copies were hashed before/after the ordinary build.
                // A later source edit cannot silently repin this exact artifact
                // or force unrelated private component work to stop executing.
                frozen_sources.join(name)
            } else {
                let candidate = PathBuf::from(name);
                let candidate = if candidate.is_absolute() {
                    candidate
                } else {
                    Path::new(env!("CARGO_MANIFEST_DIR")).join(candidate)
                };
                assert_eq!(
                    std::fs::canonicalize(candidate).unwrap(),
                    path,
                    "unlisted source in provision receipt"
                );
                path.clone()
            };
            let before = stamp(&target);
            let actual = bounded_parent_hash(&target);
            assert_eq!(actual.0, expected, "preapproved bytes changed: {name}");
            if name == "tests/dream_read.rs" {
                assert_eq!(
                    actual,
                    w::hash(include_bytes!("dream_read.rs")),
                    "parent test source binding"
                );
            }
            assert_eq!(stamp(&target), before);
            if target == path {
                assert!(worker_hash.replace(actual).is_none());
            }
            guards.push((target, before));
        }
        for name in allowed {
            assert!(seen.contains(name), "missing source receipt: {name}");
        }
        assert_eq!(
            w::hash(&source_rows).0,
            frozen_source_id,
            "closed source snapshot digest"
        );
        assert_eq!(worker_hash.as_ref().unwrap().0, declared_worker_hash);
        for name in [
            "bootstrap-support.txt",
            "source-commit.txt",
            "compiler.txt",
            "hashes.txt",
            "source-snapshot.txt",
        ] {
            let p = folder.join(name);
            guards.push((p.clone(), stamp(&p)));
        }
        ApprovedRunner {
            path,
            hash: worker_hash.unwrap(),
            guards,
        }
    });
    for (path, expected) in &approved.guards {
        let at = parent_cpu();
        let checked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_eq!(
                &stamp(path),
                expected,
                "approved build changed before birth"
            );
        }));
        parent_paid(at);
        if let Err(error) = checked {
            std::panic::resume_unwind(error);
        }
    }
    approved
}
struct PrivateAuthority {
    manifest: Option<(PathBuf, Stamp)>,
    worker: PathBuf,
    worker_stamp: Stamp,
    context: Option<w::Context>,
}
impl PrivateAuthority {
    fn fresh(&self) {
        // Two private immutable rows. This scripted fixture is not the term
        // public/provider/root fence and never certifies that Q3 integration.
        if let Some((path, expected)) = &self.manifest {
            assert_eq!(&stamp(path), expected);
        }
        assert_eq!(stamp(&self.worker), self.worker_stamp);
        if let Some(context) = &self.context {
            assert!(context.generation.0 > 0 && context.grant_epoch.0 > 0);
        }
    }
}
fn bootstrap_pipe() -> (File, File) {
    let mut p = [-1; 2];
    assert_eq!(unsafe { libc::pipe(p.as_mut_ptr()) }, 0);
    for fd in p {
        assert!((3..=65535).contains(&fd));
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) },
            0
        );
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
            0
        );
    }
    unsafe { (File::from_raw_fd(p[0]), File::from_raw_fd(p[1])) }
}
// Adapted from the canonical C1 entry observer. Keep the raw s/g/u/k/r/p
// counters opaque: only the e/m monotonic chain shares the parent's clock.
fn bootstrap_waiting_marker(trace: &[u8], observed_ns: u64) -> Result<Option<&str>, &'static str> {
    let Some(first_end) = trace.iter().position(|b| *b == b'\n') else {
        return if trace.len() > 256 {
            Err("oversized entry")
        } else {
            Ok(None)
        };
    };
    if first_end + 1 > 256 {
        return Err("oversized entry");
    }
    let entry = std::str::from_utf8(&trace[..first_end]).map_err(|_| "entry UTF-8")?;
    let mut fields = entry.split(' ');
    if fields.next() != Some("dream-bootstrap") || fields.next() != Some("entry") {
        return Err("entry must precede waiting and permit");
    }
    let mut values = [0; 8];
    for (i, key) in ["s=", "g=", "e=", "m=", "u=", "k=", "r=", "p=", "ok="]
        .iter()
        .enumerate()
    {
        let number = fields
            .next()
            .and_then(|field| field.strip_prefix(key))
            .ok_or("entry key order")?;
        if number.is_empty()
            || number.len() > 20
            || (number.len() > 1 && number.starts_with('0'))
            || !number.bytes().all(|b| b.is_ascii_digit())
        {
            return Err("entry numeric field");
        }
        let value = number.parse::<u64>().map_err(|_| "entry u64 overflow")?;
        if i < values.len() {
            values[i] = value;
        } else if value > 1 || (value == 0 && [0, 4, 5, 6, 7].iter().any(|i| values[*i] != 0)) {
            return Err("entry rusage status");
        }
    }
    if fields.next().is_some() || values[2] > values[3] || values[3] > observed_ns {
        return Err("entry fields or monotonic clock chain");
    }
    let waiting = &trace[first_end + 1..];
    let Some(waiting_end) = waiting.iter().position(|b| *b == b'\n') else {
        return if waiting.len() > 256 {
            Err("oversized waiting")
        } else {
            Ok(None)
        };
    };
    if waiting_end + 1 > 256 || waiting_end + 1 != waiting.len() {
        return Err("exactly one entry then waiting required");
    }
    let waiting = std::str::from_utf8(waiting).map_err(|_| "waiting UTF-8")?;
    if !waiting.starts_with("dream-bootstrap state=waiting ") {
        return Err("entry must be followed by waiting");
    }
    Ok(Some(waiting))
}
impl Runner {
    fn start(job: &Job) -> Self {
        Self::launch(job, false, |_, _| {})
    }
    fn launch(job: &Job, probe: bool, held: impl FnOnce(&mut Self, u64)) -> Self {
        let approved = approved_runner();
        let job_deadline = Instant::now() + Duration::from_secs(30);
        // Fixed pre-birth units: files/buffers, lease/IDs, binding, custody/
        // channel, then command. No CPU sample spans a previously paid unit.
        let prepare_at = parent_cpu();
        let job_deadline_ns = boot::monotonic_ns()
            .unwrap()
            .checked_add(30_000_000_000)
            .unwrap();
        let stderr = job.path.join("worker.stderr");
        private_file(&stderr, b"");
        let stderr_bytes = Vec::with_capacity(64 * 1024);
        let reader_stop = Arc::new(AtomicBool::new(false));
        let expected_schema = w::schema_hash();
        parent_paid(prepare_at);
        let prepare_at = parent_cpu();
        let (read, write) = bootstrap_pipe();
        let fd = read.as_raw_fd();
        let birth = ulid::Ulid::new();
        let op = w::Ulid(ulid::Ulid::new().to_string());
        let run_id = if probe {
            w::Ulid(ulid::Ulid::new().to_string())
        } else {
            job.start.run_id.clone()
        };
        let nonce = if probe {
            w::Ulid(ulid::Ulid::new().to_string())
        } else {
            job.start.nonce.clone()
        };
        let launch_path = if probe {
            job.home.path().join(&run_id.0)
        } else {
            job.path.clone()
        };
        static INCARNATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let incarnation = INCARNATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        parent_paid(prepare_at);
        let prepare_at = parent_cpu();
        let binding = boot::Binding::new(
            boot::Purpose::ReadPrepare,
            probe,
            &op,
            &run_id,
            &nonce,
            incarnation,
            (!probe).then_some(&job.captured_context),
            &approved.hash,
        )
        .unwrap();
        let binding_json = binding.bytes().unwrap();
        let binding_hash = w::hash(&binding_json);
        let environment = format!("1:{fd}:{birth}:{}", binding_hash.0);
        let token = boot::token(environment.as_bytes()).unwrap();
        parent_paid(prepare_at);
        let prepare_at = parent_cpu();
        let authority = PrivateAuthority {
            manifest: (!probe).then(|| {
                let path = job.path.join("input/manifest.json");
                let identity = stamp(&path);
                (path, identity)
            }),
            worker: approved.path.clone(),
            worker_stamp: stamp(&approved.path),
            context: (!probe).then(|| job.captured_context.clone()),
        };
        let (sender, receiver) = mpsc::sync_channel(64);
        parent_paid(prepare_at);
        let prepare_at = parent_cpu();
        let mut command = Command::new(&approved.path);
        command
            .args(["dream", "prepare-read", "--job-dir"])
            .arg(&launch_path)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/opt/homebrew/bin")
            .env("HOME", job.home.path())
            .env("TMPDIR", job.home.path())
            .env("LANG", "C.UTF-8")
            .env(boot::ENV, environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(provision) = &job.provision {
            command
                .env("BBOXY_DREAM_HELPER_PROVISION", &provision.receipt)
                .env("BBOXY_DREAM_HELPER_PROVISION_SHA256", &provision.digest);
        }
        unsafe {
            command.pre_exec(move || {
                if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
        parent_paid(prepare_at);
        let birth_at = parent_cpu();
        let native_at = parent_cpu();
        let wall_at = Instant::now();
        let child = command.spawn().unwrap();
        // Observe, but defer every post-effect verdict until Runner owns the
        // actual Child and lease. An unavailable clock cannot orphan Child.
        let native_end = observed_parent_cpu();
        let wall = wall_at.elapsed();
        let spawn_return = boot::monotonic_ns();
        // Returned Child and writer lease are owned before any cost/readiness
        // verdict can reject a native effect. Drop always performs real reap.
        let mut this = Self {
            child,
            input: None,
            receiver,
            reader: None,
            reader_stop,
            stderr,
            stderr_pipe: None,
            stderr_bytes,
            reaped: false,
            reap_status: None,
            reap_usage: None,
            hello_failure: None,
            lease: Some(write),
            authority,
            job_deadline,
            run_id,
            nonce,
            probe,
            binding_json,
            launch_path,
        };
        drop(read);
        this.stderr_pipe = this.child.stderr.take();
        let stderr_fd = this
            .stderr_pipe
            .as_ref()
            .map(AsRawFd::as_raw_fd)
            .unwrap_or(-1);
        let stderr_flags = unsafe { libc::fcntl(stderr_fd, libc::F_GETFL) };
        let nonblocking = if stderr_flags >= 0 {
            unsafe { libc::fcntl(stderr_fd, libc::F_SETFL, stderr_flags | libc::O_NONBLOCK) }
        } else {
            -1
        };
        let birth_end = observed_parent_cpu();
        let birth_cost = native_end.zip(birth_end).and_then(|(call_end, end)| {
            let native = call_end.checked_sub(native_at)?;
            let app = native_at
                .checked_sub(birth_at)?
                .checked_add(end.checked_sub(call_end)?)?;
            Some((native, app, end.checked_sub(birth_at)?))
        });
        // Close and pay the FULL creation/custody/setup span before marker
        // work begins. Its fresh debt cannot overlap marker work or marker
        // debt. Unknown clocks retain raw unknown evidence and pay mandatory
        // 3ms-span settlement debt before refusal, never an inferred cost.
        parent_rest(
            birth_cost
                .map(|(_, _, inclusive)| inclusive)
                .unwrap_or(3_000_000),
            Instant::now(),
        );
        let marker_setup_at = parent_cpu();
        let marker_setup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (native_cpu, app_cpu, inclusive_cpu) =
                birth_cost.expect("known nonreversed birth clocks");
            assert!(
                native_cpu <= 2_000_000
                    && app_cpu <= 1_000_000
                    && inclusive_cpu <= 3_000_000
                    && wall <= Duration::from_millis(25),
                "private parent birth native_cpu={native_cpu} app_cpu={app_cpu} inclusive_cpu={inclusive_cpu} native_wall={wall:?}"
            );
            assert!(stderr_flags >= 0 && nonblocking == 0);
            spawn_return
                .unwrap()
                .checked_add(boot::WINDOW_NS)
                .unwrap()
                .min(job_deadline_ns)
        }));
        parent_paid(marker_setup_at);
        let readiness_ns = match marker_setup {
            Ok(deadline) => deadline,
            Err(error) => std::panic::resume_unwind(error),
        };
        let marker = this.wait_marker(readiness_ns);
        let held_at = parent_cpu();
        let held_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_eq!(
                marker.trim(),
                "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0"
            );
            held(&mut this, readiness_ns);
            drop(marker);
        }));
        parent_paid(held_at);
        if let Err(error) = held_result {
            std::panic::resume_unwind(error);
        }
        let setup_at = parent_cpu();
        drop(command);
        this.input = this.child.stdin.take();
        let permit = boot::Permit {
            purpose: boot::Purpose::ReadPrepare,
            probe,
            birth: token.birth,
            run_id: boot::ulid(this.run_id.0.as_bytes()).unwrap(),
            nonce: boot::ulid(this.nonce.0.as_bytes()).unwrap(),
            binding: token.binding,
            incarnation,
            sequence: 1,
            deadline_ns: readiness_ns,
        };
        boot::bound(
            &permit,
            &token,
            boot::Purpose::ReadPrepare,
            this.launch_path.file_name().unwrap().as_encoded_bytes(),
        )
        .unwrap();
        let raw = boot::encode(&permit);
        parent_paid(setup_at);
        // One prepared fresh private-authority callback plus one tiny write.
        // No hashing, parser, inference, retry or recursive debt at this fence.
        let grant_at = parent_cpu();
        this.authority.fresh();
        assert!(Instant::now() < this.job_deadline && boot::monotonic_ns().unwrap() < readiness_ns);
        let wrote = unsafe {
            libc::write(
                this.lease.as_ref().unwrap().as_raw_fd(),
                raw.as_ptr().cast(),
                raw.len(),
            )
        };
        assert_eq!(
            wrote,
            boot::LENGTH as isize,
            "single permit transfer failed; no retry"
        );
        assert!(parent_cpu().checked_sub(grant_at).unwrap() <= 1_000_000);
        let hello_at = parent_cpu();
        let hello = this.read_hello(readiness_ns);
        this.drain_stderr();
        assert_eq!(hello.protocol, w::Protocol::V1);
        assert_eq!(hello.algorithm, w::Algorithm::ReadPrepare);
        assert_eq!(hello.schema_sha256, expected_schema);
        assert_eq!(hello.ir_major, 1);
        assert_eq!(hello.ir_minor_max, 3);
        assert!(
            boot::monotonic_ns().unwrap() < readiness_ns,
            "paid Hello exceeded original readiness"
        );
        assert!(
            std::str::from_utf8(&this.stderr_bytes)
                .unwrap()
                .contains("dream-bootstrap state=paid_hello profile=dream-bootstrap/1")
        );
        drop(this.lease.take());
        parent_paid(hello_at);
        let reader_at = parent_cpu();
        let reader_wall = Instant::now();
        let stdout = this.child.stdout.take().unwrap();
        let stdout_flags = unsafe { libc::fcntl(stdout.as_raw_fd(), libc::F_GETFL) };
        assert!(stdout_flags >= 0);
        assert_eq!(
            unsafe {
                libc::fcntl(
                    stdout.as_raw_fd(),
                    libc::F_SETFL,
                    stdout_flags | libc::O_NONBLOCK,
                )
            },
            0
        );
        let stop = Arc::clone(&this.reader_stop);
        let reader = thread::spawn(move || read_stdout(stdout, sender, stop));
        let reader_wall = reader_wall.elapsed();
        this.reader = Some(reader);
        let reader_cpu = parent_cpu().checked_sub(reader_at).unwrap();
        // This fixture IO owner is ordinary parent work. It has no compute-
        // owner/native exception and stays inside the existing parent budget.
        assert!(reader_cpu <= 1_000_000 && reader_wall <= Duration::from_millis(25));
        parent_rest(reader_cpu, Instant::now());
        this
    }
    fn wait_marker(&mut self, readiness_ns: u64) -> String {
        loop {
            let at = parent_cpu();
            let observation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.drain_stderr();
                let observed_ns = boot::monotonic_ns().unwrap();
                assert!(
                    observed_ns < readiness_ns,
                    "original readiness expired before wait marker; stderr={}",
                    String::from_utf8_lossy(&self.stderr_bytes)
                );
                let marker = match bootstrap_waiting_marker(&self.stderr_bytes, observed_ns) {
                    Ok(marker) => marker.map(str::to_owned),
                    Err(error) => panic!(
                        "bootstrap entry/waiting refused {error}; stderr={}",
                        String::from_utf8_lossy(&self.stderr_bytes)
                    ),
                };
                if marker.is_none() {
                    let status = self.child.try_wait().unwrap();
                    assert!(
                        status.is_none(),
                        "child refused before wait marker; status={status:?} stderr={}",
                        String::from_utf8_lossy(&self.stderr_bytes)
                    );
                }
                marker
            }));
            // Pay exactly this bounded observation, including a refusal, before
            // another read/parse or returning the marker. The original absolute
            // readiness deadline is never renewed by a payment or observation.
            parent_paid(at);
            match observation {
                Ok(Some(marker)) => return marker,
                Ok(None) => {}
                Err(error) => std::panic::resume_unwind(error),
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    fn drain_stderr(&mut self) -> bool {
        let Some(pipe) = self.stderr_pipe.as_mut() else {
            return true;
        };
        let mut chunk = [0u8; 4096];
        match pipe.read(&mut chunk) {
            Ok(0) => {
                self.stderr_pipe.take();
                true
            }
            Ok(n) => {
                assert!(
                    self.stderr_bytes.len() + n <= 64 * 1024,
                    "bounded private diagnostics"
                );
                self.stderr_bytes.extend_from_slice(&chunk[..n]);
                false
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                false
            }
            Err(error) => panic!("private diagnostic pipe: {error}"),
        }
    }
    fn save_stderr(&self) {
        std::fs::write(&self.stderr, &self.stderr_bytes).unwrap();
    }
    fn finish_stderr(&mut self) {
        let until = Instant::now() + Duration::from_secs(2);
        while !self.drain_stderr() {
            assert!(Instant::now() < until, "diagnostic EOF after actual reap");
            thread::sleep(Duration::from_millis(1));
        }
        self.save_stderr();
    }
    fn stop_reader(&mut self) {
        self.reader_stop.store(true, Ordering::Release);
        // A stop flag interrupts nonblocking IO; receiver disposal separately
        // wakes an existing bounded send. Neither path abandons its owner.
        let (_sender, empty) = mpsc::channel();
        drop(std::mem::replace(&mut self.receiver, empty));
    }
    fn join_reader_eof(&mut self) {
        let until = Instant::now() + Duration::from_secs(2);
        while !self.reader.as_ref().unwrap().is_finished() && Instant::now() < until {
            self.drain_stderr();
            thread::sleep(Duration::from_millis(1));
        }
        let timely = self.reader.as_ref().unwrap().is_finished() && Instant::now() <= until;
        if !timely {
            self.stop_reader();
        }
        let exit = self.reader.take().unwrap().join().unwrap();
        eprintln!(
            "read-private-parent reader_pid={} actual_join=true outcome={exit:?} eof_within2s={timely}",
            self.child.id()
        );
        assert!(timely, "stdout EOF exceeded existing two-second deadline");
        assert_eq!(exit, ReaderExit::Eof, "reader must actually observe EOF");
    }
    fn read_hello(&mut self, readiness_ns: u64) -> w::Hello {
        let fd = self.child.stdout.as_ref().unwrap().as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
            0
        );
        let mut bytes = [0u8; w::MAX_FRAME];
        let mut filled = 0;
        loop {
            let poll_started_ns = boot::monotonic_ns().unwrap();
            let remaining = readiness_ns
                .checked_sub(poll_started_ns)
                .expect("Hello readiness not reset");
            let mut ready = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let polled = unsafe { libc::poll(&mut ready, 1, remaining.div_ceil(1_000_000) as i32) };
            let rawerrno = if polled < 0 {
                std::io::Error::last_os_error().raw_os_error()
            } else {
                None
            };
            if polled <= 0 {
                // Capture the actual post-poll clock before diagnostics or
                // status observation; the pre-poll remaining time is not it.
                let post_poll_ns = boot::monotonic_ns().ok();
                let failure = HelloPollFailure::observe(
                    &mut self.child,
                    &self.run_id,
                    readiness_ns,
                    poll_started_ns,
                    post_poll_ns,
                    (ready, polled, rawerrno),
                );
                let prior_status_raw = failure.prior_status_raw;
                self.hello_failure = Some(failure);
                self.drain_stderr();
                panic!(
                    "Hello readiness exceeded: stage=hello_poll pid={} run={} poll={polled} revents={} errno={rawerrno:?} prior_status_raw={prior_status_raw:?} original_deadline_ns={readiness_ns} poll_started_ns={poll_started_ns} post_poll_ns={post_poll_ns:?} original_remaining_ns={remaining} stderr={}",
                    self.child.id(),
                    self.run_id.0,
                    ready.revents,
                    String::from_utf8_lossy(&self.stderr_bytes)
                );
            }
            let n = unsafe {
                libc::read(
                    fd,
                    bytes[filled..].as_mut_ptr().cast(),
                    bytes.len() - filled,
                )
            };
            assert!(n > 0, "Hello EOF/error is a refusal");
            filled += n as usize;
            if bytes[..filled].ends_with(b"\n") {
                break;
            }
            assert!(filled < bytes.len());
        }
        assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFL, flags) }, 0);
        let w::WorkerOut::Hello(hello) = w::decode(&bytes[..filled]).unwrap() else {
            panic!("missing read Hello");
        };
        hello
    }
    fn send(&mut self, msg: &impl serde::Serialize) {
        let at = parent_cpu();
        let mut value = serde_json::to_value(msg).unwrap();
        let start = value["type"] == "start";
        if start {
            if let Some(timeout) = value["timeout_ms"]
                .as_u64()
                .filter(|v| (1..=30000).contains(v))
            {
                let remaining = self
                    .job_deadline
                    .checked_duration_since(Instant::now())
                    .expect("original job deadline expired before Start")
                    .as_millis();
                assert!(remaining > 0);
                value["timeout_ms"] = serde_json::json!(timeout.min(remaining as u64));
            }
        }
        let mut bytes = w::canonical(&value).unwrap();
        bytes.push(b'\n');
        assert!(bytes.len() <= w::MAX_FRAME);
        if start {
            parent_paid(at);
        }
        let input = self.input.as_mut().unwrap();
        if start {
            let control_at = parent_cpu();
            self.authority.fresh();
            assert!(Instant::now() < self.job_deadline);
            let wrote =
                unsafe { libc::write(input.as_raw_fd(), bytes.as_ptr().cast(), bytes.len()) };
            assert_eq!(
                wrote,
                bytes.len() as isize,
                "single prepared Start transfer failed; no retry"
            );
            assert!(
                parent_cpu().checked_sub(control_at).unwrap() <= 1_000_000,
                "whole fresh Start callback/write exceeds1ms"
            );
        } else {
            input.write_all(&bytes).unwrap();
            input.flush().unwrap();
        }
    }
    fn next(&mut self, timeout: Duration) -> w::WorkerOut {
        let until = Instant::now() + timeout;
        let line = loop {
            self.drain_stderr();
            match self.receiver.recv_timeout(Duration::from_millis(1)) {
                Ok(line) => break line,
                Err(mpsc::RecvTimeoutError::Timeout) if Instant::now() < until => {}
                Err(error) => {
                    self.save_stderr();
                    panic!(
                        "worker frame {error}; stderr: {}",
                        String::from_utf8_lossy(&self.stderr_bytes)
                    );
                }
            }
        };
        w::decode(&line).unwrap()
    }
    fn finish(mut self, job: &Job) -> (w::Done, i32, String) {
        let mut seq = 0;
        let done = loop {
            match self.next(Duration::from_secs(40)) {
                w::WorkerOut::Progress(p) => {
                    assert_eq!(p.run_id, job.start.run_id);
                    assert_eq!(p.nonce, job.start.nonce);
                    assert!(p.seq.0 > seq);
                    seq = p.seq.0;
                }
                w::WorkerOut::Done(d) => {
                    assert!(d.seq.0 > seq);
                    break d;
                }
                _ => panic!("duplicate Hello"),
            }
        };
        assert_eq!(done.run_id, job.start.run_id);
        assert_eq!(done.nonce, job.start.nonce);
        let code = self.wait_exit();
        self.input.take();
        self.join_reader_eof();
        self.finish_stderr();
        assert!(self.receiver.try_recv().is_err(), "frame after Done");
        let stderr = std::fs::read_to_string(&self.stderr).unwrap();
        self.verify_post_reap_usage(&stderr);
        (done, code, stderr)
    }
    fn finish_probe_refusal(mut self) -> String {
        assert!(self.probe);
        drop(self.input.take());
        assert_eq!(
            self.wait_exit(),
            2,
            "matching probe Start must permanently refuse"
        );
        match self.receiver.recv_timeout(Duration::from_secs(2)) {
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
            other => {
                panic!("probe post-Hello lane did not close empty after actual exit: {other:?}")
            }
        }
        self.join_reader_eof();
        self.finish_stderr();
        assert!(
            self.receiver.try_recv().is_err(),
            "probe produced a post-Hello protocol frame"
        );
        let metrics = std::fs::read_to_string(&self.stderr).unwrap();
        self.verify_post_reap_usage(&metrics);
        assert!(metrics.len() <= 16 * 1024);
        assert!(metrics.contains("dream-bootstrap state=paid_hello profile=dream-bootstrap/1"));
        assert!(
            metrics.contains("cutoff=joined_final_control_pre_telemetry"),
            "missing post-join terminal receipt: {metrics}"
        );
        let native = metrics
            .lines()
            .find(|line| line.starts_with("dream-c1 native tag=compute_owner_create "))
            .expect("actual parked owner receipt");
        assert_eq!(metric(native, "count"), 1);
        assert_eq!(metric(native, "effects"), 1);
        assert!(
            !metrics.contains("dream-read metrics ")
                && !metrics.contains("dream-read helper_native "),
            "probe entered source/proof compute: {metrics}"
        );
        assert!(
            !self.launch_path.exists(),
            "source-free probe created a job/input path"
        );
        metrics
    }
    fn wait_exit(&mut self) -> i32 {
        let until = Instant::now() + Duration::from_secs(4);
        loop {
            let mut status = 0;
            let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
            let pid = unsafe {
                libc::wait4(
                    self.child.id() as i32,
                    &mut status,
                    libc::WNOHANG,
                    usage.as_mut_ptr(),
                )
            };
            if pid > 0 {
                assert_eq!(pid as u32, self.child.id());
                self.reaped = true;
                self.reap_status = Some(status);
                self.reap_usage = Some(unsafe { usage.assume_init() });
                return if libc::WIFEXITED(status) {
                    libc::WEXITSTATUS(status)
                } else {
                    -1
                };
            }
            if pid < 0 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::EINTR)
                );
            }
            assert!(
                Instant::now() < until,
                "worker did not actually exit after terminal frame"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn verify_post_reap_usage(&self, metrics: &str) {
        let usage = self
            .reap_usage
            .as_ref()
            .expect("actual wait4 usage required");
        let ns = |t: libc::timeval| {
            assert!((0..1_000_000).contains(&t.tv_usec));
            u64::try_from(t.tv_sec)
                .unwrap()
                .checked_mul(1_000_000_000)
                .unwrap()
                .checked_add(u64::try_from(t.tv_usec).unwrap() * 1000)
                .unwrap()
        };
        let user = ns(usage.ru_utime);
        let system = ns(usage.ru_stime);
        let post_upper = user.checked_add(system).unwrap().checked_add(1998).unwrap();
        let captured_process = metric(metrics, "process_user_cpu_ns")
            .checked_add(metric(metrics, "process_system_cpu_ns"))
            .unwrap();
        assert!(
            captured_process <= post_upper,
            "terminal cutoff exceeds actual post-reap usage"
        );
        eprintln!(
            "read-private-parent post_reap_pid={} user_cpu_ns={user} system_cpu_ns={system} rss_high_water_bytes={} coverage=worker_and_possible_waited_helpers cpu_totals_overlap=true",
            self.child.id(),
            usage.ru_maxrss
        );
    }
}
#[derive(Debug, serde::Serialize)]
struct SettlementSignal {
    result: i32,
    errno: Option<i32>,
}
#[derive(Debug, serde::Serialize)]
struct RunnerSettlement {
    pid: u32,
    phase: &'static str,
    started_ns: Option<u64>,
    finished_ns: Option<u64>,
    actual_status_raw: Option<i32>,
    actual_exit_code: Option<i32>,
    actual_signal: Option<i32>,
    term: Option<SettlementSignal>,
    kill: Option<SettlementSignal>,
    observation_errors: u64,
    first_observation_errno: Option<i32>,
    wait_errno: Option<i32>,
}
impl RunnerSettlement {
    fn status(&mut self, status: std::process::ExitStatus, phase: &'static str) {
        self.phase = phase;
        self.actual_status_raw = Some(status.into_raw());
        self.actual_exit_code = status.code();
        self.actual_signal = status.signal();
    }
    fn finish(mut self) -> Self {
        self.finished_ns = boot::monotonic_ns().ok();
        eprintln!(
            "read-private-parent settlement={}",
            serde_json::to_string(&self).unwrap()
        );
        self
    }
}
fn retain_read_failure(
    stderr: &Path,
    pid: u32,
    captured: &[u8],
    mut record: serde_json::Value,
) -> std::io::Result<PathBuf> {
    if captured.len() > 64 * 1024 {
        return Err(std::io::Error::other("failure stderr exceeds64KiB"));
    }
    let stderr_result = File::options()
        .write(true)
        .truncate(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(stderr)
        .and_then(|mut file| file.write_all(captured));
    record["stderr_persisted"] = stderr_result.is_ok().into();
    record["stderr_errno"] = stderr_result.err().and_then(|e| e.raw_os_error()).into();
    let bytes = serde_json::to_vec(&record)?;
    if bytes.len() > 8192 {
        return Err(std::io::Error::other("failure receipt exceeds8KiB"));
    }
    let path = stderr.with_file_name(format!("worker-failure-{pid}.json"));
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    file.write_all(&bytes)?;
    Ok(path)
}
impl Drop for Runner {
    fn drop(&mut self) {
        let failing = thread::panicking() || self.hello_failure.is_some();
        drop(self.lease.take());
        self.input.take();
        drop(self.child.stdin.take());
        let stop_at = Instant::now();
        self.stop_reader();
        let settlement = (!self.reaped).then(|| settle_runner(&mut self.child));
        if let Some(reader) = self.reader.take() {
            // The owned lane has only <=4KiB nonblocking reads, <=5ms polls,
            // and sends whose receiver was disposed above. Keep the handle
            // until actual join, including a failed deadline or thread panic.
            let join_at = Instant::now();
            let joined = reader.join();
            let timely = join_at.elapsed() <= Duration::from_secs(2);
            eprintln!(
                "read-private-parent reader_pid={} actual_join=true stop_requested=true outcome={joined:?} join_wait_within2s={timely} stop_to_join_ns={}",
                self.child.id(),
                stop_at.elapsed().as_nanos()
            );
            if !timely || joined.is_err() {
                eprintln!(
                    "dream-read private_parent reader_gate=failed pid={} completion=unproven actual_join=true",
                    self.child.id()
                );
            }
        }
        if failing {
            // Failure evidence has a fixed capture and at most16 nonblocking
            // reads. A descendant retaining stderr cannot hold up this unwind.
            let mut discarded = 0usize;
            let mut stderr_errno = None;
            for _ in 0..16 {
                let Some(pipe) = self.stderr_pipe.as_mut() else {
                    break;
                };
                let mut bytes = [0u8; 4096];
                match pipe.read(&mut bytes) {
                    Ok(0) => {
                        self.stderr_pipe.take();
                        break;
                    }
                    Ok(n) => {
                        let keep = n.min((64usize * 1024).saturating_sub(self.stderr_bytes.len()));
                        self.stderr_bytes.extend_from_slice(&bytes[..keep]);
                        discarded += n - keep;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(error) => {
                        stderr_errno = error.raw_os_error();
                        break;
                    }
                }
            }
            let record = serde_json::json!({
                "kind":"private_fixture_failure_evidence", "pid":self.child.id(),
                "run":self.run_id.0, "hello":self.hello_failure,
                "prior_wait4_status_raw":self.reap_status, "settlement":settlement,
                "stderr_captured_bytes":self.stderr_bytes.len(), "stderr_discarded_bytes":discarded,
                "stderr_eof":self.stderr_pipe.is_none(), "stderr_read_errno":stderr_errno,
                "helper_settlement":"not_certified_by_fixture_child_reap", "cpu_accounting":"not_measured_by_failure_evidence"
            });
            let saved =
                retain_read_failure(&self.stderr, self.child.id(), &self.stderr_bytes, record);
            eprintln!(
                "read-private-parent failure_evidence pid={} result={saved:?}",
                self.child.id()
            );
        }
    }
}
fn settle_runner(child: &mut Child) -> RunnerSettlement {
    // Adapted from the canonical C1 fixture: EOF2s, exact retained PID TERM,
    // another2s, then final exact Child escalation. Never kill the worker first
    // while its independent owner can still cancel/reap its helper descendants.
    let mut receipt = RunnerSettlement {
        pid: child.id(),
        phase: "unproven",
        started_ns: boot::monotonic_ns().ok(),
        finished_ns: None,
        actual_status_raw: None,
        actual_exit_code: None,
        actual_signal: None,
        term: None,
        kill: None,
        observation_errors: 0,
        first_observation_errno: None,
        wait_errno: None,
    };
    for phase in ["eof", "term"] {
        let until = Instant::now() + Duration::from_secs(2);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    receipt.status(status, phase);
                    return receipt.finish();
                }
                Ok(None) => {}
                Err(error) => {
                    receipt.observation_errors += 1;
                    if receipt.observation_errors == 1 {
                        receipt.first_observation_errno = error.raw_os_error();
                    }
                    if error.kind() != std::io::ErrorKind::Interrupted {
                        // No fresh unreaped identity means no safe signal and
                        // no invented reap. Retain the actual unknown verdict.
                        return receipt.finish();
                    }
                }
            }
            if Instant::now() >= until {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        if phase == "eof" {
            let result = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
            receipt.term = Some(SettlementSignal {
                result,
                errno: (result != 0)
                    .then(|| std::io::Error::last_os_error().raw_os_error())
                    .flatten(),
            });
        }
    }
    match child.try_wait() {
        Ok(Some(status)) => receipt.status(status, "term"),
        Err(error) => {
            receipt.observation_errors += 1;
            if receipt.observation_errors == 1 {
                receipt.first_observation_errno = error.raw_os_error();
            }
        }
        Ok(None) => {
            eprintln!(
                "dream-read private_parent quarantine=forced_worker_exit pid={} helper_settlement=unproven",
                child.id()
            );
            let killed = child.kill();
            receipt.kill = Some(SettlementSignal {
                result: if killed.is_ok() { 0 } else { -1 },
                errno: killed.err().and_then(|e| e.raw_os_error()),
            });
            match child.wait() {
                Ok(status) => receipt.status(status, "kill"),
                Err(error) => {
                    receipt.wait_errno = error.raw_os_error();
                }
            }
        }
    }
    receipt.finish()
}
fn run(job: &Job) -> (w::Done, i32, String) {
    let mut worker = Runner::start(job);
    worker.send(&w::WorkerIn::Start(job.start.clone()));
    worker.finish(job)
}
fn assert_refused(job: &Job) -> w::Code {
    let (done, exit, stderr) = run(job);
    assert_eq!(done.outcome, w::Outcome::Failed, "{done:?} {stderr}");
    assert_ne!(exit, 0, "{stderr}");
    assert!(done.result_sha256.is_none());
    assert_eq!(done.pages, 0);
    assert!(!job.path.join("output/result.json").exists());
    let code = done.error.unwrap().code;
    assert!(
        matches!(
            code,
            w::Code::BadRequest
                | w::Code::SourceMismatch
                | w::Code::SourceChanged
                | w::Code::BlobMismatch
                | w::Code::ProducerUnapproved
                | w::Code::VerificationUnproven
                | w::Code::VerificationFailed
                | w::Code::HarnessIncompatible
        ),
        "semantic poison must fail its proof/input gate, not a clock, resource, storage, protocol or worker-loss gate: {code:?} {stderr}"
    );
    code
}
fn metric(metrics: &str, key: &str) -> u64 {
    metrics
        .split_whitespace()
        .find_map(|f| f.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("missing {key}: {metrics}"))
        .parse()
        .unwrap()
}
fn helper_record<'a>(metrics: &'a str, name: &str) -> &'a str {
    let lines = metrics
        .lines()
        .filter(|line| line.starts_with(&format!("dream-read {name} ")))
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "missing/duplicate {name}: {metrics}");
    lines[0]
}
fn assert_proof_helper_gates(metrics: &str) {
    let native = helper_record(metrics, "helper_native");
    assert_eq!(
        native
            .split_whitespace()
            .find(|v| v.starts_with("profile=")),
        Some("profile=read-helper-native/1")
    );
    assert!(native.contains("tag=process_create"));
    let attempts = metric(native, "reservations");
    assert!(attempts > 0 && attempts <= 256);
    assert!(metric(native, "effects") > 0 && metric(native, "effects") <= metric(native, "count"));
    assert!(metric(native, "count") <= attempts);
    assert!(
        metric(native, "max_cpu_ns") <= 2_000_000,
        "native class CPU gate: {native}"
    );
    assert!(
        metric(native, "max_wall_ns") <= 25_000_000,
        "native class wall gate: {native}"
    );
    assert!(
        metric(native, "inclusive_max_cpu_ns") <= 3_000_000,
        "inclusive class gate: {native}"
    );
    assert!(
        metric(native, "app_max_cpu_ns") <= 1_000_000,
        "application class gate: {native}"
    );
    assert!(
        metric(native, "debt_ns") > 0,
        "creation must incur real rest debt: {native}"
    );
    let life = helper_record(metrics, "helper_lifetime");
    assert!(life.contains("coverage_known=true"));
    assert!(metric(life, "settled") > 0 && metric(life, "descendants") > 0);
    assert!(
        metric(life, "grants") > 0 && metric(life, "charge_ns") > 0 && metric(life, "debt_ns") > 0
    );
    assert!(
        metric(life, "max_charge_ns") <= 1_000_000,
        "ordinary shared helper grant gate: {life}"
    );
    assert!(
        metric(life, "observed_direct_cpu_ns") > 0
            && metric(life, "observed_descendant_cpu_ns") > 0
    );
    assert!(
        metric(life, "rolled_descendant_cpu_ns") > 0 && metric(life, "reaped_rollup_cpu_ns") > 0
    );
    let memory = helper_record(metrics, "helper_memory");
    assert_eq!(metric(memory, "group_rss_limit_bytes"), 16 * 1024 * 1024);
    assert_eq!(metric(memory, "mixed_limit_bytes"), 96 * 1024 * 1024);
    assert!(
        metric(memory, "simultaneous_group_rss_peak_bytes") > 0
            && metric(memory, "simultaneous_group_rss_peak_bytes") <= 16 * 1024 * 1024
    );
    assert!(metric(memory, "mixed_main_heap_plus_group_rss_peak_bytes") <= 96 * 1024 * 1024);
    assert!(memory.contains("native_heap_coverage=sampled_rss_not_allocator"));
}
fn assert_memory_gate(metrics: &str) {
    assert_eq!(
        metrics
            .lines()
            .filter(|line| line.starts_with("dream-read metrics "))
            .count(),
        1
    );
    assert!(
        metric(metrics, "heap_live_bytes") <= 96 * 1024 * 1024,
        "{metrics}"
    );
    assert!(
        metric(metrics, "heap_peak_bytes") <= 96 * 1024 * 1024,
        "{metrics}"
    );
    assert!(
        metric(metrics, "storage_reserved_peak_bytes") <= 78 * 1024 * 1024,
        "{metrics}"
    );
    assert_eq!(
        metric(metrics, "storage_declared_live_bytes"),
        0,
        "{metrics}"
    );
    assert_eq!(metric(metrics, "control_reserved_bytes"), 2 * 1024 * 1024);
    assert_eq!(metric(metrics, "helper_reserved_bytes"), 16 * 1024 * 1024);
    assert_eq!(metric(metrics, "credit_ceiling_bytes"), 96 * 1024 * 1024);
    assert_eq!(metric(metrics, "overruns"), 0, "{metrics}");
    if metrics
        .lines()
        .any(|line| line.starts_with("dream-read helper_native "))
    {
        let native = helper_record(metrics, "helper_native");
        assert!(metric(native, "app_max_cpu_ns") <= 1_000_000);
        let exact_native_span = metric(native, "inclusive_max_cpu_ns");
        assert!(exact_native_span <= 3_000_000);
        assert!(
            metric(metrics, "max_unit_cpu_ns") <= 1_000_000u64.max(exact_native_span),
            "only observed exact native spans may exceed ordinary1ms: {metrics}"
        );
    } else {
        assert!(metric(metrics, "max_unit_cpu_ns") <= 1_000_000, "{metrics}");
    }
    assert!(
        metric(metrics, "control_max_cpu_ns") <= 1_000_000,
        "{metrics}"
    );
    assert!(
        metric(metrics, "max_cleanup_unit_cpu_ns") <= 1_000_000,
        "{metrics}"
    );
    assert!(metrics.contains("native_cumulative_bytes=unmeasured"));
}
#[derive(Clone, Debug)]
struct ProcessSample {
    pid: i32,
    ppid: i32,
    pgid: i32,
    birth: u64,
    cpu: u64,
    user_cpu: u64,
    threads: u32,
    runnable: u32,
    rss: u64,
    status: u32,
    image: PathBuf,
}
fn absolute_cpu_ns(raw: u64) -> Option<u64> {
    #[repr(C)]
    struct Timebase {
        numer: u32,
        denom: u32,
    }
    unsafe extern "C" {
        fn mach_timebase_info(value: *mut Timebase) -> i32;
    }
    let mut t = Timebase { numer: 0, denom: 0 };
    if unsafe { mach_timebase_info(&mut t) } != 0 || t.denom == 0 {
        return None;
    }
    u64::try_from((raw as u128).checked_mul(t.numer as u128)? / t.denom as u128).ok()
}
fn process_sample(pid: i32) -> Option<ProcessSample> {
    let mut b = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let mut t = std::mem::MaybeUninit::<libc::proc_taskinfo>::zeroed();
    let mut path = [0u8; 4096];
    unsafe {
        if libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            b.as_mut_ptr().cast(),
            std::mem::size_of::<libc::proc_bsdinfo>() as i32,
        ) != std::mem::size_of::<libc::proc_bsdinfo>() as i32
            || libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTASKINFO,
                0,
                t.as_mut_ptr().cast(),
                std::mem::size_of::<libc::proc_taskinfo>() as i32,
            ) != std::mem::size_of::<libc::proc_taskinfo>() as i32
            || libc::proc_pidpath(pid, path.as_mut_ptr().cast(), path.len() as u32) <= 0
        {
            return None;
        }
        let b = b.assume_init();
        let t = t.assume_init();
        Some(ProcessSample {
            pid,
            ppid: b.pbi_ppid as i32,
            pgid: b.pbi_pgid as i32,
            birth: b.pbi_start_tvsec * 1_000_000 + b.pbi_start_tvusec,
            cpu: absolute_cpu_ns(t.pti_total_user.checked_add(t.pti_total_system)?)?,
            user_cpu: absolute_cpu_ns(t.pti_total_user)?,
            threads: t.pti_threadnum as u32,
            runnable: t.pti_numrunning as u32,
            rss: t.pti_resident_size,
            status: b.pbi_status,
            image: PathBuf::from(
                std::ffi::CStr::from_ptr(path.as_ptr().cast())
                    .to_str()
                    .unwrap(),
            ),
        })
    }
}
fn exact_children(pid: i32) -> Vec<i32> {
    let mut ids = [0i32; 3];
    let count = unsafe {
        libc::proc_listchildpids(
            pid,
            ids.as_mut_ptr().cast(),
            std::mem::size_of_val(&ids) as i32,
        )
    };
    assert!(count >= 0 && count as usize <= ids.len());
    ids[..count as usize]
        .iter()
        .copied()
        .filter(|v| *v > 0)
        .collect()
}
fn worker_group(worker_pid: i32, provision: &HelperProvision) -> Option<Vec<ProcessSample>> {
    let leaders = exact_children(worker_pid);
    assert!(leaders.len() <= 1, "one helper family per worker");
    let pid = *leaders.first()?;
    let leader = process_sample(pid)?;
    assert_eq!(leader.ppid, worker_pid);
    assert_eq!(leader.pgid, pid);
    assert!(leader.birth > 0);
    assert!(
        leader.image == provision.git || leader.image == provision.gpg,
        "exact provisioned leader image"
    );
    let children = exact_children(pid);
    assert!(children.len() <= 1, "at most two members per group");
    let mut result = vec![leader];
    for child in children {
        let sample = process_sample(child)?;
        assert_eq!(sample.ppid, pid);
        assert_eq!(sample.pgid, pid);
        assert!(
            sample.image == provision.gpg || sample.image == provision.git,
            "descendant stays within the two exact provisioned images"
        );
        assert!(
            exact_children(child).is_empty(),
            "provisioned GPG cannot add descendants"
        );
        result.push(sample);
    }
    assert!(
        result.iter().map(|v| v.rss).sum::<u64>() <= 16 * 1024 * 1024,
        "actual simultaneous group RSS"
    );
    Some(result)
}
#[cfg(target_os = "macos")]
fn parent_live_heap() -> u64 {
    #[repr(C)]
    struct Statistics {
        blocks: u32,
        size: usize,
        peak: usize,
        allocated: usize,
    }
    unsafe extern "C" {
        fn malloc_zone_statistics(zone: *mut libc::c_void, stats: *mut Statistics);
    }
    let mut stats = Statistics {
        blocks: 0,
        size: 0,
        peak: 0,
        allocated: 0,
    };
    unsafe { malloc_zone_statistics(std::ptr::null_mut(), &mut stats) };
    stats.size as u64
}
#[cfg(not(target_os = "macos"))]
fn parent_live_heap() -> u64 {
    u64::MAX
}
fn assert_qualified_snapshot(job: &Job, done: &w::Done, entry_id: &str) {
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?}");
    assert!(done.error.is_none());
    let descriptor = std::fs::read(job.path.join("output/result.json")).unwrap();
    assert_eq!(done.result_sha256, Some(w::hash(&descriptor)));
    let snapshot: w::Snapshot = w::decode(&descriptor).unwrap();
    assert_eq!(snapshot.run_id, job.start.run_id);
    assert_eq!(snapshot.nonce, job.start.nonce);
    assert_eq!(snapshot.input_digest, job.start.input_sha256);
    assert_eq!(
        snapshot.source_revision,
        w::source_revision(&job.input).unwrap()
    );
    assert_eq!(snapshot.entry_ids, vec![w::Id(entry_id.into())]);
    assert_eq!(snapshot.coverage.indexed, 1);
    assert_eq!(snapshot.coverage.omitted, 0);
    assert!(snapshot.coverage.complete);
    let mut entries = vec![];
    let mut total = descriptor.len();
    for (index, pin) in snapshot.pages.iter().enumerate() {
        assert_eq!(pin.index, index as u32);
        let bytes = std::fs::read(job.path.join(format!("output/page-{index:06}.json"))).unwrap();
        assert!(bytes.len() <= w::MAX_PAGE);
        assert_eq!(pin.len.0, bytes.len() as u64);
        assert_eq!(pin.sha256, w::hash(&bytes));
        total += bytes.len();
        let page: w::Page = w::decode(&bytes).unwrap();
        assert_eq!(page.run_id, job.start.run_id);
        assert_eq!(page.nonce, job.start.nonce);
        assert_eq!(page.index, index as u32);
        assert_eq!(
            page.next,
            if index + 1 < snapshot.pages.len() {
                Some(index as u32 + 1)
            } else {
                None
            }
        );
        entries.extend(page.entries);
    }
    assert!(total <= w::MAX_SNAPSHOT);
    assert_eq!(entries.len(), 1);
    w::validate_entry(&entries[0]).unwrap();
    assert_eq!(entries[0].hit.id.0, entry_id);
    assert_eq!(entries[0].hit.title, "Run the private verifier");
    assert_eq!(entries[0].why, "A checked source protects this fixture.");
    assert!(!entries[0].context.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires separately qualified proof tools; re-enable via tasks/2026-11-05-dream-proof-tools-provisioner"]
async fn prepared_tool_capability_probe_has_stopped_start_and_actual_git_gpg_rollup() {
    // This constructs the existing private publication and runs only the separate
    // native/tool capability fixture. It never launches a managed Dream worker.
    let template = Template::publication().await;
    let provision = &template.provision;
    let receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&provision.receipt).unwrap()).unwrap();
    let evidence = std::fs::read(provision.home.path().join("capability-evidence.json")).unwrap();
    assert_eq!(
        receipt["capability"]["evidence_sha256"],
        w::hash(&evidence).0
    );
    assert_eq!(
        receipt["source_sha256"],
        bboxy450_harness::dream::resolve::provision_source_sha256()
    );
    let native = &provision.evidence["native"];
    let git = &provision.evidence["git"];
    assert_eq!(native["created_suspended"], true);
    assert_eq!(native["cloexec_default"], true);
    assert_eq!(native["audit_token_signals"], true);
    assert!(
        matches!(native["stale_audit_error"].as_i64(),Some(v) if v==libc::ESRCH as i64||v==libc::EINVAL as i64)
    );
    assert_eq!(native["stale_pid_version_control"], true);
    assert!(native["waited_child_cpu_ns"].as_u64().unwrap() >= 12_000_000);
    assert_eq!(git["max_members"], 2);
    assert!(git["sampled_gpg_cpu_ns"].as_u64().unwrap() > 0);
    assert!(
        git["rolled_gpg_cpu_ns"].as_u64().unwrap() >= git["sampled_gpg_cpu_ns"].as_u64().unwrap()
    );
    assert!(
        git["git_wait4_cpu_ns"].as_u64().unwrap()
            >= git["sampled_git_cpu_ns"].as_u64().unwrap()
                + git["sampled_gpg_cpu_ns"].as_u64().unwrap()
    );
    assert_eq!(native["reaped"], true);
    assert_eq!(git["reaped"], true);
    let retained = provision.retain_evidence();
    eprintln!(
        "capability_component_evidence={} source_sha256={} git_sha256={} gpg_sha256={} native_child_cpu_ns={} git_sampled_gpg_cpu_ns={} git_rolled_gpg_cpu_ns={} git_wait4_cpu_ns={}",
        retained.display(),
        receipt["source_sha256"],
        receipt["git"]["sha256"],
        receipt["gpg"]["sha256"],
        native["waited_child_cpu_ns"],
        git["sampled_gpg_cpu_ns"],
        git["rolled_gpg_cpu_ns"],
        git["git_wait4_cpu_ns"]
    );
}

#[test]
fn private_candidate_outer_codec_refuses_noncanonical_and_overflow_before_birth() {
    // Synthetic syntax vectors carry no source/build/OS premise or approval.
    // The wrapper invokes only the canonical outer decoder, never pretest or
    // the helper engine; the builder's pending C closure codec is independent.
    let home = tempfile::Builder::new()
        .prefix("dream-read-private-outer-codec-")
        .tempdir_in("/private/tmp")
        .unwrap();
    let source = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/dream-read/private-input-controls.c"
    ));
    let probe = home.path().join("controls");
    provision_command(
        Path::new("/usr/bin/clang"),
        &[
            "-O2",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            source.to_str().unwrap(),
            "-o",
            probe.to_str().unwrap(),
        ],
        home.path(),
        None,
    );
    let file = serde_json::json!({"dev":1,"ino":2,"len":3,"mtime_nsec":999999999,
        "mtime_sec":i64::MIN,"path":"provenance/pretest-basis.json","root":"candidate","sha256":"a".repeat(64)});
    let dir = serde_json::json!({"dev":1,"ino":2,"path":"/private/fixture","uid":u32::MAX});
    let mut key = file.clone();
    key["root"] = "fixture".into();
    let input = serde_json::json!({"arch":"aarch64","build":file,"case":"positive",
        "closure":file,"commands":file,"config":file,"deadline_ns":u64::MAX,
        "fixture":{"corrupt_commit":"b".repeat(40),"manifest":file,"public_keys":key,"repo":dir,"valid_commit":"a".repeat(40)},
        "git":file,"gpg":file,"incarnation":u64::MAX,"kind":"private_candidate_test",
        "nonce":"01M6RNNVQX1JAY0X2TX1GFEGE4","os_build":"25A123","output":dir,"prefix":dir,
        "probe_exe_sha256":"a".repeat(64),"probe_source_sha256":"b".repeat(64),
        "run_id":"01M6RNNVQX1JAY0X2TX1GFEGE4","source_lock":file,"source_review":file});
    let base = String::from_utf8(w::canonical(&input).unwrap()).unwrap();
    let mut cases = vec![("full-width", base.as_bytes().to_vec(), true)];
    for (name, from, to) in [
        (
            "u64-overflow",
            "\"deadline_ns\":18446744073709551615",
            "\"deadline_ns\":18446744073709551616",
        ),
        ("u32-overflow", "\"uid\":4294967295", "\"uid\":4294967296"),
        (
            "i64-underflow",
            "\"mtime_sec\":-9223372036854775808",
            "\"mtime_sec\":-9223372036854775809",
        ),
        (
            "i64-positive-overflow",
            "\"mtime_sec\":-9223372036854775808",
            "\"mtime_sec\":9223372036854775808",
        ),
        (
            "negative-zero",
            "\"mtime_sec\":-9223372036854775808",
            "\"mtime_sec\":-0",
        ),
        ("leading-zero", "\"dev\":1", "\"dev\":01"),
        ("exponent", "\"dev\":1", "\"dev\":1e0"),
        ("fraction", "\"dev\":1", "\"dev\":1.0"),
        (
            "zero-incarnation",
            "\"incarnation\":18446744073709551615",
            "\"incarnation\":0",
        ),
        (
            "nanosecond-overflow",
            "\"mtime_nsec\":999999999",
            "\"mtime_nsec\":1000000000",
        ),
        (
            "unknown-case",
            "\"case\":\"positive\"",
            "\"case\":\"approved\"",
        ),
        ("null-union", "\"arch\":\"aarch64\"", "\"arch\":null"),
        (
            "duplicate-key",
            "\"arch\":\"aarch64\"",
            "\"arch\":\"aarch64\",\"arch\":\"aarch64\"",
        ),
        (
            "unknown-key",
            "\"arch\":\"aarch64\"",
            "\"approved\":true,\"arch\":\"aarch64\"",
        ),
        ("missing-key", "\"arch\":\"aarch64\",", ""),
        (
            "noncanonical-space",
            "\"arch\":\"aarch64\"",
            "\"arch\": \"aarch64\"",
        ),
        (
            "escaped-printable",
            "\"arch\":\"aarch64\"",
            "\"arch\":\"\\u0061arch64\"",
        ),
        (
            "escaped-nul",
            "\"arch\":\"aarch64\"",
            "\"arch\":\"\\u0000\"",
        ),
        ("surrogate", "\"arch\":\"aarch64\"", "\"arch\":\"\\ud800\""),
        (
            "escaped-short-control",
            "\"arch\":\"aarch64\"",
            "\"arch\":\"\\u000a\"",
        ),
        (
            "unknown-root",
            "\"root\":\"candidate\"",
            "\"root\":\"ambient\"",
        ),
    ] {
        assert!(base.contains(from), "{name}");
        cases.push((name, base.replacen(from, to, 1).into_bytes(), false));
    }
    for (name, bytes) in [
        (
            "leading-bom",
            [b"\xef\xbb\xbf".as_slice(), base.as_bytes()].concat(),
        ),
        ("trailing-lf", [base.as_bytes(), b"\n"].concat()),
        ("truncated", base.as_bytes()[..base.len() - 1].to_vec()),
        ("oversize", vec![b' '; 16 * 1024 + 1]),
        (
            "invalid-utf8",
            base.replacen("aarch64", "\u{80}", 1)
                .into_bytes()
                .into_iter()
                .map(|b| if b == 0xc2 { 0xc0 } else { b })
                .collect(),
        ),
    ] {
        cases.push((name, bytes, false));
    }
    for (name, bytes, accepted) in &cases {
        let path = home.path().join(format!("{name}.json"));
        private_file(&path, bytes);
        let output = capability_command(
            &probe,
            &[
                if *accepted { "accept" } else { "reject" },
                path.to_str().unwrap(),
            ],
            home.path(),
            name,
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("helper_created=0 runtime_approved=false")
        );
    }
    eprintln!(
        "private_outer_codec cases={} parser_only=true helper_created=0 runtime_approved=false",
        cases.len()
    );
}

#[test]
fn signature_boundary_controls_refuse_before_helper_birth() {
    // This includes the canonical controller in a test translation unit and
    // calls its parser/prebirth APIs. It creates no Git/GPG helper and cannot
    // establish the separately required real-pair signature witnesses.
    let home = tempfile::Builder::new()
        .prefix("dream-read-signature-boundary-")
        .tempdir_in("/private/tmp")
        .unwrap();
    let source = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/dream-read/signature-boundary-controls.c"
    ));
    let probe = home.path().join("controls");
    provision_command(
        Path::new("/usr/bin/clang"),
        &[
            "-O2",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            source.to_str().unwrap(),
            "-o",
            probe.to_str().unwrap(),
        ],
        home.path(),
        None,
    );
    let output = capability_command(
        &probe,
        &[home.path().to_str().unwrap()],
        home.path(),
        "signature-boundary-controls",
    );
    let text = std::str::from_utf8(&output.stdout).unwrap();
    for observation in [
        "engine_B_controls=pass",
        "token_refusals=5",
        "no_direct_verify_child=true",
        "selected_case_bound=true",
        "no_rebind=true",
        "prebirth_receipt=durable",
        "ordinary_debt=paid",
        "ledger_bound=pass",
        "forbidden_umask=refused",
        "wrong_case_receipt=refused",
        "created=0",
        "runtime_approved=false",
        "scratch_settlement_controls=pass",
        "current_scratch=settled",
        "replaced_normal=quarantined",
        "replaced_killed=quarantined",
        "foreign_unlink=false",
        "real_pair=false",
    ] {
        assert!(
            text.split_whitespace().any(|word| word == observation),
            "{text}"
        );
    }
    eprint!("{text}");
    // Causal source mutant, private to this test: removing both GS2 directory
    // fences must fail the same real-file API assertions. No runtime switch or
    // alternate product controller is introduced.
    let mutant = home.path().join("mutant");
    private_dir(&mutant);
    let original = std::fs::read_to_string(source.with_file_name("proof-helper-probe.c")).unwrap();
    let mut changed = original.clone();
    for detail in [
        "GS2 retained scratch no longer names the original directory",
        "GS2 scratch identity changed during settlement",
    ] {
        let guard = format!(
            "    if(!private_sig_scratch_current(s))\n        return private_sig_error(s,\"settlement_unknown\",\"{detail}\",1);\n"
        );
        assert_eq!(changed.matches(&guard).count(), 1);
        changed = changed.replacen(&guard, "", 1);
    }
    assert_ne!(changed, original);
    private_file(&mutant.join("proof-helper-probe.c"), changed.as_bytes());
    let mutant_source = mutant.join("signature-boundary-controls.c");
    private_file(&mutant_source, &std::fs::read(source).unwrap());
    let mutant_probe = mutant.join("controls");
    provision_command(
        Path::new("/usr/bin/clang"),
        &[
            "-O2",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            mutant_source.to_str().unwrap(),
            "-o",
            mutant_probe.to_str().unwrap(),
        ],
        home.path(),
        None,
    );
    let run = mutant.join("run");
    private_dir(&run);
    let broken = Command::new(&mutant_probe)
        .arg(&run)
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(broken.status.code(), Some(81));
    assert!(
        String::from_utf8_lossy(&broken.stderr)
            .contains("GS2 refuses detached scratch on normal or killed settlement")
    );
    eprintln!(
        "signature_scratch_source_mutant expected_exit=81 observed_exit=81 causal_assertion=GS2_refuses_detached_scratch real_pair=false"
    );
}

#[test]
#[ignore = "load-sensitive budget/wall window; re-enable via tasks/2026-11-06-qos-builds-never-starve-mcp"]
fn member_windows_use_post_ack_unequal_endpoints_and_closed_roster() {
    // Only the canonical finite C program is launched here. No Git/GPG tool
    // qualification or managed Read availability follows from this component.
    let home = tempfile::Builder::new()
        .prefix("dream-read-member-window-")
        .tempdir_in("/private/tmp")
        .unwrap();
    let source = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/dream-read/proof-helper-probe.c"
    ));
    let probe = home.path().join("probe");
    provision_command(
        Path::new("/usr/bin/clang"),
        &[
            "-O2",
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            source.to_str().unwrap(),
            "-o",
            probe.to_str().unwrap(),
        ],
        home.path(),
        None,
    );
    let control = capability_command(&probe, &["ledger-controls"], home.path(), "ledger-controls");
    let controls: serde_json::Value = serde_json::from_slice(&control.stdout).unwrap();
    assert_eq!(controls["causal_controls"], 18);
    assert_eq!(controls["tool_availability"], "verification_unproven");
    let native = home.path().join("native");
    private_dir(&native);
    let output = capability_command(
        &probe,
        &["native", native.to_str().unwrap()],
        home.path(),
        "native",
    );
    let evidence: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let ledger = &evidence["member_windows"];
    assert_eq!(ledger["basis"], "canonical_private_c_program_only");
    let t0 = ledger["t0_ns"].as_u64().unwrap();
    let close = ledger["tclose_ns"].as_u64().unwrap();
    let first = ledger["endpoint_ns"][0].as_u64().unwrap();
    let second = ledger["endpoint_ns"][1].as_u64().unwrap();
    assert!(t0 <= first && first < second && second <= close);
    let windows = ledger["W_ns"].as_u64().unwrap();
    let actual = ledger["A_ns"].as_u64().unwrap();
    let controller = ledger["C_ns"].as_u64().unwrap();
    assert_eq!(windows, (first - t0) + (second - t0));
    assert!(
        windows < 2 * (second - t0),
        "unequal endpoints must retain their actual independent bounds"
    );
    assert!(actual <= windows, "unexplained A>W cannot acquire coverage");
    assert_eq!(
        ledger["Q_ns"].as_u64().unwrap(),
        windows.max(actual) + controller
    );
    assert!(ledger["Q_ns"].as_u64().unwrap() <= 1_000_000);
    assert!(
        ledger["post_cpu_ns"].as_u64().unwrap() + ledger["late_kernel_ns"].as_u64().unwrap()
            <= 1_000_000
    );
    assert!(
        ledger["debt_deadline_ns"].as_u64().unwrap() >= close + 18_000_000,
        "active and later ordinary units owe separate fresh minimum9ms rests"
    );
    assert_eq!(ledger["roster_closed"], true);
    assert_eq!(evidence["reaped"], true);
    for field in ["leader", "child"] {
        let pid = evidence[field].as_i64().unwrap() as i32;
        let name = format!("{field}_birth");
        let birth = evidence[name.as_str()].as_u64().unwrap();
        assert!(process_sample(pid).is_none_or(|now| now.birth != birth));
    }
    let unavailable = Command::new(&probe)
        .args([
            "git",
            "/unsupported/git",
            "/unsupported/gpg",
            home.path().to_str().unwrap(),
            home.path().to_str().unwrap(),
            "unadmitted",
            home.path().to_str().unwrap(),
            "uncreated-trace",
            "uncreated-output",
        ])
        .current_dir(home.path())
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(unavailable.status.code(), Some(81));
    let refusal: serde_json::Value = serde_json::from_slice(&unavailable.stdout).unwrap();
    assert_eq!(refusal["availability"], "verification_unproven");
    assert_eq!(refusal["whole_command_lifetime_coverage"], false);
    assert!(
        !home.path().join("uncreated-trace").exists()
            && !home.path().join("uncreated-output").exists()
    );
    let retained = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!(
            "dream-read-member-window-component-{}",
            ulid::Ulid::new()
        ));
    private_dir(&retained);
    for (name, bytes) in [
        ("ledger-controls.stdout", control.stdout),
        ("native.stdout", output.stdout),
        ("native.stderr", output.stderr),
        ("unavailable.stdout", unavailable.stdout),
        ("unavailable.stderr", unavailable.stderr),
    ] {
        private_file(&retained.join(name), &bytes);
    }
    private_file(
        &retained.join("proof-helper-probe.c"),
        &std::fs::read(source).unwrap(),
    );
    private_file(&retained.join("probe"), &std::fs::read(&probe).unwrap());
    std::fs::set_permissions(
        retained.join("probe"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    private_file(&retained.join("component.json"),&w::canonical(&serde_json::json!({"profile":"read-helper-native/1","kind":"nonqualifying_component_evidence","source_sha256":w::hash(std::fs::read(source).unwrap()).0,"probe_sha256":w::hash(std::fs::read(&probe).unwrap()).0,"os":{"release":os_release(),"arch":std::env::consts::ARCH},"commands":{"compile":["/usr/bin/clang","-O2","-std=c11","-Wall","-Wextra","-Werror","proof-helper-probe.c","-o","probe"],"controls":["probe","ledger-controls"],"native":["probe","native","private-native-dir"]},"exits":{"compile":0,"controls":0,"native":0,"unavailable":81}})).unwrap());
    eprintln!(
        "member-window-component retained={} clang_exit=0 controls_exit=0 native_exit=0 unavailable_exit=81 basis=canonical_private_c_program_only",
        retained.display()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_scripted_roots_are_selected_before_signing_and_preserve_completed_bytes() {
    let roots = [
        "fixture.invalid/dream/repo-a",
        "fixture.invalid/dream/repo-b",
    ];
    let built = [
        root_fixture::completed_publication(roots[0]).await,
        root_fixture::completed_publication(roots[1]).await,
    ];
    assert_ne!(built[0].fixture.entry, built[1].fixture.entry);
    for (root, built) in roots.into_iter().zip(&built) {
        let f = &built.fixture;
        assert_eq!(f.record.root, root);
        assert_eq!(f.producer.repo, root);
        assert_eq!(f.fixture.value["roots"][0]["repo"], root);
        assert_eq!(
            f.fixture
                .repo
                .git(&["config", "--get", "remote.origin.url"]),
            format!("https://{root}.git")
        );
        let bytes = std::fs::read(f.fixture.repo.path.join(".bboxy/memories/lesson.md")).unwrap();
        let record = bboxy450_harness::dream::trust::verify_bytes(
            &bytes,
            f.record.id.as_deref().unwrap(),
            &["seat".into()],
        )
        .unwrap();
        assert_eq!(record.front["scope"]["repos"], serde_json::json!([root]));
        assert_eq!(record.validate_identity().unwrap().id().unwrap(), f.entry);
        assert_eq!(entry::digest(&bytes), f.record.digest);
        let mut immutable = BTreeMap::<PathBuf, Vec<u8>>::new();
        immutable.insert(f.fixture.repo.path.join(".bboxy/memories/lesson.md"), bytes);
        for (session, events) in &built.logs {
            let Event::SessionStarted { meta } = &events[0].event else {
                panic!("source header")
            };
            assert_eq!(meta.vcs.as_ref().unwrap().repo.as_deref(), Some(root));
            let path = f.sessions.join(format!("{session}.jsonl"));
            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(
                bytes,
                events
                    .iter()
                    .flat_map(|e| e.wire_bytes().unwrap())
                    .collect::<Vec<_>>()
            );
            immutable.insert(path, bytes);
            for event in events {
                assert!(!matches!(
                    &event.event,
                    Event::ItemCompleted {
                        body: Item::Recall { .. },
                        ..
                    }
                ));
                for blob in inspect::blobs(&event.event).unwrap() {
                    let path = f
                        .sessions
                        .join(session.to_string())
                        .join("blobs")
                        .join(&blob.sha256);
                    let bytes = std::fs::read(&path).unwrap();
                    assert_eq!(w::hash(&bytes).0, blob.sha256);
                    immutable.insert(path, bytes);
                }
            }
        }
        let mut resolver = root_fixture::resolver(f, root).await;
        let audit = bboxy450_harness::dream::audit::run(&mut resolver, Set::default(), false, 4005)
            .await
            .unwrap();
        assert_eq!(
            audit.exit_code(),
            0,
            "private constructed proof root={root} findings={}",
            serde_json::to_string(&audit.findings).unwrap()
        );
        for (path, bytes) in immutable {
            assert_eq!(
                std::fs::read(path).unwrap(),
                bytes,
                "completed proof bytes were rewritten"
            );
        }
    }
    eprintln!(
        "two-root-construction roots=2 signed_before_packaging=true read_worker_started=false tool_qualification=unproven"
    );
}

#[test]
fn read_bootstrap_marker_owned_pipe_requires_exactly_one_canonical_entry() {
    let waiting = "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0\n";
    let now = boot::monotonic_ns().unwrap();
    let entry = format!("dream-bootstrap entry s=1 g=2 e={now} m={now} u=3 k=4 r=5 p=6 ok=1\n");
    let valid = format!("{entry}{waiting}");
    let mut cases = vec![
        ("canonical", valid.clone().into_bytes(), true),
        ("missing-entry", waiting.as_bytes().to_vec(), false),
        (
            "duplicate-entry",
            format!("{entry}{entry}{waiting}").into_bytes(),
            false,
        ),
        (
            "unexpected-prefix",
            format!("diagnostic\n{valid}").into_bytes(),
            false,
        ),
        (
            "unexpected-after-waiting",
            format!("{valid}diagnostic\n").into_bytes(),
            false,
        ),
        (
            "wrong-key",
            valid.replacen(" g=", " x=", 1).into_bytes(),
            false,
        ),
        (
            "leading-zero",
            valid.replacen(" s=1 ", " s=01 ", 1).into_bytes(),
            false,
        ),
        (
            "overflow",
            valid
                .replacen(" s=1 ", " s=18446744073709551616 ", 1)
                .into_bytes(),
            false,
        ),
        (
            "bad-status",
            valid.replacen(" ok=1", " ok=2", 1).into_bytes(),
            false,
        ),
        (
            "failed-status-nonzero-counters",
            valid.replacen(" ok=1", " ok=0", 1).into_bytes(),
            false,
        ),
        (
            "reversed-clock",
            valid.replacen(&format!(" m={now}"), " m=0", 1).into_bytes(),
            false,
        ),
        (
            "future-clock",
            valid
                .replacen(&format!(" m={now}"), " m=18446744073709551615", 1)
                .into_bytes(),
            false,
        ),
        (
            "wrong-waiting",
            valid
                .replacen("argv_admissions=0", "argv_admissions=1", 1)
                .into_bytes(),
            false,
        ),
        (
            "oversized-entry",
            [vec![b'x'; 257], b"\n".to_vec(), waiting.as_bytes().to_vec()].concat(),
            false,
        ),
        (
            "oversized-waiting",
            [entry.as_bytes().to_vec(), vec![b'x'; 257], b"\n".to_vec()].concat(),
            false,
        ),
    ];
    cases.push((
        "rusage-unavailable-raw-zeroes",
        format!("dream-bootstrap entry s=0 g=2 e={now} m={now} u=0 k=0 r=0 p=0 ok=0\n{waiting}")
            .into_bytes(),
        true,
    ));
    for (case, bytes, expected) in cases {
        let at = parent_cpu();
        let deadline = boot::monotonic_ns()
            .unwrap()
            .checked_add(boot::WINDOW_NS)
            .unwrap();
        let (mut read, mut write) = bootstrap_pipe();
        write.write_all(&bytes).unwrap();
        drop(write);
        let mut captured = Vec::with_capacity(512);
        read.read_to_end(&mut captured).unwrap();
        assert_eq!(captured, bytes, "{case}: actual pipe bytes changed");
        assert!(captured.len() <= 512, "{case}: bounded prepermit capture");
        let observed = boot::monotonic_ns().unwrap();
        let verdict = bootstrap_waiting_marker(&captured, observed);
        let admitted = matches!(verdict, Ok(Some(marker)) if marker.trim() == waiting.trim());
        assert_eq!(admitted, expected, "{case}: {verdict:?}");
        assert!(
            observed < deadline,
            "{case}: original250ms observation bound"
        );
        drop(read);
        parent_paid(at);
        eprintln!(
            "read-bootstrap-marker case={case} actual_pipe_bytes={} exact_waiting={admitted} permit_writes=0 capture_limit=512 original_window_ns={}",
            captured.len(),
            boot::WINDOW_NS
        );
    }
    // Real pipe reads may split either record. No partial entry or waiting
    // record can admit a permit; the same complete bytes must still validate.
    let at = parent_cpu();
    let deadline = boot::monotonic_ns()
        .unwrap()
        .checked_add(boot::WINDOW_NS)
        .unwrap();
    let (mut read, mut write) = bootstrap_pipe();
    let mut captured = Vec::with_capacity(512);
    for (i, chunk) in [
        &entry.as_bytes()[..entry.len() - 1],
        &entry.as_bytes()[entry.len() - 1..],
        &waiting.as_bytes()[..waiting.len() - 1],
        &waiting.as_bytes()[waiting.len() - 1..],
    ]
    .into_iter()
    .enumerate()
    {
        write.write_all(chunk).unwrap();
        let mut bytes = vec![0; chunk.len()];
        read.read_exact(&mut bytes).unwrap();
        captured.extend_from_slice(&bytes);
        let observed = boot::monotonic_ns().unwrap();
        let marker = bootstrap_waiting_marker(&captured, observed).unwrap();
        if i < 3 {
            assert_eq!(marker, None, "partial record admitted a permit");
        } else {
            assert_eq!(marker, Some(waiting));
        }
        assert!(observed < deadline);
    }
    drop(write);
    drop(read);
    parent_paid(at);
    eprintln!(
        "read-bootstrap-marker case=fragmented actual_pipe_reads=4 permit_writes=0 original_deadline_not_reset=true"
    );
}

#[test]
fn read_bootstrap_failure_receipt_keeps_actual_poll_reap_and_private_home() {
    let home = ReadFailureHome(
        tempfile::Builder::new()
            .prefix("dream-read-failure-control-")
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in("/private/tmp")
            .unwrap(),
    );
    let root = home.path().to_owned();
    let stderr = root.join("worker.stderr");
    private_file(&stderr, b"");
    struct OwnedControl(Child, bool);
    impl Drop for OwnedControl {
        fn drop(&mut self) {
            drop(self.0.stdin.take());
            if !self.1 {
                let _ = settle_runner(&mut self.0);
            }
        }
    }
    let mut child = OwnedControl(
        Command::new("/bin/cat")
            .env_clear()
            .env("HOME", &root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
        false,
    );
    let pid = child.0.id();
    let started = boot::monotonic_ns().unwrap();
    let deadline = started.checked_add(boot::WINDOW_NS).unwrap();
    let mut poll = libc::pollfd {
        fd: child.0.stdout.as_ref().unwrap().as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // A zero-time poll of this held empty pipe is deterministic observation
    // evidence only; this fixture never certifies a worker's Hello readiness.
    let result = unsafe { libc::poll(&mut poll, 1, 0) };
    let errno = (result < 0)
        .then(|| std::io::Error::last_os_error().raw_os_error())
        .flatten();
    let after = boot::monotonic_ns().ok();
    let failure = HelloPollFailure::observe(
        &mut child.0,
        &w::Ulid(ulid::Ulid::new().to_string()),
        deadline,
        started,
        after,
        (poll, result, errno),
    );
    assert_eq!(failure.poll_result, 0);
    assert!(failure.post_poll_ns.unwrap() >= failure.poll_started_ns);
    assert!(failure.prior_status_known && failure.prior_status_raw.is_none());
    // Follow the original cleanup order: close the actual stdin first, allow
    // the original two-second EOF phase, then escalate only if still needed.
    drop(child.0.stdin.take());
    let settlement = settle_runner(&mut child.0);
    child.1 = settlement.actual_status_raw.is_some();
    assert_eq!(settlement.actual_exit_code, Some(0));
    assert_eq!(settlement.phase, "eof");
    assert!(settlement.term.is_none() && settlement.kill.is_none());
    assert_eq!(settlement.observation_errors, 0);
    assert_eq!(
        child.0.try_wait().unwrap().unwrap().into_raw(),
        settlement.actual_status_raw.unwrap()
    );
    let mut captured = Vec::new();
    child
        .0
        .stderr
        .take()
        .unwrap()
        .take(64 * 1024 + 1)
        .read_to_end(&mut captured)
        .unwrap();
    assert!(captured.len() <= 64 * 1024);
    let record = serde_json::json!({"kind":"owned_fixture_observation_control", "pid":pid, "hello":failure, "settlement":settlement, "qualifying_readiness":false, "stderr_captured_bytes":captured.len(), "stderr_eof":true, "cpu_accounting":"not_measured_by_failure_evidence"});
    let receipt = retain_read_failure(&stderr, pid, &captured, record).unwrap();
    let unwind = std::panic::catch_unwind(move || {
        let _owned_home = home;
        panic!("source-known failure-home retention control");
    });
    assert!(unwind.is_err());
    assert!(root.is_dir(), "panic erased the owned failure HOME");
    assert_eq!(std::fs::metadata(&root).unwrap().mode() & 0o777, 0o700);
    for path in [&stderr, &receipt] {
        assert_eq!(std::fs::metadata(path).unwrap().mode() & 0o777, 0o600);
    }
    let bytes = std::fs::read(&receipt).unwrap();
    assert!(bytes.len() <= 8192);
    let saved: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(saved["pid"], pid);
    assert_eq!(saved["settlement"]["actual_status_raw"], 0);
    assert_eq!(saved["stderr_persisted"], true);
    eprintln!(
        "read-private-parent owned_failure_control pid={pid} receipt={} actual_status_raw=0 phase=eof term_attempted=false kill_attempted=false preserved_home={} receipt_bytes={} qualifying_readiness=false",
        receipt.display(),
        root.display(),
        bytes.len()
    );
}

#[test]
#[ignore = "load-sensitive budget/wall window; re-enable via tasks/2026-11-06-qos-builds-never-starve-mcp"]
fn read_bootstrap_held_birth_has_zero_admission_then_paid_hello_start_and_actual_reap() {
    let job = Job::empty();
    let actual_pid = std::cell::Cell::new(0u32);
    let mut worker = Runner::launch(&job, false, |held, original_readiness| {
        actual_pid.set(held.child.id());
        assert!(held.child.try_wait().unwrap().is_none());
        assert!(held.lease.is_some());
        assert!(exact_children(held.child.id() as i32).is_empty());
        let marker = bootstrap_waiting_marker(&held.stderr_bytes, boot::monotonic_ns().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(
            marker.trim(),
            "dream-bootstrap state=waiting profile=dream-bootstrap/1 argv_admissions=0 channel_admissions=0 owner_admissions=0"
        );
        let mut stdout = libc::pollfd {
            fd: held.child.stdout.as_ref().unwrap().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(
            unsafe { libc::poll(&mut stdout, 1, 0) },
            0,
            "ungranted read worker emitted Hello"
        );
        thread::sleep(Duration::from_millis(15));
        assert!(held.child.try_wait().unwrap().is_none());
        assert!(boot::monotonic_ns().unwrap() < original_readiness);
        assert!(!job.path.join("output/result.json").exists());
    });
    assert_eq!(
        worker.child.id(),
        actual_pid.get(),
        "grant replaced or retried actual held birth"
    );
    assert!(worker.lease.is_none());
    worker.send(&w::WorkerIn::Start(job.start.clone()));
    let (done, exit, metrics) = worker.finish(&job);
    assert_eq!(exit, 0, "{done:?} {metrics}");
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {metrics}");
    assert!(job.path.join("output/result.json").is_file());
    assert!(metrics.contains("dream-bootstrap state=paid_hello profile=dream-bootstrap/1"));
    assert!(metrics.contains("cutoff=joined_final_control_pre_telemetry"));
    assert_memory_gate(&metrics);
    eprintln!(
        "read-bootstrap-held actual_reaped_pid={} original_readiness_not_reset=true {metrics}",
        actual_pid.get()
    );
}

// Retained failure measured1353291ns in parent_paid before permit; hello:null,
// entry/waiting then revoked, original PID37477 actually reaped2. The precise
// parent callsite and cause of that measured CPU overrun were not retained.
#[test]
#[ignore = "load-sensitive budget/wall window; re-enable via tasks/2026-11-06-qos-builds-never-starve-mcp"]
fn read_bootstrap_probe_null_context_permanently_refuses_matching_start_before_source() {
    let job = Job::empty();
    let before = std::fs::read(job.path.join("input/manifest.json")).unwrap();
    let mut probe = Runner::launch(&job, true, |held, original_readiness| {
        assert!(!held.launch_path.exists());
        assert!(held.child.try_wait().unwrap().is_none());
        assert!(boot::monotonic_ns().unwrap() < original_readiness);
    });
    let binding: serde_json::Value = serde_json::from_slice(&probe.binding_json).unwrap();
    assert_eq!(binding["purpose"], "read-prepare");
    assert_eq!(binding["probe"], true);
    assert!(
        binding
            .get("context")
            .is_some_and(serde_json::Value::is_null)
    );
    assert_ne!(probe.run_id, job.start.run_id);
    assert_ne!(probe.nonce, job.start.nonce);
    let actual_pid = probe.child.id();
    let mut hostile = job.start.clone();
    hostile.run_id = probe.run_id.clone();
    hostile.nonce = probe.nonce.clone();
    // Deliberately adversarial transport: even matching approved probe IDs can
    // never authorize Start. No public-parent integration claim is made here.
    probe.send(&w::WorkerIn::Start(hostile));
    let metrics = probe.finish_probe_refusal();
    assert_eq!(
        std::fs::read(job.path.join("input/manifest.json")).unwrap(),
        before
    );
    assert!(!job.path.join("output/result.json").exists());
    eprintln!(
        "read-bootstrap-probe actual_reaped_pid={actual_pid} matching_start_refused=true context_explicit_null=true {metrics}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires separately qualified proof tools; re-enable via tasks/2026-11-05-dream-proof-tools-provisioner"]
async fn qualifying_four_jobs_complete_real_proofs_with_foreground_debt_memory_and_reap() {
    let templates = [
        Template::publication_root("fixture.invalid/dream/repo-a").await,
        Template::publication_root("fixture.invalid/dream/repo-b").await,
    ];
    for template in &templates {
        template.hide_originals();
    }
    let jobs = (0..4)
        .map(|index| {
            let mut job = templates[index / 2].job();
            job.start.foreground = true;
            job
        })
        .collect::<Vec<_>>();
    assert!(jobs.iter().all(|j| !j.input.records.is_empty()
        && !j.input.publications.is_empty()
        && j.provision.is_some()));
    let mut root_counts = BTreeMap::<&str, usize>::new();
    for job in &jobs {
        assert_eq!(job.input.context.account, jobs[0].input.context.account);
        assert_eq!(job.captured_context.root, job.input.context.root);
        assert!(
            job.input
                .records
                .iter()
                .all(|pin| pin.root == job.input.context.root)
        );
        *root_counts
            .entry(job.input.context.root.0.as_str())
            .or_default() += 1;
    }
    assert_eq!(
        root_counts.len(),
        2,
        "four qualifying jobs require two genuinely signed roots"
    );
    assert!(root_counts.values().all(|count| *count == 2));
    let baseline = parent_live_heap();
    let mut workers = jobs
        .iter()
        .map(|job| Some(Runner::start(job)))
        .collect::<Vec<_>>();
    for (worker, job) in workers.iter_mut().zip(&jobs) {
        worker
            .as_mut()
            .unwrap()
            .send(&w::WorkerIn::Start(job.start.clone()));
    }
    for worker in &mut workers {
        loop {
            if matches!(
                worker.as_mut().unwrap().next(Duration::from_secs(2)),
                w::WorkerOut::Progress(w::Progress {
                    phase: w::Phase::Paused,
                    ..
                })
            ) {
                break;
            }
        }
    }
    assert!(
        workers
            .iter()
            .all(|w| exact_children(w.as_ref().unwrap().child.id() as i32).is_empty()),
        "foreground prevents first helper"
    );
    for (worker, job) in workers.iter_mut().zip(&jobs) {
        worker
            .as_mut()
            .unwrap()
            .send(&w::WorkerIn::Activity(w::Activity {
                protocol: w::Protocol::V1,
                run_id: job.start.run_id.clone(),
                nonce: job.start.nonce.clone(),
                revision: w::Number(2),
                foreground: false,
            }));
    }
    // Observe the actual provisioned Git/GPG pair before applying foreground.
    let until = Instant::now() + Duration::from_secs(15);
    let observed = loop {
        if let Some(group) = worker_group(
            workers[0].as_ref().unwrap().child.id() as i32,
            &templates[0].provision,
        ) && group.len() == 2
            && group[1].image == templates[0].provision.gpg
            && group[1].cpu > 0
        {
            break group;
        }
        assert!(
            Instant::now() < until,
            "qualifying worker never reached an observed Git/GPG proof pair"
        );
        thread::sleep(Duration::from_micros(100));
    };
    let at = Instant::now();
    workers[0]
        .as_mut()
        .unwrap()
        .send(&w::WorkerIn::Activity(w::Activity {
            protocol: w::Protocol::V1,
            run_id: jobs[0].start.run_id.clone(),
            nonce: jobs[0].start.nonce.clone(),
            revision: w::Number(3),
            foreground: true,
        }));
    let paused = loop {
        match worker_group(
            workers[0].as_ref().unwrap().child.id() as i32,
            &templates[0].provision,
        ) {
            None => break None,
            Some(group)
                if group
                    .iter()
                    .all(|p| p.status == 4 && p.threads == 1 && p.runnable == 0) =>
            {
                break Some(group);
            }
            _ => {
                assert!(
                    at.elapsed() < Duration::from_millis(100),
                    "actual helper foreground stop exceeded100ms"
                );
                thread::sleep(Duration::from_micros(100));
            }
        }
    };
    assert!(at.elapsed() < Duration::from_millis(100));
    thread::sleep(Duration::from_millis(20));
    if let Some(paused) = paused {
        for prior in paused {
            if let Some(now) = process_sample(prior.pid) {
                assert_eq!(now.birth, prior.birth);
                assert_eq!(now.pgid, prior.pgid);
                assert_eq!(now.status, 4);
                assert_eq!(now.threads, 1);
                assert_eq!(now.runnable, 0);
                assert_eq!(
                    now.user_cpu, prior.user_cpu,
                    "foreground must stop actual target user work"
                );
                assert!(
                    now.cpu >= prior.cpu,
                    "kernel CPU accounting stays monotonic"
                );
                eprintln!(
                    "foreground_stopped_helper pid={} birth={} held_user_delta_ns=0 late_kernel_delta_ns={}",
                    now.pid,
                    now.birth,
                    now.cpu - prior.cpu
                );
            }
        }
    }
    if let Some(group) = worker_group(
        workers[0].as_ref().unwrap().child.id() as i32,
        &templates[0].provision,
    ) {
        assert!(
            group
                .iter()
                .all(|p| p.status == 4 && p.threads == 1 && p.runnable == 0),
            "no target can execute under the foreground fence"
        );
    }
    assert!(
        !jobs[0].path.join("output/result.json").exists(),
        "foreground cannot certify a new result"
    );
    workers[0]
        .as_mut()
        .unwrap()
        .send(&w::WorkerIn::Activity(w::Activity {
            protocol: w::Protocol::V1,
            run_id: jobs[0].start.run_id.clone(),
            nonce: jobs[0].start.nonce.clone(),
            revision: w::Number(4),
            foreground: false,
        }));
    let sampled = parent_live_heap();
    assert!(
        sampled.saturating_sub(baseline) <= 32 * 1024 * 1024,
        "private fixture parent retained share"
    );
    for (index, (worker, job)) in workers.iter_mut().zip(&jobs).enumerate() {
        let (done, exit, metrics) = worker.take().unwrap().finish(job);
        assert_eq!(exit, 0, "{done:?} {metrics}");
        assert_qualified_snapshot(job, &done, &templates[index / 2].entry);
        assert_memory_gate(&metrics);
        assert_proof_helper_gates(&metrics);
        eprintln!("qualified-four-job run={} {metrics}", job.start.run_id.0);
    }
    for prior in observed {
        assert!(
            process_sample(prior.pid).is_none_or(|now| now.birth != prior.birth),
            "observed helper identity must actually disappear after exit"
        );
        assert_eq!(unsafe { libc::kill(-prior.pgid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires separately qualified proof tools; re-enable via tasks/2026-11-05-dream-proof-tools-provisioner"]
async fn qualifying_helper_foreground_cancel_settles_actual_descendant_before_done() {
    let template = Template::publication().await;
    template.hide_originals();
    let job = template.job();
    let mut worker = Runner::start(&job);
    worker.send(&w::WorkerIn::Start(job.start.clone()));
    let until = Instant::now() + Duration::from_secs(15);
    let observed = loop {
        if let Some(group) = worker_group(worker.child.id() as i32, &template.provision)
            && group.len() == 2
            && group[1].image == template.provision.gpg
            && group[1].cpu > 0
        {
            break group;
        }
        assert!(
            Instant::now() < until,
            "qualifying proof never reached real GPG descendant"
        );
        thread::sleep(Duration::from_micros(100));
    };
    let at = Instant::now();
    worker.send(&w::WorkerIn::Activity(w::Activity {
        protocol: w::Protocol::V1,
        run_id: job.start.run_id.clone(),
        nonce: job.start.nonce.clone(),
        revision: w::Number(2),
        foreground: true,
    }));
    loop {
        if worker_group(worker.child.id() as i32, &template.provision).is_none_or(|group| {
            group
                .iter()
                .all(|p| p.status == 4 && p.threads == 1 && p.runnable == 0)
        }) {
            break;
        }
        assert!(
            at.elapsed() < Duration::from_millis(100),
            "helper foreground fence exceeded100ms"
        );
        thread::sleep(Duration::from_micros(100));
    }
    let cancel_at = Instant::now();
    worker.send(&w::WorkerIn::Cancel(w::Cancel {
        protocol: w::Protocol::V1,
        run_id: job.start.run_id.clone(),
        nonce: job.start.nonce.clone(),
    }));
    let (done, exit, metrics) = worker.finish(&job);
    assert_eq!(done.outcome, w::Outcome::Cancelled, "{done:?} {metrics}");
    assert_eq!(exit, 0, "{metrics}");
    assert!(
        cancel_at.elapsed() < Duration::from_millis(250),
        "actual helper cancel plus settlement exceeded250ms: {metrics}"
    );
    assert_memory_gate(&metrics);
    let lifetime = helper_record(&metrics, "helper_lifetime");
    assert!(lifetime.contains("coverage_known=true"), "{lifetime}");
    assert!(metric(lifetime, "settled") > 0 && metric(lifetime, "observed_descendant_cpu_ns") > 0);
    assert!(metric(lifetime, "max_charge_ns") <= 1_000_000, "{lifetime}");
    assert!(done.result_sha256.is_none() && done.pages == 0);
    assert!(!job.path.join("output/result.json").exists());
    for prior in observed {
        assert!(
            process_sample(prior.pid).is_none_or(|now| now.birth != prior.birth),
            "Done preceded actual observed helper disappearance"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires separately qualified proof tools; re-enable via tasks/2026-11-05-dream-proof-tools-provisioner"]
async fn helper_provision_capability_and_executable_guards_refuse_without_path_fallback() {
    let template = Template::publication().await;
    template.hide_originals();
    let positive = template.job();
    let (done, exit, metrics) = run(&positive);
    assert_eq!(exit, 0, "{done:?} {metrics}");
    assert_qualified_snapshot(&positive, &done, &template.entry);
    assert_proof_helper_gates(&metrics);
    let variants: [(&str, Box<dyn Fn(&mut serde_json::Value)>, bool); 13] = [
        (
            "receipt-bytes",
            Box::new(|v| v["git"]["build"] = "changed bytes".into()),
            false,
        ),
        (
            "source-hash",
            Box::new(|v| v["source_sha256"] = "0".repeat(64).into()),
            true,
        ),
        (
            "probe-source-hash",
            Box::new(|v| v["capability"]["probe_source_sha256"] = "0".repeat(64).into()),
            true,
        ),
        (
            "native-capability",
            Box::new(|v| {
                v["capability"]["features"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|f| f != "cloexec-default")
            }),
            true,
        ),
        (
            "rollup-capability",
            Box::new(|v| {
                v["capability"]["features"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|f| f != "descendant-cpu-rollup")
            }),
            true,
        ),
        (
            "single-thread-capability",
            Box::new(|v| {
                v["capability"]["features"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|f| f != "single-threaded-helpers")
            }),
            true,
        ),
        (
            "fork-exec-capability",
            Box::new(|v| {
                v["capability"]["features"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|f| f != "git-fork-before-gpg-exec")
            }),
            true,
        ),
        (
            "cpu-unit-capability",
            Box::new(|v| {
                v["capability"]["features"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|f| f != "mach-timebase-cpu-ns")
            }),
            true,
        ),
        (
            "os-release",
            Box::new(|v| v["os"]["release"] = "unobserved-host".into()),
            true,
        ),
        (
            "image-hash",
            Box::new(|v| v["gpg"]["sha256"] = "0".repeat(64).into()),
            true,
        ),
        (
            "relative-image",
            Box::new(|v| v["git"]["absolute_path"] = "git".into()),
            true,
        ),
        (
            "missing-build",
            Box::new(|v| v["git"]["build"] = "".into()),
            true,
        ),
        (
            "tool-capability",
            Box::new(|v| v["gpg"]["features"] = serde_json::json!(["public-key-stdin"])),
            true,
        ),
    ];
    for (name, edit, digest) in variants {
        let mut job = template.job();
        job.provision = Some(template.provision.altered(edit, digest));
        let code = assert_refused(&job);
        assert_eq!(code, w::Code::VerificationUnproven, "{name}");
        let stderr = std::fs::read_to_string(job.path.join("worker.stderr")).unwrap();
        if stderr
            .lines()
            .any(|line| line.starts_with("dream-read helper_native "))
        {
            assert_eq!(
                metric(helper_record(&stderr, "helper_native"), "effects"),
                0,
                "invalid readiness cannot create a helper: {name}"
            );
        }
        eprintln!("helper-provision-refusal kind={name} code={code:?}");
    }
    let mut missing = template.job();
    missing.provision = None;
    assert_eq!(assert_refused(&missing), w::Code::VerificationUnproven);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires separately qualified proof tools; re-enable via tasks/2026-11-05-dream-proof-tools-provisioner"]
async fn signed_publication_exact_bundle_survives_offline_sources_and_refuses_tamper() {
    let template = Template::publication().await;
    let revived = template.changed_committed_record(true);
    let terminal = template.changed_committed_record(false);
    template.hide_originals();
    let job = template.job();
    let (done, exit, stderr) = run(&job);
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {stderr}");
    assert_eq!(exit, 0, "{stderr}");
    assert!(done.error.is_none());
    let descriptor = std::fs::read(job.path.join("output/result.json")).unwrap();
    assert_eq!(done.result_sha256, Some(w::hash(&descriptor)));
    let snapshot: w::Snapshot = w::decode(&descriptor).unwrap();
    assert_eq!(snapshot.input_digest, job.start.input_sha256);
    assert_eq!(
        snapshot.source_revision,
        w::source_revision(&job.input).unwrap()
    );
    assert_eq!(snapshot.entry_ids, vec![w::Id(template.entry.clone())]);
    assert_eq!(snapshot.coverage.indexed, 1);
    assert!(snapshot.coverage.complete);
    assert_eq!(snapshot.coverage.omitted, 0);
    let mut entries = vec![];
    for (index, page_ref) in snapshot.pages.iter().enumerate() {
        assert_eq!(page_ref.index, index as u32);
        let bytes = std::fs::read(job.path.join(format!("output/page-{index:06}.json"))).unwrap();
        assert_eq!(page_ref.sha256, w::hash(&bytes));
        assert_eq!(page_ref.len.0, bytes.len() as u64);
        let page: w::Page = w::decode(&bytes).unwrap();
        assert_eq!(page.run_id, job.start.run_id);
        assert_eq!(page.nonce, job.start.nonce);
        assert_eq!(page.index, index as u32);
        assert_eq!(
            page.next,
            if index + 1 < snapshot.pages.len() {
                Some(index as u32 + 1)
            } else {
                None
            }
        );
        entries.extend(page.entries);
    }
    assert_eq!(entries.len(), 1);
    let lesson = &entries[0];
    w::validate_entry(lesson).unwrap();
    assert_eq!(lesson.hit.title, "Run the private verifier");
    assert_eq!(lesson.why, "A checked source protects this fixture.");
    assert_eq!(lesson.how_to_apply, "Run the verifier before publication.");
    assert_eq!(lesson.hit.proof.publication, template.publication);
    assert_eq!(
        lesson.hit.proof.record,
        w::RecordPin::try_from(template.fixture.record.clone()).unwrap()
    );
    assert!(!lesson.context.is_empty());
    assert!(!lesson.nodes.is_empty());
    assert!(!lesson.citation.is_empty());
    assert_eq!(
        std::fs::read_dir(job.path.join("output")).unwrap().count(),
        snapshot.pages.len() + 1
    );
    for file in &job.input.evidence.files {
        let bytes = std::fs::read(job.path.join(&file.path.0)).unwrap();
        assert_eq!(bytes.len() as u64, file.len.0);
        assert_eq!(
            w::hash(&bytes),
            file.sha256,
            "exporter changed admitted input"
        );
    }

    let changed = template.job();
    let slot = changed.input.evidence.policy as usize;
    let path = changed
        .path
        .join(&changed.input.evidence.files[slot].path.0);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[0] = b'[';
    private_file(&path, &bytes);
    assert_eq!(assert_refused(&changed), w::Code::SourceChanged);

    let mut missing = template.job();
    missing.input.evidence.public_keys.clear();
    missing.rewrite_input();
    assert_eq!(assert_refused(&missing), w::Code::VerificationUnproven);
    let mut provider = template.job();
    provider.input.context.provider = w::Id("native".into());
    provider.rewrite_input();
    assert_eq!(assert_refused(&provider), w::Code::VerificationUnproven);
    let mut current = template.job();
    current
        .input
        .evidence
        .catalogue
        .retain(|p| p.path.0 != ".bboxy/memories/lesson.md");
    current.rewrite_input();
    assert_ne!(assert_refused(&current), w::Code::Cancelled);
    let mut wrong_record = template.job();
    wrong_record.input.records[0].digest = w::Digest(format!("sha256:{}", "0".repeat(64)));
    wrong_record.rewrite_input();
    assert_ne!(assert_refused(&wrong_record), w::Code::Cancelled);
    let mut artifact = template.job();
    artifact
        .input
        .evidence
        .artifacts
        .retain(|a| !a.logical.0.ends_with(".md"));
    artifact.rewrite_input();
    assert_ne!(assert_refused(&artifact), w::Code::Cancelled);

    // Rehashing admitted transport bytes cannot substitute for the independent
    // current-record, producer, trial, publication or constructive-seal checks.
    for (kind, poison) in [
        ("signed-distinct-revival", revived),
        ("signed-current-terminal", terminal),
    ] {
        let code = assert_refused(&poison);
        assert_ne!(code, w::Code::Cancelled);
        eprintln!("semantic-poison kind={kind} code={code:?}");
    }
    let mut producer = template.job();
    producer.changed_policy(|policy| policy["producers"] = serde_json::json!([]));
    let code = assert_refused(&producer);
    assert!(matches!(
        code,
        w::Code::ProducerUnapproved | w::Code::VerificationUnproven
    ));
    eprintln!("semantic-poison kind=producer-unapproved code={code:?}");
    let mut source = template.job();
    let session = source.input.closed_sources[0].session.clone();
    source.rehash_source(&session, |events| {
        events[0]["event"]["meta"]["vcs"]["repo"] = "foreign.invalid/repository".into()
    });
    assert_eq!(assert_refused(&source), w::Code::SourceMismatch);
    let mut trial = template.job();
    let trial_session = w::Ulid(r::sid(2).to_string());
    trial.rehash_source(&trial_session, |events| {
        let item = events
            .iter_mut()
            .find(|e| e["event"]["body"]["kind"] == "trial")
            .unwrap();
        item["event"]["body"]["verdict"] = "fail".into();
    });
    let code = assert_refused(&trial);
    assert_ne!(code, w::Code::Cancelled);
    eprintln!("semantic-poison kind=held-out-trial code={code:?}");
    let missing_trial = template.job().missing_source(&trial_session);
    let code = assert_refused(&missing_trial);
    assert_eq!(code, w::Code::VerificationUnproven);
    eprintln!("semantic-poison kind=missing-trial-custody code={code:?}");
    let outer = template.job().changed_outer(&template.publication.session);
    let code = assert_refused(&outer);
    assert_eq!(code, w::Code::VerificationFailed);
    eprintln!("semantic-poison kind=outer-seal code={code:?}");
    let mut publication = template.job();
    publication.rehash_source(&template.publication.session, |events| {
        events[(template.publication.seq.0 - 1) as usize]["event"]["body"]["outcome"] =
            "skipped".into()
    });
    assert_eq!(assert_refused(&publication), w::Code::VerificationFailed);
    let mut harness = template.job();
    harness.rehash_source(&template.publication.session, |events| {
        events[(template.publication.seq.0 - 1) as usize]["event"]["body"]["harness"] =
            "foreign-harness".into()
    });
    assert_eq!(assert_refused(&harness), w::Code::HarnessIncompatible);
    let mut state = template.job();
    let state_session = w::Ulid(r::sid(3).to_string());
    state.rehash_source(&state_session, |events| {
        let item = events
            .iter_mut()
            .find(|e| {
                e["event"]["body"]["kind"] == "memory_state"
                    && e["event"]["body"]["state"] == "adopted"
            })
            .unwrap();
        item["event"]["body"]["state"] = "expired".into();
    });
    let code = assert_refused(&state);
    assert_ne!(code, w::Code::Cancelled);
    eprintln!("semantic-poison kind=terminal-projection code={code:?}");
    let unlisted = template.job();
    let root = &unlisted.input.evidence.roots[0].dir.0;
    private_file(
        &unlisted.path.join(root).join(".git/refs/heads/unlisted"),
        format!("{}\n", unlisted.input.evidence.roots[0].main.0).as_bytes(),
    );
    let code = assert_refused(&unlisted);
    assert_ne!(code, w::Code::Cancelled);
    eprintln!("semantic-poison kind=unlisted-git-ref code={code:?}");
}

#[test]
#[ignore = "load-sensitive budget/wall window; re-enable via tasks/2026-11-06-qos-builds-never-starve-mcp"]
fn no_candidates_protocol_pause_cancel_and_descriptor_custody_are_separate() {
    let empty = Job::empty();
    let (done, exit, stderr) = run(&empty);
    assert_eq!(done.outcome, w::Outcome::Completed, "{done:?} {stderr}");
    assert_eq!(exit, 0, "{stderr}");
    let descriptor: w::Snapshot =
        w::decode(&std::fs::read(empty.path.join("output/result.json")).unwrap()).unwrap();
    assert_eq!(descriptor.coverage.indexed, 0);
    assert!(descriptor.entry_ids.is_empty());
    assert_eq!(descriptor.pages.len(), 1);
    let page: w::Page =
        w::decode(&std::fs::read(empty.path.join("output/page-000000.json")).unwrap()).unwrap();
    assert!(page.entries.is_empty());
    for generation in [true, false] {
        let mut invalid = Job::empty();
        if generation {
            invalid.input.context.generation = w::Number(0);
        } else {
            invalid.input.context.grant_epoch = w::Number(0);
        }
        invalid.rewrite_input();
        assert_eq!(assert_refused(&invalid), w::Code::BadRequest);
    }
    let mut paused = Job::empty();
    paused.start.foreground = true;
    let mut worker = Runner::start(&paused);
    worker.send(&w::WorkerIn::Start(paused.start.clone()));
    worker.send(&w::WorkerIn::Cancel(w::Cancel {
        protocol: w::Protocol::V1,
        run_id: paused.start.run_id.clone(),
        nonce: paused.start.nonce.clone(),
    }));
    let (done, exit, _) = worker.finish(&paused);
    assert_eq!(done.outcome, w::Outcome::Cancelled);
    assert_eq!(exit, 0);
    assert!(!paused.path.join("output/result.json").exists());
    let protocol = Job::empty();
    let mut worker = Runner::start(&protocol);
    let mut wrong = serde_json::to_value(w::WorkerIn::Start(protocol.start.clone())).unwrap();
    wrong["protocol"] = "bboxy.dream.c1/1".into();
    worker.send(&wrong);
    assert_eq!(worker.wait_exit(), 2);
    assert!(!protocol.path.join("output/result.json").exists());
    let immutable = Job::empty();
    private_file(
        &immutable.path.join("output/result.json"),
        b"host-owned immutable bytes",
    );
    let (done, exit, _) = run(&immutable);
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert_ne!(exit, 0);
    assert_eq!(
        std::fs::read(immutable.path.join("output/result.json")).unwrap(),
        b"host-owned immutable bytes"
    );
}

#[test]
fn selected_multi_mib_four_jobs_keep_control_and_measured_storage_limits() {
    let baseline = parent_live_heap();
    let jobs = (0..4)
        .map(|index| {
            let mut job = Job::large(48, 64 * 1024);
            job.start.foreground = true;
            job.rebind_empty_scripted_root(&format!("fixture-root-{}", index / 2));
            job
        })
        .collect::<Vec<_>>();
    let mut roots = BTreeMap::<&str, usize>::new();
    for job in &jobs {
        assert_eq!(job.input.context.account, jobs[0].input.context.account);
        assert_eq!(job.captured_context.root, job.input.context.root);
        *roots.entry(job.input.context.root.0.as_str()).or_default() += 1;
    }
    assert_eq!(roots.len(), 2);
    assert!(roots.values().all(|count| *count == 2));
    for job in &jobs {
        assert!(job.input.closed_sources[0].len.0 > 3 * 1024 * 1024);
    }
    let parent_sample = parent_live_heap();
    assert!(
        parent_sample.saturating_sub(baseline) <= 32 * 1024 * 1024,
        "fixture parent retained heap delta exceeds existing 32MiB share"
    );
    let mut workers = jobs
        .iter()
        .map(|job| Some(Runner::start(job)))
        .collect::<Vec<_>>();
    for (worker, job) in workers.iter_mut().zip(&jobs) {
        worker
            .as_mut()
            .unwrap()
            .send(&w::WorkerIn::Start(job.start.clone()));
    }
    for worker in &mut workers {
        loop {
            if matches!(
                worker.as_mut().unwrap().next(Duration::from_secs(2)),
                w::WorkerOut::Progress(w::Progress {
                    phase: w::Phase::Paused,
                    ..
                })
            ) {
                break;
            }
        }
    }
    workers[0]
        .as_mut()
        .unwrap()
        .send(&w::WorkerIn::Activity(w::Activity {
            protocol: w::Protocol::V1,
            run_id: jobs[0].start.run_id.clone(),
            nonce: jobs[0].start.nonce.clone(),
            revision: w::Number(2),
            foreground: false,
        }));
    let cancel_at = Instant::now();
    for (worker, job) in workers[1..].iter_mut().zip(&jobs[1..]) {
        worker
            .as_mut()
            .unwrap()
            .send(&w::WorkerIn::Cancel(w::Cancel {
                protocol: w::Protocol::V1,
                run_id: job.start.run_id.clone(),
                nonce: job.start.nonce.clone(),
            }));
    }
    for (worker, job) in workers[1..].iter_mut().zip(&jobs[1..]) {
        let (done, exit, metrics) = worker.take().unwrap().finish(job);
        assert_eq!(done.outcome, w::Outcome::Cancelled);
        assert_eq!(exit, 0);
        assert!(
            cancel_at.elapsed() < Duration::from_millis(500),
            "paused independent control exceeded existing 500ms gate"
        );
        assert_memory_gate(&metrics);
    }
    let (done, exit, metrics) = workers[0].take().unwrap().finish(&jobs[0]);
    assert_eq!(done.outcome, w::Outcome::Completed, "{metrics}");
    assert_eq!(exit, 0, "{metrics}");
    assert_memory_gate(&metrics);
    assert!(metric(&metrics, "bytes") >= jobs[0].input.closed_sources[0].len.0);
    assert!(
        metric(&metrics, "envelopes") >= 98,
        "multi-MiB source must be parsed, not only hashed"
    );
    eprintln!(
        "read-private-parent baseline_heap_bytes={baseline} sampled_retained_heap_bytes={parent_sample} native_overhead=measurement_only\n{metrics}"
    );
}

#[test]
fn selected_dense_owner_and_total_input_overflow_refuse_before_publication() {
    let mut dense = Job::large(0, 0);
    let bytes = std::fs::read(dense.source_path()).unwrap();
    let mut lines = bytes
        .split(|b| *b == b'\n')
        .filter(|b| !b.is_empty())
        .map(|b| serde_json::from_slice::<serde_json::Value>(b).unwrap())
        .collect::<Vec<_>>();
    lines[0]["ext"]["dense"] = serde_json::json!(vec![vec![0u8; 4096]; 10]);
    let bytes = lines
        .iter()
        .flat_map(|line| {
            let mut b = serde_json::to_vec(line).unwrap();
            b.push(b'\n');
            b
        })
        .collect::<Vec<_>>();
    dense.replace_source(&bytes);
    let (done, exit, metrics) = run(&dense);
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert_ne!(exit, 0);
    assert_eq!(
        done.error.unwrap().code,
        w::Code::ResourceLimit,
        "{metrics}"
    );
    assert_memory_gate(&metrics);
    assert!(!dense.path.join("output/result.json").exists());
    assert_eq!(std::fs::read(dense.source_path()).unwrap(), bytes);

    let mut overflow = Job::large(0, 0);
    let length = w::MAX_INPUT + 1;
    File::options()
        .write(true)
        .open(overflow.source_path())
        .unwrap()
        .set_len(length)
        .unwrap();
    overflow.input.closed_sources[0].len = w::Number(length);
    let source_path = format!("input/{}.jsonl", overflow.input.closed_sources[0].session.0);
    overflow
        .input
        .evidence
        .files
        .iter_mut()
        .find(|f| f.path.0 == source_path)
        .unwrap()
        .len = w::Number(length);
    overflow.rewrite_input();
    let (done, exit, metrics) = run(&overflow);
    assert_eq!(done.outcome, w::Outcome::Failed);
    assert_ne!(exit, 0);
    assert_eq!(done.error.unwrap().code, w::Code::ResourceLimit);
    assert_memory_gate(&metrics);
    assert_eq!(
        metric(&metrics, "bytes"),
        0,
        "total input overflow must precede source IO"
    );
    assert!(!overflow.path.join("output/result.json").exists());
    assert_eq!(
        std::fs::metadata(overflow.source_path()).unwrap().len(),
        length
    );
}

#[test]
fn stdout_backpressure_and_activity_flood_leave_cancel_independent() {
    let mut job = Job::large(48, 64 * 1024);
    job.start.foreground = true;
    let mut worker = Runner::start(&job);
    worker.send(&w::WorkerIn::Start(job.start.clone()));
    // The private reader's fixed 64-frame lane fills and then leaves the actual
    // child pipe unread. Control traffic still reaches the independent owner.
    for revision in 2..50002 {
        worker.send(&w::WorkerIn::Activity(w::Activity {
            protocol: w::Protocol::V1,
            run_id: job.start.run_id.clone(),
            nonce: job.start.nonce.clone(),
            revision: w::Number(revision),
            foreground: true,
        }));
    }
    let at = Instant::now();
    worker.send(&w::WorkerIn::Cancel(w::Cancel {
        protocol: w::Protocol::V1,
        run_id: job.start.run_id.clone(),
        nonce: job.start.nonce.clone(),
    }));
    let (done, exit, metrics) = worker.finish(&job);
    assert_eq!(done.outcome, w::Outcome::Cancelled, "{metrics}");
    assert_eq!(exit, 0, "{metrics}");
    assert!(
        at.elapsed() < Duration::from_millis(500),
        "blocked stdout must retain existing cancel deadline"
    );
    assert_memory_gate(&metrics);
    assert!(!job.path.join("output/result.json").exists());
}
