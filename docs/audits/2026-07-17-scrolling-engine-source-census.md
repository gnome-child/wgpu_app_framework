# Scrolling engine source census

Status: **SE-000 FROZEN BASELINE**

Date: 2026-07-17

Execution base: `master` at `1c480c82cf750df445a00b755f7a47ed13af60f4`

Authority: `docs/audits/2026-07-17-scrolling-engine-campaign.md`

This census assigns every current scrolling fact to one of the six target
layers. It does not bless the current placement. The exhaustive predecessor
census remains valid for renderer topology, submitted geometry, property
generation, redraw demand, and surface clocks; this file recasts those facts
around the scrolling-engine rewrite.

## 1. Current fact ownership and destination

| Current fact/state | Current owner | Current consumers | Target layer/disposition |
|---|---|---|---|
| Winit wheel delta | `src/platform/event.rs::scroll_delta` | host/shell input adaptation | Scroll container input adapter. Pixel and fractional-line values survive, but `TouchPhase` is currently discarded. |
| Pointer location used for wheel targeting | `src/platform/event.rs::WindowEvents` | host event and runtime routing | Scroll container session entrance; pointer state itself remains platform input state. |
| Target-local fractional x/y carry and compensation | `src/interaction/scroll.rs::ScrollRemainder` | `Scroll::request` | Axis adjustment, one independent continuous coordinate per axis. Current integral-boundary quantization must go. |
| Desired and resident-accepted x/y offsets | `src/interaction/scroll.rs::Position` | session, runtime transition, layout projection | Canonical value moves to axis adjustment; desired/resident remain private residency receipts that project it. |
| Public `ScrollOffset` (`i32` x/y) and `ScrollDelta` (`f64` x/y) | `src/interaction/scroll.rs` | all input, routing, layout, scene, render, tests | Replace internal global coordinate with wide per-axis adjustment values. Keep GPU/local conversion bounded. Public names are provisional. |
| Per-target scroll revision | `src/interaction/scroll.rs::Scroll::revisions` | presentation/layout invalidation | Axis adjustment observation revision. |
| Viewport and active-descendant reveal requests | `src/interaction/scroll.rs::reveal_requests` | session, layout text/reveal | Scroll container operation; native text may author target geometry but not own ancestor traversal. |
| Programmatic relative/absolute/geometry requests | `src/input/mod.rs`, `src/runtime/input/dispatch.rs` | session interaction and layout | Scroll container source-neutral operation using the same session/outcome path as direct input. |
| Viewport rect/content/page geometry, max range, clamped resolution | `src/layout/viewport.rs::Viewport` | layout routing, chrome, reveal, scene | Range/page values project into two adjustments; eager adapter/container owns generic geometry. Native views supply domain extents. |
| Desired/preparation/range/runway projection | `src/layout/mod.rs::ScrollProjection` | runtime presentation, scene residency | Split: adjustment owns canonical range/value; residency/presentation privately owns coverage/preparation/admission. |
| One-target “can consume any axis” decision | `src/layout/viewport.rs::can_consume_from`, `src/layout/mod.rs::scroll_target_at` | runtime pointer scrolling | Scroll container nested handoff. Replace with child-first applied/remainder result independently per axis. |
| Generic frame viewport layout | `src/layout/frame.rs`, `src/layout/mod.rs` | eager panels plus native text/table/list frames | Viewport adapter for arbitrary eager content; native views bypass it for domain realization. |
| Scrollbar track/thumb geometry and drag-to-offset mapping | `src/layout/chrome.rs` | runtime routing and scene chrome | Scroll container chrome projected from the matching axis adjustment. |
| Scrollbar opacity, hover/press thickness, fade deadline | `src/runtime/visual.rs`, `src/scene/visual.rs` | scene chrome painting | Scroll container presentation/chrome; overlay/layout consumption and axis behavior become distinct policies. |
| Theme `OverlayAuto` / `GutterAlways` | `src/theme/mod.rs`, `src/theme/toml.rs` | layout chrome, runtime visuals, scene paint | Appearance default only after SE-004. Axis behavior becomes Always/Automatic/Never/External and overlay/consuming is separate. |
| Text reveal/caret correction and height/width indexes | `src/text/layout/**`, `src/runtime/input/text/**` | text area layout, IME, selection | Native text owns domain layout and anchor geometry; container owns scrolling and ancestor reveal. |
| Virtual-list provider identity/count/query | `src/virtual_list.rs::Provider` | virtual list model/request construction | List model. Current query-only contract lacks observable mutation, content revision, uniqueness enforcement, and slot lifecycle. |
| Virtual-list measurements and variable-height region | `src/virtual_list.rs::Measurements`, `src/virtual_list/variable.rs` | list layout and correction | Native list/list model. Preserve anchored correction; separate membership identity from recycled slot identity. |
| Table row/provider realization and cell layout | `src/table.rs`, `src/layout/table.rs`, `src/runtime/services/table.rs` | native table presentation | Native view. It shares container/adjustment behavior but keeps table-owned domain layout. |
| Candidate spatial ancestry and property values | `src/scene/spatial.rs::SpatialTopology`, `src/scene/commit.rs::Properties` | renderer and submitted snapshot | Residency/presentation private projection. Not public scroll content. |
| Desired coverage, candidate selection, runway, coalescing, follow-ups | `src/runtime/presentation.rs`, `src/scene/residency.rs`, `src/platform/native/surface.rs` | native preparation and diagnostics | Residency/presentation. Preserve selected-front and latest-intent invariants. |
| Requested/present-submitted epoch and property serial | `src/session/window.rs::PresentationState` | runtime admission and geometry installation | Residency/presentation submission clock. |
| Installed submitted geometry/offset snapshot | `src/runtime/access.rs`, `src/scene/spatial.rs::SpatialSnapshot` | hit testing, routing, IME, chrome | Residency/presentation atomic snapshot; it projects adjustment values at successful submission. |
| Installed GPU property generation | `src/render/retained.rs` property slots | sparse property preparation | Residency/presentation renderer adapter. |
| Surface acquire/submit/present-call receipts | `src/render/surface.rs` | runtime reports and diagnostics | Residency/presentation hardware boundary; no scanout claim. |
| Redraw demand/in-flight deduplication | `src/platform/mod.rs::RedrawRequests` | backend request/delivery/retry | Platform scheduling owned outside scroll; the container/residency layers may request redraw but do not own this clock. |
| Scroll, residency, frame, property, memory diagnostics | `src/diagnostics/**`, `src/render/report.rs` | receipts and benchmark gates | Diagnostic observers only; never behavioral owners. |

No current scroll state is unassigned. Missing target concepts are recorded as
gaps rather than assigned phantom owners: source-neutral sessions, terminal
velocity/deceleration, exact applied/remainder outcomes, per-axis adjustments,
generic eager adapter, independent axis policy, accessible range/value/actions,
observable list mutation, and factory slot lifecycle.

## 2. Production entrance census

### Direct platform input

- `src/platform/event.rs`: maps main-window and popup `MouseWheel` to a host
  `Scrolled` event; both matches ignore winit phase.
- `src/host/event.rs`: host scroll payload.
- `src/shell/event.rs` and `src/shell/input.rs`: shell adaptation.
- `src/platform/native/surface.rs`: main/popup surface input and native
  presentation continuation.
- `src/platform/runner/native.rs`: native event classification and delivery.

### Framework-authored operations

- `src/input/mod.rs`: `Input::Scroll` and `Input::ScrollTo`.
- `src/runtime/input/dispatch.rs`: applies relative and absolute transitions.
- `src/runtime/input/key.rs`: specialized keyboard behavior.
- `src/runtime/input/text/mod.rs` and `src/runtime/input/text/transfer.rs`: text
  reveal and focus/caret operations.
- `src/runtime/palette.rs` and `src/interaction/command_palette.rs`: command
  results scrolling/reveal.
- `src/layout/chrome.rs` plus `src/runtime/routing.rs`: scrollbar drag maps to
  an absolute offset.
- `src/interaction/scroll.rs` and `src/session/interaction/scroll.rs`: generic
  reveal, active-descendant reveal, and relative/absolute/geometry requests.

These sources currently converge only after target selection. SE-003 makes
them inputs to one source-neutral session/outcome contract.

## 3. Owner and consumer census by stage

### Interaction and adjustment candidates

- `src/interaction/scroll.rs`
- `src/interaction/mod.rs`
- `src/interaction/target.rs`
- `src/session/interaction/scroll.rs`
- `src/session/window.rs`

### Container, routing, and eager viewport candidates

- `src/layout/viewport.rs`
- `src/layout/frame.rs`
- `src/layout/mod.rs`
- `src/layout/chrome.rs`
- `src/runtime/input/dispatch.rs`
- `src/runtime/pointer.rs`
- `src/runtime/routing.rs`
- `src/runtime/visual.rs`
- `src/scene/visual.rs`
- `src/scene/paint/viewport_chrome.rs`
- `src/theme/mod.rs`
- `src/theme/toml.rs`

### Native views and list ownership

- `src/text/view.rs`
- `src/text/layout/**`
- `src/view/control/text_area.rs`
- `src/widget/control/text_area.rs`
- `src/virtual_list.rs`
- `src/virtual_list/variable.rs`
- `src/table.rs`
- `src/layout/table.rs`
- `src/runtime/services/table.rs`

### Private residency and presentation

- `src/runtime/presentation.rs`
- `src/runtime/access.rs`
- `src/scene/residency.rs`
- `src/scene/spatial.rs`
- `src/scene/commit.rs`
- `src/scene/paint/mod.rs`
- `src/render/retained.rs`
- `src/render/scene.rs`
- `src/render/surface.rs`
- `src/platform/native/surface.rs`
- `src/platform/mod.rs`
- `src/session/window.rs`

### Diagnostic consumers

- `src/diagnostics/scroll.rs`
- `src/diagnostics/residency.rs`
- `src/diagnostics/render.rs`
- `src/diagnostics/scroll_bench.rs`
- `src/render/report.rs`
- `tools/renderer_debug/**`
- `examples/control_gallery/**`
- `examples/text_editor/**`

## 4. Verified gaps at the freeze

1. `WinitWindowEvent::MouseWheel { delta, .. }` drops phase for main and popup
   windows.
2. `Viewport::can_consume_from` answers whether any axis can move, then
   `Layout::scroll_target_at` chooses one target. The dispatch path has no
   applied/remainder outcome and no independent per-axis ancestor handoff.
3. `ScrollOffset` is global `i32`; fractional deltas accumulate privately until
   they cross an integral pixel, so touchpad motion cannot remain continuously
   positioned.
4. `VirtualList::Provider` exposes count/key/view/measurement queries but no
   membership event, same-key revision, unique-key failure, or recycled slot
   lifecycle.
5. `OverlayAuto` and `GutterAlways` are theme presentation modes, not
   independent horizontal/vertical behavior.
6. Reveal, programmatic scrolling, scrollbar drag, direct wheel input, and
   specialized keyboard behavior do not share one session/outcome operation.
7. There is no generic accessible range/value/action projection or platform
   accessibility adapter.
8. No existing public trait is justified by three application-meaningful
   implementations; framework-private typed dispatch remains the default.

## 5. Repeatable census commands

Run these at every stage boundary, then inspect and classify new production
hits rather than relying on raw counts:

```text
rg -n -g '*.rs' 'MouseWheel|Scrolled|Input::Scroll|Input::ScrollTo|scroll_to|reveal' src examples
rg -n -g '*.rs' 'ScrollOffset|ScrollDelta|ScrollProjection|resident_offset|desired_offset|present_submitted' src examples
rg -n -g '*.rs' 'scroll_target|can_consume|viewport|scrollbar|OverlayAuto|GutterAlways' src examples
rg -n -g '*.rs' 'candidate_epoch|candidate_property_serial|gpu_submitted_property_serial|present_submitted_property_serial|RedrawRequests' src examples
rg -n -g '*.rs' 'Provider|VirtualList|Measurements|items_changed|bind|unbind|teardown' src examples
rg -n -g '*.rs' 'pub (struct|enum|trait|mod).*?(Content|Sequence|ResidentSet|Request|Plan|Coverage)' src
```

The final forbidden-name search must be interpreted at public scrolling
boundaries: private renderer/residency `Request`, `Plan`, and `Coverage` types
are expected and remain legal.

