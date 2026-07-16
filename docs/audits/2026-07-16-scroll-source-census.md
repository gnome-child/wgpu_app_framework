# Scroll source census

Status: **SC-000 BASELINE — update at every ownership/deletion boundary**

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
| Candidate spatial/scope order | `src/scene/commit.rs::Commit` draw order | semantic, compatibility, retained, encoder paths | Candidate commit becomes sole normalized topology owner in SC-002. |
| Runtime-presented geometry | `src/runtime/access.rs::finish_render_report_with_kind` | `src/runtime/mod.rs::PresentedGeometry`, pointer/routing/context menu | SC-004 names the boundary `present_submitted`; SC-008 consumes normalized topology. |
| Surface submission/present call | `src/render/surface.rs` | runtime report, diagnostics, runner pulse | No scanout claim. SC-006 audits cadence separately. |
| Redraw throttling | `src/platform/runner/mod.rs::PresentationPulse` | `handler.rs`, `native.rs` | Independent SC-006 evidence track. |

## 2. Scope/spatial interpreter census

These paths currently interpret scroll, clip, group, or coordinate-boundary semantics independently:

| Interpreter | File | Production consumer | Required migration/deletion gate |
|---|---|---|---|
| Residency scope rewrite | `src/scene/commit.rs::semantic_order` | semantic/resident commit construction | Generate a projected topology/view from the candidate topology; no independent push/pop semantics. |
| Compatibility recursive order | `src/scene/commit.rs::compatibility_order_until` | runtime frame realization and popup compatibility scenes | Generate compatibility output from normalized bindings or migrate consumers. |
| Bounded retained planner | `src/render/retained.rs::PendingPlan::advance` | incremental preparation | Resume one shared spatial compiler rather than own semantics. |
| Direct retained planner | `src/render/retained.rs::PlanBuilder::build_order` | direct/fallback plan construction | Use the same compiler/output as the bounded path. |
| Runtime encoder state | `src/render/renderer.rs::PlanEncoder` | direct and sampled surface encoding | Consume compiled bindings; retain representation-specific encoding only. |
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

## 5. Repeatable census commands

Run from the repository root and inspect additions/removals rather than comparing counts alone:

```powershell
rg -l 'ScrollOffset|ScrollDelta|ScrollUpdate' src tools
rg -l 'scroll_ancestr|scroll_projection|ScrollProjection' src tools
rg -l 'PushScroll|PopScroll|ScrollDeclaration' src tools
rg -l 'TargetSpace|ScrollBinding|PropertyBinding' src tools
rg -l 'semantic_order|compatibility_order_until|PendingPlan|PlanBuilder|PlanEncoder' src tools
rg -l 'project_point|presented_geometry|acknowledge_presentation' src tools
rg -l 'PresentationPulse|request_redraw|RedrawRequested' src tools
rg -l 'property_upload_bytes|prepare_node_properties|prepare_scroll_properties' src tools
```

SC-010 is not closed by finding no new names. It must prove that every remaining production hit is either the named sole owner, a generated/representation-specific consumer, or an independently justified non-spatial track.
