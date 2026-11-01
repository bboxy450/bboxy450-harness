---
id: "0RGWY2T5YQ2EX40CHDYKY17F7V"
number: 1
type: note
title: .bboxy — bboxy450-harness knowledge base
---
# .bboxy — bboxy450-harness knowledge base

Records for this repository live here as Markdown with frontmatter. They follow the same schema as bboxy450-term (`.bboxy/specs/work-items.md` upstream). No secrets and no raw transcripts go in this directory.

Start with [[harness-zoomed-out]], then [[toolkit-lowering]].

| Dir | Holds | Owner |
|---|---|---|
| `research/` | Source evidence on prior harnesses and the toolkit, as `file:line — mechanism` | research agents |
| `specs/` | The zoomed-out spec and the lowering onto the toolkit | Oracle, reviewed by the Architect |
| `interfaces/` | ICDs: protocol (Op/Event/Item), gateway, provider, embed/seat | Architect |
| `invariants/` | Laws, each with its check command and cost | Architect |
| `decisions/` | Decision records: alternatives, choice, reason, date | Oracle and Architect |
| `tasks/` | One file per Executor assignment: scope, allowed paths, verify command, evidence | Oracle |
| `runs/` | Logs and summaries referenced from tasks | anyone |
| `log/` | bboxy450-term provenance journal | tool-written |
