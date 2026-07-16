# Scroll source census

Status: **SC-006 PRESENTATION-CADENCE OWNERSHIP BOUNDARY — update at every ownership/deletion boundary**

Date: 2026-07-16

Execution base: `master` after integration commit `cd00554d`

Authority: `docs/audits/2026-07-16-payload-neutral-scroll-architecture-campaign.md`

This is a production-path census, not a claim that every listed path is wrong. It records every known owner, projection, interpreter, representation adapter, scheduling boundary, and diagnostic consumer that can affect scrolling. SC-002 and SC-010 must update this file rather than relying on memory or a narrower symbol search.

## 1. Fact ownership baseline

| Fact | Current owner | Derived readers/writers | Campaign disposition |
|---|---|---|---|
| Raw precise wheel/trackpad delta | `src/platform/event.rs::scroll_delta` | `src/host/event.rs`, `src/shell/event.rs`, `src/shell/input.rs` | SC-005 closed lossless pixel/fractional-line transport through target routing; the adapter does not quantize visual motion. |
| Target-local fractional remainder plus requested/desired and resident-accepted offset | `src/interaction/scroll.rs::Scroll` | `src/interaction/mod.rs`, `src/session/interaction/scroll.rs` | SC-004 closed state names; SC-005 made interaction the sole fractional accumulator and kept desired/resident offsets integral. |
| Request, clamp, resident acceptance | `src/runtime/input/dispatch.rs::apply_scroll_transition` | presented layout and virtual request lookup | SC-004 closed the transition vocabulary; desired intent survives residency rejection. |
| Legal range and resident acceptance | `src/layout/mod.rs::ScrollProjection` and `Layout` methods | `src/layout/viewport.rs`, `src/layout/frame.rs`, `src/scene/residency.rs` | Layout/residency remains range owner; axis ownership becomes typed in SC-002/SC-004. |
| Node-to-scroll ancestry | `src/layout/mod.rs::scroll_ancestries` | scene paint ordering and runtime point projection | Superseded by normalized topology in SC-002/SC-008, then deleted in SC-010 if no non-spatial consumer remains. |
| Candidate property values and dirty indices | `src/scene/paint/mod.rs::property_snapshot` and `src/scene/commit.rs::Properties` | scene stack, renderer, compatibility scene | SC-003 closed stable topology-local indices, block-shared values, and authoritative dirty production independently of spatial replacement. |
| Candidate spatial ancestry, surface roots, clip/effect references, target identity, and axis ownership | `src/scene/spatial.rs::SpatialTopology`, immutably owned by `src/scene/commit.rs::Commit` | semantic/drawable projection, compatibility emission, retained plan compiler, renderer property adapters | SC-002 closed this ownership replacement. Draw order remains structural input/output, not an ancestry owner. |
| Requested and present-submitted presentation epochs | private `src/session/window.rs::PresentationState` | runtime candidate selection and successful render feedback | SC-004 closed one owner. Requests advance requested state; retry does not; stale, duplicate, and future receipts cannot advance present-submitted state. |
| Installed GPU property generation | retained node/scroll `PropertySlot::property_serial` derived from `scene::Properties::{serial, predecessor}` | sparse property preparation and explicit full resynchronization | SC-004 closed skipped-generation recovery. Sparse mutation requires the direct predecessor and exclusive commit ownership. |
| Runtime-presented geometry | `src/runtime/access.rs::finish_render_report_with_kind` | `src/runtime/mod.rs::PresentedGeometry`, pointer/routing/context menu | SC-004 names the boundary `present_submitted`; SC-008 consumes normalized topology. |
| Surface acquire, queue submit, and present call | `src/render/surface.rs` | runtime report and scroll/render diagnostics | SC-006 records actual call boundaries. `present_submitted` remains a submit/present-call fact with no scanout claim. |
| Redraw demand and in-flight deduplication | `src/platform/mod.rs::RedrawRequests` | all backend request, delivery, retry, continuation, and close paths | SC-006 closed one immediate issuance owner. Native runner completion clocks and duplicate demand sets are deleted. |

## 2. Scope/spatial interpreter census

SC-002 removed the independent scene/renderer ancestry interpreters. Remaining paths are either topology-owned projection/emission, representation adapters consuming compiled bindings, or the still-open runtime-presented consumer:

| Interpreter | File | Production consumer | Required migration/deletion gate |
|---|---|---|---|
| Semantic/resident projection adapter | `src/scene/spatial.rs::SpatialTopology::project_semantic_order` | semantic/resident commit construction | Topology-owned structural projection; every resulting commit recompiles its projected topology through `Commit::from_parts`. |
| Compatibility emission adapter | `src/scene/spatial.rs::SpatialTopology::emit_compatibility_until` | runtime frame realization and popup compatibility scenes | Structural grouping/clip emission; all movement comes from normalized world bindings. The old commit-local interpreter is deleted. |
| Shared resumable retained compiler | `src/render/retained.rs::PendingPlan::advance` | both incremental and direct preparation | Sole plan compiler. Direct preparation executes this compiler with an unbounded slice; `PlanBuilder::build_order` is deleted. |
| Representation encoder | `src/render/renderer.rs::PlanEncoder` | direct and sampled surface encoding | Consumes compiled bindings for groups, panes, clips, scroll viewports, shapes, and text. Mutable `scroll_translation` ancestry is deleted. |
| Presented point projection | `src/runtime/mod.rs::PresentedGeometry::project_point` | hit testing, pointer, drag, context routing | Evaluate the last present-submitted normalized topology in SC-008. |

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

## 4. Property work census

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

## 5. Repeatable census commands

Run from the repository root and inspect additions/removals rather than comparing counts alone:

```powershell
rg -l 'ScrollOffset|ScrollDelta|ScrollUpdate' src tools
rg -l 'from_physical_pixels|from_logical_pixels|ScrollRemainder|quantize_scroll_axis|split_scroll_component' src tools
rg -l 'scroll_ancestr|scroll_projection|ScrollProjection' src tools
rg -l 'PushScroll|PopScroll|ScrollDeclaration' src tools
rg -l 'TargetSpace|ScrollBinding|PropertyBinding' src tools
rg -l 'project_semantic_order|emit_compatibility_until|PendingPlan|PlanBuilder|PlanEncoder' src tools
rg -l 'SpatialTopology|SpatialBinding|SurfaceRoot|AxisOwnership|ScrollTarget' src tools
rg -l 'project_point|presented_geometry|record_present_submitted|present_submitted_epoch' src tools
rg -l 'RedrawRequests|request_backend_redraw|request_redraw|RedrawRequested|redraw_no_progress' src tools
rg -l 'redraw_requested_at|redraw_delivered_at|candidate_constructed_at|acquire_started_at|queue_submitted_at|surface_present_called_at' src tools
rg -l 'property_upload_bytes|prepare_node_properties|prepare_scroll_properties' src tools
rg -l 'PropertyIndex|apply_updates|property_dependents|scroll_dependents|plan_property_transfer' src tools
rg -l 'predecessor_serial|property_serial|property_slot_exclusively_owned_by' src tools
rg -l 'property_full_initializations|property_full_buffer_replacements|property_full_topology_replacements|property_full_dense_transfers|property_full_generation_resyncs' src tools
rg -l 'admit_scroll|desired_presentation_epoch|acknowledged_presentation_epoch|frames_presented|key_to_present_us' src tools
```

SC-010 is not closed by finding no new names. It must prove that every remaining production hit is either the named sole owner, a generated/representation-specific consumer, or an independently justified non-spatial track.
