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
| 0. Reductions, census, DropShadow gate | In progress | Four reductions, bounds census, hardware probe verdict |
| 1. Explicit material resolution | Pending | Realized frost remains transparent; fallback is selected, not guessed |
| 2. One silhouette and visual bounds | Pending | Popup surface consumes existing shadow reach while hit geometry stays panel-local |
| 3. One framework edge | Pending | Composition-backed DWM border/rounding retired; four-scale silhouette agreement |
| 4. One compositor timeline | Pending | Frost, content, border, shadow fade under one root; hide precedes teardown |
| 5. Honest redirected fallback | Pending | Vulkan opens and closes immediately; delayed pseudo-fade absent |
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

> One framework silhouette defines every popup pixel; the platform realizes
> declared effects, and one compositor timeline carries them from first frame
> to last.
