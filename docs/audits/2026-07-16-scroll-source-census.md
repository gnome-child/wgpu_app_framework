# Scroll source census

Status: **SC-002 SPATIAL OWNERSHIP BOUNDARY — update at every ownership/deletion boundary**

Date: 2026-07-16

Execution base: `master` after integration commit `cd00554d`

Authority: `docs/audits/2026-07-16-payload-neutral-scroll-architecture-campaign.md`

This is a production-path census, not a claim that every listed path is wrong. It records every known owner, projection, interpreter, representation adapter, scheduling boundary, and diagnostic consumer that can affect scrolling. SC-002 and SC-010 must update this file rather than relying on memory or a narrower symbol search.

## 1. Fact ownership baseline

| Fact | Current owner | Derived readers/writers | Campaign disposition |
|---|---|---|---|
| Raw wheel delta | `src/platform/event.rs::scroll_delta` | `src/host/event.rs`, `src/shell/event.rs`, `src/shell/input.rs` | SC-005 preserves fractional remainder before integral visual quantization. |
| Requested/desired and resident-accepted offset | `src/interaction/scroll.rs::Scroll` | `src/interaction/mod.rs`, `src/session/interaction/scroll.rs` | SC-004 separates names/generations; interaction remains intent owner. |
| Request, clamp, resident acceptance | `src/runtime/input/dispatch.rs::apply_scroll_transition` | presented layout and virtual request lookup | SC-000 traces the stages; SC-004 defines their transition contract. |
| Legal range and resident acceptance | `src/layout/mod.rs::ScrollProjection` and `Layout` methods | `src/layout/viewport.rs`, `src/layout/frame.rs`, `src/scene/residency.rs` | Layout/residency remains range owner; axis ownership becomes typed in SC-002/SC-004. |
| Node-to-scroll ancestry | `src/layout/mod.rs::scroll_ancestries` | scene paint ordering and runtime point projection | Superseded by normalized topology in SC-002/SC-008, then deleted in SC-010 if no non-spatial consumer remains. |
| Candidate property values | `src/scene/paint/mod.rs::property_snapshot` and `src/scene/commit.rs::Properties` | scene stack, renderer, compatibility scene | SC-003 adds indexed dirty production independently of spatial replacement. |
| Candidate spatial ancestry, surface roots, clip/effect references, target identity, and axis ownership | `src/scene/spatial.rs::SpatialTopology`, immutably owned by `src/scene/commit.rs::Commit` | semantic/drawable projection, compatibility emission, retained plan compiler, renderer property adapters | SC-002 closed this ownership replacement. Draw order remains structural input/output, not an ancestry owner. |
| Runtime-presented geometry | `src/runtime/access.rs::finish_render_report_with_kind` | `src/runtime/mod.rs::PresentedGeometry`, pointer/routing/context menu | SC-004 names the boundary `present_submitted`; SC-008 consumes normalized topology. |
| Surface submission/present call | `src/render/surface.rs` | runtime report, diagnostics, runner pulse | No scanout claim. SC-006 audits cadence separately. |
| Redraw throttling | `src/platform/runner/mod.rs::PresentationPulse` | `handler.rs`, `native.rs` | Independent SC-006 evidence track. |

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

## 4. Property work census

Current warm property uploads are emitted by:

| Category | Current write site | SC-000 diagnostic field |
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

## 5. Repeatable census commands

Run from the repository root and inspect additions/removals rather than comparing counts alone:

```powershell
rg -l 'ScrollOffset|ScrollDelta|ScrollUpdate' src tools
rg -l 'scroll_ancestr|scroll_projection|ScrollProjection' src tools
rg -l 'PushScroll|PopScroll|ScrollDeclaration' src tools
rg -l 'TargetSpace|ScrollBinding|PropertyBinding' src tools
rg -l 'project_semantic_order|emit_compatibility_until|PendingPlan|PlanBuilder|PlanEncoder' src tools
rg -l 'SpatialTopology|SpatialBinding|SurfaceRoot|AxisOwnership|ScrollTarget' src tools
rg -l 'project_point|presented_geometry|acknowledge_presentation' src tools
rg -l 'PresentationPulse|request_redraw|RedrawRequested' src tools
rg -l 'property_upload_bytes|prepare_node_properties|prepare_scroll_properties' src tools
```

SC-010 is not closed by finding no new names. It must prove that every remaining production hit is either the named sole owner, a generated/representation-specific consumer, or an independently justified non-spatial track.
