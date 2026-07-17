# Scroll source census

Status: **SC-010 FINAL CENSUS — CLOSED**

Date: 2026-07-16

Execution base: `master` at pushed SC-009 boundary `ad006439` plus the SC-010 closeout revision

Authority: `docs/audits/2026-07-16-payload-neutral-scroll-architecture-campaign.md`

This is a production-path census, not a claim that every listed path is wrong. It records every known owner, projection, interpreter, representation adapter, scheduling boundary, and diagnostic consumer that can affect scrolling. SC-002, SC-008, and SC-010 update this file rather than relying on memory or a narrower symbol search.

## 1. Fact ownership baseline

| Fact | Current owner | Derived readers/writers | Campaign disposition |
|---|---|---|---|
| Raw precise wheel/trackpad delta | `src/platform/event.rs::scroll_delta` | `src/host/event.rs`, `src/shell/event.rs`, `src/shell/input.rs` | SC-005 closed lossless pixel/fractional-line transport through target routing; the adapter does not quantize visual motion. |
| Target-local fractional remainder plus requested/desired and resident-accepted offset | `src/interaction/scroll.rs::Scroll` | `src/interaction/mod.rs`, `src/session/interaction/scroll.rs` | SC-004 closed state names; SC-005 made interaction the sole fractional accumulator and kept desired/resident offsets integral. |
| Request, clamp, resident acceptance | `src/runtime/input/dispatch.rs::apply_scroll_transition` | presented layout and virtual request lookup | SC-004 closed the transition vocabulary; desired intent survives residency rejection. |
| Legal range and resident acceptance | `src/layout/mod.rs::ScrollProjection` and `Layout` methods | `src/layout/viewport.rs`, `src/layout/frame.rs`, `src/scene/residency.rs` | Layout/residency remains range owner; axis ownership becomes typed in SC-002/SC-004. |
| Node-to-scroll ancestry | candidate-owned `src/scene/spatial.rs::SpatialTopology::{frame_scroll_paths,binding_scroll_paths}` for submitted rendering/input | lazy present-submitted frame/content evaluators and generated renderer adapters | CLOSED. Layout retains only `scene_scroll_paths`, a pre-candidate construction projection generated from frame parentage and confined to residency/scene scope emission. It is not named or consumed as submitted ancestry. |
| Candidate property values and dirty indices | `src/scene/paint/mod.rs::property_snapshot` and `src/scene/commit.rs::Properties` | scene stack, renderer, compatibility scene | SC-003 closed stable topology-local indices, block-shared values, and authoritative dirty production independently of spatial replacement. |
| Candidate spatial ancestry, surface roots, clip/effect references, target identity, and axis ownership | `src/scene/spatial.rs::SpatialTopology`, immutably owned by `src/scene/commit.rs::Commit` | semantic/drawable projection, compatibility emission, retained plan compiler, renderer property adapters | SC-002 closed this ownership replacement. Draw order remains structural input/output, not an ancestry owner. |
| Requested and present-submitted presentation epochs | private `src/session/window.rs::PresentationState` | runtime candidate selection and successful render feedback | SC-004 closed one owner. Requests advance requested state; retry does not; stale, duplicate, and future receipts cannot advance present-submitted state. |
| Installed GPU property generation | retained node/scroll `PropertySlot::property_serial` derived from `scene::Properties::{serial, predecessor}` | sparse property preparation and explicit full resynchronization | SC-004 closed skipped-generation recovery. Sparse mutation requires the direct predecessor and exclusive commit ownership. |
| Runtime-presented geometry | `src/runtime/access.rs::finish_render_report_with_kind` installs `src/scene/spatial.rs::SpatialSnapshot` from the successful submitted stack | `src/runtime/mod.rs::PresentedGeometry`, pointer/drag/routing/context menu, scrollbar routing, IME | SC-008 closed lazy topology-backed projection. Candidate/skipped geometry cannot replace this snapshot. |
| Surface acquire, queue submit, and present call | `src/render/surface.rs` | runtime report and scroll/render diagnostics | SC-006 records actual call boundaries. `present_submitted` remains a submit/present-call fact with no scanout claim. |
| Redraw demand and in-flight deduplication | `src/platform/mod.rs::RedrawRequests` | all backend request, delivery, retry, continuation, and close paths | SC-006 closed one immediate issuance owner. Native runner completion clocks and duplicate demand sets are deleted. |

## 2. Scope/spatial interpreter census

SC-002 removed the independent scene/renderer ancestry interpreters, and SC-008 moved runtime-presented geometry to the submitted snapshot. Remaining paths are topology-owned projection/emission or representation adapters consuming compiled bindings:

| Interpreter | File | Production consumer | Required migration/deletion gate |
|---|---|---|---|
| Semantic/resident projection adapter | `src/scene/spatial.rs::SpatialTopology::project_semantic_order` | semantic/resident commit construction | Topology-owned structural projection; every resulting commit recompiles its projected topology through `Commit::from_parts`. |
| Compatibility emission adapter | `src/scene/spatial.rs::SpatialTopology::emit_compatibility_until` | runtime frame realization and popup compatibility scenes | Structural grouping/clip emission; all movement comes from normalized world bindings. The old commit-local interpreter is deleted. |
| Shared resumable retained compiler | `src/render/retained.rs::PendingPlan::advance` | both incremental and direct preparation | Sole plan compiler. Direct preparation executes this compiler with an unbounded slice; `PlanBuilder::build_order` is deleted. |
| Representation encoder | `src/render/renderer.rs::PlanEncoder` | direct and sampled surface encoding | Consumes compiled bindings for groups, panes, clips, scroll viewports, shapes, and text. Mutable `scroll_translation` ancestry is deleted. |
| Presented spatial evaluator | `src/scene/spatial.rs::SpatialSnapshot`, installed by `src/runtime/access.rs` | hit testing, pointer, drag, context routing, scroll target/axis lookup, caret/IME projection, and native-popup surface input | SC-008 closed. The snapshot captures Arc-backed submitted layers, evaluates precompiled frame paths/caret states/target bindings lazily, and performs no scene-node scan on a warm submission. |

Known payload adapters that must consume the compiled spatial result without independently deriving ancestry:

- retained shape node/scroll properties in `src/render/retained.rs` and `src/render/retained_quad.wgsl`;
- retained text transforms and glyph viewport offsets in `src/render/retained.rs` and `src/render/text_renderer.rs`;
- group, pane, filter, and offscreen surface translation in `src/render/renderer.rs` and `src/render/filter/`;
- fixed/moving clips represented by scene draw order and renderer clip frames;
- scrollbar/chrome projections in `src/layout/chrome.rs`, `src/runtime/visual.rs`, and `src/scene/visual.rs`.

## 3. Production file census by stage

### Input and host adaptation

- `src/platform/event.rs`
- `src/host/event.rs`
- `src/input/mod.rs`
- `src/shell/event.rs`
- `src/shell/input.rs`
- `src/platform/native/surface.rs`

### Interaction and session intent

- `src/interaction/scroll.rs`
- `src/interaction/mod.rs`
- `src/interaction/target.rs`
- `src/session/interaction/scroll.rs`
- `src/session/window.rs`

### Runtime transition, routing, and presentation

- `src/runtime/input/dispatch.rs`
- `src/runtime/input/effect.rs`
- `src/runtime/presentation.rs`
- `src/runtime/access.rs`
- `src/runtime/mod.rs`
- `src/runtime/pointer.rs`
- `src/runtime/routing.rs`
- `src/runtime/context_menu.rs`
- `src/runtime/visual.rs`
- `src/ime.rs`
- `src/shell/presentation.rs`
- `src/shell/work.rs`

### Layout, viewport, and residency

- `src/layout/mod.rs`
- `src/layout/algorithm.rs`
- `src/layout/viewport.rs`
- `src/layout/frame.rs`
- `src/layout/chrome.rs`
- `src/layout/text.rs`
- `src/view/node/builder.rs`
- `src/scene/residency.rs`

### Scene commit, properties, and stack

- `src/scene/commit.rs`
- `src/scene/spatial.rs`
- `src/scene/paint/mod.rs`
- `src/scene/stack.rs`
- `src/scene/visual.rs`
- `src/scene/mod.rs`

### Retained realization and GPU representation

- `src/render/retained.rs`
- `src/render/renderer.rs`
- `src/render/text_renderer.rs`
- `src/render/scene.rs`
- `src/render/report.rs`
- `src/render/surface.rs`
- `src/render/retained_quad.wgsl`
- `src/render/filter/`

### Native scheduling

- `src/platform/runner/mod.rs`
- `src/platform/runner/handler.rs`
- `src/platform/runner/native.rs`
- `src/platform/backend.rs`
- `src/platform/native/adapter.rs`
- `src/platform/native/window.rs`
- `src/platform/native/popup.rs`
- `src/platform/native/surface.rs`

### Diagnostics and executable witnesses

- `src/diagnostics/scroll.rs`
- `src/diagnostics/render.rs`
- `src/diagnostics/pipeline.rs`
- `src/render/debug.rs`
- `tools/renderer_debug/src/main.rs`
- `tools/check_renderer_receipts.py`
- `tools/generate_scroll_pairwise_manifest.py`
- `docs/audits/fixtures/scroll-pairwise-manifest-v1.json`

SC-001 adds these test-only spatial witnesses without changing production ownership:

- `src/scene/commit.rs::renderer_scroll_oracle_fixture` owns the eight independently authored actual/static-expected Tier A fixture pairs and discriminating regions;
- `src/render/debug.rs::Harness::compare_scroll_oracle_case` owns per-object, fixed-region, full-image, and direct/incremental-plan validation;
- `src/render/retained.rs::Plan::debug_signature` and the renderer debug adapter expose complete recursive retained-plan structure for equality only;
- `tools/renderer_debug` exposes `tier-a-scroll-oracle` and `tier-a-negative-controls` as executable GPU witnesses.

These are oracle/observation paths. They are not an alternate spatial source of truth and must not become production consumers.

SC-002 production ownership changes:

- `Commit::from_parts` compiles one typed `SpatialTopology` containing root, transform, scroll, and surface-root nodes plus clip/effect references and content property states.
- Production scroll declarations carry their `interaction::Target`; test fixtures carry an explicit scene target. Per-axis conflicts are rejected while split-axis shared targets are accepted.
- Repeated sibling scopes intern the same logical scroll node and surface coordinate space.
- `PropertyBinding`, retained text batches, group/pane/clip/viewport plan steps, and GPU scroll uniforms consume `SpatialBinding` values.
- Stable declared surface bounds replaced baseline-content-union inference; the latter clipped scrolling payload after the first property tick and is deleted.
- Scene painting now omits an own-scroll fragment when that projection lacks drawable residency, so a dangling `PushScroll` can no longer rely on renderer no-op behavior.

SC-003 property ownership changes:

- `Commit::from_parts` assigns stable topology-local `PropertyIndex` values and owns O(1) property/node maps.
- `Properties` stores canonical values in shared 256-entry immutable blocks; `apply_updates` coalesces index writes and path-copies only touched blocks.
- `interaction::Scroll` owns per-target source revisions. Runtime `Visuals` owns prior scrollbar/caret property baselines and dirty target sets. Scene painting consumes those sources and commits its source ledger only after constructing a valid property snapshot.
- Retained plans precompute dirty property dependents for node/chrome bindings and compiled scroll paths. `Properties::changed()` is a production input rather than a diagnostic-only list.
- `SpatialTopology` interns scroll-only paths independently of transform topology. Transform-only bindings share the root path; real nested scroll contributions retain explicit owner/baseline chains.
- Node and scroll property buffers share `plan_property_transfer`, including one sparse range model and named initialization, buffer replacement, topology/viewport replacement, and dense reasons.
- `src/diagnostics/render.rs`, renderer reports, and `renderer_debug` expose value visits, index lookups, dirty indices, write ranges, and every full-transfer reason.

SC-004 generation/state ownership changes:

- `interaction::Scroll` exposes `resident_offset` and `accept_resident`; pending state carries `resident_accepted` and `desired`. The old scroll admission API is absent from production.
- `session::Window` contains one private `PresentationState { requested, present_submitted }`. Request mutation and successful feedback are its only writers; failed retries preserve the requested epoch.
- `RenderReport`, native surface handling, diagnostics, runner pulse names, receipt keys, receipt validation, and the text-editor debug panel use `present_submitted`. The success path is downstream of queue submission and `frame.present()` and makes no scanout claim.
- `scene::Properties` records an optional direct predecessor serial. Retained node and scroll property slots record their installed serial and mutate sparsely only from that predecessor while exclusively owned by the same commit.
- A skipped/mismatched candidate selects `PropertyFullReason::GenerationResync`; initialization, buffer replacement, topology/viewport replacement, and density retain their independent reasons.
- The exact eight-case state suite is source-counted across deterministic runtime/layout cases and one explicit release GPU scale-change case. A separate release GPU witness requires skipped-generation resynchronization.

SC-005 input-precision ownership changes:

- `platform::scroll_delta` converts pixel and fractional-line input to finite logical `f64` components without per-event integer rounding. Host, shell, input routing, and runtime dispatch carry that value unchanged.
- `layout::Viewport::can_consume_from` reads precise signs, so subpixel input selects the correct target before visual quantization.
- Each `interaction::ScrollEntry` owns one private compensated `ScrollRemainder`. Remainders cannot leak between targets, windows, or popup payloads.
- Exact integral delta components apply directly. Fractional components accumulate with truncation toward zero at whole logical pixels and an eight-ULP normalization at computed integral boundaries.
- Absolute thumb/programmatic and geometry/reveal requests reset remainder; fraction-only updates persist without advancing source revision or scheduling a candidate.
- `ScrollOffset`, scene properties, spatial topology, renderer bindings, chrome, and present-submitted geometry remain integral. The exact 20-case suite and source architecture gate enforce this boundary.

SC-006 presentation-cadence ownership changes:

- `Platform::request_backend_redraw` is the only framework call site for `backend.request_redraw`. `RedrawRequests` deduplicates one in-flight request per window; delivery/failure/close clears it.
- Native runner `PresentationPulse`, `frame_demands`, `issued_frame_redraws`, due-frame execution, and pulse-derived control-flow deadlines are deleted. The runner retains only application animation/task polling policy.
- Scroll trace schema v2 records input-relative redraw request/delivery, candidate construction, acquire start, queue submit, surface present call, and present-submitted receipt plus acquire duration.
- `render::surface` timestamps acquire start/finish, queue submission, and the completed `frame.present()` call. Runtime `present_submitted_at` uses that present-call timestamp rather than later post-present work.
- Renderer diagnostics and external receipt validation expose redraw requests issued, redraw deliveries, and no-progress redraws in addition to event latency, frame intervals, missed opportunities, CPU stages, acquire, encode/submit/present, and skipped frames.
- The exact 12-case suite covers steady/burst/delayed demand at 60/90/120/144 Hz. A retained test model proves the deleted completion anchor rejects demand immediately after a present.

SC-007 residency ownership changes:

- `Layout::residency_demand` returns one payload-neutral target/offset demand with optional materialization adapters; text, table, and virtual-list crossings share the same transition boundary.
- Scroll trace schema v3 attributes candidate and renderer work to the selected candidate/property/present-submitted generation and leaves resident property ticks without cold-work fields.
- Text cold admission streams bounded source/glyph bands. Exact global no-wrap extent discovery remains separately named `ColdStart`; it does not admit whole-document glyph storage.

SC-008 present-submitted geometry ownership changes:

- `SpatialTopology` compiles frame scroll paths, caret content states, and typed interaction-target axis bindings once with the candidate commit. Fixed frame/hit space and moving content/caret space are distinct named projections of the same topology.
- `SpatialSnapshot::from_stack` captures only Arc-backed drawable/property layers and native-popup spatial supplements. It does not traverse `commit.nodes()` or eagerly rebuild every translated node on a warm property submission.
- `PresentedGeometry` delegates point, rectangle, clip, and target-offset queries to that lazy snapshot. Runtime production contains no `scroll_ancestry` read.
- Native popup layers enter the stack only as geometry-only `SpatialSupplement` records; they never become parent draw layers. Popup-surface hit testing therefore uses the same candidate topology while preserving independent native presentation.
- IME carries an authored `ime::Projection` with the actual backend presentation. Only a matching successful present-submitted epoch/property serial resolves it through the installed snapshot. Failed, deferred, stale, and superseded candidates cannot expose caret geometry.
- The end-to-end text witness proves a 20-pixel property scroll moves submitted parent IME geometry by exactly 20 pixels. Popup host preparation may precede parent submission, but popup IME activation waits for that exact parent receipt.
- No AccessKit/accessibility tree or platform adapter exists in production. SC-008 therefore records the absent consumer and preserves the snapshot seam; it does not claim accessibility bounds were emitted or tested.

SC-009 payload-locality ownership changes:

- Scroll-bench version 10 separates guarded edit, resident horizontal projection, caret reveal, exact global extent cold start, and renderer property-tick currencies. Text layout receipts explicitly do not claim property-scroll measurement.
- Incremental horizontal-index diagnostics record aggregate and per-update maximum source/glyph work. The executable benchmark rejects any edit splice over the 4,096-byte guard or any warm full-width/index scan.
- Table and virtual-list payload edits preserve normalized spatial/property topology, scroll owner/range revisions, and scroll values while changing one retained composition identity.

## 4. SC-010 final disposition

The repeated production census classifies every remaining scroll/spatial hit as one of the following, with no unclassified authority:

1. **Pre-candidate construction:** `Layout::scene_scroll_paths` is generated once from immutable frame parentage. It computes residency bounds and emits ordered scene scroll scopes. The former `scroll_ancestries` name and API are deleted, and no runtime/renderer module can read this construction map.
2. **Candidate authority:** `Commit::spatial_topology` is the only normalized parented spatial topology. Semantic/resident projection and compatibility output are topology-owned generated adapters.
3. **GPU adapters:** retained `TargetSpace` contains raster origin/extent plus one `SpatialBinding`; it contains no scroll ancestry. Retained `ScrollBinding` contains only a topology-owned `ScrollPathId`. `PropertyBinding` obtains that path through `SpatialTopology::scroll_path`, and `PlanBuilder` consumes the candidate topology. These names describe GPU allocation/binding, not a second spatial graph.
4. **Presentation adapter:** `SpatialSnapshot` captures the actual submitted stack and lazily evaluates candidate-compiled paths. Runtime input, popup, context, scrollbar, caret, and IME geometry have no layout ancestry fallback.
5. **Independent mechanisms:** precise input remainder, requested/resident/present-submitted state, residency, redraw scheduling, and sparse/dense property transfer retain their named sole owners and do not derive spatial ancestry.

Deleted/superseded production species remain absent: commit-local semantic/compatibility interpreters, direct plan order builder, mutable group-local scroll ancestry, renderer encoder scroll-offset reconstruction, runtime layout-ancestry projection, completion-anchored cadence, per-event input rounding, ambiguous admitted/visible generation names, and unconditional full property transfer on sparse warm ticks. Initialization, buffer/topology replacement, generation recovery, and cost-selected dense transfer remain explicit full-transfer paths by design.

The architecture closure gate parses the bounded retained adapter structs and fails if raster target space gains scroll state, GPU scroll binding gains parentage, construction paths escape layout/scene paint, or a retired interpreter/name returns. The repeatable commands below remain the audit entry point for future changes.

## 5. Property work census

Current warm property uploads are emitted by:

| Category | Current write site | Diagnostic field |
|---|---|---|
| Retained viewport uniform | `src/render/retained.rs::prepare_node_properties` | `viewport_property_upload_bytes` |
| Retained node properties, including current chrome/effect entries | `src/render/retained.rs::prepare_node_properties` | `node_property_upload_bytes` |
| Retained scroll properties | `src/render/retained.rs::prepare_scroll_properties` | `scroll_property_upload_bytes` |
| Retained glyph viewport/transform offset | `src/render/text_renderer.rs::prepare_retained_transforms` | `text_property_upload_bytes` |
| Any unclassified producer | aggregate minus named categories | `unattributed_property_upload_bytes` |

The 2026-07-16 release `table-scroll-work` receipt is identical at scales 1.0, 1.25, 1.5, 1.75, and 2.0:

```text
property_upload_bytes=11072
viewport_property_upload_bytes=0
node_property_upload_bytes=11008
scroll_property_upload_bytes=32
text_property_upload_bytes=32
unattributed_property_upload_bytes=0
```

This attributes the baseline. It does not close SC-003 or prove that 11,008 node bytes cause cadence defects.

The SC-002 release boundary produces a new, separately recorded five-scale receipt after explicit topology and dangling-scope rejection:

```text
property_upload_bytes=10656
viewport_property_upload_bytes=0
node_property_upload_bytes=10496
scroll_property_upload_bytes=32
text_property_upload_bytes=128
unattributed_property_upload_bytes=0
```

All five scales are identical, with zero semantic/content preparation, zero text shaping, zero GPU resource churn, and one plan reuse. The total is 416 bytes below the SC-000 baseline; text-transform bytes increased by 96 while node bytes decreased by 512. This is an SC-003 input, not a sparse-update budget or a claim that the category shift is optimal.

The SC-003 indexed-delta boundary produces this separately recorded five-scale receipt:

```text
property_upload_bytes=464
viewport_property_upload_bytes=0
node_property_upload_bytes=64
scroll_property_upload_bytes=272
text_property_upload_bytes=128
unattributed_property_upload_bytes=0
property_value_visits=7
property_index_lookups=7
property_dirty_indices=1
property_write_ranges=2
property_full_initializations=0
property_full_buffer_replacements=0
property_full_topology_replacements=0
property_full_dense_transfers=0
property_full_generation_resyncs=0
```

The 272 scroll bytes are one cost-selected contiguous transfer spanning two scroll-path slots that depend on the dirty shared target; they are not a topology-wide scroll upload. All five scales retain zero semantic/content preparation, shaping, payload upload, GPU resource churn, and plan rebuilds with one plan reuse. The retained text adapter traverses resident text batches and writes only changed snapped offsets; its payload-local traversal remains under the SC-007/SC-009 residency and locality rails rather than becoming another property-topology owner.

The final SC-010 production fixture contains one additional retained text transform and produces this identical receipt at scales 1.0, 1.25, 1.5, 1.75, and 2.0:

```text
property_upload_bytes=528
viewport_property_upload_bytes=0
node_property_upload_bytes=64
scroll_property_upload_bytes=272
text_property_upload_bytes=192
unattributed_property_upload_bytes=0
property_value_visits=17
property_index_lookups=17
property_dirty_indices=1
property_write_ranges=2
property_full_initializations=0
property_full_buffer_replacements=0
property_full_topology_replacements=0
property_full_dense_transfers=0
property_full_generation_resyncs=0
```

This supersedes the 464-byte fixture count without weakening the invariant: one dirty source drives bounded compiled dependents, and warm scrolling performs zero semantic/content preparation, shaping, payload upload, GPU resource churn, or plan rebuild while reusing one plan. A separate nested-scroll GPU oracle uses a real predecessor update and records one dirty source, three visits/lookups, one 16-byte sparse scroll write, and no full-transfer reason.

## 6. Repeatable census commands

Run from the repository root and inspect additions/removals rather than comparing counts alone:

```powershell
rg -l 'ScrollOffset|ScrollDelta|ScrollUpdate' src tools
rg -l 'from_physical_pixels|from_logical_pixels|ScrollRemainder|quantize_scroll_axis|split_scroll_component' src tools
rg -l 'scroll_ancestr|scene_scroll_path|scroll_projection|ScrollProjection' src tools
rg -l 'PushScroll|PopScroll|ScrollDeclaration' src tools
rg -l 'TargetSpace|ScrollBinding|PropertyBinding' src tools
rg -l 'project_semantic_order|emit_compatibility_until|PendingPlan|PlanBuilder|PlanEncoder' src tools
rg -l 'SpatialTopology|SpatialBinding|SurfaceRoot|AxisOwnership|ScrollTarget' src tools
rg -l 'SpatialSnapshot|frame_scroll_paths|caret_states|target_bindings|spatial_supplement' src tools
rg -l 'project_point|presented_geometry|presented_ime_update|record_present_submitted|present_submitted_epoch' src tools
rg -l 'RedrawRequests|request_backend_redraw|request_redraw|RedrawRequested|redraw_no_progress' src tools
rg -l 'redraw_requested_at|redraw_delivered_at|candidate_constructed_at|acquire_started_at|queue_submitted_at|surface_present_called_at' src tools
rg -l 'property_upload_bytes|prepare_node_properties|prepare_scroll_properties' src tools
rg -l 'PropertyIndex|apply_updates|property_dependents|scroll_dependents|plan_property_transfer' src tools
rg -l 'predecessor_serial|property_serial|property_slot_exclusively_owned_by' src tools
rg -l 'property_full_initializations|property_full_buffer_replacements|property_full_topology_replacements|property_full_dense_transfers|property_full_generation_resyncs' src tools
rg -l 'admit_scroll|desired_presentation_epoch|acknowledged_presentation_epoch|frames_presented|key_to_present_us' src tools
```

SC-010 closure includes both this classification and the executable evidence recorded in the campaign. Finding no new names remains insufficient for future changes: every remaining production hit must stay the named sole owner, a generated/representation-specific consumer, or an independently justified non-spatial track.
