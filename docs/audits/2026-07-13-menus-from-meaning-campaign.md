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
| 1. Promote standard meaning | In progress | Role lives on `Spec`; Delete admitted; CloseWindow corrected; defaults/overrides and uniqueness proven; authored menus unchanged |
| 2. One population owner | Pending | Shared primitives/policies with live bar targeting and captured context targeting; no observable context/palette drift |
| 3. Cultural topology | Pending | `command::menu::{Category, Placement}`, virtual slots, platform reuse, custom categories, shortcut visibility, pure topology witnesses |
| 4. Automatic bar | Pending | `ui.standard_menu_bar()` emits ordinary nodes with stable disabled membership and no ambient UI |
| 5. Authored deviations | Pending | Static metadata and typed dynamic/replacement extensions coexist with unchanged authored bars |
| 6. Migration and closeout | Pending | Examples delete duplicated culture; full witness matrix and ritual green; item 30 pruned |

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
