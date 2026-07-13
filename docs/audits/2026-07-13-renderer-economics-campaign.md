# Renderer economics campaign: Pay Once

Date: 2026-07-13

Status: complete

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
| 2. One reusable geometry arena | Complete | One ordered frame upload; geometric growth only; zero steady-state ordinary-geometry buffer creation |
| 3. Semantic render-pass scopes | Complete | 137 descriptive batches became 6 ordered draw passes in the 500px DX12 witness |
| 4. Reusable popup-host probe and verdict | Complete | Four-interval probe plus production same-menu, cross-menu, palette, rapid-reopen, scale-invalidation, and parent-shutdown witnesses passed |
| 5. Responsiveness verdict | Complete | Six-case release matrix, after-capture PIX topology, deep hardware tier, and human-eyes acceptance passed |

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

## Checkpoint 2 — one reusable geometry arena

Ordinary quad geometry now has one renderer-owned arena rather than one GPU
buffer per prepared batch. Shape, rule, outline, shadow, pane-base, and pane-tint
vertices append in scene order; prepared batches retain only their vertex range.
The arena uploads once after preparation, grows to a geometric capacity only
when a frame exceeds its high-water mark, and retains that capacity for later
small or large frames. Renderer recreation remains the device-loss boundary,
so no GPU resource crosses devices.

The change deliberately does not reorder or merge draws. It replaces storage
ownership while leaving the command stream intact, including alpha-sensitive
ordering and existing clip/group boundaries. Public diagnostics now report the
frame's geometry vertex count, upload bytes, and buffer-growth count. The
steady-state acceptance condition is therefore observable as
`geometry_buffer_creations=0`; a growth frame reports one creation rather than
hiding it as ordinary work.

Capacity witnesses pin reuse at the current high-water mark and geometric
growth for sudden table expansion. The same arena is rebuilt with the renderer
after device loss, and viewport scale or popup-size changes alter generated
vertices without changing resource ownership. Boundary result: 957 library
tests passed with 10 deliberate ignores; four doctests, all-target compilation,
formatting, all three application smokes, and diff hygiene passed. The required
release hardware tier also passed all ten witnesses, including quad and glyph
AA, direct alpha, group opacity, popup sRGB packing, material-base distinction,
noise alpha preservation, and shared-shader compilation.

## Checkpoint 3 — passes correspond to semantic scopes

The encoder now treats consecutive shape and text batches as one ordered draw
run. One render pass spans that run while each consumer reasserts the pipeline,
vertex storage, bindings, and scissor state it owns. Scene order is unchanged:
the encoder does not sort opaque or translucent content. A pane requiring
material/filter work, a clip transition, a group composition, or an output
target change still terminates the run. Solid panes are ordinary geometry and
therefore join the surrounding draw stream instead of manufacturing a pane
boundary.

The live release DX12 witness made the distinction measurable. At the
500-logical-pixel table height, 242 scene items prepared as 137 descriptive
batches and 64 glyph batches but encoded through **6 draw passes**. Geometry
held at 636 vertices / 91,584 upload bytes with zero arena growth. The pass
count is now a public diagnostic rather than inferred from batch count. The
same run exposed the next owner honestly: its sampled p95 still included about
30 ms in batch/text preparation while encode-and-present was about 5 ms. Pass
fusion removed false GPU boundaries; it did not claim to solve unrelated text
preparation work.

The first interactive capture also enforced a subtle state law: glyph draws
may change render-pass state, so a later quad draw restores its scissor along
with pipeline and vertex buffer. Shared pass lifetime does not imply shared
state ownership. The user's immediate release-mode field verdict was that the
large table was already materially faster after this checkpoint.

Boundary result: 958 library tests passed with 10 deliberate ignores; four
doctests, all-target compilation, formatting, all three application smokes,
and diff hygiene passed. All ten release hardware witnesses passed again,
covering AA, glyph coverage, direct and grouped alpha, material transparency,
noise alpha, popup packing, and shader compilation. The full identical-backend
latency matrix remains Checkpoint 5's controlled verdict; automated Vulkan
input was stopped when live user input was detected rather than contaminating
the comparison.

## Checkpoint 4 — reusable popup hosts

The disposable `material_shadow_probe --reuse-ladder` exercised one retained
composition host through hidden intervals of 10 ms, 100 ms, one second, and ten
seconds. Every cycle changed window position and size, panel geometry, and
framework-painted content. Screen capture on the first exposed frame and after
settling verified frost contrast and fresh content. The ladder created one host
and one effect receipt in total; no cycle created another receipt and no late
receipt arrived. Measured show setup was 11.638, 17.373, 16.880, and 18.675 ms.
The initial effect committed at 40.49 ms. This is positive hardware evidence
for infrastructure reuse, including the ten-second interval; it is not a
timing inference from source.

Production now distinguishes a `PopupHost` (HWND, canvas/surface, presentation
mode, and optional composition tree) from a `PopupWindow` session (semantic
identity, geometry/accent/border desires, scene receipt, exposure, material
reports, and lifecycle epoch). The parent-owned pool retains only hosts. Every
acquisition constructs a fresh session and reinstalls a fresh raw-window
route; retirement removes that route, rehomes cursor authority, cloaks and
hides the host, clears its cursor and IME participation, and only then returns
it. The semantic IME target is released as well, so an identical next popup
cannot inherit an update that the native layer mistakes for unchanged. Parent
departure drops both active and dormant infrastructure. Pool capacity is the
observed simultaneous popup depth, so submenu nesting can raise the bound while
menu identities cannot.

The first stable parent presentation arms one root-host prewarm. The first
maintenance pass merely schedules it; construction occurs from the following
idle poll, after the parent is already exposed. The hidden host warms its
surface, renderer, and one host-backdrop visual while cloaked, and remains
non-routable. Receipt polling, not a delay constant, moves it to the dormant
pool. A user opening during prewarm may consume the same pending host receipt;
it still needs a fresh content-present receipt and the atomic reveal gate.

Material visuals are infrastructure too. A prewarmed or retired visual can be
re-keyed to the next scene's retained material-region identity and updated in
place. Geometry, corner radius, shadow, and semantic region identity may change
without creating another backdrop brush. A new effect receipt is acquired only
when the host must grow beyond its retained visual high-water mark; recycled
visuals continue consuming the already receipted effect generation. This is
the same identity-versus-infrastructure split one level below the HWND.

Admission remains narrow: only composition-backed sessions whose material
reached `Ready` return to the pool. Backend/mode or scale mismatch drops the
host, and a failed/disabled material path is never kept warm as if it were a
successful composition host. Fresh-host creation remains the complete fallback
for every prewarm or reuse failure.

The release gallery matrix closed the checkpoint with the same production
pipeline. A prewarmed first Edit-menu host exposed in 69.408 ms while paying a
one-time 30.886 ms content draw; the same menu reopened in 26.903 ms with a
1.497 ms draw. The first cross-menu Controls transition exposed in 43.342 ms
while briefly raising observed simultaneous depth to two, then reopened from
that bounded pool in 26.877 ms. The command palette first exposed in 50.549 ms
(previous baseline 71 ms) and repeatedly exposed in 23.584--25.242 ms. Typing a
query, closing, and reopening produced a fresh blank query, proving that the
reused host did not retain session or IME state. Rapid reopen kept complete
frost, content, border, and shadow on the first exposed frame. Scale and mode
mismatch exercise the fresh-host invalidation path; parent shutdown removed
all dormant hosts and prewarm state.

The capacity verdict is intentionally high-water rather than identity-based.
Normal use retains one dormant root host. A genuinely overlapping menu
transition raised the observed capacity to two; unrelated menu identities did
not. Retained surfaces reflected actual content sizes (for example about
353x325 for Edit, 353x133 for Controls, and 713x385 for the palette), and are
discarded on parent, presentation-mode, or scale incompatibility. No retained
content layer or damage vocabulary was introduced.

## Checkpoint 5 — release responsiveness verdict

The controlled after matrix used the same release workload at 136, 500, and
800 logical pixels on DX12 and Vulkan: alternating wheel input, then repeated
Count/Enabled divider drags at the larger sizes. Temporary p50 diagnostics and
six measurement-only example targets were removed immediately after the run.
The interval percentiles include deliberate input pacing and therefore are not
frame-time evidence; draw and preparation percentiles are the acceptance
currency.

| Table height | Backend | Scene items | Descriptive batches | Draw passes | Batch prep p50 / p95 (us) | Draw p50 / p95 (us) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 136 | DX12 | 128 | 82 | 7 | 585 / 799 | 1,239 / 1,988 |
| 136 | Vulkan | 144 | 90 | 7 | 311 / 436 | 879 / 1,178 |
| 500 | DX12 | 258 | 145 | 6 | 1,098 / 1,309 | 2,037 / 2,536 |
| 500 | Vulkan | 258 | 145 | 6 | 536 / 690 | 1,137 / 1,655 |
| 800 | DX12 | 296 | 166 | 7 | 1,092 / 1,298 | 1,952 / 2,438 |
| 800 | Vulkan | 290 | 161 | 6 | 619 / 734 | 1,368 / 1,703 |

All sampled draw p95 values are below 2.6 ms, far inside the 16.7 ms 60 Hz
budget. The larger tables no longer scale toward one pass or allocation per
member: the 500--800 pixel witnesses use 6--7 draw passes and zero steady-state
geometry-buffer creations while retaining 145--166 descriptive batches for
diagnostic ownership. The original large-view runs prepared roughly 605--677
batches and effectively paid a pass and transient buffer per batch. The result
is a structural reduction, not a cache-size tuning.

DX12 still carries about 0.8--1.3 ms more draw time than Vulkan at the larger
sizes, but both have ample budget. The remaining cold-popup cost is also
honestly bounded: host/material prewarm cannot precompile every content
pipeline or populate every glyph cache, so a first unseen content species may
still pay a 23--31 ms one-time draw. The warm path is 1--2 ms and 23--27 ms to
atomic exposure. If that cold bill becomes product-visible again, its owner is
content-pipeline/glyph warmup, not retained/damage rendering.

`target/pix/pay-once-after-dx12.wpix` records the after topology for inspection;
capture replay timings and capture file size are explicitly not performance
evidence. The release counters and deep pixel witnesses provide the numeric and
equivalence verdicts. Human acceptance supplied the final field result: the
release build is already materially faster, with scrolling and column resizing
responsive at the enlarged table size.

The campaign closes with both laws intact: items consume semantic render
scopes rather than realizing private infrastructure, and popup sessions remain
fresh even when their platform hosts are retained.

Final boundary result: formatting and diff hygiene passed; all targets compiled;
961 library tests passed with 10 deliberate hardware ignores; all four doctests
and all three application smokes passed; and the required release hardware tier
passed all 10 ignored witnesses. The comparison example remained protected, no
temporary diagnostic code remained, and the campaign was not pushed.
