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
| 0. Name the bill with PIX | In progress | Release DX12 timing and GPU captures; Vulkan framework comparison |
| 1. One clip owner | Pending | One logical viewport clip, one necessary realization |
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
