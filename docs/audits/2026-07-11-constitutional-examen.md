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
| D-01 | First principles and governing shape | Verified | Retired roots are structurally absent and guarded by `src/tests/architecture.rs:2`; examples remain private fixtures in `src/lib.rs:46-53` and import the crate through public `wgpu_l3` paths. Renderer dependency direction has a separate recursive import census at `src/tests/architecture.rs:94`. No `util`, `common`, `helper`, or empty root bucket was found. |
| D-02 | Geometry, window, color, and text ownership | Verified | Paint vocabulary is private and unit-distinguished (`src/tests/architecture.rs:109-256`); geometry file modules remain behind the facade at `src/tests/architecture.rs:260`. All session/shell/host/platform projections wrap `window::Facts` (`src/window/facts.rs:6-102`). Color conversion has one owner in `src/color.rs:1-35`. Text dependency independence is recursively checked at `src/text/tests.rs:71-100`; caret affinity, bidi direction, obscuring, persistent spans/line identity, and atomic versioned saves have independent witnesses at `src/tests/architecture.rs:374-510` and `src/tests/architecture.rs:1331-1403`. |
| D-03 | Widget, view, task, composition, and layout | Verified | Composition is identity without behavior and remains private (`src/tests/architecture.rs:751-805`); interaction storage and layout structures remain private (`src/tests/architecture.rs:814-1035`). Generic reveal is palette-agnostic at `src/tests/architecture.rs:943-955`. Viewport geometry, clips, projected scrollbar chrome, cursor promises, and reveal behavior are exercised through behavioral scenes beginning at `src/tests/layout_scene.rs:279`, `src/tests/layout_scene.rs:665`, `src/tests/layout_scene.rs:1028`, `src/tests/layout_scene.rs:1166`, and `src/tests/layout_scene.rs:2142`. Worker dispatch and the no-poll-wake rule are pinned at `src/tests/architecture.rs:2948-3010`. |
| D-04 | Scene and presentation-space laws | Verified | Layout-to-scene paint stays internal (`src/tests/architecture.rs:1044-1068`). Resting/moving geometry lacks a primitive mode axis (`src/tests/architecture.rs:1415-1441`), renderer snapping is a witness (`src/render/quad.rs:208-214`), and moving/resting endpoint equality is behavioral case law at `src/render/quad.rs:880-925`. `Rule` is a distinct scene/paint primitive (`src/scene/mod.rs:15`, `src/paint/mod.rs:130`). Refraction constraints have one scene owner and projection witness (`src/tests/architecture.rs:1707-1733`). Group bounds include shadow and blur reach (`src/paint/mod.rs:1321-1416`). |
| D-05 | Overlay portable contract and native-popup lifecycle | Verified | Backend preference resolves solely from capability with explicit fallback (`src/overlay.rs:389-407`) and behavioral cases at `src/overlay.rs:803-865`. Live entries, in-frame ghosts, and retiring native popups are distinct representations (`src/overlay.rs:23-58`, `src/overlay.rs:500-626`). Per-parent authoritative synchronization and physical cursor/IME host axes are guarded at `src/tests/architecture.rs:1906-2011`; overlay and native stores own `window::Departed` cleanup at `src/overlay.rs:676` and `src/platform/native/adapter.rs:113`. Linux Wayland is capability-fallback-only while X11/macOS popup windows remain available (`src/platform/native/popup.rs:689-700`). |
| D-06 | Windows popup map and alpha/material pipeline | Verified, with declared open flag | Shell-style enforcement, nonactivation, accent acrylic, backend choice, settle-rate applicators, border color ownership, finite first-present recovery, `DwmFlush`, popup packing, and premultiplied-alpha witnesses are separately guarded through `src/tests/architecture.rs:2042-2859`. Implementation receipts include `src/platform/native/sys/windows.rs:23-75`, `src/platform/native/sys/windows.rs:179`, `src/platform/native/mod.rs:75-101`, and `src/render/popup_pack.rs:1-173`. The unresolved unlogged first-frame skip is honestly still open in `docs/master_design.md:515-518` and roadmap items 2/21. |
| D-07 | Theme, session, interaction, command, and notification | Verified | Theme keeps metrics and appearance separate (`src/theme/mod.rs:188-207`); typography affects measurement (`src/tests/layout_scene.rs:2265-2437`) while shortcut formatting is measured and painted together (`src/tests/layout_scene.rs:1878-1919`). Palette scope behavior is exercised at `src/tests/commands.rs:390-493`. Standard editing is enumerable and declinable at `src/tests/commands.rs:4-40`. `window::Departed` is the single close fact with listener and no-checklist witnesses at `src/tests/architecture.rs:1610-1652`. |
| D-08 | Keymap, state, buffer, target/responder, response, timeline, clipboard | Verified | Mac/Windows semantic shortcut resolution is behavioral at `src/tests/commands.rs:82-177`; state reasons are kept command-ignorant at `src/tests/architecture.rs:1406-1413`. Persistent source spans and line identity are guarded at `src/tests/architecture.rs:374-431`. Target/responder erasure stays private through the command boundary tests around `src/tests/architecture.rs:546-748`. Invalidation merging is owned by `src/response/effect.rs`; scoped history grouping is pinned at `src/tests/architecture.rs:1548-1604`. Clipboard result/empty/failure semantics and command-owned paste availability are guarded at `src/tests/architecture.rs:1264-1327`. |
| D-09 | Runtime, diagnostics, platform, and public API rule | Verified | Coarse invalidation and layout reuse live in `src/runtime/presentation.rs:346-370`; snapshot restore has one transient reset (`src/tests/architecture.rs:1655-1677`); frame preparation has one paint recipe (`src/tests/architecture.rs:70-90`). Diagnostic targets are cross-checked against doctrine at `src/tests/architecture.rs:1811-1853`. Native launch versus explicit clipboard and lower seams are behavioral at `src/tests/platform_tests.rs:619-663`, with all examples consuming `platform::launch` under `src/tests/architecture.rs:47-68`. Root re-exports remain the named central concepts in `src/lib.rs:55-78`. |
| D-10 | Implementation protocol, smells, answers, and review standard | Verified as practiced architecture | The test suite contains direct witnesses for One Truth/One Owner (frame preparation, caret conversion, refraction), Witness Demotion (renderer snap assertions), Axis Splitting (logical/physical areas, command/notification, cursor value/host), Structural Absence (notification and composition APIs), Exceptions Become Citizens (`Rule`, `Pane`, retiring popup), Endpoints Are Truth (motion/reveal), and retired-shape tombstones. This cell records practice, not a claim that every future smell is absent. |
| D-11 | Narrow command/view doctrine | Stale doctrine | `docs/ui_command_architecture.md:153-156` says `text::edit::State` owns cursor, selection, preedit, scroll, and blink. The current `State` contains only cursor and selection (`src/text/edit/state.rs:4-24`); surface facts live in `text::edit::ViewState` (`src/text/edit/view.rs:309-390`). `docs/command_module_organization.md:27-39` also omits the now-public `command::Set`/`Member` surface, and its `Spec` description omits `Listing`, despite `src/command/mod.rs:13-17` and `src/command/spec.rs:119-170`. These are documentation candidates, not code violations. |

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
