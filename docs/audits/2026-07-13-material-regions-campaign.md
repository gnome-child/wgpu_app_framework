# Material regions campaign

Status: complete. `comparison_open: true`. No push.

Mission: the scene submits ordered, keyed material requests; the platform
reports the parts it actually realized; one resolver derives the remaining
paint plan and final fidelity. Windows targets one HWND, one WinRT composition
target/tree, and wgpu as the content tenant.

## Constitution

- A material request carries retained identity, logical pane geometry and
  rounding, inherited clip provenance, effective opacity, material recipe,
  and independent scene order.
- Identity comes from the retained declaring world. Primitive order, traversal
  index, conditional position, and anonymous ordinal are never identity.
- Forecast, actual platform outcome, residual paint, and final fidelity are
  separate facts. Forecast never authorizes removing material from paint.
- Platform reports name realized material parts. `Full` / `Frost` / `Fallback`
  summarize the final result; they are not residual-assembly currency.
- Platform `None` is not fallback: in-frame can have no platform realization
  and still fully realize glass in the renderer.
- Presence changes are immediate. Scalar parameter settling remains a leaf
  policy; the keyed region set owns collection identity and report lifetime.

## Checkpoints

| Checkpoint | State | Boundary |
| --- | --- | --- |
| 1. Retained material-region truth | Complete (`4b33530f`) | Stable declaring identity; ordered request projection with clip and opacity provenance |
| 2. Reports and residual resolver | Complete (`00083457`) | Actual realized-part coverage controls one residual plan and derived fidelity |
| 3. Windows ownership/backend policy | Complete (`7fbd318f`) | DX12-first tenancy when earned; explicit/failed Vulkan path remains truthful |
| 4. Keyed Windows frost regions | Complete (`0a4c3aa5`) | Retained region diff, one tree/fade, four-scale/lifecycle/hardware matrix |
| 5. Retirement and doctrine | Complete (`7186c7e0`) | Evidence-based native-call inventory, master doctrine, roadmap close-out |

## Prerequisite receipt

The Windows show-cycle contract landed at `d4a6072b` and closed at
`2882d586`: popup content is presented and synchronized under an application
DWM cloak before first exposure. Full library gate and live repeated native
popup acceptance passed.

## Checkpoint 1 census

- `scene::Pane` currently owns `rect + rounding + material`, with no identity.
- Glass pane emission occurs in `scene::paint::paint_panel` while the declaring
  `layout::Frame`, its retained node identity, current frame clip, and panel
  presentation are available.
- The current widget grammar emits at most one material pane for a material
  panel frame. No current multi-region-per-frame caller has been found; an
  explicit local-key API is therefore not admitted.
- `scene::Primitive::Group` owns opacity and scene clip scopes are ordered
  `Clip` / `PopClip` primitives. Request extraction after the fact would have
  to reconstruct facts available directly during paint, so derivation belongs
  at pane emission rather than a second primitive walk.
- Scene order and retained identity are independent. The request collection is
  an ordered keyed projection, not a map and not separately authored state.

The remaining census before code is the exact retained ID accessor on `Frame`
and whether popup scene translation requires requests to translate through the
same owner as primitives.

## Checkpoint 1 close-out

The final identity is `layout::Frame::node_id()`, the existing process-transient
retained composition identity. The census found one material pane per current
material-owning frame, so no local key or public API was added. The ordered
`Vec<MaterialRegion>` is paint order only.

`MaterialRegion` retains the declaring ID, pane rect/rounding/recipe, effective
clip projection, and opacity. It is emitted atomically with the glass pane
while the `Frame` facts still exist. Overlay composition multiplies request
opacity through the same append operation that creates the primitive group;
native popup localization translates request geometry through the same `dx/dy`
as primitives. Ghost projection clears requests because ghosts are paint-only.

Evidence:

- Explicitly keyed panels retained IDs across sibling reorder; the request
  vector changed order independently.
- Conditional insertion before existing panels did not rename them; removal
  removed exactly the departed request and retained the survivor.
- Geometry, rounding, material, clip, opacity, native translation, and popup
  fallback absence have direct witnesses.
- The architecture witness pins derivation at `push_material_pane(frame.node_id(),
  ..., clip)` and forbids enumeration-derived identity.
- Full library gate: 912 passed, 8 ignored. Doctests: 4 passed. All-target
  compilation, formatting, all three smokes, and diff checks passed.
- No public API change. The unrelated gallery-height edit remains untouched.

## Checkpoint 2 census

The existing `native_popup_scenes` owns three coupled whole-popup decisions:
removing every glass pane, replacing every glass pane with fallback, and
selecting the first tint. `PopupMaterialRealization` then selects one complete
scene. This is the duplication checkpoint 2 replaces with keyed actual
coverage plus one residual resolver.

## Checkpoint 2 close-out

The runtime now passes one intact, localized material request to the native
boundary. It no longer predicts an OS-material scene and an opaque fallback
scene before the platform acts. Windows' legacy accent bridge reports frost
only for the supported single full-window region and only after the system
call succeeds; a forecast or failed call consumes no material.

`Scene::resolve_material` is the single complement owner. It combines the
requested pane, uniquely matching realized-part reports, and renderer context
to derive residual primitives plus per-region and aggregate fidelity. In-frame
uses the same resolver with no platform reports and retains the complete glass
recipe as `Full`. Native reports can consume backdrop frost and, only when the
applied accent parameters match, surface tint; unsupported or unreported
regions become their declared fallback without demoting reported siblings.

Evidence:

- Mixed frost/fallback reports produce independent residuals and one final
  fidelity per retained request.
- Reordered reports do not reorder regions or change identity.
- Missing, stale, identity-mismatched, and duplicate reports consume nothing.
- The intact-request boundary and successful-call gate have architecture
  witnesses; the former `opaque_fallback_scene` fork is structurally absent.
- The existing Windows accent and explicit fallback modes remain available as
  the legacy realization tier until tenancy replaces them.
- Full library gate: 916 passed, 8 ignored. Doctests: 4 passed. Formatting,
  all-target compilation, all three external smokes, diff hygiene, and
  `comparison_open: true` passed.
- No public API change. The unrelated gallery-height edit remains untouched.

## Checkpoint 3 close-out

Windows composition ownership is one UI-thread `DispatcherQueueController` and
`Compositor`, shared by the native runtime. Each earned popup supplies an
unattached classic DComp visual to wgpu, retrieves the live DX12 swapchain
through the public hal escape hatch, wraps it as a WinRT composition surface,
and places its content sprite in one framework-owned desktop target and tree.
No second HWND and no second target slot exist in the production path.

Backend policy is explicit and observable. An explicit `WGPU_BACKEND` attempts
only the requested backend. The implicit Windows policy attempts DX12 first so
tenancy can be earned, then retains the existing all-backends fallback if that
attempt fails. A forced Vulkan live run remained functional and reported
`tenancy=false`; a DX12 live run reported `tenancy=true`. Partial tenancy
construction is local and drops on failure before the popup records a host.

Opacity was separated from scene paint at this boundary. Legacy native
realization still bakes it once into paint; tenancy projects it once at the
common composition root. Reconfiguration preserves the swapchain COM identity
observed by the probe. A lost surface is now reconfigured and skips one frame
instead of terminating the runtime. True adapter/device removal remains the
renderer context's recovery boundary; the popup layer neither claims nor
attempts an independent device reconstruction.

Evidence:

- Live DX12 repeated presents, resize/reconfiguration, first-present conceal /
  synchronization / exposure, and full popup teardown passed.
- Forced Vulkan remained on the legacy realization path without partial
  composition ownership; explicit backend selection made no hidden attempt.
- Full library gate before checkpoint 4: 919 passed, 8 ignored; 4 doctests,
  all targets, three application smokes, and comparison state passed.
- No public API change. Direct Windows dependencies match wgpu-hal's crate
  family, avoiding a cross-version COM ownership transfer.

## Checkpoint 4 close-out

The Windows host now owns an ordered region container below the live content
sprite and a retained `NodeId -> RegionVisual` map. Scene order is refreshed
independently from identity; existing visuals survive reorder and parameter
updates, departed IDs are pruned, and reports are emitted only after geometry,
clip, visual update, and insertion succeed. An unsupported per-corner shape or
cutting clip declines only its own region and leaves it for residual fallback.

Geometry consumes the same `paint::Grid` and scene-to-paint rounded-rectangle
projection as renderer paint. Witnesses at 1.0, 1.25, 1.5, and 2.0 pin integral
physical edges and common radii. A production scene witness submits two nested,
independently retained material regions in one popup scene. The earlier probe
photographed two independently clipped regions in one HWND; production live
acceptance showed visible host frost with the wgpu residual tint and chrome
painted once above it.

Native popup fade is one compositor scalar animation at the common root.
Content, frost, and framework-painted residual therefore share one timeline;
native overlay scheduling retains only the completion deadline and performs no
per-frame application redraw for the fade. The first visible frame remains
gated by the completed show-cycle contract; if setup consumes the fade window,
the popup is exposed fully opaque rather than exposing a stale sampled frame.

Hardware and policy receipts on this Windows 11 machine:

- Explicit DX12 produced tenancy and `Frost` reports for all three glass-tuner
  surfaces; the legacy accent desire was disabled after region realization.
- The real `EnableTransparency` registry transition and `WM_SETTINGCHANGE`
  broadcast were exercised with live DX12 tenancy and restored in a `finally`
  path (`1 -> 0 -> 1`). Host frost remained realized on this machine. This is
  an outcome receipt, not a claim that every Windows policy or machine behaves
  identically.
- Warm host setup measured 2.3-6.4 ms. Initial one-region realization measured
  21-33 us, retained updates ordinarily 8-20 us, and graceful host teardown
  7-32 us. These are local hardware observations, not portable budgets.
- The comparison fixture plus native diagnostic log exposes backend, tenancy,
  final aggregate/per-region fidelity, per-region decline reason, and timing
  without adding product configuration API.

Boundary evidence: 923 library tests passed with 8 intentional ignores; all 4
doctests passed; formatting, all-target compilation, all three application
smokes, diff hygiene, and `comparison_open: true` passed. No public API changed,
and the unrelated gallery-height edit remains untouched.

## Checkpoint 5 retirement inventory

| Existing realization | Verdict | Owner after campaign | Evidence / missing witness |
| --- | --- | --- | --- |
| Legacy accent acrylic | Narrowed | Non-tenancy single-full-window bridge only | Tenancy reports host frost per region and no longer applies or disables accent; architecture absence witness holds |
| `DWMWA_USE_HOSTBACKDROPBRUSH` | Retained | Tenancy host capability | Required for `CreateHostBackdropBrush`; failed attribute yields no region reports |
| DWM border color | Retained | Outer HWND silhouette, from the same theme border datum | Painted residual owns framework chrome, but no isolated all-scale witness yet proves the DWM silhouette copy redundant |
| DWM corner preference | Retained | Outer HWND silhouette | Region clips do not shape the window itself; deletion awaits a replacement silhouette/hit witness |
| Undecorated DWM shadow | Retained | Honest outer-window shadow | No composition `DropShadow` or painted replacement has passed silhouette, clip, hit, and fade witnesses |
| Immersive dark-mode hint | Retained | DWM nonclient/silhouette styling | The composition material tree does not replace the DWM shell |
| DWM cloak + `DwmFlush` | Retained | First-visible-frame show-cycle contract | Required to present and synchronize current content before exposure |
| Geometry settle applicator | Retained | Popup HWND geometry | Composition regions are popup-local and do not place the window |
| Border settle applicator | Retained | DWM outer border | Retained with the border call |
| Accent settle applicator | Narrowed | Legacy non-tenancy bridge | Tenancy has no accent desired/applied state |

Visual similarity authorized no deletions. The one proven redundant operation
was the tenancy popup's initial `ACCENT_DISABLED` call; `7186c7e0` removes it.

## Final ownership table

| Fact | Owner |
| --- | --- |
| Material intent, recipe, logical geometry, clip and opacity provenance | Retained scene request |
| Region identity | Retained declaring `NodeId` |
| Region order | Current ordered scene projection |
| Device snapping | Shared layout-to-paint/platform `paint::Grid` projection |
| Windows frost visual and clip | Keyed composition host region |
| Surface tint, grain and framework chrome | Resolver residual paint |
| Realized material coverage | Platform report emitted after successful operation |
| Final per-region and aggregate fidelity | One scene resolver |
| Whole-popup opacity | Composition root for tenancy; scene paint for legacy |
| Outer HWND silhouette, corner and shadow | DWM shell until replacement witnesses exist |
| Model/session mutation reached through a projection | The projection's retained source, never projected geometry |

## Backend outcome matrix

| Path | Platform report | Residual / final result |
| --- | --- | --- |
| In-frame | None for every region | Renderer consumes the complete recipe; may reach `Full` |
| Windows DX12 tenancy, supported region | Host frost report | Renderer paints tint/chrome complement; final `Frost` |
| Windows DX12 tenancy, unrepresentable/failed region | No report for that ID | Declared native fallback for that region; siblings remain independent |
| Windows redirected non-tenancy, one full-window region | Legacy accent report only after successful call | Approximate frost complement; final `Frost` |
| Windows redirected non-tenancy without successful accent | No report | Opaque readable fallback |
| Explicit Vulkan | No DX12 tenancy attempt | Functional redirected legacy/fallback path |
| Failed implicit DX12 initialization | No partial host retained | Ordinary backend fallback set is attempted |

## Environment and performance receipt

Hardware acceptance ran on Windows 11 Pro for Workstations 23H2, build 22631,
with an Intel Core i9-13900F and NVIDIA GeForce RTX 4070 Ti SUPER, driver
32.0.15.9636. Explicit DX12 and explicit Vulkan were both exercised. Local
warm measurements are recorded in checkpoint 4; they remain observations, not
portable performance promises.

Surface `Lost` now reconfigures the existing surface epoch and skips one frame,
preserving the retained tree; resize/reconfiguration kept the wrapped swapchain
identity in the hardware probe. A true adapter/device removal is a renderer-wide
context-loss boundary, not a material-region downgrade. The popup system does
not claim independent recovery from a dead renderer device; that broader
reconstruction remains a framework limitation and must be solved at the render
context owner if a reproducible caller arrives.

## Remaining flags and next slice

- Exact Windows composition material remains deliberately deferred. Bare host
  frost does not claim recipe blur sigma, refraction, luminosity, saturation,
  noise, or exact tint realization.
- Public fidelity/configuration policy waits for that exact-effect caller. The
  request/report/residual seam will not need to change.
- macOS and Linux platform realizations remain research/hardware work; their
  honest current outcome is no platform report.
- DWM border retirement waits for an isolated painted-versus-DWM silhouette
  matrix at 1.0, 1.25, 1.5, and 2.0.
- Shadow ownership remains DWM by taste and evidence; a new owner needs complete
  silhouette, clipping, hit, fade, and cost receipts.
- Renderer-wide adapter removal/recreation remains a separate recovery goal.

## Campaign close-out

Roadmap item 18 is pruned and the master design now names the material-region
constitution, Windows ownership tree, backend policy, residual law, and fade
owner. Final boundary: 924 library tests passed, 8 intentionally ignored, all
4 doctests passed, all three application smokes exited successfully, and
formatting, all-target compilation, diff hygiene, protected comparison state,
and the unrelated gallery-height edit all held. No public API was added.

> Material is requested once, realized where capability permits, and painted
> exactly once everywhere else.
