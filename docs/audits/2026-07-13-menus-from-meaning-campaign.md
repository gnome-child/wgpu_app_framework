# Menus From Meaning — one population owner

Status: in flight. `comparison_open: true`. No push during the campaign.

## Mission

Make command registration the source of conventional menu vocabulary and put
automatic command population behind one internal owner. An application places
a conventional bar with `ui.standard_menu_bar()`; the framework derives its
categories, sections, rows, separators, state, and targets from registered
meaning. Fully authored `ui.menu_bar(...)` remains supported.

The campaign consumes the two independent investigations:

- `2026-07-13-conventional-command-topology-investigation.md`
- `2026-07-13-conventional-command-topology-investigation-fable.md`

## Constitution

- `command::Standard` names conventional meaning. Label, chord, shortcut
  visibility, category, section, slot, and platform relocation are projections.
- Registration determines persistent bar vocabulary; Task traversal determines
  current state and actor. Focus never reorders cultural topology.
- One population domain owns discovery, erased triggers, registry lookup,
  responder resolution, state/default composition, and policy application.
- Surface policies remain deliberately different: the bar retains registered
  unclaimed entries disabled; context consumes captured-path claims; the palette
  keeps enabled captured-task claims and provenance/relevance ordering.
- Invocation freshness is policy-owned. Bar activation resolves the live Task
  chain; context activation keeps captured-path routes and focal semantics;
  palette behavior remains unchanged.
- Automatic population terminates in ordinary `MenuBar`, `Menu`, `Separator`,
  and `Binding` nodes. There is no parallel layout, focus, paint, or popup path.
- Derivation is opt-in. Registration alone never creates a bar.
- Public code declares meaning and deviations. Authored menus remain the escape
  hatch for dynamic, argument-bearing, replaced, or deliberately unusual UI.

Binding case law remains explicit: Undo and Copy can both resolve at
`Kind::Focused` while culture requires distinct History and Clipboard sections.
Scope does not contain topology.

## Protocol

- Census and reduction precede each mechanism.
- Each checkpoint lands independently green.
- No push occurs mid-campaign.
- Public or architectural boundaries run formatting, all-target compilation,
  library and doctest suites, three application smokes, and comparison
  protection in proportion to the change.
- `comparison_open: true` and unrelated work remain untouched.
- Checkpoint 2 has an honorable retreat: if a shared orchestration path cannot
  preserve sealed context/palette behavior with checkpoint-0 evidence, retain
  their policy adapters, share the safe primitives, record the non-merge, and
  deliver the bar without weakening captured semantics.

## Checkpoint board

| Checkpoint | State | Acceptance boundary |
| --- | --- | --- |
| 0. Freeze sealed policies | Complete | Authored bar, context capture/traversal, palette membership/order, chords, registry order, same-scope case, ordinary nodes, and opt-in behavior pinned |
| 1. Promote standard meaning | Complete | Role lives on `Spec`; Delete admitted; CloseWindow corrected; defaults/overrides and uniqueness proven; authored menus unchanged |
| 2. One population owner | Complete | Shared primitives/policies with live bar targeting and captured context targeting; no observable context/palette drift |
| 3. Cultural topology | Complete | `command::menu::{Category, Placement}`, virtual slots, platform reuse, custom categories, shortcut visibility, pure topology witnesses |
| 4. Automatic bar | Complete | `ui.standard_menu_bar()` emits ordinary nodes with stable disabled membership and no ambient UI |
| 5. Authored deviations | Complete | Static metadata and typed dynamic/replacement extensions coexist with unchanged authored bars |
| 6. Migration and closeout | In progress | Examples delete duplicated culture; full witness matrix and ritual green; item 30 pruned |

## Checkpoint 0 — census and behavioral pins

### Existing ownership

| Cell | Current truth | Campaign consequence |
| --- | --- | --- |
| P-01 | `Candidates<Global>` and `Candidates<Local>` already share typed policy machinery over erased `Candidate`s. | Extend the established pattern; do not create an unrelated menu registry. |
| P-02 | `ResolvedAction` contains a mandatory `responder::Claim`. | Do not weaken it for registered/unclaimed bar entries; add a distinct bar projection. |
| P-03 | `state_any` distinguishes unregistered hidden, registered/unclaimed disabled, and claimed state. | Reuse it for stable bar membership. |
| P-04 | Context captures `responder::Path`, selects Inspection or active-editor Task, and builds local candidates with captured routes. | Population refactoring must preserve capture and focal semantics. |
| P-05 | Palette captures a Task scope, uses global candidates, drops disabled entries, and sorts provenance then relevance then registration. | Population refactoring must preserve all three policy axes. |
| P-06 | Authored menu rows are ordinary menu-dressed bindings; layout, focus, paint, and popup code are role-generic. | Derived bars must emit the same node species. |
| P-07 | `Registry::order` is deterministic first-registration order but not cultural order. | Topology must come solely from platform data and typed placements. |
| P-08 | `KeyChordKind::Standard` currently stores role meaning under the chord projection. | Checkpoint 1 inverts ownership without deleting the useful chord representation prematurely. |
| P-09 | Private `widget::binding::Placement` means Button-versus-Menu dress. | Rename it `Form` before public command-menu placement lands. |
| P-10 | `keymap::Platform` already owns Windows/macOS/Linux identity. | Topology consumes or deliberately promotes it; no parallel platform enum. |

### Required baseline witnesses

- Authored menu-bar node order and keyboard behavior remain explicit.
- Context Inspection ordering, first-claim consumption, active-editor Task
  inversion, focal-row pinning, and captured-path invocation are pinned.
- Palette membership, self-exclusion, captured task, section provenance, and
  provenance/relevance/registration ordering are pinned.
- Every existing Standard chord is pinned on Windows, Linux, and macOS,
  including the two Windows Redo chords.
- Registry enumeration retains first-registration order.
- The real focused text service can claim both Undo and Copy at `Focused`.
- A command-rich view without a bar receives no ambient `MenuBar` node.
- An architecture witness requires derived bars to reuse existing menu roles,
  binding projection, focus/navigation, and popup lifecycle.

### Named future regressions

| ID | Regression forbidden by the campaign |
| --- | --- |
| R-01 | Focus changes reorder conventional rows or separators. |
| R-02 | A registered/unclaimed bar command disappears instead of disabling. |
| R-03 | Context activation re-resolves against a live path and loses its captured focal subject. |
| R-04 | Palette population inherits stable disabled membership or cultural ordering. |
| R-05 | Derived bars introduce a second menu widget, focus path, or popup lifecycle. |
| R-06 | A registered command creates an ambient bar without `ui.standard_menu_bar()`. |
| R-07 | An absent optional standard role moves an authored relative placement. |
| R-08 | Labels such as `"View"` become category identity or merge keys. |

### Baseline validation

- Existing context witnesses already pin Inspection order, first-claim
  consumption, active-editor Task inversion, focal-row capture, and captured
  invocation routes; checkpoint 0 adds the same-scope History/Clipboard
  witness against the real focused text service.
- Existing palette witnesses already pin captured Task resolution,
  self-exclusion, provenance sections, and the
  provenance/relevance/registration sort; checkpoint 0 pins the registry's
  first-registration order independently of that surface policy.
- `authored_menu_bars_keep_explicit_order_while_command_registration_stays_nonvisual`
  pins authored row/separator order and proves registration alone creates no
  ambient bar.
- `every_standard_role_keeps_its_cross_platform_chord_projection` pins the
  complete pre-promotion role set across Windows, Linux, and macOS.
- `cargo test --lib`: 1,006 passed, 10 ignored, zero failures.
- `cargo fmt --all -- --check` and `git diff --check`: clean.

Checkpoint 0 changes documentation and witnesses only. Production population,
role, and topology behavior remains untouched at this boundary.

## Checkpoint 1 — standard meaning owns its projections

- `Spec::standard(Standard)` is the primary declaration; `Spec::new(label).role(Standard)`
  preserves an explicit label while deriving the role chord, and either chord
  builder remains an explicit override.
- `Standard::Delete` is admitted from the existing document command and maps to
  the platform Delete key. Its menu shortcut-visibility decision remains a
  topology projection for checkpoint 3.
- Editing, timeline, session, and text-editor file registrations now derive
  conventional labels and chords from their role. `CloseWindow` therefore
  projects “Close Window”, never “Exit”.
- Registry installation rejects a second command type claiming an occupied
  standard role before replacing either registration. Re-registering the same
  command type may replace or release its role without moving discovery order.
- `KeyChordKind::Standard` remains the chord projection; public
  `KeyChord::standard` remains available for authored and migration cases.
- API flags: `Spec::standard`, `Spec::role`, `Spec::standard_role`, and
  `Standard::Delete` are new public vocabulary. Duplicate roles are treated as
  deterministic configuration errors at registration time.
- Validation: 1,010 library tests passed, 10 ignored; all targets compile;
  doctests, formatting, and diff checks pass. The checkpoint-0 authored/no-bar
  witness remains green, so no automatic bar exists yet.

## Checkpoint 2 — one population owner, distinct policies

- `command::population` now owns candidate discovery, typed policy markers,
  erased triggers, registry metadata lookup, claim resolution, and bar-state
  composition. The prior generic `command::surface` module and Registry-owned
  discovery/resolution loops are deleted.
- `Palette`, `Context`, and `Bar` are explicit policy types over the same
  `Candidates<P>` machinery. Runtime palette and context callers are adapters
  into that owner while retaining their surface-local filtering and ordering.
- Context and palette still receive `ResolvedAction` with a mandatory
  `responder::Claim`; the field was not weakened to `Option`.
- Bar resolution has a distinct claim-free projection. Its named witness proves
  a registered-but-unclaimed role remains disabled, then becomes enabled when
  the live responder chain changes. No claim or captured target is retained.
- Context continues supplying captured routes and its focal path; palette keeps
  its captured Task chain. The complete checkpoint-0 policy witness family is
  green without expectation changes.
- A narrowly scoped temporary dead-code allowance marks the bar projection
  between this internal boundary and checkpoint 4's first UI consumer; it must
  disappear when `standard_menu_bar` lands.
- Validation: 1,011 library tests passed, 10 ignored, zero failures; warning-free
  library checking and formatting pass.

## Checkpoint 3 — cultural topology is pure data

- `command::menu::{Category, Placement}` is the public vocabulary. Category
  identity is a standard constant or application marker type; visible labels
  are metadata and are never identity, merge keys, or ordering keys.
- Custom categories register identity, label, and a typed standard-category
  anchor once through `Registry::menu_category`. Their default band is between
  View and Tools. No public rank or string category key exists.
- `Spec::placement`, `Spec::unplaced`, and `Spec::show_menu_shortcut` are
  independent of role, label, and chord. A standard role uses its cultural
  slot unless explicitly moved or suppressed; Command Palette has no default
  slot.
- Windows/initial-Linux and macOS templates are data over the existing
  `keymap::Platform`. Windows groups File as Document then Persistence and Edit
  as History, Clipboard, Selection, Deletion. macOS groups File as New/Open
  then Save Item and Edit as History then Pasteboard.
- Standard slots remain in the resolver when their command is absent, so
  before/after and section-before/after anchors do not drift. Empty sections
  and categories are removed only after projection.
- Delete uses the shared shortcut icon-run path and the Phosphor delete-key
  glyph on every platform; `⌦` is only its text fallback. The menu topology
  does not hide or reformat it. Spec metadata can explicitly suppress
  visibility without changing the chord.
- Static placement on a non-unit command is rejected at registration. Missing
  custom categories, conflicting category declarations, and anchors with no
  virtual slot are rejected before exposure.
- Pure witnesses pin both platform templates, shuffled registration order,
  virtual absent anchors, custom category order and identity, equal-label
  non-merge, shortcut visibility, and the configuration errors above. The
  private widget `Placement` was renamed `Form`, leaving placement one meaning
  per namespace.
- API flags: the namespaced vocabulary, registry category declaration, and
  three Spec builders are new. Internal topology currently carries a scoped
  checkpoint-4 dead-code allowance until its first UI projection consumes it.
- Validation: 1,020 library tests passed, 10 ignored; all targets compile and
  all doctests pass. Formatting and diff checks are clean.

## Checkpoint 4 — one call derives an ordinary bar

- `ui.standard_menu_bar()` is the sole opt-in request. It enters the authored
  view as an ordinary `MenuBar` placeholder; after authored command resolution,
  the runtime replaces only that placeholder's children with ordinary `Menu`,
  `Separator`, and menu-dressed `Binding` nodes. No role, popup, layout, focus,
  paint, or activation species was added.
- `command::Population::standard_bar` is the join owner for registered
  vocabulary, cultural topology, current Task-chain state, and shortcut
  visibility. View projection only translates that complete result into the
  existing widget grammar.
- Derived bindings reuse the established erased trigger. Their projected state
  is registered-stable (`unclaimed = disabled`), while activation routes through
  the current menu focus and live responder chain. No claim or target survives
  the projection.
- Cultural sections insert separators only between nonempty projected groups.
  Standard and typed custom category identities supply retained menu ids;
  visible labels remain presentation metadata and never become identity.
- Registration still creates no ambient bar, authored `ui.menu_bar(...)`
  remains byte-for-byte in control of its children, and Command Palette remains
  absent because it has no conventional slot.
- The scoped checkpoint-2/3 dead-code allowances are deleted. The topology and
  bar policy now have their first production consumer.
- API flag: `Ui::standard_menu_bar` is the single new public convenience at
  this boundary.
- Validation: 1,021 library tests passed, 10 deep-tier tests ignored; all
  doctests, all-target checking, formatting, and diff checks pass. The named
  integration witness proves ordinary node roles, topology-owned separators,
  enabled/disabled stable membership, the unplaced role exclusion, and live
  menu-source activation.

## Checkpoint 5 — authorship declares only deviations

- `ui.standard_menu_bar_with(...)` and `widget::StandardMenuBar` add a narrow
  authored layer over the same derived projection. Static commands still use
  `Spec::placement`/`unplaced`; the builder is reserved for runtime nodes.
- Typed operations cover item insertion before/after a standard role, a new
  section before/after a standard group, explicit group replacement, section
  append in a typed category, and explicit category replacement. Their payload
  is ordinary authored `Ui`, so argument-bearing bindings, recent-file/window
  lists, and dynamic submenus retain the existing widget and activation paths.
- The population owner carries a private blueprint with virtual standard
  markers through mixed composition. Those markers never become nodes, but
  keep a dynamic extension anchored to an absent optional role or empty
  standard section from drifting.
- Registered custom categories contribute identity, label, and cultural
  position even when their contents are entirely dynamic. An unregistered
  custom category and a standard role without a cultural slot fail before
  exposure; labels never participate in lookup or merging.
- Extension bindings resolve on the same live Task chain as the derived bar,
  while their authored nodes remain ordinary menu bindings. Fully authored,
  fully derived, and mixed bars therefore coexist without changing the sealed
  authored API.
- API flags: `Ui::standard_menu_bar_with`, public `StandardMenuBar`, and its
  typed extension methods are new. Replacement is named explicitly; there is
  no implicit label-based replacement.
- Validation: 1,022 library tests passed, 10 deep-tier tests ignored; all
  targets, doctests, formatting, and diff checks pass. The mixed integration
  witness covers an argument-bearing recent-file submenu, an absent virtual
  anchor, explicit History replacement, standard-category extension,
  dynamic-only registered custom category placement, category replacement,
  and ordinary live activation.

## Platform contract

- Windows/initial Linux File: Document(New, Open, Close Window), then
  Persistence(Save, Save As).
- Windows/initial Linux Edit: History(Undo, Redo), Clipboard(Cut, Copy, Paste),
  Selection(Select All), Deletion(Delete).
- macOS File: New/Open, then Save Item(Close Window, Save, Save As).
- macOS Edit: Undo/Redo, then Pasteboard(Cut, Copy, Paste, Delete, Select All).
- Command Palette is standard but unplaced. Close Window is never Exit or Quit.
- About, Settings, Quit, Print, Find, and Help remain watch roles without callers.

## Required deletion account

- 19 hand-authored conventional binding rows.
- Six hand-authored conventional separators.
- Four literal standard file chord declarations.
- Repeated conventional registration labels replaced by role defaults.
- Registration-order dependence for conventional topology.
- Population duplication actually superseded by checkpoint 2, subject to its
  evidence-gated honorable retreat.

No deletion is credited until its replacement is explicit and witnessed.
