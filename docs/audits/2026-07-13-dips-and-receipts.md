# DIPs and Receipts — One Silhouette, Atomic Popup Arrival

Status: in flight. Comparison remains open. No push during the campaign.

## Constitution

- One logical popup silhouette feeds two independent projections: logical to
  physical for renderer/input consumers, and logical-as-DIP for Windows
  Composition. Neither projection consumes the other.
- Prepared is not presented. Committed is not visibly realized until a
  screen-space witness proves that equivalence.
- Content and material arrive atomically through one root and one timeline.
- Readiness is receipted, generation-bound, and never manufactured by delay.
- State names carry their proof level.

## Protected worktree state

`examples/control_gallery/app/view.rs` contains Shea's pre-existing gallery
height edit and is outside this campaign.

## Census

| Cell | Finding | Receipt |
|---|---|---|
| U-01 | The authoritative region geometry is already logical scene geometry. | `scene::MaterialRegion::rect`, `rounding`, and `clips` in `src/scene/region.rs`. |
| U-02 | The popup projection already retains a logical panel offset, then separately exposes a physical derivative. | `PopupProjection::panel_offset` and `panel_offset_physical` in `src/platform/native/paint.rs`. |
| U-03 | Composition currently consumes the physical derivative and scales logical region geometry again. | `sync_material_regions(... panel_offset_physical ...)`, `project_region`, and `project_geometry` in `src/platform/native/composition.rs`. |
| U-04 | The hardware reduction measured the consequence at scale 1.25: Composition x=60 DIPs landed at 75 physical pixels; x=48 DIPs landed at the intended 60 physical pixels. | `material_shadow_probe` controlled geometry run, 2026-07-13. |
| R-01 | Material realization reports are emitted after API mutation/visual insertion, without a compositor completion receipt. | `Host::sync_material_regions` in `src/platform/native/composition.rs`. |
| R-02 | First exposure currently waits only for the renderer present/synchronization path. | `PopupFirstPresentTrace::record_presented` and the `PopupFirstPresentAction::Expose` branch in `src/platform/native/popup.rs`. |
| R-03 | The root entrance is prepared before presentation and begins immediately after exposure. | `Host::prepare_entrance`, `start_prepared_entrance`, and the exposure branch in `src/platform/native/popup.rs`. |
| R-04 | The controlled probe established that `GetCommitBatch(Effect)` completes while DWM-cloaked; `GetCommitBatch(None)` returns `E_INVALIDARG`. The prior 750 ms post-receipt hold proves sufficiency, not zero-hold equivalence. | `material_shadow_probe` readiness run, 2026-07-13. |

## Checkpoints

| Checkpoint | State | Evidence |
|---|---|---|
| 1. Zero-hold readiness probe | Complete | Controlled static-underlay capture probe; `GetCommitBatch(Effect)` alone was unstable in 1/10 runs at 100 ms (19.607 mean-channel delta). Two imperceptible host frames followed by the visible-root commit were stable in 10/10 runs (maximum 0.104 delta from 100 ms to 1 second). |
| 2. One logical silhouette, two projections | Complete | Composition consumes snapped DIPs and a named 48 DIP -> 60 px regression; four campaign scales and the full library suite are green. The screen-space evidence boundary remains the controlled Windows probe rather than swapchain readback. |
| 3. Generation-bound material readiness | Complete | Content and material receipts meet at one reveal gate; replacement invalidates older generations; duplicates are inert; no-material bypass, honest fallback, and single-root animation are pinned by behavioral and architecture witnesses. |

## Checkpoint 1 verdict

The first uncontrolled capture series was invalidated: HostBackdrop sampled the
live Codex window, which changed while tool output arrived. The final probe owns
a separate static patterned HWND beneath the material window and samples the
DWM-composed screen with GDI. This removes both external background motion and
the swapchain-readback blind spot.

The evidence distinguishes three meanings:

- `GetCommitBatch(Effect)` proves the effect batch is **Committed**. It does not
  prove that HostBackdrop has sampled a visible host frame.
- `DwmFlush` while the popup remains cloaked does not supply that sample; two of
  five controlled attempts still changed after reveal.
- A composition-backed popup may uncloak at opacity `0.001`, consume two real
  DWM frame barriers, commit its visible root, and then enter on the compositor
  timeline. That sequence had no delay constant, no application redraw, and
  was stable in all ten controlled cold runs.

Production vocabulary therefore remains `Pending -> Committed -> Ready`:
`Committed` is the effect receipt, while `Ready` requires the generation's
imperceptible host-frame preparation to finish. The probe's 100 ms capture is
an observation point only; it is not part of the readiness mechanism.

The retained probe now executes that proven route directly. Its final campaign
run reached effect commit, consumed two host frames at root opacity `0.001`,
synchronized the visible-root commit, and completed in 34.673 ms with no
application redraw. The 101 ms and 1,010 ms screen captures differed by 0.031
mean channel value inside the frost region and 0.000 outside it; the visible
frost contrast remained 31.896 versus 31.865.

## Checkpoint 2 implementation

The scene's logical material region remains the source. The Composition boundary
uses the shared grid only to snap that logical truth, then passes the resulting
logical coordinates directly as DIPs. It no longer multiplies offsets, sizes,
radii, shadow spread, blur, or shadow offset by scale. The renderer retains its
independent logical-to-physical projection and popup input retains the existing
physical panel offset.

The named regression pins the observed arithmetic: at scale 1.25, 48 DIPs land
at 60 physical pixels; the renderer's already-physical 60 px value cannot feed
back into Composition as 60 DIPs and land at 75 px. Architecture witnesses also
forbid `panel_offset_physical` from the Composition module.

## Checkpoint 3 implementation

Material projection changes allocate a new generation and acquire an `Effect`
commit batch. The current batch is polled from the retained Composition host;
there is no completion callback that can outlive its popup. Replacement drops
the prior batch, teardown drops the host, and the popup gate accepts a commit or
ready transition only when its generation matches the current pending state.

The renderer's first-present state is now named `ContentReady`, not `Expose`.
When both content and material commit are present, production reproduces the
probe's evidenced sequence: uncloak at root opacity `0.001`, consume two DWM
host-frame barriers, begin the one compositor root animation, then synchronize
that commit. Only that current generation earns `Ready` and logical exposure.
No material request bypasses the effect gate entirely. Any receipt, barrier, or
entrance failure re-cloaks the popup, abandons platform material, and forces an
opaque framework rerender; failure never manufactures readiness or strands a
transparent residual on screen.

The focused native suite and the full 961-test library run are green. Exposure
logs now distinguish effect commit from ready/exposed and record the additional
host-frame cost for the eventual field comparison.

## Production field acceptance

The release DX12 gallery was exercised at the available 1.25 display scale.
Screen capture showed framework content, frost silhouette, border, and shadow
coincident after the DIP correction. An immediate dismissal capture retained
content and silhouette together during the compositor exit fade; a capture 180
ms later contained neither, confirming atomic teardown rather than a border
tail.

The timing log also caught a final one-truth violation before closeout. The
first presentation generated material generation 1, then invalidated it with
generation 2 because overlay entrance opacity was baked into each material
region and divided back out at the Composition boundary. At zero opacity the
intrinsic region opacity was unrecoverable. This delayed the observed exposure
to 258.938 ms and made the root and region compete for the same meaning.

The correction is deletion-shaped: native popup material regions now retain
their intrinsic scene opacity unchanged, while `PopupPresentation::opacity`
remains the sole entrance/exit source at the compositor root (or at the legacy
renderer boundary). `Scene::with_material_opacity` and the Composition
ancestor-division parameter were deleted. Architecture witnesses now forbid
that duplicate path. A final production timing rerun remains the last closeout
witness.

## Evidence boundary

Swapchain readback ends before Windows Composition. It cannot prove backdrop,
composition shadow, or cross-layer agreement. The campaign uses the smallest
Windows-only screen-space witness necessary and does not promote a general
capture subsystem without another design pass.

## Exclusions

No new material recipe, Vulkan composition work, split frost/content entrance,
readiness sleep, third coordinate system, public widget/table API, or unrelated
border/shadow redesign.
