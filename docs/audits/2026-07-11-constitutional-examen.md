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
| D-11 | Narrow command/view doctrine | Stale doctrine → corrected by R-01 | The initial text said `text::edit::State` owned cursor, selection, preedit, scroll, and blink, but `State` contains only cursor and selection (`src/text/edit/state.rs:4-24`) while surface facts live in `text::edit::ViewState` (`src/text/edit/view.rs:309-390`). The command guide also omitted the public `command::Set`/`Member` surface and `Listing` policy despite `src/command/mod.rs:13-17` and `src/command/spec.rs:119-170`. R-01 now records both live shapes in `docs/ui_command_architecture.md:153-168` and `docs/command_module_organization.md:22-82`. |

## Phase B — tree to doctrine

| Cell | Constellation | Verdict | Receipts and notes |
| --- | --- | --- | --- |
| C-01 | Document truth | Verified | `document::Outcome` and `text::edit::ActionResult` are adjacent projections rather than competing engines: document commands project edit availability/changes into command outcomes in `src/document/edit.rs`, while the text engine owns mutation in `src/text/edit/editor.rs`. Save identity is unforgeable outside the document implementation (`src/document/save.rs`) and is practiced by deferred, out-of-order completion cases in `src/tests/document_editor.rs:430-775`. Clipboard absence/failure and draft submission remain distinct meanings (`src/clipboard/system.rs`, `src/draft/mod.rs`). The sole mismatch is the already-recorded `State`/`ViewState` doctrine drift in D-11. |
| C-02 | Capability | Verified, with stale doctrine | Commands remain imperative and notifications past-tense across `src/document/command.rs`, `src/timeline/command.rs`, `src/session/service.rs`, `src/document/notification.rs`, and `src/window/departed.rs`. Registry metadata, enumerable sets, responders, erased targets, observations, and effects each have a named owner (`src/command`, `src/responder`, `src/target`, `src/command/observer.rs`, `src/response`). The two `AnyTarget` representations are the declared non-merge in roadmap item 22. D-11 records the only gap: narrow command doctrine omits `Set`, `Member`, and `Listing`. |
| C-03 | Interface | Flag | Ordinary callers build widgets and views while `composition`, `layout`, and `session` retain identity, geometry, and interaction; the external-style gallery scene at `src/tests/layout_scene.rs:3600` practices that sentence. The palette-specific scope predicate at `src/interaction/target.rs:178-182` names a real command-scope distinction, not a paint/layout exception. However, `layout::Frame` is a role tag plus a broad optional-field cluster (`src/layout/frame.rs:11-78`), so combinations such as text-only layout data on unrelated roles remain representable. Splitting it now would be broad internal redesign without a behavioral contradiction or concrete caller, so this remains a state-integrity flag. |
| C-04 | Presentation | Verified | `PreparedFrame` remains the one logical recipe (`src/runtime/presentation.rs`) and scene opacity/group composition has one owner (`src/scene/mod.rs:114-137`). The forced-full-opacity group bit is an explicitly named diagnostic request from `widget::Floating::diagnostic_force_promoted_at_full_opacity` (`src/widget/panel.rs:52-55`) through overlay projection to scene composition, with endpoint tests at `src/overlay.rs:985-1002` and `src/scene/mod.rs:570-576`; it is not an accidental boolean protocol. Overlay backend selection and fade scheduling remain owned by the overlay state machine. |
| C-05 | Application | Verified | Runtime produces render work, shell projects it into public `Work`, host maintains framework-facing snapshots, and platform alone synchronizes backend effects (`src/runtime/work.rs`, `src/shell/work.rs:24-45`, `src/host/mod.rs:99-151`, `src/platform/mod.rs:139-210`). Repeated window/request/cursor collections have different stage lifetimes rather than competing authority. `platform::launch` is the ordinary ceiling (`src/platform/mod.rs:22-31`), while `Platform`/`Runner` remain deliberate advanced seams and are behaviorally distinguished at `src/tests/platform_tests.rs:619-663`. Departure reaches runtime, overlay, host, platform, and native stores through one `window::Departed` fact. |
| C-06 | Native boundary | Verified | OS events normalize physical coordinates and scale once in `src/platform/event.rs:14-167` and `src/platform/event.rs:228-290`; paint owns typed logical/physical areas, points, and grid snapping (`src/paint/area.rs`, `src/paint/point.rs`, `src/paint/grid.rs`). Scene-to-paint conversion is concentrated in `src/platform/native/paint.rs`, while renderer formats, premultiplication, and popup packing remain below that boundary (`src/render/alpha.rs`, `src/render/popup_pack.rs`). Platform-specific cfg branches concern realization and adapter selection, not framework semantics. The documented scene-transform sanitization duplicate remains roadmap item 22's intentional non-merge. |
| C-07 | Root vocabulary | Verified | The crate root exports named central concepts and keeps realization modules private (`src/lib.rs:1-78`). Geometry, color, theme, keymap, subject, icon, animation, pointer, response, and state each have a domain owner; recursive architecture witnesses guard dependency direction and privacy (`src/tests/architecture.rs:94-349`, `src/color.rs:70-89`). No new `util`, `common`, `manager`, `helper`, or empty root bucket was found. Similar names such as `Platform`, `Presentation`, `Frame`, and `Surface` have distinct scopes and no concrete import collision. |
| C-08 | Overlay housing examination | Resistance; no correction admitted | `src/overlay.rs` already contains visible semantic regions: policy (`Preference`, `Backend`, `Capabilities`, `resolve_backend`), portable/native payloads (`Draft`, `Layer`, `PopupPresentation`, `PopupMaterial`), lifetime representations (`Live`, `Entry`, `Ghost`, `RetiringPopup`), and the per-window `Store`. The apparent lifetime pieces are one transition engine: `Store::update_window` creates and orders all of them, resolves backend, carries fade state, caps afterlife, and emits one schedule (`src/overlay.rs:488-649`). Extracting policy/payload types would primarily widen private fields or add constructors so that same owner could keep assembling them; it would delete no competing decision path. Size and navigation cost alone therefore do not satisfy candidate admission. |

## Phase C — instruments and memory

| Cell | Instrument | Verdict | Receipts and notes |
| --- | --- | --- | --- |
| I-01 | Architecture and source-string witnesses | Verified, with brittleness cost | `src/tests/architecture.rs` contains 83 named functions covering absence, privacy, dependency direction, unique owners, and previously escaped native failures. Source-string checks are appropriate for structural absence and cfg/FFI facts that Rust's type system cannot observe, but their 223 file reads make housing changes costly. No witness was found recomputing a production algorithm as an alternate authority; behavioral tests carry endpoints where available. The cost is acknowledged, but wholesale witness abstraction would be new machinery rather than a demonstrated correction. |
| I-02 | Behavioral, tombstone, and ignored tests | Verified | Tombstones protect promoted/retired shapes beginning at `src/tests/architecture.rs:2`, `src/tests/architecture.rs:236`, and `src/tests/architecture.rs:366`. The eight ignored tests are deliberate tiers, not dead concerns: six require GPU adapter/readback (`src/render/silhouette.rs:383`, `src/render/renderer.rs:1191-1311`), one is a 100k-operation reference property, and one is a release-mode measured benchmark (`src/text/acceptance.rs:53-142`). The ordinary suite retains smaller deterministic behavioral counterparts. |
| I-03 | Examples and smokes as external-style callers | Verified | All three example mains call `wgpu_l3::platform::launch`, and their runtime construction does not name `Runner`, `Platform`, or `Host` (`src/tests/architecture.rs:47-68`). The control gallery practices the widget-to-scene sentence (`src/tests/layout_scene.rs:3600`); the text editor crosses runtime, shell, host, and platform with persistence and task behavior (`src/tests/host_shell_tests.rs:87-619`, `src/tests/platform_tests.rs:827-1247`); glass tuner is a deliberate native-presentation diagnostic fixture (`src/platform/native/paint.rs:760-795`). |
| I-04 | Roadmap revalidation | Verified, with open manual/hardware flags | Items 2/21 still correspond to the explicit unresolved first-frame-skip note and trace (`docs/master_design.md:515-518`, `src/platform/native/popup.rs:617-744`). Context menu, text overflow, tables, accessibility, targeted redraw, and the other named arcs remain absent rather than half-implemented. Product-taste and hardware items still require their declared inputs. Item 22's two `AnyTarget` shapes and scene-transform sanitization duplicate remain real non-merges. Item 6 still names the protected glass-tuner tuning state; this run leaves `comparison_open: true` unchanged. |

## Candidate ledger

The initial census is complete. Findings are admitted only when the existing
tree supplies both a contradiction and a smaller truthful owner.

| ID | Finding | Evidence | Admission | Rank | Disposition |
| --- | --- | --- | --- | --- | --- |
| R-01 | Narrow command/view doctrine names retired ownership and omits live public command concepts. | D-11: `text::edit::State` versus `ViewState`; `command::Set`/`Member`/`Listing` versus the two narrow docs. | Admitted: direct doctrine/tree contradiction; documentation is the sole owner; correction narrows stale claims without behavior or API change. | 1 | Corrected in `3f1b2abd`; no framework mechanism added. |
| R-02 | `layout::Frame` permits role/optional-field contradictions. | C-03; `src/layout/frame.rs:11-78` and role-specific access throughout layout/scene paint. | Not admitted: no observed wrong behavior, and a split would span layout, interaction, scene paint, and inspection APIs. | Flag | Await a concrete invalid combination or accessibility/table caller. |
| R-03 | Overlay policy, payload, and lifetime concepts share one large source file. | C-08; `src/overlay.rs:11-735`. | Not admitted: the one `Store::update_window` transition owns the supposedly separate pieces, and extraction deletes no competing path. | Resistance | Keep the state machine colocated; reconsider only when an independent owner emerges. |

## Health scores

Scores are 0–4 for ownership, compression, state integrity, boundary truth,
and practiced evidence. `O/C/S/B/P` abbreviate those dimensions.

| Constellation | Baseline O/C/S/B/P | Total | Receipt and justification |
| --- | --- | --- | --- |
| Document truth | 4/4/4/4/4 | 20 | Save/version identity, edit mutation, persistence, clipboard absence, and history each have typed owners and end-to-end document tests (C-01; `src/tests/document_editor.rs`). |
| Capability | 4/3/4/4/4 | 19 | Resolution and execution are strongly owned and practiced; compression loses one point because the narrow public doctrine omits live `Set`/`Member`/`Listing` vocabulary (C-02, D-11). |
| Interface | 3/4/2/4/4 | 17 | Public widget sentences and boundaries are strong, but the role-tagged optional `Frame` weakens ownership locality and permits contradictory internal states (C-03). |
| Presentation | 3/4/4/4/4 | 19 | Frame preparation, scene composition, native handoff, and scheduling are singular; overlay's coupled concepts are truthful but costly to navigate (C-04, C-08). |
| Application | 4/4/4/4/4 | 20 | Runtime → shell → host → platform is a complete staged lifecycle with an application-altitude launch and end-to-end tests (C-05). |
| Native boundary | 4/4/4/4/3 | 19 | Coordinates, paint projection, formats, and OS facts cross at named seams; the remaining point is withheld because hardware GPU/readback witnesses are necessarily outside the default suite (C-06, I-02). |
| Root vocabulary | 4/4/4/4/4 | 20 | Central names, privacy, and dependency direction are explicit and structurally guarded (C-07). |
| **Total** |  | **134/140** | The principal deficits are one stale public explanation, one internal invalid-state cluster, overlay navigation cost, and hardware-only evidence. |

## Correction ledger

| Cell | Commit subject | Hash | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| R-01 | `R-01 Align narrow doctrine with live concepts` | `3f1b2abd` | 2 | 21 | 6 | Held: the command guide now enumerates `Set`, `Member`, and `Listing`; the UI/command guide separates persistent edit state from surface view state. |

Audit checkpoint history:

| Checkpoint | Hash | Statistics |
| --- | --- | --- |
| Begin durable ledger | `03b40bdc` | 1 file, 111 insertions |
| Record doctrine sweep | `56cdb734` | 1 file, 11 insertions, 11 deletions |
| Complete constellation census | `6026cb31` | 1 file, 39 insertions, 16 deletions |

The only admitted correction has an ID-bearing commit and is independently
green. The run made no framework-code change and no public-API or behavior
change.

## Flags

1. `layout::Frame`'s role-tagged optional cluster is the clearest state-integrity
   debt, but it lacks a demonstrated behavioral contradiction and would exceed
   an overnight correction (R-02).
2. The popup show-cycle first-frame contract remains open and explicitly belongs
   to roadmap items 2/21; it requires feature work and native verification.
3. Glass-tuner tuning state remains a manual-session concern under roadmap item
   6. The protected `comparison_open: true` value is unchanged.
4. Theme TOML datum grammar, session/interaction articulation, and the roadmap
   item 22 non-merges retain their standing flag/non-merge dispositions; the
   census found no new evidence authorizing work.

## Final fixed-point sweep

Complete. The changed capability/interface neighborhood was re-probed first:
the public command exports and guide now agree on `Set`, `Member`, `Listing`,
`Spec`, and the rest of the enumerated surface; the text guide now agrees with
the separate `State` and `ViewState` representations. Searches found the stale
claim only in this ledger's historical receipt.

The subsequent complete constellation sweep admitted no new high-confidence
correction:

- Document truth still has one mutation/save/history path; adjacent outcomes
  remain honest projections.
- Capability now has a truthful narrow public explanation as well as one
  routing/execution path.
- Interface retains the `layout::Frame` state-integrity flag; no concrete
  contradiction appeared.
- Presentation retains one frame recipe and one overlay lifetime engine; the
  housing examination still resists a split.
- Application retains the runtime → shell → host → platform lifecycle and
  application-altitude launch.
- Native boundary retains named coordinate, paint, renderer, and OS seams;
  hardware-only evidence remains honestly tiered.
- Root vocabulary still has no competing bucket, leaked realization module,
  or demonstrated naming collision.

### Final health

| Constellation | Baseline | Final | Change |
| --- | --- | --- | --- |
| Document truth | 4/4/4/4/4 | 4/4/4/4/4 | — |
| Capability | 4/3/4/4/4 | 4/4/4/4/4 | Compression +1: narrow doctrine names the live public sentence. |
| Interface | 3/4/2/4/4 | 3/4/2/4/4 | Flag retained; no score-driven patch. |
| Presentation | 3/4/4/4/4 | 3/4/4/4/4 | Housing resistance retained. |
| Application | 4/4/4/4/4 | 4/4/4/4/4 | — |
| Native boundary | 4/4/4/4/3 | 4/4/4/4/3 | Hardware tier unchanged. |
| Root vocabulary | 4/4/4/4/4 | 4/4/4/4/4 | — |
| **Total** | **134/140** | **135/140** | **+1**, earned by truthful doctrine only. |

### Closure accounting

- Initial doctrine verdicts: 10 verified cells, 0 code violations, 0 doctrine
  gaps, 1 stale-doctrine cell, and 1 declared open flag within a verified cell.
  Final state: the stale cell is corrected; the native first-frame flag remains.
- Candidates: 1 admitted and completed; 1 state-integrity flag; 1 housing
  resistance. No candidate was half-migrated.
- Correction statistics: 2 files, 21 insertions, 6 deletions. The pre-closing
  audit history from baseline through R-01 spans 3 files, 155 insertions, and
  6 deletions; this closing ledger checkpoint is intentionally not counted in
  its own pre-commit statistics.
- Outcomes held: public API frozen, behavior preserved, no visual collateral,
  no roadmap feature work, no push, and `comparison_open: true` preserved.
- Ranked flags and roadmap dispositions remain in the sections above. None is
  disguised as permission to code.
- Most surprising fact: the 1,200-line-looking overlay file is not many owners
  waiting to be modularized. Its live entries, ghosts, retiring native popups,
  backend resolution, fade timing, ordering, afterlife cap, and schedule are
  phases of one per-window transition. Splitting by noun would improve browsing
  while weakening the construction boundary.

### Final verification

- `cargo test --lib`: 804 passed, 8 deliberately ignored, 0 failed.
- `cargo check --example text_editor`: passed.
- `cargo check --example control_gallery`: passed.
- `cargo check --example glass_tuner`: passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Protected state: `examples/glass_tuner/app/state.rs:100` remains
  `comparison_open: true`.
- Final fixed-point result: no new admissible correction.
