# Retained Renderer campaign — The Edges Teach Inward

Status: **in flight; Checkpoints 0–8 complete, Checkpoint 9 in progress**. One-Way
Internals is paused at the independently green R5-70 boundary. The renderer
territory was claimed from starting HEAD `24bd0768`; the local baseline, WARP,
PIX, code-owned instrumentation, admission, and verification bracket is green.
The original paired 60 Hz iGPU receipt gate was removed by explicit campaign
correction on 2026-07-14: field hardware remains useful corroboration, but no
external machine owns campaign progress or acceptance.

The development-machine restart requested after the Checkpoint 3 implementation
safety commits completed successfully. The campaign resumed from the ledger,
re-ran the complete local boundary, and closed Checkpoint 3 without an external
machine, person, network service, or returned artifact.

This is a replacement campaign, not a collection of local renderer tweaks. It
replaces the upper rendering mechanism while preserving the lower surface seam
that the popup, material, presentation-clock, and renderer-economics campaigns
have already proved.

## Opening constitution

> The inside of a window obeys the popup laws already sealed at its edge.
> Content revisions mint scene commits. Parameter animation does not mint
> content revisions. Presentation activates and receipts commits without
> altering them. Each handoff owns disjoint fields, and the renderer realizes
> their combined active state without marrying their clocks.

Two corollaries govern the whole campaign:

> Structure belongs to the commit; values belong to property state.

> Each field has exactly one mutation clock.

These are not renderer preferences. They are the presentation, exposure,
identity, and one-owner laws already practiced by popup generations, material
regions, presented geometry, hover projection, and native fade timelines.
The rewrite makes ordinary window content obey them.

## Mission

Replace the per-frame flattened rendering core with a retained scene and a
retained GPU realization designed for a 60 Hz interaction budget on the
integrated-GPU Windows class that exposed the defect. Acceptance is owned by
code-observed work, refresh-relative local timing, GPU topology, and exact
semantic witnesses; an iGPU run may corroborate that evidence but cannot gate
the campaign.

The required first destination is Qt/GTK class:

- stable scene identity carried from composition;
- cached scene substructure and per-node revisions;
- retained GPU geometry and text preparation;
- renderer-owned global batching and opacity classification;
- property-only transform, scroll, opacity, and admitted effect updates;
- no unconditional full-window intermediate and blit;
- synchronous, bounded-cheap commit activation;
- exact popup, material, alpha, color, recovery, and surface behavior.

Chromium's pending/active compositor, raster workers, tiling, and independent
display clock are a named upgrade class, not an implied first-stage debt. They
are admitted only by the post-retention measurements in Checkpoint 7.

The campaign succeeds only when the new renderer is the sole renderer. A
faster parallel path beside the old system is not completion.

## Authority

In descending order:

1. the opening constitution and the existing one-owner, identity, exposure,
   presentation-clock, coordinate, and alpha laws;
2. observed behavior and the weak-GPU failure that exposed the performance gap;
3. `docs/master_design.md` and narrower completed campaign doctrine;
4. the live ownership, lifecycle, and dependency graph at ignition;
5. measurements and witnesses produced by this campaign;
6. named industry precedent;
7. convenience, familiarity, and the current implementation shape.

The campaign carries one interpretation rule:

> Laws are inviolable; shapes are candidates.

`paint::Scene`, `RenderBatch`, `PreparedScene`, `SceneEncoder`, the one-frame
quad arena population, the scale-flattening bridge, and the window-wide
`Paint` / `Layout` / `Rebuild` trichotomy have no constitutional standing.
They may be deleted. `composition::tree::NodeId`, one derivation with many
consumers, successful-present receipts, popup-local generations, and the
presentation clock do have standing. They survive even when the shapes that
currently carry or erase them do not.

## Independent execution and measurement law

> The campaign must be runnable, resumable, verifiable, and closeable on the
> development machine alone. No external machine, person, network service, or
> returned artifact may own a checkpoint transition or the definition of done.

The weak-GPU report establishes the defect class; it does not become a remote
build server. Optional hardware reports may corroborate a result and may expose
a new defect, but their absence is never an incomplete campaign receipt.

Measurement mechanisms are required in this order:

1. deterministic semantic/topology witnesses and in-code benchmarks in a
   dedicated development-only renderer debug crate;
2. temporary, feature-gated owner-local logging/counters when the debug crate
   cannot yet observe the required owner directly;
3. PIX or another external profiling program only as a fallback when the first
   two cannot answer a named question and no code-owned solution exists;
4. optional external-hardware corroboration.

The debug crate may consume a narrow development-only support boundary from the
main crate; production builds do not depend on it and it cannot become a second
renderer API. Every benchmark names workload, warmup, sample count, environment,
and acceptance currency. Temporary logging names its retirement checkpoint when
introduced. Temporary logging is a first-class code-owned measurement path, not
an export bridge for PIX or another external program; its own counters and
timings may close the named question. External programs do not displace
debug-crate benchmarking or temporary logging merely because they are available;
the ledger must first record why both code-owned mechanisms cannot answer the
question and why no additional in-code witness is practical. At burn-down
temporary logging is deleted unless promoted to a direct, stable
architecture/performance witness with an explicit owner. Interactive pacing,
profiler replay duration, and anecdotal feel never substitute for code-owned
timings and counters.

## Protected lower seam

The hard-won replacement boundary is:

```text
renderer realization
    -> render::Context + render::Canvas
    -> render::Surface acquire / submit / present
    -> platform/native tenancy and receipts
```

The following survive by default:

- `render::Context`, adapter/device/queue ownership, and device-loss boundary;
- `render::Canvas` and `render::Surface` acquisition, resize, recovery,
  submission, and present receipts;
- DX12-first Windows selection and the explicit backend override;
- the DirectComposition tenancy ladder and popup host/session distinction;
- premultiplied popup packing and its exact piecewise sRGB transfer;
- native material reports and residual renderer material resolution;
- popup exposure, concealment, generation, and fade laws;
- the presentation clock's attachment to successful surface outcomes;
- existing text/glyph caches and shaders unless a checkpoint supplies direct
  evidence for replacement.

This is a presumption, not a wall. If the requirements-first contract proves
that an upstream or lower contract is wrong, the renderer campaign has
standing to correct it through normal admission: trace the owner, state the
failed requirement, make the smallest honest correction, delete the displaced
path, and record the receipt. No requirement is silently dropped to preserve
an existing shape.

## Current indictment

The formulation snapshot at R5-70 has the pieces of a retained system but
destroys their relationship before rendering:

- `composition::tree::NodeId` is retained identity and
  `layout::Frame::node_id()` carries it through layout;
- `composition::tree::Changes` already reports structural additions and
  removals and is reserved as the future AccessKit change stream;
- `scene::Scene` paints a fresh flat `Vec<Primitive>` and no primitive retains
  its composition identity;
- `render::scene::to_paint_scene_at_scale` flattens that list again into a
  fresh private `paint::Scene` at every presentation;
- `Renderer::draw` recreates descriptive batches, clears the CPU geometry
  arena, prepares every shape and text batch, and uploads the resulting
  geometry every frame;
- each analytic shape expands into six vertices carrying duplicated shape
  data, even though the shader needs one unit quad plus one instance record;
- the ordinary opaque-window path clears a full-window composition texture,
  renders into it, and blits it to the swapchain even when no semantic effect
  requires an intermediate;
- `scene::Presentation` currently stores both a content/model revision and a
  `PresentationEpoch`, placing presentation's clock on a composition-time
  artifact;
- window-wide invalidation names how much of the old pipeline to rerun rather
  than which retained identities changed;
- the renderer has no retained per-node resource lifetime, property-only
  update path, or node-level opacity/effect classification.

The earlier renderer-economics campaign removed clip-per-item layers,
buffer-per-batch allocation, and pass-per-batch encoding. Its 500-pixel table
witness reached 636 vertices, 91,584 upload bytes, six draw passes, and DX12
draw p95 below 2.6 ms on the discrete-GPU development machine. Those wins are
protected. They also make the remaining defect more precise: a fast machine
can hide full regeneration and full-surface bandwidth that an integrated GPU
cannot.

## Domain map

The campaign names domains by the questions they answer, not by data
structures they happen to use.

| Domain | Owns | Must not own | Handoff |
| --- | --- | --- | --- |
| `composition` | Retained semantic identity, reconciliation, semantic change facts | GPU resources, paint order, presentation epochs | `composition::tree::NodeId` and `composition::tree::Changes` |
| `layout` | Logical measurement, bounds, clips, and hit geometry keyed by composition identity | GPU realization, animation timing, application commands | Keyed geometry consumed by scene projection and input |
| `scene` | A retained renderer-independent description of what must appear: typed content, order, local bounds, property topology, opacity/effect declarations | GPU buffers, surface acquisition, input routing, presentation activation | `scene::{Commit, Node, Content, Properties}` |
| `presentation` | Which commit is active, activation stamps, property sampling/timing, successful-present receipts, candidate-versus-visible projections | Content creation, semantic identity, GPU batch topology | `presentation::Active` plus the property snapshot sampled for a frame |
| `render` | GPU resources, instancing, batching, opacity planning, effect islands, glyph atlas use, command encoding, resource readiness | Application intent, semantic identity creation, animation policy, surface exposure | Realizes the active scene state into `Canvas` |
| `surface` / platform | Format, acquisition, resize, loss/recovery, submission, present, native tenancy and exposure | Scene meaning, property policy, batch construction | Present outcome and native receipts |

The expected flow is:

```text
app / widgets
    -> composition          identity + Changes
    -> layout               geometry keyed by NodeId
    -> scene                retained Commit + property topology
    -> presentation         Active + property state + activation/receipts
    -> render               retained GPU realization
    -> Canvas / Surface     acquire, submit, present
    -> platform             tenancy, exposure, native receipts
```

`Tree` is an implementation shape, not a new domain. A scene implementation
may use a tree, ordered arena, structurally shared nodes, or several property
tables. It does not create a top-level `tree` owner merely because hierarchy
exists.

## Identity, revisions, and addresses

There is no `RenderNodeId`.

The one semantic identity is `composition::tree::NodeId`, carried by
`layout::Frame` into `scene::Node` and consumed by renderer caches. It is never
reconstructed from primitive contents, position, order, hashes, or pointer
addresses. Reordering changes order; it does not manufacture identity.

Revisions are currencies, not identities:

- a commit revision says which semantic structure/content snapshot exists;
- a per-node content revision says whether that identity's realizable content
  changed;
- a property serial, if the final contract needs one, says which value
  snapshot a property update represents;
- a presentation epoch says which activation/presentation attempt is being
  acknowledged.

An internal array index may address a transform, clip, effect, instance, or
GPU allocation inside one commit. It is not semantic identity, may not escape
that commit's lifetime, and may not become a second cache key. Property-table
keys must be carried from or structurally derived from the owning `NodeId` and
property kind; they are not independently minted identities.

## The three handoffs

The current `draw(scene)` conflates three distinct handoffs. The replacement
keeps them separate even if the first implementation executes them
synchronously on one thread.

| Handoff | May change | Must not change | Result |
| --- | --- | --- | --- |
| Semantic commit | Node existence/order, typed content, content revisions, local bounds, property topology, effect/resource declarations | Presentation epoch, current property values, surface state | An immutable `scene::Commit` |
| Property tick | Values for already-declared transforms, scroll offsets, opacity, clips, and admitted effect parameters | Node topology, content revision, effect class/resource envelope, activation | A new `scene::Properties` value snapshot or bounded delta |
| Presentation event | Active commit pointer, activation stamp, sampled property receipt, exposed/presented acknowledgements | Commit contents, property values, semantic identity | A drawable active state and a receipt for what became visible |

The minimum epoch-bearing shape is conceptually:

```rust,ignore
presentation::Active {
    commit: Arc<scene::Commit>,
    activation: window::PresentationEpoch,
}
```

The exact API is a Checkpoint 1 result, not a formulation assumption. The
important negative is already final: `scene::Commit` does not contain a
`PresentationEpoch`. Presentation stamps activation when it activates a
commit and binds surface receipts to that stamp without altering the commit.

The likely renderer vocabulary is similarly illustrative, not pre-approved:

```rust,ignore
renderer.synchronize(&commit, &changes);
renderer.update_properties(&properties);
renderer.draw(&context, &mut canvas, &active);
```

A better concrete contract may collapse or rename these calls. It must still
make the three handoffs and their mutation clocks independently witnessable.

## Property structure and values

Adding a transform, clip, opacity group, scroll relationship, or blur effect
changes topology and therefore mints a semantic commit. Advancing the matrix,
clip rectangle, opacity, scroll offset, or admitted blur parameter changes
property state and therefore does not mint a content revision.

Effect parameters remain property-only only inside a structure declared by
the commit. For example, a blur declaration may reserve an effect island,
edge mode, and maximum sampling envelope. Sigma may tick within that envelope.
Adding the blur, removing it, changing its edge behavior, or exceeding its
declared resource envelope requires a commit. This lets blur animate as a
parameter without pretending that resource topology is a float.

Checkpoint 1 must publish an explicit property admission table. Transform,
scroll, and opacity are the mandatory first entries. Clip and effect
parameters are admitted only when their failure, bounds, hit-testing, and
resource models are complete. An unsupported change takes the semantic-commit
path; it never becomes an untracked renderer mutation.

## Capability-boundary law

The renderer campaign practices the following general rule:

> An open boundary states the smallest capabilities it requires through
> std-first, boundary-owned traits; suppliers realize those capabilities. A
> closed boundary uses exhaustive typed data. Traits never conceal ownership,
> manufacture openness, or require erasure without evidence.

The scene-to-renderer boundary is closed and performance-sensitive. Its normal
contract is therefore typed `scene::Content`, not an arbitrary `Renderable`
trait object and not a callback such as `render(&mut Renderer)`.

The slogan “hand the renderer something renderable” is realized by handing it
a value whose content species is exhaustively intelligible. The value
describes what it is. The renderer exclusively decides how all values are
realized together. Otherwise each content object can manufacture private draw
calls, buffers, clips, or effect passes and global batching becomes
unrepresentable.

Operations such as transform, opacity, clip, and blur are typed scene
structure and property relationships, not imperative methods executed by
content. A future genuinely open resource supplier—an application texture
source, for example—may earn a narrow capability trait at its owning boundary.
That does not make drawing itself open-ended.

Trait admission follows six rails:

1. use an existing standard trait when it already names the capability;
2. admit a framework trait only for a framework-owned capability;
3. require a genuinely open implementor axis or a second real implementor;
4. use an enum or other exhaustive data for a framework-closed species set;
5. prefer static realization before erasure;
6. reject traits that mirror one concrete consumer or launder a back-edge.

## Requirements-first contract ledger

Checkpoint 1 starts from the renderer's requirements and diffs them against
the live framework. It does not reverse-engineer the contract from whatever
the current flattened scene happens to expose.

| Required fact | Formulation snapshot | Required disposition |
| --- | --- | --- |
| Stable semantic node identity | `composition::tree::NodeId` reaches `layout::Frame`, then is erased during scene painting | Carry it into every retained `scene::Node`; mint nothing new |
| One authoritative change stream | `composition::tree::Changes` reports add/remove and cleanup facts | Improve it at its owner for per-node revision needs; renderer becomes its second consumer beside future AccessKit |
| Complete typed content | `layout::FrameContent` and `scene::Primitive` are typed, but the result is a flat primitive list | Define closed `scene::Content` without reintroducing optional payload clusters |
| Bounds and order | Layout owns keyed geometry; flat scene retains only primitive order | Carry local bounds, clip/effect ancestry, and stable order into the commit |
| Property topology | Transforms and opacity are embedded in primitives/groups | Separate declared relationships from ticked values |
| Opacity/effect class | Individual primitives know alpha/material details | Make enough classification available for renderer-global planning without caller-authored batch hints |
| Per-node revisions | Only window-wide invalidation and whole-scene regeneration exist | Derive node revision changes from the authoritative stream; no parallel `render::Dirty*` system |
| Commit versus activation | `scene::Presentation` carries revision and `PresentationEpoch` together | Move activation currency to presentation; commit remains presentation-agnostic |
| Pixel oracle | Deep-tier readbacks cover alpha, glyphs, groups, and popup packing separately | Generalize them to old/new rendering of the same prepared scene |
| Surface seam | `Context` / `Canvas` / `Surface` already isolate acquire/present and platform realization | Preserve and reuse it |

Every gap receives one of three recorded outcomes:

- satisfied by an existing owner and carried through;
- corrected at the existing owner through a One-Way cell;
- explicitly rejected with a named deviation and evidence.

“The old path did not provide it” is not a deviation. It is the defect under
investigation.

## Change-stream law

`composition::tree::Changes` is the root change stream. The renderer does not
create a second invalidation authority beside it.

The current stream is intentionally too coarse for the final renderer. It may
be enlarged at the composition owner to distinguish the semantic facts the
retained projection needs: addition, removal, reparenting/order where
relevant, and content-affecting change. Layout and scene may enrich the same
flow with their authoritative keyed results as it crosses those boundaries;
they do not accept caller-authored dirty bits or infer identity from flattened
primitives.

Global facts remain possible. Scale, surface size, theme replacement, font
atlas loss, or device loss can truthfully invalidate a root, resource class,
or every node. “Per-node” does not require lying about a global cause. It
requires that the cause be explicit, owned, and projected into the same
revision flow rather than selecting one of three old pipeline recipes.

The final API must let future AccessKit consume semantic additions, removals,
and changes without importing renderer vocabulary. If a proposed change kind
can only be explained as a GPU action, it belongs in renderer synchronization,
not `composition::tree::Changes`.

## Scroll is the first property client

Scroll has two truths:

1. the viewport/scroll transform is a property value;
2. the set of virtualized content resident around that viewport is semantic
   structure and therefore a commit concern.

Inside the retained guard window, one scroll event updates authoritative
scroll truth and the corresponding property value. The next redraw samples
the latest value. It performs:

- zero scene painting;
- zero text shaping;
- zero quad or primitive preparation;
- zero content-buffer upload;
- zero content revision minting.

Those are literal zero-counter witnesses, not relative improvements.

Virtual lists and tables retain measured guard rows around the viewport. The
Qt-class presenter sizes that guard so replenishment commits are bounded and
cheap enough that synchronous activation does not create a visible hitch.
Crossing the guard boundary may create a semantic commit in this stage. The
stronger law—replenishment cannot prevent the active state from presenting—is
Chromium-class and is not claimed until a pending/active path is admitted and
implemented.

Guard sizing is not a magic row count. Checkpoint 6 derives it from measured
materialization/realization time, maximum accepted input delta, row-height
regimes, and the pinned 16.667 ms ceiling. One million logical rows must still
produce bounded resident nodes and GPU resources.

### Scroll and visible input truth

Property-only rendering may not move pixels away from hit testing.

Authoritative scroll truth may run ahead, candidate property state may be
sampled for a frame, and only a successful surface receipt makes that sampled
property state visible to input. The receipt therefore identifies both the
active commit activation and the property snapshot used to draw it. A skipped
frame promotes neither.

Presented geometry becomes the combination of:

- the active commit's keyed layout geometry;
- the successfully presented property snapshot;
- the existing scale and coordinate-space projection.

Pointer hit testing, capture routing, cursor projection, hover, and popup
placement consume that same visible combination. They do not ask the renderer
where it drew, and they do not use a newer unpresented scroll value. This is
one derivation with many consumers, extended to presentation transforms.

Many scroll inputs may still coalesce into one frame. Every semantic delta
survives in the latest scroll truth; only obsolete visual samples disappear.

## Renderer realization laws

The new renderer owns global realization. Its required laws are:

1. GPU resources are keyed by carried node identity plus the relevant content
   revision, never by flattened primitive hashes.
2. An unchanged node reuses its GPU realization without CPU re-preparation or
   content upload.
3. Node removal, window departure, device loss, and renderer recreation each
   have complete cleanup paths.
4. Property updates write only bounded property/instance state for the changed
   properties; they do not rebuild content buffers.
5. Analytic quads use a static unit-quad topology with instance data by
   default. Any content species that keeps duplicated six-vertex records must
   present a measured reason.
6. Text shaping and glyph preparation are retained by node/content revision;
   property ticks reuse them. The existing glyph atlas and inline caches are
   inputs, not discarded accomplishments.
7. Batch planning is renderer-global. Scene nodes declare content, order,
   bounds, opacity/effect facts, and relationships; they do not declare GPU
   batches.
8. Opaque and blended content are classified explicitly. Safe opaque
   reordering/early rejection may be admitted; blended order remains exact.
9. CPU occlusion culling is measurement-gated. Opacity classification is
   required; an expensive occlusion walk is not assumed to be free.
10. Every offscreen target has a semantic owner, bounded effect envelope, and
    diagnostic receipt. Ordinary opaque windows with no sampling effect have
    zero extra full-surface intermediate clears and zero full-surface blits.
11. Popup packing remains a named full-popup conversion where required by the
    Windows premultiplied surface contract. It is not miscounted as accidental
    ordinary-window work.
12. Retained memory is bounded by active scene content plus named capped
    caches/pools. High-water pools report bytes and entries; removed nodes do
    not leave immortal allocations.

Damage tracking is deliberately absent from these laws. Retention first makes
CPU preparation proportional to change. Direct rendering and effect islands
remove accidental bandwidth. Checkpoint 7 measures the residual before damage
or partial present can be admitted.

## Named precedents and deviations

Every major mechanism carries a precedent or a named reason to deviate.

| Mechanism | Precedent | Campaign use |
| --- | --- | --- |
| Retained renderer-independent scene | GTK 4 cached GSK render nodes; Qt Quick `QSGNode` scene graph | Retain scene substructure and regenerate only changed identities |
| Retained GPU geometry and batch roots | Qt Quick default scene-graph renderer | Keep unchanged geometry resident; make scrolling a transform update |
| Instanced analytic quads | iced's per-instance quad buffers and `draw(0..6, instances)` | Replace six duplicated vertices per analytic shape with a unit quad plus instance data |
| Bounded effect islands | Flutter's documented `saveLayer`/render-target-switch cost | Offscreen only for a named group, filter, material, or popup pack |
| Property topology separate from content | Chromium transform/clip/effect property trees | Make updates proportional to interesting properties rather than all nodes |
| Active state remains drawable | Chromium pending/active layer-tree law | Reserved upgrade if synchronous commit preparation misses deadlines |
| Parameter animation without content generations | Existing DComp popup fade | Generalize inward for scroll, transforms, opacity, and admitted effects |
| Activation and exposure receipts | Existing popup generation and presentation-clock campaigns | Keep commit, property, activation, and visibility clocks distinct |

Primary source registry:

- Qt Quick scene graph renderer:
  <https://doc.qt.io/qt-6.8/qtquick-visualcanvas-scenegraph-renderer.html>
- GTK 4 drawing model:
  <https://docs.gtk.org/gtk4/drawing-model.html>
- iced solid-quad renderer:
  <https://github.com/iced-rs/iced/blob/master/wgpu/src/quad/solid.rs>
- Flutter performance guidance:
  <https://docs.flutter.dev/perf/best-practices>
- Chromium compositor architecture:
  <https://chromium.googlesource.com/chromium/src.git/+/HEAD/docs/how_cc_works.md>
- wgpu adapter selection and fallback contract:
  <https://docs.rs/wgpu/29.0.3/wgpu/struct.RequestAdapterOptionsBase.html>

Named deviations from Chromium are intentional at formulation:

- application windows are bounded, trusted UI rather than unbounded
  untrusted documents;
- no raster worker pool, checkerboarding, multiresolution tiles, GPU process,
  or delegated-layer explosion is admitted initially;
- one synchronous presentation owner is cheaper until measurements prove that
  commit preparation must become deadline-independent;
- wgpu remains the graphics abstraction and the existing Windows
  DirectComposition path remains the platform edge;
- popup DComp animation is retained rather than reimplemented inside the GPU
  renderer.

Named deviations are not permanent immunity. New receipts may reopen them.

## Territory claim and One-Way cleanup

### Ignition receipt — 2026-07-14

The user countersigned execution with One-Way paused at its independently green
R5-70 boundary. Starting production HEAD is `24bd0768`. The only pre-ignition
worktree changes are this campaign formulation and its roadmap entry; both are
campaign-owned and protected. The territory below is exclusively owned by this
campaign until closeout. The matching handoff is recorded in the One-Way
ledger. Any cleanup finding inside the claim becomes an `OW-*` cell here;
One-Way remains paused and does not select overlapping work.

At ignition, this campaign publishes a semantic territory list covering the
live owners of:

- `composition::tree::Changes` as changed by renderer requirements;
- keyed layout-to-scene projection;
- `scene::{Commit, Node, Content, Properties}` and scene painting;
- runtime presentation preparation, activation, and visible property receipts;
- render planning, resources, shaders, text preparation, diagnostics, and
  readback witnesses;
- the native surface handoff, but not unrelated platform realization;
- architecture tests and the doctrine/roadmap entries affected by the rewrite.

One-Way Internals does not concurrently select cells in claimed territory
without explicit coordination. The claim is not a ban on urgent fixes; it is
one owner for structural motion while the replacement contract is being
built.

Every production module touched by the rewrite leaves at One-Way standard.
The renderer campaign uses the established loop verbatim:

**Select → Trace → Model → Challenge → Admit → Reduce → Rewire → Prove →
Ratchet → Re-scan.**

Cleanup findings are recorded as bounded `OW-*` cells in this campaign's
ledger using the One-Way cell format: question, complete trace, owner graph,
admission/resistance, displaced path, implementation/deletion, gauge delta,
proof, and fixed-point result. This keeps one cleanup physics. It does not
authorize opportunistic renaming, module churn, or aesthetic work merely
because a diff touched a file.

At each checkpoint boundary:

- re-scan every modified production module and its immediate consumers;
- census new/removed `pub(crate)`, aliases, helpers, panic/expect paths,
  allowances, compatibility branches, and concealed dependency arrows;
- delete old-path helpers made unreachable by that checkpoint rather than
  postponing every deletion to the end;
- retain only the legacy pieces still required by the equivalence oracle;
- record resistance where cleanup would widen scope without evidence.

## Checkpoint board

| Checkpoint | State | Required outcome |
| --- | --- | --- |
| 0. Claim territory and bracket the defect | Complete | Clean baseline, WARP correctness path, PIX matrix, code-owned counters and thresholds pinned |
| 1. Ratify the retained-scene contract | Complete | Requirements-first API, ownership, revision, property, clock, cleanup, and gap ledgers approved |
| 2. Build the equivalence oracle | Complete | Same commit through legacy and new adapters with deep-tier readback comparison and a mandatory retirement plan |
| 3. Retain scene identity and revisions | Complete | NodeId survives painting; unchanged substructure reused; window-wide invalidation begins retirement |
| 4. Retain GPU realization | Complete | Identity/revision-keyed resources, instanced primitives, retained text prep, bounded cleanup and loss recovery |
| 5. Make render work semantic | Complete | Global planning, direct ordinary-window path, bounded effect islands, explicit opacity classes, no accidental full blit |
| 6. Make scroll a property tick | Complete | Literal zero work counters in-window; receipted property-aware hit testing; bounded-cheap synchronous replenishment |
| 7. Prove Qt class and decide the ceiling | Complete | Instrumented Qt-class verdict; evidence-based accept/reject decisions for pending/active, render thread, damage, and partial present |
| 8. Optional Chromium-class upgrade | Complete; admitted by Checkpoint 7 | Active remains drawable while pending prepares; atomic activation and deadline independence proved |
| 9. Burn down the old species | In progress | Legacy renderer/oracle adapter/flattening/orphans deleted; tombstones and new topology witnesses planted |
| 10. Close out and teach master design | Pending | Full matrix green, instrumented acceptance, One-Way fixed point, roadmap/design synchronized, sole renderer proved |

## Checkpoint 0 — claim territory and bracket the defect

Checkpoint 0 changes instrumentation and harnesses only. No renderer mechanism
changes until the causal bracket and acceptance currencies are pinned.

### Starting state — closed

There is no continuing ignition gate. One-Way is paused at its clean R5-70
boundary, the renderer territory handoff is recorded in both campaign ledgers,
all unrelated worktree state is named and protected, and the pre-campaign full
verification ritual and deep GPU tier are green. Those are historical facts of
the ignition receipt above, not conditions that a compacted or resumed task may
reopen.

The remaining environment requirement belongs to Checkpoint 0 itself: every
performance receipt reports adapter, OS, display refresh, scale, surface
format, alpha mode, present mode, desired latency, presentation system, and
renderer topology before the checkpoint can close.

### Hardware matrix

| Rail | Purpose | Acceptance use |
| --- | --- | --- |
| Development discrete GPU, DX12 | In-code high-resolution profiling and refresh-relative timing; external GPU capture only for an unresolved question | Primary causal, topology, and performance acceptance |
| Available integrated GPU, DX12 | Reproduce the originating user-visible defect when conveniently available | Optional corroboration; never a progress gate |
| Windows fallback adapter, DX12 | Driver-independent correctness, CPU preparation amplification, resource/loss coverage | Correctness only; never treated as an iGPU performance proxy |
| Vulkan where available | Backend-independent comparison | Diagnostic only; absence on a test machine is valid |

The diagnostic build adds `WGPU_L3_FORCE_FALLBACK_ADAPTER=1`, which maps to
the existing `render::context::Options::force_fallback_adapter` request while
`WGPU_BACKEND=dx12` keeps the backend explicit. The receipt records full
`AdapterInfo` and fails the WARP/fallback rail if the selected adapter is not
reported as fallback/CPU-class. No production adapter preference or fallback
policy changes.

WARP is deliberately not a speed threshold. It is software rendering and does
not model shared-memory iGPU bandwidth. Its value is deterministic coverage
and making accidental whole-scene CPU work painfully visible.

### Workload matrix

The deterministic control-gallery harness covers:

- idle redraw and resize at 136, 500, and 800 logical table pixels;
- sustained wheel and trackpad-equivalent scrolling wholly inside a retained
  guard window;
- controlled crossing of a virtual-content guard boundary;
- 100 divider positions coalesced before one redraw;
- hover, selection drag, text selection, text editing, caret blink, and IME;
- command palette open/query/close and menu/submenu open, hover, transition,
  rapid reopen, and scale change;
- transform and opacity animation with stable content;
- ordinary opaque, transparent premultiplied, glass/material, grouped opacity,
  clip, shadow, text, and popup-pack scenes;
- occlusion, minimize/restore, surface loss, device recreation, close/reopen,
  and multi-window teardown.

The instrumented build exposes one explicit local “renderer receipt” action.
It writes a shareable JSON/text artifact and sends nothing over the network.
The receipt contains the environment facts above plus p50/p95/p99/max timings,
frame-deadline misses, counters, memory high-water marks, and workload identity.
These campaign diagnostics remain through Checkpoint 7. At burn-down, each is
either promoted to a direct architectural/performance ratchet or deleted with
the temporary harness; temporary logging is not a new permanent subsystem.

### Required counters

Checkpoint 0 retains the existing diagnostics and adds at least:

- semantic commits created and activated;
- scene nodes added, removed, reused, and rebuilt;
- scene paint calls and painted nodes;
- property ticks, changed property values, and property bytes uploaded;
- text shape calls, glyph prepare calls, and text node reuse;
- primitive/quad preparation calls;
- content upload bytes versus property upload bytes;
- retained GPU resource counts/bytes, creations, replacements, and removals;
- render-plan rebuilds and reused plans;
- opaque, blended, clipped, effect-island, and culled node counts;
- draw calls, passes, pipeline/bind changes, and command preparation time;
- ordinary surface clears, extra full-surface intermediate clears, blits, and
  estimated bytes moved by each;
- surface acquire, encode/submit, present, frame interval, and key-to-present;
- candidate property serial, sampled property serial, successful visible
  property receipt, and skipped attempts;
- virtual guard crossings and replenishment commit time.

Counters distinguish a value sampled for an attempted frame from a value that
became visible. They never call an attempted frame “presented.”

### External-profiler fallback and historical PIX matrix

Code-owned counters and debug-crate readback are the default evidence for the
following DX12 workloads. A matching before/after external capture may be
admitted only when those mechanisms cannot distinguish GPU fill, copy, state
setup, or present ownership and no practical in-code witness can be added:

1. 500-pixel in-window table scroll;
2. 800-pixel guard-boundary replenishment;
3. ordinary opaque window with no sampling effect;
4. in-frame material/effect islands;
5. native popup pack and exposure.

Any admitted external-profiler receipt names the unresolved question first,
then CPU preparation, GPU duration, render-target switches, resource barriers,
draw/pass topology, full-size clears/copies, overdraw, and present wait as
relevant. GPU-capture replay duration is never accepted as timing evidence.

### Threshold pinning

Literal-zero laws are fixed now. Numeric thresholds are pinned from the local
instrumented baseline before Checkpoint 1 production work.

At minimum, final in-window scrolling must report renderer draw p95 inside the
recorded display refresh period and zero renderer-owned deadline misses after
warmup. On the 240 Hz development rail that is 4.167 ms; 16.667 ms remains the
portable 60 Hz ceiling. Guard-boundary replenishment p95 must remain below the
16.667 ms ceiling. Event-injection spacing is not presentation timing evidence,
so frame-interval distributions are reported but do not gate this code-owned
bracket. The campaign additionally requires the literal-zero scroll family,
zero unchanged-content upload, and removal of the unconditional full-surface
intermediate/blit. Those currencies target the weak-GPU defect directly and
cannot be hidden by a fast adapter.

The discrete machine must not regress the completed Pay Once topology or deep
pixel tier. Popup cold/warm exposure keeps its existing semantic thresholds
unless the baseline records a legitimate environment shift.

### Checkpoint 0 evidence ledger — complete

Implementation receipt, 2026-07-14:

- `RendererEnvironment` now records the full adapter report, OS/architecture,
  display/refresh, scale, surface configuration, requested fallback state, and
  presentation system on every renderer receipt.
- `WGPU_L3_FORCE_FALLBACK_ADAPTER=1` is an exact opt-in. The ignored release
  witness
  `render::context::tests::dx12_fallback_adapter_reports_cpu_class` passed on
  DX12 and reported `DeviceType::Cpu`; this is a WARP correctness receipt, not
  a speed threshold.
- renderer, scene, property, text, preparation, upload, resource, opacity,
  command, surface-traffic, acquisition, activation, virtual guard, and timing
  currencies are emitted by `wgpu_l3.renderer_receipt.v1`. Attempted and
  presented frames remain distinct. Distribution fields include sample count,
  p50, p95, p99, and max.
- the control gallery exposes 136/500/800 logical-pixel viewports, a named
  workload field, a visible action, and `Primary+Shift+R`. Choosing a workload
  or viewport starts a fresh measurement session. The action writes a unique
  local text artifact beside the executable and owns no network path.
- live UI verification found and corrected a launcher-dependent relative-path
  failure before accepting the field harness.

Discrete DX12 topology bracket:

- environment: Windows x86-64, NVIDIA GeForce RTX 4070 Ti SUPER, DX12,
  `DxgiFromVisual`, 240 Hz display, 1.25 scale, `Bgra8UnormSrgb`, `Mailbox`,
  desired latency 1;
- isolated 500-pixel in-window scroll: 106/106 attempted/presented, zero guard
  crossings, renderer draw p95 2.675 ms, but 106 scene paints, 27,108 painted
  nodes, 7,309 quad preparations, 7,102 text preparations, 10,250,496 content
  bytes uploaded, 106 plan rebuilds with zero reuse, and 106 whole-surface
  blits moving an estimated 961,916,928 bytes;
- 800-pixel guard-boundary stream: 24/24 guard crossings and replenishment
  commits, replenishment p95 4.692 ms, renderer draw p95 2.852 ms, 67 scene
  paints, 18,728 painted nodes, 7,130,592 content bytes uploaded, 67 plan
  rebuilds, and 608,004,096 estimated blit bytes;
- the 136-pixel resize receipt proves the compact topology path, but its
  automation-spaced frame intervals are not threshold evidence.

PIX receipt, 2026-07-14:

- PIX 2603.25 captured replayable DX12 GPU frames for the ordinary opaque
  500-pixel window, 500-pixel in-window scroll, 800-pixel guard stream,
  effect-baseline frame, and popup-exposure request. Matching New Timing
  Captures were written for ordinary presentation, the 500-pixel scroll, the
  800-pixel guard stream, and popup/material exposure. The local evidence set
  lives under `target/pix-receipts/`; GPU event lists and gold screenshots were
  exported beside the captures so the receipt does not depend on manual PIX
  inspection alone.
- the ordinary GPU frame contains 1,411 recorded events, 132 instanced draws,
  65 buffer copies, two render-target clears, eight render-target bindings,
  129 pipeline binds, 82 scissor writes, and 71 resource barriers. The
  in-window scroll grows to 1,531 events, 143 draws, 80 buffer copies, 140
  pipeline binds, 88 scissor writes, and 79 barriers. The historical PIX
  fallback therefore corroborates the source counters' diagnosis: an ordinary
  scroll frame reconstructs and
  uploads more work than the already-heavy steady frame instead of reducing to
  a property update.
- the 800-pixel guard and effect-baseline frames remain in the same topology
  family: 1,408 events, 132 draws, 65 buffer copies, two clears, eight
  render-target bindings, and 129 pipeline binds each. A manual three-frame
  capture then separated popup exposure into its parent and cold popup-surface
  presents. The exported popup gold image is the 336-by-217 transparent menu,
  not an inferred parent frame; its event list contains 215 events, seven
  draws, nine buffer copies, eight texture copies, four clears, nine
  render-target bindings, seven pipeline binds, and 25 barriers. Warm popup
  opacity remains compositor-owned and emits no renderer present during the
  fade, as required.
- GPU replay time is not used as performance evidence. The New Timing Capture
  files and the renderer receipt are the timing artifacts; the event-list
  counts above are topology evidence only.
- an optional 60 Hz iGPU corroboration bundle is packaged at
  `target/renderer-field-harness.zip` with a release build and exact 500/800
  receipt procedure. Its SHA-256 is
  `3F1DAE3E4667E8172A8AD58283ED63AD8C08C6947D8FCB720D7FDE7FE068B7A7`.
  It may be used later without rediscovering the field protocol, but returning
  the pair is not required for any checkpoint.
- `tools/check_renderer_receipts.py` is the admission clerk for any matching
  local DX12 pair. It rejects ambiguous or incomplete files, mismatched
  machines, too-short samples, a contaminated in-window run, and an
  un-witnessed guard run. `--require-field-igpu-60hz` adds the optional strict
  field rail. Baseline mode reports whether each workload already meets one
  refresh without treating expected failure as malformed evidence;
  `--require-final-renderer-budget` turns the same code-owned currencies into
  the Checkpoint 7 ratchet. Parser/admission tests cover both local and optional
  field policies. The command is:

  ```text
  python tools/check_renderer_receipts.py --in-window <500.txt> --guard <800.txt>
  ```

The bracket identifies the causal species: even the no-boundary scroll
rebuilds, reshapes, prepares, uploads, and full-blits every attempted frame.
Interactive baseline addendum, 2026-07-14: live Control Gallery use reports
that drag-selection highlighting becomes visible only after the drag and typed
text becomes visible only once typing pauses. This is recorded as a pre-existing
frame-scheduling/invalidation defect, not renderer-throughput evidence.
The same session reports table clicks passing through the vertical scrollbar
while the horizontal scrollbar owns its clicks correctly. This is a
source-of-truth deviation: both orientations must derive visible geometry,
hit-testing, capture, and scroll mutation through the same path.
Checkpoint 6 must retest continuous selection and editing presentation so the
campaign neither claims an accidental fix nor carries the idle-flush behavior
through its presentation witnesses; its scrollbar witnesses must also require
horizontal/vertical hit-test and capture parity with zero click-through.

Checkpoint 6 live-regression addendum, 2026-07-15: the in-flight retained-scroll
path made interaction broadly choppy, and a large fast Control Gallery scroll
attempted to create a `Retained Scroll Layer Texture` with Y dimension 16,862
against the selected device's 8,192 limit, causing a fatal WGPU validation
panic. These are campaign defects, not optional field observations. The
checkpoint must bound every retained scroll window independently of total
content/nested descendants, reject any offscreen target above the actual device
limit before WGPU creation, and pass a real release-gallery fast-scroll and
general-interaction smoke before it may close.

The code-owned baseline pins renderer draw p95 at 2.675 ms for in-window scroll
and 2.852 ms for the witnessed guard stream on the 240 Hz DX12 rail, with guard
replenishment p95 at 4.692 ms. Those values are comparison baselines, not proof
that the legacy mechanism is acceptable on weak hardware. Computer-control
pacing is not accepted as display-cadence evidence because its interval samples
  contain injected-event spacing. The literal nonzero code-owned work is the
  causal bracket; the thresholds above and exact-zero laws are the closing
  bracket; the already-collected PIX topology is corroboration. Checkpoint 0 is
  therefore independently green and production contract work may begin.

Verification freeze, 2026-07-14: `cargo fmt --check`,
`cargo check --all-targets`, the 1,154-test library tier (1,143 passed, 11
ignored), both control-gallery example tests, four doctests, all 11 ignored
release GPU/WARP witnesses, all ten census parser tests, all five renderer
receipt admission tests, the full census with zero forbidden edges / zero
external violations / zero slot SCCs, and `git diff --check` passed.

## Checkpoint 1 — ratify the retained-scene contract

This checkpoint completes the requirements-first ledger before committing to
a storage layout.

It must settle:

- the exact crate-private/public status of `scene::{Commit, Node, Content,
  Properties}` while preserving current public `scene::Scene` behavior or
  deliberately migrating it with evidence;
- how `composition::tree::Changes` names per-node semantic change without
  importing GPU vocabulary;
- how layout geometry and scene content revisions join that authoritative
  flow;
- which data is immutable commit structure and which is property state;
- property compatibility with a commit and how stale property updates are
  rejected;
- how property samples become visible only through successful receipts;
- how node addition, removal, reorder, content change, theme/scale change,
  window departure, device loss, and popup retirement clean up;
- exact opacity and effect declarations required for global planning;
- the first property admission table and effect-envelope rule;
- the renderer synchronization failure and resource-readiness model;
- the thin synchronous presentation path and the future pending/active
  compatibility constraints.

The checkpoint publishes a field-by-field mutation-clock table and a complete
upstream correction list. Every omitted renderer requirement is recorded as a
named deviation. The contract is rejected if any field participates in two
handoffs or if a trait lets content issue renderer commands.

Structural witnesses pin:

- no independently minted renderer identity;
- no `PresentationEpoch` inside `scene::Commit`;
- one authoritative change stream;
- closed typed content and exhaustive dispatch;
- no `Renderable::render`-style callback;
- property topology in commit and values outside it;
- old and future presenters consuming the same contract.

### Checkpoint 1 contract ledger — complete

Ratification receipt, 2026-07-14: the following contract is requirements-first.
It describes the information and ownership the retained renderer requires; it
does not bless an arena, tree, map, allocation strategy, or public API expansion.

#### Visibility and compatibility

- `scene::{Commit, Node, Content, Properties}` and their revision/property
  vocabulary are `pub(crate)`. Composition identity is also crate-private, so
  exposing these types publicly would either leak framework internals or force a
  second identity space.
- Existing public `scene::Scene`, `scene::Primitive`, and
  `scene::Presentation::{scene, into_scene}` behavior remains source-compatible
  during this campaign. `Scene` becomes the public authored/inspection facade
  of a committed scene; it is not the new renderer handoff and owns no retained
  identity. The normal runtime path may materialize its compatibility snapshot
  once per semantic commit, never once per presentation. Checkpoint 9 either
  backs that facade directly from retained data or retains the once-per-commit
  snapshot with a measured receipt; it may not leave a per-frame flattening.
- No public widget, application type, or public `Scene` value implements a
  render callback. The boundary is closed typed data. The renderer owns
  exhaustive dispatch and global ordering/batching.

The conceptual API is fixed at this level:

```rust,ignore
pub(crate) struct Commit {
    revision: scene::Revision,
    size: geometry::Size,
    clear: scene::Color,
    nodes: /* ordered retained nodes */,
    property_topology: /* declarations derived from NodeId + kind */,
}

pub(crate) struct Node {
    id: composition::tree::NodeId,
    parent: Option<composition::tree::NodeId>,
    content_revision: scene::ContentRevision,
    geometry_revision: scene::GeometryRevision,
    topology_revision: scene::TopologyRevision,
    local_bounds: geometry::Rect,
    content: /* ordered closed Content values */,
    properties: /* references declared by this commit */,
    opacity: scene::OpacityDeclaration,
    effect: scene::EffectDeclaration,
}

pub(crate) enum Content {
    Quad(scene::Quad),
    Rule(scene::Rule),
    Text(scene::Text),
    TextViewport(scene::TextViewport),
    Icon(scene::Icon),
    Shadow(scene::Shadow),
    Pane(scene::Pane),
    Outline(scene::Outline),
}

pub(crate) struct Properties {
    commit: scene::Revision,
    serial: scene::PropertySerial,
    values: /* complete values for the commit's declared topology */,
    changed: /* bounded NodeId-derived property references */,
}
```

The field spelling and storage remain implementation choices, but the negative
space is final. `Content` has no `Custom`, `Callback`, `Renderable`, raw GPU
handle, nested arbitrary `Scene`, `Clip`, `PopClip`, or `Group` variant. Clips,
groups, transforms, opacity, and sampling effects are topology/properties, not
content species. One composition node may own several ordered `Content` values.
A content-slot index is valid only within that node and content revision; it is
a commit-local address, not identity, and cannot become a cross-commit cache
key without the owning `NodeId` and relevant revision.

#### One change stream and revision currencies

`composition::tree::Changes` becomes the one per-commit change ledger rooted in
composition identity. It preserves the existing removed-element/table-cell
facts and gains closed, renderer-neutral per-node facts:

| Fact | Originating owner | Revision/result |
| --- | --- | --- |
| Added / removed | composition reconciliation | Node lifetime begins / ends |
| Parent or sibling order changed | composition reconciliation | topology revision |
| Semantic/view content changed | composition reconciliation | candidate content change |
| Logical bounds or static clip changed | layout keyed comparison | geometry revision |
| Painted typed content changed | scene projection | content revision |
| Property/effect declaration changed | scene projection | topology revision |

Composition initializes the ledger; layout and scene append only the facts they
own through narrow `composition::Changes` methods. The frozen ledger travels
with the new commit. AccessKit consumes the semantic/lifetime subset; scene and
renderer consume the subsets they understand. There is no second renderer
dirty set and no renderer-authored content revision.

Revisions are monotonic per-window currencies, not identities. An unchanged
node carries its content, geometry, and topology revisions into the next
commit. A relevant fact advances only its relevant currency. Commit revision
advances for any semantic commit. A property tick advances `PropertySerial`
without advancing any commit/node revision. Reorder never destroys content or
GPU resources. Removed identity can never return; a later node receives a new
composition-owned `NodeId`.

The existing window-wide `Paint` / `Layout` / `Rebuild` values may remain as
scheduler hints while callers migrate. They cease to authorize renderer work.
Only the keyed change ledger and revision comparisons authorize scene/GPU
replacement; once all callers emit precise facts, the trichotomy is deleted.

#### Field-by-field mutation clocks

| Field | Sole mutation clock | Consequence |
| --- | --- | --- |
| `NodeId` lifetime | composition reconciliation | Never minted or reconstructed by scene/render |
| Parent, order, content species, static bounds, property/effect topology, clear color | semantic commit | Immutable for a commit |
| Content / geometry / topology revisions | semantic commit change ledger | Advance only for their named fact |
| Transform, scroll, opacity, admitted clip/effect values | property serial | Cannot alter topology or resource envelope |
| Candidate/complete property snapshot | property clock | Compatible with exactly one commit revision |
| Active commit pointer and activation stamp | presentation event | Presentation stamps; commit never contains epoch |
| Sampled and visible property serial | presentation attempt/success receipt | Failed/skipped attempt promotes nothing |
| GPU allocations, readiness, recycling | renderer/device lifecycle | Derived caches, never semantic truth |
| Surface generation/acquire/present outcome | surface lifecycle | Cannot mutate scene or property state |

`Properties` is a complete snapshot at the presentation boundary and names the
bounded set changed since the prior serial. Internal producers may apply deltas,
but a drawable state never depends on replaying an unbounded history. Property
references are structurally derived from `(NodeId, PropertyKind)`; no independent
property identity is minted. A snapshot whose commit revision or declared
topology differs is rejected as `IncompatibleProperties`, not truncated,
best-effort applied, or silently rebound.

#### Property and effect admission

| Value/change | First-stage verdict | Boundary condition |
| --- | --- | --- |
| Scroll offset | Admit; first client | Existing scroll subtree; zero content/geometry revision |
| 2D transform | Admit | Declared transform property and fixed effect envelope |
| Opacity | Admit | Declared variable-opacity class; clamped finite value |
| Clip rect/offset | Admit narrowly | Remains within declared maximum clip envelope; no clip-stack topology change |
| Blur/effect parameter | Admit narrowly | Effect kind and maximum sampling radius/envelope are in commit; value cannot exceed them |
| Hover/pressed/selected color or style | Semantic content change initially | No generic style-property bag |
| Text, glyph/style, icon, material species | Semantic content change | Relevant content revision advances |
| Theme, scale, surface size, effect kind, parent/order | Semantic commit | May advance geometry/topology/content revisions as facts require |
| Caret visibility and scrollbar chrome | Semantic content initially | Promote only after a separate zero-work receipt; no implicit special case |

`OpacityDeclaration` distinguishes statically opaque, statically blended, and
variable opacity. Coverage from typed content and current effective opacity are
separate inputs to renderer classification. `EffectDeclaration` is a closed
description of no effect, group opacity, blur, backdrop/material sampling, and
other already-supported sampling work. Every sampling effect declares an
`EffectEnvelope`: logical owner bounds plus maximum sampling reach. A property
tick may change values only inside that envelope. Exceeding it or changing the
effect class requires a semantic commit.

An effect declaration does not promise an offscreen. The renderer decides
globally whether direct drawing, an effect island, or native realization
satisfies it. Any allocated intermediate reports the declaring `NodeId`,
effect class, bounds, and bytes. This is the operation boundary: content says
what operation applies; it never performs the operation.

#### Activation, readiness, and failure

The Qt-class presenter uses the same contract intended for a future
pending/active implementation:

1. composition/layout/scene produce an immutable commit, frozen changes, and a
   compatible complete property snapshot;
2. renderer synchronization creates/reuses only resources named by those
   revisions and returns either `Ready` or a typed error/not-ready result;
3. the synchronous presenter activates only a complete ready commit and stamps
   its own `PresentationEpoch`;
4. draw samples one complete compatible property snapshot;
5. successful surface presentation receipts the activation and sampled
   property serial; failure leaves visible truth unchanged.

Synchronization is transactional at the commit boundary. Allocation, shader,
text, or device failure cannot expose a partly synchronized commit. Before any
active commit exists, failure follows the existing clear/recovery error path.
After one exists, a future pending/active presenter may keep it drawable. The
first synchronous implementation may return the error to the current recovery
owner, but may not mark the candidate active. Device loss deletes GPU
realization only; the immutable commit and properties remain sufficient to
recreate it.

#### Cleanup and lifetime table

| Event | Required cleanup |
| --- | --- |
| Node added | No eager GPU allocation; realize only required typed content |
| Node removed | Retire node/property/cache entries after no active commit references them |
| Reorder | Update order/topology only; preserve content/text resources |
| Content change | Replace only caches keyed by the advanced content revision |
| Geometry change | Update geometry/instance data; do not reshape unchanged text |
| Property tick | Update bounded property data only |
| Theme/scale/surface-size commit | Advance only evidenced node revisions; invalidate physical resources by the owning scale/format generation |
| Window departure | Drop commit, properties, active state, and all window-scoped realization |
| Device loss | Drop device-scoped realization; retain semantic scene state for recovery |
| Popup retirement | Preserve popup generation/exposure law; release retained scene/GPU state only after retirement no longer references it |

#### Upstream correction and deviation ledger

Admitted upstream corrections:

1. `composition::Changes` currently exposes only additions/removals and is
   effectively test-only. Promote it to the runtime change envelope and add
   categorized reorder/content/geometry/property-topology facts without GPU
   vocabulary.
2. `Composition::project_transient_state` currently mutates projected view
   state without recording scene-relevant node facts. Route resulting semantic
   content changes into the same ledger; property-admitted values bypass commits
   only through `Properties`.
3. `layout::Layout` already carries `Frame::node_id()`, and chrome/table tracks
   already carry composition owners, but it rebuilds flat projections. Add
   keyed comparison/reuse and geometry facts; mint no layout/render identity.
4. `scene::paint` currently erases identity into `Vec<Primitive>`. Replace it
   with the NodeId-keyed commit projection. Shared clip scopes, groups, and
   overlay opacity become topology/effect declarations.
5. Overlay drafts currently lead with interaction identity even though their
   root panel frame has composition identity. Carry that `NodeId` through the
   retained popup scene; popup `Generation` remains exposure/lifecycle currency,
   not scene identity.
6. `scene::Presentation` currently stores `PresentationEpoch` and the runtime
   creates it before renderer readiness. Move activation and epoch to
   presentation-owned `Active`; preserve public inspection behavior.
7. Window-wide invalidation currently decides how much pipeline work runs.
   Demote it to a temporary scheduling hint, then delete it after precise facts
   cover its callers.

Named deviations from heavier precedents:

- no public retained-scene API in this campaign;
- no generic `Renderable` trait or open content species;
- no independent compositor/render thread, pending/active pair, tiling,
  checkerboarding, or damage in the Qt-class stage;
- complete property snapshots at the presentation boundary rather than an
  unbounded mutation log;
- caret/scrollbar visual promotion is deferred until a separate caller and
  zero-work receipt exist;
- public `Scene` compatibility is retained, but it is removed from the renderer
  handoff and cannot justify per-frame flattening.

Contract proof: every field above has one clock; every open-world application
axis remains above this crate-private boundary; every scene/render axis is
closed typed data; identity is carried from `composition::tree::NodeId`; the
same commit/properties contract supports both the thin synchronous presenter
and an evidence-admitted pending/active presenter. Checkpoint 1 is ratified.

## Checkpoint 2 — build the equivalence oracle

The existing deep tier already reads GPU results for premultiplied alpha,
glyph coverage, group opacity, material layers, shared shader compilation, and
popup sRGB packing. This checkpoint generalizes those witnesses before the new
renderer can claim equivalence.

The oracle and repeatable renderer benchmarks live in the dedicated
development-only renderer debug crate required by the independent-execution
law. If crate-private inputs require support from the main crate, that support
is feature-gated, capability-shaped, absent from production builds, and listed
for retirement or promotion at Checkpoint 9. Temporary in-process counters are
the second measurement rail in their own right; they do not exist merely to
export data to an external profiler. PIX or another external program is invoked
only as a fallback if the ledger names a question neither the debug crate nor
temporary logging can answer and records why no further code-owned solution is
practical.

The same prepared `scene::Commit` and property snapshot run through:

1. a temporary legacy adapter that lowers to `paint::Scene` and the old
   renderer;
2. the new retained renderer under construction.

Readback compares final pixels and named intermediate witnesses. Tolerances
are explicit by case:

- exact bytes for format packing and structural clears where exactness is the
  law;
- bounded per-channel tolerance for equivalent floating blend paths;
- silhouette/coverage tolerance for antialiased fractional edges;
- exact ordering and alpha-class expectations for overlapping blended content.

The oracle matrix covers every `scene::Content` species, nested clips, groups,
transform/property values, text, icons, shadows, outlines, panes, material
fallback/full paths, transparent popups, scale factors 1.0/1.25/1.5/2.0, and
zero/empty cases.

The legacy adapter is campaign scaffolding, not a compatibility promise. Its
retirement checkpoint is named at creation. It may not become a product
feature, default fallback, or permanent test backend. At Checkpoint 9 the old
renderer and adapter are deleted together; surviving readback witnesses target
the new renderer directly.

### Checkpoint 2 evidence ledger — complete

Debug-crate cell, 2026-07-14:

- the root is now a workspace whose default member remains `wgpu_l3`; the
  specialized `tools/renderer_debug` package is `publish = false` and depends
  on the main crate only through the non-default `renderer-debug` feature;
- the main crate exposes one feature-gated `render::debug` capability facade.
  It owns the private context/legacy renderer and returns closed `Case`,
  `Image`, `Environment`, and timing `Sample` data. It exposes no
  `paint::Scene`, renderer object, WGPU device/queue/texture, or production
  selector;
- the harness reuses one device and renderer across samples. Windows defaults
  to DX12 through the production context backend policy and still honors the
  explicit WGPU backend override;
- the debug crate owns exact, bounded per-channel, and silhouette comparison
  policies. Three pure comparator tests pass;
- `scene::{Commit, Node, Content, Properties}` is implemented as a closed,
  crate-private retained contract. A commit carries
  `composition::tree::NodeId`, parent-before-child order, independent content,
  geometry, and topology revisions, declared property topology, opacity class,
  and bounded effect declarations. Complete property snapshots reject missing,
  duplicate, undeclared, incompatible, non-finite, and out-of-envelope values;
- the contract and its synthetic composition-identity constructor are compiled
  only by the non-default oracle feature at this checkpoint. Checkpoint 3 makes
  the contract production-owned by connecting it to the authoritative runtime
  projection; ordinary builds therefore carry neither fixture machinery nor a
  dormant parallel renderer path;
- `renderer_debug oracle-all` prepares one exact `Commit` and `Properties`
  instance per pair. The legacy adapter lowers its compatibility scene through
  the former semantic-scene path; the candidate adapter lowers retained nodes
  and properties directly. Both then feed separate instances of the existing
  raster core. This checkpoint proves retained-contract/lowering equivalence
  and the protected lower inputs; it does not falsely claim that the retained
  GPU core already exists;
- the closed 16-case matrix covers the transparent empty case, solid and
  gradient quads, transform plus scroll properties, rules, text, clipped text
  viewports, icons, shadows, outlines, panes, rounded and nested clips, group
  opacity, full glass panes, and transparent-popup packing. At
  1.0/1.25/1.5/2.0 scale, all 64 same-fixture pairs returned
  `differing_pixels=0` and `maximum_channel_delta=0` on the NVIDIA GeForce RTX
  4070 Ti SUPER DX12 rail. Empty output was literally clear and every visual
  case had nonzero coverage;
- transparent-popup cases execute the real sRGB-to-premultiplied-linear pack
  into an `Rgba8Unorm` target. At every scale each adapter produced the expected
  approximately `(64, 64, 64, 128)` center byte and maintained `rgb <= alpha`
  within the two-byte packing tolerance;
- the oracle caught a real coordinate-order defect before closure: the first
  candidate applied scroll after raster-grid resolution. At scale 1.25 that
  produced 94 differing pixels with maximum channel delta approximately
  `0.9019608`. Moving scroll into logical space before grid snapping restored
  exact equivalence and preserves the static relationship between a node's own
  clip and its scroll while moving descendant clips with inherited offsets;
- `renderer_debug bench solid-quad 12` uses one warmup and 12 measured samples.
  The first debug-build DX12 receipt reported p50 1.352 ms, p95/max 1.860 ms.
  This includes offscreen allocation, legacy preparation/encoding, submission,
  and readback, so it is an oracle/harness baseline rather than the window
  renderer budget;
- `cargo check --features renderer-debug -p wgpu_l3`, `cargo check -p
  renderer_debug`, the three debug-crate comparison tests, the three retained
  contract tests, the non-production/identity architecture ratchet, ordinary
  all-target compilation, formatting, and diff hygiene pass without warnings;
- boundary verification then passed `cargo check --workspace --all-targets`,
  the complete workspace suite (1,147 passed, 11 explicitly ignored, plus the
  three debug-crate tests), all four root doctests, the five renderer-receipt
  parser/admission tests, formatting, and diff hygiene without warnings;
- no external program or external machine participated in this cell.

`OW-RR-2` cleanup cell: the complete trace found one tempting leak—the oracle
needed crate-private context, renderer, scene, and readback owners. Admission
kept one narrow feature-gated `render::debug` facade and an unpublished debug
package instead of widening those owners or exporting WGPU vocabulary. The
fixture `NodeId` constructor remains feature-gated at composition, text-surface
visibility widened only within `scene`, and ordinary builds compile neither the
retained fixture contract nor debug facade. The ratchet proves the feature is
non-default, the package cannot publish, the capability is hidden in production,
the commit contains no renderer identity or presentation epoch, content has no
callback/custom escape hatch, and no legacy runtime selector exists. No
superseded production path became unreachable in this checkpoint; the two
adapter paths remain intentionally and exclusively for the oracle until their
mandatory joint deletion at Checkpoint 9. Re-scan result: fixed point for the
development boundary; production scene identity integration is the selected
Checkpoint 3 work, not deferred cleanup.

Checkpoint 2 is independently green. The exact oracle remains a mandatory
ratchet through renderer displacement. Checkpoint 3 now moves composition
identity and authoritative revisions into the ordinary runtime scene path.

## Checkpoint 3 — retain scene identity and revisions

Scene painting stops erasing `layout::Frame::node_id()`.

The retained scene projection must:

- produce complete immutable commits that remain drawable independently of
  future application/composition mutation;
- carry composition identity into each retained node;
- preserve typed content, order, bounds, clips, property topology, material
  identity, and effect declarations;
- structurally reuse unchanged nodes/subtrees between commits;
- derive content revision changes from the authoritative change flow;
- remove departed nodes and every associated scene-side cache exactly once;
- preserve overlays, ghosts, native-popup requests, and material-region
  identities without creating a separate scene species;
- retain bounded virtualization and pin laws;
- keep candidate commits distinct from what presentation has activated or
  receipted.

GTK-style cached node reuse is the model: an unchanged node is not asked to
re-snapshot merely because a sibling or property value changed. Whole-window
theme, scale, or layout causes may still rebuild the truthful affected set.

Acceptance witnesses include:

- sibling content change rebuilds only that identity and necessary ancestors;
- reorder preserves identities and content revisions while changing order;
- property-only transform/opacity changes rebuild zero content nodes;
- removed virtual rows delete retained scene nodes while ordinary
  dematerialization remains distinct from semantic deletion;
- popup live/ghost/retiring species retain their established identities and
  lifecycle clocks;
- unchanged second commit has zero scene paint calls and zero new content
  revisions;
- every old window-wide invalidation use has a disposition: translated to an
  owned change cause, retained temporarily for the legacy adapter, or deleted.

### Checkpoint 3 evidence ledger — complete

Retained projection cell, 2026-07-14:

- `composition::tree::ContentRevision` is the authoritative per-node content
  currency. `Changes` now reports added, changed, and removed composition
  `NodeId`s; reconciliation advances only the changed node's content revision.
  Stable keyed reorder changes structural order without changing either
  identity or content revision. No renderer revision or identity namespace was
  admitted;
- each `layout::Frame` carries its originating composition `NodeId`, parent,
  and content revision. `FrameSceneKey` combines that semantic currency with
  truthful geometry/static visual inputs, including the existing transient
  pixel projection where required. Layout therefore preserves identity instead
  of forcing scene painting to reconstruct it from flattened primitives;
- `scene::{Commit, Node, Content, Properties}` is now the ordinary crate-private
  production contract rather than a dormant oracle-only type. A commit is
  complete and immutable, owns typed content and structure, and contains no
  presentation epoch. Property values remain a separate complete snapshot;
- one `scene::Builder` registers each composition identity exactly once and
  carries cross-owner paint order in a separate structural order list. Reorder
  can therefore reuse the exact `Arc<Node>` allocations while changing order;
  the order carrier has no identity and cannot become a third identity space;
- one per-window `scene::Store` owns retained frame, table-track, chrome, node,
  and commit caches. Runtime preparation enters it once, produces the immutable
  commit first, and only then lowers through the temporary compatibility
  `paint::Scene` adapter required by the equivalence oracle and legacy GPU core;
- unchanged frame keys reuse their cached fragments without invoking scene
  paint. An unchanged complete commit reuses the exact `Arc<Commit>` and
  `Arc<Node>` allocations. Departed identities are removed from frame and node
  caches once; the following unchanged projection reports zero further
  removals;
- `overlay::{Draft, Entry, Live, Ghost, RetiringPopup, Layer}` carries the same
  retained `Arc<Commit>` and `Arc<Properties>` through live, in-frame ghost,
  and native-popup retirement lifetimes. Overlay identity, popup generation,
  fade timing, and exposure clocks were not replaced or borrowed;
- material regions retain their composition `NodeId` through commit construction
  and compatibility lowering. Candidate scene construction remains distinct
  from presentation activation and successful-present receipts;
- the retained painter's per-projection seen set and statistics now have one
  `RetainedWork` owner. That cleanup removed the checkpoint's unreasoned
  `too_many_arguments` allowance instead of normalizing it as rewrite residue.

Acceptance receipts:

- `one_sibling_content_change_mints_only_that_nodes_revision` proves one
  changed sibling produces exactly one `Changes::changed` identity, advances
  only its content revision, and leaves root/stable-sibling revisions alone.
  `one_sibling_content_change_repaints_only_that_scene_identity` then proves
  the production projection creates one semantic commit, rebuilds one scene
  node, makes one scene paint call, and reuses the remaining nodes;
- `explicit_ids_preserve_node_ids_across_sibling_movement` and
  `reordered_commit_reuses_nodes_but_changes_structural_order` prove stable
  reorder preserves composition revisions and exact retained node allocations
  while changing draw order;
- `property_only_opacity_tick_changes_zero_node_revisions` changes a complete
  property snapshot while content, geometry, and topology revisions remain
  byte-for-byte unchanged. No content paint is available on that handoff;
- `unchanged_second_commit_paints_zero_scene_nodes` reports literal
  `semantic_commits_created == 0`, `scene_nodes_rebuilt == 0`, and
  `scene_paint_calls == 0`, with positive retained-node reuse;
- `departed_scene_nodes_are_removed_once` proves a departed ordinary identity
  retires scene cache state exactly once. The mutable virtual-list witness also
  observes positive retained-node removal after provider deletion, while the
  existing focus/draft/capture/context witnesses continue to distinguish
  ordinary dematerialization from provider deletion and retain the bounded pin
  laws;
- `removed_entry_creates_fading_ghost` and
  `removed_native_popup_retires_on_its_native_surface_without_a_parent_ghost`
  use `Arc::ptr_eq` to prove the ghost/retiring layers retain the exact commit
  and property objects from their live entries;
- retained-contract material, order, allocation-reuse, property, completeness,
  opacity, and effect witnesses pass, and the 16-case by four-scale equivalence
  oracle remains exact: all 64 pairs report `differing_pixels=0` and
  `maximum_channel_delta=0`, including transparent-popup packing.

Window-wide invalidation disposition, from the complete 36-use production/test
scan:

- the nine `response::effect` uses remain the closed public response/scheduling
  vocabulary. They do not identify scene nodes, mint content revisions, or
  invalidate renderer caches and remain temporarily because the compatibility
  runtime still accepts the old scheduling classes;
- three pointer/focus `Layout` request sites and the two focus sites are owned
  geometry/input causes. Layout comparison and `FrameSceneKey`, not the enum,
  determine which retained frames actually change;
- the pointer/input/selection/text `Rebuild` request sites ask runtime to
  reconcile application view truth. `composition::Changes` is the sole
  per-node result consumed downstream; no parallel renderer change ledger was
  created;
- the eight `runtime::presentation` uses and three `session::window` uses are
  initial-candidate, retry, animation-scheduling, and compatibility-adapter
  control. They remain until the property-scroll transition and legacy
  burn-down checkpoints, and they cannot choose a retained node revision;
- the three test uses witness the public effect contract and legacy scheduling
  behavior. Thus every surviving use is either an owned upstream cause or an
  explicitly temporary adapter scheduler; zero uses act as a scene/GPU cache
  invalidation authority.

`OW-RR-3` cleanup cell: the trace is
`composition::Tree -> layout::Frame -> scene::Store/Builder -> immutable
scene::Commit -> temporary compatibility adapter -> presentation`, with overlay
lifecycle as a retaining consumer rather than a second scene species. Admission
kept the existing composition identity, change stream, material identity,
popup generations, and presentation receipts; it added one scene projection
owner and no renderer identity, invalidation trait, callback, or production
selector. The old ordinary immediate-paint entry is no longer the runtime
recipe; legacy lowering survives only after commit construction for the oracle
and old GPU realization and is scheduled for deletion at Checkpoint 9.

The touched-module re-scan corrected two stale architecture ratchets to name the
retained recipe and total parent identity. It also found one new
`render -> pollster` external-boundary violation in feature-gated debug support.
The capability facade now exposes asynchronous harness construction and the
specialized debug crate owns the blocking executor, restoring the external
dependency boundary. The scene painter cleanup removed its new Clippy
allowance. The final census reports 47 top-level modules, 328 production and
112 test-only module edges, three split responsibilities, 55 provisional slot
edges, **zero forbidden edges, zero external-boundary violations, and zero slot
SCCs**, 1,915 `pub(crate)` declarations in 194 production files, cross-slot
upper bound 1,868, 90 cross-slot test edges, 120 source-root mentions, 377
filesystem reads, seven allowances, five production panics, and 53 production
expects. The remaining allowances/expects have existing owners or the explicit
effect-contract admission; none conceal a second renderer path.

Boundary verification after the restart passed:

- `cargo check --workspace --all-targets`, root all-target compilation, and all
  maintained examples without warnings;
- the complete workspace suite: three debug-crate tests, 1,156 root tests
  passed with 11 intentional deep-tier ignores, and all four root doctests;
- all 157 layout/scene witnesses and all 151 architecture witnesses;
- the release deep tier: all 11 WARP, shader, alpha, glyph, popup packing,
  glass/material, and text acceptance witnesses;
- all five renderer-receipt and all ten One-Way census parser/admission tests;
- the full ownership census above, formatting, diff hygiene, and protected
  `comparison_open: true` state;
- no external profiler, machine, person, network service, or returned artifact
  participated. The implementation was preserved across the requested restart
  by safety commits `0462b3ab` and `9d47c19d`; this ledger and cleanup form the
  intentional independently green Checkpoint 3 closeout commit.

Checkpoint 3 is independently green. Checkpoint 4 now replaces one-frame GPU
preparation with identity/revision-keyed realization while keeping this retained
scene, the exact oracle, and the protected lower surface seam intact.

## Checkpoint 4 — retain GPU realization

The new renderer synchronizes a commit into identity/revision-keyed GPU state.

Required resource families include:

- analytic shape instances over a static unit quad;
- text/glyph preparation keyed by content revision while preserving glyphon
  cache/atlas ownership;
- icon/image texture resources when those content species exist;
- clip/effect topology and bounded intermediate pools;
- property buffers or equivalent bounded state separate from content buffers;
- render-plan/batch metadata that can be reused until relevant structure or
  opacity/effect classification changes.

An unchanged commit followed by another draw must report:

- zero scene-node realization rebuilds;
- zero primitive preparation calls;
- zero text shape/prepare calls for unchanged text;
- zero content-buffer upload bytes;
- zero GPU resource creations;
- only the frame-global/property data demonstrably required to present.

One changed node uploads only that node's affected resource range or a measured
bounded batch-root replacement. It does not silently rewrite a monolithic
whole-scene buffer merely because the API calls the buffer “retained.”

Device loss and renderer recreation rebuild all GPU state from the active
commit without mutating semantic identity or content revisions. Window close,
popup host return/drop, node removal, and commit retirement each have named
resource-count witnesses. Retained bytes and pool high-water marks must settle
after churn.

### Checkpoint 4 evidence ledger — complete

Retained realization cell, 2026-07-14:

- `render::retained::Realizer` synchronizes the immutable commit into retained
  shape, text, property, and plan state. `ResourceKey` borrows
  `composition::tree::NodeId` plus content, geometry, and topology revisions;
  content index/part, scale, and target-space facts distinguish truthful
  realizations. It contains no property serial, presentation epoch, hashed
  primitive identity, or renderer-minted identifier. Weak commit/node owners
  let overlapping commits share the same semantic resource and make retirement
  collectable without a parallel invalidation authority;
- analytic shapes use one static six-corner unit quad and a 160-byte instance
  record. Retained instances live in a geometrically grown suballocated buffer
  with coalesced free ranges. A changed node writes only its new range; unchanged
  siblings retain theirs. The former six-vertex immediate arena remains solely
  for the oracle, native-popup, and overlay compatibility suffix until the
  semantic-plan and burn-down checkpoints;
- property state lives in a separate dynamically offset uniform buffer. Its
  bindings are keyed by `(NodeId, TargetSpace)`, so the same semantic node may
  be realized correctly inside distinct nested targets without contaminating
  content identity. The cache samples one compatible `PropertySerial`, binding
  list, viewport, and commit pointer; an unchanged second draw uploads zero
  property bytes as well as zero content bytes;
- retained text is keyed by the same semantic resource family. Each retained
  text target owns its prepared glyphon renderer/viewport while every target
  shares the renderer-global font system, swash cache, inline cache, GPU cache,
  and atlas. Text and icon glyphs therefore shape/prepare once per content
  revision without creating private atlases. The closed scene contract has no
  image content species yet, so no speculative image-texture owner was added;
- commit order, clips, nested groups, panes/effects, opacity, and target-space
  bounds are retained in reusable plans. Existing bounded filter-layer and
  scratch pools remain the only intermediate owners; Checkpoint 5 now makes
  their allocation and ordinary-surface topology semantic rather than
  one-frame descriptive;
- the production runtime/native ordinary-window path now hands the exact
  `Commit`, `Properties`, and in-frame compatibility overlay suffix to
  `Renderer::draw_commit`. Retained and temporary batches converge at the
  existing post-preparation encoder and the proven
  `Context -> Canvas -> Surface -> present` seam is unchanged. The control
  gallery therefore uses the retained base renderer in ordinary execution as
  of this checkpoint; Checkpoints 5 and 6 remove the remaining whole-frame
  topology and scroll-content work that determine its visible performance;
- the specialized debug crate owns exact readback, retention, partial-update,
  recreation, retirement, and churn receipts. The default application API
  exposes no GPU resource type, renderer identity, retained cache handle, or
  selector between production renderers.

Exactness and reuse receipts on the local NVIDIA GeForce RTX 4070 Ti SUPER DX12
rail:

- the expanded 17-case matrix covers empty, shape, gradient, transformed,
  rule, text, clipped text viewport, icon, shadow, outline, pane, rounded and
  nested clips, grouped opacity, production-shaped ordered nested groups,
  glass, and transparent-popup packing. At 1.0/1.25/1.5/2.0 scale, all **68 of
  68** legacy/retained pairs returned `differing_pixels=0` and
  `maximum_channel_delta=0`. The ordered case uses the real `scene::Builder`
  order carrier, a nonzero frame origin, internal clipping, nested opacity
  groups, and text rather than a flattened fixture shortcut;
- solid-quad, text, ordered-group, and transparent-popup retention receipts all
  report the same unchanged-draw law: `node_rebuilds=0`,
  `primitive_prepare_calls=0`, `text_prepare_calls=0`, `text_shape_calls=0`,
  `content_upload_bytes=0`, `property_upload_bytes=0`, `gpu_creations=0`, and
  `plan_reuses=1`. Ordered-group first realization created three resources,
  prepared two analytic instances and one text target, and uploaded 320 content
  plus 528 property bytes; its unchanged draw performed none of that work;
- the partial-update receipt starts with two nodes/two instances/320 content
  bytes. Changing one identity reports exactly one node rebuild, one primitive
  preparation, one 160-byte content write, and one resource creation. The next
  unchanged draw keeps the surviving sibling and reports every content/property
  work currency at zero while collecting the displaced resource;
- a fresh `Renderer` rebuilt the same commit to byte-identical pixels with the
  same semantic `NodeId` and revisions. This is the device/renderer recreation
  witness: semantic state is sufficient to reconstruct GPU state and no GPU
  generation is written upstream;
- dropping the ordinary solid/text/ordered commits and drawing an empty commit
  returns retained counts to the four renderer-global shape-buffer resources,
  with one, one, and three removals respectively. The transparent-popup
  host-lifecycle fixture returns its popup-format renderer to the same
  four-resource baseline. These are the named window/commit, text,
  ordered-node, and popup host return/drop resource witnesses; renderer drop
  itself remains the final RAII owner of the four global WGPU handles;
- 64 alternating partial-update commits reached a post-warm retained-resource
  range of exactly `(7, 7)` and byte range `(49216, 49216)`. After the active
  commit retired, the receipt settled at four resources/49,216 owned buffer
  bytes with zero preparation/upload work and three resource removals. Shared
  glyph-atlas storage remains one existing global cache and is deliberately not
  double-counted as per-node retained bytes.

Upstream correction and cleanup cells:

- `OW-RR-4A` — complete trace:
  `scene::Commit -> retained plan/resources -> shared encoder -> Canvas/Surface`,
  with the legacy `paint::Scene` route beside it only for the oracle and the
  explicitly temporary popup/overlay suffix. Challenge found that the prior
  candidate path flattened a commit in `render::scene` and therefore erased
  identity a second time before GPU preparation. Admission kept one production
  `draw_commit` handoff and the common post-preparation encoder; Reduce deleted
  the displaced 165-line candidate flattening adapter and its helper family.
  Rewire moved native ordinary surfaces onto the retained handoff. Ratchets
  require native `draw_commit`, borrowed composition identity/revisions, static
  unit-quad instancing, separated property storage, weak semantic ownership,
  and structural absence of the deleted adapter. Fixed point: one retained
  production base path, one protected legacy oracle/popup suffix with a named
  terminal checkpoint, and no selector exposed upward;
- `OW-RR-4B` — the exact ordered-group witness found an upstream projection
  defect: translating an outer group moved the inner group's declared bounds
  and also translated items already local to that inner group. The renderer did
  not accommodate the bad shape. `paint`, the translation owner, now moves the
  nested group envelope once and leaves its local contents local. A focused
  owner test and all four exact scale witnesses ratchet the correction. Retained
  text preparation likewise localizes glyph items at the target boundary rather
  than baking target position into semantic content;
- the touched-module re-scan corrected two stale architecture tests: ordinary
  native realization is now required to consume retained commits while popup
  lowering remains renderer-owned, and glyphon viewport ownership now covers
  both immediate per-batch and retained per-resource cases. Obsolete
  `Content::translated`, unused node accessors, and the flattened candidate
  helpers were deleted. The one retained `Properties::changed` helper is marked
  with an explicit Checkpoint 6 sparse-property-upload reason rather than an
  unowned allowance;
- the final gauge reports 47 top-level modules, 329 production and 112 test-only
  module edges, three split responsibilities, 55 provisional slot edges,
  **zero forbidden edges, zero external-boundary violations, and zero slot
  SCCs**, 1,938 `pub(crate)` declarations in 194 production files, cross-slot
  upper bound 1,889, 90 cross-slot test edges, 120 source-root mentions, 383
  filesystem reads, seven allowances, five production panics, and 53 production
  expects. All external WGPU/glyphon/bytemuck use remains in `render`; no
  dependency arrow was concealed by the retained contract.

Boundary verification after the restart passed:

- `cargo fmt --check` and `cargo check --workspace --all-targets` without
  warnings; the complete workspace target suite passed with three debug-crate
  unit tests, 1,157 root tests, and both control-gallery example tests, with 14
  intentional GPU/acceptance ignores across the root and debug packages;
- all four root doctests passed. The release deep tier passed all 11 existing
  WARP, shader, alpha, glyph, popup packing, glass/material, and text witnesses;
  the specialized release tier passed all three new exact ordered-commit,
  retained lifecycle/partial-update, and high-water churn witnesses;
- all 15 renderer-receipt and One-Way parser/admission tests, the full ownership
  census above, exact 68-cell oracle, formatting, and diff hygiene passed;
- no external profiler, out-of-process measurement program, external machine,
  person, network service, or returned artifact participated. All measurements
  and readbacks were produced locally in code by the specialized debug crate.

Checkpoint 4 is independently green. Checkpoint 5 now removes accidental
whole-window work by making plan roots, opacity, order, target changes, and
effect islands the sole owners of render scope while preserving the retained
resource and exactness laws above.

## Checkpoint 5 — make render work semantic

The retained resources feed a renderer-global plan.

This checkpoint replaces descriptive one-frame batches with semantic planning:

- batch roots isolate subtrees whose property changes should not invalidate
  stable siblings;
- compatible analytic instances merge without violating clip/effect/order;
- opacity classification separates safe opaque work from blended work;
- blended content preserves exact back-to-front constraints;
- clip changes, effect islands, popup packing, and target changes are the only
  render-scope boundaries;
- text and shapes may share a pass only while each reasserts the GPU state it
  owns, preserving the Pay Once finding;
- ordinary opaque windows render directly to the acquired surface unless a
  named sampling dependency requires otherwise;
- group opacity, blur, glass/material sampling, and popup format conversion
  allocate only bounded, named intermediates;
- every intermediate clear/composite reports owner, bounds, and bytes.

Acceptance includes:

- zero extra full-surface intermediate clears and zero full-surface blits for
  the ordinary opaque no-effect witness;
- popup pack pixels and premultiplied transfer remain exact;
- local effect work is bounded by its declared envelope, not automatically by
  the whole window;
- the prior 6–7 pass table topology does not regress without measured cause;
- instanced content upload is materially below the six-vertex baseline and
  unchanged content uploads zero;
- code-owned counters confirm the target-switch, copy, draw, and barrier
  account; an external capture is admitted only to resolve a named mismatch;
- the oracle remains green across all content and alpha classes.

CPU occlusion may be tested behind the harness. It is admitted only if in-code
benchmarks and counters show that saved GPU/fill work exceeds its traversal and
plan-churn cost. An external profiler may resolve an otherwise unowned GPU-fill
question; an available weak-GPU receipt strengthens the verdict but is not
required. Opaque classification does not depend on it.

### Checkpoint 5 evidence ledger — complete

Implementation receipt, 2026-07-14:

- the retained plan now owns semantic facts and the surface-sampling verdict.
  Plan reuse preserves scene/item, instance, text, clip, opacity, effect, and
  group counts without rebuilding or rescanning the commit. Adjacent analytic
  ranges merge only when their retained instance ranges are contiguous and
  their borrowed node/property binding is identical; text, clip, pane, group,
  and noncontiguous ranges remain hard order boundaries. Focused tests prove
  both the admitted merge and the boundary refusal;
- `SurfacePath` is a closed renderer decision with three realizations:
  `Direct`, `SampledComposition`, and `PackedPremultiplied`. Ordinary retained
  commits select `Direct`; a named backdrop/noise sampling dependency selects
  `SampledComposition`; Windows popup packing retains its exact dedicated
  path. The direct traffic witness asserts one acquired-surface clear and
  literal zero extra full-surface intermediate clears, intermediate bytes,
  full-surface blits, blit bytes, and popup packs;
- opacity declaration is consumed from the commit and retained as a plan fact.
  Production overlays are conservatively blended rather than unclassified.
  The opaque rule work receipt reports `opaque_nodes=1`,
  `blended_nodes=0`, and `opacity_unclassified_nodes=0`; the classification
  survives plan reuse in the lifecycle witness;
- the rule work receipt reports one direct plan, zero sampling plans, one draw,
  two render-target passes, zero explicit copy commands, one resource-transition
  boundary, and zero effect clears/composites/bytes. The ordered mixed
  shape/text/group witness remains at five target passes, below the prior 6–7
  pass ordinary-table family, so semantic planning did not purchase the blit
  removal by multiplying pass boundaries;
- filter output and scratch space are now distinct. The final composite writes
  directly to the original parent target with the original prepared geometry;
  ping/pong targets are sized only to `paint::pane_effect_bounds`, and filter
  geometry is translated into that local envelope. The glass receipt reports
  seven draws, eight target passes, zero explicit copy commands, seven
  resource-transition boundaries, three scratch clears/41,760 bytes, two
  bounded composites/16,192 bytes, and a largest intermediate of 13,920 bytes
  against the 16,384-byte full target;
- every filter invocation carries the closed owner (`PaneBackdrop` or
  `PaneSurfaceNoise`). Code-owned debug logging emits owner, output bounds,
  scratch bounds, clear/composite counts, and bytes; renderer receipts retain
  current/total work and the intermediate-byte high-water mark. Actual WGPU
  barrier insertion remains backend-owned, so the in-code account reports
  render passes and resource-transition boundaries rather than inventing
  driver barrier counts. No mismatch required an external profiler;
- the static unit-quad instance upload remains 160 bytes for the one-instance
  fixture, materially below the displaced six-vertex stream, while unchanged
  retained work still reports zero primitive/text preparation and zero content
  upload. CPU occlusion was not admitted: semantic coalescing and target
  isolation delivered the checkpoint without a second traversal or churn cost.

Oracle correction and exactness receipt:

- the new work counters exposed that the offscreen oracle had not initialized
  filter scratch textures, so its glass case had exercised tint geometry but
  not the filter chain. `filter::Renderer::prepare_target` now gives both legacy
  and retained offscreen adapters the same complete filter resources. This is
  a harness correction, not a production accommodation;
- after that correction, all 17 cases at 1.0/1.25/1.5/2.0 scale remain exact:
  **68 of 68 comparisons report zero differing pixels and zero channel delta**.
  The glass cells now execute blur/refraction/noise work, and transparent popup
  packing remains exact;
- a pane-sized offscreen experiment was rejected before admission. Its variants
  introduced 453, then 416, then 1,824 differing pixels through duplicate edge
  coverage or an extra color quantization. The experiment and its copy helpers
  were deleted completely. Bounding only scratch storage retained the original
  final write and restored literal pixel identity.

Touched-module cleanup cells:

- `OW-RR-5A` — complete trace:
  `scene::Commit -> retained Plan -> SurfacePath -> SceneEncoder -> Surface`.
  Challenge found the unconditional ordinary composition clear plus final blit
  and a stale popup ratchet that could have let an empty operation list become
  material truth. Admission placed the sampling verdict in the retained plan,
  consumed resolved material filters at the renderer owner, and kept popup
  packing separate. Reduce removed the ordinary blit route; Rewire sends only
  named sampling plans through composition. Fixed point: one plan verdict, one
  closed surface-path decision, and no selector or renderer fact exported
  upward;
- `OW-RR-5B` — complete trace:
  `scene Pane/effect declaration -> paint effect envelope -> FilterScratch ->
  ping/pong passes -> parent-target composite`. Challenge rejected the
  pane-island copy/composite species on exactness evidence. Admission split
  output and scratch target/prepared geometry at the filter boundary. Reduce
  deleted the island texture/copy family. Rewire made the declared paint
  envelope the one scratch-sizing derivation. Owner/bounds/bytes logging and
  the exact glass oracle are the ratchets;
- `OW-RR-5C` — the instrumentation trace found two reporting defects and
  corrected them at their owners: plan facts previously disappeared on plan
  reuse, and draw-pass counts omitted filter, layer-clear, layer-composite, and
  final presentation passes. Plan-owned facts and pass-boundary accounting now
  survive reuse and report actual encoded structure. The oracle initialization
  omission was likewise corrected at `filter::Renderer`, not papered over in
  the harness;
- the touched-module gauge reports 47 top-level modules, 329 production and
  112 test-only module edges, three split responsibilities, 55 provisional slot
  edges, **zero forbidden edges, zero external-boundary violations, and zero
  slot SCCs**, 1,949 `pub(crate)` declarations in 194 production files,
  cross-slot upper bound 1,899, 90 cross-slot test edges, 120 source-root
  mentions, 383 filesystem reads, seven allowances, five production panics,
  and 53 production expects. The added visibility is renderer/debug receipt
  currency; no external WGPU ownership or dependency arrow moved.

Verification freeze, 2026-07-14: formatting and workspace checks passed; the
complete workspace target tier passed three debug-crate comparison tests,
1,174 root tests (1,163 passed, 11 intentional ignores), and both Control
Gallery tests. All four doctests passed. The release deep tier passed all 11
existing WARP/shader/alpha/glyph/popup/material/text witnesses and all four
specialized retained/oracle/lifecycle/semantic-work witnesses. All 15
renderer-receipt and One-Way parser/admission tests, the full ownership census,
the exact 68-cell oracle, and diff hygiene passed. No external profiler,
external machine, network service, or returned artifact was required.

Checkpoint 5 is independently green. Checkpoint 6 now moves scroll onto the
property clock and owns the two interactive baseline defects recorded under
Checkpoint 0.

## Checkpoint 6 — make scroll a property tick

Scroll is moved onto the property handoff without changing virtualization,
input, focus, selection, editing, popup, or presentation truth.

For a controlled stream wholly inside the guard window, the aggregate
per-tick counters must remain exactly:

```text
scene_paint_calls       == 0
text_shape_calls        == 0
text_prepare_calls      == 0
primitive_prepare_calls == 0
content_upload_bytes    == 0
content_revisions       == 0
```

Only bounded changed-property count/bytes may rise. The renderer draws the
same retained content under the latest sampled scroll transform. Scrollbar
visuals join the property path only if their geometry/value contract satisfies
the same structure/value and hit-testing laws.

The successful receipt promotes the exact sampled scroll property to visible
input truth. Named witnesses cover:

- a skipped scroll frame leaving input on the prior visible transform;
- a successful retry promoting the latest transform without a content commit;
- stationary pointer hover moving to the content now presented beneath it;
- capture continuing to route by the visible transformed geometry;
- sticky headers and fixed chrome remaining outside the scrolled property
  subtree;
- native popup placement from transformed presented geometry;
- selection, editing, caret, IME, reveal, and table divider behavior;
- many wheel deltas coalescing into one property sample without losing total
  offset.

Guard-boundary witnesses then prove the Qt-class law: replenishment is a small
semantic commit, resident work remains bounded, synchronous activation stays
inside the pinned 16.667 ms ceiling, and no visible hitch occurs in the local
interactive smoke. They do not claim
that active presentation can proceed concurrently with replenishment.

### Checkpoint 6 evidence ledger — complete

Implementation receipt, 2026-07-15:

- scroll state now has a property-clock route distinct from semantic scene
  commits. A guard-contained wheel stream updates the retained scroll property,
  preserves the commit structure and baseline coordinates, coalesces to the
  latest sampled value, and promotes that exact value to input truth only after
  a successful present. Focused skipped/retry witnesses prove that an
  unsuccessful frame promotes neither geometry nor input;
- layout owns retained scroll projections, group ancestry, fixed chrome, sticky
  headers, nested clips, and presented-coordinate hit testing. The property
  handoff no longer rebuilds the large table hit-test tree. Selection, editing,
  caret/reveal, native popup placement, header/divider hit envelopes, pointer
  capture, stationary hover, and nested scroll clips all consume the same
  successfully presented transform;
- the literal-zero runtime witness applies four 8 px deltas wholly inside the
  half-viewport guard and requires each input effect to remain `None`. Its
  aggregate asserts zero scene painting, text shaping, text preparation,
  primitive preparation, content upload, and content revisions. The dedicated
  GPU harness independently renders the same retained transition and asserts
  pixel equivalence with zero primitive/text preparation, shaping, or content
  upload;
- the actual Control Gallery now participates in the specialized debug-crate
  oracle through the narrow feature-gated diagnostics observation boundary.
  The oracle runs the real gallery commit before and after a guard-contained
  property tick. Exact pixels remain the first acceptance path; equivalent
  floating blend routes are bounded at a maximum four-channel-value delta. The
  renderer does not import the gallery, layout tests do not import the renderer,
  and the production build has no debug-crate dependency;
- guard replenishment remains a semantic commit and rebases its property state
  atomically. Resident retained scroll textures are bounded by viewport/guard
  scope and the selected device limit, including nested descendants. This
  corrected the reported fatal attempt to create a 16,862 px texture on an
  8,192 px device; repeated large/fast release-gallery scrolling now survives
  without validation failure;
- transient text and selection changes request and present interaction frames
  while the input stream is active. In an uncontested release smoke, 32 typed
  characters completed and were visible in the same 156 ms transaction, a
  selection drag completed with its highlight visible in 75 ms, and a slider
  capture completed with its value visible in 68 ms. A stale overflow tooltip
  also disappeared when its text ceased overflowing without requiring pointer
  motion. These replace the recorded idle-only typing/selection behavior;
- vertical and horizontal scrollbar chrome now share the same chrome-owned
  admission path. `Hit::Chrome` cannot be reinterpreted as the row or table cell
  behind it, so scrollbar capture leaves row selection and active-cell truth
  unchanged. The focused witness closes the reported vertical-only
  click-through/source-of-truth deviation.

Paired local release receipt:

- the admitted 500 px in-window run is
  `target/release/examples/renderer-receipts/control-gallery-500px-in-window-scroll-cp6-admitted-1784099811391.txt`.
  It records 68/68 presented/acquired frames, 52 property ticks, zero guard
  crossings, zero replenishments, draw p95 2.716 ms, zero full-surface blits,
  and an exact final sampled/visible property serial;
- the matching 800 px guard run is
  `target/release/examples/renderer-receipts/control-gallery-800px-guard-boundary-cp6-admitted-1784099836196.txt`.
  Sixteen large/fast scrolls produced 16 guard crossings, 16 replenishment
  commits, and 16 timing samples. Replenishment p95 was 7.065 ms, inside the
  Checkpoint 6 synchronous 16.667 ms ceiling; the process survived and the
  retained-texture device-limit regression did not recur;
- `tools/check_renderer_receipts.py` admits the pair without an external
  machine or profiler. The receipts deliberately do not use the Checkpoint 7
  final renderer-budget flag: on the local 240 Hz display, the in-window run
  still records three renderer deadline misses and the guard run records draw
  p95 8.761 ms plus 32 misses against a 4.167 ms refresh. The user's broader
  "all interaction is choppy" report therefore remains an explicit Checkpoint
  7 pacing defect, not a claim silently erased by Checkpoint 6. Automation
  injection/waits contaminate frame-interval timing, so code-owned draw and
  replenishment timings remain the acceptance currency.

Touched-module cleanup cells:

- `OW-RR-6A` — complete trace:
  `wheel input -> layout scroll owner -> scene::Properties -> retained property
  upload -> successful presentation receipt`. Challenge found that the old
  window-wide invalidation and flattened hit tree forced semantic rebuilding
  for a value-only change. Admission retained scroll projections and ancestry
  at layout's owner, preserved baseline commit geometry, and bound guard
  crossing to the one semantic replenishment path. Rewire moved ordinary wheel
  motion to the property clock; focused zero-work, retry, nesting, sticky, and
  divider ratchets prevent the displaced coupling from returning;
- `OW-RR-6B` — complete trace:
  `sampled property -> renderer draw -> presentation activation -> presented
  geometry -> input/hover/popup consumers`. Challenge found consumers reading
  candidate or commit-time geometry and interaction updates waiting for the
  input stream to go idle. Admission placed the sampled property on the frame
  receipt and the activated property on the presentation owner. A
  refresh-relative presentation pulse services active interaction without
  minting content truth. Skipped/retry, typing, selection, capture, and popup
  witnesses ratchet the one receipt-owned promotion;
- `OW-RR-6C` — complete trace:
  `layout scrollbar chrome -> Hit::Chrome -> chrome capture`. Challenge proved
  that row-gesture derivation could reinterpret vertical scrollbar ownership as
  the underlying virtual row. Admission makes chrome terminal for row/table
  gestures; Reduce removes the accidental second interpretation. One shared
  horizontal/vertical path and the no-selection-change witness are the fixed
  point;
- `OW-RR-6D` — the real-gallery GPU oracle initially crossed an upward test
  dependency from layout tests into `render`. Rewire moved the observation to a
  feature-gated `diagnostics::render` helper consumed by the dedicated
  `renderer_debug` crate. The main renderer keeps its closed typed boundary,
  production does not depend on the debug crate, and architecture ratchets
  reject a demo-app or GPU dependency leak;
- touched production modules shed stale commit-coordinate expectations and
  unused pulse state while preserving the sole semantic identity and the
  structure/value split. The final gauge reports 47 top-level modules, 333
  production and 113 test-only edges, three split responsibilities, 55
  provisional slot edges, **zero forbidden edges, zero external-boundary
  violations, and zero slot SCCs**, 1,989 `pub(crate)` declarations in 195
  production files with a 1,939 cross-slot bound, 90 cross-slot test edges, 120
  source-root mentions, 383 filesystem reads, seven allowances, five production
  panics, and 55 production expects.

Verification freeze, 2026-07-15: `cargo fmt --all --check`,
`cargo check --workspace --all-targets`, diff hygiene, and the protected
`comparison_open: true` state passed. The full library tier passed 1,173 tests
with 11 intentional ignores; all 162 layout/scene and 151 architecture tests
passed independently. Workspace all-target compilation, all maintained example
tests, and all four doctests passed. The release deep tier passed all 11 WARP,
shader, alpha, glyph, popup, material, and text witnesses; the specialized
release debug tier passed all six semantic-work, ordered-commit, retained-scroll,
churn, real-gallery oracle, and lifecycle witnesses. All five renderer-receipt
and ten One-Way parser/admission tests plus the full ownership census passed.
The release gallery was launched and exercised directly after the final build;
no external profiler, external machine, person, network service, or returned
artifact participated.

Checkpoint 6 is independently green. Checkpoint 7 now owns the measured 240 Hz
deadline misses and the broad interaction-choppiness report while deciding, by
evidence, whether Qt-class synchronous activation is sufficient or a
Chromium-class mechanism is admitted.

## Checkpoint 7 — prove Qt class and decide the ceiling

Repeat the complete Checkpoint 0 matrix with the legacy path still available
only to the oracle.

Qt-class closure requires:

- the literal-zero in-window scroll family;
- renderer draw p95 inside the recorded refresh period with zero
  renderer-owned deadline misses;
- bounded-cheap guard replenishment inside the pinned 16.667 ms ceiling;
- no discrete-GPU regression in latency or completed Pay Once topology;
- ordinary opaque rendering free of the unconditional full-window
  intermediate/blit;
- unchanged content reusing scene and GPU resources with zero content upload;
- deep-tier pixel, popup alpha, material, scale, and hardware witnesses green;
- bounded retained memory after virtual scrolling, menu churn, multi-window
  churn, device loss, and close/reopen;
- local human-eyes acceptance, with any available weak-GPU run recorded as
  corroboration rather than authority.

Then classify every remaining missed deadline by owner:

| Residual | Candidate mechanism | Admission evidence |
| --- | --- | --- |
| Semantic commit/replenishment blocks a drawable active state | pending/active commits plus independent preparation/render ownership | Repeated instrumented deadline misses during bounded commits after all synchronous work is minimized |
| GPU fill/bandwidth remains dominant | damage, occlusion, or partial present | In-code renderer counters after direct rendering and effect bounding; external profiler only if fill ownership remains unresolved; surface preservation contract proven |
| Property submission itself is too expensive | more compact property buffers or compositor delegation | Changed-property counts/bytes and GPU/CPU timing isolate it |
| Draw/state setup remains dominant | stronger batch roots/atlas/indirect plan | Debug-crate/state counters prove setup rather than content upload/fill/present wait, or an admitted external profiler resolves the remaining ambiguity |
| Present cadence/driver dominates | surface/backend follow-up | Renderer counters and GPU work are inside budget while present wait is not |

Each candidate receives an accept/reject receipt. A render thread is not
admitted because Chromium has one. Damage is not admitted because a retained
renderer exists. If Qt-class meets the pinned product thresholds, rejecting
all heavier mechanisms is a successful deletion-shaped verdict, not unfinished
work.

### Checkpoint 7 evidence ledger — complete

Owner split and causal correction, 2026-07-15:

- attempted/presented frames now classify property-only and semantic work
  independently. Batch preparation, encode/submit/present, complete renderer
  draw, and refresh-relative deadline misses carry the same classification.
  The existing upstream phase distributions for native translation, event
  handling, the complete native event pass, view rebuild, composition
  reconciliation, presentation layout, and scene assembly are emitted beside
  them. No profiler or external machine was required to assign the owner;
- the admitted property witness is
  `target/release/examples/renderer-receipts/control-gallery-500px-in-window-scroll-cp7-split-1784100266289.txt`.
  Its 52 property frames report batch-prepare p95 0.273 ms,
  encode/submit/present p95 1.219 ms, complete draw p95 1.533 ms, and **zero
  property renderer deadline misses** on the 4.167 ms development refresh.
  The three aggregate misses belong exclusively to 16 semantic setup frames;
  they are not relabeled as scroll misses. The Checkpoint 6 receipt and the
  literal-zero/runtime/readback witnesses remain the admitted clean workload
  proof because the current native automation bridge cannot reproduce small
  wheel deltas at a requested cursor coordinate reliably;
- the matching guard witness is
  `target/release/examples/renderer-receipts/control-gallery-800px-guard-boundary-cp7-split-1784100277718.txt`.
  Sixteen guard crossings produced 91 semantic frames and 32 renderer deadline
  misses. Semantic batch preparation p95 is 6.059 ms,
  encode/submit/present p95 is 3.392 ms, complete draw p95 is 9.418 ms, and the
  complete native event pass reaches 20.965 ms, including 9.107 ms of
  presentation layout. Replenishment itself remains bounded at 6.490 ms,
  inside the Qt-class 16.667 ms synchronous ceiling, but a 240 Hz active state
  is not independently drawable while that semantic work completes;
- the broad interaction-choppiness report exposed one additional whole-commit
  coupling. `CachedScrollLayer` was keyed by the enclosing commit pointer, so
  typing, selection, sliders, and any unrelated semantic change discarded and
  rerasterized the million-row table layer. The cache now borrows the exact
  ordered subtree identity/revision currencies and samples only that subtree's
  raster-affecting transform, opacity, clip, and blur values. Scroll remains a
  composited parameter and does not invalidate its content texture. A new
  dedicated GPU witness changes an outside node, proves exact pixels, records
  retained-layer hits with zero misses, and proves fewer draws;
- the same audit found the retained shape-property buffer using the enclosing
  commit pointer as an upload key. It now compares the realized bytes. A
  semantic commit with identical property realization uploads zero property
  bytes; a scroll tick also uploads zero unchanged shape-property bytes while
  remaining pixel exact. No duplicate invalidation authority was introduced;
- the post-correction live receipt is
  `target/release/examples/renderer-receipts/control-gallery-typing-selection-cp7-final-1784101498259.txt`.
  Forty-eight continuous typing/selection semantic frames record zero property
  upload, 182 scroll-layer cache hits, 52 draws on the latest frame, renderer
  p95 4.116 ms, and key-to-present p95 6.979 ms. Repeated large/fast scrolling
  in the 800 px gallery reached record 140 without recreating the reported
  over-limit texture or terminating the process. The gallery was closed after
  each smoke.

Ceiling decisions:

| Candidate | Verdict | Receipt |
| --- | --- | --- |
| pending/active commits with independent preparation/presentation ownership | **Accept for Checkpoint 8** | Guard semantic work produces 32 refresh misses and a 20.965 ms native event pass after unrelated subtree rerasterization and redundant property upload were removed. A complete active state must remain drawable while that pending work prepares. |
| stronger guard-path batch roots/atlas/indirect planning | **Accept as supporting Checkpoint 8 work** | The guard frame still carries 257 draws with one pipeline and bind transition per draw and 6.059 ms batch preparation. This is setup/content preparation evidence, not fill or present-wait inference. |
| damage, occlusion, or partial present | **Reject** | Ordinary rendering is direct with zero unconditional full-surface blit; property draw is 1.533 ms and acquire wait p95 is 0.031 ms. No code-owned evidence assigns the remaining guard miss to whole-surface fill or copy bandwidth. |
| more compact property buffers or compositor delegation | **Reject** | Exact realized-byte reuse reduces unchanged semantic and scroll shape-property upload to zero; property frames have zero misses. Popup opacity already uses the earned OS-compositor path. |
| surface/backend cadence follow-up | **Reject** | Acquire wait is 0.026–0.031 ms and encode/submit/present remains below the semantic preparation/layout costs. Renderer and upstream preparation, not the DX12 surface clock, own the residual. |

Touched-module cleanup cells:

- `OW-RR-7A` traces `native event -> semantic/property classification -> draw
  report -> presentation receipt`. One classification is sampled at render
  attempt and consumed by every timing distribution; no diagnostic clock can
  promote an attempted frame to presented truth;
- `OW-RR-7B` traces `scene ordered subtree -> borrowed NodeId/revisions ->
  retained scroll texture`. The whole-commit weak key and changed-list
  false-positive path are deleted. Exact structure plus exact subtree property
  values are the sole cache admission, and liveness remains borrowed from the
  owning scene nodes;
- `OW-RR-7C` traces `scene properties -> realized NodeProperty bytes -> GPU
  buffer`. The commit pointer no longer causes an upload. Viewport and property
  bytes each write only when their owned realized value changes; the debug
  crate ratchets both semantic and property-only reuse;
- `OW-RR-7D` traces each missed deadline through upstream preparation,
  renderer preparation, surface acquisition, and presentation. It admits one
  larger mechanism only where the measured owner demands it and records four
  explicit rejections, so Checkpoint 8 cannot absorb unrelated Chromium
  machinery by analogy.

Verification freeze, 2026-07-15: formatting, workspace all-target compilation
without warnings, diff hygiene, and the protected `comparison_open: true`
state passed. The full library tier passed 1,175 tests with 11 intentional
ignores; all 162 layout/scene and 151 architecture tests passed independently.
All maintained example tests and all four doctests passed. The release deep
tier passed all 11 WARP, measured text, shader, alpha, glyph, popup, and
material witnesses; the specialized release debug tier passed all seven
semantic-work, ordered-commit, retained-scroll, unrelated-semantic, churn,
real-gallery oracle, and lifecycle witnesses. All 15 renderer-receipt and
One-Way parser/admission tests passed. The ownership census reports 47 modules,
333 production and 113 test-only edges, three split responsibilities, **zero
forbidden edges, zero external-boundary violations, and zero slot cycles**,
1,991 `pub(crate)` declarations in 195 files with a 1,941 cross-slot upper
bound, 90 cross-slot test edges, 120 manifest-root mentions, 383 filesystem
reads, seven allowances, five production panics, and 55 production expects.
The exact-tree release gallery showed continuous typing/selection, survived 12
large 800 px scroll injections through record 140, and was closed with no
remaining process.

Checkpoint 7 is independently green. Checkpoint 8 owns the admitted
pending/active and preparation/batching work; damage, compact property
submission, and surface/backend changes carry no silent acceptance debt.

## Checkpoint 8 — optional Chromium-class upgrade

This checkpoint exists only if Checkpoint 7 admits it. Otherwise its closing
record is the rejection receipt.

If admitted, the new invariant is:

> A complete active state is always drawable while a pending semantic commit
> prepares. Missing a commit deadline keeps presenting the active state;
> activation is atomic when all resources required by the pending state are
> ready.

The upgrade may introduce a render/preparation thread only with one owner for
device/queue/resource mutation and a complete failure/teardown model. The
already-ratified scene contract does not change.

Required witnesses:

- active commit remains presentable throughout a deliberately delayed pending
  commit;
- property ticks, scroll, and admitted animations continue on the active state
  while pending prepares;
- pending resources never leak into active output before activation;
- one activation atomically switches commit structure and compatible property
  state;
- stale pending work is cancelled or retired without becoming active;
- surface loss, resize, device loss, window departure, and popup teardown
  cannot strand either state;
- input continues consuming only receipted active geometry/properties;
- bounded memory includes active plus at most the admitted pending/recycle
  states;
- instrumented telemetry demonstrates the deadline miss that was removed.

This is not license for Chromium's tiling, checkerboarding, GPU process, or
unbounded raster task graph. Each would require a separate caller and receipt.

### Checkpoint 8 evidence ledger — complete

The following progress-preservation receipt records the evidence and open
defects at its 2026-07-15 historical boundary; the closing receipts follow it:

- incremental preparation, active-output, pending-property, resize, caret, and
  production-gallery transition witnesses now exercise the admitted
  pending/active mechanism through the specialized debug crate;
- strict suppression of every completed commit with a newer successor was
  found to guarantee starvation under continuous semantic input. The queue now
  permits one complete accepted prepared state to activate atomically while
  retaining only the latest successor for continued preparation. A failed
  activation is requeued ahead of that successor. “Newer exists” is therefore
  not by itself the definition of stale; invalid, cancelled, incompatible, or
  explicitly displaced work remains forbidden from activation;
- the intermittent fresh-scene topology panic was traced to sampling newer
  live scroll truth after declaring a commit's resident scroll topology. Fresh
  properties now originate from the declaration baseline; current interaction
  truth belongs to the property clock. Focused and repeated live proof remains
  open at this preservation point;
- the physical-grid correction is now code-owned at all three boundaries that
  exposed it. Near-integral localized clip edges preserve their original grid
  line before outward scissor rounding; immediate and retained rounded shapes
  use one analytic coverage width independent of target-local derivative-quad
  parity; group paint localization preserves the global device phase while
  retained text prepares on that global grid and only then subtracts the
  snapped target raster origin. Focused source/coordinate ratchets cover each
  owner;
- the exact 18-case compatibility/retained oracle is green at
  1.0/1.25/1.5/2.0. The real 760-by-660 Control Gallery property transition is
  also green at all four scales under its original four-channel-value floating
  blend tolerance; no tolerance was widened. The synthetic retained-scroll
  transition now runs the same four-scale matrix and reports exact pixels plus
  literal zero content work at every scale;
- the cached scroll layer was subsequently rejected as the wrong resource
  shape. Its texture envelope scaled with the retained resident window, had
  already attempted a 16,862 px allocation against an 8,192 px device limit,
  and made transparent cache-edge sampling representable. Scroll content now
  renders directly under its declared rectangular clip. Shapes consume one
  shared sparse scroll uniform per structural scroll scope, retained text uses
  one bounded viewport offset per scroll/target scope, and clips, groups, and
  pane effects translate at their existing owners. No retained scroll texture,
  layer composite, segment cache, or device-limit branch survives;
- the four-scale retained-scroll witness is exact after that deletion. A
  changed nested scroll writes exactly one 16-byte shape scroll property,
  unchanged repetition writes zero bytes, and both paths report zero scene
  paint, shaping, text preparation, primitive preparation, and content upload.
  The complete 15-witness specialized release tier remains green, including
  production-gallery pending scroll, pending/active isolation, atlas pressure,
  resize, caret, lifecycle, and churn;
- the final non-debug release smoke survived two consecutive ten-input fast
  scroll bursts from record 0 through record 428, then resized the Detail/Note
  divider, presented continuous 34-character text input, and presented a live
  drag selection with the focus outline stable. The process remained alive and
  was explicitly closed with no remaining Control Gallery process;
- `OW-RR-8A` traces active/pending realization through the renderer's mutable
  state. Shape properties now use commit-owned copy-on-write GPU slots: equal
  bytes may share, an exclusively owned slot may advance, and divergent active
  and candidate states cannot overwrite one another. Candidate synchronization
  no longer encodes or submits retained scroll-layer GPU work ahead of the
  active surface draw. The active-output witnesses require literal zero active
  property upload during every pending slice. Architecture ratchets forbid the
  displaced single `last_property_commit` cache, pending scroll-layer submit
  path, and the former `Prepared` readiness species;
- that correction split the original black-output interval into two measured
  owners. Before the queue-order correction,
  `control-gallery-500px-idle-1784129047732.txt` recorded semantic
  encode/submit/present p95/max at 1,919,798 us because pending GPU raster work
  sat ahead of the active draw. After removal,
  `control-gallery-500px-idle-1784129389531.txt` reduced that distribution to
  5,824 us and the observed incomplete interval from about 2.5 seconds to about
  0.5 seconds. This is preserved as causal evidence, not claimed as closure;
- `OW-RR-8B` found the remaining incomplete-text owner at the shared glyph
  atlas. Upstream glyphon 0.11.0 clears its live allocation set on
  `TextAtlas::trim` but exposes no capability for retained prepared renderers to
  reassert the glyph allocations still referenced by their vertex buffers. The
  code-owned pressure witness first failed with **7,577 changed active pixels**
  while unchanged text reported zero preparation. The pinned local glyphon
  source adds only `TextRenderer::retain_prepared`: preparation records opaque
  cache keys and the trim owner re-pins every live retained renderer without
  shaping, rasterization, vertex rebuilding, or GPU upload. The source copy
  preserves upstream licenses and carries an explicit removal condition when
  upstream gains an equivalent capability;
- the first synthetic atlas witness was falsely green because retained
  offscreen readback omitted production's trim boundary. Correcting the harness
  made the failure deterministic before the mechanism changed. The complete 15
  witness release debug tier is now green, including forced atlas pressure,
  active/pending output, production-gallery transition, scroll, caret, resize,
  churn, and exact semantic work. The architecture ratchet requires the
  source-pinned capability and the one shared renderer atlas rather than
  permitting per-node atlases or unchanged-text repreparation;
- the post-correction release gallery survived and immediately presented two
  consecutive ten-input downward fast-scroll bursts from records 0 to 210 to
  424 with complete pixels. Column resizing, rapid typing, selection dragging,
  and caret blinking also remained complete; the focus outline stayed stable
  across the caret interval. The process was explicitly closed. An earlier
  non-moving-direction capture occurred while native feedback/hover windows
  were present and was not accepted as a scroll receipt;
- a later Windows Graphics Capture smoke briefly displayed a black fixed
  prefix during rapid input even though the settled gallery was complete. A
  temporary code-owned native-surface readback hashed the exact submitted
  upper half on every active refresh, pending projection, activation, and
  property tick. Every 1,270-by-672 sample was identical
  (`aa518477cb6ed005`) with 853,440 nonblack samples while the capture API still
  showed black. A blocking queue fence and FIFO-present experiment changed
  neither the capture behavior nor the hash. This classifies that observation
  as stale/partial capture output rather than renderer output; the temporary
  readback, presentation logging, fence, and present-mode experiment were all
  deleted. The independently exact surface pixels and real process-survival
  smoke remain the acceptance rails;
- the new `tools/renderer_debug/README.md` documents the independently runnable
  oracle, code-owned evidence order, comparison discipline, witness-extension
  rules, and the separate mandatory runtime-smoke rail;
- the touched-module scan found the feature-gated caret oracle constructing an
  `input::Input` inside diagnostics, creating a forbidden `diagnostics -> input`
  edge and a diagnostics/runtime slot cycle. The oracle now uses the runtime's
  existing programmatic focus capability, which owns caret-clock reset without
  importing input vocabulary. The focused caret GPU witness passes and the
  census is restored to zero forbidden edges, external-boundary violations, and
  slot cycles;
- the final readiness cell moved retained text viewport transforms out of draw
  and into commit-owned copy-on-write property slots. Equal active/candidate
  values share one slot; divergent values allocate or reuse a distinct slot,
  cancellation removes only the cancelled commit's ownership, and draw performs
  an exact key/value lookup with no queue write. A missing exact transform is a
  typed renderer error rather than silently omitted text. Candidate readiness
  is now minted only after shape and text property realization both complete;
- the replacement code-owned production-gallery activation benchmark runs one
  warmup and eight measured DX12 samples through the specialized debug crate.
  On the NVIDIA GeForce RTX 4070 Ti SUPER, Windows x86-64, scale 1.0 rail, the
  final full-tier receipt reports 70 us warmup, 25 us p50, and 52 us p95/max
  against the 4,167 us refresh-relative ceiling. Every sample required multiple
  incremental preparation slices and remained pixel-equivalent to synchronous
  realization. This replaces the historical 38,962 us activation owner with a
  complete transactional readiness receipt rather than hiding it in draw;
- the post-mechanism ordinary release Control Gallery smoke sustained and
  crossed scroll guards through record 243, moved slider capture from level 42
  to 68, resized the Detail/Note divider, presented continuous text input and a
  live drag selection with a stable focus outline, and remained alive until
  explicit closure. Windows Graphics Capture again showed transient partial
  black regions during high-rate input and then settled to complete output;
  this is the already code-classified capture artifact, not a new surface-output
  failure. No Control Gallery process remained;
- progress-boundary verification passed formatting, workspace all-target
  compilation without warnings, 1,188 library tests with 11 intentional
  ignores, three debug-crate unit tests, both Control Gallery tests, and all
  four doctests. The release deep tier passed all 11 WARP, text, shader, alpha,
  glyph, popup, and material witnesses; the specialized release tier passed all
  15 retained/oracle/pending/gallery witnesses. All five renderer-receipt and
  ten census parser tests passed. The final gauge reports 47 top-level modules,
  333 production and 114 test-only edges, three split responsibilities, 55
  provisional slot edges, **zero forbidden edges, zero external-boundary
  violations, and zero slot cycles**, 2,020 `pub(crate)` declarations in 196
  production files with a 1,970 cross-slot upper bound, 91 cross-slot test
  edges, 120 manifest-root mentions, 390 filesystem reads, seven allowances,
  five production panics, and 68 production expects. The increased test edge
  is the architecture ratchet reading the pinned dependency capability; no
  production ownership arrow moved.

Checkpoint-close verification, 2026-07-15: formatting and warning-free
workspace all-target compilation passed. The complete workspace target suite
passed three debug-crate tests, 1,187 root tests with 11 intentional ignores,
and both Control Gallery tests; all four doctests passed. The release deep tier
passed all 11 WARP, text, shader, alpha, glyph, popup-pack, and material
witnesses. The specialized release tier passed all 15 retained, oracle,
pending/active, atlas-pressure, resize, lifecycle, scroll, caret, and
production-gallery witnesses, including the eight-sample activation benchmark
above. All five renderer-receipt and ten census parser tests passed. The final
ownership gauge remains at 47 modules, 333 production and 114 test-only edges,
three split responsibilities, 55 provisional slot edges, **zero forbidden
edges, zero external-boundary violations, and zero slot cycles**, 2,020
`pub(crate)` declarations in 196 production files with a 1,970 cross-slot
upper bound, 91 cross-slot test edges, 120 manifest-root mentions, 390
filesystem reads, seven allowances, five production panics, and 68 production
expects. No external profiler, external machine, person, network service, or
returned artifact participated.

Checkpoint 8 is independently green. Checkpoint 9 now owns deletion of the
legacy renderer, compatibility oracle adapter, flattened paint/batch species,
and every orphan that served them; sole-renderer and campaign-closeout claims
remain pending until that burn-down and Checkpoint 10 complete.

## Checkpoint 9 — burn down the old species

The burn-down is one terminal sweep, not a deferred trash list.

Delete, subject to the live names at that checkpoint:

- the legacy renderer implementation;
- the oracle's legacy adapter and runtime A/B selector;
- private flattened `paint::Scene` and its item list;
- `RenderBatch`, `PreparedScene`, and `SceneEncoder`;
- semantic-scene-to-flat-paint conversion and scale-flatten traversal;
- per-frame whole-scene quad regeneration/upload paths;
- the unconditional ordinary-window composition texture/blit path;
- window-wide invalidation variants and translations with no surviving owner;
- legacy-only diagnostics, environment flags, tests, aliases, helpers,
  allowances, compatibility branches, and comments;
- orphaned `pub(crate)` visibility and dead modules left by the replacement.

The exact list is re-censused rather than blindly matched to formulation-era
spellings. Every surviving old-looking mechanism must name a current owner and
consumer.

Plant structural-absence tombstones for the defect vocabulary with high
recurrence risk. At minimum, architecture witnesses forbid:

- a second scene identity type;
- `PresentationEpoch` in `scene::Commit`;
- reintroduction of `paint::Scene`, `RenderBatch`, or `SceneEncoder` species;
- content-owned renderer callbacks;
- the unconditional opaque-window full-surface blit;
- a permanent legacy renderer selector.

Re-pin renderer-topology witnesses to the new authorized physics. The old
oracle's deletion does not delete pixel evidence: direct new-renderer readback
witnesses and independently runnable debug-crate benchmarks remain. Delete all
temporary logging/export and any debug support that did not earn promotion to a
direct witness.

Run the One-Way census over every claimed and touched module. Record deletion
counts, visibility deltas, dependency/gauge changes, retained resource counts,
and the complete orphan sweep. Success is one renderer standing in a clean
territory, not one renderer standing in the other's rubble.

## Checkpoint 10 — close out and teach master design

The full instrumented matrix, WARP correctness rail, deep GPU tier, behavioral
suite, architecture suite, examples, and One-Way fixed-point sweep all pass from
a clean tree. Any evidence-gated external-profiler captures and available
weak-GPU receipts are recorded as corroboration, never as missing closeout
gates.

Only now does `docs/master_design.md` change from the old flattened account to
the practiced retained account. It records at least:

- the opening constitution verbatim;
- laws versus shapes;
- capability-boundary doctrine with open/closed guardrails;
- scene/presentation/render/surface ownership;
- carried composition identity and revision currencies;
- structure in commit, values in property state;
- three disjoint handoffs and one mutation clock per field;
- property-aware presented geometry and input;
- retained GPU realization, effect-island, and offscreen-owner laws;
- Qt-class synchronous activation and the accepted/rejected Chromium upgrade;
- cleanup/burn-down result and sole-renderer status.

The roadmap marks the campaign complete, prunes mechanisms rejected by the
Checkpoint 7 verdict, and names any independently admitted follow-up. Capture
artifacts stay ignored build output; the ledger retains their paths and
interpretation.

## Verification discipline

Every production checkpoint is independently reviewable and green. In
proportion to the changed seam, run:

- focused semantic and architecture witnesses during the cell;
- `cargo fmt --check`;
- root all-target compilation and `cargo check --workspace --all-targets`
  without warnings;
- the full library suite and doctests;
- all maintained example/application smokes;
- a real release Control Gallery launch-and-interaction smoke after every
  checkpoint that changes renderer topology, presentation scheduling, or input
  projection, and after any correction to those paths. The smoke exercises fast
  and large scrolling, continuous typing/selection feedback, and representative
  controls while monitoring process survival; code-only tests and offscreen
  readbacks do not substitute for this runtime-crash and pacing rail;
- One-Way census parser tests and the full ownership gauge;
- diff hygiene and protected-state checks;
- the release deep GPU tier whenever scene, alpha, text, material, shader,
  surface, or renderer topology changes;
- matching debug-crate benchmarks and code-owned telemetry at Checkpoints 0, 5,
  7, and after any admitted Checkpoint 8 mechanism; an external profiler is
  used only when that checkpoint records the question internal evidence could
  not answer.

Test counts are recorded at each boundary rather than frozen in formulation.
Optional unavailable hardware is recorded as unavailable, never simulated or
replaced by a source inference, and does not block a code-owned checkpoint.

One coherent mechanism or ownership correction is committed at a time where
practical. The user explicitly authorized checkpoint publication on
2026-07-15: every campaign commit from that point onward is immediately
followed by `git push` to its configured upstream.

## Non-goals

- changing public application behavior or visual design;
- changing the DX12-first production backend policy;
- requiring Vulkan or a particular external test machine;
- rewriting the proven `Context` / `Canvas` / `Surface` and native tenancy
  ladder without new evidence;
- adopting a generic immediate-mode drawing trait;
- exposing GPU vocabulary to widgets, layout, composition, or scene content;
- minting renderer identity or hashing primitives to reconstruct it;
- retaining two production renderers after closeout;
- making damage, partial present, a render thread, tiles, raster workers,
  checkerboarding, or a GPU process mandatory without the Checkpoint 7 gate;
- opportunistic naming/module cleanup outside touched territory;
- treating WARP as an integrated-GPU performance proxy;
- weakening pixel/alpha/material/exposure laws to meet a timing target;
- carrying a rejected requirement as silent technical debt.

## Exit theorem

The campaign is complete only when all of the following are evidenced:

1. The opening constitution is practiced by ordinary content, not merely
   stated in documentation.
2. `composition::tree::NodeId` is the sole semantic identity through layout,
   scene, presentation, renderer caches, material regions, and cleanup.
3. The authoritative change stream produces per-node content revisions and no
   parallel renderer invalidation authority exists.
4. Commit structure, property values, activation, and visibility receipts have
   disjoint fields with one mutation clock each.
5. The renderer consumes closed typed content and owns all global realization;
   no content-owned draw callback or private batch entitlement exists.
6. Unchanged scene content causes zero scene painting, text shaping/preparing,
   primitive preparation, content upload, and GPU resource creation.
7. In-window scrolling satisfies the literal-zero family and input consumes
   the successfully presented property transform.
8. Guard-boundary replenishment is bounded-cheap under the Qt-class presenter,
   or an evidence-admitted pending/active path keeps a complete active state
   drawable.
9. Ordinary opaque no-effect windows perform zero extra full-surface
   intermediate clears and blits; every remaining offscreen has a named
   semantic owner and bounded envelope.
10. Popup packing, premultiplied alpha, sRGB transfer, material realization,
    exposure, resize/recovery, device loss, and teardown remain exact.
11. Code-owned telemetry meets the pinned refresh-relative and 60 Hz ceilings,
    local human acceptance is recorded, and the discrete and WARP rails are
    green for their stated uses; available weak-GPU evidence is corroborative.
12. Every touched module is at One-Way fixed point, and the terminal census
    finds no orphaned visibility, helper, compatibility, diagnostic, or
    old-renderer structure.
13. The legacy renderer, oracle adapter, flattened paint scene, batch compiler,
    unconditional blit path, and mixed-species selectors are absent.
14. The surviving readback, architecture, topology, and performance witnesses
    ratchet the new physics directly.
15. `master_design.md` and the roadmap describe the practiced system and no
    rejected Chromium/damage mechanism remains disguised as pending debt.

At that boundary the answer to “can the new core plug in underneath the
current surface?” is no longer a design claim. It is the shape of the sole
running renderer.
