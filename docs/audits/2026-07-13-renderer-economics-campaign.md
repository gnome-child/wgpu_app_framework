# Renderer economics campaign: Pay Once

Date: 2026-07-13

Status: in flight

Comparison remains open: true

## Mission

Restore responsive menus, table scrolling, and column resizing by ensuring that
one semantic boundary produces one necessary realization. A frame is an ordered
stream of drawing commands, not a collection of items each entitled to its own
buffer, clip layer, and render pass. A popup session owns identity and
interaction; its native host is reusable infrastructure when hardware evidence
proves that reuse preserves readiness and correctness.

Retained content layers, damage tracking, dirty-rectangle presentation, custom
`Present1` integration, and composition-driven hover chrome are excluded.

## Rails

- Evidence precedes optimization. PIX Timing Captures supply performance
  evidence; GPU Captures supply frame-structure evidence only.
- Temporary diagnostic labels are permitted and must be removed at closeout
  unless admitted as generally useful diagnostics.
- Existing unrelated worktree changes remain protected.
- Each checkpoint is independently green and committed without pushing.
- Formatting, all-target compilation, the full library and doctest suites, all
  three application smokes, and comparison-example protection run at each
  implementation boundary.
- Checkpoints 2, 3, and 5 additionally run the release ignored hardware tier
  (`cargo test --release --lib -- --ignored`), including quad antialiasing,
  direct alpha, glyph coverage, group opacity, shared shader compilation, and
  popup sRGB packing. Pixel equivalence is not inferred from structural tests.

## Protected starting state

Starting HEAD: `d7747c76 Close DIPs and receipts campaign`.

The worktree already contains the active DIPs/readiness and alpha-coverage
closeout, the accepted 40% glass tint, and the user's 500-logical-pixel table
witness. These files are baseline inputs and must not be reverted, silently
reclassified, or absorbed into unrelated renderer commits:

- `docs/audits/2026-07-13-dips-and-receipts.md`
- `examples/control_gallery/app/view.rs`
- `examples/glass_tuner/app/state.rs`
- `src/platform/native/composition.rs`
- `src/platform/native/paint.rs`
- `src/platform/native/popup.rs`
- `src/render/filter/draw.rs`
- `src/render/renderer.rs`
- `src/scene/material.rs`
- `src/scene/mod.rs`
- `src/tests/architecture.rs`
- `src/tests/layout_scene.rs`
- `src/theme/toml.rs`

Campaign changes that must overlap one of these files will be isolated and
reviewed against the starting diff before commit.

## Checkpoint board

| Checkpoint | State | Evidence / decision |
| --- | --- | --- |
| 0. Name the bill with PIX | Complete | DX12 Timing Capture plus source/counter census; numeric floors pinned before renderer changes |
| 1. One clip owner | Complete | Contiguous members share one scene scope; rectangular clips are scissor-only; rounded clips retain one mask layer |
| 2. One reusable geometry arena | Pending | Zero steady-state ordinary-geometry buffer creation |
| 3. Semantic render-pass scopes | Pending | Pass count reaches the evidenced semantic floor |
| 4. Reusable popup-host probe and verdict | Pending | Reuse or negative result, never mechanism by assumption |
| 5. Responsiveness verdict | Pending | Identical before/after matrix and human-eyes confirmation |

## Checkpoint 0 protocol

The release-build matrix covers cold and repeated menu opens, menu switching,
command-palette open, menu hover, table wheel scrolling at 136/500/800 logical
pixels, and Count/Enabled divider resizing at 500/800. DX12 receives PIX Timing
and GPU captures. Vulkan uses the existing framework timing boundary.

Record application input-to-draw/exposure, CPU command preparation and encoding,
GPU execution and present wait, render-pass begins, draws and pipeline changes,
buffer creation/upload, clip pushes and unique geometries, offscreen clears and
composites, popup host construction, material commit/readiness, and cold-versus-
repeated open cost. Numeric success thresholds are pinned from these receipts
before production optimization begins.

## Findings log

- Painter law: items consume scopes; they do not realize them. Clip-per-member,
  buffer-per-batch, and pass-per-batch are the same entitlement error at three
  renderer altitudes.
- Popup law: semantic session identity and native-host infrastructure are
  separate truths. Reuse may change host lifetime but never session generation,
  focus, capture, input, or presentation freshness.
- Starting census: the 800px witness previously produced roughly 690 scene
  items and 677 renderer batches, but production submits one command buffer per
  frame. The campaign must not mislabel renderer batches as queue submissions.
- The current renderer opens main passes per prepared shape/text batch and
  realizes every pushed clip through an offscreen layer. The PIX census will
  determine their actual contribution before either mechanism changes.

## Checkpoint 0 — causal bill and pinned thresholds

The release DX12 control-gallery process was attached through PIX 2603.25. The
saved baseline Timing Capture is
`target/pix/pay-once-baseline-dx12.wpix` (capture artifacts remain ignored build
output, not repository history). The identical workload exercised the
500-logical-pixel table, repeated wheel movement, and repeated menu exposure.
A second 29.56-second GPU-timed run reported 19.91 frames/s and 50.43 ms between
presents during the interaction window. GPU process utilization remained about
23%, while the source census proved one command encoder and one queue submit per
frame. The dominant structural bill is therefore not 677 submissions: it is
CPU-side construction of hundreds of passes, transient vertex buffers, and
clip layers before that single submission, with DX12 making the waste more
visible than Vulkan.

The source/counter census supplied the causal counts PIX does not name in the
application vocabulary:

- every prepared ordinary-shape batch created its own `Quad Vertex Buffer`;
- every shape and text batch began a fresh load/store render pass;
- every `PushClip`, including ordinary rectangular table viewports, allocated,
  cleared, and later composited a full-target clip layer;
- scene painting emitted the same inherited table clip around each frame and
  each track rule independently;
- surface presentation already used one encoder, one submission, and one
  present, so additional submit coalescing is not an available mechanism.

Numeric success floors were fixed before production edits:

- rectangular viewport clip-layer realizations: **zero**;
- one clip command pair per contiguous equal scene scope, with nested distinct
  scopes intersecting once at their owner boundary;
- steady-state ordinary-geometry GPU buffer creation: **zero**, with one
  bounded frame upload and geometric growth only when capacity is exceeded;
- ordinary table draw passes: one per contiguous target/clip scope, plus only
  documented group, filter, pack, and presentation transitions;
- 500-pixel DX12 interaction: sustain the 60 Hz budget where presentation is
  available (p95 <= 16.7 ms), or name the remaining owner if hardware/present
  cadence prevents it;
- warm composition-backed menu exposure p95: at or below the prior present-only
  61–65 ms baseline, without weakening atomic frost readiness.

GPU-capture replay time is excluded from these thresholds. GPU captures are
used only to confirm pass/resource topology and pixel output.

## Checkpoint 1 — one clip owner

The first duplication existed on both sides of the paint boundary. Scene
painting wrapped every clipped frame and every table-track rule independently,
even when adjacent members inherited the identical viewport. Contiguous frames
and tracks now share one clip command pair; a clip transition, rather than a
member boundary, owns the scope.

At encode time, clips are classified by the geometry they actually require.
Axis-aligned rectangular clips remain on the clip stack and intersect through
the existing scissor calculation without allocating a texture. Full-target and
immediately duplicated clips become pass-through stack entries so ordering and
balanced pop semantics remain intact without realizing another scope. Rounded
clips keep the existing offscreen mask/composite path, and nested rectangular
clips inside a rounded owner draw into that owner's one layer.

The distinction is structural rather than a table special case: the same path
serves scroll viewports, table rules, popup-hosted content, focus chrome, and
scrollbar chrome. Named witnesses cover equal-scope coalescing, nested scissor
intersection, scale conversion, rounded-mask retention, and mixed scissor/layer
LIFO behavior. Boundary result: 955 library tests passed with 10 deliberate
ignores; four doctests, all-target compilation, formatting, all three
application smokes, and diff hygiene passed. The protected comparison state and
preexisting DIPs/alpha work remain untouched.
