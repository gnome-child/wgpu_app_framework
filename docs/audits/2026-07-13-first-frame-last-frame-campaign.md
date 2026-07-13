# First Frame, Last Frame campaign

Status: in flight. `comparison_open: true`. No push.

Mission: make production native popups reproduce the composition probe's
visible frost and compositor-only fade through one framework-derived
silhouette and one opacity owner.

## Rails

- Census and deterministic reductions precede repair.
- Each checkpoint lands independently green.
- Formatting, all-target compilation, the library suite, doctests, three
  application smokes, hardware witnesses, and diff hygiene gate boundaries.
- The user's `examples/control_gallery/app/view.rs` 136-to-500 table-height
  comparison remains unstaged and untouched.
- No new blur or fade engine, arbitrary shadow margin, second silhouette,
  Vulkan composition path, public policy API, or unrelated renderer
  optimization is admitted without its required evidence.

## Constitution

- The framework owns popup appearance and geometry; a platform may rasterize
  only framework-declared effects.
- One retained panel rectangle and rounding derive frost clip, residual
  coverage, border, shadow mask, hit geometry, containment, and panel position
  within the native surface.
- Layout bounds own participation. Visual bounds own paint reach. Shadow
  outsets derive from the existing shadow recipe and paint bounds calculation.
- Fallback is an explicit resolution outcome, never an inference from an empty
  backdrop-operation list.
- Composition-backed popup frost, shadow, residual content, and border live
  beneath one root opacity. Fade frames require no application redraw or
  repeated `DwmFlush`.
- `DwmFlush` is the first-show freshness barrier, never a frame clock.
- Native placement does not imply compositor animation. A path without a
  demonstrated animator uses immediate show and hide.

## Ownership map

| Fact | Owner |
| --- | --- |
| Panel rectangle and rounding | Retained floating-panel geometry |
| Popup silhouette | One derived projection |
| Shadow recipe | Framework theme |
| Shadow outsets / visual bounds | Existing paint visual-reach calculation |
| Popup surface envelope | Native adapter consuming visual bounds |
| Hit and containment bounds | Layout geometry |
| Backdrop request | Framework material intent |
| Host backdrop sample | Windows composition realization |
| Frost geometry and clip | Framework silhouette projected by the adapter |
| Tint, noise, content, border | Framework scene and renderer |
| Realized coverage | Platform report |
| Material complement | Scene resolver |
| Whole-popup opacity | Composition root |
| First-show synchronization | Windows show-cycle adapter |
| Native placement / activation / z-order | OS window shell |
| Redirected fallback lifecycle | Immediate until a smooth animator is proven |

## Checkpoints

| Checkpoint | State | Boundary |
| --- | --- | --- |
| 0. Reductions, census, DropShadow gate | Complete | Four reductions, bounds census, hardware probe verdict |
| 1. Explicit material resolution | Complete | Explicit framework-backdrop / transparent / fallback base; GPU alpha witness |
| 2. One silhouette and visual bounds | Complete | Shared shadow reach expands composition surface; panel-local paint, frost, and input offsets |
| 3. One framework edge | Complete | DWM edge/corner/shadow retired for composition; painted one-pixel edge and framework shadow |
| 4. One compositor timeline | Complete | Root owns every visible part; retarget is continuous; hide precedes every teardown |
| 5. Honest redirected fallback | Complete | Native placement and animation split; Vulkan opens/closes with no pseudo-fade or afterlife |
| 6. Complete open-bill trace | Pending | Semantic-open through fade-complete timings; measured verdict applied |
| 7. Resistance and close-out | Pending | Deletion census, hardware eyes, doctrine, roadmap, full ritual |

## Checkpoint 0 initial receipts

### Four named reductions

1. **Frost lid.** `Scene::resolve_material` removes platform-realized backdrop
   layers from a retained glass pane. `Renderer::encode_pane` then interprets
   `backdrop_layers.is_empty()` as permission to paint the opaque glass
   fallback. A live HostBackdropBrush is therefore covered by renderer paint.
2. **Redirected pseudo-fade.** A native popup's first retained opacity is zero,
   but native scheduling requests only the compositor-fade completion deadline.
   Redirected realization bakes that sample into its scene and owns no
   compositor animation between the two states.
3. **Chrome tail.** Composition root opacity animates content, frost, and
   residual paint. `DWMWA_BORDER_COLOR` remains a full-opacity DWM fact until
   the retiring popup expires and its HWND is dropped.
4. **Late trace epoch.** `PopupFirstPresentTrace::new` runs inside
   `PopupWindow::new`, after HWND creation, surface configuration, and tenancy
   attachment. The stage named `created` excludes the work most likely to
   distinguish cold and warm opens.

### Bounds census

- `paint::group_bounds` already expands visual reach for shadow blur, spread,
  and offset; `group_bounds_include_shadow_blur_and_spread` pins the rule.
- Floating-panel scene paint emits shadow and border from the same panel frame
  and theme recipe.
- Overlay `Layer::bounds` currently serves panel placement, native surface
  size, popup request localization, native event routing, IME hosting, and
  dismissal containment.
- `PopupWindow.bounds` and `PopupEventTarget.bounds` retain only that panel
  rectangle. No native visual-envelope projection exists yet.
- Material-region geometry already carries the declaring pane rectangle,
  rounding, inherited clip, identity, and effective opacity.

The lower concept is therefore a promotion: the native surface becomes the
second consumer of existing visual-reach truth. Panel bounds remain the input
and containment currency.

### DropShadow gate verdict

The Windows hardware probe admits a composition `DropShadow`, with two
constraints that production must preserve:

- the shadow carrier itself must remain unclipped; clipping the carrier to the
  panel silhouette also clips away the blur outset;
- the rounded alpha mask must come from a visual participating in the live
  composition tree. An unattached `CompositionVisualSurface` source silently
  degenerates to a rectangular caster on this machine.

The accepted probe uses the framework's rounded panel geometry for frost,
surface tint, and the shadow mask; suppresses DWM border/shadow treatment; and
animates the container root while the application schedules only the next
phase deadline. Frost, tint, and shadow fade continuously with no redraw loop
or swapchain submission. A transparent dedicated caster was rejected because
it produces no shadow; production must use the already-visible residual
content/tint visual as the caster rather than add an opaque lid.

The probe's static capture background is instrumentation only. Transparent
surface margins, screen-edge placement, DPI derivation, and panel-local hit
geometry remain acceptance riders on checkpoints 2 through 4, where the real
surface envelope and event target exist.

> A visual may cast beyond its layout bounds only when its silhouette mask is
> live and its carrier is not clipped to that silhouette.

## Checkpoint 1 — explicit material resolution

The material projection now carries a resolved base disposition independently
of its remaining operation lists:

- `FrameworkBackdrop` asks the renderer to execute retained backdrop work;
- `Transparent` leaves the platform-realized frost uncovered;
- `Fallback` paints the declared readable base.

Native realization selects `Transparent`; missing, duplicated, or mismatched
reports still resolve to the fallback quad; paint-only ghosts explicitly select
`Fallback` while retaining their surface layers. The renderer no longer asks
whether `backdrop_layers` is empty to choose a base.

Hardware readback
`resolved_glass_base_witness_distinguishes_transparent_from_fallback` passed:
the center pixel was `[0, 0, 0, 0]` for a transparent realized base and
`[1, 0, 0, 1]` for the same recipe resolved to fallback. The ordinary boundary
held at 939 passed, 9 deliberate ignores, four doctests, all targets, and all
three application smokes.

> Fallback is selected by resolution truth, never guessed from the absence of
> work.

## Checkpoint 2 — one silhouette and visual bounds

`PopupProjection` resolves the panel and its visual envelope once per scale.
Its visual reach calls the existing `paint::shadow_visual_bounds` owner—the
same blur, spread, offset, and physical-pixel fringe used by group
compositing—then supplies:

- HWND/swapchain logical area and screen origin;
- panel origin within the expanded surface;
- paint-scene translation;
- composition material-region translation;
- physical-to-parent pointer translation.

Composition-backed popups consume the expanded envelope. Redirected popups
remain panel-sized because their shell shadow is not a framework composition
visual. The screen-edge policy preserves the panel anchor and permits visual
reach to clip at the physical screen edge; it never moves the panel to save a
shadow. Margin events translate outside the panel's retained bounds and cannot
become panel hits, containment, cursor, or accessibility geometry.

Four-scale projection tests prove positive integral physical offsets, larger
shadow envelopes, and exact pane translation. The popup event witness now
starts from a nonzero physical panel offset and lands on the unchanged parent
coordinate.

> Layout owns participation; visual bounds own reach.

## Checkpoint 3 — one framework edge and rounded shape

Composition-backed window construction now disables independent DWM rounding
and undecorated shadow. Its border applicator sends `DWMWA_COLOR_NONE` instead
of the theme color; redirected fallback retains its rounded DWM shell, shadow,
and `COLORREF` border path.

The retained popup outline remains the only visible edge. Outline width crosses
the scene-to-paint boundary as physical-pixel count, so the floating-panel
border remains one physical pixel at 1.0, 1.25, 1.5, and 2.0. The same material
region projection supplies the frost radius and the composition shadow mask.
The shadow consumes the framework recipe's sRGB color, alpha, blur, spread,
offset, and silhouette; no DWM shadow coexists on the composition path.

The production DX12 gallery witness showed one rounded dark menu with clean
corners, a natural unclipped shadow in the expanded envelope, visible material
behind the transparent residual, and no second DWM edge. Light-theme and
remaining scale captures stay in the final pending-eyes matrix.

> One visible edge has one painter.

## Checkpoint 4 — one composition timeline

The existing root animation is now the sole opacity owner for the composition
shadow, host-frost regions, transparent wgpu tenant, residual tint/noise,
controls, and painted border. Overlay scheduling still emits one semantic
presentation plus one completion deadline—no frame-rate redraw loop or
per-frame `DwmFlush` was introduced.

Retargeting no longer restarts from the new overlay model's zero sample. The
host retains the previous timeline parameters, evaluates their current eased
opacity at the retarget instant, and starts the replacement animation from that
exact value. A deterministic midpoint witness proves no discontinuity.

`PopupWindow::drop` now cloaks and hides the HWND before removing its subclass
or releasing any field. This covers ordinary retirement, parent teardown,
surface failure cleanup, and every other removal path; composition objects are
dropped only after the shell is no longer visible. A rapid 35 ms
open/close/reopen production DX12 witness retained one popup with no stale
content, duplicate border, shadow tail, or naked HWND frame.

> First presentation earns exposure; one compositor timeline owns every frame
> until disappearance.

## Checkpoint 5 — honest redirected fallback

Overlay capabilities now report native placement and native animation as
separate facts. A live DX12 context reports both. Explicit Vulkan and an
uninitialized/non-DX12 context report placement without animation.

Immediate native entries start stable at opacity one, schedule no fade
completion, and allocate no `RetiringPopup` when removed. They still cross the
fresh-first-present gate before exposure, retain legacy accent acrylic when it
realizes, and use readable fallback otherwise. The composition path is
unchanged.

The explicit-Vulkan release witness captured full menu content on the first
state observation after the click and no popup on the first observation after
dismissal. There was no 80/90 ms blank entrance, translucent first sample,
invisible retiring HWND, WinRT composition runtime, or DX12 device.

> Unsupported animation becomes immediate lifecycle, not simulated latency.

> One framework silhouette defines every popup pixel; the platform realizes
> declared effects, and one compositor timeline carries them from first frame
> to last.
