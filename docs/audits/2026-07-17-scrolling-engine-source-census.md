# Scrolling engine source census

Status: **SE-004 RE-CENSUS — CONTAINER AND EAGER ADAPTER CONNECTED**

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
| Winit wheel delta and phase | `src/platform/event.rs::scroll_delta_with_phase` | host/shell input adaptation | Connected scroll-container entrance. Pixel and fractional-line values survive with source, unit, timestamp, and Begin/Update/End/Cancel phase metadata. |
| Per-target input session and active kinetic chain | `src/interaction/scroll.rs::ScrollSession`, `src/runtime/mod.rs::KineticScroll` | runtime nested dispatch and animation scheduling | Connected scroll-container state. It owns monotonic phase, velocity/deceleration, interruption, clamped/elastic edge resolution, and the current child-first kinetic target chain. |
| Pointer location used for wheel targeting | `src/platform/event.rs::WindowEvents` | host event and runtime routing | Scroll container session entrance; pointer state itself remains platform input state. |
| Per-axis canonical value and range geometry | `src/interaction/scroll.rs::AxisAdjustment` | `Scroll::request`, configuration, exact projection helpers | Connected target layer: independent horizontal/vertical fixed-point coordinates, lower/upper/page/step/page-increment configuration, clamping, and per-axis revision. |
| Desired and resident-accepted x/y offsets | `src/interaction/scroll.rs::ScrollEntry` | session, runtime transition, layout projection | Desired is derived from the two adjustments; resident-accepted remains a private residency receipt and never competes for canonical ownership. |
| Public `ScrollOffset` accessors and `ScrollDelta` (`f64` x/y) | `src/interaction/scroll.rs` | all input, routing, layout, scene, render, tests | Public constructor/accessor signatures remain unchanged; private coordinates are signed `i64` whole plus 32 fixed fraction bits. Renderer conversion occurs only after rebasing. Public names remain provisional. |
| Per-target and per-axis scroll revisions | `src/interaction/scroll.rs::Scroll::revisions`, `AxisAdjustment::revision` | presentation/layout invalidation and adjustment observation | Value and configuration changes are revisioned; configuration-only changes are observable without pretending a clamp occurred. |
| Viewport, active-descendant, and keyboard-focus reveal requests | `src/interaction/scroll.rs::reveal_requests`, `src/layout/mod.rs::reveal_offsets_for_focus` | session, native reveal, eager container ancestors | Connected scroll-container operation. Ordinary eager focus reveal is minimal, one-shot, and inner-to-outer; native multi-projection views retain domain target selection and merge axes before admission. |
| Programmatic relative/absolute/geometry requests | `src/input/mod.rs`, `src/runtime/input/dispatch.rs` | session interaction and layout | Scroll container source-neutral operation using the same session/outcome path as direct input. |
| Viewport rect/content/page geometry, max range, clamped resolution | `src/layout/viewport.rs::Viewport`, `Layout::scroll_adjustment_geometry` | layout routing, adjustment configuration, chrome, reveal, scene | Connected target layer: each presented target configures both adjustments from aggregated max/page geometry; eager adapter/container still owns generic geometry. Native views supply domain extents. |
| Desired/preparation/range/runway projection | `src/layout/mod.rs::ScrollProjection` | runtime presentation, scene residency | Split: adjustment owns canonical range/value; residency/presentation privately owns coverage/preparation/admission. |
| Deepest-first scroll ancestor chain | `src/layout/mod.rs::scroll_target_chain_at_surface_projected` | runtime pointer scrolling | Connected scroll-container routing. It selects the deepest visible scroll frame, retains only its ancestors, deduplicates shared targets, and never pre-filters on one coupled-axis consume predicate. |
| Exact applied/remaining result | `src/interaction/scroll.rs::ScrollOutcome`, `src/runtime/input/dispatch.rs::dispatch_scroll_event` | nested direct and kinetic dispatch | Connected scroll-container outcome. Actual fixed-point changes are measured after each target transition and the independent x/y remainder is handed to the next ancestor. |
| Generic frame viewport layout and authored container contract | `src/view/node/content.rs::ScrollContainer`, `src/layout/algorithm.rs`, `src/layout/frame.rs` | arbitrary eager widget content | Connected eager viewport adapter. It owns per-axis policy, presentation, sizing, direction, and bounded monotonic chrome introduction without residency knowledge; native views bypass it for domain realization. |
| Scrollbar track/thumb geometry and drag-to-offset mapping | `src/layout/chrome.rs` | runtime routing and scene chrome | Connected scroll-container chrome projected from the matching axis adjustment and resolved per-frame policy/presentation/direction. |
| Scrollbar opacity, hover/press thickness, fade deadline | `src/runtime/visual.rs`, `src/scene/visual.rs` | scene chrome painting | Scroll container presentation/chrome; overlay/layout consumption and axis behavior become distinct policies. |
| Theme `OverlayAuto` / `GutterAlways` | `src/theme/mod.rs`, `src/theme/toml.rs` | eager default resolution plus native layout chrome | Appearance default. Ordinary eager layout resolves it into independent axis behavior plus overlay/consuming presentation; authored container state is no longer a theme-presentation alias. |
| Text reveal/caret correction and height/width indexes | `src/text/layout/**`, `src/runtime/input/text/**` | text area layout, IME, selection | Native text owns domain layout and anchor geometry; container owns scrolling and ancestor reveal. |
| Virtual-list provider identity/count/query | `src/virtual_list.rs::Provider` | virtual list model/request construction | List model. Current query-only contract lacks observable mutation, content revision, uniqueness enforcement, and slot lifecycle. |
| Virtual-list measurements and variable-height region | `src/virtual_list.rs::Measurements`, `src/virtual_list/variable.rs` | list layout and correction | Native list/list model. Preserve anchored correction; separate membership identity from recycled slot identity. |
| Table row/provider realization and cell layout | `src/table.rs`, `src/layout/table.rs`, `src/runtime/services/table.rs` | native table presentation | Native view. It shares container/adjustment behavior but keeps table-owned domain layout. |
| Candidate spatial ancestry and property values | `src/scene/spatial.rs::SpatialTopology`, `src/scene/commit.rs::Properties` | renderer and submitted snapshot | Residency/presentation private projection. Not public scroll content. |
| Desired coverage, candidate selection, runway, coalescing, follow-ups | `src/runtime/presentation.rs`, `src/scene/residency.rs`, `src/platform/native/surface.rs` | native preparation and diagnostics | Residency/presentation. Preserve selected-front and latest-intent invariants. |
| Requested/present-submitted epoch and property serial | `src/session/window.rs::PresentationState` | runtime admission and geometry installation | Residency/presentation submission clock. |
| Installed submitted geometry/offset snapshot | `src/runtime/access.rs`, `src/scene/spatial.rs::SpatialSnapshot` | hit testing, routing, IME, chrome | Residency/presentation atomic snapshot; exact horizontal/vertical fixed-point values are merged by typed axis at successful submission. |
| Installed GPU property generation | `src/render/retained.rs` property slots | sparse property preparation | Residency/presentation renderer adapter. |
| Surface acquire/submit/present-call receipts | `src/render/surface.rs` | runtime reports and diagnostics | Residency/presentation hardware boundary; no scanout claim. |
| Redraw demand/in-flight deduplication | `src/platform/mod.rs::RedrawRequests` | backend request/delivery/retry | Platform scheduling owned outside scroll; the container/residency layers may request redraw but do not own this clock. |
| Scroll, residency, frame, property, memory diagnostics | `src/diagnostics/**`, `src/render/report.rs` | receipts and benchmark gates | Diagnostic observers only; never behavioral owners. |

No current scroll state is unassigned. Missing target concepts are recorded as
gaps rather than assigned phantom owners: a platform accessibility adapter,
observable list mutation, factory slot lifecycle, and proved native text/list
container sharing. The existing ordinary `Scroll` frame is now the complete
proved eager viewport adapter slice; SE-005 owns the next native vertical
slices.

## 2. Production entrance census

### Direct platform input

- `src/platform/event.rs`: maps main-window and popup `MouseWheel` to a host
  `Scrolled` event while retaining winit phase and wheel/touchpad unit/source
  metadata privately inside the delta.
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

Direct wheel/touchpad input, public relative/absolute programmatic input,
scrollbar absolute input, generic keyboard operations, and ordinary eager
focus reveal now converge on canonical adjustments and the session/transition
contract. Specialized native keyboard/reveal paths remain ahead of the generic
adapter; touchscreen is in the internal source vocabulary but has no current
platform gesture entrance.

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

1. Closed by SE-003: main and popup `MouseWheel` retain phase plus source/unit
   metadata through host and shell adaptation.
2. Closed by SE-003: production pointer routing uses a deepest-first ancestor
   chain and exact applied/remainder outcome independently per axis.
3. Closed by SE-002: `ScrollOffset` retains its public integral accessors but
   privately carries a wide fixed-point coordinate; fractional deltas update
   and submit continuously without an integral-pixel gate.
4. `VirtualList::Provider` exposes count/key/view/measurement queries but no
   membership event, same-key revision, unique-key failure, or recycled slot
   lifecycle.
5. Closed by SE-004 for ordinary eager content: independent horizontal and
   vertical Always/Automatic/Never/External behavior, overlay/consuming
   presentation, sizing, direction, and bounded convergence are separate
   container facts.
6. Closed by SE-004 for the generic container: direct wheel/touchpad,
   programmatic, scrollbar, keyboard, and ordinary focus reveal operations use
   canonical adjustment/session ownership. Native text/list sharing remains
   SE-005.
7. Partially closed by SE-004: generic accessible lower/upper/page/value and
   seven actions project from canonical adjustments. No platform accessibility
   adapter exists yet.
8. No existing public trait is justified by three application-meaningful
   implementations; framework-private typed dispatch remains the default.

## 5. SE-001 delta

SE-001 adds only `src/tests/scroll_engine_oracles.rs` and its test-module
declaration. The file is compiled under `#[cfg(test)]`; it imports no
production scrolling type and introduces no public path. Re-running the
entrance, owner, consumer, clock, list, and forbidden-name searches found no
production ownership delta from SE-000. The independent models are behavioral
evidence for later connections, not a seventh ownership layer.

The broad forbidden-name search reports `session::Request` and
`session::RequestKind`. They are the pre-existing public file-dialog request
vocabulary in `src/session/request.rs`, not scrolling, residency, or
virtualization planning, so they do not violate the scrolling API prohibition.

## 6. SE-002 delta

SE-002 replaces `ScrollRemainder` and `Position` with two internal
`AxisAdjustment` values in `src/interaction/scroll.rs`. The adjustment owns
configuration, canonical clamping, external relative/absolute/geometry
control, and revision. `ScrollEntry::desired` projects both canonical values;
`resident_accepted` remains the separately named lifecycle receipt.

`src/layout/mod.rs`, `src/runtime/input/dispatch.rs`,
`src/runtime/presentation.rs`, and `src/session/interaction/scroll.rs` connect
aggregated viewport maximum/page geometry to atomic adjustment configuration
before eager input and on layout feedback. Exact-axis comparisons replace
legacy accessor comparisons in routing, acceptance, residency, pending native
reversal detection, active-stack projection, and submitted-target merging.

`src/scene/spatial.rs` rebases fixed coordinates before `f32` conversion for
GPU transforms and retains the exact axis values in `SpatialSnapshot`; integer
frame/hit geometry rounds only the already-rebased local delta. The ordinary
eager vertical/horizontal integration witness in `src/tests/layout_scene.rs`
proves property-only subpixel motion and same-pixel reversal through the
submitted snapshot. Architecture gates now reject restoration of the old
remainder/quantization owner.

The boundary census found 328 entrance, 1,002 scroll-state, 1,939
routing/container, 104 presentation-clock, and 1,014 list/lifecycle source
hits. Inspection found no new owner or public forbidden-name candidate. The
two broad forbidden-name hits remain the unrelated file-dialog
`session::Request` and `RequestKind` described in SE-001.

## 7. SE-003 delta

SE-003 retains winit phase, source, unit, and monotonic timestamp metadata in a
private `ScrollDelta` sample without changing its public x/y surface. Each
target now owns a `ScrollSession`; runtime owns only the active per-window
kinetic chain and schedules deceleration through the existing animation clock
at a four-millisecond minimum cadence. New direct input removes that chain
even when submitted geometry is temporarily unavailable, and departed windows
remove it through the normal listener ledger.

`Layout::scroll_target_chain_at_surface_projected` replaces the production
single-target consume probe with a deepest-first, ancestor-only target chain.
`Runtime::dispatch_scroll_event` measures actual fixed-point changes after
each existing scroll transition, produces an independent x/y `ScrollOutcome`,
and hands only the exact remainder to the next ancestor. Clamped edges remain
the default; private elastic state absorbs only the final outer remainder and
does not compete with the canonical adjustments.

Direct wheel/touchpad input, programmatic relative/absolute input, and
scrollbar drag now converge on session/transition ownership. Keyboard and
reveal enter the complete container contract in SE-004. Nine production
witnesses cover metadata preservation, session lifecycle and stale input,
terminal velocity and cancellation, fractional diagonal boundary/reversal
outcomes, three-target handoff, deepest-first ancestry, real kinetic motion,
direct interruption, and independent kinetic-axis stopping.

The boundary census found 334 entrance, 1,194 scroll-state/session, 1,951
routing/container, 104 presentation-clock, and 1,014 list/lifecycle source
hits. Inspection found no new owner or public forbidden-name candidate. The
only broad public-name hits remain the unrelated file-dialog
`session::Request` and `RequestKind` described in SE-001. The complete
all-target/all-feature suite passed 1,395 library tests with four intentional
hardware ignores, three renderer-debug tests with 27 hardware ignores, and two
example tests; all 18 Python manifest/receipt/census checks and the frozen
release table-scroll smoke also passed.

## 8. SE-004 delta

SE-004 adds a private `ScrollContainer` configuration only to ordinary eager
scroll nodes. `src/layout/algorithm.rs` resolves theme defaults or authored
state into independent axis visibility, overlay/consuming chrome, min/natural
sizing, direction, and a monotonic initial-plus-two-introduction-pass layout.
`src/layout/chrome.rs` retains the resolved contract per frame and projects
full-track Always chrome, hidden External/Never chrome, and left-gutter RTL
vertical placement. Non-scroll overlay stacks and native view layout retain
their prior placement paths.

`src/interaction/scroll.rs` now adapts step/page/start/end/set operations and
accessible range/value/actions to the two canonical adjustments. Runtime key
dispatch preserves specialized text/table/list/palette ownership, then walks
ordinary/native scroll ancestors deepest first for unmodified generic scroll
keys. Ordinary keyboard focus reveal is pending exactly once per visible focus
change, traverses only eager scroll ancestors, and does not pin later manual
scrolling. Existing active-descendant native reveal continues combining split
horizontal/vertical projections sharing one target.

The boundary census found 380 entrance, 1,214 scroll-state/session, 2,011
routing/container, 104 presentation-clock, and 1,014 list/lifecycle source
hits. Inspection found no new public forbidden-name candidate. The only broad
public-name hits remain the unrelated file-dialog `session::Request` and
`RequestKind` described in SE-001. The complete all-target/all-feature suite
passed 1,402 library tests with four intentional hardware ignores, three
renderer-debug tests with 27 hardware ignores, and two example tests; all 18
Python checks and the frozen release table-scroll smoke also passed.

The reported symptom boundary for the next stages is intentionally retained:
large text documents scroll cleanly, while large virtual lists exhibit lag.
SE-005 through SE-009 must compare those paths and attribute any delta to native
list realization/provider/residency/scheduling work rather than assuming a
shared adjustment defect.

## 9. Repeatable census commands

Run these at every stage boundary, then inspect and classify new production
hits rather than relying on raw counts:

```text
rg -n -g '*.rs' 'MouseWheel|Scrolled|Input::Scroll|Input::ScrollTo|scroll_to|reveal' src examples
rg -n -g '*.rs' 'ScrollOffset|ScrollDelta|ScrollEvent|ScrollOutcome|ScrollProjection|kinetic_scrolls|resident_offset|desired_offset|present_submitted' src examples
rg -n -g '*.rs' 'scroll_target|can_consume|viewport|scrollbar|OverlayAuto|GutterAlways' src examples
rg -n -g '*.rs' 'candidate_epoch|candidate_property_serial|gpu_submitted_property_serial|present_submitted_property_serial|RedrawRequests' src examples
rg -n -g '*.rs' 'Provider|VirtualList|Measurements|items_changed|bind|unbind|teardown' src examples
rg -n -g '*.rs' 'pub (struct|enum|trait|mod).*?(Content|Sequence|ResidentSet|Request|Plan|Coverage)' src
```

The final forbidden-name search must be interpreted at public scrolling
boundaries: private renderer/residency `Request`, `Plan`, and `Coverage` types
are expected and remain legal.
