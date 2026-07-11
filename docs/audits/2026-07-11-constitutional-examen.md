# Constitutional Examen — 2026-07-11

This is the durable ledger for the bidirectional examination of the framework
constitution and implementation. The baseline is commit `8737a871` (`Add
blessed application launch ceiling`). Audit cells use stable IDs; code commits
will carry the corresponding ID in their subject, and the final checkpoint
will reconstruct the ID-to-hash map from Git history.

## Protocol

Verdicts are `Verified`, `Code violation`, `Doctrine gap`, `Stale doctrine`,
`Flag`, or `Intentional non-merge`. Exclusive claims require two differently
shaped probes. No framework correction is admitted before the initial
doctrine-to-tree, tree-to-doctrine, and instruments census is complete.

Public API and behavior are frozen during this run. A correction must have a
clear owner, delete or demote an old path, remain independently reviewable,
and pass the full ritual. Uncertainty produces a flag, not code.

## Baseline

- Worktree: clean at `8737a871`.
- Constitution: `docs/master_design.md`, 1,270 lines.
- Narrow doctrine: `docs/ui_command_architecture.md`, 187 lines;
  `docs/command_module_organization.md`, 149 lines.
- Roadmap: 89 lines; no item currently in flight.
- Protected state: `examples/glass_tuner/app/state.rs` contains
  `comparison_open: true`.
- Full library suite: 804 passed, 8 ignored, 0 failed.
- Example checks: `text_editor`, `control_gallery`, and `glass_tuner` passed.
- `cargo fmt --check` and `git diff --check`: passed.

## Initial constellation map

| ID | Constellation | One-sentence promise | Lowest coordinating owner | Ordinary public entrance |
| --- | --- | --- | --- | --- |
| CT-1 | Document truth | Persistent text and explicit editing state become versioned, undoable, safely persisted document changes. | `text`, `document`, and `timeline`, coordinated by runtime transactions | `Document`, `TextArea::from_document`, document commands |
| CT-2 | Capability | Typed requests resolve through explicit scopes to typed executors and return schedulable results. | `command`, `responder`, `target`, and `response` | command types, `Editing::standard`, responder declarations |
| CT-3 | Interface | Declarative descriptions acquire retained identity, derived geometry, and ephemeral interaction without absorbing behavior. | `view`, `composition`, `layout`, and `session` | widget builders and `Runtime::view` |
| CT-4 | Presentation | One invalidation decision becomes one prepared logical frame and capability-appropriate physical outputs. | runtime presentation, with scene and overlay as owned engines | `Runtime::render_scene` and native pending presentation |
| CT-5 | Application | An application runtime crosses lifecycle and event-loop boundaries without exposing ordinary callers to realization machinery. | `runtime`, `shell`, `host`, and `platform` | `platform::launch(app)` |
| CT-6 | Native boundary | Resolved framework facts cross once into paint, GPU, window-system, clipboard, and dialog representations. | `platform/native`, `paint`, and `render` | platform backends and advanced `Platform`/`Runner` seams |
| CT-7 | Root vocabulary | Shared facts retain one meaning and one dependency direction wherever constellations meet. | the named root module for each datum | central crate re-exports and namespaced supporting concepts |

The map is a hypothesis to test, not a conclusion. Inputs, outputs, identity,
time, failure, absence, coordinates, lifecycle, competing owners, and practiced
tests will be added during the constellation sweep.

## Phase A — doctrine to tree

| Cell | Doctrine range | Verdict | Receipts and notes |
| --- | --- | --- | --- |
| D-01 | First principles and governing shape | Pending | |
| D-02 | Geometry, window, color, and text ownership | Pending | |
| D-03 | Widget, view, task, composition, and layout | Pending | |
| D-04 | Scene and presentation-space laws | Pending | |
| D-05 | Overlay portable contract and native-popup lifecycle | Pending | |
| D-06 | Windows popup map and alpha/material pipeline | Pending | |
| D-07 | Theme, session, interaction, command, and notification | Pending | |
| D-08 | Keymap, state, buffer, target/responder, response, timeline, clipboard | Pending | |
| D-09 | Runtime, diagnostics, platform, and public API rule | Pending | |
| D-10 | Implementation protocol, smells, answers, and review standard | Pending | |
| D-11 | Narrow command/view doctrine | Pending | |

## Phase B — tree to doctrine

| Cell | Constellation | Verdict | Receipts and notes |
| --- | --- | --- | --- |
| C-01 | Document truth | Pending | |
| C-02 | Capability | Pending | |
| C-03 | Interface | Pending | |
| C-04 | Presentation | Pending | |
| C-05 | Application | Pending | |
| C-06 | Native boundary | Pending | |
| C-07 | Root vocabulary | Pending | |
| C-08 | Overlay housing examination | Pending | File size is not evidence; semantic seams must earn any split. |

## Phase C — instruments and memory

| Cell | Instrument | Verdict | Receipts and notes |
| --- | --- | --- | --- |
| I-01 | Architecture and source-string witnesses | Pending | |
| I-02 | Behavioral, tombstone, and ignored tests | Pending | |
| I-03 | Examples and smokes as external-style callers | Pending | |
| I-04 | Roadmap revalidation | Pending | |

## Candidate ledger

No candidate is admitted until the initial census is complete.

| ID | Finding | Evidence | Admission | Rank | Disposition |
| --- | --- | --- | --- | --- | --- |

## Health scores

Scores are 0–4 for ownership, compression, state integrity, boundary truth,
and practiced evidence. Baseline scores remain pending until the complete
constellation census provides receipts.

## Correction ledger

| Cell | Commit subject | Hash | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- | --- |

## Flags

Flags will be ranked by architectural consequence after the initial census.

## Final fixed-point sweep

Pending.
