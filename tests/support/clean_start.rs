// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
#![allow(dead_code, clippy::disallowed_methods)]
use std::{
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};
pub fn home() -> tempfile::TempDir {
    let home = tempfile::Builder::new()
        .prefix("bb-clean-start-")
        .tempdir_in("/tmp")
        .unwrap();
    std::fs::create_dir(home.path().join("bin")).unwrap();
    home
}
pub fn install(home: &Path, adapter: bool) {
    std::fs::write(home.join("bin/codex"), "#!/bin/sh\nexit 91\n").unwrap();
    std::fs::set_permissions(
        home.join("bin/codex"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    if adapter {
        let python = std::env::var("CLEAN_START_PYTHON").unwrap();
        let body = format!("#!{python}\n")
            + r#"
import json, os, sys
from pathlib import Path
with (Path(os.environ['HOME']) / 'adapter-argv').open('a') as f:
    f.write(json.dumps(sys.argv) + '\n')
for line in sys.stdin:
    m = json.loads(line)
    method, rid = m.get('method'), m.get('id')
    if method == 'initialize':
        result = {'protocolVersion': 1, 'agentCapabilities': {}, 'authMethods': []}
    elif method == 'session/new':
        result = {'sessionId': 'private-clean-start'}
    elif method == 'session/prompt':
        print(json.dumps({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':'private-clean-start','update':{'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':'PRIVATE-CLEAN-START'}}}}), flush=True)
        result = {'stopReason': 'end_turn'}
    else:
        continue
    print(json.dumps({'jsonrpc':'2.0','id':rid,'result':result}), flush=True)
"#;
        std::fs::write(home.join("bin/codex-acp"), body).unwrap();
        std::fs::set_permissions(
            home.join("bin/codex-acp"),
            std::fs::Permissions::from_mode(0o700),
        )
        .unwrap();
    }
}
pub fn command(program: &Path, home: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .env_clear()
        .env("HOME", home)
        .env("PATH", home.join("bin"))
        .env("BBOXY_REDUCED_MOTION", "1")
        .env("RUST_TEST_THREADS", "1")
        .current_dir(home);
    command
}
pub fn run(command: &mut Command) -> Output {
    // Reuse the bounded exact-child runner; no desktop or shared process access.
    super::onboarding_startup_fixture::run(command)
}
pub fn profile(home: &Path, name: &str) -> Vec<u8> {
    std::fs::read(
        home.join(".bboxy450-harness/profiles")
            .join(format!("{name}.md")),
    )
    .unwrap()
}
