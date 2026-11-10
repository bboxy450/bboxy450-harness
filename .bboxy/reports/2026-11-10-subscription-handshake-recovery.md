---
id: 01M732K87DA4X39ZFDBDB8W4J1
number: 1794325979373
type: report
title: Bounded subscription handshake and explicit reconnect failures
task: '[[tasks/2026-11-04-codex-trusted-handoff]]'
commits: []
author: seat:01M732K87DXDW1NJPK0D2CMM42
created: '2026-11-10T15:52:59.373791+00:00'
verify:
  command: cargo +1.99.1 test --offline --locked --no-default-features --test gui_bridge unix_bridge_
  exit_code: 0
---

Direct operator brief: investigate the subscription loader failure, push a quick fix to main, and skip most tests. This is standalone Codex execution in its own worktree, based on 20e52d0f937c7ed19abbf506564a3dd8a0c8cdd4. The task link supplies historical context, not a native assignment. The author identifies this execution, not a registered seat or seat signature.

Finding and limit: the screenshot wording comes from ClientError::Disconnected, so it does not establish a timeout. Passive, approver=false subscriptions to the current worker's alias and session sockets received Hello in 0.38 and 0.06 ms. No user operation was sent and no transcript or secrets were retained. The original transient failure was not reproduced. Source inspection found that only the final Hello read had a deadline; connect and subscription write could wait indefinitely. Subscription failures also reused uncertain-command/op-id instructions and hid the specific Hello timeout behind “invalid socket protocol.”

Changes:
- src/gateway/unix.rs:415 — one five-second deadline encloses connect, subscription write and Hello read. Cancelling drops the socket halves before the receive owner is spawned. Hello read errors preserve their diagnostic reason.
- src/frontends/gui/bridge.rs:267 — passive failures say reconnecting and preserve protocol/handshake reasons. The existing two-second retry loop remains; operation dispatch and same-op-id semantics are unchanged.
- tests/gui_bridge.rs:301,322 — missing endpoint notice and a real Unix listener that never sends Hello. The latter observes cancellation/EOF with no user operation, then a healthy server on the same address; the bridge reconnects, clears its error and has no turns.
- .bboxy/interfaces/gateway.md and verification/contracts.toml record the transport bound as implementation-only. Existing finite turn-delivery actions and models are unchanged; no additional formal proof is claimed.

Verification on this source: the unix_bridge_ filter passed three tests (including the existing off-thread dispatch/detach check), and arbitrary_seat_address_reconnects_across_aba_without_retargeting_intent passed one. Both commands exited 0. make check-licenses, rustfmt +1.99.1 --check on the three Rust files, git diff --check and TOML parsing exited 0. An initial checks.py licenses invocation exited 1 because that subcommand does not exist; the canonical make check-licenses passed. Exact commands, source and log hashes: .bboxy/runs/2026-11-10-subscription-handshake.json. Private logs: /var/folders/zd/mv_mr2d92p5_1r8yghzbs5vw0000gn/T/bboxy-subscription-fix-20261110-60pn6n1q.

Full CI, native/TLC/formal, broad smoke and visual checks were skipped under the operator's quick-fix exception; no make land was run. This is bounded recovery and honest diagnostics, not proof of the screenshot's original cause. No app build, installation, process restart, runtime/history mutation or working-seat interruption was performed. GUI 35862, Logic 36049 and Supervisor 84759 retain their recorded start times.

The containing signed commit is the source checkpoint. The Term pin is recorded separately in its repository. Only exact scoped paths are staged; shared primary checkouts remain untouched.
