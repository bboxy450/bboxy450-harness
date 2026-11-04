---
type: decision
title: Reconcile shared KB checkout without discarding local history or records
status: accepted
links:
- to: '[[2026-11-03-swarm-dreaming-succession]]'
  rel: implements
id: 01M6J7VRWN0N5M1YMS3F6MNJXW
updated: 2026-11-04T02:57:58.515Z
created: 2026-11-04T02:57:58.515Z
number: 20261103135936
created_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
updated_by: seat:01M6J5YFATHYCK148JT0C4XGZ1
authorship:
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  kind: seat
  public_key: Oc7KBLmX2fSuTNz6+a4eyDWCf4djX4fk13BizgaL+gc=
  process_generation: '1'
provenance:
  digest: sha256:329afa1a2f160ddd741d1581448919c02ac2dc64fb1477385b82c8249873076f
  frontmatter_digest: sha256:b4ca98f7d7a7fdb6555cc20f825fc688b6bda3f05fa2ec5198702c79917ab4c7
  signer: seat:01M6J5YFATHYCK148JT0C4XGZ1
  sig: iXgvT0t84xXj7p7zvDYivIPpKq6qrNBU/lmIpiEiYbWB7DVxWxRYL/+LMjr1dXQxxXGuns8lXpnY7GI3gI/aBw==
  log: .bboxy/log/2026-11-04.jsonl#41085
---
# Shared checkout reconciliation

Date: 2026-11-03 (2026-11-04 UTC). Authority: succession task #5981, step 3. Nyx Copperwing, Oracle.

Evidence: initial git rev-list --left-right --count HEAD...origin/main = 14 / 257. All fourteen local commits had good machine-key signatures. Subsequent Oracle KB commits were also signed. git merge-tree --write-tree HEAD main from the current-origin publication worktree showed one conflict only: a report log called the identical key “Signing key” locally and “Signing key fingerprint” on origin. All other local-history content was already present on origin.

Choice: signed ancestry-only reconciliation merge 4c65451 in /Users/feral/mydev/bboxy450-harness-dream-land, keeping canonical origin bytes. git diff --exit-code HEAD^ HEAD exited 0; git verify-commit exited 0. Push advanced origin/main with no file-content change and retained all previous local commits as ancestors. This was a KB-history repair; no implementation was merged.

Before updating the shared checkout, archived its complete .bboxy directory at /Users/feral/mydev/bboxy-dream-reconcile-qitg76bg/bboxy-before.tar.gz. manifest.json records old head, target and sha256 values for 7 modified tracked records overlapping incoming changes and 58 untracked collisions. Untracked collisions also have individual copies under collisions/. Canonical origin records now occupy those paths; the previous local bytes remain in the archive. No cleanup of this backup is authorized here.

A scripted guard refused staged changes, non-.bboxy dirty collisions and any collision with Lumen’s four active spec paths. All guards passed. Only the seven overlapping tracked modifications were restored for the fast-forward; other dirty work stayed in place. git merge --ff-only 4c65451 exited 0. Immediately afterward HEAD...origin/main was 0 / 0; 267 pre-existing modified files and 17 untracked entries remained. Those unrelated dirty records are not reviewed or claimed by this task.

Alternatives: resetting main would discard ancestry; blanket cleanup would lose untracked records; merging generated dirty metadata would publish unrelated material. None was used. The retained backup is local and was not committed. Future recovery can inspect its manifest and restore exact saved bytes deliberately.
