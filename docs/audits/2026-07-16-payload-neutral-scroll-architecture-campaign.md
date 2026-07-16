# Payload-neutral scrolling architecture audit and campaign

Status: **EXECUTION AUTHORIZED; SC-000 THROUGH SC-003 CLOSED; SC-004 READY**

Date: 2026-07-16

Repository: `C:\Users\Shea\projects\wgpu_l3`

Branch at audit: `codex/scroll-truth-campaign`
Audited HEAD: `d4909a8d` (`perf(text): share sparse horizontal edit checkpoints`)

This document is the durable context for the next scrolling campaign. It is deliberately self-contained so that a new agent can resume after context compaction without inheriting conclusions from chat history.

Execution was authorized on 2026-07-16 with the requirements to integrate all branch history into `master`, re-read this document after context compaction, and commit and push each closed campaign boundary directly to `master`.

The table is a test payload. It is not the architectural unit. A viewport containing table cells, rules, text, quads, groups, filters, or virtualized content must consume the same scroll state through the same spatial contract.

## 1. Field report and scope

Current observed behavior:

- Text-document scrolling is substantially better and now feels consistent with table scrolling.
- Disabling text wrapping produces a horizontal scrollbar at the correct content width.
- Character input in a large unwrapped document remains slow.
- Table rules can travel ahead of the rest of table geometry during a scroll.
- Horizontal table scrolling can leave geometry stationary until a later update snaps it into place.
- Table scrolling remains choppy beyond those correctness defects.

Scope of this audit:

- scrolling in general, independent of viewport payload;
- the new property-update path, retained scene model, presentation scheduling, input projection, and residency boundary;
- comparison with architectures used by Blink/Chromium, Firefox, WebKit, GTK, Qt, Iced, and COSMIC;
- a sequence of bounded, high-confidence work loops with explicit red/green witnesses and negative controls.

Out of scope for this pass:

- implementing a fix;
- closing the in-progress grouped-table transform work;
- staging, committing, or pushing the five inherited modified files;
- adopting a compositor thread, tiling, or a complete browser property-tree system without local evidence.

## 2. Provenance and inherited state

At audit time the worktree had five modified files, all inherited from the previous U-002 investigation:

- `src/diagnostics/mod.rs`
- `src/diagnostics/render.rs`
- `src/render/retained.rs`
- `src/scene/commit.rs`
- `tools/renderer_debug/src/main.rs`

The relevant unstaged hypothesis changes direct group-local content under an outer scroll to an identity scroll binding while preserving genuine nested-scroll deltas. The accompanying `renderer_group_under_scroll_pair` fixture contains an ungrouped rule, a grouped opaque quad, a 20-logical-pixel scroll tick, and a separately authored static expected commit.

That work is **not closed**. The fixture was not connected to the renderer harness at the start of this audit, and the required independent first-tick, per-shape, multi-scale, and negative-control witnesses have not run. Do not treat compilation, the structural unit test, or the legacy entering-strip test as closure.

An independent review found no immediate logic defect in the inherited group-binding correction: direct group-local content becomes identity-bound while genuine nested scrolling retains only the delta below the group root. The structural test passed, the legacy entering-strip test passed, release `table-scroll-work` reproduced 11,072 property-upload bytes at the four inherited scales, and the independent fixture remained dead code. This supports local coherence but does not upgrade the patch from hypothesis to proven production correction.

Prior completed evidence remains useful but is not a substitute for this campaign:

- The U-001 large-unwrapped-text edit work is pushed at `d4909a8d`.
- Exact 4 MiB edit benchmark p50 changed from 506 microseconds to 392 microseconds.
- The official 1,024-sample p50/p95/p99/max was 348/369/458/550 microseconds.
- Horizontal index residency in the recorded comparison fell from 7,566,468 bytes to 308,628 bytes.
- A synthetic 64 MiB edit witness requires all but four checkpoint blocks to remain shared.
- A separate 64 MiB cold glyph-admission path can consume roughly 24 GiB and page heavily. It remains open and must not be represented as solved by U-001.

The earlier campaign remains an evidence index at `docs/audits/2026-07-16-scroll-unification-campaign.md`. This document supersedes its architectural framing for general scrolling; it does not retroactively change its recorded receipts.

## 3. Executive verdict

The property-tree idea is applicable, but a full Blink clone is not the right starting prescription.

The repository has a real property snapshot and a retained renderer, but it does **not** currently have one explicit property/spatial tree comparable to Blink's paint property trees. Instead, it has a distributed implicit property graph:

1. interaction state owns requested and locally named “admitted” offsets;
2. layout stores scroll projections and a separate node-to-scroll-ancestry map;
3. scene commits encode scroll, clip, and group scopes in draw order plus a flat property topology;
4. candidate commits are rewritten into semantic/resident order, and compatibility scenes recursively interpret the scopes again;
5. the retained renderer reconstructs inherited scroll from node parents and translates several payload types through different paths;
6. hit testing independently traverses layout ancestry and reads the runtime's last present-submitted property stack.

Those representations can describe the same spatial fact differently. The grouped-table failure is one observed consequence: a group surface can be composited at scroll-translated bounds while a member is assigned an outer-scroll-relative binding that counter-translates it. Rules and grouped geometry then move on different ticks even though they share one viewport.

The smallest architecture that fits this renderer is:

- one immutable, explicit spatial topology compiled into each scene commit;
- typed parent relationships and coordinate-space boundaries for root, scroll, transform, and offscreen/effect surfaces;
- explicit clip and effect references where their parentage differs from the spatial parent;
- one normalized property-state binding per draw/content item;
- one scroll node referencing its interaction owner, owned axes, range/residency declaration, and property slot;
- stable property identities/slots attached to that topology, without presuming the separate delta-production and upload policy;
- one spatial compiler shared by candidate, semantic, drawable, compatibility, incremental, and direct representations;
- renderer-specific final application for quads, text, and surfaces, all consuming the same compiled spatial result;
- runtime-presented hit testing and input projection consuming the last present-submitted spatial snapshot or an evaluator generated from the same topology.

This is a property graph/tree in the useful sense. It need not begin as Blink's four fully independent trees, and it does not require a compositor thread. Split transform, clip, scroll, and effect ancestry only where the local scene model needs independent parentage. The invariant matters more than the number of structs: every visual item names one spatial state, and each ancestor contribution is applied exactly once.

### Recommended disposition: rewrite spatial semantics behind stable boundaries

The campaign should not optimize for preserving the current spatial/scroll-scope internals. S-A01, S-A02, S-A03, S-A08, S-A10, and the spatial-consumer portion of S-A07 share an ownership problem: the same ancestry, coordinate boundary, or scroll identity can be interpreted more than once. SC-002 and SC-008 should therefore be treated as a controlled replacement of spatial semantics.

That replacement is **not** the remedy for every scrolling symptom. Property delta production/upload (S-A04/S-A05), high-resolution input (S-A06), state-generation naming (the non-spatial portion of S-A07), pacing (S-A09), and cold residency (S-A11) are separate mechanisms with separate evidence, negative controls, and rollback boundaries. They may integrate with the spatial topology through stable identifiers, but must remain independently reversible and independently closable.

This is not a big-bang repository rewrite. Preserve the public interaction/widget behavior and validated layout/payload algorithms while replacing the internal path in vertical slices:

1. freeze independent external witnesses and receipts;
2. compile a new normalized spatial topology alongside the old path;
3. prove structural and pixel equivalence where the old behavior is correct, and prove intentional divergence with negative controls where it is not;
4. move candidate/semantic/drawable commit projection, compatibility output, retained planning, rendering, and input spatial projection to the new path one boundary at a time;
5. delete each old owner immediately after its consumers move.

Rewrite freedom does not relax closure. It raises the requirement for dual-path witnesses, bounded migration seams, source censuses, and deletion gates. If a current type or cache already has one clear owner and satisfies the constitution, it may be retained; no current internal abstraction is presumed sacred.

“One source of truth” does not mean one mutable global object. It means one owner per fact:

| Fact | Sole owner |
|---|---|
| Raw interaction intent and fractional remainder | interaction/input state |
| Legal range and resident acceptance | presented layout/residency model |
| Candidate visual property values | scene property snapshot |
| Coordinate ancestry and property bindings | immutable scene spatial topology |
| Last `present_submitted` property generation | surface/presentation receipt |
| GPU representation | renderer cache derived from the scene topology and snapshot |

The current word `admitted` spans more than one of these boundaries. Before implementation, the campaign must name and trace requested intent, coalesced desired intent, clamped value, resident-accepted value, candidate property snapshot, GPU submission, and `present_submitted` generations distinctly. Multiple requests may coalesce or be superseded before candidate construction. `present_submitted` means commands were submitted and `SurfaceTexture::present` was called; it does not claim scanout or successful human-visible display.

## 4. Current architecture trace

### 4.1 Input and interaction state

`src/platform/event.rs:276-289` converts both line and pixel wheel input directly to integer logical pixels. Pixel deltas are divided by scale and rounded per event. `src/interaction/scroll.rs` then stores integral `ScrollOffset` and `ScrollDelta` values in an interaction entry with `Admitted` or `Pending { admitted, desired }` position.

`src/runtime/input/dispatch.rs:211-260` requests the desired offset, resolves/clamps it through the presented layout, and either:

- calls `session.admit_scroll` immediately and requests a property tick when the presented resident window accepts it; or
- requests semantic/residency work when it does not.

This is a useful fast path. Its risk is contractual: interaction admission advances before candidate construction, queue submission, and the surface-present call, while scene documentation also uses admission language for a visual snapshot. The generations must be explicit enough that input, diagnostics, and recovery cannot mistake intent accepted by residency for the last present-submitted frame.

### 4.2 Layout and viewport projection

`src/layout/mod.rs` stores both `scroll_ancestries` and `ScrollProjection` records. A projection ties an interaction target to viewport geometry, layer bounds, and residency proof. `scroll_property_accepts` checks changed axes across projections; `resolve_scroll_offset` merges maximum offsets component-wise.

Ordinary scrolling, text, tables, and virtual lists all ultimately use viewport projections. A table currently combines a horizontal table viewport and a nested vertical virtual-list viewport that share one interaction target. This works by convention and component-wise merging. The ownership of each axis is not represented as a typed contract.

### 4.3 Scene commit and property snapshot

`src/scene/commit.rs` contains:

- a flat `property_topology: Vec<PropertyRef>`;
- property values and changed refs in `Properties`;
- ordered `Draw` operations for content, clips, groups, and scroll scopes;
- `ScrollDeclaration` geometry and residency bounds.

`Commit::semantic` calls `semantic_order` (`src/scene/commit.rs:742`) to independently rewrite clip/group/scroll scopes at the residency boundary. `Commit::compatibility_scene` is used by production popup and frame-realization paths; its recursive `compatibility_order_until` (`src/scene/commit.rs:545`) interprets groups and scrolls again. The campaign must therefore account for candidate, semantic/resident, drawable, and compatibility representations—not only retained planning.

`src/scene/paint/mod.rs:312` rebuilds the property value vector for a property tick and compares it with the prior values. It separately reconstructs scroll scope ordering from layout ancestry. This is a property snapshot, but the flat topology has no explicit parent links and content does not name a normalized spatial state.

### 4.4 Retained planning and rendering

`src/render/retained.rs` derives inherited scroll values by walking node parents. `TargetSpace`, `PropertyBinding`, and `ScrollBinding` then steer content into GPU node and scroll properties. Groups introduce coordinate-space boundaries and are composited by a separate encoder path.

There are two additional substantial draw-order interpreters in this file:

- bounded/incremental `PendingPlan::advance`;
- direct `PlanBuilder::build_order`.

Both interpret clip, group, and scroll pushes/pops and construct group-local target spaces. Production can exercise both paths. `src/render/renderer.rs` adds another runtime encoder state machine for scroll translation, clips, and groups. Together with `semantic_order` and `compatibility_order_until`, this is a minimum census of five production scope/spatial interpreters before counting the separate hit-test projection.

Payloads then consume spatial state differently:

- retained shapes use GPU scroll/node properties;
- text uses glyph viewport/render offsets and special group handling;
- panes, filters, and offscreen groups use encoder/surface translations;
- clips intentionally remain fixed or move according to their own scope.

Different final representations are unavoidable. Independently deriving ancestry and boundary semantics in each representation is not.

### 4.5 Presented input geometry

`PresentedGeometry::project_point` in `src/runtime/mod.rs:60-98` traverses layout scroll ancestry and reads the runtime's last present-submitted stack offsets to map input. Hit testing, scroll routing, context behavior, and drag logic use this projection. The existing type name `PresentedGeometry` is retained here only when referring to code.

This independently reconstructed transform can diverge from what the renderer submitted. A correct architecture either evaluates the same present-submitted spatial topology for both rendering and input or proves that both consumers are generated from the same normalized binding.

### 4.6 Frame pacing

`PresentationPulse` in `src/platform/runner/mod.rs:35-58` computes a software deadline as `last_presented_at + display_interval`. Both redraw issuance and handling can defer work until that deadline (`src/platform/runner/native.rs:128-157`, `src/platform/runner/handler.rs:61-83`). The clock is marked after the runtime reports a surface-present submission, not after scanout feedback.

This may be appropriate throttling, but it is not yet an evidence-backed explanation for choppiness. A completion-anchored software clock can phase against OS redraw/vsync behavior. The campaign must trace event-to-present-submitted timing and run a controlled pacing negative control before changing it.

## 5. Findings register

### S-A01 — The spatial source of truth is distributed (P0 correctness)

Scroll ancestry and coordinate boundaries are reconstructed in layout, scene ordering, retained planning, encoder state, and presented hit testing. There is no explicit content-to-spatial-state binding shared by all consumers.

Observed witness: grouped table geometry can remain stationary while ungrouped rules move, then snap after a semantic rebuild.

Required direction: compile one explicit spatial topology and bind every payload to it. Ancestor transforms must be applied once.

### S-A02 — Group/effect boundaries expose a double-application ambiguity (P0 correctness)

A group under a scroll is composited at translated group bounds while its direct local members can also receive an outer-scroll-relative binding. The inherited U-002 patch addresses this case locally, but its correctness has not been independently demonstrated and the architecture still permits recurrence in text, panes, filters, nested groups, or another planner.

Required direction: make surface-local roots explicit in the topology; no content path should infer that boundary from mutable planner state.

### S-A03 — Candidate, semantic, compatibility, direct, and incremental paths duplicate spatial semantics (P1 recurrence risk)

`semantic_order`, `compatibility_order_until`, `PendingPlan::advance`, `PlanBuilder::build_order`, and the runtime `PlanEncoder` all interpret some combination of scroll, group, clip, or coordinate-boundary semantics. `compatibility_scene` is exercised in production popup and frame-realization paths. A shared bug can make retained-versus-fresh image comparison false-green, while a one-path fix can make behavior depend on residency, compatibility projection, or planner selection.

Required direction: one normalized spatial topology owned by the candidate commit. Semantic/resident and drawable commits must carry a projected topology or a view generated from it; compatibility output and retained plans must be generated adapters. Bounded/incremental execution is a scheduling mode, not another spatial interpreter.

### S-A04 — Property-tick GPU work is not bounded by changed properties (P1 performance)

`prepare_node_properties` constructs the complete node-property byte vector and uploads the entire vector when any node property differs. A scroll tick also moves scrollbar chrome, so a one-offset change can induce a full node-property upload. `prepare_scroll_properties` already performs sparse entry writes, showing the desired pattern is feasible.

Live release receipt for `table-scroll-work` at scales 1.0, 1.25, 1.5, and 2.0 was identical:

```text
node_rebuilds=0
primitive_prepare_calls=0
text_prepare_calls=0
text_shape_calls=0
content_upload_bytes=0
property_upload_bytes=11072
gpu_resource_creations/replacements/removals=0/0/0
plan_rebuilds=0
plan_reuses=1
draw_calls=97
draw_passes=17
resource_transition_boundaries=16
```

This proves semantic zero work, but it does not prove property work is sparse. The 11,072-byte total needs category-level attribution before assigning causality.

Required direction: receipts split node, scroll, text viewport, clip/effect, and chrome property writes. Use a measured sparse/dense policy: sparse ticks write changed ranges, while full upload remains valid for initialization, buffer replacement, topology changes, or updates dense enough that one full transfer is cheaper.

### S-A05 — Property preparation still scans full topology on a tick (P1 scalability)

Scene property snapshot construction and retained node-property preparation iterate broad property/binding sets before discovering what changed. `Properties::snapshot` and projection/rebase paths repeatedly call `Properties::value`, which is a linear search, making change production potentially quadratic. `property_snapshot` also repeatedly scans commit nodes. At audit time `Properties::changed()` is observed by diagnostics/tests, not consumed by production rendering. Sparse GPU writes alone would therefore leave scene-wide or quadratic CPU work.

Required direction: assign a stable `PropertyIndex` within each topology revision, provide indexed O(1) value access, and have authoritative mutation/source boundaries produce dirty indices as values change or coalesce. Production preparation must consume those indices directly. Measure lookup/visit counts and bytes separately; add large unrelated-property and changed-density controls.

### S-A06 — High-resolution input is quantized before accumulation (P1 behavior/feel)

Pixel wheel deltas are converted to integer logical pixels per event. At fractional scales, small events can be discarded or delivered unevenly. This can create choppy input even if property presentation is perfect.

Required direction: preserve a fractional remainder at the input/interaction boundary and quantize only at the chosen visual-position contract. This does **not** pre-decide that all scene/GPU offsets must become fractional.

### S-A07 — “Admitted” conflates resident acceptance and present submission (P1 state correctness)

The interaction fast path calls `admit_scroll` when the current resident layout can support a requested offset, before a candidate property snapshot has been submitted and passed to `SurfaceTexture::present`. Diagnostics and recovery need unambiguous generations.

Required direction: adopt explicit terms and serials for requested/coalesced intent, clamped, resident-accepted, candidate, GPU-submitted, and `present_submitted` state. A request may be coalesced or superseded without receiving a candidate generation. No local state should be named “visible” because the platform provides no scanout feedback.

### S-A08 — Shared-target axis ownership is conventional, not typed (P1 maintainability)

A table's horizontal viewport and nested vertical viewport can share a target, and limits are merged component-wise. This is valid only while each projection's axis responsibility and clamping rules remain compatible.

Required direction: declare owned axes and composition policy on scroll/spatial nodes; reject conflicting owners; test diagonal, nested, one-axis-saturated, and programmatic updates.

### S-A09 — Frame cadence lacks a causal trace (P1 performance)

The software `PresentationPulse`, OS redraw requests, property schedules, surface acquisition, and presentation receipts are individually visible but not correlated by one interaction/frame serial. Choppiness cannot be assigned confidently to event coalescing, deadline phase, CPU preparation, GPU submission, or residency.

Required direction: one low-overhead timeline receipt with event, desired update, optional candidate generation, redraw request, redraw delivery, acquire, queue submit, and surface-present-call timestamps. Run pacing bypass only as a controlled negative test.

### S-A10 — Existing visual tests can share the defect (P0 test quality)

The legacy table entering-strip test can pass when only rules move. A retained-versus-fresh comparison can pass when both renderers use the same incorrect transform semantics.

Required direction: independent expected geometry, per-object old/new occupancy assertions, topology/payload permutation, multiple scales, and a demonstrated failing negative control.

### S-A11 — Large-text cold admission is a payload/residency defect, not a scroll exception (P1 performance)

The 64 MiB cold glyph-admission path can page heavily and consume roughly 24 GiB. It affects scrolling because scrolling crosses residency boundaries, but the solution belongs to bounded payload admission and shaping, not a text-specific scroll transform.

Required direction: retain it as a campaign rail under residency/locality; do not fork text scrolling semantics.

## 6. Industry comparison and local lessons

| System | Relevant architecture | Lesson for this repository |
|---|---|---|
| Blink/Chromium | Blink paint uses transform, clip, effect, and scroll property trees; paint chunks name property state. Chromium's compositor maintains property trees and can update scroll offsets without repainting content. | Adopt explicit parented property/spatial state and content bindings. Do not infer that a compositor thread or browser-scale tiling is required. |
| Firefox/WebRender/APZ | APZ maintains a hit-testing tree and WebRender scroll metadata grouping content that moves together; ordering and spatial relationships are explicit. | Rendering and hit testing need the same presented spatial relationships. Scroll-frame identity must not be duplicated by payload type. |
| WebKit | The scrolling tree owns async scroll behavior; the display tree explicitly separates scrolling container and scrolled contents so scrolling need not invalidate subtree geometry. | Model the viewport/container and moving contents as explicit nodes; group/effect boundaries are coordinate-space structure, not table behavior. |
| GTK 4 | `GtkScrollable` exposes horizontal/vertical `GtkAdjustment`; the same adjustment value controls the viewport and scrollbar. Snapshot rendering uses a stack of render nodes/transforms. | One adjustment/owner per axis and shared chrome/content value. GTK's snapshot model is guidance on ownership, not a retained-tree blueprint. |
| Qt | `QAbstractScrollArea` uses scrollbar values to drive one viewport; widgets render content according to those values. `QPlainTextEdit` uses visible-block geometry/content offsets for large text. | Keep scroll ownership at the viewport and payload layout local/bounded. Avoid geometry-specific scroll state. |
| Iced | `scrollable::State` supplies one translation used by event routing, operations, drawing, overlays, and mouse interaction around a clipped layer. | One state must feed all consumers. This small-toolkit pattern closely matches the local scale even though the renderer internals differ. |
| COSMIC | COSMIC builds on Iced and reuses its scrollable model/runtime. | COSMIC does not provide evidence for a separate table path; it reinforces reuse of a generic viewport contract. |

These comparisons support explicit topology, shared spatial state, and payload-neutral viewport ownership. They do not by themselves justify a property-upload policy, fractional-input representation, residency algorithm, or frame-pacing change; those tracks require local evidence.

Primary references:

- [Blink paint architecture and property trees](https://chromium.googlesource.com/chromium/src/%2Bshow/refs/heads/main/third_party/blink/renderer/platform/graphics/paint/README.md)
- [Chromium compositor architecture](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/how_cc_works.md)
- [Firefox Async Pan/Zoom architecture](https://firefox-source-docs.mozilla.org/gfx/AsyncPanZoom.html)
- [WebKit GTK/WPE graphics and scrolling tree](https://docs.webkit.org/Ports/WebKitGTK%20and%20WPE%20WebKit/Graphics.html)
- [WebKit display tree](https://docs.webkit.org/Deep%20Dive/Layout%20%26%20Rendering/DisplayTree.html)
- [GTK `Scrollable`](https://docs.gtk.org/gtk4/iface.Scrollable.html), [`Adjustment`](https://docs.gtk.org/gtk4/class.Adjustment.html), and [`Snapshot`](https://docs.gtk.org/gtk4/class.Snapshot.html)
- [Qt `QAbstractScrollArea`](https://doc.qt.io/qt-6/qabstractscrollarea.html) and [`QPlainTextEdit`](https://doc.qt.io/qt-6/qplaintextedit.html)
- [Iced scrollable source](https://github.com/iced-rs/iced/blob/master/widget/src/scrollable.rs)
- [COSMIC/Iced relationship](https://pop-os.github.io/libcosmic-book/) and [COSMIC scrollable API](https://pop-os.github.io/libcosmic/cosmic/iced/widget/scrollable/struct.Scrollable.html)

## 7. Campaign constitution

Every implementation checkpoint must preserve these invariants:

1. **Payload neutrality.** A viewport scrolls spatial content. Table, text, rule, quad, group, filter, and virtual-list identities do not change transform semantics.
2. **One fact, one owner.** Interaction intent, legal range, spatial ancestry, candidate values, and present-submitted values have distinct owners and generations.
3. **Exactly-once spatial application.** For every content item, each ancestor transform contributes once; a local surface root resets only the ancestry explicitly represented by that root.
4. **Explicit clipping.** Moving content and fixed viewport clips are distinct bindings; clip behavior never follows accidentally from draw order.
5. **Atomic submitted state.** Geometry, rules, text, clips, overlays, scrollbars, hit testing, and accessibility consume one `present_submitted` property generation. This is an internal atomicity claim, not a scanout claim.
6. **Cost-based warm ticks.** Resident scrolling performs no semantic rebuild, shaping, or payload upload. Property delta production is indexed and bounded by dirty producers; upload chooses measured sparse ranges or a dense/full transfer according to explicit cost and lifecycle rules.
7. **Bounded cold crossing.** Residency admission is bounded by viewport/guard policy and cannot scale to the whole document.
8. **Lossless input accumulation.** High-resolution input deltas are not discarded before accumulation; visual quantization is a named policy.
9. **Typed axes.** Axis ownership and composition are explicit and conflicts are rejected.
10. **Independent oracles.** Expected first-tick geometry is authored independently of production transform evaluation, and every oracle has a proven negative control.

## 8. Required test matrix

The campaign uses a common viewport fixture with replaceable payload. Table cases are mandatory but have no special oracle. No closure condition means the Cartesian product of every dimension below.

### Tier A — mandatory core transform cross-product

Run these eight fixture/topology pairs at scales **1.0, 1.25, 1.5, 1.75, and 2.0**: exactly **40 positive render cases**. Each case applies one first property tick of 20 logical pixels on each active axis. Twenty logical pixels maps to integral physical pixels at all five scales.

| Fixture | Payload and topology | Active axis |
|---|---|---|
| F01 | mixed opaque quad and independent rule under one scroll | horizontal |
| F02 | text and opaque quad under one scroll | vertical |
| F03 | grouped opaque quad plus ungrouped rule under one outer scroll (inherited defect) | horizontal |
| F04 | scroll and mixed text/quad beneath an offscreen/effect group | horizontal |
| F05 | fixed outer clip plus moving inner clip/content | horizontal |
| F06 | nested scroll and nested group containing text/virtual rows | vertical |
| F07 | table cells, rules, fills, and text under the generic viewport | horizontal |
| F08 | shared target with split horizontal/vertical nested viewports and table/virtual payload | diagonal |

Every positive core case must prove that:

- each moving object vacates a discriminating old-position sample;
- each moving object occupies a discriminating translated-position sample;
- fixed clips/chrome remain at discriminating fixed samples;
- the result matches a manually translated static expected commit;
- direct and incremental planning produce the same normalized spatial plan before pixels are compared.

The mandatory negative-control executions are exactly:

| Control | Execution |
|---|---|
| N01 missing translation | F01 at scale 1.0 |
| N02 inherited outer-scroll group binding | F03 at all five scales |
| N03 double surface translation | F04 at scale 1.25 |
| N04 incorrectly moving fixed clip | F05 at scale 1.5 |
| N05 rule-only movement with stationary grouped/table geometry | F07 at scale 2.0 |
| N06 intentionally divergent direct/incremental normalized plan | F06 at scale 1.75 |

This is **10 negative executions**, for a fixed Tier A total of **50 executions**. Each control must fail its intended assertion before the corrected behavior is accepted.

### Tier B — bounded pairwise behavior manifest

SC-000 must create and check in a deterministic pairwise manifest over the following values. The checked-in case IDs—not an implicit Cartesian product—become the closure authority.

| Dimension | Values |
|---|---|
| Fixture | F01-F08 plus repeated sibling scopes, empty payload, large unrelated-property scene, pane/filter surface |
| Input | pixel wheel/trackpad; line wheel; thumb drag; keyboard; caret reveal; programmatic absolute; residency-triggering request |
| Scale | 1.0; 1.25; 1.5; 1.75; 2.0 |
| Tick | first property; coalesced events; no-op; bound saturation; first after residency; next semantic rebuild |
| Direction state | forward; reverse; diagonal; one axis saturated |

Manifest rules:

- use a recorded deterministic generator/version and seed `20260716`;
- cover every value and every valid pair at least once;
- reject invalid combinations explicitly rather than silently dropping them;
- cap the manifest at 128 cases; if valid pair coverage exceeds that cap, amend this campaign with partitioned manifests before implementation;
- record the generated case count and manifest hash in E-001 or a successor receipt.

### Tier C — exact targeted suites

These suites test mechanisms that the spatial rewrite does not fix:

- **Property economics:** nine steady-state cases from property counts `{1, 256, 4096}` crossed with dirty densities `{one entry, 25 percent, 100 percent}`, plus initialization, buffer replacement, topology replacement, and coalesced repeated-write cases (13 total).
- **Input precision:** tiny-delta, reversal, and burst/coalescing pixel traces at all five scales (15), plus one trace each for line wheel, thumb, keyboard, reveal, and programmatic absolute input (20 total).
- **Generation/state:** coalesced requests, superseded request, no-op request, failed acquire, delayed redraw, resize, scale change, and residency race (8 total).
- **Pacing:** refresh rates `{60, 90, 120, 144}` crossed with steady, burst, and delayed-frame traces (12 total), each with the declared pacing negative control where applicable.
- **Residency:** payloads `{text, table, virtual list}` crossed with resident interior, guard edge, forward crossing, reverse crossing, and large jump at scale 1.0 (15), plus one fractional-scale crossing per payload (18 total).

Manual native coverage remains a named checklist rather than a combinatorial suite: text, table, virtual list, nested panes, wheel/trackpad, thumb drag, keyboard/reveal, resize, scale change, reversal/bounds, and cold residency crossing.

## 9. Work loops

Only one loop may be active at a time. A loop closes only with its evidence recorded here or in a linked closeout, formatter/tests passing at the declared scope, and a commit boundary. Do not begin the next loop in the same uncommitted diff.

### Campaign ledger

Update this table first whenever a loop changes state. `PENDING` means intentionally not started, not technically blocked.

| Loop | State at revision | Depends on | Durable note |
|---|---|---|---|
| SC-000 | CLOSED | — | Baseline, generation vocabulary, bounded trace, property attribution, source census, and deterministic Tier B manifest are recorded at the SC-000 boundary on `master`. |
| SC-001 | CLOSED | SC-000 vocabulary/receipts | Eight payload-neutral fixtures, 40 five-scale executions, direct/incremental plan equality, and all 10 negative controls are frozen. Twenty current executions are intentionally red and become SC-002 production gates. |
| SC-002 | CLOSED | SC-001 red oracle | One immutable candidate-owned spatial topology now supplies candidate, semantic/drawable, compatibility, retained-planning, shape, text, clip, pane, viewport, and surface adapters. Tier A is 40/40 green with all 10 negative controls preserved. |
| SC-003 | CLOSED | SC-000 property receipts | Stable property indices, dirty-source deltas, compiled dependents, shared sparse/dense transfer planning, and the exact 13-case economics suite are recorded. |
| SC-004 | READY | SC-000 generation trace | Independent state contract; separate resident acceptance from `present_submitted`. |
| SC-005 | PENDING | SC-000 input traces | Independent input-precision track; rounding remains a hypothesis pending sum-preservation controls. |
| SC-006 | PENDING | SC-000 causal trace | Independent pacing track; `PresentationPulse` remains a hypothesis pending negative controls. |
| SC-007 | PENDING | SC-000 residency receipts, SC-004 state contract | Independent residency track; includes the open 64 MiB cold glyph-admission defect. |
| SC-008 | PENDING | SC-002 topology, SC-004 terminology | Spatial track resumes to replace independent runtime-presented hit-test ancestry. |
| SC-009 | PENDING | SC-003/SC-007 | Preserve U-001 and add typing/scroll locality rails. |
| SC-010 | PENDING | SC-001 through SC-009 | Source deletion and bounded native closure. |

Initial evidence ledger:

| Evidence | State | Receipt |
|---|---|---|
| E-000 repository provenance | RECORDED | The only divergent campaign branch was a linear 33-commit descendant of `master`; it was fast-forwarded into `master`. Campaign formulation was pushed at `cd00554d`. The inherited U-002 correction and independent fixture remain uncommitted SC-001 provenance. |
| E-001 warm table property tick | RECORDED | Release `table-scroll-work`, scales 1.0/1.25/1.5/1.75/2.0: zero semantic/content preparation, zero resource churn, one plan reuse, 11,072 property-upload bytes split as node 11,008, scroll 32, text 32, viewport/unattributed 0. |
| E-002 grouped first-tick oracle | RECORDED | F03 compares the retained first property tick with a separately authored static commit, checks rule and grouped-quad old/new regions, and checks direct/incremental plan signatures at all five scales. The inherited correction passes all five; the actual legacy binding fails all five on grouped-quad translated occupancy. |
| E-003 unwrapped edit locality | RECORDED/PRIOR | Pushed `d4909a8d`; 4 MiB and synthetic 64 MiB sharing receipts are summarized in section 2. |
| E-004 cold glyph admission | OPEN | Roughly 24 GiB observed for a synthetic 64 MiB path; bounded scaling receipt still required. |
| E-005 independent campaign/patch review | RECORDED | Review found the inherited group binding locally coherent; its structural test and legacy visual test passed, release property bytes reproduced, and the unused fixture still emitted dead-code evidence. This does not close E-002. |
| E-006 bounded causal trace | RECORDED | `wgpu_l3.scroll_trace.v1` retains at most 32 transition records. Monotonic request serial ranges, target identity, request/clamp/resident values, outcome, candidate attempts, candidate/GPU/present-submitted serials, supersession, and latency are correlated by presentation epoch. One candidate selects the latest pending request per target; only earlier requests for the same target are superseded. |
| E-007 native scroll trace | RECORDED | Release control gallery, 136 px viewport, eight wheel events: six bounded records account for all requests through two coalesced pairs; five property transitions and three residency transitions all reach matching candidate/GPU/present-submitted serials. Frames 52/52, skipped 0; key-to-present-submitted p50/p95/p99/max 6,523/26,034/26,034/26,034 microseconds; unattributed property bytes 0. |
| E-008 deterministic Tier B manifest | RECORDED | Generator version 1, seed 20260716, 87 cases under the 128-case cap, explicit diagonal constraint, all valid pairs covered, cases SHA-256 `1265b5ec68cccbffbb042e5ee38fbb7e8f723932650971d0084f44368c2a660f`. |
| E-009 source census | RECORDED | `docs/audits/2026-07-16-scroll-source-census.md` records fact owners, six production spatial interpreters, payload adapters, property-write sites, scheduling, diagnostics, and repeatable census commands. |
| E-010 large-text rail | RECORDED | Current official release `text-horizontal-edit-4m` 64/1,024 receipt: p50/p95/p99/max 399/526/558/698 microseconds, 1,024 incremental updates, zero full index builds/full-width visits, maximum index residency 3,540,660 bytes, bounded 1,432 by 640 render window. The synthetic 64 MiB block-sharing test remains green; cold 64 MiB glyph admission remains E-004 open. |
| E-011 Tier A positive classification | RECORDED/RED | With the inherited F03 correction present, F01/F02/F03/F05 pass at 1.0/1.25/1.5/1.75/2.0 (20 green executions). F04/F06/F07/F08 fail at every scale (20 red executions), exposing grouped text, nested-group text, table text, and split-axis nested geometry respectively. SC-002 owns production green. |
| E-012 Tier A negative controls | RECORDED | N01, N03, N04, N05, N06 and N02 at all five scales produce exactly 10 failing mutations for missing translation, legacy group binding, double surface translation, moving fixed geometry, rule-only movement, and direct/incremental plan divergence. Each fails its named assertion. |
| E-013 direct/incremental plan equality | RECORDED | Every Tier A fixture serializes the complete direct and bounded retained plan structures, including nested batches, geometry, resource/property bindings, and surface-sampling choice, before pixel comparison. No current positive failure is caused by direct/incremental divergence. |
| E-014 static scroll-clip correction | RECORDED/SUPERSEDES PART OF E-011 | SC-002 preflight found that F07/F08 static expected commits omitted the fixed viewport clips introduced by production scroll scopes, and F08's first text probe began outside its initial clip. Adding independently authored fixed clips and moving the probe inside legal initial geometry makes F07/F08 green at all five scales. The corrected matrix is 30 green/10 red; only F04/F06 grouped text remains red. This invalidates E-011's inference that table text or split-axis geometry was defective, without changing its recorded first execution. |
| E-015 explicit spatial topology | RECORDED/CLOSED | Every commit compiles a typed root/transform/scroll/surface topology with clip/effect references, content property states, scroll-target identity, and typed axis ownership. Release Tier A passes 40/40 across five scales and all 10 mutation controls remain discriminating. Direct and bounded plans execute the same resumable compiler; compatibility payload geometry matches independently authored static commits; repeated sibling, empty-payload, and filter-surface topology tests pass. |
| E-016 post-topology warm table receipt | RECORDED | Release `table-scroll-work` is identical at scales 1.0/1.25/1.5/1.75/2.0: zero semantic/content preparation, shaping, payload upload, resource churn, and plan rebuilds; one plan reuse; 10,656 property bytes split as node 10,496, scroll 32, text 128, viewport/unattributed 0. This is 416 bytes below E-001 but remains SC-003 input, not a budget. |
| E-017 spatial-owner deletion gate | RECORDED | The old commit-local semantic and compatibility interpreters, direct planner, mutable planner scroll ancestry, baseline-content-union surface bounds, and encoder scroll-offset reconstruction are deleted. The source census and architecture test require `SpatialTopology` to remain the sole candidate spatial compiler. Runtime-presented point projection remains explicitly owned by SC-008. |
| E-018 indexed property economics | RECORDED/CLOSED | The release negative control wrote the complete node-property vector at every density: 256 bytes for 1 property, 65,536 for 256, and 1,048,576 for 4,096. The final exact 13-case suite makes one dirty property cost 64 bytes at both 256 and 4,096 properties, reports two value visits/lookups, chooses sparse quarter-density transfers and dense full-density transfers, coalesces repeated writes to one dirty index, and names initialization/buffer/topology/dense full-transfer reasons. |
| E-019 post-index warm table receipt | RECORDED | Release `table-scroll-work` is identical at all five scales: one dirty index, seven value visits/lookups, two write ranges, zero full-transfer reasons, and 464 property bytes split as node 64, scroll 272, text 128, viewport/unattributed 0. Semantic/content/shaping work, resource churn, and plan rebuilds remain zero; one retained plan is reused. |

Append evidence; do not silently rewrite a failed receipt. When superseding a conclusion, add the new receipt and identify which prior inference it invalidates.

### SC-000 — Freeze baseline, names, and observability

Goal: make the campaign falsifiable before changing semantics.

Work:

- Recheck HEAD, branch, worktree provenance, and inherited U-002 diff.
- Define serials/terms for input event, requested intent, coalesced desired intent, clamped value, resident-accepted value, optional candidate property snapshot, GPU-submitted frame, and `present_submitted` frame.
- Extend receipts in a diagnostic-only boundary to correlate those stages and split property CPU/write categories.
- Record the current table-scroll-work receipt, native traces, and large-text baselines.
- Add a source census for every read/write of scroll offset, ancestry, clip, and group-local translation.
- Generate and check in the deterministic Tier B pairwise manifest, recording its generator version, seed, case count, and hash.

Closure:

- One event-to-`present_submitted` receipt can explain a frame without relying on log order or ambiguous “admitted” labels, including when requests are coalesced or superseded.
- Existing behavior and resource counts are captured at the five required scales.
- Diagnostics themselves perform no semantic work and have bounded overhead.
- The Tier B manifest satisfies its bounded deterministic coverage rules.

#### SC-000 closeout — falsifiable baseline and vocabulary

The generation vocabulary frozen at this boundary is:

| Term | Receipt representation | Meaning |
|---|---|---|
| Scroll request | `first_request_serial` through `last_request_serial`, `target_key` | Monotonic diagnostic identity for one request or an epoch-coalesced range of requests to one interaction target. It does not claim candidate selection. |
| Coalesced desired intent | `coalesced_inputs`, `requested_x/y` | Latest desired offset for that target within the recorded request epoch; the serial range preserves how many requests contributed. |
| Clamped value | `clamped_x/y` | Offset resolved against the current legal range. |
| Resident-accepted value | `resident_offset_x/y`, `resident_accepted`, `outcome` | Value supported by the current presented residency declaration. `property-tick` means the warm path accepted it; `needs-residency` preserves desired intent while the prior resident value remains authoritative. |
| Candidate property snapshot | `candidate_epoch`, `candidate_attempts`, `candidate_property_serial` | Optional frame selection. Repeated attempts at one epoch update the recorded candidate serial; requests may be superseded before this stage. |
| GPU-submitted frame | `gpu_submitted_property_serial` | Property snapshot whose command buffers were successfully submitted. |
| Present-submitted frame | `present_submitted_property_serial`, `input_to_present_submitted_us` | The same submitted property snapshot after the surface present call. It is not scanout feedback. |

The bounded trace keeps 32 records and selects the latest pending request independently for every target represented by one candidate. Older pending requests are marked superseded only by a later request for the same target. Unit witnesses cover single-target correlation, request ranges, coalescing, same-target supersession, multiple targets selected into one candidate, a residency request linked to a later semantic epoch, repeated candidate attempts, unchanged input isolation, and the storage bound.

The release `table-scroll-work` receipt is identical at scales 1.0, 1.25, 1.5, 1.75, and 2.0: zero node rebuilds, primitive/text preparation, text shaping, content upload, resource churn, and plan rebuilds; one plan reuse; 11,072 property bytes split as node 11,008, scroll 32, text 32, viewport/unattributed 0. This records the full-node upload as an SC-003 baseline rather than accepting it as a warm-tick budget.

The final native 136 px control-gallery trace recorded eight wheel requests as six records with two coalesced ranges, five property transitions, and three residency transitions. All selected records report matching candidate, GPU-submitted, and present-submitted property serials; frames attempted/presented/skipped were 52/52/0; key-to-present-submitted p50/p95/p99/max was 6,523/26,034/26,034/26,034 microseconds. Aggregate property attribution was viewport 16, node 450,560, scroll 928, text 944, unattributed 0. Manual pauses and residency work make this an observability witness, not an SC-006 cadence conclusion.

The checked-in Tier B manifest is generated by version 1 with seed 20260716. It contains 87 cases, explicitly rejects 2,310 invalid diagonal combinations, covers every valid pair, remains under the 128-case cap, and has cases SHA-256 `1265b5ec68cccbffbb042e5ee38fbb7e8f723932650971d0084f44368c2a660f`. The generator check and three manifest invariants are executable in `tools/test_scroll_pairwise_manifest.py`.

Large-text locality remains a campaign rail: the current official release 64-warmup/1,024-sample 4 MiB edit receipt reports p50/p95/p99/max 399/526/558/698 microseconds, 1,024 incremental updates, zero full index builds or full-width source visits, 3,540,660 maximum index residency, and a bounded 1,432 by 640 render window. The synthetic 64 MiB sharing witness passes. The separate roughly 24 GiB cold glyph-admission path remains open; it was deliberately not executed or represented as fixed here.

The durable source census is `docs/audits/2026-07-16-scroll-source-census.md`; the deterministic manifest is `docs/audits/fixtures/scroll-pairwise-manifest-v1.json`. This loop changes diagnostics and attribution only. It does not close the inherited group-binding hypothesis, change scroll semantics, or claim a pacing cause.

Verification at the isolated boundary: formatter and diff checks passed; the manifest regeneration check passed; 18 Python manifest/receipt/census tests passed; and `cargo test --workspace --all-targets --all-features` passed with 1,244 library tests, three renderer-debug tests, and two example tests, with the existing hardware-dependent tests ignored. The first broad execution exposed one failed architecture assertion because it searched only `diagnostics/render.rs` for the scroll-owned schema. The assertion was corrected to require the schema in `diagnostics/scroll.rs` and its assembly through `diagnostics/mod.rs`; the targeted test and complete suite then passed. The inherited unused SC-001 fixture remains the only new dead-code warning at this boundary.

### SC-001 — Build the independent payload-neutral first-tick oracle

Goal: turn transform unity into a generic correctness gate.

Work:

- Connect the inherited grouped-rule/quad fixture without making it table-specific.
- Generalize fixture construction across the payload/topology matrix.
- Compare a property transition with a separately authored static expected commit.
- Add object-specific old/new/fixed sample assertions.
- Add normalized-plan equality between direct and incremental construction.
- Demonstrate negative controls for missing translation, double translation, wrong clip movement, and the inherited old group binding.

Closure:

- The current defect fails for the intended reason.
- All 40 Tier A positive cases obey the same oracle.
- A rule moving alone cannot make the witness green.
- All 10 specified Tier A negative executions fail the intended assertion.
- This loop may add test/diagnostic structure but must not claim the production fix.

#### SC-001 closeout — independent oracle frozen with production reds

SC-001 closes the oracle, not the spatial implementation. “All 40 positive cases obey the same oracle” means every case is constructed and executed through the same independently authored static-geometry, per-object occupancy, fixed-region, full-image, and direct/incremental-plan contract. Requiring all 40 to be production-green inside this test-only loop would contradict the prohibition on claiming or adding the spatial fix here. SC-002 retains the original all-green gate before spatial migration can close.

The eight fixtures are payload/topology cases, not table branches:

| Case | Current five-scale result with inherited F03 hypothesis | First red object/region when red |
|---|---|---|
| F01 mixed quad/rule under one horizontal scroll | 5/5 green | — |
| F02 text/quad under one vertical scroll | 5/5 green | — |
| F03 grouped quad plus ungrouped rule under outer scroll | 5/5 green | — |
| F04 text/quad in a scroll beneath an offscreen group | 0/5 red | grouped text translated occupancy; mismatched pixels 76/116/147/199/262 |
| F05 fixed outer clip/chrome plus moving inner clip/content | 5/5 green | — |
| F06 nested scroll/group with text and virtual-row payload | 0/5 red | nested-group text translated occupancy; mismatched pixels 173/253/328/433/558 |
| F07 table fill/rule/text under the generic viewport | 0/5 red | table text old occupancy; mismatched pixels 12/15/36/42/48 |
| F08 split-axis nested viewports with diagonal payload | 0/5 red | diagonal rule old occupancy; mismatched pixels 23/41/53/65/92 |

Counts are ordered by scales 1.0/1.25/1.5/1.75/2.0 and identify the fail-fast discriminating region, not total image differences. F01/F02 prove ordinary horizontal/vertical payload motion; F03 isolates the inherited surface-root defect; F05 proves the fixed/moving clip fixture; F04/F06/F07/F08 establish broader red gates for SC-002. Table is only F07, and its oracle is identical to every other case.

E-014 supersedes the F07/F08 production inference in this initial table. Production scroll scopes apply fixed viewport clips; their static expected commits initially omitted those independent clips, and F08's text probe began outside the initial viewport. After authoring the same fixed clip geometry explicitly (without using production scroll evaluation), F07 and F08 pass every scale. The corrected SC-002 entry matrix is F01/F02/F03/F05/F07/F08 green at all five scales and F04/F06 red at all five scales. The red mechanism is therefore text below a group/surface root, not table or split-axis identity.

The actual legacy `PropertyBinding::scroll` behavior was temporarily rebuilt and executed for F03 at all five scales. It failed only the grouped quad's translated occupancy with 384/600/864/1,176/1,536 mismatched pixels while the independently drawn rule passed. Restoring the inherited identity binding made F03 pass all scales. This proves the correction's narrow effect without treating it as the final architecture.

All 10 declared negative executions pass as negative controls: N01 missing translation; N02 legacy group binding at five scales; N03 double surface translation; N04 moving fixed geometry; N05 rule-only movement; and N06 deliberately divergent direct/incremental plans. The mutation suite first proves the static expected image satisfies the positive oracle, then introduces exactly one named violation and requires the intended assertion text. Rule-only movement therefore cannot green F07.

Direct and incremental retained plans are compared before pixels using complete recursive debug signatures. All eight fixture plans agree at every tested scale; the N06 signature mutation fails. The four production-red cases are consequently transform/application failures shared after plan construction, not evidence of direct/incremental plan drift.

The SC-001 boundary contains fixture, oracle, signature, CLI, and ignored hardware-test structure only. The inherited production identity binding and its structural unit test remain separate working-tree provenance for SC-002; this closeout does not claim that local correction as the complete spatial architecture.

Verification with the inherited hypothesis present: release `group-scroll-oracle` passed all five scales; release `tier-a-negative-controls` passed all 10 executions; the five release Tier A audits reproduced the exact 20-green/20-red classification above; the two explicit ignored GPU tests for F03 and the negative suite passed; formatter and diff checks passed; and `cargo test --workspace --all-targets --all-features` passed with 1,245 library tests, three non-hardware renderer-debug tests, and two example tests. After removing the production hypothesis from the staged boundary, the release negative-control test still passed and the complete suite passed with 1,244 library tests. The 21 renderer GPU tests remain opt-in by design, with the SC-001 witnesses executed explicitly at release profile.

### SC-002 — Replace spatial semantics with one explicit topology

Goal: remove distributed transform ancestry from production planning.

Candidate model, subject to implementation evidence:

```text
SpatialTopology
  SpatialNode { id, parent, kind: Root | Transform | Scroll | SurfaceRoot }
  ClipNode    { id, parent/spatial_binding, geometry }
  EffectNode  { id, parent/spatial_binding, kind }
  PropertyState { spatial, clip, effect }
  ContentBinding { content, property_state }
```

Work:

- Compile normalized parented topology and content bindings in the scene commit.
- Introduce the new topology behind a narrow compile/evaluate boundary so old and new plans can be compared without allowing both to remain long-term owners.
- Represent group/effect local roots explicitly.
- Make scroll-node axis ownership and interaction-target identity explicit.
- Define the candidate commit as the topology owner. Generate semantic/resident and drawable topology views from it rather than rewriting scope order independently.
- Generate `compatibility_scene` output from normalized bindings; migrate production popup and frame-realization consumers or retain a generated adapter with no independent ancestry logic.
- Replace duplicated direct/incremental spatial interpretation with one plan compiler; bounded work becomes resumable execution of that compiler.
- Enumerate `semantic_order`, `compatibility_order_until`, `PendingPlan::advance`, `PlanBuilder::build_order`, and `PlanEncoder` in migration/deletion gates.
- Keep compatibility adapters only while their consumers and removal loop are explicit.

Closure:

- All Tier A and the spatial cases in the checked-in Tier B manifest are green.
- Direct and incremental plan structures are identical.
- Candidate, semantic/resident, drawable, and compatibility outputs either carry the normalized topology or are generated from it without reinterpreting draw scopes.
- No content path infers an outer scroll from mutable group planner state.
- Architecture tests reject a second authoritative spatial ancestry owner/interpreter, including in compatibility and residency paths.

#### SC-002 closeout — candidate-owned spatial topology replaces distributed ancestry

SC-002 closes the renderer-side spatial ownership replacement. `Commit::from_parts` now compiles and immutably owns one `SpatialTopology` for every candidate or projected commit. The topology contains typed root, transform, scroll, and surface-root nodes; independent clip and effect references; a normalized property state for every draw/content binding; explicit scroll-target identity; and per-axis ownership with conflict rejection. A split-axis target remains legal, while two nodes claiming the same target axis fail compilation.

The original grouped-table symptom was not table behavior. A surface below an outer scroll was composited in parent scroll space while direct surface-local members could also inherit the outer scroll, and the inferred group allocation could shrink to the baseline content union. Grouped text then left that stale allocation after the first property update and disappeared or lagged while separately drawn rules moved. Direct surface-local bindings now begin at an explicit surface root, genuine nested scrolls below that root retain their local delta, and declared stable group/effect bounds replace baseline-content-union inference. This is the same contract for text, quads, rules, table payload, virtual rows, panes, clips, and filters.

The migration deleted the alternate production ancestry owners rather than preserving a permanent dual path:

- semantic/resident order projection and compatibility emission are methods of the candidate topology owner;
- direct and bounded retained planning both execute `PendingPlan::advance`, with bounded work only changing scheduling;
- mutable `TargetSpace` scroll ancestry, `PlanBuilder::build_order`, `project_order_group_bounds`, and renderer `scroll_translation`/`scroll_offset` reconstruction are gone;
- shapes, retained text, groups, panes, clips, scroll viewports, and sampled surfaces consume compiled `SpatialBinding` values;
- scene painting no longer emits an own-scroll fragment when its layout projection has no drawable residency, so a dangling scroll scope cannot rely on renderer no-op behavior.

The independently authored release oracle changed from F04/F06 red at all five scales to all eight fixtures green at scales 1.0, 1.25, 1.5, 1.75, and 2.0: 40/40 executions. All 10 negative executions still fail their intended missing, legacy, double, fixed-clip, rule-only, or plan-divergence assertion. The actual legacy F03 binding remains proven red at every scale by N02. Direct and incremental recursive signatures agree before pixels are compared. A separate compatibility test recursively compares payload kind/geometry against each static expected commit, ignores only representational clip wrappers, and requires every independent moving probe to occupy translated payload geometry.

The Tier B first-property spatial rows P003, P012, P015, P019, P031, P052, P058, P061, P065, and P080 are geometrically subsumed by the stronger five-scale Tier A execution. P026, P032, and P057 add the pane/filter-surface, empty-payload, and repeated-sibling first-property fixture values; topology tests require stable filter roots, root-only empty commits, and one interned spatial node for repeated logical sibling scopes. Their input production, state-generation, and residency dimensions remain in SC-004, SC-005, and SC-007 rather than being falsely claimed by a transform test.

The post-migration release `table-scroll-work` receipt is identical at all five scales: zero node rebuilds, primitive/text preparation, text shaping, content upload, resource creation/replacement/removal, and plan rebuilds; one plan reuse; 93 draw calls in 17 passes; and 10,656 property-upload bytes split as node 10,496, scroll 32, text 128, viewport/unattributed 0. The 416-byte reduction from SC-000 is recorded without treating a full-vector warm upload as acceptable; SC-003 owns indexed dirty production and sparse/dense transfer policy.

This is a local property/spatial tree, not a claim that Blink's complete affine paint-property architecture or compositor threading was copied. Transform nodes are typed in the topology, while renderer payload adapters still apply their final transform representation. Runtime `PresentedGeometry::project_point` also still consumes layout ancestry and remains the deliberate SC-008 boundary. Those limits prevent SC-002 from absorbing property economics, generation state, input precision, pacing, residency, or present-submitted hit testing.

Verification at this boundary: release `tier-a-scroll-oracle` passed all five scales; release `tier-a-negative-controls` passed all 10 executions; release `table-scroll-work` reproduced E-016 at all five scales; the compatibility, repeated-sibling, empty-payload, filter-surface, surface-root, shared-axis, and source-architecture tests passed; the pairwise manifest regeneration test passed; formatter and diff checks passed; and the complete all-target/all-feature suite passed with 1,252 library tests plus four ignored hardware tests, three renderer-debug tests, and two example tests.

### SC-003 — Index property deltas and choose sparse/dense updates

Goal: make property delta production and transfer proportional to authoritative dirty sources, while retaining full transfers when lifecycle or density makes them correct.

Work:

- Assign each declared property a stable `PropertyIndex` within a topology revision and maintain O(1) `PropertyRef`-to-index/value access.
- Move dirty production to authoritative sources for scroll, scrollbar/chrome, caret, transform, clip/effect, and other mutable properties. Coalesced writes update one index and preserve old/new comparison without scanning every property.
- Make `Properties::snapshot`, projection, rebase, and activation consume indexed values and dirty sets; remove repeated linear `Properties::value` lookup from warm paths.
- Make production renderer preparation consume dirty indices/generations rather than recomputing change from complete binding arrays.
- Split/range-write node/chrome properties for sparse ticks and define a measured sparse/dense threshold.
- Preserve full transfer for initialization, buffer replacement, topology revision, and dense changes where it is cheaper; record the chosen reason in diagnostics.
- Run the 13-case Tier C property-economics suite, including a 4,096-property unrelated-scene control.

Closure:

- Warm resident scroll reports zero semantic, shape preparation, text shaping, and payload upload work.
- Sparse-tick lookup/visit counts and uploaded bytes are bounded by dirty entries/ranges, independent of unrelated property count.
- Initialization, replacement, topology-change, and dense cases select an explicit full-transfer reason and remain within their separate budgets.
- Production rendering consumes indexed dirty information; `changed()` is not diagnostics-only.
- Resource create/replace/remove counts remain zero.
- The visual oracle remains green on the first tick.

#### SC-003 closeout — indexed dirty production and one transfer policy

SC-003 closes the property-economics boundary without changing scroll geometry, input precision, generation terminology, pacing, or residency policy. Every property declared by a commit now receives a stable `PropertyIndex` in topology order. `Commit` owns O(1) node and property maps, and `Properties` stores canonical values in immutable 256-entry blocks. A local update validates and coalesces writes by index, copies only touched blocks, preserves untouched blocks by `Arc`, and emits sorted dirty indices. Full snapshot, semantic projection, activation rebase, and drawable-revision paths retain complete-value validation while replacing their former repeated linear value lookup with indexed access.

Dirty production now starts at the authoritative mutable sources used by current property ticks. Scroll intent/resident changes carry per-target source revisions; scrollbar and caret visuals compare against the prior per-window property baseline. Scene painting emits only dirty scroll/chrome/caret values when the commit topology is compatible, applies them through `Properties::apply_updates`, and commits its observed source ledger only after a valid property snapshot is produced. Removed windows clear the retained visual baseline. Transform, clip/effect, and other complete lifecycle paths remain validated full snapshots until they acquire an authoritative mutable producer; no renderer-side comparison is presented as source truth.

Retained plans precompute property-index-to-node-binding and property-index-to-scroll-path dependents. Production preparation consumes `Properties::changed()` directly. Scroll paths are compiled and interned by `SpatialTopology`; transform-only geometry shares the identity path instead of allocating one apparent scroll uniform per transform. Node and scroll buffers use one `plan_property_transfer` policy: contiguous dirty binding slots become ranges, and sparse cost is payload bytes plus one aligned property stride per queue write. Sparse is selected only when that cost is below the complete buffer; otherwise the transfer is dense. Initialization, buffer replacement, topology/viewport replacement, and cost-selected density remain explicit full-transfer reasons in renderer and diagnostic receipts.

The recorded release negative control predates the indexed transfer change: all dirty densities wrote 256 bytes at one property, 65,536 bytes at 256 properties, and 1,048,576 bytes at 4,096 properties. The final steady-state matrix is:

| Properties | One dirty | 25 percent dirty | 100 percent dirty |
|---:|---:|---:|---:|
| 1 | 256 bytes, dense | 256 bytes, dense | 256 bytes, dense |
| 256 | 64 bytes, sparse | 16,192 bytes, sparse | 65,536 bytes, dense |
| 4,096 | 64 bytes, sparse | 261,952 bytes, sparse | 1,048,576 bytes, dense |

One dirty entry reports exactly two value visits and two index lookups at both 256 and 4,096 properties: one source update and one dependent renderer evaluation. Quarter-density and full-density cases report twice the dirty count for the same reason. The four lifecycle cases independently report initialization (256 bytes), buffer replacement (1,048,576 bytes), topology replacement (512 bytes), and three coalesced writes plus the final renderer evaluation (one dirty index, four visits/lookups, 64 bytes). `property-economics` asserts all 13 cases rather than printing an observational benchmark.

The production table remains only a payload witness. At every required scale its first horizontal property tick now reports 464 bytes: 64 node bytes for the dirty chrome binding, a 272-byte cost-selected range spanning two dependent scroll-path slots, and 128 bytes of changed retained text transform offsets. It reports one dirty property index, seven total value visits/lookups, two GPU write ranges, no full-transfer reason, no semantic/node/content/text preparation or shaping, no payload upload, no GPU resource creation/replacement/removal, no plan rebuild, and one plan reuse. E-019 supersedes E-016 as the warm property budget while retaining E-016 as the failed full-vector baseline.

The text transform adapter continues to visit retained text batches and writes only changed snapped offsets; that traversal is payload residency/locality work rather than a scan of unrelated property topology. SC-007 and SC-009 retain the bounded residency and large-document locality rails. SC-004 must still prove coalesced/superseded/failed presentation state and may strengthen renderer resynchronization across skipped candidate generations; SC-003 does not rename resident acceptance or claim present submission.

Verification at this boundary: release `property-economics` passed all 13 exact cases; release Tier A passed 40/40 first-tick executions and all 10 mutation controls; release `table-scroll-work` reproduced E-019 at all five scales; formatter, diff, all-target/all-feature check, targeted default-feature source-revision/index/block-sharing/transfer/spatial tests, and 18 Python census/receipt/manifest tests passed. The complete `cargo test --workspace --all-targets --all-features` run passed 1,258 of 1,262 library tests with four intentional hardware ignores, three renderer-debug non-hardware tests with 22 hardware ignores, and two example tests. The broad run first exposed two stale source-architecture patterns—renderer-only synthetic identity and the old direct scrollbar accessor—and those gates were updated to preserve their original invariants through test-layout identity and indexed property lookup before the suite was rerun green.

### SC-004 — Separate intent, residency acceptance, and present submission

Goal: make state transitions atomic and recoverable.

Work:

- Apply the SC-000 generation vocabulary to interaction, scene, renderer, diagnostics, and receipts.
- Preserve pending desired intent while clamping candidate visual state to legal/resident bounds.
- Permit requests to coalesce or be superseded without requiring one candidate per request.
- Advance `present_submitted` only after queue submission and the `SurfaceTexture::present` call; do not name it visible or scanout-complete.
- Define behavior for failed acquire, superseded candidate, resize, scale change, and semantic rebuild racing a property tick.
- Type axis ownership and composition for shared targets.

Closure:

- Deterministic tests cover coalescing, failed/late presentation, resize, scale change, saturation, shared axes, and residency crossing.
- Input and chrome cannot observe a generation newer than the last present-submitted content they describe.
- No layer uses `admitted` for two different transition boundaries.
- The eight-case Tier C generation/state suite passes and explicitly covers coalesced and superseded requests.

### SC-005 — Preserve high-resolution input

Goal: determine and fix input loss independently of rendering and pacing policy.

Work:

- Preserve fractional pixel/line accumulation through the interaction owner; choose and document visual quantization.
- Run the 20-case Tier C input-precision suite with tiny deltas, reversals, bursts, coalescing, and non-wheel input paths.
- Prove the existing per-event rounding behavior as a negative control before changing it.
- Keep scene/GPU offsets integral unless separate evidence requires a broader coordinate change.

Closure:

- Sum-preservation tests prove no fractional input is lost across an event sequence at all five scales.
- Reversal and coalescing preserve sign and total without drift or staircase artifacts beyond the named quantization policy.
- The fix is independently revertible and does not alter scheduler or spatial-topology policy.

### SC-006 — Audit and correct presentation cadence

Goal: establish whether scheduling contributes to choppiness and change it only with causal evidence.

Work:

- Correlate event, optional candidate, property preparation, redraw request/delivery, acquire, queue submit, `SurfaceTexture::present`, and present-submitted receipt timing.
- Run the 12-case Tier C pacing suite at 60/90/120/144 Hz.
- Compare the completion-anchored `PresentationPulse` with a controlled test-only bypass or platform-aligned policy.
- Measure p50/p95/p99 event-to-present-submitted latency, missed frame opportunities, no-progress redraws, CPU stage time, and submission cadence.

Closure:

- Cadence conclusions are backed by correlated traces and a demonstrated negative control, not subjective feel alone.
- The selected policy behaves at 60/90/120/144 Hz and under delayed frames without busy polling or duplicate redraws.
- The pacing change is independently revertible and does not depend on input-precision, residency, or spatial-topology changes.

### SC-007 — Bound residency crossings for every payload

Goal: keep a property scroll cheap until a deliberate, bounded residency handoff.

Work:

- Exercise text, table, and virtual-list resident interior, guard edge, forward crossing, reverse crossing, and large jump.
- Attribute layout, shaping, glyph admission, primitive preparation, and uploads to the crossing generation.
- Close the 64 MiB cold glyph-admission memory explosion with a bounded viewport/guard policy.
- Ensure a residency rebuild is submitted atomically with the selected/coalesced scroll generation and never causes a later geometry snap.
- Run the 18-case Tier C residency suite.

Closure:

- Interior ticks remain property-only.
- Crossing work is bounded by viewport plus declared guards, not document length.
- Peak memory and latency have explicit budgets and multi-size scaling receipts.
- Text/table/virtual payloads use one residency contract even if their admission algorithms differ.

### SC-008 — Unify present-submitted geometry consumers

Goal: make interaction target the geometry from the runtime's last present-submitted frame.

Work:

- Replace independent layout-ancestry projection in hit testing with the present-submitted spatial snapshot/evaluator.
- Audit pointer, drag, selection, caret reveal, IME, context menus, overlays, accessibility bounds, and scrollbar thumb behavior.
- Test stale/superseded candidate versus present-submitted generations and nested/grouped payloads.

Closure:

- Every runtime-presented geometry consumer names the same `present_submitted` generation.
- Submitted pixel targets and hit-test/accessibility targets agree after first-tick scroll at all scales; no scanout claim is made.
- No input path reconstructs scroll ancestry independently.

### SC-009 — Payload locality regression rails

Goal: prevent the general architecture from regressing document editing or virtualization.

Work:

- Preserve the U-001 4 MiB and synthetic 64 MiB sparse-checkpoint witnesses.
- Add typing-during-horizontal-scroll and caret-reveal traces for large unwrapped text.
- Verify table and virtual-list edits invalidate payload-local regions without changing scroll topology unnecessarily.
- Separate edit locality, cold admission, and property-scroll budgets in receipts.

Closure:

- U-001 sharing and latency bounds remain green.
- Character input cost scales with the guarded edit region, not document length or horizontal content width.
- Scroll topology/property state is reused across payload-local edits when ownership/range did not change.

### SC-010 — Delete alternate paths and close the campaign

Goal: leave one enforceable architecture rather than compatibility layers.

Work:

- Repeat the scroll/transform/clip source census.
- Delete superseded ancestry maps, independent scope/spatial interpreters, group-local scroll inference, unconditional full uploads on sparse ticks, and ambiguous state names once no consumer remains.
- Preserve explicit full-transfer paths for initialization, buffer/topology replacement, and cost-selected dense updates.
- Fail closure if production still contains independently authoritative old/new spatial architectures, even if both pass current pixels.
- Run Tier A, the checked-in Tier B manifest, all Tier C mechanism suites, native control gallery, diagnostics, benchmarks, formatter, lints, and the complete all-target/all-feature suite.
- Perform a manual native pass for text, table, virtual list, nested panes, thumb drag, wheel/trackpad, resize, scale change, and cold residency crossing.

Closure theorem:

> For any content item bound to a viewport, one or more legal scroll requests may coalesce or be superseded before frame selection. If an unsuperseded intent is selected, it contributes to at most one candidate property state for that frame. Every renderer representation and every runtime-presented geometry consumer evaluates the same spatial ancestry exactly once. Successful queue submission followed by `SurfaceTexture::present` advances one `present_submitted` generation containing moving geometry and chrome atomically as a submitted frame; it does not assert scanout. Warm property work follows indexed dirty production and an explicit sparse/dense transfer policy, while cold work is bounded by declared residency guards, independent of payload type or document length.

## 10. Per-loop operating protocol

At the start of every resumed task:

1. Read this document completely.
2. Run `git status --short`, `git branch --show-current`, and `git rev-parse --short HEAD`.
3. Inspect every pre-existing diff and preserve its provenance.
4. Identify the single active `SC-*` loop and its first unmet closure condition.
5. Re-run the narrow red witness before editing production code.
6. Record a compact receipt: command, build profile, scale/input fixture, expected invariant, observed result, and negative-control result.
7. Run formatting and tests proportional to the change, then the loop's required broad suite.
8. Update this document with findings and evidence.
9. Commit and push the closed loop before starting another.

Do not:

- close a loop from a structural test alone;
- use a fresh realization built by the same transform interpreter as the only oracle;
- add table-, text-, or rule-specific scroll correction;
- change scheduler policy without a causal trace and negative control;
- equate resident acceptance, `present_submitted`, and scanout;
- carry a compatibility path beyond its explicitly recorded consumer migration/removal loop;
- stage or commit unrelated inherited work.

## 11. Immediate next action

Resume at **SC-004**. Start from the SC-000 generation vocabulary and eight-case Tier C state suite. Trace the current ownership and mutation points for requested/coalesced intent, clamped and resident-accepted values, candidate property serials, GPU submission, and `present_submitted`; prove the existing ambiguous or non-atomic transition as the negative control before changing names or state. Include coalesced and superseded requests, no-op, failed acquire, delayed redraw, resize, scale change, residency race, shared-axis composition, and renderer resynchronization when a candidate generation is skipped.

SC-000 added bounded diagnostics, receipt vocabulary, property-write attribution, a source census, and a deterministic test manifest. SC-001 froze the independent payload-neutral oracle and negative controls. SC-002 replaced distributed renderer-side ancestry with one candidate-owned topology and closed all 40 Tier A executions. SC-003 closed indexed dirty property production and sparse/dense transfer economics. Generation state, input precision, pacing, residency, present-submitted geometry consumers, and locality remain separate open loops.
