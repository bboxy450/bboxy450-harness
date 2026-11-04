---
type: decision
title: Trial-only engine capability and exact lightweight verifier with native isolation fail-closed
links:
- to: '[[01M6M5H1XGSX52MZKX8YWKTC0Z]]'
  rel: task
  weight: 3
- to: '[[dream-log]]'
  rel: spec
  weight: 3
id: 01M6M6BM2PCXRYYKG9FR5RYAB1
updated: 2026-11-04T21:10:07.447Z
created: 2026-11-04T21:10:07.447Z
number: 20261103136594
created_by: seat:01M6KDR825XMADBA0A88NE4Y3V
updated_by: seat:01M6KDR825XMADBA0A88NE4Y3V
authorship:
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  kind: seat
  public_key: eL1Vp6l+B+jyTduqCTc5mfj6tqgD6/QY2QnbyrGInHM=
  process_generation: '1'
provenance:
  digest: sha256:05e0c574f388714b27fb9e3d784dff7a1ae0de267e2a340706fc808b13963278
  frontmatter_digest: sha256:6dfb7194ebe5f40a2beb984fc6f6fa122285dfe95bd286bfa168a7c4468cf42d
  signer: seat:01M6KDR825XMADBA0A88NE4Y3V
  sig: dFit1Fgc4zENR/FHqOnvWSia4H8+YFXxuw2IV3tShbZDEnjHIThmgx7kOWelQONYZA4voOyo3Tmdic+DywiKDw==
  log: .bboxy/log/2026-11-04.jsonl#46077
---
Oracle interface ruling for WP7 [[01M6M5H1XGSX52MZKX8YWKTC0Z]]. Owners: Nyx Copperwing (contract) and Executor01M6KDR90VJTRA1S722FZFECRZ (implementation). This authorizes the minimal positive engine seam proposed in blocker01M6M5QDET6VDDYJQ82TYZR4EA; no general OS sandbox or ordinary/Yolo policy rewrite.

Source read: policy.rs Action has only tool/effect/generation; engine/mod.rs:332 unconditionally makes Registry::new, :781 consults those labels, :298–310 selects and starts MCP servers, and :685–705 can reload profile policy. tools.rs:483/533 joins caller paths and :334 can execute arbitrary programs. LaunchOptions:50 is a trusted Rust launch structure, separate from profile/ACP data, already Clone/Default. These existing paths do not establish trial confinement. The proposed capability must enforce before effects, not be an asserted flag.

SUPPORTED INITIAL ADAPTER:
Ordinary bboxy ENGINE continuation with the same selected engine model/profile, private scratch fork and actual restricted builtins is the supported positive path. Configured inference transport remains the ordinary trusted Provider path and existing budget; denied network here means model tools/verifier effects, not falsely claiming that model HTTP transport needs no network. Development/tests use only ScriptedProvider/private identities and no live inference. Native/ACP harnesses, external tools and arbitrary verifiers remain Inconclusive BEFORE execution until an independently proved isolation adapter exists. No fallback from a native episode to an unrelated engine profile, no forged isolation/pass/adoption, and no per-job operator approval. This is explicit initial support with safe degradation, not an assertion that every profile is isolated.

1. TRUSTED CAPABILITY / LAUNCH
Add an optional opaque trial capability to trusted LaunchOptions; ordinary/default is None. The capability is minted only by repository-owned trial orchestration after creating a private owned scratch root/copy. It owns or shares a pinned directory descriptor and root identity; fields/constructors are not supplied by profile JSON, MCP/ACP, session records or model data. A public opaque type/Option field with restricted constructor is acceptable to preserve existing external LaunchOptions struct-update usage; it must not be Deserialize/Default-forged. Do not add a new Config/SessionMeta field or broad mechanical constructor sweep.

Engine initialization validates the trusted capability against the intended scratch cwd and private trial launch before VCS/Store/MCP effects. The orchestrator supplies private profiles, sessions, HOME and fork custody roots; a capability cannot authorize arbitrary sessions/profile roots from untrusted data. Invalid/mismatched launch is refused/Inconclusive. Trial orchestration never falls back to LaunchOptions None for an effectful arm. Ordinary read-only replay stays compatible.

Select the restricted Registry only for Some(capability); ordinary Registry::new, ordinary/Yolo behavior and existing profile/ACP launch behavior remain unchanged. No launch.yolo override in trial: reject a conflicting request or enforce trial denial before it can widen anything. Trial capability remains an independent hard boundary even if an Allow Permit is forged or a policy rule says Allow. Pin the profile/context for the arm; profile/Yolo/allow-always/reload/MCP updates must not widen trial effects. Refuse unsupported trial mutations rather than reinitializing an unrestricted Registry.

2. NO MCP / EXEC STARTUP OR TOOL FALLBACK
Before Manager::start, trial mode must ensure NO MCP servers can start from profile, override or launcher sources. Reject configured trial MCP/unsupported profiles or construct the existing manager with a proved empty server set; never start then hide the tools. Restricted with_mcp/rebinding cannot attach/expose/call MCP later. Deny exec, bash, terminals, network/install/publish/gh/git-push operations and any unrecognized tool at the actual Registry entry point BEFORE any binding, child spawn or unrestricted file_tool/exec branch. Advertise only supported confined builtins. No filtering merely after their effects and no reliance on prompts/effect labels.

Automatic native/bboxy Recall is off in both arms; do not load live operator instruction/memory/profile data. Pin unrelated context identically and keep the candidate in quoted user/reference context. Disable automatic/manual context/profile transformations that would invalidate that pairing; unsupported transformations are refused. Scope here is only these trial gates, not ordinary compaction/profile behavior.

3. CONFINED FILE OPERATIONS
Allow only actually implemented confined read/write/edit/apply_patch (a smaller advertised set is acceptable if honest). File operations use the pinned root descriptor and component-relative opens, not a canonicalize-then-cwd.join fallback. Reject absolute paths, parent traversal, NUL/invalid components, symlinks at EVERY traversed component/leaf and administrative .git paths. An explicit root-directory read may use the root fd without traversal. Never let an untrusted Context.cwd redirect the capability.

For regular-file reads/edits/writes require actual descriptor fstat/type and single-link identity; hardlinked files are refused. Directory link counts are not regular-file link counts. Use no-follow/nonblocking/close-on-exec flags and reject FIFO/device/socket inputs so a metadata check cannot introduce an unbounded open. Open existing targets without destructive truncation, validate the opened inode/link/type, then modify through that same descriptor. Create new files exclusively relative to a checked parent descriptor; no path reopen after validation, no destructive check-then-open. Validate/stage every patch target before beginning its checked writes; no partial escape by a second unchecked edits element. Reuse cancellation/generation fencing and bounded tool output/IO rules; failure remains a failed/inconclusive arm, never a passing blank result.

Keep all trial filesystem effects serialized within that capability where needed for component/patch consistency. This is local ownership, not a machine-global lock/queue. All arm tools finish/cancel/join before verification and scratch cleanup. Trusted private scratch has no other writers; model tools cannot create links, rename roots/administrative directories, spawn a process or add a mount. Do not claim this in-process capability is an OS sandbox against malicious native Rust/provider code, arbitrary same-user external directory relocation/mount injection, or deliberate external writers. Unsupported platforms/filesystems/adapters refuse. Test symlink substitution between component resolution and use using controlled private hooks; do not claim unrestricted pathname race immunity from one canonical check.

Prevent secrets/instruction escapes according to the existing common deny/redactor rules before inference/recorded derived content. Do not acquire operator credentials, Keychain, live memory or seat capability in an arm. Provider implementations are trusted program code; the capability restricts exposed tool effects, not arbitrary code inside a malicious Provider implementation.

Primary API background read: Apple open(2) https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/open.2.html documents no-follow/exclusive/nonblocking opens; stat(2) https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/stat.2.html documents descriptor metadata including type and link count. The stronger multi-component confinement is THIS application contract plus actual race/escape witnesses, not a claim that O_NOFOLLOW on one pathname proves it.

4. FIRST SAFE LIGHTWEIGHT VERIFIER
The initial supported verifier is EXACTLY the pinned raw task.verify string:
sh -c 'test -f fixed'
No whitespace normalization that broadens this grammar, shell interpolation from model data, extra commands, arguments, arbitrary filename, repo scripts, PATH binary or general shell support. Require the task's original exact command/digest/pin; do not replace an unsupported task command and pretend it ran.

Run this exact predicate through a fixed trusted launcher/argv (absolute system sh and controlled PATH), with private cwd/HOME/TMPDIR and a per-child allowlisted/cleared environment; no global environment mutation or inherited shell startup/credential/function variables. Existing owned-process API/absolute deadlines/cancellation/reaping stay intact. A fixed /usr/bin/env -i wrapper is permitted if no existing per-child environment seam exists; this does not authorize changing src/process.rs.

Before invocation, use the same capability to establish fixed is absent or a safe single-link regular file under the root, with no model/other arm writers active. Missing fixed is a legitimate verifier failure; unsafe path/link/type or unsupported setup is Inconclusive, not success. The exact predicate performs no arbitrary repo code, writes, network, install, signing or credential action. Retain actual exit and bounded sanitized output/usage evidence; no invented stored exit/boolean substitute. All other commands, including make/cargo/gh/custom scripts, are unsupported/Inconclusive in this initial adapter. Do not spawn them, invent a heavy-resource claim or request an operator approval.

5. VERIFICATION / MODELS
At least one complete real ScriptedProvider→ordinary engine fork/continuation→restricted write of fixed→actual safe verifier positive path must run in private scratch; also without-memory and held-out/control/scoring/usage/deadline cases remain required. Source repository and external sentinels/records remain byte-identical.

Actual negative API/process witnesses: ../ and absolute paths; nested/leaf symlinks; hardlink to an external PRIVATE sentinel; FIFO/device refusal; context cwd mismatch; multiple patch targets with an escape; stale/cancelled generation; forged Allow Permit; forged profile/session/MCP trial flags; configured MCP startup trap creating a sentinel (trap must never run); exec/bash/network/unknown tool requests; policy/profile widening; no operator/global HOME changes. Ordinary Registry/Yolo tests remain green. Every child/hook/sentinel belongs to private fixtures; no desktop/Keychain/user credential test.

Model BEFORE stateful capability integration: extend only WP7's new trial capability/claim/isolation families under its already granted formal paths; retain existing147 families unchanged. State root identities, operands/link kinds, owner/generation/cancellation and pending/validated/effect/cleanup phases, including substitution/delayed completion. Pair no-outside-read/write, no-exec/MCP and no-forged/widened-capability properties with reachable broken controls and actual counterpart witnesses. Abstract fd/syscall semantics honestly; finite model membership is not OS confinement proof. Eligibility/scoring pure functions keep specialized tests; no changing thresholds@2, timeout/spend/score numbers.

6. PATH AMENDMENT / HANDOFF
Add src/tools.rs only for the opaque trial capability, restricted Registry/descriptor operations and its private tests; add src/engine/mod.rs only for trusted LaunchOptions→restricted Registry selection, early validation/MCP prevention, pinning/non-widening and the minimal private tests. Existing allowed headless/policy/trial/tests/model paths cover continuation/witnesses. No broad Config/ordinary policy/tool rewrite, new dependency, ACP backend, MCP implementation, process/Store/receipt/schema edit. Existing LaunchOptions literals mostly use Default; any actual extra compiler-driven path needs a named amendment, not a sweep.

Exact verify becomes: cargo test --lib tools:: && cargo test --test dream_pipeline trial && cargo test --test policy_gate && make test-formal && make test-formal-parity && make check-formal-evidence.
Also source-bound full default/ACP/canonical isolated process checks, all-feature clippy, IR/golden and unattended native/check-transcript smoke after shared hooks. Changed verify requires a fresh signed report/task_report. Supported Engine/safe-verifier subset and unsupported native/general commands must be reported explicitly; unsupported cases never create Pass/Adopted. No live policy/bootstrap, inference spend during tests, main merge or canonical KB5974 waiver.