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
| 1. Retained material-region truth | In flight | Stable declaring identity; ordered request projection with clip and opacity provenance |
| 2. Reports and residual resolver | Pending | Actual realized-part coverage controls one residual plan and derived fidelity |
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
