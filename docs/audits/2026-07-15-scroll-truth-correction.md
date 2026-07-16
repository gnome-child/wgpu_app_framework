# Scroll Truth correction — One Position, Complete Pixels, Independent Residency

Status: **active**. This correction follows the completed retained-renderer
campaign and starts from production HEAD `66c84945`. It owns the scroll,
virtual-residency, scene-property, frame-scheduling, and pending/active
presentation seams until the exit theorem below is proved.

This is an architectural correction, not a compatibility patch. It supersedes
the retained-renderer campaign's classification of virtual cache membership as
semantic scene structure. Public APIs remain unchanged unless a real external
caller appears; internal call sites may move when the practiced ownership model
requires it.

The field defects that ignited this correction and the subsequent code audit
are seed evidence, not an exhaustive defect list and not the campaign boundary.
The campaign repeatedly re-censuses every scroll entrance, owner, projection,
consumer, axis, control species, backend, and failure path. A green seed queue
does not close the campaign; only the fixed-point exit theorem does.

## Indictment

The framework owns one logical session offset but projects it into resident
layout, scene properties, and pending/active presentation without one atomic
admission contract. Four invalid states are consequently representable:

1. an integral logical offset is narrowed to `f32`, then rounded back to a
   different integer at large content extents;
2. layout or scene state may be drawable without proving that every requested
   virtual row and every required viewport pixel is resident;
3. an older prepared semantic state may activate after a newer accepted scroll
   state exists, visibly restoring the older offset;
4. ordinary virtual cache replenishment mints a semantic commit, so continuous
   desired movement can outrun every prepared commit and force a choice between
   stale snap-back and activation starvation.

Wheel input and scrollbar-thumb input mutate the same session owner but pass
through duplicated transition logic, allowing their scheduling behavior and
event receipts to diverge. The runner also owns one process-wide presentation
pulse even though window presentation epochs and popup generations are local.

## Non-exhaustive ignition evidence

These findings establish that a system correction is necessary. They may be
split, merged, generalized, or displaced by deeper ownership findings during
the campaign:

- layout feedback and presentation receipts can write interaction-owned
  desired/admitted offsets, and an existing pointer-drag witness demonstrates a
  newly admitted offset returning to an older layout value;
- `TextArea` scroll and other transient view fields participate in semantic
  scene equality, so text property movement can mint content work that generic
  and table scroll avoid;
- text layout admits a bounded resident runway while glyph preparation is
  clipped to the current visible rectangle, allowing drawable residency to
  promise pixels that the GPU has not prepared;
- the horizontal text render surface grows with absolute `scroll_x`, and
  unwrapped width measurement may flatten and measure the whole document;
- text scroll crosses an integral-to-float-to-integral boundary and variable
  line-height refinement has no stable anchor correction path;
- text and table scrollbar visibility/activity differ because their node
  topology selects different scroll targets rather than one explicit two-axis
  chrome policy; and
- table rules and fixed viewport clips have lacked a pixel-level transition
  witness even where their layout rectangles and property values look correct.

The first re-census begins with these cells but is required to search beyond
them. A newly discovered defect is admitted by evidence and ownership impact,
not by resemblance to this list.

## Constitution

> Scroll has one authoritative interaction owner. Requested movement and
> admitted position are different lifecycle facts inside that owner. Input
> advances desired movement immediately; only a position covered by complete
> active residency becomes the admitted `ScrollOffset`.

> A scrollable presentation is drawable only when immutable scene residency
> proves complete pixel coverage for the integral property value it presents.

> Residency revision is a fourth handoff, disjoint from semantic commit,
> property tick, and presentation event. Activation may change residency; it
> may never regress the newest accepted scroll property, starve behind a farther
> desired value, or borrow another window or popup clock.

This corrects one retained-renderer ruling while preserving its other laws:
semantic structure remains in `scene::Commit`, bounded virtual realization
belongs to `scene::Residency`, values remain in `scene::Properties`, and
presentation alone owns activation and visibility. Events mutate desired
interaction truth immediately, `RedrawRequested` samples admitted truth once,
and only a successful receipt promotes visible geometry.

## Ownership and representation

### Requested and admitted interaction state

`interaction` / `session` owns scroll mutation. Relative wheel deltas, absolute
thumb positions, programmatic scrolling, and layout-derived extent corrections
enter one `ScrollUpdate`. The owner accumulates a desired integral offset and
retains a distinct admitted integral `ScrollOffset`.

Runtime asks last-presented layout whether the desired value is accepted by
active residency. It either admits that value and requests a property tick or
keeps it pending and requests residency. Runtime scheduling does not duplicate
mutation policy. Relative deltas accumulate against desired state, so delaying
admission never loses input.

For provisional variable-height geometry, a stable row anchor and its within-
row displacement may resolve the same desired logical position after a geometry
revision. That anchor is a resolution rule, not a second offset authority:
virtual layout proposes the correction and the interaction owner applies it
through the same request/admission path.

### Integral property law

Desired and admitted logical scroll coordinates remain `ScrollOffset` / `i32`
through interaction, layout, scene residency, properties, projection,
validation, activation, and receipts. The renderer may convert only the bounded
difference between a residency baseline and the active offset to GPU `f32`.
Total content extent never crosses that floating-point boundary.

### Exact residency law

Virtual residency is a proved snapshot, not a bounding-box inference. The
layout proof binds:

- the existing composition identity and interaction target of the scroll owner;
- the requested contiguous index range;
- the actual realized row identities, provider keys, and indices;
- ordered row rectangles and their contiguous pixel coverage;
- the viewport, integral baseline offset, and admitted property interval.

No independent materialization identity is minted. The request and realized
rows are checked in the same immutable layout snapshot. Scene projects that
proof into one immutable `scene::Residency` for the scroll `NodeId`.
`scene::residency::Revision` is its local structural currency. It is not
semantic identity, application content revision, property serial, presentation
epoch, popup generation, or renderer-generated key.

A virtual snapshot is complete only when every requested index occurs exactly
once, row geometry is ordered and gap-free across the visible axis, and every
required viewport pixel is covered. Distant focus/menu pins are retained
content but cannot contribute to contiguous residency.

Drawable completeness includes every painted descendant of that range, not
only row backgrounds and rules. Retained text must shape, allocate, and prepare
glyph vertices over the bounded resident runway; culling preparation to the
current surface would admit offsets whose rows exist structurally but whose
text cannot draw.

An incomplete snapshot is `NotReady`. It cannot silently omit a scroll
declaration, create drawable scene residency, update presented geometry, or
activate. Reaching the refinement limit with incomplete coverage is a rejected
candidate, not a warning followed by presentation.

### Fixed viewport clip law

The viewport coverage clip encloses its scroll transform and therefore remains
fixed in target space while resident content moves beneath it. Row-local and
content-local clips may move with their scroll scope. Paint may close and
reopen scroll scopes to change a fixed clip, but it may not translate the
viewport clip with the rows or extend table rules beyond complete content.

### Progress-preserving activation law

When prepared residency `B` completes while a farther desired offset `C`
exists, presentation chooses the newest offset admitted by `B`.

- If `B` accepts `C`, it activates with `C`.
- If `B` accepts only an intermediate offset, it may activate forward with the
  newest compatible value while `C` remains pending and continues preparing.
- If `B` accepts no forward compatible offset, it does not activate; the
  complete active state remains drawable while the newest target prepares.

While `B` prepares, the active stack projects `C` to the furthest value inside
its own accepted interval, independently on each axis. It never treats one
out-of-range component as a reason to reuse the entire older property snapshot.
The unclamped `C` remains desired interaction truth and continues preparation;
only the projected active value may receive an intermediate presentation.

Generic best-effort property projection is insufficient for this decision. A
missing or rejected admitted scroll value is an activation veto, while a farther
desired value is pending intent rather than proof that an intermediate forward
activation is stale. Successful presentation receipts bind the exact layout,
semantic commit, residency revisions, and property snapshot that drew.

### Clock locality law

Application revision, semantic commit revision, residency revision, property
serial, window presentation epoch, and popup generation name different facts.
A residency revision is local to one scroll `NodeId`. In-frame panels share the
parent window's frame cadence but retain independent scroll residencies. Native
popups retain popup-local generations; parent presentation activity cannot make
their content stale, and pending parent preparation cannot block independently
ready popup realization.

The native runner retains one `PresentationPulse` per `window::Id`. Deadline
wakeups request redraw for the affected window; they do not present directly.
`RedrawRequested` remains the platform frame boundary.

### Unified input law

Wheel, touch/trackpad deltas, scrollbar tracks/thumbs, and programmatic absolute
scroll requests share one transition result:

```text
Unchanged | PropertyTick(offset) | NeedsResidency(offset)
```

`PropertyTick` means the offset was admitted. `NeedsResidency` means the offset
is retained only as desired movement; it cannot reach properties, scrollbar
projection, hit testing, or a successful receipt until admitted.

Horizontal and vertical scrollbars consume the same axis-generic geometry,
capture, mutation, and scheduling path. Axis-specific arithmetic may select a
component; it may not select different ownership or presentation physics.

### Bounded-work performance law

Correct scroll architecture must also make the cheapest truthful path the
ordinary path:

- an admitted offset already covered by active residency is a property update;
  after warmup it performs zero view rebuilds, layout recompositions, semantic
  commits, scene assemblies, text shaping/preparation, retained-resource
  creation, and geometry-buffer creation;
- a residency crossing performs work proportional to the newly exposed strip
  and its descendants, never to absolute offset, total document length, total
  table length, or the whole already-resident runway;
- horizontal and vertical text targets remain bounded by viewport plus named
  runway guards. No allocation, texture, buffer, shaping request, or cache key
  may grow as `viewport + absolute_scroll`;
- scrolling a stable document does not rescan, concatenate, hash, clone, shape,
  or measure the whole document. Global extents are incrementally maintained or
  revision-keyed and reused;
- generic, table, and text controls may have different content economics but
  share one property/admission/presentation fast path; and
- metrics are code-owned, release-meaningful, allocation-free on the measured
  hot path, and disabled or bounded outside diagnostics. Instrumentation may not
  become a second scheduling or dirty-state authority.

Timing ceilings are acceptance floors, not permission to stop optimizing. The
campaign continues until the performance fixed-point sweep below finds no
remaining unbounded work, duplicated work, false invalidation, avoidable hot-
path allocation, or evidence-backed optimization with material benefit.

## Operating mandate

The campaign is an examination loop, not a finite symptom checklist:

**Census -> Trace -> Model -> Measure -> Challenge -> Admit -> Correct ->
Optimize -> Prove -> Ratchet -> Re-census.**

1. **Census** one bounded scroll cell and enumerate every relevant entrance,
   output, owner, consumer, lifecycle, coordinate space, clock, backend, and
   failure path.
2. **Trace** the fact from native input or programmatic intent through session,
   runtime, layout, scene commit/residency/properties, retained realization,
   presentation, receipt, hit testing, and chrome.
3. **Model** the current and proposed owner graphs, including identities,
   revisions, caches, bounds, allocation lifetimes, and pending/active states.
4. **Measure** semantic work, CPU latency, GPU work, cadence, memory high-water,
   and scaling behavior before selecting an optimization.
5. **Challenge** every conversion, feedback edge, cache key, clone, allocation,
   full-range walk, invalidation, branch by widget species, optional state, and
   rectangle-only proof encountered by the trace.
6. **Admit** a correction or optimization only with a named violated law,
   reproduced output defect, structural waste, complexity failure, or measured
   owner. A zero-change ruling is still recorded.
7. **Correct** the earliest owner that makes the paths disagree and delete the
   displaced path rather than synchronizing two truths.
8. **Optimize** the corrected path from structural work removal outward: first
   zero false work, then bounded incremental work, then cache/resource reuse,
   then data layout and micro-optimization.
9. **Prove** pixels, state, work counts, bounds, timing distributions, cadence,
   and cleanup with the narrowest deterministic witness plus proportional GPU
   and native evidence.
10. **Ratchet and re-census** with architecture witnesses and receipts, then
    select the next highest-value cell. Clearing the ignition evidence merely
    unlocks the next sweep.

Every sweep covers at least these lenses:

| Lens | Required census |
|---|---|
| Entrances | wheel, precision/touch deltas, scrollbar track/thumb, keyboard, programmatic scroll/reveal, caret/selection reveal, focus/accessibility movement, resize and scale correction |
| Controls | generic scroll, fixed and virtual tables, wrapped and unwrapped text, editable/read-only text, nested scroll owners, panels, windows, and native popups |
| Axes | vertical, horizontal, diagonal, independently saturated axes, RTL/bidi where horizontal coordinates participate |
| Handoffs | interaction, session, runtime routing, layout, scene commit, residency, properties, retained resources, pending/active presentation, receipts, hit testing, and chrome |
| Transitions | warm property tick, residency boundary, large jump, reversal, acceleration, resize, scale/backend/device change, content edit, provider shrink/reorder, focus/capture pin, activation race, occlusion, and device loss |
| Economics | asymptotic work, allocations, clones/hashes, shaping/measurement, uploads, resource churn, passes/draws, memory high-water, input-to-submit/present latency, and missed cadence |

## Industry reference line

The named frameworks are mandatory comparison witnesses and a floor for mature
behavior, not authorities to copy. The campaign records an explicit comparison
at baseline, after the owner/residency correction, and at final fixed point. A
framework pattern may inspire a cell; it is admitted only when it fits this
repository's one-owner, integral-coordinate, complete-pixel, and disjoint-clock
laws. Conversely, a different implementation here requires evidence that it
meets or exceeds the relevant industry property.

| Reference | Industry-standard property to challenge against | Campaign use, not imitation |
|---|---|---|
| [Chromium compositor](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/how_cc_works.md) and [input](https://chromium.googlesource.com/chromium/src/+/HEAD/cc/input/README.md) | Main/pending/active/recycle state, property-driven compositor movement, readiness before activation, and visible/near-viewport raster prioritization | Challenge commit/residency/property/activation separation, active progress during preparation, retained reuse, and zero semantic work. Do not import Chromium process/thread complexity or float total-extent coordinates. |
| [GTK `Scrollable`](https://docs.gtk.org/gtk4/iface.Scrollable.html), [`Adjustment`](https://docs.gtk.org/gtk4/class.Adjustment.html), and [`TextView`](https://docs.gtk.org/gtk4/class.TextView.html) | One adjustment model per axis carrying value/range/page geometry, a common scrollable interface across list/table/text species, and explicit non-scrolling border | Challenge sole value ownership, per-axis parity, extent correction, and common chrome semantics. Do not treat GTK's scalar type or signal mechanics as architecture law. |
| [Qt `QAbstractScrollArea`](https://doc.qt.io/qt-6/qabstractscrollarea.html) and [`QPlainTextEdit`](https://doc.qt.io/qt-6/qplaintextedit.html) | One central viewport coordinated with both bars, content drawn from their values, as-needed visibility when range is non-zero, and paragraph/block-oriented large plain-text handling | Challenge the two-axis viewport solver, resize/gutter feedback, block text indexing, visible-block lookup, and incremental repaint. Do not copy QWidget repaint ownership or platform-specific policy. |
| [Iced scrollable source](https://docs.iced.rs/src/iced_widget/scrollable.rs.html) | One state owns both offsets and interaction; one viewport binds offset/bounds/content bounds; fixed visible layer encloses the negative content translation; both scrollbar axes derive from the same state | Challenge shared draw/hit/chrome coordinates, clip ordering, axis status, and topology independence. Preserve this engine's integral truth instead of adopting Iced's floating offsets. |
| [COSMIC toolkit](https://pop-os.github.io/libcosmic-book/) and [widget catalog](https://pop-os.github.io/libcosmic/cosmic/widget/index.html) | Production desktop integration built on Iced primitives, including scrollable, text editor, and table consumers | Treat COSMIC as the Iced integration/desktop-product witness, not a fictitious independent engine. Inspect any COSMIC policy or widget deviations that expose gaps in nested controls, theming, input, or accessibility. |
| [Firefox APZ/WebRender](https://firefox-source-docs.mozilla.org/gfx/AsyncPanZoom.html) and [rendering overview](https://firefox-source-docs.mozilla.org/gfx/RenderingOverview.html) | Low-latency property transforms, an independent tree of scroll owners, transform-consistent hit testing, and a bounded velocity/direction-informed displayport rather than whole-page residency | Challenge predictive runway sizing, nested-owner routing/handoff, input-coordinate projection, and uninterrupted active progress. Firefox documents checkerboarding as a failure to mitigate; this campaign keeps complete pixels as a hard gate rather than accepting it. |
| [Flutter `ScrollPosition`](https://api.flutter.dev/flutter/widgets/ScrollPosition-class.html), [`RenderViewport`](https://api.flutter.dev/flutter/rendering/RenderViewport-class.html), and [viewport-aware slivers](https://docs.flutter.dev/resources/inside-flutter#infinite-scrolling) | Position/activity and content dimensions have explicit APIs; viewport/cache extent drives finite on-demand realization of potentially unbounded content; layout correction is named | Challenge desired/admitted/dimensions separation, anchor correction, cache/runway extent, and viewport-proportional realization. Do not import floating authoritative offsets or interleaved build/layout where existing repository layers can express the proof more cleanly. |

Across these references, the campaign uses the following shared industry line as
a minimum challenge set:

1. ordinary scrolling changes a retained property/position rather than
   rebuilding semantic content;
2. a fixed viewport clips content transformed underneath it, and hit testing,
   scrollbars, fixed/sticky content, and paint consume compatible coordinates;
3. position/activity ownership is explicit, while layout supplies dimensions
   and corrections through a named contract instead of becoming a second owner;
4. large or unbounded content is realized from viewport/cache/runway demand and
   remains bounded independently of total extent;
5. prepared/active presentation remains usable while replacement content is
   produced, and incomplete pixels are not mislabeled ready;
6. both axes and all scrollable control species share a coherent viewport and
   input contract; and
7. large-text controls use block/line indexing and incremental visible-region
   work rather than a whole-document surface or scan on scroll.

Checkpoint receipts cite the exact upstream document/source revision and date
consulted. External evolution can open a comparison cell, but cannot silently
rewrite repository doctrine or make network access part of the test boundary.

## Checkpoints

### 0. Ratify the four-handoff contract and pin the baseline

- update master doctrine and supersede the historical semantic-residency ruling;
- retain established names and module visibility;
- plant structural witnesses for the fourth handoff, clock locality, sole stack
  renderer entrance, and absence of a second identity runtime;
- preserve every known red witness as ignition evidence rather than normalizing
  its expectation;
- add the code-owned scroll metric schema and deterministic workload driver;
- record cold and warm release baselines before performance edits, including
  environment, sample count, semantic work, latency distributions, cadence,
  resource high-water, and pixels.

### 1. Preserve integral requested/admitted truth

- split private `interaction::Scroll` entries into admitted and pending species;
- accumulate `ScrollUpdate` against desired state and admit only proved values;
- preserve integral coordinates on both sides of `2^24` and near the gallery
  maximum;
- convert only bounded renderer deltas to `f32`.

### 2. Make complete layout residency constructible

- retain `virtual_list::{Request, Materialization}` as the materialization
  vocabulary;
- require exact keyed row and pixel coverage in `layout::ScrollProjection`;
- reject non-converged/incomplete candidates before scene residency;
- derive the resident runway from viewport/cost bounds rather than a two-row
  magic default while retaining bounded nodes for one million rows.
- establish the fixed viewport clip outside moving scroll scopes.

### 3. Separate scene residency from semantic commits

- introduce `scene::Residency` plus its namespaced local revision;
- keep stable scroll topology in `scene::Commit` while resident row membership
  moves through residency revision;
- extend `scene::Stack` as the sole native handoff for commit, residency, and
  properties;
- synchronize resident membership incrementally while retaining GPU identity by
  carried `NodeId` and content/geometry revisions;
- prepare retained text over the full bounded resident runway while projecting
  glyphs into the actual target surface;
- remove scroll, reveal, caret blink, and other presentation-only text fields
  from semantic content equality and renderer content-resource identity;
- prove ordinary boundary crossings mint zero semantic commits.

### 4. Make activation monotonic and progress-preserving

- admit only scroll values covered by active or activating residency;
- allow one completed residency to activate the newest compatible forward
  offset even while a farther desired value remains pending;
- project a pending desired offset to the active residency boundary instead of
  reverting the whole scroll property to its prior value;
- retain the complete active state while the newest target prepares;
- receipt only the commit/residency/property/layout combination actually drawn.

### 5. Preserve local presentation clocks

- keep a `PresentationPulse` per `window::Id` and schedule the earliest due
  deadline without presenting outside `RedrawRequested`;
- preserve popup-local `popup::Generation` currentness and independently
  presentable popup work;
- prove slow parent residency cannot block popup realization and popup work
  cannot block parent scrolling.

### 6. Bound, predict, recycle, unify input, and optimize to fixed point

- size a directional runway from measured preparation time, refresh, input
  velocity/delta, viewport, and row heights;
- retain bounded current, preparing, and recycle states;
- replace absolute-offset-sized text surfaces with bounded two-dimensional
  windows and retain/recycle prepared text by stable line/block identity;
- maintain unwrapped width and variable height incrementally so scrolling never
  performs a whole-document text flatten/measure pass;
- make property-only updates allocation-free after warmup and make residency
  replenishment proportional to the entering strip;
- route relative and absolute input through one request/admission transition;
- prove horizontal/vertical scrollbar capture and mutation parity;
- optimize every measured owner above the campaign materiality threshold, then
  repeat the profile/census sweep until the fixed-point rule holds;
- remove the displaced semantic-replenishment, float, rectangle-only, global-
  pulse, stale-activation, and duplicated scheduling paths.

### 7. Runtime, resistance, and burn-down

- run focused owner, architecture, cross-layer, and pending/active tests;
- run workspace all-target, doctest, renderer-debug, and deep GPU tiers in
  proportion to changed code;
- repeatedly launch the release Control Gallery and exercise slow/fast/sustained
  wheel scrolling, large thumb jumps, both scrollbars, multiple panels/windows,
  column resizing, typing, and selection while checking process survival;
- run two complete post-seed re-census/profile sweeps, admitting or recording a
  resistance ruling for every newly exposed scroll or performance cell;
- prove the final performance receipts against both structural and timing gates;
- remove temporary diagnostics and commit/push each coherent checkpoint.

## Live campaign ledger

This queue is an ignition map, not a closed backlog. Every cell receipt records
the bounded question, traced paths, current/proposed owners, measurements,
admission or resistance ruling, implementation/deletion, witnesses, performance
delta, and the cells exposed by re-census.

| Cell | Initial question | Ignition evidence | Status | Close condition |
|---|---|---|---|---|
| S-000 | Whole-system scroll census | Three controls and several input paths exhibit different ownership/economics | Open | Every required lens has an entrance-to-receipt map; unknowns become cells |
| S-001 | Sole requested/admitted owner | Layout feedback and presentation receipts can restore an older offset | **Complete; R-001** | Only interaction mutates desired/admitted state; stale feedback cannot regress it |
| S-002 | Semantic versus transient text identity | `TextArea` property movement participates in semantic equality | **Complete; R-002** | Scroll/reveal/blink property ticks mint zero content, geometry, or topology revisions |
| S-003 | Exact drawable residency | Layout runway and prepared glyph bounds disagree | **Complete; R-003** | Admission and prepared pixels name the same integral region for every descendant |
| S-004 | Bounded text economics | Horizontal surface size and width work can scale with absolute offset/document length | Open | Warm movement is property-only; replenishment and storage are viewport/runway bounded |
| S-005 | Stable variable-height position | Refined line heights can move the visible anchor | Complete | R-005: one bounded resident line-anchor band plus within-line displacement resolves through `ScrollUpdate::Geometry` |
| S-006 | Table content/rule/clip unity | Rules and overflow lack a transition pixel oracle | **Complete; R-006** | Cells, backgrounds, rules, text, and fixed clip pass the same scroll pixel witness |
| S-007 | One two-axis viewport/chrome policy | Text and table activity/visibility depend on different node topology | **Complete; R-007** | Presence, gutter, activity, hit, capture, and fade policies are explicit and axis-parity proven |
| S-008 | Pending/active and clock locality | Residency, property, window, and popup progress can block or regress one another | Open | Activation is monotonic and each window/popup retains its local clock |
| S-009 | Scroll performance fixed point | Existing counters do not fully attribute admission, replenishment, text, allocation, or cadence cost | Open | Metric contract, workload matrix, gates, two clean optimization sweeps, and final receipt hold |
| S-010 | Native presentation atomicity | Fresh release capture intermittently observed a partially composed deadline frame | Open | Native property, residency, and semantic redraws expose only complete receipted frames; capture artifact versus presentation defect is resolved |
| S-011 | Text runway versus semantic identity | The first bounded horizontal runway crossing minted a new semantic commit because prepared text and transient rule geometry leaked through the owner node | **Complete; R-011** | Bounded text replenishment changes drawable/residency identity while stable owner content and property topology reuse the semantic commit |

### R-001 — sole requested/admitted owner

- **Bounded question and trace.** Wheel and absolute input, scrollbar drag,
  caret reveal, extent clamp, full scene preparation, property ticks, candidate
  receipts, and native active-refresh receipts were traced from runtime routing
  through `session`/`interaction`, layout projection, scene properties, and the
  installed presented stack. The final source census finds desired/admitted
  mutation only through `session::{request_scroll, admit_scroll}` into
  `interaction::Scroll`; view-node offsets are transient projections, not
  authorities.
- **Displaced owners.** Unconditional frame-baseline feedback was replaced by
  extent clamp and explicit reveal corrections through `ScrollUpdate::Geometry`.
  Full paint no longer forces commit baselines into live properties. Receipt
  admission consumes only the exact property carried by the successful stack;
  the layout fallback and duplicate frame walk were deleted.
- **Admission and activation ruling.** Legal intent is first normalized to the
  last-presented content extent. Active residency admits it immediately or
  retains it as desired. A newly prepared stack samples the newest compatible
  desired property; a property tick samples admitted truth. Candidate receipts
  must advance the window epoch. Active-refresh receipts must match both the
  acknowledged epoch and visible stack structure, so neither receipt species
  can regress admitted state.
- **Structural performance delta.** Ordinary relative and absolute requests at
  the content boundary now coalesce into one property tick with zero semantic
  commits, scene-node rebuilds, or scene-paint calls. A clamped no-op schedules
  no property or layout work. Quantitative release timing remains owned by
  S-009 rather than being inferred from unit-test elapsed time.
- **Witnesses.** `generic_scroll_feedback_clamps_session_offset_after_present`,
  `generic_scroll_pointer_drag_updates_viewport_offset`,
  `in_window_scroll_inputs_coalesce_into_one_literal_zero_property_tick`,
  `text_area_input_clamps_to_presented_extent_before_admission`,
  `text_area_caret_reveal_resolves_framework_owned_scroll_after_edit`, and
  `older_successful_scroll_receipt_cannot_regress_admitted_property` cover input
  parity, exact full-paint/property-tick sampling, explicit geometry correction,
  structural-zero work, out-of-order candidates, and stale active refreshes.
  The checkpoint suite is green after the stale-active-refresh ratchet:
  `cargo test --lib --quiet` reports 1,188 passed and 4 intentional ignores.
- **Re-census result.** Variable-height anchor correction remains S-005.
  Compatible forward projection and cancellation across changing residencies
  remains S-008. Exact drawable coverage remains S-003. No additional desired
  or admitted authority was found.

### R-002 — semantic versus transient text identity

- **Bounded question and trace.** Multi-line scroll/reveal/caret epochs and
  single-line caret epochs were traced from interaction projection through
  `view::SceneKey`, composition content revisions, layout, retained frame keys,
  scene content/properties, animation deadlines, renderer bindings, compatibility
  output, and the native runner. Selection, focus, preedit, text, and caret
  geometry remain semantic; position requests and blink phase do not.
- **Displaced identity and scheduling.** `TextArea::same_scene_state` excludes
  scroll, reveal, and caret epoch, while `TextBox::same_scene_state` excludes its
  caret epoch. Full model equality remains available for diagnostics. Caret
  visibility was removed from the retained paint-cache key and from conditional
  scene painting. The stable caret rule now declares one `Caret` content
  projection/property; compatibility output omits it when hidden and the retained
  renderer projects the same rule through per-content opacity. Caret deadlines
  have a property schedule distinct from paint/overlay/hover deadlines, so a due
  blink requests `FrameNeed::Properties` without weakening other animations.
- **Structural performance delta.** A resident text scroll retains the same
  semantic and drawable commit. Visible-to-hidden-to-visible blink advances only
  the property serial and exactly one caret property. The release GPU receipt
  reports zero scene-node realization rebuilds, primitive/text prepare or shape
  calls, content upload bytes, retained resource creates/replacements/removals,
  and render-plan rebuilds for both transitions. Only the bounded property upload
  and ordinary draw remain.
- **Witnesses.** `scroll_reveal_and_blink_epoch_are_not_scene_content_state`,
  `blink_epoch_is_not_scene_content_state`,
  `runtime_host_scroll_coordinates_route_to_scroll_target`,
  `focused_text_area_caret_blinks_and_schedules_next_deadline`,
  `text_box_caret_blinks_from_interaction_epoch`, and
  `caret_blink_is_one_projected_property_with_a_property_only_deadline` cover
  identity, revision reuse, scheduling, and both text controls. The release GPU
  witness `control_gallery_caret_blink_preserves_complete_output` passes exact
  visible/hidden/visible projection with literal-zero semantic retained work. A
  freshly rebuilt release Control Gallery also showed the caret present after
  focus, absent at the 500 ms phase, and restored at the 1,000 ms phase.
- **Post-closure resistance correction.** User observation exposed that the
  preceding native conclusion and GPU oracle were false-green. `PlanBuilder`
  preserved content projection for quads but replaced every retained `Rule`
  projection with `Normal`; because the caret is a rule, its property serial and
  compatibility scene changed while the production GPU batch stayed permanently
  visible. Both sides of the old GPU comparison shared that plan-builder defect,
  and the oracle never required a pixel delta. Retained rules now preserve their
  declared projection. The GPU transition witness additionally rejects any caret
  property change whose before/after pixels are identical. The ignored GPU tier
  passes with this ratchet, and a newly rebuilt release gallery shows
  present-at-focus, absent-at-500-ms, present-at-1,000-ms output. This correction
  re-establishes R-002; S-010 remains open because Windows capture can still
  sample a partially composed deadline frame even while the caret pixel itself
  follows the three expected phases.
- **Configuration and re-census result.** The closure suite reports 1,194 passed
  and four intentional ignores; renderer-debug's pure tier reports three passed
  and seventeen intentional GPU ignores; the workspace all-target/all-feature
  check is green. Native capture intermittently exposed a partially composed
  deadline frame, which is admitted as S-010 until capture artifact versus real
  presentation defect is proved. Bounded text layout/storage remains S-004,
  variable-height anchoring remains S-005, and broader clock/activation races
  remain S-008. No second text presentation identity was found.

### R-003 — exact drawable residency

- **Bounded question and trace.** Virtual requests, provider keys, realized row
  roots and descendants, text surfaces, table tracks, scene draw order, local
  residencies, explicit layer drawable selection, retained preparation, fixed
  clips, pending activation, and exact GPU output were traced from layout through
  the production retained renderer. The re-census covers ordinary and nested
  scroll scopes, distant pins, independently changing lists, property hits, and
  actual residency crossings.
- **Exact proof and rejection.** A virtual proof binds every requested index to
  the provider's current stable key in the same immutable layout snapshot.
  Missing, duplicate, reordered, stale-key, gap-producing, non-positive, zero-
  surface, and unmodelled multi-surface candidates cannot create drawable
  residency. Pins remain retained material but cannot extend contiguous proof.
- **One drawable and complete membership.** Each layer owns one explicit shared
  drawable commit beside its semantic commit; local `Residency` objects no
  longer select the drawable by incidental vector order. Membership contains
  exactly requested row roots and their nearest-scroll descendants, while a
  nested scroll carries its own local descendant proof. Residency draw order
  binds every member that actually paints. Thus one list may reuse its revision
  while another advances without selecting an obsolete shared drawable.
- **Integral pixels, glyphs, and clips.** Text paint and layout residency use one
  integral surface-rectangle conversion. Retained glyph preparation covers the
  complete resident surface, while a fixed target-space viewport clip encloses
  every moving scroll scope. Clip changes first close the moving scopes, so
  neither table rows nor text can translate the viewport clip and expose an
  unprepared edge band.
- **Structural performance delta.** Ordinary movement inside accepted residency
  remains a property-only retained-plan reuse; a boundary crossing advances a
  local residency and the explicit drawable while minting zero semantic commits.
  Quantitative latency, allocation, and memory comparison remains S-009 rather
  than being inferred from correctness-test elapsed time.
- **Witnesses.** The direct proof rejects missing, duplicate, reordered, stale-
  key, and gapped rows. `virtual_scroll_residency_does_not_bridge_a_distant_focus_pin`,
  `control_gallery_keeps_viewport_clips_outside_repeated_nested_scroll_scopes`,
  `one_scroll_residency_can_advance_while_an_independent_one_stays_reusable`, and
  the architecture ratchets cover membership, clocks, explicit drawable
  ownership, fixed-clip ordering, and full-runway text preparation. The release
  GPU witnesses `retained_scroll_tick_realizes_text_entering_from_the_runway`
  and `control_gallery_slow_scroll_never_exposes_unprepared_output` pass exact
  readback; the latter crosses at least one real residency boundary across 64
  consecutive four-pixel inputs and compares both pending active and activated
  output against independent realization.
- **Configuration and re-census result.** Production and test configurations
  compile with the presentation-epoch accessor available outside `cfg(test)`.
  The closure suite reports 1,191 passed and four intentional ignores, and the
  workspace all-target/all-feature check is green.
  Dedicated table rule/cell transition coverage remains S-006, bounded text
  storage and incremental width/height work remain S-004/S-005, and activation
  races beyond this slow-forward trace remain S-008. No additional drawable-
  residency owner, descendant class, or clip entrance was found.

### R-006 — table content, rule, and fixed-clip unity

- **Bounded question and trace.** Header and body cell frames, alternating row
  backgrounds, column and row tracks, cell text, divider hits, horizontal and
  virtual-body projections, shared interaction target, scene scroll ancestry,
  retained draw order, fixed clips, fractional scaling, resize, and entering
  side pixels were traced from table layout through compatibility output and the
  production retained renderer. Ordinary horizontal and diagonal property ticks
  and a real vertical residency crossing were included.
- **Owner ruling.** No second table paint offset was admitted. Column tracks own
  the same scroll ancestry as their header cells and therefore move only on the
  horizontal projection; row tracks own their row ancestry and therefore move
  on the shared horizontal plus vertical projections. Scene paint already binds
  rules through `Track::owner_node`, exactly as frames bind backgrounds and text.
  The open defect was an unratcheted cross-species pixel contract; S-007's shared
  target correction removed the last interaction/receipt path capable of making
  those otherwise-correct projections sample different admitted values.
- **Pixels and transitions.** The compatibility-scene witness now proves that a
  horizontal property tick translates the header background, alternating row,
  vertical rule, horizontal rule, and body text by the same exact x displacement
  while leaving semantic geometry untouched. A following vertical tick leaves
  the sticky header and column rule y fixed while translating the row background,
  row rule, and text by the same y displacement. The release GPU witness requires
  the narrow table's newly exposed right-side region to change on that property
  tick and match an independent retained realization at 1.0, 1.25, 1.5, and 2.0
  scale. The 64-step four-pixel slow-scroll oracle crosses real vertical
  residency while retaining exact complete output, and the resized expanded-table
  witness keeps one-pixel rules grid-aligned at all four scales.
- **Industry comparison ruling.** Against the upstream references recorded in
  the industry line (consulted 2026-07-15), this meets Chromium's retained
  property movement, GTK/Qt's content-from-shared-position and central viewport,
  Iced/COSMIC's fixed visible layer over one two-axis state, Firefox APZ/WebRender's
  transform-consistent retained scroll tree, and Flutter's finite viewport-driven
  realization. The repository keeps the stronger local requirements that the
  admitted coordinate is integral, incomplete residency is never drawable, and
  the entering strip is checked by exact pixels rather than inferred from a
  settled repaint.
- **Structural performance delta.** The horizontal transition retains both the
  semantic and drawable commits and reports zero scene-node realization rebuilds,
  primitive/text preparation or shaping, content uploads, retained resource
  creates/replacements/removals, and render-plan rebuilds. Quantitative timing,
  allocation, and memory comparisons remain S-009.
- **Witnesses and re-census.** `table_projects_minimum_tracks_once_and_scrolls_header_body_and_rules_together`,
  `expanded_resized_table_rules_remain_aligned_at_supported_scales`,
  `horizontal_table_scroll_updates_entering_pixels_at_supported_scales`, and
  `control_gallery_slow_scroll_never_exposes_unprepared_output` cover ancestry,
  per-species displacement, side-pixel immediacy, scale, resize, fixed clipping,
  and residency crossing. The checkpoint suite reports 1,201 passed and four
  intentional ignores; renderer-debug reports three passed and eighteen
  intentional GPU ignores; workspace all-target/all-feature and doctest checks
  are green. No additional table offset, clip, track, or overflow repaint
  authority was found. Bounded large-text economics and anchor stability remain
  S-004/S-005; activation races and quantitative fixed-point work remain S-008/S-009.

### R-007 — one two-axis viewport and chrome policy

- **Bounded question and trace.** Generic scrolls, table column/body projections,
  wrapped and unwrapped text areas, fixed clips, gutter reservation, scrollbar
  presence, activity/fade, hover thickness, hit testing, pointer capture, thumb
  mutation, keyboard reveal, layout feedback, property construction, retained
  projection, and successful receipt admission were traced on both axes. The
  trace included diagonal movement and independently saturated axes rather than
  treating vertical and horizontal tests as unrelated one-dimensional cases.
- **Displaced topology truths.** Gutter choice no longer derives from a node's
  stack layout axis, and text no longer bypasses the common visible-frame and
  visible-content calculation. A table's horizontal wrapper and virtual body
  now borrow the table element's one interaction target. Clamp, reveal,
  feedback, hit projection, and receipt admission aggregate every projection of
  that target instead of selecting or overwriting with the first/last one.
  `Ctrl+End` on a wide million-row table consequently retains both the far
  column and far row instead of the vertical receipt restoring x to zero.
- **Explicit activity and property policy.** Offset movement or interaction on
  either bar activates the viewport's shared activity/fade clock; hover and
  pressed thickness remain local to the individual bar. Runtime resolves all
  activity before projecting either bar, eliminating one-frame behavior based
  on chrome iteration order. Horizontal and vertical scrollbar properties use
  distinct property kinds on a shared text/generic frame, so scene and retained
  projection no longer fold one bar's hover thickness into the other. Tables
  receive the same semantics despite their bars living on separate layout
  projection nodes.
- **Integral chrome arithmetic.** Layout thumb sizing, property projection, and
  absolute drag use rounded integral ratios. The remaining layout-chrome `f32`
  conversion over total content extent was deleted; exact witnesses cover
  `16_777_215`, `16_777_216`, `16_777_217`, and three odd values near
  24,000,000 where the former calculation returned a different logical offset.
- **Industry comparison ruling.** The result meets the shared line represented
  by GTK's per-axis adjustments and common `Scrollable`, Qt's coordinated
  central viewport and as-needed ranges, and Iced/COSMIC's one state and fixed
  visible layer. Chromium's retained property movement and Firefox APZ's
  transform-consistent hit testing challenge the property/hit path; Flutter's
  explicit position/content-dimension separation challenges correction and
  admission. This implementation keeps stronger repository-specific integral
  offsets, complete-pixel admission, and per-axis property slots rather than
  importing any reference's floating authority or widget/process topology.
- **Structural performance delta.** A viewport offset is sampled once per
  shared scroll target during visual activity projection, and property
  construction emits one value per `(owner, axis)` without rescanning and
  max-folding every chrome sharing an owner. Two-axis property movement remains
  semantic-commit-free; quantitative CPU/GPU/allocation deltas remain assigned
  to S-009 and are not inferred from unit elapsed time.
- **Witnesses and closure.** `two_axis_scroll_gutters_follow_viewport_capability_not_stack_axis`,
  `text_area_uses_the_same_two_axis_gutter_geometry_as_other_viewports`,
  `text_area_projects_scrollbars_like_generic_viewports`,
  `two_axis_table_activity_and_fade_follow_one_scroll_owner`,
  `two_axis_table_scrollbar_capture_and_mutation_are_axis_symmetric`,
  `two_axis_text_scrollbars_share_activity_but_keep_per_axis_hover_and_mutation`,
  `table_keyboard_navigation_reveals_current_cell_across_horizontal_overflow`,
  `scrollbar_drag_preserves_integral_truth_past_f32_precision`, and the
  architecture ratchet cover presence, gutter, shared activity/fade, per-bar
  hover, hit, capture, mutation, diagonal reveal, receipt aggregation, and
  integral large extents. Release GPU witnesses
  `retained_scroll_tick_is_pixel_exact_and_reuses_all_content_work` and
  `control_gallery_slow_scroll_never_exposes_unprepared_output` remain green
  after the per-axis property split. The library suite reports 1,201 passed and
  four intentional ignores; renderer-debug reports three passed and seventeen
  intentional GPU ignores; the workspace all-target/all-feature check is green.
  Table rule/cell pixel unity remains S-006, bounded text economics and stable
  anchoring remain S-004/S-005, and quantitative fixed-point work remains S-009.

### S-004/S-009 checkpoint A — attributable input and text-window economics

- **Metric ownership.** The production diagnostic receipt now distinguishes
  unchanged input, admitted property hits, and residency requests; records
  desired/admitted axis lag and fixed-capacity request timing; and exposes text
  render-window bounds, resident source lines/bytes, shaping phases, height-index
  work, and unwrapped-width cache/source/measure work. The legacy counters remain
  compatible, while the transition enum carries the exact admitted value beside
  pending desired truth instead of reconstructing lag in the observer.
- **Exposed cache-identity violation.** The first cold/warm width witness failed
  because ordinary committed preedit projection clones its `Buffer`, clones
  intentionally mint a new editor identity, and the document-wide width cache
  used that editor identity. Unchanged unwrapped text therefore flattened and
  shaped the whole document on every layout. A typed `ContentVersion` now
  survives harmless persistent-document clones and advances on every real edit,
  including edits made independently on two clones. The width key consumes that
  version plus style; it cannot alias unrelated revision-zero buffers.
- **Structural delta and witnesses.** The cold no-wrap witness still records the
  currently admitted whole-source width pass (200 lines and every source byte),
  while the immediately repeated layout records one cache hit, zero misses, and
  zero source lines/bytes visited. `document_width_key_survives_clones_but_not_independent_content`
  ratchets the two identities directly. The library checkpoint reports 1,202
  passed and four intentional ignores. This is not S-004 closure: absolute-
  offset-sized horizontal surfaces and cold/edit-time whole-document width work
  remain red owners, and the versioned release `scroll-bench` matrix plus baseline
  receipts remain required before timing claims.

### S-004/S-009 checkpoint B — versioned long-line driver and false caret work

- **Code-owned workload.** `renderer_debug scroll-bench text-horizontal-1m`
  now drives the production text layout engine with a one-MiB editable unwrapped
  line and emits `scroll-bench-version=1`. The receipt names transition class,
  commit/profile/timer/OS/architecture, viewport, source shape, logical and
  absolute offset, official-matrix conformance, cold and warm p50/p95/p99/max,
  render-window high-water, width/render/cache/source/shape work, and caret run
  and glyph scans. The official defaults are 64 warmups and 1,024 samples;
  development sample counts are explicitly marked non-official.
- **Measured owner and correction.** The first five-sample release probe on the
  reference machine reported a warm median near 108 ms despite literal width and
  render-buffer cache hits. The owner was `cursor_position`: it cloned the entire
  shaped `glyphon::Buffer` only to construct a mutable editor for an immutable
  caret query. Borrowed layout-run projection removed that clone; a semantics
  witness matches cosmic-text's editor result across LTR, RTL, combining-cluster,
  and empty text. The ordinary end-of-line case now inspects one glyph rather
  than scanning all 1,048,576 glyphs. A following five-sample development probe
  reported a 2 us median, five caret runs, and five glyph inspections total.
  These small probes identify structural ownership and are not accepted as the
  campaign's three-trial official timing receipt.
- **Still-red structural fact.** The same receipt records a 1,176 px window at
  x=0 and a 5,159,085 px window around x=5,157,889, with 3,301,814,400 logical
  pixel area and `bounded_window=false`. Thus removing the caret clone does not
  normalize the absolute-offset-sized surface; bounded horizontal residency,
  entering-strip preparation, exact far-offset pixels, and GPU/storage high-water
  remain required for S-004.

### S-004/S-009 checkpoint C — bounded horizontal preparation runway

- **Earliest measured owner and correction.** The absolute-offset-sized surface
  came from using one rectangle as both prepared/cull bounds and glyph-buffer
  coordinate zero. `TextAreaSurface` now carries those facts separately. Its
  prepared rectangle is a 256 px bucketed window with one guard on each side;
  its text origin continues to address the shaped buffer. Scene and paint retain
  both values, and the text renderer culls/prepares against the bounded rectangle
  while positioning glyphs from the origin. This follows the industry line
  represented by Chromium's stable property/layer identity, GTK and Qt scene
  snapshots, and Iced/COSMIC retained text: viewport-local realization may move
  without redefining semantic content.
- **Official release receipt.** On the reference Windows/x86_64 machine, the
  version-1 official 64-warmup/1,024-sample `text-horizontal-1m` matrix reports
  identical 1,432 px near and far window widths, 916,480 px maximum logical
  area, and `bounded_window=true` at x=5,157,889. Warm p50/p95/p99 are
  2/2/3 us and max is 10 us; all 1,024 samples hit both render and width caches,
  shape zero lines, visit zero source bytes, and inspect one caret glyph. The
  historical far width was 5,159,085 px and area 3,301,814,400 px, so prepared
  surface width and area are now independent of absolute offset for this trace.
- **Still-red cold/storage fact.** Cold time is 729,572 us, including 220,271 us
  render shaping and 278,583 us width measurement over the complete 1 MiB line.
  The shaped buffer is still whole-line storage and glyph placement still carries
  a far absolute origin. This checkpoint therefore closes only the prepared
  surface/cull bound; entering-strip glyph shaping/recycling, edit-time width
  maintenance, far-offset precision witnesses, GPU resident-byte high-water, and
  the 64 MiB matrix remain open under S-004/S-009.
- **Witnesses.** `horizontal_render_window_is_bounded_and_covers_near_far_and_boundary_offsets`,
  `horizontal_render_window_reuses_one_bucket_until_its_trailing_guard_is_spent`,
  `text_surface_keeps_prepared_runway_separate_from_buffer_origin`, and
  `text_area_horizontal_boundary_replenishes_one_bounded_window_without_semantic_commit`
  cover the local geometry, bucket reuse, pipeline handoff, requested/admitted
  boundary, stable size, semantic zero, drawable replacement, and residency
  revision. The versioned release driver supplies the official timing/work
  receipt; these results do not claim the full S-004 or S-009 exit theorem.

### R-011 — text runway versus semantic identity

- **Newly exposed source-of-truth violation.** The first end-to-end boundary
  witness stayed red after the runway became bounded: its next private layout
  admitted the requested offset and produced a new drawable residency, but the
  presentation also carried a new semantic commit. `ScrollDeclaration::semantic`
  already normalized the baseline and resident bounds. The leak was the prepared
  text viewport plus caret/scrollbar rule geometry still participating in the
  owner node's semantic content and draw order.
- **Correction.** Layout explicitly names scroll owners whose painted content is
  independent residency. Store passes that classification beside virtual row
  membership into the single semantic projector. The projector removes content
  inside those bounded scroll scopes, removes caret content, canonicalizes
  scrollbar-thumb geometry back to its stable track origin, remaps retained
  content indices, and keeps the owner node plus its scroll/caret/axis property
  topology and validation envelope. Stable body/theme content remains in the
  semantic projection, so an actual owner-content change still advances it.
- **Proof and resistance.** `semantic_projection_excludes_bounded_and_caret_content_and_normalizes_scrollbars`
  proves changed runway, scroll baseline, caret, and scrollbar geometry reuse one
  semantic `Arc`, while a body-color change does not. The end-to-end text boundary
  witness proves admitted progress, a new drawable, a new local residency
  revision, constant runway size, and pointer-identical semantic commit.
  `bounded_scroll_content_and_transient_rules_cannot_enter_semantic_scene_identity`
  ratchets the layout/store/projector handoff. The earlier virtual-list zero-
  semantic crossing and all 33 caret-focused tests remain green. The checkpoint
  suite reports 1,210 passed and four intentional ignores; renderer-debug's pure
  tier reports three passed and eighteen intentional GPU ignores; the workspace
  all-target/all-feature check is green.

### S-004/S-009 checkpoint D — reuse one exact complete observation

- **Measured duplicate owner.** The version-1 cold receipt exposed three complete
  shapes of the same one-MiB logical line: visible interaction layout, independent
  whole-document width measurement, and render-surface construction. The latter
  two contributed 278,583 us and 220,271 us respectively to the 729,572 us cold
  checkpoint-C receipt even though the first layout already held the exact shaped
  line. This was duplicated work, not a residency requirement.
- **Correction and correctness fence.** A no-wrap paint layout may now publish its
  observed document width only when the ordered display segments cover every
  logical source line exactly. The render-surface path may reuse the committed
  one-line display buffer only under the same line-layout identity, style, width,
  wrap, and direction key. Width continues to use cosmic-text's `line_w`, so
  trailing-space and scrollbar-extent truth are unchanged. Documents with any
  unrealized logical line retain the independent document-width measurement;
  viewport observation can never guess the longest unseen line. The same trace
  also exposed and corrected last-line source metadata that formerly clamped the
  terminal range to a zero-length line start.
- **Versioned metric semantics.** `scroll-bench-version=2` adds observed-width
  updates and render-line reuses. Render source-line/byte counters now mean actual
  source visits, so a cache hit or shape reuse records literal zero instead of
  restating resident metadata. `one_line_text_area_reuses_its_observed_shape_for_width_and_render`
  compares the reused width with an independent document measurement and proves
  one line shape, one observed-width update, one render-line reuse, complete
  terminal metadata, and zero duplicate measure/shape/source work. The existing
  unrealized-line and render-cache witnesses keep their independent/freshness
  paths.
- **Official release receipt.** Exact commit `7d42f30922b3` on the reference
  Windows/x86_64 machine produced three official 64-warmup/1,024-sample trials.
  Cold times were 233,336, 233,792, and 233,401 us; the median is 233,401 us,
  3.13x faster than checkpoint C's 729,572 us. The median warm p50/p95/p99/max
  are 3/3/4/48 us. Every trial preserves logical width 6,878,106 px, identical
  1,432 px near/far windows, and 916,480 px bounded area. Each cold receipt records
  one render-line reuse and one observed-width update with zero render source
  bytes, render shape time, width source bytes, or width measure time. All 1,024
  warm samples hit both caches and perform zero line shaping or source visits.
- **Industry and resistance ruling.** Against the upstream line recorded above,
  this applies Chromium/Firefox-style retained artifact reuse, GTK/Qt text-layout
  reuse, and Iced/COSMIC retained-content identity without importing another
  position owner; it also preserves Flutter's finite viewport-demand boundary.
  The repository's stronger all-line proof prevents reuse from weakening exact
  extent truth. This does not close S-004/S-009: the surviving one-line buffer
  still contains the complete one-MiB line, entering-strip glyph shaping/recycling
  and GPU resident-byte high-water are unproved, and the 64-MiB/far-precision
  matrix remains red. The checkpoint suite reports 1,211 passed and four
  intentional ignores; renderer-debug reports three passed and eighteen ignored
  GPU tests; the workspace all-target/all-feature check and diff check are green.

### S-004/S-009 checkpoint E — bounded resident glyph fragments

- **Measured storage owner.** Checkpoint D bounded the prepared rectangle but its
  shared `glyphon::Buffer` still retained all 1,048,576 source bytes and glyphs.
  Thus absolute offset no longer enlarged the surface while source length still
  enlarged active CPU glyph storage, the line cache, renderer input, and the
  transitive cosmic-text shape-run cache. The bounded-work law requires both
  geometry and prepared glyph storage to be independent of total line length.
- **Conservative fragmentation fence.** The engine now admits horizontal word
  fragmentation only for one-run LTR ASCII no-wrap lines wider than the prepared
  runway. It derives exact source/prefix checkpoints from the first complete
  cosmic-text layout, and each interval must contain at most 4,096 source bytes.
  This follows cosmic-text 0.18.2's practiced independent ASCII-word shaping at
  Unicode line-break boundaries rather than guessing a shaping context radius.
  Bidi, complex text, multi-run output, more-than-4-GiB lines, and unbreakable
  intervals over the bound retain the exact whole-line fallback and remain named
  resistance, not silently approximate output.
- **Bounded identities and memory.** Checkpoints use one `u32` byte position plus
  one `f32` observed prefix coordinate and live in a 64-entry immutable `Rc` LRU;
  a cache hit bumps shared identity instead of cloning the source-proportional
  index. Only word fragments intersecting the 1,432 px runway plus one safe word
  guard are shaped and retained. The shared line-shaping cache has a 16-MiB
  weighted ceiling, and the unbounded transitive shape-run cache is discarded
  after full-index construction then age-trimmed as bounded fragments enter.
- **One projection across fragments.** Interaction, overlay, render, selection,
  caret reveal, source mapping, and hit testing consume the same fragment source
  ranges and target-local text origins. Hit testing chooses `(line distance,
  fragment-x distance)` instead of the first fragment on a matching y. Multiple
  word surfaces prove identical horizontal prepared rectangles and vertically
  contiguous coverage before their union can become resident bounds; the final
  logical line extends that proof through the viewport's blank remainder. A
  boundary crossing therefore still changes drawable/residency identity with
  zero semantic commit.
- **Versioned work and memory receipt.** `scroll-bench-version=3` adds index
  build/source/glyph/checkpoint counts, the live index-byte gauge, entering-window
  shapes/source bytes, active source/glyph/estimated-byte high-water, and weighted
  line-cache high-water. Exact commit `43626ac67823` produced official cold times
  of 270,578, 269,563, and 275,321 us; the median is 270,578 us. Median warm
  p50/p95/p99/max are 8/9/14/89 us. Every trial preserves logical width
  6,878,106 px, identical 1,432 px near/far windows, and 916,480 px bounded area.
  Cold builds 53,775 eight-byte checkpoints (430,200 bytes), then retains 246
  source bytes/glyphs (45,510 estimated bytes) across 13 initial word fragments.
  Across all 1,024 measured transitions, only four entering fragments totaling
  78 source bytes shape; active high-water is 273 source bytes/glyphs and 50,505
  estimated bytes, while the weighted line cache peaks at 110,445 bytes. All
  13,820 render-fragment accesses hit, with zero render source visits, render
  shape time, width source visits, or width measurement.
- **Timing tradeoff and industry ruling.** The median remains 2.70x faster than
  checkpoint C's 729,572 us cold owner and warm residency work remains orders of
  magnitude below a frame budget. It is nevertheless 15.9% slower cold than
  checkpoint D's unbounded 233,401 us median, and warm p95 rises from 3 to 9 us.
  The campaign accepts that measured regression as the explicit cost of deleting
  whole-line active glyph residency; the prohibited-shortcut law does not permit
  keeping unbounded storage to win one timing row. Against the recorded upstream
  line, this implements Chromium/Firefox-style visible/near-visible retained
  fragments, Qt's block-oriented large-text challenge, and Iced/COSMIC retained
  content with a fixed visible layer, while preserving this repository's stronger
  exact source, hit, caret, integral admission, and complete-pixel contracts.
- **Witnesses and remaining red paths.** `far_ascii_window_matches_independent_full_line_glyphs`
  proves source clusters, glyph identities/advances, runway-local coordinates,
  and far x-aware hits; `far_ascii_caret_reveal_uses_its_resident_fragment` proves
  caret mapping; `unbreakable_ascii_line_keeps_the_exact_full_line_fallback`
  proves the resistance fence; the boundary/semantic and weighted-cache witnesses
  cover residency and eviction. The suite reports 1,218 passed and four
  intentional ignores; renderer-debug reports three passed and eighteen ignored
  GPU tests; workspace all-target/all-feature and diff checks are green. S-004
  remains open: cold/index construction still shapes and walks the complete line,
  complex/bidi/unbreakable resident glyph storage is not yet bounded, exact
  `2^24`/maximum pixel output, GPU resident glyph bytes, the 64-MiB matrix, and
  edit-time incremental index maintenance remain unproved.

### S-004/S-009 checkpoint F — exact large horizontal truth and packed indices

- **Newly censused precision owner.** The text control projected the session's
  integral `ScrollOffset` through `as f32` before visible-line lookup, resident
  window selection, caret reveal, extent clamping, and viewport construction.
  Consequently `16_777_217` could become `16_777_216` even though generic and
  table chrome retained the integer. Clamp-change detection then compared the
  same rounded accessors and could fail to install a corrected state. This was
  a text-only second representation truth, not a renderer limitation.
- **Exact logical path.** `ViewState` now retains lossless `f64` logical values
  for the framework's complete `i32` session domain. The production text-area
  projection enters through an integral constructor; viewport offsets, content
  extents, clamping, horizontal index lookup, metrics, and caret reveal consume
  the exact value. Only the difference between an exact resident-window origin
  and the exact admitted value narrows to the bounded `f32` surface coordinate
  used by glyphon and the renderer. Compatibility accessors remain floating but
  no longer decide the framework-owned offset or extent.
- **Shaper-preserving exact width.** Naively summing glyph widths was rejected:
  the independent glyph oracle caught lost kerning, shifted far fragments, and
  incorrect hits. Lines whose complete shaped width reaches `2^24` are instead
  re-shaped once in at most 262,144-byte, word-safe LTR ASCII bands. Each band's
  local shaped advances remain below the float precision boundary and are
  accumulated into exact `f64` checkpoints. The ordinary one-MiB path performs
  zero band shapes; complex, bidi, and unbreakable resistance paths remain
  explicitly outside this admission fence.
- **Memory optimization and metrics.** Exact checkpoint coordinates initially
  made each `(u32, f64)` record occupy 16 padded bytes. Commit `1797f5657c3a`
  split byte positions and x coordinates into parallel arrays, reducing the
  four-MiB index from 3,441,504 to 2,581,128 bytes and the one-MiB index from
  860,400 to 645,300 bytes: exactly 25% with unchanged lookup behavior. Version
  5 receipts add exact-band shape/source work and explicit required/checked
  precision fields so the extra cold work cannot hide inside a generic shape
  count.
- **Official precision receipt.** Exact commit `1797f5657c3a` produced three
  official `text-horizontal-4m-exact` trials. Cold times were 1,958,357,
  1,894,816, and 1,907,691 us; the median is 1,907,691 us. The median trial's
  warm p50/p95/p99/max are 8/8/15/44 us. Every trial reports logical width
  27,538,192 px, `precision_offsets_required=true`, and
  `precision_offsets_checked=true` at 16,777,215, 16,777,216, 16,777,217, and
  24,000,001. Near and far preparation remains exactly 1,432x640
  (916,480 px). Cold construction performs one complete index shape plus 17
  exact bands over 4,194,304 source bytes; the warm 1,024-transition trace
  performs only two entering-fragment shapes over 39 bytes, retains at most 273
  source bytes/glyphs and 50,505 estimated active bytes, and records zero width
  source visits or measurement.
- **One-MiB non-regression receipt.** The same commit's three official
  `text-horizontal-1m` trials report cold times 267,878, 274,597, and 273,367 us;
  the median is 273,367 us, 1.0% above checkpoint E's 270,578 us and inside the
  five-percent gate. The median trial's warm p50/p95/p99/max are 7/7/9/55 us,
  with zero exact-band shapes and the same 1,432x640 bound. Thus precision work
  is paid only by content that needs it; ordinary warm movement does not regress.
- **Witnesses and industry ruling.** Integral session-to-text round trips,
  exact checkpoint selection, renderer-local unit deltas, exact-band merging,
  caret accumulation, and the versioned release trace cover the CPU/layout
  contract. The full library suite reports 1,226 passed and four intentional
  ignores; renderer-debug CPU tests and workspace all-target/all-feature checks
  are green. This meets GTK/Qt's shared adjustment/central-viewport expectation
  without their floating value becoming authoritative, preserves Iced/COSMIC's
  one state beneath a fixed clip, and matches Chromium and Firefox's bounded
  local raster/displayport coordinates while keeping an exact main-thread
  logical position. Flutter's explicit position/dimension correction remains
  the closest lifecycle comparison; the repository adds exact integral
  admission and complete-pixel gating. WebRender-style spatial separation and
  block-editor indexing beyond the named frameworks support the banded design,
  but do not excuse its remaining resistance paths.
- **Remaining red paths.** S-004/S-009 remain open. Vertical text height/index
  coordinates still narrow before the large-offset boundary; complex/bidi and
  unbreakable long lines still retain whole-line buffers; cold construction
  still walks and shapes the complete line; incremental edit maintenance,
  exact GPU pixel/readback at these offsets, GPU resident glyph-byte high-water,
  and the 64-MiB mixed-long-line matrix remain unproved.

### S-004/S-009 checkpoint G — streamed sparse indices and 64-MiB ASCII scale

- **Displaced cold whole-line owner.** Checkpoint F still materialized and shaped
  the complete line once before splitting it into exact bands. For long admitted
  LTR ASCII lines, index construction now reads at most 262,144 source bytes at
  a time from the span tree, chooses a word-safe boundary, shapes that bounded
  band, and merges its local coordinates directly into the exact index. No
  whole-line `String` or glyph buffer exists on this path. If ASCII, direction,
  safe-boundary, or 4,096-byte context validation fails, the engine returns to
  the named exact fallback rather than approximating complex output.
- **Sparse safe checkpoints.** The prior index recorded every word start, even
  though residency needs only enough safe boundaries to bound one independently
  shaped fragment. The builder now targets a safe checkpoint every 256 source
  bytes and retains 4,096 bytes as a hard maximum. It remembers the preceding
  safe boundary so a long legal word cannot make the selected interval exceed
  the cap. The far-glyph oracle was expanded across a real band boundary and
  still matches the independent complete cosmic-text layout in cluster, glyph,
  advance, source position, and x-aware hit output.
- **Cache budget and attribution.** The horizontal-index LRU keeps its 64-entry
  identity cap and now also trims to 64 MiB of resident checkpoint arrays, except
  that one newly required oversize index may remain as the sole entry. Hits,
  misses, evictions, exact-band work, and resident bytes are code-owned receipt
  fields. A warm 1,024-transition trace records 2,048 index hits, zero misses,
  and zero evictions; cold construction records one miss followed by two hits.
- **Four-MiB optimization receipt.** Version 6 at exact commit `0fe86c5c2ed9`
  produced cold times 1,092,850, 1,018,481, and 1,064,432 us for
  `text-horizontal-4m-exact`; the median is 1,064,432 us, 44.2% below checkpoint
  F's 1,907,691 us. The median trial's warm p50/p95/p99/max are 3/4/5/72 us.
  Checkpoint count falls from 215,094 to 15,379 and index residency from
  2,581,128 to 184,548 bytes, a 92.9% reduction. Exact width remains 27,538,192
  px and all four precision positions remain checked. The explicit trade is a
  larger but still bounded warm fragment high-water: 1,092 source bytes/glyphs,
  202,020 estimated active bytes, and 663,780 weighted cache bytes.
- **One-MiB optimization receipt.** The same commit reports cold times 268,617,
  254,636, and 253,160 us; the median is 254,636 us, 6.9% below checkpoint F's
  273,367 us. Median-trial warm p50/p95/p99/max are 3/3/4/12 us. The index holds
  3,846 checkpoints in 46,152 bytes rather than 645,300 bytes, while warm
  movement performs zero fragment shapes and retains the same fixed surface.
- **New 64-MiB ASCII receipt.** Three official
  `text-horizontal-64m-ascii` trials report cold times 15,586,456, 15,844,615,
  and 15,523,888 us; the median is 15,586,456 us. Median-trial warm
  p50/p95/p99/max are 3/3/5/54 us across 1,024 transitions. All trials preserve
  exact logical width 440,610,925 px, check the four precision offsets, and keep
  near/far surfaces at 1,432x640. Cold construction visits and shapes the
  67,108,864 source bytes once in 257 bounded bands, producing 246,041
  checkpoints in 2,952,492 bytes. Warm work performs one entering-fragment shape
  over 273 bytes, with 1,092 source bytes/glyphs, 202,020 estimated active bytes,
  and 757,575 weighted cache bytes at high water. The 15.6-second cold owner is
  still material and remains in the optimization queue; document length no
  longer affects warm movement or resident surface/glyph bounds.
- **Industry and closure ruling.** This now meets Qt's large-plain-text
  block/index expectation and the Chromium/Firefox visible-region economics for
  the admitted ASCII species, while preserving GTK's one adjustment-derived
  extent and Iced/COSMIC's one position beneath a fixed clip. Flutter's finite
  cache-extent model remains the comparison for bounded resident work. The local
  implementation is stronger where it keeps an exact integral position and
  rejects incomplete pixels, but it is not stronger for complex shaping yet.
  The suite reports 1,227 passed and four intentional ignores; renderer-debug
  CPU tests and workspace all-target/all-feature checks are green.
- **Remaining red paths.** This is an ASCII scale receipt, not the required
  64-MiB mixed-long-line closure. Complex scripts, bidi, and unbreakable long
  lines still take the whole-line fallback; edit-time index reuse is absent;
  vertical coordinates beyond `2^24`, exact GPU readback, GPU glyph-byte
  high-water, native cadence, and cold-index profiling/reduction remain open.

### R-005 — stable variable-height anchor and sole line geometry

- **Bounded question and trace.** Property-only text movement, width/style
  reflow, persistent line marks, height-index lookup/refinement, wrapped visual
  rows, interaction surfaces, render residency, caret pins, layout feedback,
  session desired/admitted state, and the next scene property were traced in
  both directions. The existing `scroll_anchor_for_text_area` and
  `text_area_scroll_y_for_anchor` functions had no production caller. More
  seriously, wrapped paint built a second aggregate multi-line glyph buffer:
  its actual row heights could place source line 0 across the viewport while
  the height index and interaction surfaces claimed source line 2. The visible
  chop was therefore not merely absent feedback; paint and hit geometry had
  distinct height truths.
- **One bounded resolution path.** The layout text service retains a 128-entry
  LRU keyed by retained composition `NodeId`. Each entry contains only stable
  source-line marks and compact `(y, height)` samples for the bounded resident
  runway; it retains no glyph buffer and no authoritative offset. A property-
  only offset inside that runway resolves the current top source line and its
  within-line displacement even when the semantic layout was reused. Reflow
  resolves the same mark against the refined height index, rebuilds at the
  proposed integral value until stable, and attaches one geometry correction.
  `Frame::resolved_scroll_correction` feeds that proposal into runtime's
  existing `ScrollUpdate::Geometry` request/admission path. Explicit caret
  reveal and preedit bypass anchoring, so a semantic reveal cannot be canceled
  by stale viewport stability.
- **Sole line geometry and structural work removal.** Wrapped and unwrapped
  paint now use the same bounded per-logical-line display cache as hit testing,
  selection, caret, and interaction. Render residency refines those line
  heights first; interaction projection consumes the resulting index. The
  independent aggregate render-buffer cache, whole-range source flatten,
  shape, key, capacity, and reset path were deleted. A distant pending caret is
  one explicit non-contiguous pin and cannot extend resident coverage. The
  guarded render window is capped at 128 logical lines and its refinement loop
  at four passes. Its transient preparation registry uses one pre-sized
  contiguous vector rather than one B-tree allocation per resident line.
- **Metrics and official receipt.** `scroll-bench-version=4` adds
  `text-vertical-8m` plus logical height/y offset, near/far window height,
  height-index queries/updates/refined pixels, and anchor candidate/correction
  count and distance. Exact commit `d5edc49b6765` produced three release
  64-warmup/1,024-sample trials. Cold times were 6,371, 6,232, and 6,193 us
  (median 6,232 us). Median warm p50/p95/p99/max were
  1,107/1,164/1,207/1,966 us. All trials retained the same 1,432 by 1,154 px
  far window (1,652,528 px area) at y=772,780 in an 8,388,608-byte,
  63,431-line document. Across every 1,024-transition trial, only 12 entering
  lines totaling 1,575 source bytes shaped, 57,850 render accesses hit, and
  three height updates refined 51 px. Line-cache high-water was 3,494,686
  bytes; aggregate render shape time remained literal zero. Cold preparation
  visited 8,400 resident source bytes, performed 16 refinements totaling 272
  px, and retained 1,551,056 cache bytes.
- **Measured optimization ruling.** Before replacing the per-line B-tree with
  the contiguous registry, exact commit `4e2f1ba312bf` produced median warm
  p50/p95/p99/max of 1,196/1,258/1,295/2,006 us with identical semantic work,
  window, and memory counts. The accepted path improves those distributions by
  7.4%, 7.5%, 6.8%, and 2.0%, respectively; median cold time is statistically
  unchanged (6,223 versus 6,232 us). A hash-table challenge measured a 1,277 us
  p50 and was rejected. These comparisons isolate container overhead rather
  than presenting unit-test elapsed time as scroll performance.
- **Industry comparison.** The result meets GTK's one-adjustment-per-axis rule
  by returning geometry through the sole owner, Qt's block-oriented large-text
  line by preparing indexed logical lines rather than a document surface, and
  Iced/COSMIC's fixed viewport/state contract by moving one property under one
  clip. Chromium's pending/active separation and Firefox's bounded displayport
  challenge the retained runway and uninterrupted active state; neither
  justifies a second layout offset or incomplete pixels here. Flutter's named
  layout-correction contract is the closest direct comparison: this repository
  keeps the same explicit correction property while adding stable document
  marks, integral request/admission, and complete-pixel gating. The named
  frameworks remain the floor, not copied widget/process topology.
- **Witnesses and closure.** `resident_anchor_band_preserves_source_line_and_within_line_offset_across_reflow`
  proves the text-engine line mark, displacement, real numeric drift, and
  corrected surface. `wrapped_text_resize_preserves_the_presented_source_anchor_through_geometry_feedback`
  proves property-only movement, reflow, session desired truth, and non-zero
  correction metrics across layout/runtime. `variable_height_text_has_one_bounded_anchor_feedback_path`
  ratchets sole feedback, bounded storage, render-before-interaction ordering,
  and absence of aggregate shaping. The library suite reports 1,219 passed and
  four intentional ignores; workspace all-target/all-feature checks are green.
  S-004's complex/64-MiB/edit-index work, S-008 activation/clock races, S-009's
  remaining matrix, and S-010 native deadline capture remain open; no second
  variable-height position or render-height owner remains.

When a trace discovers another authority, cache, consumer, widget species,
backend discrepancy, complexity failure, or material performance owner, append a
new `S-*` cell before continuing. New cells are not deferred merely because the
seed symptoms are already fixed.

## Performance measurement contract

### Metric ownership and schema

Extend the existing `diagnostics::{Scroll, Pipeline, Render}` receipt rather
than creating a benchmark-only truth. Reuse existing render timing and resource
counters; add the missing scroll-specific attribution below. Metric names are
stable receipt vocabulary once implemented.

| Domain | Required counters / gauges | Required distributions |
|---|---|---|
| Intent and admission | `scroll_input_events`, `scroll_desired_changes`, `scroll_admitted_changes`, `scroll_unchanged`, `scroll_property_ticks`, `scroll_needs_residency`, `scroll_active_boundary_projections`, desired-minus-admitted distance per axis | request-to-admit time, desired/admitted lag magnitude |
| Pipeline work caused by scroll | scroll-attributed view rebuilds, routing layouts, presentation layouts, composition reconciliations, scene assemblies, semantic commits, residency requests/revisions/activations, property serials, attempts and successful presents | scroll event handling, reconciliation, layout, assembly, and input-to-submit CPU time |
| Text layout and preparation | source lines/bytes visited, width-index queries/updates, height-index queries/updates, anchor corrections, shaped lines/glyphs, prepared lines/glyphs, render-window origin/size/area, text cache hits/misses, resident text bytes | text measure, shape, resident-window build, glyph prepare, and replenishment time |
| Table/layout residency | requested/realized rows, entering/leaving rows, pins, covered pixels, runway extent, refinement iterations, rejected incomplete candidates | residency proof, materialization, and incremental layout time |
| Retained renderer | existing plan reuse/rebuild, quad/text prepare, upload bytes, buffer/resource create/replace/remove, retained resource count/bytes, draws/passes, cache hits/misses; add scroll-binding bytes and resident glyph vertex bytes | existing property/semantic batch prepare, encode/submit/present, draw, key-to-present; add residency preparation |
| Cadence and progress | input deltas received/coalesced, redraws requested, attempts, presents, property/residency deadline misses, consecutive missed frames, longest no-progress interval | frame interval, event-to-attempt, event-to-present, residency wait |
| Allocation and memory | hot-path allocation count/bytes through a test/diagnostic allocator, active/preparing/recycle CPU and GPU high-water | allocation size only in diagnostic runs |

Every sample is tagged or separated by transition class:

```text
PropertyHit | ResidencyCrossing | SemanticChange | ResizeOrScale | ColdStart
```

Property and residency metrics never infer causality from a total frame count.
The transition result, active commit, residency revisions, property serial, and
presentation receipt supply the attribution. Long-running histograms/rings are
fixed-capacity; recording a sample performs no heap allocation. Diagnostics-off
overhead must be statistically indistinguishable from the pre-metric baseline
and below one percent in the controlled benchmark.

### Deterministic workload matrix

The code-owned driver lives in `renderer_debug` beside the retained renderer
oracle and emits a versioned `scroll-bench` receipt. Pure state/work contracts
remain ordinary tests; GPU timing/readback witnesses remain ignored release
tests. Native release-gallery runs add real pacing and process-survival evidence
without replacing the code-owned workload.

| Workload | Required trace | Sizes / positions | What it must expose |
|---|---|---|---|
| Generic property runway | 4 px, 120 px, reversal, diagonal, independent axis saturation | ordinary and nested viewports; offsets around `2^24` and near maximum | pure property cost, coordinate precision, nested clip/transform reuse |
| Virtual table vertical | slow continuous, fast wheel, large thumb jump, provider shrink/reorder | 136/500/800 px gallery heights; one million logical rows | admission/replenishment, row/rule/text completeness, bounded nodes/resources |
| Wide table horizontal | slow/fast drag, wheel/trackpad, maximum jump, column resize during scroll | narrow and wide viewports; production column grammar | rule/cell motion, fixed clip, two-axis chrome, track/layout reuse |
| Text vertical | slow 4 px stream, page movement, jump, reversal, active edit/selection | 8 MiB and 64 MiB documents plus one-million-line synthetic source | stable anchor, visible-line lookup, bounded shaping/preparation, cadence |
| Text horizontal | slow/fast movement and jumps on unwrapped content | 1 MiB line and 64 MiB mixed long-line document; `2^24 - 1`, `2^24`, `2^24 + 1`, far maximum | target-size independence, incremental width truth, glyph-window recycling |
| Text diagonal/two-axis | continuous diagonal input and one saturated axis | wrapped and unwrapped, editable and read-only | independent axis admission, shared viewport policy, no all-or-nothing rollback |
| Residency race | `active A -> preparing B -> desired C`, acceleration and reversal | C inside B, beyond B, and one-axis-only compatible | monotonic projection, cancellation/reuse, progress without input settling |
| Multi-owner/native | two panels, two windows, parent plus popup under uninterrupted input | 1–2 ms input trace, occlusion and reveal | local clocks, fair attempts, no cross-owner starvation or process failure |
| Lifecycle stress | resize, fractional scale, backend/device recreation, content edit during movement | scales 1.0, 1.25, 1.5, 2.0; DX12 and Vulkan where available | invalidation truth, cleanup, cache compatibility, exact pixels after recovery |

Each warm property workload uses at least 64 unmeasured warmup transitions and
1,024 measured transitions. Residency workloads use at least 16 warmups and 128
measured crossings; cold-start/device cases use at least 16 independent samples.
Run three trials and compare the median trial p50/p95/p99/maximum. Receipts name
workload version, commit, release profile, timer, refresh rate, adapter, backend,
device class, driver, OS, architecture, scale, viewport, document/table shape,
runway policy, warmup, samples, and whether readback or native presentation was
included. Deliberate input pacing is never reported as CPU or GPU work.

### Structural performance floors

Portable structural gates outrank noisy elapsed-time comparisons:

1. A warm `PropertyHit` changes the admitted offset/property serial and performs
   **zero** view rebuilds, routing or presentation layouts, composition
   reconciliations, semantic commits, residency revisions, scene assemblies,
   node realization rebuilds, quad/text prepare calls, text shape/measure work,
   retained-resource creates/replacements, geometry-buffer creations, and hot-
   path heap allocations.
2. A residency crossing mints **zero semantic commits**. Its materialization,
   proof, shaping, glyph preparation, uploads, and allocation scale with the
   entering strip plus retained pins, not the complete runway or total source.
3. Text render-window width/height/area and prepared glyph storage have a fixed
   implementation bound derived from viewport, scale, runway, and content
   density. Holding those inputs constant while increasing absolute offset or
   document length cannot raise the bound.
4. Reversing within current/recycle residency reuses the prior realization and
   resources. Direction changes do not throw away compatible work.
5. Table rules, cells, row backgrounds, text, hit testing, and both scrollbars
   consume the same admitted property. No species pays an extra semantic/layout
   path to obtain behavior another species gets from a property tick.
6. Continuous input may be coalesced only by accumulating into the one desired
   position. It may not drop net movement, wait for quiescence, or reduce work by
   presenting stale state.

### Timing and cadence gates

Checkpoint 0 pins actual reference-machine numbers before optimization. The
following are minimum final gates on that same environment; meeting them does
not end the optimization loop:

- warm `PropertyHit` input-to-submit CPU p95 is at most 25% of one refresh
  interval and p99 at most 50%; its draw p95 is also at most 25%;
- warm input-to-present p95 is at most one refresh interval plus the measured
  acquire/present wait that the application cannot schedule away;
- a residency crossing's CPU preparation p95 fits within one refresh interval,
  and sustained motion never records two consecutive no-progress frame
  intervals while compatible active residency exists;
- the seeded large-text vertical and horizontal traces improve their baseline
  p95 by at least 2x unless the baseline is already below 10% of one refresh
  interval, in which case structural zero/bounded-work and the fixed-point rule
  govern;
- generic and table p95/p99, cadence, and memory do not regress by more than 5%
  across the median of three controlled trials; any exception requires a named
  correctness tradeoff and explicit campaign ruling; and
- diagnostics-on CPU time and allocation overhead remain below 1% versus the
  same release workload with diagnostics recording disabled.

Evaluate both 60 Hz and 120 Hz budgets when the display/backend permits it. A
faster optional machine may corroborate a result but cannot redefine the pinned
campaign threshold or become necessary for closure.

### Optimization order and fixed point

Optimize in this order so local speedups cannot preserve the wrong graph:

1. delete false semantic work, duplicate ownership, feedback loops, and stale
   activation;
2. make work asymptotically viewport/runway bounded and entering-strip
   incremental;
3. retain and recycle layout indices, shaped runs, glyph vertices, GPU
   resources, render plans, and bounded allocations by their real revisions;
4. coalesce compatible input/preparation without losing desired-state truth;
5. remove hot-path allocation, cloning, concatenation, hashing, sorting,
   virtual dispatch, and redundant coordinate conversion identified by metrics;
6. improve data layout, batching, upload ranges, and branch locality only after
   the preceding structural receipts hold.

After every optimization cell, re-run pixels, structural work, timing, cadence,
and memory receipts. Profile the highest remaining owner with code counters
first and PIX/ETW, a CPU profiler, or GPU captures only when the code-owned
receipt cannot answer the named question. Every owner consuming at least 10% of
scroll input-to-submit CPU time receives either a reduction or a recorded
resistance ruling.

The performance loop reaches fixed point only after two consecutive full
workload sweeps find no admissible change that does any of the following without
violating correctness or another measured gate:

- removes unbounded, duplicated, invalidated, or allocation-producing work;
- turns required warm work into a structural zero;
- improves p95 by at least 5% or p99 by at least 10% in at least two of three
  controlled trials; or
- reduces steady/high-water CPU or GPU memory by at least 10%.

### Prohibited performance shortcuts

- widening overscan/runways to hide preparation latency without proving memory
  and worst-case preparation bounds;
- retaining a whole document/table, an absolute-offset-sized surface, or stale
  duplicate scene/layout truth to make one trace fast;
- dropping input, delaying work until input stops, checkerboarding, exposing
  incomplete pixels, loosening pixel tolerances, lowering coordinate precision,
  or changing scrollbar behavior to improve timing;
- benchmark-only production branches, permanent verbose logging, unbounded
  sample storage, or a diagnostic dirty flag that production consumes;
- caches without explicit owner, key, revision, bound, eviction/recycle rule,
  device-loss behavior, and hit/miss/resource receipts; and
- declaring victory from average FPS, one settled frame, one backend, one axis,
  one widget species, or a faster external machine.

## Required witnesses

The correction cannot close without deterministic witnesses for:

1. exact integral round trips at `16_777_215`, `16_777_216`, `16_777_217`,
   representative odd values near 24,000,000, and the gallery maximum;
2. missing, duplicate, out-of-order, and gap-producing virtual rows yielding
   `NotReady` and no scene residency;
3. a complete requested range covering every required viewport pixel;
4. a distant pin being excluded from contiguous residency;
5. `active A -> preparing B -> desired C` activating `B` with `C` when accepted,
   otherwise making compatible forward progress while `C` remains pending;
6. no successful receipt whose property serial or admitted value regresses;
7. relative and absolute requests for the same resulting offset producing the
   same scheduling transition;
8. vertical and horizontal scrollbar hit/capture parity;
9. one-million-row resident node/resource bounds and a crash-free large-thumb
   release-gallery run;
10. zero semantic commits for ordinary resident boundary crossings;
11. two independently scrolling panels and two windows with independent pulses;
12. parent/native-popup preparation and receipt independence;
13. uninterrupted native input every 1–2 ms producing regular attempts and
    successful forward progress without waiting for input to stop;
14. fixed viewport clips enclosing every repeated scroll scope;
15. retained row backgrounds, rules, and text all remaining pixel-complete at
    both an ordinary property tick and the active residency boundary;
16. generic, table, and text property hits sharing the structural-zero fast path
    while preserving their distinct content semantics;
17. horizontal text movement at near and far offsets retaining the same bounded
    render-window/resource high-water and performing no whole-document scan;
18. vertical text height refinement preserving a stable line/block anchor and
    within-line displacement without a visible chop or second offset owner;
19. text and table vertical/horizontal scrollbar presence, gutter, activity,
    fade, hit, capture, and mutation following the campaign's explicit common
    policy rather than incidental node topology;
20. table rules maintaining exact pixel displacement relative to their cells
    through horizontal and vertical property ticks, residency crossings,
    fractional scales, and resize;
21. every deterministic workload emitting the versioned semantic-work, timing,
    cadence, allocation, memory, environment, and pixel receipt;
22. warm property ticks, residency crossings, large text, and one-million-row
    workloads satisfying all structural and timing gates; and
23. two complete post-seed re-census/profile sweeps producing no new admitted
    correctness or material performance cell; and
24. baseline, post-ownership, and final comparison receipts against every named
    industry reference, with each relevant property either met/exceeded or
    represented by an open campaign cell.

Tests that inspect only a union rectangle, one non-clear pixel, settled output,
or an offscreen renderer without the native pending queue are insufficient.

## Exit theorem

The goal is complete only when:

1. desired and admitted scroll coordinates remain integral until a bounded
   renderer-local delta;
2. every drawable virtual scroll scope carries exact, complete scene residency;
3. incomplete or non-converged residency cannot enter scene drawing or activation;
4. viewport clips remain fixed outside moving scroll scopes and every painted
   descendant, including text, is prepared across accepted residency;
5. resident membership changes mint residency revisions and zero semantic commits;
6. pending/active projection and activation cannot regress the newest accepted scroll value or
   starve behind a farther desired value;
7. all scroll inputs and both axes share one request/admission contract;
8. variable-height correction has one explicit anchor-resolution path;
9. per-window and popup-local clocks remain independent;
10. presented geometry, scrollbar geometry, hit testing, and renderer output
    consume the same receipted property state;
11. the required deterministic, workspace, renderer, native-stream, and real
    release-gallery witnesses pass;
12. every displaced float, rectangle-only, semantic-replenishment, global-pulse,
    stale-activation, and duplicated transition path is deleted;
13. property-hit and residency-crossing work obey the structural performance
    floors, resource and memory use remain viewport/runway bounded, and the
    pinned release timing/cadence gates pass;
14. every scroll owner consuming at least 10% of input-to-submit CPU time has an
    evidence-backed reduction or resistance ruling, and no admitted material
    optimization remains after two consecutive full workload sweeps;
15. the live ledger contains receipts for every admitted and zero-change cell,
    including cells discovered after the ignition queue was green; and
16. a final bidirectional entrance-to-receipt and receipt-to-entrance sweep
    across every required lens finds no competing truth, uncovered drawable
    state, inconsistent control/axis path, unbounded scaling, false invalidation,
    avoidable hot-path allocation, or unowned performance cost; and
17. the final industry comparison meets or exceeds every applicable shared-line
    property, with repository-specific differences justified by stronger local
    correctness, boundedness, or measured performance evidence.
