# Practiced Constitution — 2026-07-12

This is the crash-safe ledger for the adversarial behavioral audit begun at
commit `a7180ae2` (`Close constitutional Examen at fixed point`). The prior
Examen established structural ownership; this run asks whether the promised
wholes survive difficult event orders, boundary values, failure, absence,
replacement, and departure.

## Protocol

No framework correction may begin until the complete witness map and scenario
census are recorded. Existing fixtures, fake backends, deterministic random
generators, injected clocks, and reference models must be reused before any
test-local machinery is added.

A failure authorizes a correction only when it is deterministic, reduced to a
small sequence, contradicts current doctrine or practiced behavior, has an
existing owner, and can be corrected without public API, feature, product,
hardware, or visual decisions. At most eight independently green framework
corrections may be admitted.

Public API and behavior are frozen by default. No roadmap feature work, push,
permanent benchmark dependency, general test framework, sleep-based timing
assertion, speculative `layout::Frame` reorganization, or overlay housing split
is authorized. The protected glass-tuner state remains
`comparison_open: true`.

## Baseline

- Commit: `a7180ae2`.
- Worktree: clean.
- Prior Examen health: 135/140, with interface state-integrity, overlay
  navigation, native hardware evidence, and the first-frame show-cycle contract
  carried as explicit limits rather than correction authority.
- Most recent library verification: 804 passed, 8 deliberately ignored,
  0 failed.
- Most recent example checks: `text_editor`, `control_gallery`, and
  `glass_tuner` passed.
- Baseline checks will be rerun as part of the execution matrix rather than
  treated as proof of the adversarial cells below.

## Constellation promises

| ID | Constellation | Behavioral promise |
| --- | --- | --- |
| PC-1 | Document truth | Persistent text and explicit edit state remain versioned, undoable, and safely persisted through deferred and failed work. |
| PC-2 | Capability | Typed requests resolve through the right scope and target, then report effects and history exactly once. |
| PC-3 | Interface | Declarative descriptions retain the right identity, geometry, focus, drafts, capture, and scroll behavior through replacement. |
| PC-4 | Presentation | Invalidation produces the right logical frame and portable/native outputs across time, scale, fallback, and departure. |
| PC-5 | Application | Runtime, shell, host, platform, tasks, requests, windows, and lifecycle remain coherent under reordered and repeated events. |
| PC-6 | Native boundary | Logical facts cross once into physical coordinates, paint, surfaces, cursor/IME hosts, formats, and OS capability choices. |
| PC-7 | Root vocabulary | Shared facts preserve round trips, endpoints, merge laws, identity, and dependency meaning wherever constellations meet. |

## Witness classification

Each cell receives one primary classification:

- `Whole` — exercises the complete public promise across owners.
- `Mechanism` — checks one component or transition only.
- `Structural` — checks source shape, privacy, or dependency direction.
- `Hardware/manual` — requires a GPU, native compositor, timing environment,
  or human perception.
- `Duplicate` — recomputes production reasoning as a competing oracle.
- `Missing` — no current witness reaches the required adversarial case.

The lifecycle table abbreviates these as `W`, `M`, `S`, `H`, `D`, and `Gap`;
`—` means the axis has no honest referent in that constellation.

## Lifecycle-axis census

Axes: identity (`I`), ordering (`O`), time (`T`), absence (`A`), failure (`F`),
cancellation (`C`), repetition (`R`), replacement (`X`), departure (`D`),
restoration (`S`), coordinates/scale (`K`), and concurrent/deferred completion
(`Q`). Receipts and classifications are filled during Phase A.

| Cell | Constellation | I | O | T | A | F | C | R | X | D | S | K | Q |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W-01 | Document truth | W | W | M | W | W | W | W | W | — | W | — | W |
| W-02 | Capability | W | W | M | W | W | W | W | W | W | W | — | W |
| W-03 | Interface | W | W | W | W | M | W | W | Gap X-01 | W | W | W | M |
| W-04 | Presentation | W | W | W | W | W | W | W | W | W | W | M / X-04 | W |
| W-05 | Application | W | W | W | Gap X-02 | Gap X-03 | W | W | W | W | W | — | W |
| W-06 | Native boundary | W | W | W | H | W | — | W | W | W | W | M / X-04 | W |
| W-07 | Root vocabulary | W | W | W | W | W | — | M / X-06 | W | — | W | M / X-04 | W |

### Witness receipts

`W-01 — Document truth`

- Identity, replacement, ordering, and deferred completion are whole-promise
  witnesses in `src/tests/document_editor.rs:450`, `:619`, `:670`, and `:711`:
  captured revisions stay dirty after newer edits, replacement identities
  reject old completions, and the newest save generation wins.
- Clipboard failure versus empty, optimistic paste availability, atomic replace,
  and persistence failure are practiced at `src/tests/document_editor.rs:301`,
  `:330`, `:426`, `:490`, and `src/tests/runtime_tests.rs:461`.
- Focus-scoped history and restore cancellation are practiced at
  `src/tests/document_editor.rs:194` and `:777`; the 100k acceptance property
  supplies the long edit/undo/redo oracle.

`W-02 — Capability`

- Nearest, hidden-fallthrough, disabled-claiming, and ambiguous responder cases
  are whole routing witnesses in `src/tests/responder_tests.rs:4-141`.
- Declined sets, duplicate/replaced shortcuts, captured palette focus, state
  queries, observers, and stale-focus clearing are practiced throughout
  `src/tests/commands.rs:4-1146`.
- Zero-to-many notification ordering and departure delivery are practiced at
  `src/tests/notifications.rs:86-141`; history time windows are currently a
  mechanism witness rather than a long boundary matrix.

`W-03 — Interface`

- Retained reorder/removal identity is practiced by
  `src/tests/composition_tests.rs:4-108`; focus restoration and mixed control/text
  history are whole journeys in `src/tests/widget_focus_tests.rs:57-631`.
- Capture, cancel ordering, preedit, scroll, and drag routing are practiced in
  `src/tests/interaction_tests.rs:303-1101`; close-during-slider-capture purges all
  per-window state at `src/tests/widget_slider_tests.rs:150`.
- Draft independence/retention and clip/reveal/scroll behavior are practiced in
  `src/tests/widget_text_box_tests.rs:418-959` and
  `src/tests/layout_scene.rs:279-2985`.
- Gap X-01: production prunes hover, press, capture, scroll, and text drafts when
  reconciliation removes their identities (`src/interaction/mod.rs:267-297`),
  but no behavioral test removes the active target mid-interaction and then
  delivers the late pointer event.

`W-04 — Presentation`

- Overlay first/entering/live/exiting/expired/reopened states, zero-duration
  cancellation, ordering, native fallback, and afterlife caps are practiced in
  `src/overlay.rs:778-1249`, including exact 4,999/5,000/5,001 ms boundaries.
- Parent popup work, popup-local IME geometry, non-presentational pointer work,
  and per-parent stale cleanup are practiced at
  `src/tests/platform_tests.rs:888-1078` and `src/platform/native/popup.rs:725-770`.
- Layout-to-paint scale change and moving/resting geometry have mechanism and
  endpoint witnesses in `src/platform/native/paint.rs:549-795` and
  `src/render/quad.rs:880-925`. X-04 will execute the required four-scale matrix
  and long deterministic endpoint load as one recorded family.

`W-05 — Application`

- The suite has 31 runtime, 11 host/shell, and 24 platform test journeys. They
  practice start-once, poll, task execution/completion/cancellation, requests,
  multi-window revision staleness, repeated drains, close, and restore
  (`src/tests/runtime_tests.rs`, `src/tests/host_shell_tests.rs`, and
  `src/tests/platform_tests.rs`).
- Gap X-02: `Host` explicitly drops events and dialog results for unknown
  windows (`src/host/mod.rs:99-116`), but no behavioral witness delivers them
  after departure.
- Gap X-03: public platform operations return backend errors, while the reusable
  `FakeBackend` can only succeed. Error source formatting is tested, but an
  operational backend failure has no journey witness. The current contract is
  propagation and terminal handoff, not retry or rollback after failure.

`W-06 — Native boundary`

- Physical input conversion, per-window scale, popup-local coordinates, cursor
  host, popup IME routing, capability fallback, surface reconfiguration, alpha
  convention, and popup packing all have behavioral witnesses in
  `src/tests/platform_tests.rs:4-505`, `src/platform/native`, and `src/render`.
- Device-grid focus outsets already cover 1.0, 1.25, 1.5, and 2.0 at
  `src/paint/grid.rs:347`; X-04 broadens the recorded adversarial scale family
  beyond that single decoration property.
- Six ignored renderer/native witnesses require a GPU adapter or readback.
  Hardware availability will be attempted and reported, never inferred.

`W-07 — Root vocabulary`

- Window facts, state revisions/snapshots, keymap profiles, color transfer,
  animation endpoints, response invalidation precedence, logical/physical paint
  types, and grid snapping each have focused mechanism witnesses.
- X-06 will run 10,000 deterministic cases for the applicable algebraic and
  endpoint owners instead of mistaking a handful of examples for long-sequence
  evidence. It will assert governing laws, not duplicate their algorithms.

## Required scenario families

| Cell | Family | Census | Execution | Receipts / result |
| --- | --- | --- | --- | --- |
| S-01 | Document edits, history, save snapshots, out-of-order completion, persistence failure, clipboard, focus, departure | Complete | Pending | Existing whole journeys plus 10k/100k reference tiers. Document has no independent departure apart from its window/application owner. |
| S-02 | Hidden/disabled/declined commands, nested responders, captured focus, palette, grouping, observers, target departure | Complete | Pending | Existing responder, command, notification, and close/stale-focus journeys cover the family. |
| S-03 | Retained reorder/removal, focus traversal, pointer capture, drafts, scrolling, reveal, clipping, replacement | Complete with X-01 | Pending | Add one reduced removal-during-active-interaction witness using current reconciliation/pruning mechanics. |
| S-04 | First/entering/live/exiting/reopened overlays, fallback, redraw-only work, multi-parent, scale, departure | Complete with X-04 | Pending | Existing overlay and popup cases cover lifecycle; run the consolidated endpoint/scale load. |
| S-05 | Start, poll, task completion, requests, window lifecycle, stale events, repeated drain, backend failure, shutdown | Complete with X-02/X-03 | Pending | Add stale-after-departure and terminal backend-error propagation journeys; clean shutdown is window departure because no broader shutdown protocol exists. |
| S-06 | 1.0/1.25/1.5/2.0 conversion, reconfiguration, alpha, cursor/IME hosts, fallback, hardware absence | Complete with X-04/X-05 | Pending | Run four-scale matrix and attempt all six GPU/readback witnesses. |
| S-07 | Geometry/color/keymap/animation/effect/revision/window-fact round trips and boundary laws | Complete with X-06 | Pending | Add long deterministic law checks only for owners with stable observable invariants. |

## Minimum-breadth ledger

| Gate | Required | Result |
| --- | --- | --- |
| Full lifecycle table for all seven constellations | 7 | Pending |
| Adversarial sequence for every meaningful lifecycle axis | Complete census | Pending |
| Deterministic operations per applicable pure state machine | At least 10,000 | Pending |
| Multi-step application/platform traces | At least 25 | Pending |
| Coordinate scale matrix | 1.0, 1.25, 1.5, 2.0 | Pending |
| Transition boundary matrix | Immediately before, at, and after endpoints | Pending |
| Existing library suite | Full | Pending |
| Example smokes | 3 | Pending |
| Ignored 100k text reference property | 1 | Pending |
| Release text acceptance benchmark | 1 | Pending |
| GPU/native diagnostic tiers | Attempt when available | Pending |
| Replay every reduced failure | All | Pending |

## Existing mechanics inventory

- `src/text/acceptance.rs` has the fixed seed `0xd1b5_4a32_d192_ed03`, a String
  and line-start reference model, 100,000 edit/undo/redo operations, and a
  release-only 8 MiB load/typing/clone benchmark.
- `src/text/buffer/document/span_tree.rs:881` has seed
  `0x9e37_79b9_7f4a_7c15` and 10,000 edit/reference/invariant operations.
- `src/tests/mod.rs:245-389` supplies `FakeBackend`, backend event capture,
  native-popup capability selection, and common input/presentation helpers.
- `Runtime::render_scene_at`, `overlay::Store::update_window`, animation
  transitions, and `SysApplicator::due` all accept explicit `Instant` values.
- `platform::Events` translates winit events without a native event loop and
  maintains per-window pointer/scale state.
- The three examples expose `--smoke`; the text editor, gallery, and tuner are
  also compiled into the crate as external-style fixtures.
- Exact scale owners are `paint::Grid`, typed paint areas/points, native paint
  projection, and `platform::Events`; no test-local conversion owner is needed.
- Renderer diagnostics contain six ignored GPU witnesses; the text engine adds
  the two ignored reference/benchmark tiers.

## Planned execution cells

| Cell | Execution |
| --- | --- |
| X-01 | Remove a captured/pressed/scrolling retained target during rebuild; verify reconciliation prunes all interaction state and a late release is inert. |
| X-02 | Close a host window, then deliver stale window and dialog events; verify no state or work resurrects. |
| X-03 | Use a local failing backend to prove an operational error crosses `Platform` exactly as `Error::Backend`; make no retry-policy claim. |
| X-04 | Run 10,000 deterministic animation/overlay/paint endpoint cases and the 1.0/1.25/1.5/2.0 scale matrix. |
| X-05 | Attempt the six ignored GPU/native witnesses individually and record adapter availability/failures. |
| X-06 | Run 10,000 deterministic response-effect, transition/schedule, settle-state, and coordinate-law cases without reimplementing production algorithms. |
| X-07 | Run all 66 existing runtime/host/platform journeys, the full suite, and all three example smokes. |
| X-08 | Run the ignored 100k text reference property and release acceptance benchmark. |

## Execution ledger

| Run | Scenario cells | Seed / operations / matrix | Result | Evidence |
| --- | --- | --- | --- | --- |
| E-001 | X-01 removal during active command capture | Reduced four-step sequence: press slider → command changes model → rebuild omits slider → inspect interaction/gesture | Failed deterministically | Pointer hover/press/capture were pruned, but `window_residues(window).gesture` remained 1. Focused test: `tests::interaction_tests::rebuilding_away_captured_command_prunes_pointer_and_history_gesture`. |
| E-002 | X-01 replay and C-01 ritual | Same reduced sequence; full library and three examples | Held | Reduced test passed. Library: 805 passed, 8 ignored. All example checks, formatting, diff check, and protected-state check passed. |

## Failure and reduction ledger

| ID | Initial failure | Deterministic reduction | Governing promise | Disposition |
| --- | --- | --- | --- | --- |
| F-01 | A coalesced command gesture outlives the composition identity and pointer capture that own it. | One slider, one press, one state-driven view replacement. No timing, OS, random input, or second control is required. Reconciliation clears pointer state but leaves runtime gesture residue 1. | Interface replacement must remove ephemeral interaction state; runtime gesture lifetime cannot exceed its captured target. The existing close-mid-drag witness discards the same unfinished gesture (`src/tests/widget_slider_tests.rs:150`). | Admitted as C-01. |

## Candidate ledger

The witness map and scenario census are complete. Test gaps X-01 through X-06
authorize test-local evidence, not framework behavior. Framework corrections
still require a reduced failing sequence.

| ID | Finding | Admission evidence | Existing owner | Displaced path | Disposition |
| --- | --- | --- | --- | --- | --- |
| P-01 | Glass-tuner smoke waits 100 ms in wall-clock time although internal presentation accepts injected time. | The attempted reduction proved `Runtime::render_scene_at` is crate-private, while the example binary deliberately exercises only the external API. | Internal injected time and the external smoke are distinct surfaces. | None without widening public API, adding a configuration mode, or weakening the overlay-content assertion. | Withdrawn. Keep the current witness and record public deterministic presentation as a limit; the no-sleep rail applies only where injected time is actually available. |
| P-02 | Active interaction removal has production pruning but no whole-promise witness. | X-01 failed in E-001: `src/interaction/mod.rs:267-297` prunes capture, while the separate gesture in `src/runtime/transaction/gesture.rs` survives. | Runtime gesture lifecycle coordinated from reconciliation. | The path where capture disappears but `Runtime::gesture` remains `Some`. | Promoted to C-01. |
| P-03 | Stale host events and backend failure propagation lack journeys. | X-02/X-03 and the existing branches in `src/host/mod.rs:99-116` / `src/platform/mod.rs:139-210`. | Existing Host and Platform boundaries. | None. | Add local witnesses; do not invent retry or rollback policy. |
| C-01 | Reconciliation must cancel an unfinished command gesture when it removes that gesture's captured target. | F-01 plus existing close-mid-drag policy. The failure is deterministic and reduced. | `runtime::transaction::gesture`; interaction reconciliation reports the causal capture removal. | Ownerless gesture state after capture pruning. | Corrected and independently green. `interaction::Pruned` distinguishes causal capture removal; runtime discards only the gesture belonging to that window. |

## Correction ledger

| ID | Commit subject | Hash | Files | Insertions | Deletions | Witness and outcome |
| --- | --- | --- | --- | --- | --- | --- |
| C-01 | `C-01 Cancel gestures whose capture is pruned` | Final checkpoint | 6 | 147 | 10 | `rebuilding_away_captured_command_prunes_pointer_and_history_gesture`; old belief: pruning pointer state was sufficient. New belief: the runtime gesture cannot outlive its captured command target. Owner: runtime gesture lifecycle, informed by interaction's causal prune result. |

## Deliberately untouched limits

- `layout::Frame` remains flag-only without a reproduced contradiction.
- Overlay housing remains intact unless behavior reveals an independent owner.
- Roadmap item 22 non-merges require new evidence.
- Native show-cycle item 2/21 remains feature work, not an audit correction.
- Hardware/manual absence will be reported honestly.

## Fixed-point sweep

Pending complete census, execution, reduction, replay, and final admission
sweep.
