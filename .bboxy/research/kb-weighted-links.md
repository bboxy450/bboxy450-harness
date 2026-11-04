---
type: research
title: How the bboxy KB does weighted links between documents (code and spec, 2026-11-03)
status: current
links:
- to: '[[swarm-dreaming]]'
  rel: evidence-for
id: 01M6J6DF6C0YJHFRPQNNW0749S
updated: 2026-11-04T02:32:39.032Z
created: 2026-11-04T02:32:39.032Z
number: 5970
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:effd08ce61daca1f1f25c8602deb23ca552b46b863fce2ee9d06bdfb9c65184b
  frontmatter_digest: sha256:72a6b5167b9a612db1b7b85cb331b62276ab908e4877829f7d009b073852a350
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: SAtsFD7uByv9GLvxgVzBpFbRpMDfKBak7xIhtRDfW2wrHCSVIjjgnmY1+LKjHzBIzq0IOwZvQZiucR42Ld2fBQ==
  log: .bboxy/log/2026-11-04.jsonl#40185
---
# Weighted links in the bboxy KB

Sources: bboxy450-term at its local main checkout, 2026-11-03. All paths are under `/Users/feral/mydev/bboxy450-term`.

## Syntax: three ways to write a link, all parsed by the index

| Form | Parsed at | Meaning |
|---|---|---|
| `[[slug]]`, `[[slug\|display]]` anywhere in the body | `crates/bbt-mux/src/index.rs:136-180` `wikilinks()` | relation `ref`, weight `1.0` |
| `[[slug]]{rel:w}` (trailing annotation) | same function, `:155-167` | relation `rel`, weight `w`. `w` must parse as an f64 that is finite and `> 0`, otherwise the annotation is ignored and the defaults stay. `{rel}` with no `:w` means weight 1 |
| frontmatter `links: [{to, rel, weight}]` | `index.rs:226-247` | `to` may itself be a `[[wikilink]]` (unwrapped). Missing `rel` means `ref`; a missing or invalid `weight` means `1.0`. **The key is `weight`, not `w`** |
| frontmatter fields that imply links | `index.rs:249-276` | `parent` → rel `parent`, w 3. `depends_on` → `depends`, `children` → `child`, `assignee`, `delegated_by`, `produced_by`, `for`, `derived_from`, `created_by`/`updated_by` → `wasAttributedTo`, all w 2 |

- **Duplicates.** Links are deduped by `(to, relation, weight bits)` (`index.rs:278-285`). The same target and relation at two different weights are therefore two edges.
- **Obsidian compatibility.** Decided in `.bboxy/specs/provenance.md` §5: the relation and weight sit in a trailing `{rel:w}` annotation, so the link stays a plain Obsidian link. The older pipe form `[[slug|rel|w]]` in `.bboxy/specs/work-items.md:197-199` is superseded, and the code treats the text after a pipe as display text only (`index.rs:144-149`).

## Graphs and how weight is used

- **Graph kinds** (`index.rs:291-297`): `Markdown`, `Task`, `Execution`, `Artifact`, `Provenance`. Edge weights are `{relation, weight}`.
- **Degree** = sum of incident edge weights, capped at f64::MAX (`crates/bbt-mux/src/knowledge/bboxy.rs:413-421`). It is used in ranking.
- **Edge export** is sorted by `(from, to, rel)`, then weight (`bboxy.rs:422-441`). The MCP `links` method returns weighted links in both directions (`knowledge/mod.rs:63`). The graph tool emits `{from, to, relation, weight}` (`knowledge/mod.rs:227`).
- **Intended uses** (`work-items.md:207-215`): navigation, "related", search ranking. The messaging graph uses "count decayed by recency" weights. That is the one existing precedent for dynamic weights, and it is computed, not stored.

## Roots

`.bboxy/decisions/d257-knowledge-base-first-plugin.md:28`: "roots mount several at once with the repository's `.bboxy` first". Multiple mounted roots are therefore the decided design, which makes a graph of graphs a native shape. `knowledge_status` in this swarm today shows one root, `/Users/feral/mydev/bboxy450-harness/.bboxy`, with 5475 records.

## Drift found

- **Child weight.** `work-items.md:201` says `parent`/`child` weigh 3 and `depends`/`blocks` 2. The code gives only `parent` weight 3. `children` → `child` gets 2, and nothing maps a `blocks` field.
- **Unbounded weights.** Weights are unbounded positive numbers. There is no normalisation and no decay in the index.

## What this means for memories

- Memory records can use the existing link forms unchanged.
- Dynamic strength (recency, recall, Hebbian co-activation) should be computed from logs, like the messaging graph's decayed weights, and not rewritten into files on every recall. Rewriting on every recall would churn git and race concurrent writers.
