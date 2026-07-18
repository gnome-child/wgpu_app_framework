# Presentation Compiler source census

**Status:** TERMINAL EXECUTION CENSUS — PC-005 FAILED THE FLAT-COMMIT GATE

**Date:** 2026-07-17

**Formulation snapshot:** `master` at `1fe6af199b501163e1f5ccf0d8065e9b079a43ea` with inherited uncommitted SE-009 changes

**Campaign authority:** [Presentation Compiler campaign](2026-07-17-presentation-compiler-campaign.md)
**Purpose:** crash-safe source evidence and ownership map; this census is not a second plan

Line numbers below describe the formulation worktree and may move during execution. PC-000 revalidated the load-bearing source facts before production edits; later checkpoint receipts supersede line numbers without changing this file's ownership rulings.

Evidence labels:

- **VERIFIED:** read directly from current source or an existing receipt.
- **INFERENCE:** the source/receipt supports the claim, but a campaign counter must prove causality.
- **OPEN:** unresolved and assigned to a checkpoint.
- **PROTECTED:** already-established architecture that survives unless contradicted by a new receipt.

## Terminal execution delta

The detailed sections below preserve the formulation census. The following
live-source facts supersede their descriptions of the pre-execution path:

- virtual rows are content-local; canonical scroll no longer participates in
  their rectangles, clips, layout paths, or row scene keys;
- keyed residency mutates end-editable installed-view/composition sequences,
  preserves overlap identities, and binds/constructs only entering rows;
- layout stores ordinary immutable chunks plus persistent keyed row sequences;
  scene stores persistent keyed row-fragment sequences and visits/paints only
  entering row bodies on the keyed path;
- table providers now publish a capability-gated residency revision; unknown
  callback-backed sources retain the correctness fallback;
- semantic commits are reused for typed residency-only changes, and spatial
  topology owns compiled residency membership/draw order;
- scene row-sequence lookup is generation-exact. Each `RowSequence` carries an
  immutable identity, `Layout` carries the exact delta predecessor, and scene
  accepts only a `Weak::ptr_eq` match. This repairs the observed
  `InvalidSpatialTopology(UnknownNode(73))` active/pending race;
- table-track owner discovery indexes frames once and walks bounded ancestry,
  removing the earlier full-frame rescan per track.

The terminal unmet boundary is also exact. `scene::paint::commit_builder`
registers a flat resident node set; scoped fragments lower into one flat draw
order; `scene::SpatialTopology` assigns flat draw indices and flat residency
membership; `scene::Residency` copies resident node/order snapshots; and
`render::retained::PendingPlan` rebuilds a flat node map and walks the flat draw
order. Cache cleanup and table-track projection also visit resident
collections. Thus unchanged row bodies are literal zero at expensive owners,
but total candidate CPU remains `O(R)` and PC-005 fails.

An immutable row-commit prototype was removed because row nodes receive table
track content in a later global order phase and all four downstream consumers
still require flat indices. The admissible successor is one segmented
generation-spanning representation for commit nodes/order, spatial topology,
residency membership, and render-plan sections, with table tracks carried as a
separate structurally shared row-order phase. A wrapper that subsequently
flattens, per-row surfaces, reordered tracks, or worker placement is not an
admissible substitute.

---

## Executive census

The current table/list pipeline already bounds provider binding and preserves composition identity, but it does not preserve presentation work at the same granularity. A residency request invalidates layout, rematerializes a cloned installed view, reconciles composition, constructs layout frames, scans those frames for scene fragments, then feeds the retained GPU renderer. The expensive part remains synchronous in the native event/redraw path.

The concrete verified law violation is:

```text
resolved scroll property
  → subtracted into every virtual row rectangle
  → rectangle and full Viewport enter layout::frame::SceneKey
  → scene paint cache treats every moved row as a changed presentation key
```

Stable row slots therefore do not imply stable scene cache keys.

The intended repair is not “remove geometry from keys.” It is:

```text
content-local row geometry + fixed viewport coverage
  + existing parent spatial/property translation
  + keyed residency deltas
  + retained renderer-independent row derivations
```

The source does not yet prove whether this synchronous Qt/GTK-class repair is sufficient. Active/pending CPU preparation, immutable snapshots, and workers remain conditional on PC-006.

---

## Formulation worktree

`git status --short` showed pre-existing modifications in:

| Territory | Modified files | Census disposition |
|---|---|---|
| earlier ledgers | Scrolling Engine campaign and census | protected evidence; do not overwrite |
| semantic identity | `src/composition/tree.rs` | protected inherited work; current owner remains composition |
| diagnostics | `src/diagnostics/mod.rs`, `render.rs`, `residency.rs` | likely SE-009 instrumentation; PC-000 attributes |
| layout | `src/layout/frame.rs` | overlaps the concrete defect; inspect before any edit |
| virtualization | `src/list.rs`, `src/table.rs` | overlaps campaign territory; preserve inherited changes |
| renderer | `src/render/debug.rs`, `renderer.rs`, `retained.rs`, `text_renderer.rs` | retained-renderer and SE-009 work; lower seam protected |
| presentation | `src/runtime/presentation.rs` | overlaps orchestration and current rebuild path |
| scene | `src/scene/primitive.rs`, `spatial.rs` | protected spatial/fragment work |
| view projection | `src/view/node/builder.rs`, `mod.rs`, `traversal.rs` | current materialization path and inherited changes |
| tests/tools | architecture/composition/layout/residency tests and renderer-debug tool | preserve and extend rather than replace |

This campaign added documentation only at formulation. Production ownership begins only when PC-000 records an execution base.

---

## Current synchronous call path

### Native/runtime entry

| Stage | Source | Verified behavior | Campaign question |
|---|---|---|---|
| native event pump | `src/platform/**` and application shell/lifecycle path | events and redraw ultimately enter runtime presentation | pin input-to-redraw and event-pass phase counters |
| frame need classification | `src/runtime/presentation.rs:45-80` | `FrameNeed::Residency` maps to `Invalidation::Rebuild` | can residency remain a typed delta instead? |
| selected residency entry | `src/runtime/presentation.rs:1524-1550` | `prepare_frame` calls `present_residency` synchronously before layout when the primary need is residency-only | measure selected-candidate monopolization |
| refinement/re-entry | `src/runtime/presentation.rs:830-925` | after composing layout, the convergence/reveal loop can call `present_residency` again and recompose layout | count passes and prevent resident-sized re-entry |
| normal rebuild | `src/runtime/presentation.rs:1210-1270` | `present_with_virtual_pin` removes layout cache and rebuilds application view projection | separate true app rebuild from residency |

### Residency projection

`src/runtime/presentation.rs:1141-1212` is load-bearing:

1. verifies the window still exists;
2. removes the window layout cache;
3. refreshes virtual pins;
4. clones the previously installed view twice (`previous` and mutable `view`);
5. projects table widths;
6. scans selectable virtual lists and reconciles selection;
7. clones materialization and measurement maps;
8. calls `view.materialize_virtual_lists(..., Some(&previous))`;
9. projects virtual selection, active table cells, and input feedback;
10. reuses virtual-row text buffers where possible;
11. calls `composition.prepare(window, &view)`;
12. prunes removed interaction state;
13. projects retained interaction and focus;
14. installs the prepared composition and clones the presented view;
15. records composition reconciliation time.

**VERIFIED:** this path performs no application `view(model, context)` callback; it clones and rematerializes the installed view. The earlier shorthand “view rebuild” must distinguish this residency projection from a full application view rebuild.

**VERIFIED:** layout cache removal happens before the residency view/composition work.

**INFERENCE:** the broad view clone/projection and composition preparation contribute resident-window work. PC-000/PC-004 must count nodes and bytes rather than infer complexity from cloning syntax.

### Downstream presentation

The resulting installed composition feeds layout and scene construction through the existing runtime drain/prepare path. The exact platform call chain must be re-traced at PC-000 because SE-009 has modified diagnostics and continuation scheduling.

**VERIFIED from current campaign receipts:** layout p95 is 12.489 ms and scene assembly p95 is 14.825 ms in the historical native run.

**OPEN:** which portions occur on every residency candidate after the latest runway changes, and which are diagnostic overhead?

---

## Canonical scroll ownership

`src/interaction/scroll.rs` and the sealed SE-002 receipt establish:

| Fact | Owner | Current disposition |
|---|---|---|
| per-axis canonical value and monotonic revision | `interaction::scroll::AxisAdjustment` | sole input truth for relative, absolute, scrollbar, reveal, and programmatic requests |
| lower/upper/page/step/page-increment configuration | authored by layout, stored/applied atomically by `AxisAdjustment` | configuration changes may clamp the canonical value and advance its revision |
| `desired` offset | derived from the two canonical adjustments | projection, not a second owner |
| `resident_accepted` | residency/presentation receipt | may lag desired; never canonical input |
| `scene::Properties` scroll value | compatible sample of canonical adjustment for a scene handoff | downstream snapshot, not canonical input |
| `present_submitted` spatial/property state | presentation receipt | active input/pixel truth after submission |

**VERIFIED:** exact axis values survive canonical adjustment, residency admission, scene-property projection, active/pending projection, and the installed submitted snapshot.

**RULING:** “property scroll” in this campaign means a downstream presentation cause. It does not transfer canonical value ownership to `scene::Properties`.

**RULING:** PC-002 must prove that after `AxisAdjustment` changes, no semantic/presentation compiler work runs inside prepared coverage; only compatible sampling, spatial evaluation, retained draw, and receipt work remain.

---

## Virtualization ownership

### Model and state

`src/list.rs` defines:

| Fact | Current owner/shape | Evidence |
|---|---|---|
| logical key | `list::Key` supplied by `Model` | model key/index inverse assertions |
| item content revision | `Model::item_revision` | captured in `DesiredItem` |
| membership revision | `Model::membership_revision` | observed by slots |
| ordered membership journal | `Model::changes_since` returning `Change::{Insert, Remove, Replace, Move}` | existing delta authority for membership/order mutation |
| row factory | `Rc<dyn Factory>` | `State` and `Slots::materialize` |
| active/recycled slots | `Rc<RefCell<Slots>>` | `State` |
| variable measurements | `Rc<RefCell<variable::Region>>` | `Measurements` |
| overscan | bounded to 32 rows | `State::overscan` |
| transition materialization | bounded to 80 rows | `MAX_TRANSITION_MATERIALIZED_ROWS` |
| leading runway | maximum two viewports; minimum one complete viewport where budget permits | runway calculation |
| recycled slots | bounded to 32 | `MAX_RECYCLED_SLOTS` |

### Slot materialization

`src/list.rs:642-711`:

- takes the prior active map;
- computes desired keys and departing keys;
- unbinds departing slots and pushes them to recycle;
- reuses unchanged-key/unchanged-revision bindings;
- rebinds revised or entering items;
- clones the bound `view::Node` for every desired item into a fresh `Vec`;
- builds a fresh active `HashMap`;
- trims the recycle pool to 32.

**VERIFIED:** factory `bind` is limited to entering or revised items.

**VERIFIED:** unchanged active rows preserve their slot, bound node, logical key, and revision.

**VERIFIED:** every desired resident row still contributes a cloned node and active-map insertion during materialization.

**INFERENCE:** the latter is `O(R)` bookkeeping and may be material at large runways. PC-004 must measure and determine whether a persistent ordered map/index is necessary.

**VERIFIED:** a same-key `Move` is a real model delta even when entering/departing sets are empty. The campaign’s pure-scroll delta and model-order delta are distinct currencies.

### Runway policy

The current bounded-runway constants are a mitigation, not the architectural fix:

```text
MAX_TRANSITION_MATERIALIZED_ROWS = 80
MAX_LEADING_RUNWAY_VIEWPORTS = 2
MIN_LEADING_RUNWAY = 1 viewport
MAX_RECYCLED_SLOTS = 32
overscan ≤ 32
```

**PROTECTED:** bounded runway, latest-intent coalescing, and correct required coverage.

**OPEN:** the final policy’s native confirmation after inherited SE-009 changes.

**LAW:** a larger runway may reduce crossing frequency but cannot excuse `O(R)` work per crossing or unbounded memory.

---

## Table/provider boundary

`src/table.rs` contains substantial UI-authority tissue:

- provider held as `Rc<dyn Provider>`;
- key/index/record/revision functions held in `Rc<dyn Fn...>`;
- cell projections held in `Rc<CellProjection<_>>`;
- projected records, ordering, and columns held behind `RefCell`;
- presentation and sort projections held in `Rc<Cell<_>>`;
- row context returns application triggers;
- many cell builders capture application callbacks.

**VERIFIED:** the live table/provider/control graph is not generally `Send` or immutable.

**VERIFIED:** some validation/edit mapping callbacks are `Send + Sync`, but this does not make the whole graph worker-safe.

**VERIFIED PROVIDER GAP:** `table::Rows` currently implements `membership_revision() -> 0` and `changes_since(..) -> Vec::new()`. Generic list models have an ordered mutation journal; the table adapter does not currently publish one. PC-001 must make this gap explicit and PC-005 must repair it before claiming incremental table insert/remove/replace/move behavior.

**RULING:** moving the current object graph to workers is not viable and is not an admitted shortcut.

**OPEN PC-008 only:** whether a measured residual requires an immutable, generation-tagged provider snapshot API.

If admitted, provider callbacks execute under UI authority and export values. They do not cross the boundary as capabilities.

---

## Composition ownership

`src/composition/tree.rs` currently owns:

- process-transient retained `NodeId`;
- `ContentRevision`;
- the retained semantic `Tree` and `Node` hierarchy;
- parent/child relationships;
- authored and transiently projected view scene keys;
- element/subject/provided-row/table-cell identity;
- `Changes` sets for added, changed, removed, departed, removed elements, and removed table cells.

The documented v1 reconciliation rule preserves explicit identity across sibling reorder under one parent, keeps id-less nodes positional, and reports a parent move as remove-plus-add.

### Current node key details

Composition `Key` includes:

- ordinary role/axis;
- provided row role/axis/list/slot;
- table cell role/axis/table/column;
- header cell role/axis/cell.

The logical provider row identity is carried separately as `ProvidedRow`. Architecture doctrine establishes logical table-cell identity as:

```text
(table id, provider row key, column id)
```

**PROTECTED:** composition owns identity and the root semantic change stream.

**RISK:** the provided-row composition key visibly includes recycled slot. Existing tests must be read in PC-001 to distinguish process identity reuse from logical cell identity. No campaign cache may adopt slot identity as model identity.

**INFERENCE:** `composition.prepare` may visit more of the tree than the semantic delta requires. PC-000 must count visited/reconciled/rebuilt nodes, and PC-005 must preserve `Changes` as the only semantic invalidation authority.

**FORBIDDEN:** a `PresentationNodeId`, externally mutable compiler dirty API, or independent content revision.

---

## Layout ownership and concrete clock contamination

### Fixed-height virtual rows

`src/layout/algorithm.rs:440-476`:

1. constructs `Viewport` from the fixed viewport rectangle, content height, and node scroll offset;
2. resolves/clamps the scroll offset;
3. asks the model for the viewport request;
4. creates a row clip from visible content;
5. pushes the virtual-list container frame;
6. visits every materialized child;
7. computes logical row position;
8. computes absolute row `y` by adding viewport origin and subtracting resolved scroll;
9. recursively lays out each child with that absolute rectangle.

The load-bearing expression is:

```text
y = viewport_rect.y + row.index × row_height − resolved_scroll.y
```

**VERIFIED:** one property change changes the absolute rectangle of every resident child even when model, key, content revision, constraints, and logical row position are stable.

### Variable-height rows

`layout_variable_virtual_list` also resolves current offset and interacts with the mutable measured region. Its coordinate and dependency behavior must be independently classified during PC-001/PC-002; the fixed-height proof cannot be blindly generalized.

### Flat layout product

The layout product exposes a frame sequence and additional table/chrome/path facts. Scene painting iterates `layout.frames()` by layer/panel and drawable scroll path.

**INFERENCE:** fresh frame-vector construction and scans are closer to `O(resident subtree)` even when cached bodies hit. PC-003/PC-005 decide whether retained derivation is per row, per small chunk, or structurally shared segments.

**PROTECTED:** layout remains the sole owner of local measurement, geometry, and hit meaning.

**FORBIDDEN:** renderer or compiler reconstructing local geometry or scroll ancestry.

---

## `Viewport` census

`src/layout/viewport.rs:6-15` stores:

| Field | Clock classification |
|---|---|
| `rect` | container/local-target geometry |
| `visible_frame` | coverage/clip geometry |
| `visible_content` | coverage/clip geometry |
| `content` | semantic/layout content extent |
| `offset` | layout-consumed requested projection derived from canonical `AxisAdjustment` |
| `max` | geometry-derived scroll bound |
| `resolved` | locally clamped layout projection, not canonical scroll ownership |

`Viewport` is therefore a mixed-clock aggregate.

**VERIFIED:** it derives `PartialEq + Eq` as one value.

**VERIFIED:** placing the whole value in a reusable scene key couples downstream requested/resolved scroll projections to geometry/content/coverage.

**RULING:** PC-001 must split cache currency by owner/clock. This need not imply deleting the useful runtime aggregate everywhere.

---

## `layout::frame::SceneKey` census

`src/layout/frame.rs:219-238` defines a key containing:

- parent `NodeId`;
- composition content revision;
- `rect` and `active_rect`;
- clip and floating-layer state;
- focus, selected, and active-item presentation state;
- provided row and table row;
- header presentation;
- label/shortcut widths;
- complete optional `Viewport`.

`Frame::scene_key` copies those values at `src/layout/frame.rs:707-732`.

### Field ruling

| Field | Current role | Campaign disposition |
|---|---|---|
| parent / content revision | stable owner facts | retain where derivation depends on them |
| local `rect` / `active_rect` | legitimate geometry if truly local | repair coordinate contamination; do not delete blindly |
| clip | split fixed coverage, local clip, and property-derived output | key only by true owner inputs |
| focus/selected/active | semantic/visual state | explicit revisions/dependencies; no viewport coupling |
| row/table/header facts | semantic/local facts | carry owner identity/revision |
| widths | layout/text inputs | retain with actual revision/constraint |
| whole `Viewport` | mixed clock | reject from keys spanning property ticks |

**VERIFIED:** `SceneKey` is consumed by scene paint caching.

**OPEN:** whether one general `SceneKey` should remain after clocks are split, or whether owner-specific fragment keys are required. Shapes are candidates.

---

## Scene paint and fragment reuse

`src/scene/paint/mod.rs:532-605`:

- filters and iterates layout frames for a layer/panel;
- records every seen `NodeId`;
- computes `frame.scene_key()`;
- reads visual state;
- looks up a cached frame by `NodeId` and compares key + visual;
- on hit, clones cached presentation;
- on miss, paints the frame into one or more `Scene` bodies and stores `CachedFrame`;
- creates a fresh `Fragment` for assembly.

**VERIFIED:** scene paint already has stable `NodeId`-addressed cached bodies.

**VERIFIED:** the contaminated `SceneKey` can turn property motion into cache misses.

**VERIFIED:** even cache hits still require scanning frames and assembling fresh fragment vectors.

**INFERENCE:** the 163–170 node paints / 81–86 primitive preparations are substantially caused by cache-key contamination. PC-004 must prove how much disappears after PC-002/PC-003.

**OPEN:** exact cost of cache-hit cloning, fragment-vector construction, layer filtering, and commit splice/copy.

**RULING:** scene-owned reusable fragments are the likely first-class retained unit. They remain renderer-independent and cannot own scroll truth.

---

## Scene, spatial, commit, and renderer boundaries

### Scene

`src/scene` owns typed primitives, groups, content, spatial topology/projection, commits, residency, properties, and the final stack.

Current compatibility paths can translate/project primitives while evaluating spatial properties. This is downstream property output, not permission to store current offset in semantic fragment identity.

### Spatial

Existing architecture tests require:

- candidate topology as the only scroll ancestry compiler;
- no GPU/runtime adapter reconstruction of ancestry;
- canonical scene-property lookup within the scene handoff, distinct from canonical `AxisAdjustment` ownership;
- submitted geometry for input/access projection.

**PROTECTED:** content-local row repair must reuse this grammar.

### Commit

The commit builder and compatibility projection can still copy/flatten primitives. PC-000 instrumentation must distinguish:

- row fragment rebuild;
- fragment reuse;
- commit segment splice;
- commit node/primitive copy;
- property projection.

**OPEN:** whether current commit representation supports `O(D_s)` pure-scroll and `O(A_m)` model-order structural sharing or needs a breaking value-semantic segment API.

### Renderer

The retained renderer already has:

- active/pending realization;
- sliced/budgeted preparation;
- carried composition identity/revisions in resource keys;
- shared text renderer/cache/atlas;
- batched draw plan;
- successful-present receipt distinctions.

**PROTECTED:** renderer consumes only `scene::Stack`.

**RULING:** the renderer cannot receive `PresentationTree`, layout frames, providers, or worker snapshots.

**RULING:** per-row GPU layers/surfaces are not the presentation-fragment model.

---

## Existing active/pending grammar

The native adapter and retained renderer contain active/pending presentation state and stale/freshness laws established by the Retained Renderer campaign.

Current doctrine distinguishes:

- preparation attempt;
- candidate readiness;
- activation;
- GPU submission;
- `present` call;
- visible/successful-present receipt.

**PROTECTED:** one grammar and atomic generation consistency.

**OPEN PC-007 only:** whether CPU presentation preparation must become an upstream payload of the same generation.

**FORBIDDEN:** a separate CPU epoch, queue, activation, or visibility receipt.

---

## Input, selection, editing, IME, and accessibility

The runtime projects interaction and focus onto the prepared view/composition, while layout/spatial state supplies submitted geometry used by hit/access paths.

Required control behavior from existing campaigns:

- read-only text selection and copy;
- editable caret, draft, insertion/deletion, focus, commit/cancel;
- caret established before typing;
- IME target and composition geometry;
- accessibility roles/bounds/actions;
- shift-range selection and multiple resident editable rows;
- focus/capture/hover retirement for departed semantic owners;
- correct re-entry and recycling by logical identity.

**PROTECTED:** no cached visual fragment may become semantic authority.

**OPEN:** which hit/access/IME fragments can be retained as local derivations and which are projected dynamically from submitted properties.

PC-001 owns this field-level classification; PC-002 and PC-003 own equivalence witnesses.

---

## Current cache/reuse map

| Layer | Existing retained fact | Existing miss/rebuild cause | Campaign disposition |
|---|---|---|---|
| list slots | key/revision-bound nodes | entering/revised/departing | preserve; make downstream equally granular |
| row text buffers | explicit reuse from previous view | content/binding changes | preserve and measure |
| composition tree | `NodeId`/content revisions | reconciled semantic changes | preserve root identity/change authority |
| runtime layout cache | whole-window entry | explicitly removed on residency | replace coarse invalidation with retained derivation |
| scene paint cache | `NodeId → CachedFrame` | `SceneKey` or visual mismatch | repair key clocks; retain bodies/fragments |
| scene commit/spatial | retained typed topology/properties exist | broad assembly/copy still possible | splice/structural-share by delta |
| renderer content | carried semantic/geometry/topology key | actual content/geometry/target changes | preserve |
| renderer plan | retained batched plan | semantic/topology/target changes | preserve; property ticks bypass |
| text renderer | shared retained text resources | actual text/style/target changes | preserve |

Stable identity currently stops too early: slot → composition → layout/scene reconstruction. The campaign extends stable lifetime through renderer-independent presentation.

---

## Complexity census

Let `R` be resident rows, `K` visible columns, `D_s` the pure-scroll union of entering/departing/content-revised rows under stable model order, and `A_m` the declared affected set for insert/remove/replace/move plus true order/height-prefix dependencies.

| Operation | Current source indication | Required proof |
|---|---|---|
| provider bind/setup | `O(entering/revised rows × affected columns)` | retain exact row/cell counters |
| active slot map / node vector | loops all desired residents | eliminate expensive overlap visits; classify bounded index operations |
| view clone/projection | clones installed tree and projects resident state | measure nodes/bytes; target keyed structural sharing |
| composition prepare | receives full materialized view | count visited/reconciled nodes; target changes + bounded ancestry |
| fixed-row layout | recursively visits every materialized row/cell child | unchanged row/cell overlap zero measure/place |
| scene paint cache | scans all layout frames; misses when key moves | unchanged overlap zero paint and no full scan |
| fragment/commit assembly | fresh vectors/copy paths visible | target keyed segment splice |
| GPU realization | retained and batched | unchanged overlap zero content upload |

The required fixed-delta experiment crosses resident windows 32, 64, 128, and 256 with column counts 3, 12, and 48 where feasible, using identical one-row and four-row deltas. It also compares logical model sizes 10k, 100k, and 1m at fixed residency/delta. Timing is secondary; unchanged row/cell owner-work counters remain zero and entering-cell work follows `K × entering rows + revised cells`.

---

## Historical receipt census

Receipt:

`target/release/examples/renderer-receipts/control-gallery-500px-idle-1784317515595.txt`

This ignored file was present during formulation. Load-bearing values are copied here:

| Field | Value |
|---|---:|
| frames | 761 |
| GPU draw p95 | 1,745 µs |
| property draw p95 | 2,007 µs |
| semantic draw p95 | 1,942 µs |
| surface acquire p95 | 42 µs |
| batch preparation p95 | 329 µs |
| encode/submit/present p95 | 1,335 µs |
| scene assembly p95 | 14,825 µs |
| native event pass p95 | 30,819 µs |
| view rebuild p95 | 6,187 µs |
| composition reconciliation p95 | 4,006 µs |
| layout p95 | 12,489 µs |
| full redraws / layout reuses | 312 / 131 |
| scene nodes reused / rebuilt | 43,200 / 52,146 |
| wheel samples / needing residency | 768 / 750 |
| typical residency node paints | 163–170 |
| typical primitive preparations | 81–86 |

At 240 Hz, the frame interval is approximately 4,167 µs.

### Receipt limitations

- the p95 values are not additive;
- the receipt predates final confirmation of the bounded-runway change;
- “view rebuild” may aggregate full view callback and residency projection unless the current diagnostic schema distinguishes them;
- present call is not scanout;
- one machine/workload cannot establish general performance;
- diagnostic overhead must be measured;
- phase-specific property/residency distributions are required.

PC-000 creates the actual campaign baseline. This historical receipt is causal evidence, not the final regression rail.

---

## Required diagnostic additions

The exact schema is ratified at PC-000. The census requires at least:

### Population and binding

```text
primary_frame_need
change_species_property / residency / semantic / device / diagnostic
resident_rows_before / after
entering_rows / departing_rows / overlapping_rows / revised_rows / moved_rows
membership_revision_before / after
insert_changes / remove_changes / replace_changes / move_changes
visible_columns / entering_cells / revised_cells / overlapping_cells
pinned_rows / recycle_rows
provider_model_reads
factory_row_binds / factory_cell_binds
```

For the fixed-`K` table fixture:

```text
factory_row_binds = entering_rows + revised_rows
factory_cell_binds = K × entering_rows + revised_cells
overlap row/cell provider binds = 0
```

### View and composition

```text
view_nodes_visited / created / reused / cloned
overlap_view_nodes_visited
composition_nodes_visited / reconciled / created / reused
overlap_composition_nodes_reconciled
```

### Layout, text, fragments

```text
layout_nodes_visited / measured / placed / reused
overlap_rows_measured / placed
overlap_cells_measured / placed
row_fragments_reused / rebuilt
overlap_fragments_rebuilt
overlap_cell_fragments_rebuilt
text_layout_calls / overlap_text_layout_calls
hit_fragments_rebuilt
access_fragments_rebuilt
ime_fragments_rebuilt
```

### Scene and commit

```text
scene_frames_visited
scene_fragments_reused / rebuilt
scene_primitives_copied
commit_segments_spliced
commit_nodes_copied
overlap_scene_node_paints
overlap_cell_scene_node_paints
overlap_primitive_prepare_calls
overlap_text_prepare_calls
overlap_content_upload_bytes
unattributed_rebuilds
```

### Clock proof

```text
semantic_key_checks
semantic_keys_changed_by_property_only_motion
row_geometry_revisions_changed_by_property_only_motion
row_fragment_keys_changed_by_property_only_motion
```

The last three are literal-zero invariants.

---

## One-Way territory census

Expected allowed graph:

```text
semantic path:
  application/provider facts
    → composition
    → private retained presentation derivation
    → layout + scene owners
    → runtime presentation
    → scene::Stack
    → render
    → surface/platform

property path:
  input → interaction::scroll::AxisAdjustment
    ← layout-authored range/page configuration
    → desired/resident projection
    → scene::Properties compatible sample
    → runtime submitted spatial truth
    → scene::Stack → render
```

Current slot-map doctrine already permits runtime/renderer/platform to depend downward into UI territory but not reverse edges.

### Risks to inspect

| Risk | Source territory | Required result |
|---|---|---|
| compiler becomes independent invalidation owner | new/private UI code | derived frontier only |
| runtime callback smuggled downward | compiler scheduling API | pure inputs/results |
| scene learns renderer types | fragment contract | renderer-independent values |
| renderer sees providers/layout/compiler | render API | `scene::Stack` only |
| diagnostics becomes production dependency | new counters | owner-published observation |
| public module/crate created for convenience | module visibility | private internal stage until independently admitted |
| spatial evaluation duplicated | table/compiler/renderer | existing canonical adjustment → scene topology/properties projection only |
| snapshot keeps live callbacks/interior mutability | conditional API | immutable values and commands |

The current UI internal SCC is not itself evidence for a new crate boundary.

---

## Architecture ratchet census

Existing tests to preserve include witnesses that:

- composition owns retained identity, not behavior;
- `scene::Stack` is the sole native renderer handoff;
- GPU keys borrow `NodeId` and content/geometry/topology revisions;
- property serial, residency revision, presentation epoch, and renderer IDs are not GPU content identity;
- residency borrows composition identity and cannot create a renderer entrance;
- obsolete residency candidates cannot activate;
- scene-property lookup is canonical within its handoff while `AxisAdjustment` remains canonical scroll input;
- scene topology is the only scroll ancestry compiler;
- GPU/runtime adapters do not reconstruct ancestry.

New required witnesses:

1. `row_presentation_keys_are_content_local_and_property_free`;
2. `presentation_tree_borrows_composition_identity_and_changes`;
3. `property_scroll_bypasses_the_presentation_compiler`;
4. `residency_delta_preserves_every_overlap`;
5. `presentation_tree_is_private_and_renderer_invisible`;
6. conditional `one_presentation_generation_grammar`;
7. conditional `worker_snapshots_are_pure_candidates`.

Source-string tests are resurrection alarms, not complexity proof. Each ratchet needs behavior/counter evidence.

---

## Threadability boundary

### Currently UI-affine or non-worker-safe

- application view callback and model store;
- `Rc<dyn Model>` / `Rc<dyn Factory>`;
- table providers and cell projections;
- `RefCell`/`Cell` projected state;
- application triggers/commands;
- semantic interaction/focus/draft/selection ownership;
- WGPU device/queue/surface realization;
- presentation activation and visible receipts.

### Potentially pure only after contract

- immutable logical row facts;
- immutable style/constraint snapshots;
- text measurement/shaping inputs if the current text engine boundary permits;
- local layout derivation;
- renderer-independent scene fragment construction;
- keyed segment assembly.

### Never worker authority

- canonical `interaction::scroll::AxisAdjustment` value/revision;
- semantic model mutation;
- `NodeId` or owner revision minting;
- residency truth;
- active/pending selection or activation;
- input/hit/access active truth;
- GPU/surface mutation;
- visible receipt.

PC-009 can move only work already proven pure and delta-bounded. PC-007 must first make the worker result pending payload in the one presentation generation; PC-008 must be complete or rejected with proof existing inputs are already immutable/pure. Worker placement cannot legalize `O(R)` or `O(R×K)` reconstruction.

---

## Open questions by checkpoint

### PC-000

- Which exact diagnostic phases include the current 30.819 ms event pass?
- What is the post-runway baseline?
- How much overhead do counters add?
- Does large text stay clean under an identical script?

### PC-001

- Which `Viewport` fields must split into owner-specific keys?
- How will table `Rows` expose truthful membership revisions and ordered `Change` events instead of revision 0/empty changes?
- What revision represents local constraints/column metrics?
- What is the smallest complete row-fragment contract?
- How are hit/access/IME local artifacts represented without duplicate truth?

### PC-002

- How do variable-height correction, sticky chrome, nested scroll, and clips express local coordinates?
- Which current consumers assume absolute row rectangles?
- Can all semantic projection use existing presented spatial evaluation?

### PC-003

- Per row or small immutable chunk?
- Which text artifacts can be shared without tying to absolute viewport?
- Can current scene `Fragment` become the retained value, or is a smaller owner type needed?
- What cache bound preserves edit/focus pins without unbounded history?

### PC-004

- How much repaint disappears solely from clock divorce?
- Which resident-sized scans remain in view, composition, layout, or commit?

### PC-005

- What ordered persistent structure permits key/order splices without full maps/vectors?
- How do row and cell counters scale across 3/12/48 columns, and what is the exact `A_m` for same-key moves and height-prefix effects?
- Can `composition.prepare` consume targeted changes without another authority?
- Does commit assembly support segment structural sharing?

### PC-006

- Does optimized on-thread work meet the campaign `B` input/layout/scene budget, `T` property/event budget, and `2T` input-to-`present` budget?
- Which optional mechanism, if any, owns the measured residual?

### PC-007–PC-009

- Only the questions explicitly admitted by PC-006 may remain open.

---

## Fixed-point resume checklist

Before editing:

- [ ] Read the campaign and this census completely.
- [ ] Record current HEAD/branch/status.
- [ ] Reconcile inherited changes with the formulation list.
- [ ] Verify `FrameNeed::Residency` mapping and `present_residency` path.
- [ ] Verify `AxisAdjustment` canonical value/configuration ownership and downstream scene-property sampling.
- [ ] Verify `list::Change`/`changes_since` and the table `Rows` membership-journal gap.
- [ ] Verify fixed and variable virtual-row coordinate code.
- [ ] Re-census every `SceneKey` and `Viewport` field.
- [ ] Trace scene cache lookup and fragment/commit assembly.
- [ ] Confirm renderer `scene::Stack`-only handoff and active/pending grammar.
- [ ] Freeze receipt schema, native script, and large-text control.
- [ ] Select only the first unmet checkpoint exit.

At closeout:

- [ ] No property-clock value remains in a reuse key spanning property ticks.
- [ ] No unchanged overlap visits an expensive owner phase.
- [ ] Row × column and multi-model-size fixed-delta counters are flat except declared entering/revised work.
- [ ] No full residency rebuild or dual presentation path remains.
- [ ] Optional mechanisms have explicit accept/reject receipts.
- [ ] Final source ownership matches `docs/master_design.md`.
- [ ] This census contains no stale “current” fact.
