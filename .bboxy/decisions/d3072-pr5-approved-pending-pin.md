---
id: "03R63NSMGCB7VBR1N0SESBN7HD"
number: 3072
type: decision
title: "PR #5 (E3b GUI phase 2) approved pending the final toolkit pin; polish follow-ups"
status: decided
decided: "2026-11-01"
links: [{ to: "[[2026-10-31-harness-gui-phase2]]", rel: amends }]
---
# PR #5 approved pending the pin

**Approved content.** At `f42bbfa`, all ten MUST items from the round-2 rejection are fixed. Their red controls are committed (`.bboxy/runs/e3b-round2-red-*.log`). The Oracle inspected the screenshot: bodies sit inside cards, accordions are subdued, the approval buttons are compact and distinct, and the toggles are styled.

**Remaining merge condition.** Re-pin `bboxy450-ui`, `bboxy450-doc` and `bboxy450-render` to the merge commit of PR #86 on bboxy450-term `main`, or to the published 0.1.2 crates. Then run the full `make test`. The Oracle merges after verifying both.

A pre-existing flake (`store_recovery.rs:233`, empty pidfile) belongs to d2019 and E1c item 7. It does not block this merge.

**Follow-ups, deferred to the next GUI task:**
- Delete the dead `prepared_on_ui` and `large_markdown_blocks` fields (`view.rs:1027-1028`, `1305-1319`).
- Add smoke checks:
  - restored `mdblock` rects sit inside their card;
  - the accordion header fill is the subdued surface;
  - the inline toggle controls sit inside the body inset;
  - there are no empty boxes after approval labels.
- Visual polish:
  - accordion body text sits on the border edge;
  - faint empty icon boxes appear at the right of the accordion headers;
  - the header pill labels (Latest, Interrupt, Stop) are vertically misaligned with their outlines.
