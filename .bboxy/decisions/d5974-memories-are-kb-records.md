---
type: decision
title: Memories are KB records in .bboxy/memories, system-wide and cross-harness, linked by weighted wikilinks; the dream log carries their provenance
status: accepted
links:
- to: '[[memory-graph]]'
  rel: decides
- to: '[[dream-log]]'
  rel: constrains
- to: '[[kb-weighted-links]]'
  rel: evidence
- to: '[[memory-formats-sota]]'
  rel: evidence
- to: '[[d5963-dream-adoption-and-cross-repo-scope]]'
  rel: builds-on
id: 01M6J6R78CYT756TQB9TVNZKTF
updated: 2026-11-04T02:38:31.247Z
created: 2026-11-04T02:38:31.247Z
number: 5974
created_by: seat:01M6J58299FFZEQTMJJMY880A1
updated_by: seat:01M6J58299FFZEQTMJJMY880A1
authorship:
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  kind: seat
  public_key: cdW8yqNDO3fr8HOuRjvSiVoU8rDeQzo8pyhgbSLnj3M=
  process_generation: '1'
provenance:
  digest: sha256:1923744c2a4f7b2e335fa7ff47811ef3d8f17b7f1d840049c34a61dd1706bd6e
  frontmatter_digest: sha256:95829eefa5ea5815ea707ad9b4eab6feb28ce38a22bfba40c75185673a9caefd
  signer: seat:01M6J58299FFZEQTMJJMY880A1
  sig: SenbRQVH3QHNhQh+HuCucKhcRM/gUxn/9U8lcQKuqCZGtuE3KT6aUPGnfZe1LsuCEq72EhRbpmlCEp9DAnJYCQ==
  log: .bboxy/log/2026-11-04.jsonl#40197
---
# Memories are KB records

**Date:** 2026-11-03. **Decided by:** the operator: "we will support system wide, .bboxy/memories, cross harness, cross system, cross everything! graph of graphs of memories, memory links with weight attached to them, same way we do in kb". **Recorded by:** Vesper Greywind <ARCHITECT>.

## Decision

1. **Each memory is one KB record.** It has `type: memory` and lives at `<root>/memories/<slug>.md`. It is written, signed, numbered and indexed like every other `.bboxy` record. There is one format for every harness. Each harness gets a lowering of it ([[memory-graph]] §5).
2. **There are two kinds of root, which together form a graph of graphs.**
   - **Repository roots:** `<repo>/.bboxy/memories/`.
   - **The system root:** `~/.agents/.bboxy/memories/`, in the private `bboxy450/dotagents` repository. It is shared by every harness and, through git, by every machine.
   - The roots mount together (d257: "roots mount several at once with the repository's `.bboxy` first").
3. **Links use the existing KB weighted-link forms, unchanged.** In the body, `[[slug]]{rel:w}`. In frontmatter, `links: [{to, rel, weight}]`. Memory relations have a closed vocabulary with default weights ([[memory-graph]] §3).
4. **Stored weights change only through consolidation.** Recency, recall and co-activation strength are computed from the IR logs and never written back to the files on each use. Rewriting files on every use would churn git and make concurrent writers collide.
5. **Provenance is split between the KB and the dream log.**
   - The KB record holds the memory's content and links.
   - The IR dream log ([[dream-log]]) holds how the memory came to be: the dream run and what started it, the evidence it cited, its trials, adoptions and recalls.
   - Each side pins the other by digest.
   - Hand-written memories are allowed. Their origin is the signed KB authorship, and they must cite evidence like dreamt ones.
6. **Opt-in for cross-repository evidence (d5963) is a committed flag.** It lives in `.bboxy/dream.toml` as `cross_repo = true`. When the file or the key is absent, the repository is not opted in.
   - A memory may enter the system root only if every repository its evidence comes from is opted in.

## Alternatives considered

- **IR items only, no Markdown.** Rejected: such memories are not human-editable, not visible in the KB graph, and cannot carry weighted links.
- **Each harness's native store as the source of truth.** Rejected: no two harnesses share a format (see [[memory-formats-sota]]), and none of them carries provenance or weights.
- **Weights updated on every use.** Rejected for the git-churn reason in item 4.
- **A system root in `~/.bboxy450-harness`.** Rejected: it is not under git and not synced across machines.
