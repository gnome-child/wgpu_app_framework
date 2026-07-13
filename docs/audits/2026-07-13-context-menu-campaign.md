# The Nearest Command World — context-menu campaign

Status: in flight.

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
| 2 | Retained contextual ownership and exact local discovery | Pending |
| 3 | Secondary/keyboard request through one menu session | Pending |
| 4 | Existing menu-row and overlay projection | Pending |
| 5 | One placement solver with viewport/work-area bounds | Pending |
| 6 | Gallery, absence witnesses, doctrine, and full closeout | Pending |

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
