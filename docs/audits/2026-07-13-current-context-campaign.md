# Current Context — one path, one realization, one generation

Status: in flight.

## Mission

Make contextual commanding and popup presentation one coherent system. A
menu's semantic meaning comes from one directional `responder::Path`; its
visible and interactive geometry comes from one `popup::Realization`; a native
popup becomes visible or changes configuration only as a complete, receipted
`popup::Generation`.

This campaign starts from two field reductions:

1. native popup placement and retained hit geometry can disagree, including a
   second interactive copy at the abandoned compensating position;
2. a visible native popup can expose intermediate geometry or stale contents
   while resizing or retargeting.

It closes by deriving contextual table menus from one semantic path, including
the existing keyed multiselection, canonical `SelectAll`, a focal row, and the
exact text/Boolean/control facet.

## Constitution

- `subject::Path` describes human-facing ancestry; `responder::Path` routes
  through command-owning semantic layers. Strings never become routing
  identity.
- `responder::Traversal::Task` serves the active task from exact to broad;
  `Inspection` examines a containing object from broad to exact. First claim
  consumes in either direction.
- Disabled ownership consumes a command identity; complete absence permits
  fallthrough. Claim order and menu-section order consume the same traversal.
- `PlacementRequest` is intent; `popup::Realization` is fact. The selected host
  resolves placement once, and paint, hit testing, context anchoring, IME,
  accessibility, clipping, and event translation consume that realization.
- A native popup owns one complete current generation and at most one pending
  generation. Birth, resize, retarget, reuse, scale change, and material change
  become visible and interactive only with receipts bound to the complete
  pending generation.
- The existing keyed `Selection`, provider domain, and `AllExcept` membership
  remain the sole table-selection truth. `focal` is distinct from anchor,
  active selection, and the selected member set.
- Captured context follows the framework's dematerialization/removal split.
  Dematerialization pins one focal row for the menu lifetime; provider deletion
  of a captured layer dismisses the menu and makes its receipts inert.

## Protocol and protected ancestry

- Census and reduction precede mechanism choice.
- Each checkpoint lands independently green; no push occurs mid-campaign.
- No parallel responder system, popup geometry model, command registry, or
  selection store.
- No arbitrary delay is accepted as readiness evidence.
- Event-order evidence and screen-output evidence are both required for native
  transition claims.
- Relevant boundaries run formatting, all-target compilation, library and
  doctest suites, three application smokes, comparison protection, the deep GPU
  tier, and Windows hardware witnesses on Vulkan and DX12.

The campaign ignites at `1ef3e28a`, after the deliberately reverted composed
context experiment. The following pre-existing modifications are protected and
must not be silently absorbed, discarded, or attributed to this campaign:

- `docs/audits/2026-07-13-dips-and-receipts.md`
- `docs/audits/2026-07-13-renderer-economics-campaign.md`
- `docs/roadmap.md`
- `examples/control_gallery/app/view.rs`
- `examples/glass_tuner/app/state.rs`
- `examples/material_shadow_probe/main.rs`
- `src/platform/native/{adapter,composition,ime,mod,paint,popup,surface,window}.rs`
- `src/render/filter/draw.rs`
- `src/render/renderer.rs`
- `src/scene/{material,mod}.rs`
- `src/tests/{architecture,layout_scene}.rs`
- `src/theme/toml.rs`

Production files in that set may receive narrowly staged campaign hunks only
after their existing delta has been preserved and censused.

## Checkpoint board

| Checkpoint | State | Acceptance boundary |
|---|---|---|
| 0. Census and named reductions | Complete | Seven transition/placement reductions, dismissal grammar, capture lifetime, and complete ownership census |
| 1. One popup realization | Pending | One resolved host geometry consumed by paint, input, IME, accessibility, material, and context anchoring |
| 2. Current popup generations | Pending | Birth, resize, retarget, reuse, scale, and material transitions expose no stale or hybrid generation |
| 3. Directional responder traversal | Pending | `Task` and `Inspection` explicitly select claim and section order without changing authored menu bars |
| 4. Table selection context domain | Pending | Existing bounded keyed multiselect owns canonical `SelectAll`; focal remains distinct |
| 5. Automatic facets and grouped context | Pending | Semantic path derives table/member/facet groups, pins dematerialized focal rows, and dismisses removed subjects |
| 6. Gallery and closeout | Pending | Comparison, doctrine, deletions, complete ritual, and pending-eyes closure |

## Checkpoint 0 — census cells

### Named reductions

| ID | Reduction | Evidence required | State |
|---|---|---|---|
| R-01 | Native visual placement flips against monitor bounds while retained hit geometry keeps the in-frame resolution. | Deterministic geometry witness plus event-source hit witness | Open |
| R-02 | The visible menu and its abandoned compensating location can both hover. | Parent-surface and popup-surface hit probes for one layer | Open |
| R-03 | DX12 command-palette result-height changes intermittently expose a smaller/stale frame. | Generation/geometry/content trace plus screen capture | Open |
| R-04 | Context A to context B briefly displays A's layout in B's host. | Same-live-id retarget trace plus screen capture | Open |
| R-05 | Menu-bar File to Edit retarget must not expose prior content. | Different-menu-id retarget trace plus screen capture | Open |
| R-06 | Menu-bar to context-menu switching must not expose the prior surface. | Cross-species host trace plus screen capture | Open |
| R-07 | Rapid A to B to A must reject both earlier generations' receipts. | Deterministic delayed-receipt state-machine witness | Open |

### Ownership census

| ID | Question | Initial receipt | Verdict |
|---|---|---|---|
| C-01 | Where is placement resolved? | Layout resolves `PlacementRequest` against `menu_available`; native popup code resolves it again against monitor work area. | Duplicate outcome owners; checkpoint 1 must retain only host resolution as fact. |
| C-02 | What maps popup events? | `popup_point_from_physical` translates popup-local input into parent logical coordinates using `PopupWindow.bounds`. | Popup event source is erased; checkpoint 1 must retain surface identity. |
| C-03 | What owns native visual reach? | `PopupProjection` derives panel bounds, shadow reach, offsets, and surface area, but is recomputed and does not own placement or hit geometry. | Promote into popup realization rather than adding parallel arithmetic. |
| C-04 | What does parent hit testing see? | `Runtime::hit_test` delegates to the complete retained parent `Layout`; native-hosted frames are not filtered by event surface. | Explains the second interactive copy. |
| C-05 | What gates first exposure? | `PopupFirstPresentTrace` and material readiness gate only a newly concealed host. | Birth law exists; change/reuse do not re-enter it. |
| C-06 | What mutates a visible native popup on change? | `configure_popup_window` moves/resizes the HWND before `sync_popup_surface` and renderer presentation. | Explains palette resize and live retarget hybrids. |
| C-07 | How does context discovery work? | The deepest retained node climbs to the nearest marked context owner and captures one responder/focus/binding. | Safe bounded baseline, insufficient for domain/member/facet composition. |
| C-08 | What order does the palette use? | `Kind::rank` hard-codes Captured/Transient/Focused through Framework; provenance uses the same rank for palette sections. | Direction is implicit; promote to named traversal. |
| C-09 | Does `subject::Path` already own routing? | Its contract explicitly says human-facing ancestry and explicitly rejects routing identity. | Principled non-merge: subjects describe, responder paths route. |
| C-10 | Does bounded table selection already exist? | `Membership::AllExcept`, provider-key reconciliation, and `select_all_virtual_rows` are present. | Reuse; checkpoint 4 is integration, not a new selection engine. |
| C-11 | What is a context menu's current identity? | Every contextual menu uses stable id `context_menu`, with owner and anchor stored in origin. | Path fingerprint and monotonic generation must distinguish retargets. |
| C-12 | What is a menu-bar retarget? | Pointer hover replaces one open authored `Menu` with another while the popup system may reuse/reconfigure hosts. | Add all authored/context retarget species to checkpoint 2. |

Baseline validation on the ignition tree:

- `cargo check --lib` passed.
- All 35 `interaction_tests` passed, including nearest-owner context,
  application-scan exclusion, owner removal, authored menu retarget, outside
  click, parent-pointer transfer, activation dismissal, and Escape-before-focus.
- `platform_tests::popup_window_events_map_to_parent_overlay_coordinates`
  passed, characterizing the surface-erasing adapter that checkpoint 1 retires.
- `platform_tests::popup_pointer_motion_without_presentation_does_not_close_native_popups`
  passed, pinning parent/popup pointer-transfer behavior.

### Dismissal and capture grammar

Checkpoint 0 must freeze these inherited behaviors before popup staging or path
capture changes:

- Escape closes the active menu before broader focus cancellation.
- Pointer down outside the menu surface closes it.
- Pointer movement between menu-bar titles retargets the one authored menu
  session.
- Parent pointer leave does not close a native popup merely because the cursor
  entered its child surface.
- Opening a menu closes the command palette; opening the palette closes the
  menu.
- Departed windows purge overlay, popup, menu, pointer, IME, and captured-path
  state.
- Dematerializing a focal virtual row leaves the contextual menu alive, keeps
  exactly one bounded pin, and invocation still targets the focal provider key.
- Provider deletion of any captured command-owning layer dismisses the menu,
  releases the pin, and makes late generation receipts inert.

Existing witnesses pin every inherited dismissal item except menu/palette
session exclusivity as a direct behavioral test. The new
`opening_command_palette_replaces_an_open_menu_session` and
`opening_a_menu_replaces_an_open_command_palette_session` witnesses pin both
directions before checkpoint 2 changes session staging.

The current table context witness treats `state.visible = false` as "virtual
row removal" and expects dismissal. It does not distinguish viewport
dematerialization from provider deletion. Checkpoint 5 must replace that
conflation with paired witnesses modeled on the existing pointer-capture pin:
scroll-away survives through one bounded pin; provider deletion dismisses.

### Naming case law

- `responder::Traversal::{Task, Inspection}` names the question; variant docs
  own the mechanical direction.
- `responder::Path` is distinct from `subject::Path` because routing identity
  and human-facing labels are separate meanings.
- `popup::Generation` is the monotonic serial; typed receipts prove facts bound
  to it; `popup::Realization` is the resolved host outcome.
- `focal` is retained without synonym.

### Checkpoint 0 boundary

- `cargo fmt --all -- --check` passed.
- Both new menu/palette exclusivity witnesses passed.
- The complete interaction-test baseline and the two popup event-transfer
  characterizations passed before production work.
- All seven native defects have named deterministic state or event-order
  reductions. Screen-output captures for R-01 through R-06 remain hardware
  acceptance evidence at their owning checkpoints; R-07 is a pure delayed-
  receipt state-machine witness owned by checkpoint 2.

## Exclusions

- No OS-native `TrackPopupMenu` replacement.
- No renderer retained/damage-layer work.
- No command-palette UI redesign.
- No generic ancestor aggregation or implicit application-root context.
- No new table-selection representation or untyped row-command envelope.
- No submenu system without a witnessed caller.
- No header/column context taxonomy beyond preservation of existing behavior.
