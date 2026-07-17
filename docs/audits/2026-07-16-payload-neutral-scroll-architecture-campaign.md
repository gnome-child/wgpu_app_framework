# Payload-neutral scrolling architecture audit and campaign

Status: **EXECUTION COMPLETE; SC-000 THROUGH SC-010 CLOSED**

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
| SC-004 | CLOSED | SC-000 generation trace | Requested intent, resident acceptance, candidate property generations, and `present_submitted` now have distinct owners/names; skipped property generations resynchronize explicitly. |
| SC-005 | CLOSED | SC-000 input traces | Pixel/fractional-line input stays precise through routing; one per-target interaction accumulator owns compensated fractions and integral visual quantization. |
| SC-006 | CLOSED | SC-000 causal trace | One platform redraw ledger issues immediately and deduplicates in flight; the completion-anchored runner clock is deleted and causal trace v2 records every submission stage. |
| SC-007 | CLOSED | SC-000 residency receipts, SC-004 state contract | One accepted-offset/demand contract, exact 18-case suite, trace-v3 work attribution, fractional GPU witness, and bounded 1/4/64 MiB text receipts are recorded. |
| SC-008 | CLOSED | SC-002 topology, SC-004 terminology | One lazy present-submitted spatial evaluator now supplies runtime hit/input geometry, typed target offsets, native-popup surface geometry, and IME caret projection. |
| SC-009 | CLOSED | SC-003/SC-007 | Version-10 guarded-edit, moving horizontal projection, caret-reveal, and payload-topology reuse rails are recorded. |
| SC-010 | CLOSED | SC-001 through SC-009 | Alternate ownership paths are deleted, selected native preparation has immutable identity, the final census is classified, and deterministic/GPU/native closure is recorded. |

Initial evidence ledger:

| Evidence | State | Receipt |
|---|---|---|
| E-000 repository provenance | RECORDED | The only divergent campaign branch was a linear 33-commit descendant of `master`; it was fast-forwarded into `master`. Campaign formulation was pushed at `cd00554d`. The inherited U-002 correction and independent fixture remain uncommitted SC-001 provenance. |
| E-001 warm table property tick | RECORDED | Release `table-scroll-work`, scales 1.0/1.25/1.5/1.75/2.0: zero semantic/content preparation, zero resource churn, one plan reuse, 11,072 property-upload bytes split as node 11,008, scroll 32, text 32, viewport/unattributed 0. |
| E-002 grouped first-tick oracle | RECORDED | F03 compares the retained first property tick with a separately authored static commit, checks rule and grouped-quad old/new regions, and checks direct/incremental plan signatures at all five scales. The inherited correction passes all five; the actual legacy binding fails all five on grouped-quad translated occupancy. |
| E-003 unwrapped edit locality | RECORDED/PRIOR | Pushed `d4909a8d`; 4 MiB and synthetic 64 MiB sharing receipts are summarized in section 2. |
| E-004 cold glyph admission | RECORDED/PRIOR; SUPERSEDED BY E-028 | Roughly 24 GiB was observed for an earlier synthetic 64 MiB path. Current pre-fix and final capped executions do not reproduce it; E-028 records the superseding multi-size evidence without erasing this prior receipt. |
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
| E-020 generation/state contract | RECORDED/CLOSED | One private per-window `PresentationState` owns requested and present-submitted epochs. New requests advance requested state; retries do not; stale, duplicate, or impossible future receipts cannot advance present-submitted state. The exact Tier C suite passes seven deterministic core cases plus one release GPU scale-change case. |
| E-021 skipped property generation | RECORDED/CLOSED | The release negative control exposed one stale pixel when the retained renderer received generation N+2 whose local dirty set omitted N+1. The final renderer stores each slot generation, permits sparse mutation only from the declared predecessor with exclusive ownership, and otherwise selects one named 65,536-byte generation resynchronization. |
| E-022 post-state scroll regression | RECORDED | Release Tier A remains 40/40 with all 10 mutation controls discriminating. At all five scales the table payload remains at 464 warm property bytes and zero generation resynchronizations. Scale 1.0 to 1.25 selects one topology/viewport replacement totaling 65,552 property bytes. No receipt claims scanout. |
| E-023 high-resolution input | RECORDED/CLOSED | The negative control mapped five 0.4-logical-pixel events to zero instead of their 2.0-pixel sum. The final exact 20-case suite passes tiny, reversal, and burst/coalesced traces at all five scales plus fractional line, thumb, keyboard, reveal, and programmatic absolute paths. Fractions are target-local; visual offsets remain integral. |
| E-024 post-input scroll regression | RECORDED | Release Tier A remains 40/40, all 10 mutation controls remain discriminating, all 13 property-economics cases pass, skipped-generation recovery remains explicit, and the table payload remains at 464 warm property bytes at all five scales. |
| E-025 presentation cadence | RECORDED/CLOSED | The negative control rejected a demand arriving 1 ms after a 60 Hz present until the completion-anchored interval expired. One platform-owned in-flight ledger now issues immediately and coalesces duplicates; exact steady/burst/delayed cases pass at 60/90/120/144 Hz. Scroll trace v2 correlates redraw request/delivery, candidate, acquire, queue submit, surface present call, and present-submitted receipt. |
| E-026 post-cadence scroll regression | RECORDED | Release Tier A remains 40/40 with all 10 mutation controls, all 13 property-economics cases, skipped-generation recovery, and the five-scale 464-byte warm table receipt green. The complete suite passes after deleting the runner cadence owner. |
| E-027 payload-neutral residency contract | RECORDED/CLOSED | The exact 18-case suite crosses text/table/virtual-list with resident interior, guard edge, forward, reverse, large-jump, and fractional-scale cases. Interior ticks attach no cold-work fields; crossings attach CPU/GPU work to one candidate/present-submitted serial; the next tick reuses the drawable with no snap. |
| E-028 bounded large-text admission | RECORDED/CLOSED; SUPERSEDES E-004 | Version-9 release 1/4/64 MiB receipts peak at 79,921,152/97,910,784/171,175,936 private bytes under a 256 MiB budget. Cold exact indexing streams 5/17/257 bounded bands; the resident source window remains 546 bytes and resident glyph storage 101,010 bytes. The official 64 MiB crossing is p50/p95/p99/max 16/20/25/73 microseconds. |
| E-029 post-residency scroll regression | RECORDED | Release Tier A remains 40/40 with all 10 controls. The table property hit remains 464 bytes at all five scales; all 13 property-economics cases, skipped-generation recovery, scale replacement, 18 Python checks, and the complete all-target/all-feature suite are green. |
| E-030 present-submitted spatial evaluator | RECORDED/CLOSED | Runtime point/rect/clip and target-axis projection now use one `SpatialSnapshot` generated from the submitted candidate topology. Snapshot capture clones only Arc-backed layers; frame paths, caret states, and target bindings are compiled once, evaluated lazily, and guarded against an eager `commit.nodes()` warm-frame scan. |
| E-031 stale geometry, popup, and IME atomicity | RECORDED/CLOSED | Failed and deferred candidates emit no IME update; a newer request supersedes a skipped candidate; a late successful stale receipt cannot replace hit geometry. Native-popup frames use geometry-only topology supplements. The IME negative control kept y=22 after a 20-pixel text scroll; the final exact submitted projection reports y=2. |
| E-032 post-geometry scroll regression | RECORDED | Release Tier A remains 40/40 at all five scales with all 10 controls discriminating. The table warm hit remains 464 bytes at all scales with zero payload/resource/plan-rebuild work. Pointer, divider drag, virtual rows, sticky headers, selection, caret reveal, scrollbar capture, popup surface, context-routing, and platform IME rails pass. |
| E-033 large unwrapped edit/reveal locality | RECORDED/CLOSED | Version-10 official 4/64 MiB edit p50s are 358/375 microseconds with the same 820-byte maximum guarded splice and zero full index/width scans. Moving horizontal projection is p50 363 microseconds across 1,019 offset changes; caret reveal is p50 387 microseconds with 1,024/1,024 exact reveal changes. |
| E-034 payload edit topology reuse | RECORDED/CLOSED | One table-cell or virtual-row payload mutation changes exactly one retained composition identity, adds/removes none, rebuilds at most one scene node, and preserves equal normalized spatial/property topology, every scroll owner/range/topology revision, and every scroll property value. |
| E-035 immutable selected presentation identity | RECORDED/CLOSED | A same-structure newer frame could replace the selected in-flight presentation while residency retirement remained keyed to the original epoch. The red table-generated witness now requires immutable exact `Arc` identity, duplicate suppression, and one latest-intent follow-up; the unrelated-completion inverse remains green. |
| E-036 final native and campaign closure | RECORDED/CLOSED | Release native table wheel/reversal/thumb testing converged without blank rows, later clicks, virtual rejection, property mismatch, or stranded selection in receipts `1784266131025` and `1784266180280`. Release stress-text wheel/reversal/horizontal-thumb/edit/resize testing also stayed live. All 27 GPU witnesses, 1,363 library tests, 18 Python checks, the executable mechanism matrix, and the final census pass. |

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

#### SC-004 closeout — one state vocabulary and recoverable property generations

SC-004 closes generation ownership and terminology without changing high-resolution input, scheduling policy, residency bounds, or runtime point projection. Interaction state now names its two values `desired` and `resident_accepted`; request/clamp processing preserves desired intent when current residency cannot accept it. Candidate scene properties remain immutable snapshots. A private per-window `PresentationState` is the sole owner of requested and `present_submitted` epochs. New invalidation/property requests advance requested state, retrying failed work does not mint a new epoch, and stale, duplicate, or impossible future receipts cannot advance `present_submitted`.

The successful receipt boundary is structural: `src/render/surface.rs` submits the command buffer and then calls `frame.present()` before returning present timing. Native surface handling derives `present_submitted` only from that successful timing, and runtime state records the epoch/property serial only from that report. Failed acquisition therefore retains the prior submitted geometry and requested epoch. Diagnostics, runner pulse state, executable receipt validation, and the text-editor debug panel use `present_submitted` explicitly. This remains a command-submission plus surface-present-call fact; it is not scanout or human-visibility feedback.

The state audit exposed a separate renderer recovery defect. A retained slot could receive property generation N+2 after N+1 was skipped, apply only N+2's local dirty indices, and leave N+1's value stale. The release negative control changed one property in each generation and observed one stale pixel. `Properties` now records its direct predecessor, retained node and scroll property slots record their installed serial, and sparse in-place mutation is legal only when an exclusively owned slot contains that predecessor. A slot already at the requested serial is a no-op; every other compatible generation mismatch rebuilds the complete slot and records `property_full_generation_resyncs`. The corrected witness is pixel exact and selects exactly one 65,536-byte generation resynchronization. This recovery path is distinct from initialization, buffer replacement, topology/viewport replacement, and cost-selected dense transfer.

The exact eight-case Tier C suite is source-counted by an architecture gate: coalesced requests, superseded request, no-op request, failed acquire, delayed redraw, resize, residency race, and scale change. The first seven are deterministic library tests; the scale case is an explicit release GPU witness that changes 1.0 to 1.25, remains pixel exact against a fresh realization, and selects one topology/viewport replacement totaling 65,552 property bytes. Existing typed-axis tests continue to reject conflicting same-axis owners and accept split-axis shared targets; saturation and residency-crossing tests remain deterministic rails outside the exact eight names.

Payload-neutral scroll behavior and SC-003 economics did not regress. The release Tier A oracle remains 40/40 at scales 1.0, 1.25, 1.5, 1.75, and 2.0, and all 10 mutation controls remain discriminating. At every scale `table-scroll-work` remains one dirty index, seven value visits/lookups, two ranges, 464 property bytes split 64/272/128 across node/scroll/text, zero semantic/content/shaping/resource/plan-rebuild work, one plan reuse, and zero generation resynchronizations. The release 13-case property-economics suite remains exact.

Verification at this boundary: the seven deterministic generation cases, exact-suite architecture gate, presentation-state monotonicity guard, release GPU scale and skipped-generation tests, release Tier A/negative/property/table witnesses, formatter, diff check, Python receipt/census/manifest suites, all-target/all-feature check, and complete all-target/all-feature test suite passed. The complete Rust suite reports 1,260 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 24 hardware ignores, and two example tests passed.

`PresentedGeometry` still independently projects layout ancestry and deliberately remains SC-008. The completion-anchored `PresentationPulse` policy is only renamed here and deliberately remains SC-006. The 64 MiB cold glyph-admission defect remains SC-007, and fractional per-event input loss is now the ready SC-005 boundary.

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

#### SC-005 closeout — target-local precise input with integral visual state

SC-005 closes input precision without changing scene/GPU coordinate types, spatial topology, residency, presentation generations, or scheduling. The platform adapter previously divided pixel deltas by scale and rounded every event to `i32`; line-wheel fractions were likewise rounded after multiplying by 28 logical pixels. The red witness sent five 0.4-logical-pixel events at scale 1.0. Their aggregate is 2.0 pixels, but the old adapter emitted five zeros and produced `actual=0`.

`ScrollDelta` now carries finite logical `f64` components from pixel and fractional-line conversion through host, shell, target routing, and runtime dispatch. It does not own visual position. `interaction::Scroll` remains the per-target owner and stores a private `ScrollRemainder` beside desired/resident-accepted integral offsets. This location is material: platform/window accumulation could leak remainder across payloads when the pointer changes targets, while global scene accumulation would conflate input with candidate geometry.

The named visual policy is whole logical pixels. Each exact integral component is applied exactly, so keyboard and other discrete relative motion do not inherit a one-pixel penalty from an opposite fractional remainder. Only the fractional component enters a compensated per-axis accumulator. A fraction crosses into visual motion by truncation toward zero once it reaches a whole pixel. Floating sums within eight ULPs of an integral boundary snap to that boundary; the scale-1.5 burst witness exposed and now guards this numerical edge. Absolute thumb/programmatic requests and geometry/reveal projection reset the remainder because they author a new exact position. Fraction-only changes retain state but mint no scroll revision, candidate, or redraw until visual motion exists.

The exact 20-case Tier C suite is source-counted by an architecture gate. Five tiny traces use five 0.4-physical-pixel events at scales 1.0/1.25/1.5/1.75/2.0; the resulting integral desired values are 2/1/1/1/1 while every fractional remainder accounts for the unpresented sum. Five reversal traces move forward and return to exact zero without drift. Five burst/coalescing traces preserve six physical pixels and produce integral desired values 6/4/4/3/3 with fewer visual revisions than input events. The remaining cases prove fractional line-wheel conversion (0.25 line equals 7 logical pixels), exact thumb absolute, exact keyboard relative motion with an opposite retained fraction, geometry/reveal reset, and exact programmatic absolute reset.

Target routing consumes precise delta sign, so a subpixel event can select the correct scroll owner even when it produces no immediate property tick. `ScrollOffset`, scene properties, spatial bindings, renderer uniforms, chrome, and present-submitted geometry remain integral. This loop therefore does not broaden transform semantics or introduce fractional raster movement.

Payload-neutral rendering and property economics did not regress. Release Tier A remains 40/40 at all five scales and all 10 mutation controls remain discriminating. `table-scroll-work` remains 464 warm property bytes at every scale with one dirty index, seven visits/lookups, two ranges, zero semantic/content/shaping/resource/plan-rebuild work, zero generation resynchronizations, and one plan reuse. The exact 13-case property-economics suite and skipped-generation resynchronization witness remain green.

Verification at this boundary: the red input-loss witness was observed before production edits; the final red witness, legacy negative control, exact 20-case suite, exact-suite architecture gate, platform and popup conversion tests, integral-scroll architecture gate, SC-004 generation suite, formatter, diff check, 18 Python receipt/census/manifest tests, all-target/all-feature check, release GPU/property witnesses, and complete all-target/all-feature suite passed. The complete Rust suite reports 1,283 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 24 hardware ignores, and two example tests passed.

SC-005 makes no cadence claim. `PresentationPulse`, redraw delivery, acquire timing, and present-submitted intervals remain the ready SC-006 causal-trace boundary.

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

#### SC-006 closeout — immediate deduplicated redraw demand, platform-owned cadence

SC-006 closes scheduler ownership without changing input accumulation, scene/property work, spatial topology, or residency policy. The old native runner kept one `PresentationPulse` per window, anchored its next deadline at the last present-submitted completion, deferred both redraw issuance and an already delivered redraw until that deadline, and independently reissued demands that `Platform::apply_work` had already sent to the backend. A demand arriving 1 ms after a 60 Hz present was therefore rejected for the remaining roughly 15.7 ms software interval before the platform could align it with its own redraw/present cadence. That deterministic failure is retained as the negative control.

The completion clock, `frame_demands`, `issued_frame_redraws`, due-frame pass, and pulse-derived event-loop deadline are deleted. `Platform` now owns one `RedrawRequests` ledger for every backend. A semantic/property demand immediately enters the ledger and calls the backend exactly once; additional demand while that request is in flight coalesces; delivery clears the entry before candidate construction; backend failure also clears it; close and continuation/retry paths use the same owner. The event loop returns to its animation/task schedule and never polls merely to satisfy a software presentation interval. Native surface present mode and the OS/window system own delivery/vsync alignment.

High-rate integration sends 1,000 pointer moves plus 20 button events, mutates every command immediately, emits exactly one backend redraw request while in flight, and presents the latest state once on delivery. Failed acquisition reopens the same ledger through the ordinary retry path. A separately delivered redraw with no pending presentation increments `redraw_no_progress`; renderer receipts now expose redraw requests, deliveries, and no-progress count, with external validation requiring no-progress not exceed deliveries.

The exact 12-case Tier C suite is source-counted by an architecture gate: steady, burst, and delayed delivery at 60, 90, 120, and 144 Hz. Steady demand issues immediately and remains at most one refresh from the next modeled platform opportunity. Bursts collapse to one in-flight request and allow a new request immediately after delivery. Delayed delivery remains coalesced while in flight, then permits the next demand immediately instead of adding another completion-relative interval. The suite does not simulate scanout; it proves scheduler issuance/coalescing policy against refresh-relative opportunities.

`wgpu_l3.scroll_trace.v2` now correlates the complete production path by epoch/property serial: input, backend redraw request, redraw delivery, candidate construction, acquire start/finish, queue submission, `frame.present()` call, and present-submitted receipt. Each receipt reports input-relative stage latencies plus acquire wait. Surface timing is captured at the actual calls. `present_submitted_at` now uses the surface-present-call timestamp rather than a later timestamp after post-present candidate preparation. Existing renderer receipts continue to expose event-to-present-submitted p50/p95/p99/max, frame intervals/submission cadence, CPU stage distributions, acquire/encode timing, missed refresh opportunities, skipped frames, and no-progress redraws. These are field measurements, not scanout feedback.

Payload-neutral rendering and property economics did not regress. Release Tier A remains 40/40 at all five scales and all 10 mutation controls remain discriminating. `table-scroll-work` remains 464 warm property bytes at every scale with one dirty index, seven visits/lookups, two ranges, zero semantic/content/shaping/resource/plan-rebuild work, zero generation resynchronizations, and one plan reuse. The exact 13-case property-economics suite and skipped-generation recovery witness remain green.

Verification at this boundary: the old policy failed the 1 ms post-present red witness before production edits; the legacy negative control, exact 12-case suite, source architecture gate, causal trace test, high-rate dedupe, failed-acquire retry, no-progress accounting, formatter, diff check, 18 Python receipt/census/manifest tests, all-target/all-feature check, release GPU/property witnesses, and complete all-target/all-feature suite passed. The broad run first exposed one stale v1 schema assertion; it was updated to v2 and the complete suite reran green with 1,298 library tests passed, four intentional hardware ignores, three renderer-debug non-hardware tests passed with 24 hardware ignores, and two example tests passed.

SC-006 does not claim that every field machine/display will have identical latency, and it does not claim scanout timing. Cold residency and the roughly 24 GiB 64 MiB glyph-admission defect are now the ready SC-007 boundary.

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

#### SC-007 closeout — one residency demand, bounded admission, generation-attributed work

SC-007 closes resident admission and crossing work without changing input accumulation, scheduler ownership, spatial topology, or runtime-presented hit-test projection. `Layout::residency_demand` now returns one payload-neutral demand containing the interaction target, authoritative desired offset, and any deduplicated payload materialization adapters. Text produces the same demand even though it has no virtual-list adapter. Runtime dispatch and active-descendant reveal install this contract before the existing payload-specific materializers run; there is no virtual-list-specific scroll transition path.

The exact Tier C suite is source-counted at 18 cases: text, table, and virtual-list payloads each exercise resident interior, guard edge, forward crossing, reverse crossing, and large jump at scale 1.0 plus one forward crossing at scale 1.25. The accepted interval is the authority. Interior and edge requests remain property-only. Crossings preserve the immutable semantic commit, create one newer drawable residency revision, submit the requested offset in the selected property generation, and make the immediately following offset a property-only tick against the same drawable. Provider calls are capped at 256, text line shapes at 128, and non-root frames at 512. The suite first exposed a one-pixel reverse text snap: a requested 61,019 offset was corrected back to the stale 61,020 anchor. Text anchors now run only when content version, wrap, style, or viewport width changes; a pure residency recompose no longer reapplies a cached anchor, while resize/reflow anchor tests remain green.

`wgpu_l3.scroll_trace.v3` attaches candidate work—layout recomposes, semantic commits, scene paints, line shapes, horizontal-index/window source bytes, and render source bytes—and renderer work—primitive/text preparation, renderer shapes, content/property uploads, and GPU resource churn—to the selected crossing generation. Work accumulates across genuine retries and an active refresh cannot overwrite the candidate receipt. Resident property traces carry `none` for all cold fields. Crossing traces carry numeric fields and matching candidate, GPU-submitted, and present-submitted property serials.

The release fractional GPU witness renders a forward crossing and the next resident tick for every payload at scale 1.25. Text/table/virtual candidate CPU times were 2,225/1,088/604 microseconds; provider calls were 0/45/15 and scene paints 1/62/16. Across all three, the maxima were 24 primitive prepares, 45 text prepares, 24 renderer shape calls, 3,840 content-upload bytes, 6,176 property-upload bytes, and 70 resource creations. Each crossing rebuilt exactly one retained plan. The next tick reused that plan, performed zero primitive/text preparation, shaping, content upload, resource churn, or plan rebuild, remained below 4,096 property bytes, matched the exact reference renderer, and did not snap.

The inherited roughly 24 GiB 64 MiB glyph-admission observation does not reproduce on current `master`, including capped pre-fix runs across repeated edit revisions. This boundary does not falsely attribute that earlier improvement to its own patch. It did find a separate deterministic fallback: the generated 1 MiB ASCII source ends in a partial word, its final streamed fragment validly has two absolute checkpoints, but the shared constructor rejected fewer than three and forced a whole-line glyph buffer. The red unit witness proved that safe fragment returned `None`. Allowing the shared constructor's documented two-checkpoint fragment minimum makes the cold path emit five exact bands and reduces the observed 1 MiB process peak from roughly 294–340 MiB private memory to 79,921,152 bytes.

Final version-9 release scaling receipts distinguish sampled `ResidencyCrossing` work from `cold_transition_class=ColdStart`. At 1/4/64 MiB, peak working sets were 79,642,624/84,828,160/155,619,328 bytes and peak private bytes were 79,921,152/97,910,784/171,175,936, all below the explicit 256 MiB reference budget. Cold exact indexing took 298,002/1,112,938/16,831,891 microseconds and streamed 5/17/257 bands. Horizontal-index residency was 46,332/185,268/2,964,024 bytes; the cold resident source window stayed 546 bytes, resident glyph storage stayed 101,010 bytes, and the render window stayed 1,432 by 640. The official 64 MiB 64-warmup/1,024-sample crossing receipt reports p50/p95/p99/max 16/20/25/73 microseconds, one 273-byte window shape, maximum resident glyph storage 202,020 bytes, and maximum line-cache residency 656,565 bytes.

Exact global horizontal extent discovery is intentionally not represented as viewport-local. Preserving an exact no-wrap scrollbar width requires source-wide extent metadata when no valid index exists, so its 64 MiB cold-start latency remains source-proportional under a separate 20-second release reference budget while its memory and shaping bands are bounded. This evidence corrects an overbroad assumption in the original closure theorem: cold **residency admission** must be guard-bounded; exact global range discovery is a separately named and measured `ColdStart`. It may be optimized further without forking scroll semantics.

Payload-neutral scrolling and prior economics did not regress. Release Tier A remains 40/40 at all five scales and all 10 mutation controls remain discriminating. `table-scroll-work` remains exactly 464 bytes at every scale with zero payload work and one plan reuse. The exact property-economics, skipped-generation, scale-change, horizontal-index sharing, terminal-fragment, text-anchor, manifest/census/receipt, formatter, diff, and all-target/all-feature checks passed. The complete suite reports 1,319 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 25 hardware ignores, and two example tests passed; the new release ignored GPU residency witness was also executed explicitly and passed.

SC-007 makes no claim about hit testing or accessibility geometry. `PresentedGeometry::project_point` still reconstructs layout ancestry independently and is now the ready SC-008 boundary.

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

#### SC-008 closeout — one lazy present-submitted evaluator for pixels and interaction

SC-008 closes runtime-presented spatial ownership without changing renderer transforms, input accumulation, scheduler policy, residency, or payload algorithms. `SpatialTopology` now compiles three input-facing indices beside its renderer bindings: a fixed frame/hit scroll path for every scene node, actual caret content states, and typed per-target axis bindings. A viewport owner's frame remains fixed while its content/caret binding consumes the viewport's own scroll; descendants consume outer scrolls; and content below a surface root regains world movement exactly once. The F03 structural witness proves fixed owner frame space, moving rule/content space, moving grouped descendant space, and the fixed viewport clip independently.

`SpatialSnapshot` is an evaluator, not an eager translated-scene copy. On each successful submission it captures only Arc-backed drawable/property layers from the actual submitted stack. Node, caret, clip, and target-offset queries use topology-compiled maps and short scroll paths lazily. An architecture gate rejects a `commit.nodes()` scan inside snapshot capture. This matters because the first implementation audit found that eagerly materializing every node on every property tick would have reintroduced scene-size warm-scroll work despite correct pixels.

`PresentedGeometry` no longer calls `Layout::scroll_ancestry`. Pointer hover/press, drag and divider coordinates, selection/text hit routing, context-menu nodes and bounds, scrollbar target routing, and stationary-pointer reprojection all delegate to the installed snapshot while retaining the submitted layout only for semantic frame geometry. Split-axis targets merge through the topology's typed target bindings. Missing or conflicting topology projection fails closed rather than silently substituting identity geometry.

Native popups exposed one broad-only omission: their frames are intentionally absent from the parent draw stack, so the first snapshot could not address popup-surface hits. Live native-popup commits/properties now enter the stack as `SpatialSupplement` geometry only. They are consumed by the same evaluator, retained across native active/pending projection, and never become parent render layers. Popup hosts may still be prepared and independently presented before the parent submission; that existing surface-lifecycle contract is recorded rather than mislabeled as atomic parent scanout.

IME no longer travels as an unconditionally applied parallel candidate update or an epoch-keyed pending map. The actual backend `shell::Presentation` carries an authored `ime::Projection`. After a successful current receipt, runtime resolves that projection through the exact installed snapshot/property serial and platform applies it. Failed/deferred candidates apply nothing; stale/superseded receipts cannot replace it; native popup cursor geometry remains popup-local. The end-to-end red witness scrolled focused text by 20 logical pixels and observed unchanged y=22 under the old layout-only projection. The final submitted property frame reports y=2 exactly. Popup preparation still precedes IME host activation, and deferred preparation emits one update only when the matching parent submission succeeds.

The stale-candidate witness now goes beyond a failed retry: a 16-pixel candidate is skipped, a second request supersedes it, the 32-pixel candidate presents, and a deliberately late successful receipt for the skipped candidate cannot alter the submitted property serial, hit target, or stationary hover. Pointer/table divider, virtual-row transfer, sticky header, scrolled-out clipping, text selection, caret reveal, scrollbar thumb/capture, context, native-popup surface, failed-present, and deferred-present rails all pass.

No AccessKit tree, accessibility adapter, or emitted accessibility bounds exist in production. The audit found only reserved semantic/documentation seams. SC-008 therefore does not claim accessibility output agreement; it makes the submitted spatial evaluator the sole geometry seam that a future adapter must consume. This is an explicit absent-consumer result, not a hidden exception.

Release Tier A remains 40/40 at scales 1.0, 1.25, 1.5, 1.75, and 2.0, with all 10 negative controls still failing their intended mutation. The warm table receipt remains exactly 464 property bytes at every scale, with one dirty index, seven visits/lookups, two ranges, zero semantic/content/shaping/resource/plan-rebuild work, and one plan reuse. These combined receipts prove five-scale submitted pixels plus a scale-independent logical hit evaluator; they do not claim scanout timing.

Verification at this boundary: the runtime-ancestry source gate and failed-present IME gate were observed red before production edits; the 20-pixel IME projection was separately observed red at unchanged y=22; formatter, diff check, all-target/all-feature check, 18 Python manifest/receipt/census tests, platform suite, spatial-purpose unit witness, stale/superseded candidate witness, consumer rails, release Tier A, all 10 negative controls, and five-scale table receipts pass. The first broad platform run exposed missing native-popup spatial sources and the first complete run exposed one stale source assertion; both were corrected and rerun. The final complete suite passes 1,323 library tests with four intentional hardware ignores, three renderer-debug non-hardware tests with 25 hardware ignores, and two example tests.

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

#### SC-009 closeout — guarded edits remain independent of document and scroll topology size

SC-009 closes payload-locality regression rails without changing text editing, scroll transforms, scheduler policy, residency, or renderer property transfer. `wgpu_l3` scroll-bench version 10 now names four separate currencies instead of assigning all work to scrolling: sampled guarded edit work, resident horizontal projection, caret reveal, and cold exact-extent discovery. Every text layout receipt explicitly reports `property_scroll_measured=false`; the renderer's independent property-tick receipt remains the authority for warm scrolling. Edit, projection, and reveal phases have separate p50/p95/p99/max samples, and every incremental index update records its maximum source span as well as its aggregate work.

The test-only architecture gate was first observed red because version 10, the two new workloads, phase currencies, and payload-topology witnesses were absent. The executable benchmark validator has a separate negative control: injecting a 4,097-byte edit splice fails the 4,096-byte guarded-edit contract. Final edit traces require one incremental update per sample, zero full horizontal-index builds/source bytes, zero source-wide width bytes, a bounded resident render window, and complete phase accounting. The typing trace must change horizontal projection in at least half of multi-sample runs; the reveal trace must move every deliberately hidden caret.

The final release 64-warmup/1,024-sample `text-horizontal-edit-4m` receipt reports total p50/p95/p99/max 358/382/469/773 microseconds, edit-only 6/7/8/22 microseconds, and projection 352/377/463/767 microseconds. The 64 MiB counterpart reports 375/399/416/521, edit-only 9/9/15/52, and projection 368/392/409/511 microseconds. Both perform exactly 1,024 incremental updates, rebuild the same 839,168 aggregate source bytes and 820-byte per-edit maximum, perform zero full index or width-source work, preserve exact offsets beyond the `f32` boundary, and retain a 1,432 by 640 resident render window. A 16-times larger document therefore changes measured p50 by about 4.7 percent while guarded work remains byte-for-byte equal. The synthetic 64 MiB witness still requires all untouched checkpoint blocks to remain shared.

`text-horizontal-typing-scroll-4m` edits at the caret while moving through its resident horizontal neighborhood. Its official total p50/p95/p99/max is 363/432/501/750 microseconds; edit-only is 6/7/9/67 and projection is 357/427/494/746 microseconds. It records 1,019 horizontal projection changes, 1,024 guarded incremental updates, the same 820-byte maximum splice, zero full source work, and the same bounded window. `text-horizontal-caret-reveal-4m` starts each edited caret 256 logical pixels outside the viewport and invokes the production reveal path. All 1,024 reveals move, with maximum movement 263 pixels. Total p50/p95/p99/max is 387/411/468/574 microseconds; reveal is 372/396/454/550 and the subsequent resident paint is 9/10/17/33 microseconds. It also retains the 820-byte edit maximum and zero source-wide warm work.

Table and virtual-list witnesses mutate one visible payload value without changing key, viewport, ownership, or range. Each changes exactly one retained composition identity, adds and removes none, and rebuilds at most one retained scene node. Before/after commits have equal normalized `SpatialTopology` and property-slot topology; every scroll owner's identity, parent, declaration/range, target binding, `TopologyRevision`, and property offset is unchanged. The edited payload is independently observed in the retained view or rendered scene. This is logical topology/state reuse; it does not claim that semantic payload content is a property-only frame.

Payload-neutral scrolling did not regress. Release Tier A remains 40/40 at all five scales and all 10 mutation controls remain discriminating. The table warm property hit remains exactly 464 bytes at scales 1.0, 1.25, 1.5, 1.75, and 2.0 with one dirty index, seven visits/lookups, two ranges, zero semantic/content/shaping/resource/plan-rebuild work, and one plan reuse. Formatter and diff checks, the guarded splice unit witness, synthetic 64 MiB sharing, both payload-topology witnesses, 18 Python manifest/receipt/census tests, and the complete all-target/all-feature suite pass. The complete suite reports 1,327 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 25 hardware ignores, and two example tests passed.

SC-009 does not fold cold exact global extent discovery into edit latency and does not infer renderer property economics from text layout. Source deletion, repeated ownership census, all mechanism suites, and the manual native checklist are now the SC-010 boundary.

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

> For any content item bound to a viewport, one or more legal scroll requests may coalesce or be superseded before frame selection. If an unsuperseded intent is selected, it contributes to at most one candidate property state for that frame. Every renderer representation and every runtime-presented geometry consumer evaluates the same spatial ancestry exactly once. Successful queue submission followed by `SurfaceTexture::present` advances one `present_submitted` generation containing moving geometry and chrome atomically as a submitted frame; it does not assert scanout. Warm property work follows indexed dirty production and an explicit sparse/dense transfer policy, while cold residency admission is bounded by declared guards, independent of payload type or document length. Exact global extent discovery may retain a separately budgeted source-proportional `ColdStart` contract; it may not admit whole-document payload storage.

#### SC-010 closeout — immutable selected identity and native convergence

The manual native table viewport exposed a residency-preparation failure after the source-deletion census and deterministic mechanism suites were green. The following receipts preserve the failures that kept SC-010 open until the final identity correction; they are not retroactively relabeled as passing evidence.

The initial bounded-preparation build still froze during aggressive vertical table scrolling. Receipt `control-gallery-500px-idle-1784255665430.txt` recorded 242 scroll inputs, 3 selected residency candidates, 316 preparation slices, a 46,240-microsecond maximum slice, frame-interval p95/p99/max of 671,846/1,425,310/1,517,082 microseconds, and a 675,643-microsecond fast-burst transition. Surface acquire p95 was only 32 microseconds. The trace therefore rejected GPU acquisition as the freeze owner and showed required visible content waiting behind or being inflated by cold residency work.

The first correction separated `Required` and `Proactive` residency urgency, allowed a required candidate to replace queued speculative work and preempt a selected proactive native preparation, and made required large jumps materialize only the critical visible-plus-overscan rows. The red scheduler witness proved a full selected/queued speculative pipeline rejected required work. The red large-jump witness observed 30 table rows where the critical bound was 12. Both are green after the correction; runtime, platform, native-surface, residency, architecture, diagnostics, and all-target/all-feature check rails pass. A stale runtime assertion was corrected to require property refreshes to use the last present-submitted active commit and explicitly reject the unsubmitted semantic candidate.

Native receipts `control-gallery-500px-idle-1784256926218.txt` and `control-gallery-500px-idle-1784256850848.txt` are partial green only. The user reported that the first fast scroll became markedly better, but stopping and starting another fast scroll restored the freeze. The short receipt reduced maximum preparation slice time from 46,240 to 1,930 microseconds with zero deadline misses, but selected 31 residency candidates from 4 direct schedules and emitted 27 follow-ups. It created/removed 9,200/8,861 retained GPU resources. The longer receipt selected 320 candidates from 8 direct schedules, emitted 312 follow-ups, and created/removed 80,299/80,462 resources. Both recorded zero proactive preemptions because required crossings repeatedly replaced their own narrow materialized windows before proactive runway could become useful.

The next red witness proved the remaining ownership error: installing an overlapping required range replaced materialization `100..130` with `120..145`, evicting useful prepared rows. The corrected contract produces `100..145`, rolls the overlap forward under the 80-row transition cap, and resets to only the critical range for a distant jump. The end-to-end table witness requires a second nearby hard-edge candidate to retain every provided row from the first while remaining capped; the independent large-jump witness still requires critical-only first admission. Both are green. A new native stop/fast-scroll/start/fast-scroll receipt is pending and must show behavioral recovery plus sharply lower resource churn before this mechanism can be accepted.

Native receipt `control-gallery-500px-idle-1784257621865.txt` rejects overlap retention as a complete remedy. The user observed no good fast burst; only the severity varied. The receipt recorded 245 scroll inputs but only 3 selected residency candidates, 241 coalesced requests, a 48,810-microsecond maximum preparation slice, frame-interval p95/p99/max of 527,812/1,296,257/3,426,454 microseconds, a 5,503,096-microsecond key-to-present-submitted maximum, and a 7,427,411-pixel desired/resident lag. Resource creation/removal fell to 3,344/2,981, proving that overlap retention reduced churn without restoring motion.

The trace exposed a distinct stale-front policy. One required candidate near y=1,092 remained selected while newer required intent reached roughly y=5,040; later a required candidate near y=8,232 remained in front while intent reached y=7,432,486. The current native policy preempted only proactive work and deliberately allowed a required front to advance through intermediate residency boundaries. A receipt-derived red witness built two required table candidates, placed the successor offset outside the selected candidate's declared residency, and failed because the selected front was not preempted. The experimental correction permits a newer required candidate to retire a required front only when both share the same semantic commit and the selected residency rejects the newer candidate's authored offset. Overlapping or still-usable required fronts remain intact. Receipts distinguish `scroll_residency_obsolete_required_preemptions`; targeted native-surface and diagnostics suites are green.

Native receipt `control-gallery-500px-idle-1784258223833.txt` rejects that preemption experiment. The user reported that it felt the same as the prior freezing behavior. The receipt recorded 284 scroll inputs, 278 residency requests, only 4 property ticks, 23 selected residency candidates, 259 coalesced requests, 21 superseded candidates, and exactly 21 obsolete-required preemptions. No proactive candidate was preempted or canceled. Only 8 of 16 created semantic commits activated. Candidate preparation consumed 409 slices and 264,269 microseconds, with a 15,414-microsecond maximum slice and three deadline misses; frame p50/p95/p99/max was 38,634/650,583/1,011,036/1,053,345 microseconds, and key-to-present-submitted p95/max was 2,725,960/2,767,427 microseconds. Surface acquisition remained negligible at 39/57 microseconds p95/max. Desired/resident lag reached 12,355,194 pixels. Early candidates each performed one layout recompose, about 562 scene paints, and 320 text line shapes, taking roughly 49-51 milliseconds to construct before submission. The 320 line shapes equal 80 retained rows times the table's four text columns, showing that overlap retention reduced GPU resource churn while inflating every synchronous candidate to the full 80-row transition window. Stale-required preemption therefore exchanged waiting behind an obsolete front for repeated expensive candidate construction; it is not accepted as a correction and native confirmation is no longer pending on that mechanism.

The next red gate must reproduce the production-sized six-column, 500-pixel table and prove that a nearby required crossing prepares only newly entering critical rows while preserving exact visible coverage. Retained/cache rows must not become active layout/draw rows solely to preserve reuse, and canceled or unsubmitted candidates must not become the sole materialization owner for future candidates. The witness must also distinguish text-buffer identity/cache reuse from row visibility so that the correction remains payload-neutral rather than table-specific.

That deterministic gate is now implemented with the actual control-gallery application, its six production columns, and its configured 500-pixel table viewport. After 16 nearby required crossings, the red execution exposed 56 active provided rows against a 14-row critical bound. Once the drawable-union assertion was isolated, the same final crossing performed 44 text-area line shapes while only three row keys were newly entering; the four text columns therefore had a 12-shape new-row bound. These values reproduce the receipt mechanism without depending on wall-clock timing or a GPU.

The correction restores exact replacement when a new materialization range is installed. Predictive requests can still author a bounded runway, but an older required candidate's rows are no longer unioned into the next active layout/draw range. Stable `(list, row-key)` identities now feed a reuse-only view cache: unchanged direct `TextArea` cells inherit the prior immutable text document/mark identities, while editable `TextBox` cells retain a separate inactive-display buffer that is never used for active editor state. Changed text fails both reuse controls. The production-scale witness now requires active rows to remain within the 14-row critical bound, scene paints to remain proportional to that bound, and line shapes to remain at or below four times the three newly entering rows. The 29-test residency suite, the two line-identity controls, and the residency architecture gate are green. Native fast-scroll stop/start confirmation is still required; this mechanism is not yet a campaign closeout.

Native receipts `control-gallery-500px-idle-1784260203103.txt` and `control-gallery-500px-idle-1784260213722.txt` are the next partial-green/red pair. The user reported that ordinary fast scrolling behaved much better but still missed and chopped, while thumb drag did not update until the drag ended and another update ran. The ordinary receipt records 297 inputs, 292 residency requests, 190 directly scheduled candidates, 107 coalesced requests, 201 selected candidates, 184 superseded candidates, and 184 obsolete-required preemptions. Candidate construction fell from the prior roughly 49-51 milliseconds and 320 line shapes to roughly 14-16 milliseconds and 30-64 line shapes; maximum preparation slice time was 1,862 microseconds with zero deadline misses. This accepts exact active materialization and row-key text reuse as a real improvement but rejects them as sufficient. The drag receipt accumulated 425 inputs, 418 residency requests, 191 direct schedules, 232 coalesced requests, 202 selections, and 184 preemptions. Its final trace coalesced requests 373-425 toward y=8,680,690 after a candidate at y=18,893,267, then ended `outcome=unchanged` with no candidate or submission. Desired/resident lag reached 18,496,111 pixels. This is the receipted form of the delayed thumb update.

The next scheduler red gate proved that coalescing occurred after candidate construction. While a required candidate was selected, another required request still authorized an immediate successor; the production-sized integration observed one successor instead of zero. `ResidencySchedule` now treats an equal-or-higher-urgency selected candidate as the sole construction front: later same-urgency requests advance only the latest intent/generation, build no queued candidate, and cause exactly one final latest-value follow-up when the front retires. A required request may still bypass or preempt selected proactive work. Unit, three-payload integration, stale-row reversal, and native-surface progression witnesses are green. The rejected required-versus-required native preemption experiment, its retirement variant, runtime handler, diagnostics field, and policy-specific assertions are deleted; the architecture gate requires their absence.

A broad deterministic property failure exposed a second atomicity gap. The command-palette result scroll reached desired/resident y=114, while the next submitted stack retained overlay y=0. The present-submitted-safe property path updated only the base layer and returned stale retained overlay drafts. It now matches every in-frame retained overlay draft to its exact layer in the last present-submitted stack, uses that layer's commit and property snapshot as the previous state, and rebuilds the compatibility scene from the updated properties. The oracle now requires submitted overlay y=114 exactly. Coalesced input that authorizes no new candidate also emits no invalidation and is no longer counted as another redraw request.

The first fully-clipped residency correction kept a removed captured virtual row out of scene painting, but the broad suite then exposed the opposite edge: a table row that could enter through its parent's accepted property runway had prepared nested text yet its fully clipped baseline viewport was classified empty and omitted from the drawable. The final rule is proof-based. A fully clipped nested viewport joins the drawable only when its prepared bounds contain the complete viewport/content area an ancestor property move can expose; an incomplete fully clipped viewport remains absent without blocking the current scene. The entering-row text witness and provider-deletion pointer-capture witness both pass.

Formatter and the complete `cargo test --workspace --all-targets --all-features` suite are green after these corrections: 1,358 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 26 hardware ignores, and two example tests passed. The release control gallery was rebuilt with `cargo build --release --example control_gallery --all-features`. Native stop/start fast-scroll and live thumb-drag confirmation remains pending; SC-010 is still open.

Native receipts `control-gallery-500px-idle-1784261767093.txt` and `control-gallery-500px-idle-1784261776555.txt` reject that build as closure while accepting another material improvement. The user reported that forward fast scrolling was markedly better, but reversal resumed chugging and a thumb drag still did not move the viewport; after release, another click was required before the content jumped to the requested position. The ordinary receipt records 585 inputs, 579 residency requests, only 64 selected candidates, 61 follow-ups, one supersession/proactive preemption, and zero pipeline cancellations. The cumulative drag receipt adds 113 inputs and 102 coalesced residency requests but adds zero selections and zero follow-ups while 242 more frames are present-submitted. Its bounded tail advances resident y from 13,748 to 15,489,075, then records the reversed final desired y=6,297,756 with no candidate or submission. Candidate preparation remained bounded at a 2,881-microsecond maximum slice and zero preparation deadline misses. This rules out the earlier per-input construction storm but proves that final intent can remain trapped behind a scheduler front after native realization has continued.

The deterministic reproduction separates residency-work completion from presentation-state activation. A required front is selected, a distant absolute request coalesces behind it, and an unrelated active-compatible frame with a newer epoch is successfully submitted before the selected front reports success. Before correction, the newer frame advanced `present_submitted`; the later selected-front report was correctly rejected as stale presentation state, but the same `activated` predicate also prevented the matching scheduler front from retiring. No latest-value follow-up was authored, reproducing the need for another click. Scheduler retirement now runs for every successfully submitted non-active rebuild and remains keyed to the selected candidate's own epoch; it no longer depends on that epoch replacing newer presented interaction state. The witness requires one final candidate at the exact latest offset while `present_submitted` remains at the newer overtaking epoch. Its inverse control proves that a newer unrelated completion cannot retire the selected front. A repeated no-op in the same trace epoch also no longer overwrites an already recorded residency transition, so the next native receipt will preserve the causal outcome that the prior tail mislabeled `unchanged`.

The corrected mechanism passes the 54-test residency set, all 11 native-surface policy tests, all 40 runtime-state tests, the 11-test scroll-trace set, formatter/diff checks, and the complete all-target/all-feature suite. The complete suite now reports 1,361 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 26 hardware ignores, and two example tests passed. Native reversal and thumb-drag confirmation against a newly rebuilt release gallery remains required; SC-010 is still open.

Native receipts `control-gallery-500px-idle-1784262910617.txt` and `control-gallery-500px-idle-1784262927030.txt` reject selected-front retirement as sufficient. Reversal stalled with the candidate property serial fixed at 331 while later candidate epochs continued, and the cumulative drag added 66 desired changes without adding a selected candidate or follow-up. The exact-semantic active-epoch projection that followed also fails as a complete remedy. Receipts `control-gallery-500px-idle-1784263815910.txt` and `control-gallery-500px-idle-1784263823199.txt` preserve the same manual failure: forward motion becomes smooth after a small initial hitch, reversal eventually freezes, and thumb drag requires a later click. The ordinary receipt records 883 inputs, 880 coalesced residency requests, 79 selected candidates, 76 follow-ups, and an 18,025,391-pixel maximum desired/resident lag. The drag receipt adds 139 inputs and 115 desired changes, but selections and follow-ups remain exactly 79/76 while lag reaches 18,027,264 pixels. Surface acquire p95 remains only 31-33 microseconds. Exact-semantic epoch projection therefore fixes its deterministic classification boundary but does not own this native stall.

The new receipt mechanism is an unfinished first preparation phase. `Renderer::synchronize_stack` first compiles/resumes each retained plan and only then realizes candidate properties/resources. `advance_stack_after_present`, used by every active-compatible continuation, previously called only `synchronize_candidate_layer`. If the initial 240 Hz preparation window expired while plan compilation was still pending, every later continuation skipped that unfinished phase. Candidate realization correctly returned `Pending` because no plan existed, active/hover frames remained presentable, and the selected residency front could never become ready or retire. An independent release GPU witness starts a real retained semantic commit with zero budget, proves no retained plan exists, permits only post-present progress, and failed with `post-present continuation skipped the unfinished retained-plan phase` before the correction. Post-present stack progress now delegates to the same bounded `synchronize_stack` operation as the initial attempt; the witness is green, as are the existing pending-active GPU control, 11 native-surface policy tests, 55 residency-filtered tests, the retained-renderer architecture gate, formatter, and diff checks. Release gallery timestamp `2026-07-16 23:57:54` is under native reversal/drag verification; SC-010 remains open.

Native receipts `control-gallery-500px-idle-1784264295787.txt` and `control-gallery-500px-idle-1784264326132.txt` are separate sessions and accept the retained-plan continuation as a real improvement without closing SC-010. In the first session, the user could drag and scroll regularly smoothly for substantially longer. It records 277 scroll inputs, one direct residency schedule, 276 coalesced requests, 29 selected candidates, and 28 follow-ups; one initial front plus 28 follow-ups accounts exactly for all selections, and maximum desired/resident lag stayed at 5,241 pixels. Preparation stayed below 1,891 microseconds with zero preparation deadline misses. In the second session the prior symptoms eventually returned. It records 473 inputs, 445 desired changes, three direct schedules, 442 coalesced requests, 36 selections, and 33 follow-ups; the three fronts plus 33 follow-ups again account for all selections, but maximum lag reached 10,723,138 pixels. Surface acquire p95 remained 31/34 microseconds across the two sessions, and neither receipt records a virtual-residency rejection.

The second session emitted the stronger causal witness: repeated `present-submitted IME property mismatch` errors first reported geometry serial 152 against receipt serial 158, then geometry 158 against receipt 162. The surface had therefore submitted a newer property snapshot while runtime input/IME geometry remained on an older snapshot. The presentation-state owner tracked only `PresentationEpoch`. A retry or settle frame could successfully submit a newer `PropertySerial` under the same request epoch, but epoch-only admission rejected it as a duplicate. The separate active-refresh structure heuristic was not an equivalent submission receipt and could leave the old `SpatialSnapshot` installed.

A deterministic runtime witness submits property serial 1, then deliberately reports a successful serial-2 replacement under the same epoch. Before correction it failed with presented geometry `Some(PropertySerial(1))` instead of `Some(PropertySerial(2))`. Presentation state now owns one ordered submitted-frame identity `(PresentationEpoch, PropertySerial)`. It accepts a newer epoch, or a strictly newer serial within the current epoch; it rejects future epochs, older epochs even with a higher serial, exact duplicates, and same-epoch serial regressions. Candidate and active-refresh submissions use this one gate, and every admitted frame installs its exact offsets, layout, stack, and `SpatialSnapshot` atomically. The independent `refreshes_visible`/`same_structure` admission authority is deleted. Platform IME application also requires the exact admitted frame identity before resolving a receipt. The red witness is green and its inverse proves that a late serial-1 receipt cannot regress serial-2 geometry. Two residency tests that had asserted geometry stayed on an earlier serial after actually submitting a newer same-epoch settle frame now require the last submitted serial instead. The 41 runtime, six session-window, 37 platform, 55 residency, 11 native-surface, present-submitted, IME, spatial-snapshot, formatter, and all-target compilation rails are green. The complete workspace suite is also green: 1,362 library tests passed with four intentional hardware ignores, three renderer-debug non-hardware tests passed with 27 hardware ignores, and two example tests passed. Both retained-plan release GPU controls pass. The release gallery rebuilt at `2026-07-17 00:12:44`, size 18,832,384 bytes. Sustained native reversal/drag confirmation remains pending; SC-010 is still open.

Receipts `control-gallery-500px-idle-1784265093761.txt` and `control-gallery-500px-idle-1784265161712.txt` do not test that rebuilt binary: they were written at `00:11:33` and `00:12:41`, while the corrected executable did not land until `00:12:44`. They remain valid additional controls for the prior binary but are not a verdict on submitted-frame identity admission. The latter reproduces the old failure strongly—182 inputs, only two selected candidates, one follow-up, and 20,310,757 pixels of maximum desired/resident lag—but must not be attributed to the correction.

The first receipts definitely produced by that rebuilt binary, `control-gallery-500px-idle-1784265227796.txt` and `control-gallery-500px-idle-1784265241052.txt`, reject submitted-frame serial admission as sufficient. They record 318/471 present-submitted frames with zero skipped frames, negligible preparation maxima of 1,693/1,830 microseconds, and exact final property convergence, but only one selected residency candidate and zero follow-ups in the first receipt and one follow-up in the second. Maximum desired/resident lag remained 14,804,841 pixels. This localized the remaining defect to a stranded selected front rather than rendering, acquisition, or property-serial admission.

The native pending-presentation owner was replacing the in-flight selected presentation whenever a newer frame had the same stack structure. Residency retirement correctly accepts only the exact selected epoch, so a selected presentation with identity A could be replaced by structurally equal identity B; B's completion could not retire A, and no final latest-intent follow-up was authored. The deterministic red witness uses real table-generated active, selected, coalesced, and overtaking presentations. Before correction it observed preparing epoch 3 where selected epoch 2 was required. Native preparation is now immutable after selection: only exact `Arc` identity counts as a duplicate, an exact preparing/latest retry is ignored, and the newest distinct intent may replace only the queued `latest` slot. The structural-equality replacement authority is deleted. The witness requires the exact selected identity to complete and one final latest-intent follow-up; its inverse proves an unrelated newer completion cannot retire that front.

Autonomous release-native testing closes the behavioral boundary. Four hundred aggressive forward wheel inputs moved the 500-pixel table from roughly row 47 to row 4,713; 350 reverse inputs crossed the former row-2,000 stall region to roughly row 630; a long forward thumb drag jumped to row 815,588; and a reverse drag returned to row 153,661. Segmented 100-input samples advanced 153,661 -> 154,828 -> 155,995 and reversed to 154,828 while interaction was still active, proving continuous progress rather than end-only catch-up. Every captured frame was populated, and final convergence required no later click or unrelated event.

Receipts `control-gallery-500px-idle-1784266131025.txt` and `control-gallery-500px-idle-1784266180280.txt` record 421/655 attempted and present-submitted frames with zero skips, zero virtual-residency rejection, zero pipeline cancellation, zero scheduler candidate supersession, zero preparation deadline misses, and exact final candidate/GPU/present serials 264/264/264 and 432/432/432. Their direct schedules plus follow-ups equal selections exactly: 5 + 147 = 152 and 8 + 209 = 217. Maximum preparation was 2,165 microseconds; acquire p95 was 35/41 microseconds. Deliberate thumb jumps account for the large desired/resident-lag maximum and manual pauses account for long aggregate frame-interval tails; continuous intermediate screenshots and exact final serial convergence are the governing behavioral evidence.

The release stress editor supplies the independent text payload pass. It loaded the generated Unicode stress document, disabled wrapping, scrolled aggressively from the beginning to approximately line 3,728 and reversed to approximately line 936 with fully populated frames, moved the horizontal viewport by thumb drag, accepted an 11-character edit in 428 milliseconds including a fixed 300-millisecond observation delay, and repainted the active unwrapped document after a maximize/resize to a 4,096-by-1,114 capture. The user had separately confirmed that large unwrapped text input no longer exhibited the original delay. Text and table therefore exercise the same native transition boundary without payload-specific scheduling.

The final release GPU sweep passes all 27 opt-in witnesses serially, including 40 Tier A cases, all 10 mutation controls, pending/active projection, post-present retained-plan continuation, atlas retention, residency crossing, skipped generation, scale replacement, and the corrected retained-scroll economics fixture. That fixture now authors a real predecessor-based update: one dirty inner scroll source produces three indexed visits/lookups, one 16-byte sparse write, zero full-transfer reasons, exact pixels, and zero content work. The prior 768-byte dense result was a stale fixture that falsely declared the unchanged outer scroll dirty and omitted predecessor identity.

The executable closure matrix passes 40 Tier A cases, 10 negative executions, all 13 property-economics cases, skipped-generation recovery, 1.0-to-1.25 scale replacement, all three payload residency crossings, and `table-scroll-work` at 1.0/1.25/1.5/1.75/2.0. The final table receipt is identical at every scale: 528 property bytes split as 64 node, 272 scroll, and 192 retained-text bytes; one dirty source; two write ranges; zero semantic/content/shaping/resource/plan-rebuild work; and one plan reuse. This supersedes the older 464-byte fixture count without changing its bounded warm-work conclusion.

Formatter and diff checks pass. The complete all-target/all-feature suite passes 1,363 library tests with four intentional hardware ignores, three renderer-debug non-hardware tests with 27 hardware ignores, and two example tests. Eighteen manifest/receipt/census Python checks pass. The repeated source census finds no retired ambiguous generation names and classifies every remaining production hit as pre-candidate construction, the candidate-owned topology, a compiled GPU adapter, the submitted spatial snapshot, or an independently owned input/residency/scheduling/property mechanism. SC-010 and the campaign are closed.

The glyphon source copy is audited separately from scheduler closure. It is not in the native residency control path and is not evidence for this freeze. Compared with pinned crates.io glyphon 0.11.0, the actual source delta is 55 inserted/changed lines across `lib.rs`, `shader.wgsl`, `text_atlas.rs`, `text_render.rs`, and `viewport.rs`. One capability records opaque prepared glyph cache keys so live retained text can reassert atlas allocations after `TextAtlas::trim`; the other adds a device-space offset to the existing 16-byte viewport uniform. wgpu_l3 has one call boundary for each: retained atlas trimming and copy-on-write retained transform viewport preparation. Upstream glyphon `main` currently declares 0.12.0, uses generation-based atlas eviction, retains CPU glyph vertices but exposes no prepared-allocation retention operation, and still exposes only viewport resolution with no render offset. Removing the source copy now would therefore require either retained-text repreparation, disabling bounded atlas eviction, an offscreen scroll texture, or a wgpu_l3-owned text pipeline. None is a smaller or better-proven SC-010 change. Keep the two-capability patch temporarily, forbid unrelated local glyphon behavior, preserve the atlas-pressure and retained-scroll negative controls, and remove the source copy when upstream accepts equivalent APIs or a separately audited renderer-owned adapter replaces them.

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

## 11. Campaign disposition

The campaign is closed at **SC-010**. Future scrolling changes must preserve the closure theorem, rerun the bounded source census and relevant independent oracles, and add new evidence rather than rewriting failed receipts. The retained glyphon source copy remains the separately audited two-capability dependency described above; its eventual upstreaming or adapter replacement is not an open scrolling-architecture loop.

SC-000 added bounded diagnostics, receipt vocabulary, property-write attribution, a source census, and a deterministic test manifest. SC-001 froze the independent payload-neutral oracle and negative controls. SC-002 replaced distributed renderer-side ancestry with one candidate-owned topology and closed all 40 Tier A executions. SC-003 closed indexed dirty property production and sparse/dense transfer economics. SC-004 separated requested intent, resident acceptance, candidate property generations, and `present_submitted`, including explicit renderer recovery across skipped generations. SC-005 closed target-local high-resolution input accumulation while keeping all visual state integral. SC-006 deleted the completion-anchored software cadence owner and unified immediate deduplicated redraw issuance with a full causal trace. SC-007 unified the residency demand, added generation-attributed CPU/GPU work, closed the exact 18-case suite, and bounded large-text glyph admission. SC-008 made runtime input, popup surfaces, and IME consume one lazy present-submitted spatial evaluator. SC-009 froze guarded edit, moving horizontal projection, caret reveal, and payload-topology reuse as separate executable currencies. Source deletion and whole-campaign closure are now the ready loop.
