# Scrolling engine campaign

Status: **SE-007 CLOSED — SE-008 NEXT**

Date: 2026-07-17

Execution base: `master` at `1c480c82cf750df445a00b755f7a47ed13af60f4`

Source census: `docs/audits/2026-07-17-scrolling-engine-source-census.md`

Supersession: this campaign replaces the withdrawn GS-002-and-later plan. It
rejects `scroll::Content` and treats the current code, documentation, tests,
and public API as evidence rather than authority.

## 1. Target architecture

The target is a scrolling engine with GTK-shaped separation where that
separation fits this framework. It is not a GTK API transcription.

| Layer | Owns |
|---|---|
| Axis adjustment | One axis's canonical value, legal range, page geometry, increments, atomic configuration, and revisioned observation |
| Scroll container | Input sessions, nested propagation, chrome, policy, sizing, keyboard operations, reveal, RTL behavior, and accessible projection |
| Viewport adapter | Scrolling arbitrary eager widget content that does not implement a native scrolling contract |
| Native views | Text-, list-, table-, and canvas-specific layout, realization, and anchoring |
| List model/factory | Membership mutation, stable item identity, content revision, slot setup/binding/unbinding/teardown, and recycling |
| Residency/presentation | Private desired-coverage preparation, admission, coalescing, complete-pixel activation, and atomic submission |

The private residency vocabulary remains an implementation detail:

```text
Request -> Plan -> Coverage
```

`Content`, `Sequence`, `ResidentSet`, `Request`, `Plan`, and `Coverage` are
forbidden public scrolling API candidates. Public names remain provisional
through SE-007. The likely central name is `Scroll`; `scroll::Adjustment` is a
candidate. A public `Scrollable` trait must be earned by a real application
implementation axis after the eager viewport, native text, and native list
vertical slices exist.

## 2. Campaign ledger

| Stage | State | Exit |
|---|---|---|
| SE-000 — Freeze evidence and ownership | **closed** | Baseline is reproducible; the freeze evidence is preserved; every current scroll state has an assigned layer. |
| SE-001 — Green behavioral oracles | **closed** | Independent models cover motion, sessions, handoff, sources, policy, reveal, mutation, anchoring, and accessibility; deliberate faulty adapters prove every witness. |
| SE-002 — Axis adjustment | **closed** | Eager horizontal and vertical scrolling use an internal adjustment with a wide continuous coordinate and no public break. |
| SE-003 — Sessions and nested handoff | **closed** | Fractional, diagonal, boundary, reversal, interruption, and child/ancestor remainder oracles pass per axis. |
| SE-004 — Container and eager adapter | **closed** | One ordinary eager widget exercises the full container contract. |
| SE-005 — Native text and list | **closed** | Eager viewport, text, and list share container behavior without a virtualization-shaped public abstraction. |
| SE-006 — List model/factory lifecycle | **closed** | Mutation touches affected ranges/bindings; realization is limited to entering items; identity and slot lifecycle are distinct. |
| SE-007 — Private residency/presentation | **closed** | Warm transform-only motion performs zero application-view rebuilds; every selected front retires or is superseded with one latest-intent continuation. |
| SE-008 — Names and public break | next | Proved names replace old paths in one green migration, with no aliases or compatibility layer. |
| SE-009 — Performance and closure | queued | Required CPU/GPU/native protocols meet the frozen gates and the source census reaches a fixed point. |

Every commit must be green. Defaults preserve current appearance unless a
stage explicitly owns visual policy. The campaign stops only for overlapping
user changes, irreproducible baseline evidence, or unavailable required
hardware. The glyphon fork and a replacement Cosmic Text renderer are outside
scope.

## 3. SE-000 freeze receipt

The causal freeze receipt remains the record named
`control-gallery-500px-idle-1784255665430.txt`, preserved in the predecessor
campaign's committed evidence section. It recorded 242 scroll inputs, three
selected residency candidates, 316 preparation slices, a 46,240 us maximum
preparation slice, frame-interval p95/p99/max of
671,846/1,425,310/1,517,082 us, fast-burst input-to-present of 675,643 us, and
surface-acquire p95 of 32 us. The evidence assigned the freeze to cold
residency preparation/scheduling rather than GPU surface acquisition.

The corrected predecessor campaign later closed that defect with native
receipts `control-gallery-500px-idle-1784266131025.txt` and
`control-gallery-500px-idle-1784266180280.txt`. Those receipts are controls,
not a reason to discard the original causal evidence. Their direct schedules
plus follow-ups equaled selections exactly, final candidate/GPU/present
serials converged, acquire p95 was 35/41 us, and preparation remained below
2,165 us.

The source of truth for both records is
`docs/audits/2026-07-16-payload-neutral-scroll-architecture-campaign.md`.
Generated receipt files live below ignored `target/` directories and are not
treated as durable repository inputs.

## 4. SE-000 performance baseline

All executable baselines below were taken from release artifacts built from
`1c480c82cf75` on Windows x86_64. SE-000 records one reproducible reference
run; SE-009 owns repetitions, medians, and closure gates.

### CPU and resident memory

Protocol: the benchmark's official 64 warmups and 1,024 samples.

| Workload | Cold | Projection p50/p95/p99/max | Edit p50/p95/p99/max | Resident high-water observations |
|---|---:|---:|---:|---|
| `text-horizontal-edit-4m` | 1,182,729 us | 357/525/557/736 us | 6/8/9/48 us | horizontal index 3,540,660 B; horizontal window 158,730 B; line cache 16,767,660 B; max incremental source 820 B |
| `text-vertical-8m` | 6,566 us | 1,232/1,495/1,557/1,908 us | n/a | line cache 3,494,686 B; cold max 1,551,056 B; bounded window 1,432 x 1,154 |

Reproduction:

```text
cargo run --release -p renderer_debug -- scroll-bench text-horizontal-edit-4m
cargo run --release -p renderer_debug -- scroll-bench text-vertical-8m
```

### GPU work and residency

`table-scroll-work` passed at scale factors 1.0, 1.25, 1.5, 1.75, and 2.0
with an identical warm property result at every scale: 528 upload bytes (64
node, 272 scroll, 192 retained text), 17 value visits, 17 index lookups, one
dirty index, two write ranges, zero full-transfer reasons, zero resource
creation/replacement/removal, zero plan rebuilds, and one plan reuse. The
fixture held 121 GPU resources totaling 190,752 bytes.

At 1.25x, all three cold residency crossings produced complete pixels and the
following observations:

| Payload | Candidate CPU | Provider calls | Crossing content/property upload | GPU high-water | Warm follow-up |
|---|---:|---:|---:|---:|---|
| text | 2,949 us | 0 | 800/16 B | 23 resources, 180,880 B | zero rebuild/preparation/churn; 1,568 property B |
| table | 1,646 us | 33 | 2,880/5,664 B | 169 resources, 197,376 B | zero rebuild/preparation/churn; 2,128 property B |
| virtual list | 368 us | 11 | 320/16 B | 34 resources, 184,008 B | zero rebuild/preparation/churn; 800 property B |

Reproduction:

```text
cargo run --release -p renderer_debug -- table-scroll-work <scale>
cargo run --release -p renderer_debug -- residency-crossing-work text 1.25
cargo run --release -p renderer_debug -- residency-crossing-work table 1.25
cargo run --release -p renderer_debug -- residency-crossing-work virtual-list 1.25
```

### Native Windows session

The release `control_gallery` was rebuilt from the execution base and driven
through a deterministic 500-pixel table scenario: one 500-unit probe, one
reverse probe, four 4,000-unit forward bursts, and four 4,000-unit reversal
bursts, followed by `Write receipt`. The receipt is
`control-gallery-500px-idle-1784294987028.txt`.

Environment: NVIDIA GeForce RTX 4070 Ti SUPER, DX12, driver 32.0.15.9636,
DISPLAY1 at 240 Hz, 1,270 x 1,344 Bgra8UnormSrgb Mailbox surface.

| Observation | Baseline |
|---|---:|
| Wheel/input events | 10/10 |
| Desired changes / residency selections / needs-residency | 9/9/9 |
| Virtual residency rejections | 0 |
| Scroll request p95 | 31 us |
| Frame interval p50/p95 | 8,231/20,432 us |
| Acquire wait p50/p95 | 17/21 us |
| Draw p50/p95 | 2,855/3,490 us |
| Property prepare p50/p95 | 225/317 us |
| Property encode-submit-present p50/p95 | 2,511/2,930 us |
| Commit preparation total/max | 85,810/1,876 us |
| GPU resource high-water | 616 resources, 242,144 B |
| Text line-cache high-water | 3,354,790 B |
| Maximum desired/resident y lag | 934 px |

The deliberately separated automation bursts contaminate frame p99/max and
input-to-present tails. Those values are retained in the receipt for diagnosis
but are not latency gates. SE-009 replaces this smoke protocol with three
30-second sessions after five-second warmup and refresh-relative acceptance.

## 5. Stage definitions

### SE-001 — green behavioral oracles

Land independent reference models for fractional and diagonal motion;
begin/update/end/cancel plus terminal velocity; kinetic interruption; exact
child-applied and parent-remaining displacement independently per axis; wheel,
touchpad, touchscreen, scrollbar, keyboard, reveal, and programmatic sources;
per-axis policy and cross-axis scrollbar convergence; nested focus reveal;
insertion, removal, replacement, movement, same-key revision, duplicate keys,
variable-extent anchor correction; and accessible range/value/action
projections. Deliberately faulty adapters must prove each witness. No failing
test is committed; a production connection and its correction land together.

### SE-002 — internal axis adjustment

Introduce one adjustment per axis with canonical clamped value, lower/upper
range, page size, step/page increments, atomic reconfiguration, revisioned
observation, and external control. Use a wide continuous coordinate, with a
normalized `i64` whole component plus fractional local displacement as the
preferred candidate. Rebase before GPU conversion. `desired`,
`resident_accepted`, and `present_submitted` remain private lifecycle receipts
projecting the adjustment, never competing value owners.

### SE-003 — sessions and nested handoff

Add source-neutral Begin/Update/End/Cancel/deceleration, source, unit,
timestamp, phase, velocity, and an outcome carrying exact applied and remaining
displacement per axis. Dispatch child first, then propagate remainder through
ancestors. Direct input interrupts kinetics. Support clamped and elastic edge
behavior internally while retaining clamped visuals by default.

### SE-004 — container and eager viewport adapter

Give the container horizontal and vertical adjustments; independent per-axis
Always/Automatic/Never/External scrollbar behavior; overlay versus
layout-consuming presentation as a separate choice; minimum versus natural
sizing; monotonic convergence with initial layout plus at most two
scrollbar-introduction passes; keyboard step/page/home/end; minimal nested
reveal; RTL semantics and placement; and accessible range/value/action data.
Prove arbitrary eager widget content through an adapter that knows nothing
about residency.

### SE-005 — native text and list

Migrate text and list as separate vertical slices. They share adjustments and
container behavior but retain domain layout and do not route through the eager
adapter. Decide trait visibility only after eager viewport, text, and list are
real implementations.

### SE-006 — list model and factory lifecycle

Place insertion, removal, replacement, movement, same-key invalidation, unique
stable-key enforcement, item/position/slot identity, setup/bind/unbind/teardown,
listener cleanup, logical focus/capture/editor/popup handling, and anchored
measurement correction under list ownership.

### SE-007 — private residency and presentation

Reattach desired versus resident-ready state, bounded predictive runway,
latest-request coalescing, O(entering/departing) realization, complete-pixel
activation, and one atomic submitted snapshot for content, chrome, hit testing,
IME, and geometry. Large jumps may delay presentation until coverage is ready
but may not block input, orphan a selected candidate, or lose newest intent.

### SE-008 — names and public break

Apply repository naming rules only after the vertical slices prove the
concepts. Keep list model/factory below list/collection ownership. Delete old
public paths in the same green migration as examples and consumers; retain no
aliases.

### SE-009 — performance and closure

Run release CPU with 64 warmups/1,024 samples; GPU at five scales with five
repetitions and exact work/pixel comparison; and three native 30-second
sessions after five-second warmup, recording OS, adapter, driver, and refresh.
Measure input dispatch, candidate preparation, submission, and present-call
acknowledgement separately and make no scanout claim. No median p95 may regress
more than 10% from SE-000. Warm scrolling must perform zero application-view
rebuilds and zero semantic/layout/text preparation with bounded property
writes. Residency must bound visible page, before/after runway, pins, and the
separately capped recycle reserve. Every selected front retires or is explicitly
superseded with exactly one latest-intent continuation. Run freeze, reversal,
thumb-jump, mutation, nested-boundary, and text-reveal scenarios.

## 6. SE-001 oracle receipt

`src/tests/scroll_engine_oracles.rs` is a test-only, production-independent
reference model. It adds 20 green tests covering:

- continuously visible fractional and diagonal motion;
- atomic clamped axis configuration and one revision per configuration;
- source, unit, monotonic timestamp, Begin/Update/End/Cancel/deceleration,
  terminal velocity, and direct-input kinetic interruption;
- exact child applied and ancestor remaining displacement independently per
  axis;
- wheel, touchpad, touchscreen, scrollbar, keyboard, reveal, and programmatic
  sources through one axis law;
- Always/Automatic/Never/External axis policy, overlay versus consuming
  presentation, and monotonic cross-axis convergence within two introduction
  passes;
- minimal two-axis focus reveal through every nested ancestor;
- insertion/entry, deletion, replacement/recycling, movement, same-key
  revision, unique keys, stable item/slot identity, logical interaction-state
  cleanup, and setup/bind/unbind/teardown;
- variable-extent correction around a stable visible key; and
- accessible lower/upper/page/value plus step/page/start/end/set actions.

Negative adapters deliberately quantize each update, drop one axis, lose
terminal velocity, retain kinetics across direct input, swallow remainders,
couple axes, bypass continuous/absolute/keyboard sources, stop scrollbar
layout after one pass, consume layout for overlays, reveal only the child,
align reveals to the start, key list identity by position, ignore same-key
revision, leak departed logical state, accept duplicate keys, omit anchor
correction, project stale resident accessibility values, and omit accessible
actions. Every faulty adapter is rejected by the corresponding green witness.

Narrow reproduction:

```text
cargo test --lib scroll_engine_oracles --all-features
```

SE-001 changes no production behavior and names no public API. Each following
production connection must land with the oracle it satisfies and remain green.
Formatter and the complete workspace all-target/all-feature suite pass: 1,383
library tests with four intentional hardware ignores, three renderer-debug
non-hardware tests with 27 hardware ignores, and two example tests. All 18
manifest/receipt/census Python checks also pass.

## 7. SE-002 adjustment receipt

`src/interaction/scroll.rs` now owns one internal `AxisAdjustment` for each
axis of a scroll target. Each adjustment contains one canonical value, an
atomic `AxisConfiguration` with lower/upper/page/step/page-increment values,
and a monotonic revision. Configuration-only changes advance the observable
target revision even when clamping does not change the value. Relative,
absolute, scrollbar, reveal-geometry, and programmatic requests all control
the same value.

The coordinate is a normalized signed `i64` whole component plus 32 fixed
fraction bits. Per-event input no longer crosses an integer-pixel admission
boundary: all five scale traces update continuously, and reversals below one
legacy integer pixel remain routable. Public `ScrollOffset::new`, `x`, and `y`
signatures are unchanged; the continuous representation and exact-axis
operations remain private implementation details while names stay
provisional.

Layout configures range and page geometry before input admission and again as
layout feedback. Exact values survive clamp, residency admission, property
projection, active-stack projection, pending-intent reversal checks, and the
installed present-submitted spatial snapshot. `desired` is derived from the
two adjustment values; `resident_accepted` remains a separate private receipt.
No competing value owner was introduced.

Renderer scroll transforms subtract the fixed-point baseline and current
coordinates before converting the bounded local delta to `f32`. A negative
control at 20,000,000/30,000,000 logical pixels demonstrates that converting
the global positions first loses a half/quarter-pixel displacement while the
production rebased path retains it. The ordinary eager integration witness
drives vertical and horizontal fractional updates plus same-pixel reversals
through routing, property-only rendering, and the submitted hit/IME/geometry
snapshot without rebuilding the retained commit.

Verification passed:

- the 20 independent SE-001 behavioral oracles;
- 1,386 library tests, with four intentional hardware ignores;
- three renderer-debug non-hardware tests, with 27 intentional hardware
  ignores, plus two example tests;
- all 18 manifest/receipt/census Python checks; and
- release `table-scroll-work 1.25`, reproducing the frozen 528 property bytes,
  17 visits/lookups, one dirty index, two write ranges, zero content rebuild or
  preparation, zero GPU resource churn, and one retained-plan reuse.

SE-002 adds no public scrolling path and does not settle the SE-008 naming
decision. Winit phase loss, source-neutral sessions, terminal velocity,
kinetic interruption, and exact per-axis child/ancestor remainder propagation
remain the first unmet exit and belong to SE-003.

## 8. SE-003 session and nested-handoff receipt

Winit main-window and popup wheel input now preserves `TouchPhase` as an
internal Begin/Update/End/Cancel session sample. The private sample retains
source, original unit, monotonic timestamp, phase, and optional velocity while
the public `ScrollDelta` constructor and x/y accessors remain unchanged. Line
wheel input and pixel touchpad input enter the same normalized motion law with
their original source/unit classification intact.

Each scroll target owns a private session beside its two adjustments. The
session rejects stale timestamps, tracks terminal velocity, accepts explicit
deceleration, clears cancellation state, and interrupts kinetic state on new
direct input. Touchpad/touchscreen terminal velocity drives real runtime
deceleration through the existing animation scheduler at a bounded four
millisecond cadence. Exponential drag is integrated in logical coordinates;
blocked axes stop independently, and direct relative, absolute, or scrollbar
input retires the active kinetic chain immediately. Departed windows remove
kinetic state through the standard listener ledger.

Pointer routing no longer selects the first viewport that can consume either
axis. Layout identifies the deepest containing scroll frame, retains only its
scroll ancestors, orders them deepest first, and deduplicates shared targets.
Runtime dispatch applies the current exact remainder at each target, measures
the actual fixed-point offset change, and passes the resulting x/y remainder
to the next ancestor. The existing property-tick/residency transition remains
the only mutation path and diagnostics aggregate once per physical input.

`ScrollOutcome` carries exact applied and remaining displacement independently
per axis. Default edges remain canonically clamped. A private elastic mode
absorbs only the final outer remainder into separate elastic displacement, so
default visuals do not change and ancestor handoff is never swallowed.

Production witnesses cover platform phase/source/unit/timestamp preservation;
session lifecycle, stale input, terminal velocity, cancellation, deceleration,
and interruption; exact fractional diagonal clamp/reversal outcomes; a
three-target child/middle/outer handoff; deepest-first layout ancestry; real
post-End kinetic motion; and independent kinetic boundary stopping. The 20
production-independent SE-001 oracles remain green.

Verification passed:

- 1,395 library tests, with four intentional hardware ignores;
- three renderer-debug non-hardware tests, with 27 intentional hardware
  ignores, plus two example tests;
- all 18 manifest/receipt/census Python checks; and
- release `table-scroll-work 1.25`, reproducing 528 property bytes, 17
  visits/lookups, one dirty index, two write ranges, zero content rebuild or
  preparation, zero GPU resource churn, and one retained-plan reuse.

SE-003 adds no public scrolling path and settles no SE-008 name. Keyboard
operations, nested reveal, independent per-axis policy, sizing, RTL placement,
and accessible range/value/actions remain the first unmet exit and belong to
SE-004.

## 9. SE-004 container and eager-adapter receipt

The ordinary eager `Scroll` node now carries a framework-private container
contract beside its existing offset. Horizontal and vertical policy are
independent `Always`/`Automatic`/`Never`/`External` choices; overlay versus
layout-consuming chrome, horizontal and vertical minimum/natural sizing, and
left-to-right versus right-to-left direction are separate facts. The default
adapter resolves the existing theme into this contract at layout time, so
native text, table, and list frames retain their domain layout while the eager
adapter proves arbitrary widget content without learning residency vocabulary.
All names and the authoring builder remain private pending SE-008.

Eager layout performs one initial placement followed by at most two monotonic
scrollbar-introduction passes. A consuming bar reserves only its resolved axis;
an overlay never consumes layout; a second bar may appear after the first
reduces the viewport; and an introduced bar is never removed within the layout
cycle. Overlay eager content can overflow both axes without changing ordinary
non-scroll overlay placement. Always-visible non-overflow bars project a full
track thumb, External retains adjustment geometry without internal chrome, and
right-to-left consuming vertical chrome occupies the left gutter.

Step, page, start, end, and absolute-value operations all read and update the
canonical adjustment while preserving the other axis. Unmodified arrow,
PageUp/PageDown, Home, and End keys traverse scroll ancestors deepest first;
specialized text, table, list, palette, and shortcut handling remains ahead of
the generic adapter. RTL horizontal physical arrows and logical start/end are
resolved deliberately. Accessible lower, upper, page, canonical value, and all
seven actions project from the same adjustment before any platform adapter
exists.

Keyboard focus reveal is a one-shot container operation, not a permanent pin.
It computes minimal displacement through every ordinary eager scroll ancestor,
translating the descendant rectangle after each inner offset. Existing native
active-descendant reveal retains its multi-projection axis merge, so table
horizontal and vertical projections sharing one target continue to combine
before admission. Reveal changes carry the Reveal source through the session
path; unchanged geometry does not manufacture session activity.

Production witnesses cover independent policy and presentation, two-pass
cross-axis convergence, min/natural sizing, arbitrary two-axis eager content,
Always/External/Never behavior, RTL placement and operations, canonical
accessible projection/actions, runtime step/page/home/end dispatch, nested
focus reveal, and post-reveal manual scrolling. The complete reveal family and
the 20 production-independent SE-001 oracles remain green.

Verification passed:

- 1,402 library tests, with four intentional hardware ignores;
- three renderer-debug non-hardware tests, with 27 intentional hardware
  ignores, plus two example tests;
- all 18 manifest/receipt/census Python checks; and
- release `table-scroll-work 1.25`, reproducing 528 property bytes, 17
  visits/lookups, one dirty index, two write ranges, zero content rebuild or
  preparation, zero GPU resource churn, and one retained-plan reuse.

A user-observed diagnostic distinction is now an explicit SE-005 through
SE-009 comparison case: large text documents scroll cleanly while large virtual
lists exhibit lag under the shared engine. Until measured otherwise, the
working hypothesis is list-specific realization, provider, residency-admission,
or scheduling work rather than the common adjustment/input path. SE-005 must
first connect native text and list to the proved container behavior without
erasing that contrast; SE-006 and SE-007 then own list lifecycle and residency
causes directly.

SE-004 adds no public scrolling path and settles no SE-008 name. Native text and
list sharing the same adjustment/container behavior without passing through the
eager adapter is the first unmet exit and belongs to SE-005.

## 10. SE-005 native text-and-list receipt

The private container policy now belongs to the scroll-container node rather
than the eager content variant. Ordinary eager content, native text areas, and
native virtual lists therefore carry one authored/default contract without
pretending that their content models or layout algorithms are interchangeable.
The node scene key observes policy changes, while scroll offsets remain outside
semantic scene state as before.

`layout::chrome` now owns theme-default resolution, domain-allowed axes, the
initial Always set, and monotonic Automatic introduction. Eager layout continues
to place ordinary child stacks. Native text instead resolves its own shaped
viewport and repeats domain layout only when consuming chrome actually changes
geometry. Fixed and variable virtual lists resolve the same container around
their own content-height, measurement, anchoring, and materialization request
logic. Overlay introduction reuses the already-computed native result; a
consuming variable-list gutter remeasures only after the available width changes.

Each native frame retains the resolved presentation, direction, axes, and
introduction count beside its viewport. Chrome, RTL direction, keyboard target
selection, accessible operations, canonical adjustments, sessions, and nested
handoff consequently observe the same container result across all three species.
Native text and list do not call eager `scroll_stack_placement`. Generic one-shot
focus reveal now tests eager-container identity explicitly rather than using the
presence of container state as a proxy, preserving native caret/row target
selection and the existing table multi-axis reveal merge.

No public `Scrollable` trait is introduced. Applications cannot currently
implement a native scroller without private node, frame, layout, adjustment, and
residency types, so an open implementation axis has not been earned. Framework-
private typed dispatch is the proved boundary; public names remain open until
SE-008. The behavioral witness configures one consuming RTL Automatic policy on
eager, text, and list nodes, observes identical one-pass vertical chrome and
gutter geometry, and separately proves only the ordinary node is an eager
adapter. Architecture gates retain text shaping and list request/measurement
ownership and reject routing either native species through the eager adapter.

Verification passed:

- 1,403 library tests, with four intentional hardware ignores;
- three renderer-debug non-hardware tests, with 27 intentional hardware
  ignores, plus two example tests;
- all 18 manifest/receipt/census Python checks; and
- release `table-scroll-work 1.25`, reproducing 528 property bytes, 17
  visits/lookups, one dirty index, two write ranges, zero content rebuild or
  preparation, zero GPU resource churn, and one retained-plan reuse.

SE-005 deliberately changes no list provider, materialization, residency, or
scheduling lifecycle. Large text remaining clean while large virtual lists lag
after both use the same container contract is therefore stronger evidence
against the common adjustment/input/chrome path. Observable membership,
same-key revision, unique identity, and setup/bind/unbind/teardown remain the
first unmet exit and belong to SE-006.

## 11. SE-006 list model-and-factory receipt

Virtual-list ownership now distinguishes stable logical `Key`, current index,
and process-local recycled `Slot`. The list-owned provider contract observes
ordered insert, remove, replace, and move events plus monotonic membership,
per-item content, and factory revisions. Stable-key and `index_of` queries must
round-trip exactly; duplicate keys fail instead of being silently deduplicated.

Each list model retains a bounded slot pool across application-view projections.
Departing items unbind before entering items bind, so an entering row can reuse a
departing slot without an extra setup. Unchanged keys with equal content and
factory revisions preserve their node and slot; moves update position without a
bind; a same-key revision unbinds and rebinds exactly that slot. Unknown or
changed factory revisions unbind and teardown every incompatible slot before new
setup. The separately capped recycle reserve holds at most 32 unbound slots, and
model destruction pairs every setup with one teardown.

The installed view supplies the prior list model directly to the next projected
view. Runtime does not own slot state. Table rows project current positional
metadata independently from stable-key content, so sorting/reordering preserves
row and cell identity without leaving stale indices. Callback-backed typed table
sources may supply exact item revisions; the one-million-row control gallery does
so from its record values. In-memory typed sources derive a generation revision
from their retained record allocation. `None` remains the conservative rebuild
path while public list names are provisional through SE-007; SE-008 must remove
that compatibility default when it settles the model/factory API.

Existing composition ownership continues to clean logical focus, capture,
editors, and context-popup anchors by stable key when an item departs. Existing
variable-list measurement reconciliation continues to preserve the visible
stable-key anchor through reordering, deletion, and width-dependent correction.
The new lifecycle witness exercises insertion, removal, replacement, movement,
same-key revision, factory replacement, slot reuse, and exact
setup/bind/unbind/teardown counts. Separate witnesses retain focus/editor pins,
pointer-capture pins, selection, popup/context row identity, variable measurement
anchoring, and duplicate-key rejection.

The user-observed text/list discriminator produced a direct result. At 1.25x, the
frozen release residency crossing changed from 11 to three virtual-list provider
calls and from 33 to nine table cell calls: exactly three entering rows, with no
rebuild of overlapping rows. An unchanged million-row application-view projection
constructs zero rows. The release gate now rejects any ordinary crossing that
builds more than three entering list rows (or their three table cells). Large text
remains on its separate clean native path.

Verification passed:

- the 20 independent SE-001 behavioral oracles;
- the complete all-target/all-feature suite: 1,406 library tests with four
  intentional hardware ignores, three renderer-debug tests with 27 intentional
  hardware ignores, and two example tests;
- the architecture gates for list ownership, identity, slot lifecycle, and
  entering-only residency work;
- release `residency-crossing-work` at 1.25x for virtual list and table, reporting
  three and nine provider calls respectively; and
- all 18 manifest/receipt/census Python checks; and
- formatter, diff checks, and release `table-scroll-work 1.25`, reproducing the
  frozen 528 property bytes with zero content work or GPU-resource churn and one
  retained-plan reuse.

SE-006 does not remove application-view rebuilding for cold residency work and
does not settle public names. Warm transform-only presentation, private residency
coalescing, selected-front retirement, and zero-view-rebuild scheduling are the
first unmet exit and belong to SE-007.

## 12. SE-007 entry evidence: stranded resident row interval

The user reproduced a second list-specific failure after the SE-006 lifecycle
work. Drag-to-scroll initially advances, then visible table motion becomes
confined to an approximately 22-row interval. Reaching one end and continuing
to scroll leaves the pixels stuck; later input can move the view back toward the
other end of the same interval. Clicking inside the table installs a different
approximately 22-row interval, after which the same behavior repeats. The user's
working interpretation is that adjustment/input continues while only one stale
resident coverage window remains presentable. This is the primary SE-007
freeze/liveness witness, not a reason to reopen the clean large-text path.

The corresponding native receipt is
`control-gallery-500px-idle-1784307153888.txt`, captured at 240 Hz on the same
Windows/DX12 RTX 4070 Ti SUPER environment as SE-000. The generated file remains
under ignored `target/release/examples/renderer-receipts/`; the following facts
are the durable record:

- 3,880 scroll input events produced 3,824 desired changes, but only 185 resident
  acceptances/property ticks and 3,639 needs-residency outcomes;
- maximum desired/resident vertical lag reached 12,763,262 logical pixels;
- the scheduler recorded 80 candidate schedules, 3,679 coalesced requests, 244
  selected candidates, 164 follow-ups, 29 supersessions/proactive preemptions,
  six cancelled pipelines, and only six virtual guard crossings/replenishment
  commits;
- virtual residency reported zero formal rejections and no last issue, while the
  final candidate/attempted/GPU-submitted/present-submitted property serials all
  converged at 735;
- retained traces repeatedly show the table target still producing
  needs-residency while resident y remains near 12,735,755 and desired y changes;
  several later candidates reuse property serials 698 or 706 with zero layout,
  scene-paint, preparation, or content-upload work; and
- acquire p95 was 37 us, whereas view-rebuild p95 was 4,421 us and presentation-
  layout p95 was 22,517 us. Surface acquisition is not the observed freeze
  boundary.

The converged final serials mean the receipt does not describe a permanently
unretired final frame. It does expose an intermediate liveness defect: thousands
of newest desired values are coalesced behind a small number of resident-window
updates, and a semantic click can refresh coverage that scrolling alone does not.
SE-007 must turn this observation into a deterministic witness over selected-front
retirement, latest-intent continuation, and resident-range advancement. It must
also prove that a warm transform-only tick performs no application-view rebuild;
the click may not remain an accidental recovery mechanism.

## 13. SE-007 private residency-and-presentation receipt

Residency scheduling no longer impersonates a semantic application rebuild.
`FrameNeed::Residency` is a private presentation need, and a residency request
advances the presentation clock without setting public response invalidation.
Required coverage outranks layout, paint, and property traffic; proactive
coverage yields to that traffic. Any independent invalidation displaced by a
required coverage frame is retained for the following frame.

A residency-only frame rematerializes the installed native view, reconciles its
virtual-list materialization and interaction projections, and composes the new
resident layout without invoking the application's view closure. A selected
front is retired by matching epoch after successful submission regardless of
the public invalidation kind. Retirement authors exactly one newest-intent
continuation when newer coalesced input exists. The deterministic burst witness
selects one front, coalesces twelve offsets while it is in flight, selects one
continuation carrying the exact final offset, and observes zero additional
application-view rebuilds for text, table, and virtual-list payloads. A separate
table witness proves that required coverage advances ahead of a stale resident
row click while preserving the click's independent layout invalidation.

The user's native release verification closed the approximately 22-row stranded-
window witness from section 12. Neither ordinary fast scrolling nor drag
scrolling froze, and clicking the table was no longer needed to install a new
resident range. The generated receipts remain ignored build outputs; their
durable facts are:

- `control-gallery-500px-idle-1784308618227.txt`: 512 inputs and desired
  changes; one direct schedule plus 15 follow-ups equaled all 16 selections;
  511 requests coalesced; no supersession, preemption, cancellation, or virtual
  rejection occurred; candidate, GPU-submitted, and present-submitted serials
  converged at 64; maximum desired/resident lag was 3,001 logical pixels; and
  only four application-view rebuilds occurred across the entire startup and
  interaction session, not one per residency front.
- `control-gallery-500px-idle-1784308626646.txt` is cumulative after the drag
  run. Relative to the first receipt it adds 371 inputs, 344 desired changes,
  one direct schedule, 343 coalesced requests, 25 selections, and 24 follow-ups.
  Thus the drag portion also has exact front accounting. It adds no cancellation
  or rejection, and final candidate/GPU/present serials converge at 168 despite
  a maximum observed desired/resident lag of 11,630,919 logical pixels.

The same receipts preserve a distinct unresolved performance result. At the
240 Hz, 4,167 us refresh interval, their cumulative frame-interval p50 values
were 18,711 and 13,519 us; renderer deadline misses were 47/91 and 72/237;
presentation-layout p95 was 12,328 and 9,511 us. They recorded 19/47 layout
recompositions plus 64/211 layout-reuse preparations, and resident-window
advances still painted roughly 163--172 scene nodes, prepared about 100 text
items, and created roughly 256 GPU resources. The user describes the result as
substantial chug/chop. SE-007 closes liveness and application-view ownership;
it does not waive the SE-009 cadence, bounded-work, or resource-churn gates.

Verification at the stage boundary passed the 20 independent scrolling
oracles, all 57 residency-focused tests, and 1,407 library tests with four
intentional hardware ignores. The complete workspace all-target/all-feature
suite also passed three renderer-debug checks with 27 intentional hardware
ignores and two example tests; all 18 manifest/receipt/census Python checks
passed. Release 1.25x residency crossings retained exact complete-pixel
activation with three virtual-list provider calls, nine table cell calls, and
zero application-view rebuilds; the warm table smoke retained the frozen 528
property bytes with no content work or GPU-resource churn. SE-007 changes no
public scrolling name. The public break remains wholly owned by SE-008.

## 14. Resume protocol

At every task entrance and after every context compaction:

1. Read this document completely.
2. Run `git status --short --branch`, `git branch --show-current`, and compare
   `HEAD` with `origin/master`.
3. Preserve and attribute pre-existing changes.
4. Read the source census and select the first unmet stage exit.
5. Re-run the narrow oracle before editing production.
6. Keep the stage commit green, update this ledger, commit on `master`, and
   push before beginning the next boundary.
