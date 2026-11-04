---
type: decision
title: Finish verified dreaming work, update the term harness gitlink, then shut down completed seats
status: accepted
links:
- to: '[[2026-11-03-swarm-dreaming-succession]]'
  rel: extends
id: 01M6J9W2X9P1B2RZ7BPSY0XRCZ
updated: 2026-11-04T03:33:03.742Z
created: 2026-11-04T03:33:03.742Z
number: 20261103136267
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:3cbcb36bc1498146b8b5a261fa0e40e451147528ac5489edf93c7dced3eef277
  frontmatter_digest: sha256:f2840dcc9ef1b578efb4dc20dcf14aa4ea828005d1138eeb3a448125366719fd
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: w2EhmC3HHpFlQXvV+YL+CnpeS/UDMvtpinOybeyKaZfHJkfI5UoFRe33xv8Ddf2jcCe7G4Yk3CROEMRbZGU8Cw==
  log: .bboxy/log/2026-11-04.jsonl#42766
---
# Completion and shutdown order

Operator instruction received 2026-11-03 (2026-11-04 UTC): once everything is implemented and accepted, shut down the swarm and close completed seats; the final push may update bboxy450-term's harness submodule to the latest harness so term builds it.

Order: implement the existing WP1–9 scope; independently verify every acceptance requirement (including unit/integration/smoke, transcript laws, native and reference-TLC formal evidence); publish the accepted harness revision; update only the term harness gitlink in an isolated Executor worktree under term's current instructions; verify the consumer builds/tests that exact harness revision; sign and push that final integration; close completed swarm seats and stop assigning work. The Oracle may remain idle after the final handoff. A pending or failed task never becomes accepted merely to reach shutdown.

The term gitlink target must be the latest accepted, published harness commit, pinned exactly in the report. This instruction authorizes that final submodule update/push; it does not request new crate releases or additional product scope. Preserve unrelated dirty work, branch refs, formal evidence and the cleanup archive. Do not shut down an active implementer or discard an incomplete result. Existing host numbering and companion-mirror failures remain explicit until resolved through supported tools.
