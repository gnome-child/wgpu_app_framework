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
| 2. One logical silhouette, two projections | Pending | Four-scale arithmetic tests plus current-hardware screen-space agreement. |
| 3. Generation-bound material readiness | Pending | Receipt-order, stale-generation, teardown, and single-reveal witnesses. |

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

## Evidence boundary

Swapchain readback ends before Windows Composition. It cannot prove backdrop,
composition shadow, or cross-layer agreement. The campaign uses the smallest
Windows-only screen-space witness necessary and does not promote a general
capture subsystem without another design pass.

## Exclusions

No new material recipe, Vulkan composition work, split frost/content entrance,
readiness sleep, third coordinate system, public widget/table API, or unrelated
border/shadow redesign.
