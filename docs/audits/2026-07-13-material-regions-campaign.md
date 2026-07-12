# Material regions campaign

Status: in flight. `comparison_open: true`. No push.

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
| 3. Windows ownership/backend policy | Pending | DX12-first tenancy when earned; explicit/failed Vulkan path remains truthful |
| 4. Keyed Windows frost regions | Pending | Retained region diff, one tree/fade, four-scale/lifecycle/hardware matrix |
| 5. Retirement and doctrine | Pending | Evidence-based native-call inventory, master doctrine, roadmap close-out |

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
