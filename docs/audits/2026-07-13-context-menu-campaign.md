# The Nearest Command World — context-menu campaign

Status: complete; manual native-popup comparison remains pending eyes.

## Mission

Add context menus as the nearest declared object's local projection of the
existing command system. One widget helper authorizes the projection. The
framework derives candidates from the owner's bound trigger, exact responder
targets, and applicable local services, then reuses command metadata, menu
rows, menu lifetime, overlays, popup hosts, and placement.

The application does not normally author a menu recipe.

## Protected baseline

Campaign ignition: `d2bf6c77` (`Lengthen default popup fades`). The following
pre-existing modifications are protected and must not be absorbed into a
campaign commit unless their ownership is established explicitly:

- `docs/audits/2026-07-13-dips-and-receipts.md`
- `docs/audits/2026-07-13-renderer-economics-campaign.md`
- `docs/roadmap.md` (the completed renderer campaign's `None.` transition is
  pre-existing; this campaign owns only its own ignition and eventual pruning)
- `examples/control_gallery/app/view.rs`
- `examples/glass_tuner/app/state.rs`
- `examples/material_shadow_probe/main.rs`
- `src/platform/native/{adapter,composition,ime,mod,paint,popup,surface,window}.rs`
- `src/render/filter/draw.rs`
- `src/render/renderer.rs`
- `src/scene/material.rs`
- `src/scene/mod.rs`
- `src/tests/architecture.rs`
- `src/tests/layout_scene.rs`
- `src/theme/toml.rs`

No push during the campaign.

## Constitution

1. Membership is local; availability is live.
2. Target registration is the automatic recipe.
3. Nearest contextual ownership stops candidate discovery.
4. Discovery and invocation share a route; a candidate cannot be advertised
   by one owner and execute against another. This is prospective case law from
   the stale `ApplyEdit` fallthrough failure.
5. Keyboard focus and contextual ownership are separate facts.
6. Resolved command state is a projection, never retained menu truth.
7. Public construction is typed; type erasure remains private after capture.
8. Layout receives menu geometry and content; it never reads the registry or
   application model.
9. Authored and contextual menus share one lifecycle and realization path.
10. One placement solver consumes an anchor, desired size, preferences, and
    available bounds.

## Pattern census

| Concern | Existing owner and receipt | Campaign verdict |
|---|---|---|
| Typed command erasure | `command::Trigger<C>` to private `AnyTrigger` in `src/command/trigger.rs` | Reuse as candidate currency; no public erasure |
| Typed target erasure | application `target::AnyTarget` and runtime-service `runtime/services/target::AnyTarget` | Preserve the documented non-merge; share only candidate reporting |
| Erased response | `response::AnyResponse` and `runtime/transaction/command/any.rs` | Reuse transaction, history, effects, and observation |
| Other erasure patterns | `notification::AnyListener`, `task::AnyTask`, heterogeneous typed table columns | Repeat typed construction then one private erased boundary |
| Registry metadata | `command::Registry`, `Spec`, `State`, and `AnyTrigger::state` | One resolved-action projection supplies label, shortcut, check, and availability |
| Palette discovery | `Registry::resolved_unit_commands` plus `runtime/palette.rs` | Generalize resolution beneath a global provider; retain palette policy above it |
| Palette scopes | Focused, Transient, and Captured `responder::Scope` | Promote with contextual route; do not create a rival scope system |
| Instance arguments | `view::Binding` retains `AnyTrigger` | Direct owner binding supplies parameterized candidates automatically |
| Authored menu chrome | `Source::Menu`, `Participation::MenuRow`, `layout::menu_row_parts`, `paint_menu_row` | Context rows are ordinary menu rows |
| Menu lifetime | `interaction::Menu`, `session/interaction/menu.rs`, shared cancellation and dismissal | Generalize one menu session; no second outside-click machine |
| Retained identity | composition `NodeId`, element identity, removed-node pruning | Context owner is retained and stale sessions close on removal |
| Floating realization | `overlay::Store`, in-frame/native backend selection, popup presentation | Context menu is another floating panel draft |
| Placement | `root_floating_panel_rect` and menu-title anchoring in `layout/algorithm.rs` | Promote to one pure solver; remove parallel arithmetic |
| Platform bounds | parent screen origin/scale and Windows popup positioning exist; monitor work-area query is absent | Add a platform placement fact, not platform placement policy |
| Secondary input | platform translates `pointer::Button::Secondary`; shell/runtime drops non-primary behavior | Extend the existing input path |

### Proven resistance and non-merges

- The two `AnyTarget` storage forms have different borrowing and ownership
  requirements. Context menus are not evidence to merge them. Candidate
  providers must be distinct types, while both may report into one private
  erased candidate currency.
- View construction has no implicit "current responder." Context ownership
  therefore promotes `Scope` to carry an exact responder identity independently
  of optional focus. Layout must not synthesize or discover that identity.
- Responder targets with non-unit arguments cannot invent their arguments.
  Concrete bindings supply those arguments; any further public carrier requires
  a witnessed caller before admission.

## Surface policies

Candidate-provider kinds are type-distinct so a context surface cannot be
constructed over global palette discovery:

- Global provider: registry enumeration through a captured responder chain.
- Bound provider: the contextual node's concrete trigger.
- Exact-responder provider: unit targets declared by that responder only.
- Local-service provider: unit targets owned by an applicable widget service.

Palette and context then apply different presentation policies to the same
resolved-action vocabulary. The palette keeps enabled commands and fuzzy
orders globally. Context menus omit hidden commands, retain disabled commands,
and preserve local declaration order.

Describer exclusion is per surface, not inherited globally. A surface excludes
the command that opens that same surface. `OpenCommandPalette` may therefore be
legitimate in an explicitly contextual application root; the census and tests
must decide and record each surface's policy deliberately.

## Checkpoints

| # | Boundary | Status |
|---|---|---|
| 0 | Census, constitution, protected baseline, and roadmap ignition | Complete |
| 1 | One erased command projection with type-distinct providers | Complete — palette now consumes `Candidates<Global>` through one generic resolver; surface policy owns self-describer exclusion |
| 2 | Retained contextual ownership and exact local discovery | Complete |
| 3 | Secondary/keyboard request through one menu session | Complete |
| 4 | Existing menu-row and overlay projection | Complete |
| 5 | One placement solver with viewport/work-area bounds | Complete |
| 6 | Gallery, absence witnesses, doctrine, and full closeout | Complete |

## Explicit exclusions

- No public type-erased command or context-recipe API.
- No platform-native menu implementation.
- No context-specific menu row, painter, theme family, host pool, or fade.
- No global scan after a nearest contextual owner is found.
- No focus mutation solely for discovery.
- No retained command availability.
- No submenu sessions. The anchor-general placement solver is their future
  seam; this campaign neither builds nested sessions nor implies them.

## Boundary receipts

### Checkpoint 1

- The private command-surface vocabulary lives in `src/command/surface.rs`:
  provider identity is carried by `Candidates<P>` and `ResolvedActions<P>`,
  while `ResolvedAction` carries trigger, state, metadata, provenance, and
  listing policy.
- `Registry::global_candidates` performs only global discovery;
  `Registry::resolve_candidates` is the single erased resolver. The former
  palette-only `resolved_unit_commands` path is deleted.
- Palette policy remains in `runtime/palette.rs`: enabled filtering, fuzzy
  ranking, and exclusion of its own `OpenCommandPalette` describer happen
  after shared resolution. `Listing::Describer` is not a blanket global ban.
- Verification at the boundary: 12 palette/layout/platform tests passed; the
  invocation-source regression passed; all-target check passed; format and
  diff checks clean.

### Checkpoints 2–4

- One `context_menu` bit is retained on the ordinary composition node. A
  context geometry query includes inert display nodes without changing normal
  activation hit testing, then ancestor walking stops at the first marked
  owner.
- `Scope::contextual` separates exact responder identity from optional text
  focus. Exact responder and service candidates carry a private
  `responder::Route`; the same route is revalidated by the ordinary `Binding`
  during presentation and invocation. Direct bindings retain their concrete
  erased arguments and ordinary chain semantics.
- Application and runtime-service target storage remain distinct. Each reports
  only command types into `Candidates<Local>`; `Candidates<Global>` cannot be
  substituted by construction.
- Secondary-button release now reaches the runtime through the existing shell
  pointer path. Shift+F10 and the Menu key enter the same derivation path from
  focused-owner geometry. Opening never mutates focus for discovery.
- `interaction::Menu` gained only a contextual origin (owner plus anchor).
  Authored and derived menus still share exclusivity, focus restoration,
  Escape/outside dismissal, stale-node pruning, `Source::Menu`, overlay drafts,
  native popup hosts, fades, activation transactions, and history.
- Context actions project as ordinary `Role::Binding` children of the existing
  floating panel. Existing menu participation supplies measurement, shortcut
  and checked-state geometry, disabled chrome, painting, and hit behavior; no
  context row role or recipe exists.
- Boundary witnesses cover concrete argument preservation, exactly one bound
  action, unmarked-space resistance to global scans, nearest nested ownership,
  exact-route invocation, hidden-local non-fallthrough, text-service discovery
  with focus preservation, keyboard invocation, and stale-owner pruning.
  All 32 interaction tests passed, as did all-target compilation and formatting.

### Checkpoint 5

- `geometry::PlacementRequest` is the one pure placement projection. Point and
  rectangle anchors share down-right/down-left/up-right/up-left preference,
  full-fit selection, greatest-visible-area fallback, and origin clamping
  without changing the requested menu size.
- Layout resolves that request against the contextual owner's inherited clip,
  so nested viewports constrain an in-frame menu without teaching layout about
  commands or platform monitors. Authored menu-title anchors enter the same
  solver through a rectangle anchor.
- The request survives scene extraction, overlay retention, native-popup
  presentation, and retirement. A native host re-resolves it against the work
  area of the monitor containing the anchor; an unsupported native forecast
  retains the already resolved in-frame bounds. Placement policy therefore
  remains shared while availability stays host-owned.
- The Windows boundary projects the physical monitor work area into the
  parent's logical coordinate system once. Scale witnesses at 1.0, 1.25, 1.5,
  and 2.0 and a negative-origin monitor witness pin that conversion.
- Boundary verification: point-corner, rectangle-anchor, oversize, layout
  integration, work-area projection, and all 32 interaction witnesses passed;
  library compilation is warning-free.

### Checkpoint 6

- The control gallery exercises three ownership altitudes without authored menu
  recipes: a contextual checkbox, a focused text service, and a virtual table.
  Table rows capture one typed `OpenRecord` command from their stable key;
  generated Boolean cells opt in independently and therefore win as the nearer
  contextual owner.
- `Table::context_rows<C>` and `TypedColumn::context_menu` keep construction
  typed. The heterogeneous row carrier is private and erased only after the
  command arguments have been captured. A separate context-only node binding
  prevents row context from accidentally becoming primary-click behavior.
- Live-state and lifecycle witnesses prove that an open menu re-resolves
  disabled state, a disappearing virtual row prunes the shared session, and a
  marked cell shadows its marked row without either falling through globally.
  Secondary release, Shift+F10, and the Menu key all enter that same session.
- Surface policy is explicit: the palette excludes its own global describer;
  nearest-owner context discovery has no blanket describer exclusion, so a
  describer deliberately registered in that bounded local world remains an
  honest candidate.
- Four structural-absence tests pin the negative architecture: one private
  resolver, type-distinct global/local providers, no registry import in layout,
  no context row painter or second session, no native platform menu, and one
  placement request consumed by both hosts.
- Public API flags for morning review: `TypedColumn::context_menu` and
  `Table::context_rows<C>` are the only table conveniences added by closeout;
  the general widget-level `context_menu` marker was established earlier in
  the campaign. No erased trigger or target became public.
- Verification: formatting and all-target compilation pass; 982 ordinary
  library tests pass with 10 ignored; all four doctests pass; the text editor,
  control gallery, and glass tuner smoke binaries pass; comparison mode remains
  open. Nine of ten explicit deep-tier witnesses pass, including every GPU,
  alpha, shader, and text-correctness witness. The pre-existing 8 MiB text-load
  performance gate exceeded its threshold twice (272.7 ms and 266.2 ms); no
  context-menu code participates in that path, so the result is recorded rather
  than hidden or repaired out of scope.
- Pending eyes: native placement, hover, fade, and activation should still be
  compared manually under Vulkan and DX12. Mechanical closeout does not claim
  those human-visible observations.

## Structural-absence witnesses

Closeout fails if the tree contains any of the following:

- a second command resolver beside the generalized palette resolver;
- a context surface constructible with the global provider type;
- a second menu session or outside-click dismissal path;
- layout importing the command registry or application model;
- a context-specific row painter or measurement path;
- a public erased trigger/target API;
- stored menu availability;
- parallel in-frame/native placement arithmetic;
- a platform-native menu;
- an unproven merge of the two `AnyTarget` representations.

## Final doctrine

A command surface has three parts: candidate discovery, command resolution,
and presentation. The palette discovers globally and presents search; a
context menu discovers at the nearest declared object and presents menu rows.
Target registration and bindings supply membership, the command system supplies
live meaning, and layout supplies geometry. Typed meanings are erased once at
the established boundary, then reused everywhere.
