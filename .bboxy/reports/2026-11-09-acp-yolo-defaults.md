---
id: 01M70ZAHH7NAAAF0FB9WJ241MH
number: 155111007162891
type: report
title: Full-access ACP defaults and native OMP support
task: '[[tasks/2026-10-31-harness-acp-mcp-compaction]]'
commits: []
verify:
  command: make test-acp-unrestricted TARGET_DIR=/Users/feral/mydev/bboxy450-term/target
  exit_code: 0
author: seat:01M70ZAHH7P4B75NWMQRJ3SGSR
created: '2026-11-09T20:50:08.108435+00:00'
---
Direct operator request, 2026-11-09: full YOLO permissions for ACP harnesses in bboxy450-term, prioritizing Claude and Codex and including Pi and OMP; bypass CI and restart the UI. This is standalone Codex execution in its own worktree, based on harness ad6c8b7 and Term d3f4e3c3a. The task locator supplies historical ACP context, not a new native assignment. The author identifier labels this execution, not a registered native seat signature.

Changes:
- src/provider/profile.rs:112 defaults saved ACP profiles without a policy field to yolo. Explicit ask remains available; outer --yolo overrides it. Existing profile file bytes and notes are preserved.
- src/backend/acp_permissions.rs:12 supplies native full-access launch settings: Codex INITIAL_AGENT_MODE=agent-full-access; Gemini/OMP --yolo; Gemini sandbox false; Grok always-approve/sandbox off; OpenCode wildcard allow; Goose auto. Pi RPC tools already have no native approval gate. Public launch values are applied after secret environment resolution, avoiding false redaction of mode names.
- src/backend/acp_agent.rs:1735 opts Claude into allowDangerouslySkipPermissions, then waits for acknowledgement of advertised bypassPermissions/full-access modes before prompting. Config-only/grouped permission advertisements are supported, without treating model or Pi thinking selectors as permissions. Yolo prefers allow_once and can select an offered allow_always fallback, recording the actual grant. Cancellation remains cancelled.
- src/onboarding.rs:81 discovers OMP through its native omp --mode acp server. Setup copy now describes that supported path. No unsupported Claude/Codex CLI flags are injected.
- Existing interface/invariant text and formal source mappings describe the new default. The finite lifecycle actions/models are unchanged; argument/config parsing remains implementation coverage.

Verification:
- Canonical Term make test-acp-unrestricted, with the shared target selected through BBOXY_CARGO_TARGET_DIR: exit 0. Three permission unit checks, all 26 ACP process checks, and the exact private profile-discovery check passed. Log: /tmp/bboxy-acp-20261109-bicor4fc/acp-delivery-checks.log; SHA256 8d80ce06b2c902477f98997b5b78afcdd789098caf44ebcac1f057d0bc5a036a.
- Private real pi-acp and omp --mode acp --yolo initialize probes: exit 0, protocol v1 responses, private homes, zero prompts/tool executions. Evidence: /tmp/bboxy-acp-20261109-bicor4fc/pi-omp-initialize.json; SHA256 e267263bb335ef986291e1942e73582b8c89ff94928e0c43c528106b679845da. This does not claim paid/provider tool conformance for every adapter.
- Python fixture AST, verification TOML parsing and git diff --check: exit 0. Changed input hashes: /tmp/bboxy-acp-20261109-bicor4fc/delivery-inputs.json.

Broader exploratory checks and disposition:
- The original process run found two stale/fragile assertions. A captured private event log identified the legitimate builtin Python MCP starting/ready info notices; the text fixture now permits only those exact info messages and still rejects other notices. The quiet witness now emits gauges with scheduler headroom and verifies the full 200 ms quiet interval from the final gauge; its production timer is unchanged. The canonical process family runs serially. Both checks pass in the final 26-case run. Earlier failed logs remain private alongside the final log.
- The broader six-case onboarding run passed five and failed its native raster's conversation-scroll restoration assertion (f32::MAX versus 123), outside this permission/discovery change. It is reported, left untouched, and excluded from the scoped delivery claim. Full CI/formal/lint/visual suites were skipped by explicit operator instruction; no make land was run. Rust's existing missing-libLLVM debug-stripping warnings were nonfatal.

The screenshot text is emitted by the installed Claude ACP renderer at /opt/homebrew/lib/node_modules/@agentclientprotocol/claude-agent-acp/dist/tool-calls/renderer.js:484 for an unfinished streamed tool that never reached execution, including steering. Permission bypass cannot prevent every abandoned draft call, and no fake execution or retry is introduced.

This containing signed commit is the source checkpoint. Parent pinning, signed installation and UI activation are recorded in Term's delivery report. Existing agent processes retain their old executable/config until restarted; the requested UI restart preserves Supervisor and Logic.
