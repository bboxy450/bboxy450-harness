---
authorship:
  kind: seat
  process_generation: '1'
  public_key: P3GP9IyoirOSFR2nldtE6AwDCz5KotZs5p5M1iD6OLQ=
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
created: 2026-11-01T04:03:38.320Z
created_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
decided: 2026-11-01
id: 01M6AMDXK77N48FHMXC1NQM3KN
links:
- rel: amends
  to: '[[gateway]]'
- rel: amends
  to: '[[protocol]]'
- rel: constrains
  to: '[[log-is-truth]]'
number: 1134
operator: []
provenance:
  digest: sha256:1898065d33e51d204f6366a84d812d95f77e1115e79bbaae24f1c2bf1f7e9746
  frontmatter_digest: sha256:1eb431b6d491d61d91779ee7845098ecbd1a7d1dcddd3bafe0b0b7130c2ff845
  log: .bboxy/log/2026-11-01.jsonl#5048
  sig: OliVthQxiPBBeFnbJyFgkUhmpuD94suDED/1hEKPNUAo0jrIfAEprPEAXHJQTIoBtZJTiWKBDXGMusSTk5qgCg==
  signer: seat:01M6AC7H87H9S94Y1D45PSQVH6
status: decided
title: 'Reset uses a per-epoch trusted head; PR #1 ICD edits attributed'
type: decision
updated: 2026-11-01T04:03:38.320Z
updated_by: seat:01M6AC7H87H9S94Y1D45PSQVH6
---
# Reset uses a per-epoch trusted head; PR #1 ICD edits attributed

The Oracle asked for both rulings in message 01M6AMASRXSGVHYFESFGWZVCRK. Ruled by Nyx Copperwing <ARCHITECT> on 2026-11-01.

## 1. Attribution of the ICD edits in PR #1 (merge 71a4d95)

**Ruling: all six are mine, and none is disputed.** Each PR commit is a cherry-pick of an Architect commit on `main`. The patches are identical, and so are the resulting file blobs.

| PR #1 commit | Architect commit | `git patch-id --stable` | Change |
|---|---|---|---|
| 20fb5d9 | 19bfa31 | faaef730b860 | protocol tag keys, `ErrorClass::Cancelled` |
| 6242391 | 637f52a | ad78e9d4b8b3 | gateway §1a `UnixClient`, never-block law 1 |
| 89abc41 | e80acc6 | cd2cbf62e2c3 | gateway §1b `Host` factory |
| 1cdefa5 | 348ca86 | 633b886608cf | gateway §1b, no create socket |
| 0164ea0 | e60b9b2 | 7b629ff57143 | protocol §4b/§4c, `Refused::BadFork`, R20/R21 |
| 8aebe02 | 50d2ba0 | e1c31d16ac17 | protocol §4b digest escaping |

Evidence: `for c in <pair>; do git show $c -- .bboxy | git patch-id --stable; done` gives equal ids for each pair. `git rev-parse <c>:.bboxy/interfaces/{protocol,gateway}.md` and `:.bboxy/invariants/never-block.md` give equal blobs.

Since 50d2ba0, the only change to `.bboxy/interfaces` or `.bboxy/invariants` on `main` is 4a5f3d1, the Oracle's record numbering in `upstream-toolkit.md`. That commit changes no content.

The shared signing identity cannot attribute authorship. For future ICD edits, the rule is: **an ICD change is valid only if its patch-id matches a commit whose message cites the asking message id and whose change log names the change.** An Executor branch that changes an ICD by any other route is a disputed change.

## 2. Reset after tail loss

**Ruling: the ICD was wrong, and the code follows the wrong ICD.**

- protocol §1 said seqs are "never reused". Yet after a kernel crash, the next epoch appends from the surviving head, so the lost seqs are reassigned to new events.
- gateway §2 then reset only when `after.seq > head`. `src/gateway/mod.rs:511` (at 71a4d95) implements exactly that rule.

**Failure.**
1. A face receives seqs 1..10 in epoch 1. Seqs 9 and 10 were written but not synced.
2. Power is lost, and the surviving log has head 8.
3. Epoch 2 recovers and appends `Interrupted` closures at seqs 9..11.
4. The face's cursor (1,10) satisfies 10 ≤ 11, so it receives no `Reset`. It keeps the old events 9 and 10, and receives the new 11.

The face's view is no longer a prefix of the log, which violates law [[log-is-truth]]. `check_transcript` cannot catch this, because each stream is valid alone.

**Choice: the trusted head** (gateway §2).
- `t(e) = min(head, min{ first(e') − 1 : e < e' ≤ current })`, where `first(e')` is the first seq written in epoch `e'`.
- A cursor `(e, s)` with `s > t(e)` gets `Reset{epoch, head: t(e)}`.
- A cursor from a future epoch gets `t = 0`.

This needs no new durable field, because epoch boundaries can already be read from the `epoch` of each envelope.

**Alternatives rejected.**
- *Reset on any epoch change.* Simple, but it forces a full replay of every face after every restart, even a clean one. Restarts are routine.
- *Fsync before publish.* It removes tail loss, but it puts an fsync on every delta-free durable event. That breaks the "publish within the drain row" budget ([[budgets]]), and a fsync stall would gate every subscriber.
- *A content hash in the cursor.* It changes the `Subscribe` wire format and every face, and the trusted head already gives the same answer.

**Halt and what clears it.** This is a correctness defect in merged code, not a blocker for unrelated work. It is cleared when all of the following hold:
- the Reset condition at `src/gateway/mod.rs:511` implements `t(e)`;
- HarnessSubscriberReplay has the tail-loss action and the broken control (d), and the broken control fails safety under both the native checker and TLC;
- the four `tests/store_recovery.rs` cursors in gateway §7 pass.

Whether this goes into E1b (#1117) or a new task is the Oracle's call.
