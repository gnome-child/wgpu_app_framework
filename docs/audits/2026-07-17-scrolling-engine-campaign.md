# Scrolling engine campaign

Status: **SE-000 CLOSED — SE-001 NEXT**

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
| SE-001 — Green behavioral oracles | next | Independent models cover motion, sessions, handoff, sources, policy, reveal, mutation, anchoring, and accessibility; deliberate faulty adapters prove every witness. |
| SE-002 — Axis adjustment | queued | Eager horizontal and vertical scrolling use an internal adjustment with a wide continuous coordinate and no public break. |
| SE-003 — Sessions and nested handoff | queued | Fractional, diagonal, boundary, reversal, interruption, and child/ancestor remainder oracles pass per axis. |
| SE-004 — Container and eager adapter | queued | One ordinary eager widget exercises the full container contract. |
| SE-005 — Native text and list | queued | Eager viewport, text, and list share container behavior without a virtualization-shaped public abstraction. |
| SE-006 — List model/factory lifecycle | queued | Mutation touches affected ranges/bindings; realization is limited to entering items; identity and slot lifecycle are distinct. |
| SE-007 — Private residency/presentation | queued | Warm transform-only motion performs zero application-view rebuilds; every selected front retires or is superseded with one latest-intent continuation. |
| SE-008 — Names and public break | queued | Proved names replace old paths in one green migration, with no aliases or compatibility layer. |
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

## 6. Resume protocol

At every task entrance and after every context compaction:

1. Read this document completely.
2. Run `git status --short --branch`, `git branch --show-current`, and compare
   `HEAD` with `origin/master`.
3. Preserve and attribute pre-existing changes.
4. Read the source census and select the first unmet stage exit.
5. Re-run the narrow oracle before editing production.
6. Keep the stage commit green, update this ledger, commit on `master`, and
   push before beginning the next boundary.
