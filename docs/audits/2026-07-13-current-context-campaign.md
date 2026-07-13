# Current Context — one path, one realization, one generation

Status: complete. `comparison_open: true`. No push during the campaign.

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
| 1. One popup realization | Complete | One resolved host geometry consumed by paint, input, IME, accessibility, material, and context anchoring |
| 2. Current popup generations | Complete | Content/geometry updates are atomic; a generation-bound WinRT commit receipt now gates cold entrance exposure, confirmed by the one-second reduction and restored 100 ms pass |
| 3. Directional responder traversal | Complete | `Task` and `Inspection` explicitly select claim and section order without changing authored menu bars |
| 4. Table selection context domain | Complete | Existing bounded keyed multiselect owns canonical `SelectAll`; focal remains distinct |
| 5. Automatic facets and grouped context | Complete | Semantic path derives table/member/facet groups, pins dematerialized focal rows, and dismisses removed subjects |
| 6. Gallery and closeout | Complete | Comparison, doctrine, deletions, complete ritual, and pending-eyes closure |

## Checkpoint 0 — census cells

### Named reductions

| ID | Reduction | Evidence required | State |
|---|---|---|---|
| R-01 | Native visual placement flips against monitor bounds while retained hit geometry keeps the in-frame resolution. | `native_popup_frames_are_interactive_only_on_their_realized_surface` plus live edge-placement acceptance | Closed |
| R-02 | The visible menu and its abandoned compensating location can both hover. | Four consecutive parent/popup-surface hit probes and live hover acceptance | Closed |
| R-03 | DX12 command-palette result-height changes intermittently expose a smaller/stale frame. | Atomic content/geometry state witnesses plus live DX12 update acceptance | Closed |
| R-04 | Context A to context B briefly displays A's layout in B's host. | `contextual_retarget_reuses_the_authored_menu_lifecycle` plus live retarget acceptance | Closed |
| R-05 | Menu-bar File to Edit retarget must not expose prior content. | Shared authored/context retirement lifecycle and live menu-bar acceptance | Closed |
| R-06 | Menu-bar to context-menu switching must not expose the prior surface. | Menu/palette exclusivity, fresh contextual identity, and live cross-surface acceptance | Closed |
| R-07 | Rapid A to B to A must reject both earlier generations' receipts. | `popup_receipts_are_bound_to_their_exact_generation` and prepared-entrance receipt witnesses | Closed |

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

## Checkpoint 1 — one popup realization

Native surface ownership is now a constituent of `Layout` birth rather than a
late annotation. Consumers cannot observe a layout whose popup frames have not
yet been assigned to `InFrame` or `Native`, and consecutive-frame witnesses pin
that ownership. The native host resolves placement and visual reach into one
`popup::Realization`; paint translation, popup-local hit testing, pointer-event
translation, IME routing, material projection, clipping, and retained popup
identity read that same value.

The field reduction exposed one additional consumer: the composition HWND's
non-client hit test. Visual bounds include the framework-declared shadow, but
interactive bounds remain the panel. `WM_NCHITTEST` now derives its client hit
rectangle from the current realization and returns `HTTRANSPARENT` in visual-
reach-only margins. This removed the menu-bar hover dead zone without shrinking
or duplicating the painted shadow. The user confirmed menu-bar retargeting was
restored.

## Checkpoint 2 — current popup generations

The first implementation incorrectly borrowed the parent window's presentation
epoch as popup content identity. Parent fades, hover frames, and palette edits
therefore manufactured popup staleness and repeatedly re-entered concealment.
That clock was deleted from popup identity. A popup-local content serial now
advances only when its source scene or captured responder-path fingerprint
changes; parent activity produces zero popup exposure work.

Content revision and concealment are separate axes. Same-geometry content
updates receive a fresh generation and one atomic swapchain present without
cloaking. Resolved native geometry is the sole geometry-change detector: while
a replacement extent or position is prepared, the last complete realization
remains visible; the new swapchain extent is rendered first, then the HWND and
its hit rectangle commit before the compositor-pickup barrier promotes the
pending realization. A missed acquire retries the same generation, and a
same-sized move is covered just as a resize is. Material and scale changes keep
the concealed gate because one present cannot make those contracts atomic.
Field testing confirmed that palette result-list changes no longer blink.

The remaining first-entry-only blink had two owners. A slow initial material
receipt could outlive the overlay's nominal entrance duration, so logical
`Stable` updates once overwrote the concealed compositor root's prepared opacity
with `1.0`. After that overwrite was removed, the one-second reduction exposed
the deeper transition: a warm host's prior root opacity could be sampled after
uncloak before the queued `SetOpacity(0.001)` reached a WinRT Composition commit
cycle. The old opacity briefly faded out, then the intended entrance faded in.

A prepared entrance now owns root opacity until exposure **and** carries a
`RequestCommitAsync` receipt bound to the popup generation. Exposure continues
requesting work while that receipt is pending; only its exact generation may
start the entrance. Stale receipts from an earlier warm-host tenant are inert.
The user confirmed a clean monotonic fade with the diagnostic duration extended
to one second; the theme was restored to 100 ms and the production trace showed
the prepared-root receipt completing before exposure. Pure state-machine tests
pin current/stale receipt behavior, the stability test pins prepared-opacity
ownership, and an architecture witness keeps the receipt connected to the
exposure gate.

## Checkpoints 3–5 — semantic context

`responder::Path` captures command-owning semantic layers independently from
the descriptive `subject::Path`. `responder::Traversal::{Task, Inspection}`
selects direction explicitly; the same ordered walk drives claim consumption,
menu section order, targeting, and separators. Disabled ownership consumes;
absence falls through. Palette/task routing remains exact-to-broad, resting
context inspection is broad-to-exact, and an active editor naturally restores
task traversal.

Tables reuse their existing keyed `Selection` and `Membership::AllExcept` for
multiselect and canonical `SelectAll`; no selection representation was added.
The table owns the broad command, the focal row remains distinct from the
selected member set and selection anchor, and exact text or Boolean facets add
only commands not consumed by broader layers. Right-click selection semantics,
provider-key reconciliation, focal pinning across dematerialization, dismissal
on provider deletion, standard text/Boolean/control participation, and the
removal of redundant per-column context wiring are pinned by focused tests.

## Checkpoint 6 — gallery and closeout

### Behavior matrix

The control gallery now exercises the complete composition rather than a
special context-menu scene. At rest, a text cell is inspected table to row to
facet: the table's canonical Select All appears once, the focal row action
follows, and unconsumed text commands form the exact section. A Boolean cell
uses the same first two sections and contributes its toggle at the facet. Empty
table space contributes only the table domain. Secondary-clicking a selected
row preserves the multiselection; clicking an unselected row makes it the sole
selection. Shift+arrow range extension and contraction continue using stable
provider keys.

An active editor changes the question rather than adding an exception. Its
keyboard focus declares the task frame, so Task traversal starts at the editor,
Select All targets the draft, the focal row remains a broader section, and the
table selection is unchanged. Input-source edit commit bindings are not
mistaken for contextual control actions; ordinary button and Boolean bindings
remain automatic participants. Explicit widget context remains an escape hatch,
not the default path.

Dematerialization and deletion stay distinct. A contextual virtual row retains
one bounded focal pin while scrolled away; provider deletion dismisses the
session and invalidates late receipts. Authored menu-bar panels and contextual
panels consume the same overlay retirement, z-order, entrance, and exit
lifecycle. A retiring panel keeps paint geometry for its fade but has an empty
hit region, while its entering replacement is the only interactive surface.

The existing semantic roles, stable row/cell identities, selected-row state,
and selectable table model remain the accessibility seam. Platform AccessKit
export is roadmap item 11 and was not smuggled into this campaign; the original
acceptance wording is narrowed to preservation of that seam rather than a claim
of a platform integration that does not yet exist.

### Deletion and API census

- The nearest marked-owner climb and `context_owner_for_node` were replaced by
  one captured responder path.
- Parallel reversal/rank arithmetic was deleted; `Path::ordinals` now owns both
  claim precedence and section order.
- `TypedColumn::context_menu` and the gallery's redundant text/checkbox wrappers
  were deleted; ordinary semantic participation derives from existing cells.
- Parent-window presentation epochs no longer stand in for popup content
  identity. Popup-local content serials and exact generation receipts own it.
- Popup pooling retains `PopupHost` infrastructure, never a semantic
  `PopupWindow` session. Retiring sessions are visual-only.
- Placement intent, host geometry, panel hit geometry, popup event translation,
  and non-client hit testing no longer re-resolve independent outcomes.
- Automatic control discovery excludes text-edit `Source::Input` bindings;
  explicit context bindings and ordinary button-source actions remain honest
  candidates.

The only public table API deletion is `TypedColumn::context_menu`, superseded by
automatic participation. `context_rows` remains the explicit, typed seam for
an application row action. `popup::Realization`, `popup::Generation`,
`responder::Path`, traversal, and table context services remain internal
vocabulary; no parallel public responder, selection, or popup API was added.

### Verification and pending eyes

- `cargo fmt --all -- --check` passed.
- `cargo check --all-targets` passed without warnings.
- `cargo test --lib` passed: 1,002 tests, with 10 deliberate hardware-tier
  ignores.
- `cargo test --doc` passed all four doctests, including three compile-fail API
  witnesses.
- The `text_editor`, `control_gallery`, and `glass_tuner` smoke binaries passed.
- `cargo test --release --lib -- --ignored` passed all 10 deep-tier witnesses,
  including premultiplied alpha, sRGB packing, silhouette compilation, and the
  alpha-preserving material-noise regression.
- `comparison_open: true`, the accepted 500px gallery table, and the final 0.40
  panel tint remain intact.

Live Windows acceptance supplied the screen-output half of the native contract:
edge-placed menus paint and hit at one location, menu-bar hover retargets,
context menus remain clickable, each menu has its own monotonic entrance/exit,
palette result changes do not blink, cold entrance no longer flashes, and
context-to-context replacement follows the same authored-menu choreography.
The deterministic event/state witnesses supply the complementary ordering
evidence, including four consecutive steady frames, exact generation receipts,
same-size movement, scale/material concealment, and visual-only retirement.

### Commit accounting

The code did not land as five independently green checkpoint commits. Hardware
reductions repeatedly crossed the protected native-popup ancestry, and exact
hunk separation was no longer reconstructable without inventing history.
Checkpoint 0 remains independently committed; checkpoints 1–5 and the already
accepted popup/material ancestry were consolidated at `c3bb7673`. This is a
recorded protocol deviation, not retroactive checkpoint theater. The boundary
was formatted, all-target compiled, fully tested, and deep-tier green before
commit. No push occurred.

## Exclusions

- No OS-native `TrackPopupMenu` replacement.
- No renderer retained/damage-layer work.
- No command-palette UI redesign.
- No generic ancestor aggregation or implicit application-root context.
- No new table-selection representation or untyped row-command envelope.
- No submenu system without a witnessed caller.
- No header/column context taxonomy beyond preservation of existing behavior.
