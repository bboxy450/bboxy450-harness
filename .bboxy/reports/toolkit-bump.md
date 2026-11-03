---
id: "01M6G31HGNG53RBFN3XKF22VRD"
number: 20261103065513
type: report
title: "toolkit-bump — one term revision and native Mermaid layout"
created: "2026-11-02"
owner: "Codex Executor, Oracle Vesper Greywind's toolkit-bump brief"
---

Worktree: `/Users/feral/mydev/bboxy450-harness-toolkit-bump`; branch `build/toolkit-bump`; starting head `01fe3849de416e79a54a4c89dadb4f5e23d8b3b5`. Scope: the direct toolkit-bump request and `bboxy450-harness-parity/briefs/COMMON.md`. No push, merge, publication or installation in the operator's Applications directory.

The standalone UI toolkit now resolves entirely to term `775527db16ffe33ee5cafaa0ff8566e3ea7512da`, replacing `55f50175c37aad3b4d6edd4e0dc6c74fd56f2530`. No harness API adaptations were needed and no toolkit implementation was copied or forked.

- `Cargo.toml:83` — all four crates.io UI patches use the same new revision.
- `Cargo.lock:225` — exactly five source revisions change: bboxy450-doc, bboxy450-mermaid, bboxy450-render, bboxy450-ui and bboxy450-ui-core. Registry package versions and dependency edges are unchanged; the existing shared published runtime/Yarn/TLC remain at 0.1.1.
- `tests/gui_dependencies.py:22` — the existing duplicate-package assertion now also includes Mermaid in the common-source check and rejects additional term Git revisions. A uniform workspace-path toolkit remains valid for term embedding.
- `ATTRIBUTION.md:73` — records the upstream Mermaid/toolkit revision under the existing license and absolute attribution policy URL.

`RUSTC_WRAPPER=` was used for Cargo verification because the machine's configured sccache failed at `rustc -vV` with `Operation not permitted`. Tests used private homes and scripted/offscreen controls; there was no operator input. The GUI gate's own private-home isolation and the canonical full suite's isolation were retained.

| Verification command | Result | Local evidence |
|---|---|---|
| `RUSTC_WRAPPER= python3 -I tests/gui_gate.py` | exit 0; 87 GUI/process tests, strict GUI Clippy and Metal smoke passed | `target/toolkit-bump/gui-gate.log`, `gui-gate.json` |
| `RUSTC_WRAPPER= python3 -I tests/gui_dependencies.py` after narrowing the lockfile | exit 0; one toolkit source/copy, shared UI-thread registry | stdout; repeated by the full gate |
| `cargo check --offline --locked -p bboxy450-harness --all-features --all-targets` in the private term workspace | exit 0 | `target/toolkit-bump/embedded-check.log` |
| `python3 -I tests/gui_dependencies.py` in that embedded harness | exit 0; workspace patches produce one toolkit copy | `target/toolkit-bump/embedded-dependencies.log` |
| `cargo test --offline --locked -p bboxy450-mermaid --test fixtures` in that workspace | exit 0; 17 tests, including exact 30px message pitch, exterior back-edge lanes and the straight-through broken control | `target/toolkit-bump/term-mermaid-tests.log` |
| `cargo test --offline --locked -p bboxy450-doc --test mermaid` in that workspace | exit 0; 7 tests, including UI-font spacing/label geometry and the real document corpus | `target/toolkit-bump/term-doc-mermaid-tests.log` |
| `LAB=/Users/feral/mydev/bboxy450-term-harness-ref/lab/web python3 -I scripts/parity-gate.py target/toolkit-bump/parity-after` | exit 0 | `target/toolkit-bump/parity-after.log` |
| Same-reference before/after assertions, using the gate's 0.003 image/0.02 row tolerances | exit 0; all seven fixtures | `target/toolkit-bump/parity-comparison.json` |

The embedded workspace is a private archive of the real term `775527db` tree under `target/toolkit-bump/term-embedded`, with this worktree's harness sources placed at its existing harness member path. Its root crates.io patches are unchanged. `git ls-remote` reported live term main as `6f761489957a5adf3e6e42ddfb5d8a0b7f8d7bcd`; `git diff --exit-code 775527db 6f761489 -- Cargo.toml` and all consumed toolkit crate paths exited 0. Thus the checked workspace patches and APIs are byte-identical to live term main. Receipt: `target/toolkit-bump/term-main-compatibility.json`.

The mermaid fixture was captured before and after with `EXPAND=all sh scripts/parity-web.sh tests/fixtures/parity/mermaid.jsonl OUT 900x1100`, which passes `--expand all` to the native CLI. Both images were inspected. The new sequence has 30px message pitch and the Tools → Session actor edge uses an exterior lane below the nodes. Native geometry assertions above verify the exact pitch, participant heads and label clearance.

[Before side.png](/Users/feral/mydev/bboxy450-harness-toolkit-bump/target/toolkit-bump/mermaid-before/side.png) · [After side.png](/Users/feral/mydev/bboxy450-harness-toolkit-bump/target/toolkit-bump/mermaid-after/side.png) (web | native | diff). Expanded capture scores: mean image difference `0.0217 → 0.0226`, row mismatch `0.3145 → 0.3091`. The two web PNGs have identical SHA256 `bab9c1c84d9f74fd760938fbaa850163bec2ac5afb292483d0ebb42cdf7ea666`.

![Before Mermaid capture](/Users/feral/mydev/bboxy450-harness-toolkit-bump/target/toolkit-bump/mermaid-before/side.png)

![After Mermaid capture](/Users/feral/mydev/bboxy450-harness-toolkit-bump/target/toolkit-bump/mermaid-after/side.png)

Unexpanded canonical parity scores against the same web reference:

| Fixture | Mean image difference, before → after | Row mismatch, before → after |
|---|---|---|
| acp-claude-acp | 0.0082 → 0.0083 | 0.0382 → 0.0382 |
| acp-codex-acp | 0.0075 → 0.0075 | 0.0345 → 0.0345 |
| acp-omp | 0.0081 → 0.0081 | 0.0182 → 0.0182 |
| acp-opencode | 0.0335 → 0.0335 | 0.3909 → 0.3918 |
| acp-pi | 0.0078 → 0.0079 | 0.0273 → 0.0273 |
| glyph-atlas | 0.0321 → 0.0322 | 0.2409 → 0.2409 |
| mermaid | 0.0204 → 0.0210 | 0.2709 → 0.2773 |

The stored baseline names web hash `4b4a1ac688f7…`; the current reference is `86de0fcaf06466ae39e2c1be0c5eb37d33ae367ab18430de9836f446d5dd458c`. The canonical instrument labels this `REFERENCE MOVED`, so its old-reference regression comparison is skipped. The additional assertions compare the actual before and after runs against the identical current reference and pass all gate tolerances. No baseline was changed. Existing target misses for opencode, glyph-atlas and Mermaid remain outside this dependency bump; the target gate was not claimed green.

Setup failures were resolved before the final verification: Cargo initially lacked term's embedded harness submodule in its offline cache, then completed a noninteractive fetch; unrelated Windows dependency-edge rewrites from that update were removed. The private term snapshot initially omitted required design tokens and the upstream Markdown/lab corpus; these were supplied from the same exact term revision and its build/tests rerun without changing upstream code.

`RUSTC_WRAPPER= make test` exited **0**, in the single full run from `2026-11-03T06:47:29.395484+00:00` to `2026-11-03T06:55:37.259346+00:00`. The immediately preceding `pgrep -f scripts/test.py` check found no actual Python canonical test runners (shell commands merely mentioning the filename were filtered). Unit/integration/process tests, both Clippy configurations, all eight capability red/green controls, Yarn/native formal checks, native/Java TLC full-graph and verdict parity, schema/IR checks, GUI tests/Metal smoke and private unsigned packaging all passed. Logs: `target/toolkit-bump/make-test.log`; command/exit/times: `target/toolkit-bump/make-test.json`; source-input hashes: `target/toolkit-bump/test-current.json`.

Formal checking covered **93 model variants, 33 contracts and one crate**, including their safety/deadlock/liveness broken controls. Existing bounded assumptions and implementation-only coverage remain in `verification/{corpus,contracts,coverage}.toml`. This change alters dependency selection and deterministic layout only; no harness concurrency, lifecycle, retry, ownership or persistence contract changed, so no harness model was modified. The canonical gate bound its evidence to the exact changed input hashes; these were asserted equal to the final Cargo manifest, lockfile and dependency gate bytes. The final-head formal evidence is retained under `target/formal/`, with task logs under `target/toolkit-bump/`.

Raw logs, captures, model copies and generated gate records remain outside Git under `target/`. Generated tracked `.bboxy/runs` files are restored to their starting contents. The only committed paths are `Cargo.toml`, `Cargo.lock`, `tests/gui_dependencies.py`, `ATTRIBUTION.md` and this indexed report. No assigned implementation gap remains; the unrelated parity target misses above are explicitly retained.
