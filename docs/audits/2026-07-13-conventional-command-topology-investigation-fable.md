# Conventional Command Topology — independent investigation (Fable 5)

Status: complete. Investigation only; no production code changed.

This is the independent parallel run of the conventional-command-topology
investigation, executed alongside Codex's run for comparison (the
material-regions bake-off pattern). Same mission, same constitution, same
census cells; findings gathered independently. Codex's ledger:
`2026-07-13-conventional-command-topology-investigation.md`.

## Mission verdict, first

**GO, with the promotion.** `command::Standard` should be promoted from a
shortcut subtype into the framework's canonical vocabulary for conventional
command meaning. The receipts below show the current arrangement is not
merely narrow but *inverted* (the role is a sub-species of the chord) and
*fractured* (half the standard roles in production use the role, half
re-declare the same meanings as literal strings). The derived menu bar is
buildable almost entirely from machinery that already exists; the genuinely
new object is one data table and one small placement vocabulary.

## Phase A — current-truth census

### A-01 — `Standard` residents and callsites

Twelve variants at [spec.rs:9-22](../../src/command/spec.rs): Undo, Redo,
Cut, Copy, Paste, SelectAll, New, Open, Save, SaveAs, CloseWindow,
CommandPalette.

Production callsites of `KeyChord::standard`:

| Role | Callsite |
|---|---|
| Cut / Copy / Paste / SelectAll | `document/command.rs:91,95,99,104` (`Editing::standard()`) |
| Undo / Redo | `timeline/command.rs:68,72` |
| CloseWindow / CommandPalette | `session/service.rs:98,102` |
| **New / Open / Save / SaveAs** | **none** |

**The fracture receipt:** the four file roles exist in the enum and in the
keymap table but have zero production users. The text editor registers the
same four conventional meanings as literal chord strings and hand-authored
labels at `examples/text_editor/app/runtime.rs:17-22`
(`Spec::new("New").shortcut("Primary+N")`, `"Primary+O"`, `"Primary+S"`,
`"Primary+Shift+S"`). The role vocabulary is already split down the middle
of the codebase: identical kinds of meaning expressed two different ways.
This is the disease the promotion cures, censused precisely.

### A-02 — current platform projection

`KeyChordKind::Standard` ([spec.rs:24-28](../../src/command/spec.rs)) makes
the role a *variant of the chord kind* — the meaning hierarchy inverted.
`Profile::standard_chords` ([keymap.rs:197-234](../../src/keymap.rs)) is
already a role → platform projection table, chords only:

- CloseWindow: Cmd+W (Mac) vs **Alt+F4** (Windows/Linux).
- Redo: Ctrl+Y **and** Ctrl+Shift+Z (Windows/Linux, two chords) vs
  Cmd+Shift+Z (Mac).
- All others: uniform primary chords; CommandPalette = Primary+Shift+P.

Display projection also exists per platform (symbol runs ⌘/⌥ on Mac vs
"Ctrl"/"Alt" text, keymap.rs:281-302). **The minimal inversion:** `Spec`
gains `role: Option<Standard>`; label, chord, and placement derive from the
role; `KeyChordKind::Standard` becomes an internal projection detail and its
public constructor is deletable once the eight production callsites migrate.

### A-03 — where role and placement truth can live

`Spec` = `display_name` + `shortcut` + `listing`
([spec.rs:121-126](../../src/command/spec.rs)). `AnyCommand` stores the
whole `Spec` ([registry.rs:24-31](../../src/command/registry.rs)), so **new
Spec fields flow through type erasure with zero changes**. Nothing needs to
enter the `Command` trait — confirmed viable exactly as the constitution
requires. `Listing::{Included, Describer}` is a self-reference policy
(palette does not list its own describer), not a placement concept; no
merge.

### A-04 — registry mechanics

- Enumeration: `order: Vec<TypeId>` (registry.rs:21) — deterministic but
  **registration-ordered**, which the constitution correctly bans as a
  topology source. `registration_index` flows into candidates
  (registry.rs:166-171). The derived bar must order from the placement
  table alone.
- Unit-argument restriction is enforced twice, structurally:
  `accepts_shortcut_args = args_type == TypeId::of::<()>()`
  (registry.rs:50-52) gates shortcuts and global candidates; and
  `Binding::<C>::menu()` requires `C: Command<Args = ()>` at the type level
  ([binding.rs:19-30](../../src/widget/binding.rs)). Argument-bearing
  commands *cannot* accidentally become zero-argument menu actions today;
  the derived bar inherits this for free. The authored escape hatch for
  argument-bearing rows already exists: `menu_with_args` (binding.rs:40).
- State resolution: the three-way distinction the stable-membership law
  needs already exists verbatim — `state_any` returns `State::hidden()` for
  unregistered, `State::disabled()` for registered-but-unclaimed, and the
  claim's state otherwise (registry.rs:126-137).
- Duplicate policy precedent: `Error::AmbiguousShortcut` and
  `Error::ShortcutRequiresArgs` (registry.rs:332-351) — the registry
  already *errors deterministically* on ambiguity rather than picking.

### A-05 — which projection the bar reuses

Two resolution projections coexist:

- `resolve_candidates` **drops** unclaimed commands (registry.rs:243:
  `Ok(None) | Err(_) => return None`) — correct for palette and context.
- `state_any` **disables** unclaimed commands (registry.rs:131) — correct
  for the bar.

The derived bar therefore needs registry enumeration + `state_any`-style
resolution, not the palette resolver. `State::with_command`
([state.rs:94-106](../../src/command/state.rs)) already merges spec truth
into presentation with claim overrides winning: claim label overrides spec
display name; claim shortcut overrides spec chord; `checked` flows from the
claim (state.rs:89). Dynamic labels and check state need nothing new.

### A-06 — authored menu inventory (all examples)

Full inventory with per-entry receipts gathered by sweep; totals:

| Metric | text_editor | control_gallery | glass_tuner | TOTAL |
|---|---|---|---|---|
| menu-bar authoring lines | 27 (view.rs:110-136) | 22 (view.rs:59-80) | 0 | **49** |
| in-bar entries | 15 | 12 | 0 | **27** |
| conventional entries | 12 | 7 | 0 | **19** |
| app-specific entries | 3 | 5 | 0 | **8** |
| hand-placed separators | 4 | 2 | 0 | **6** |

- The conventional Edit block (Undo/Redo ⟨sep⟩ Cut/Copy/Paste/Delete ⟨sep⟩
  SelectAll) is **duplicated verbatim** across text_editor (view.rs:122-130)
  and control_gallery (view.rs:65-73), identical separator placement —
  culture transcribed twice by hand.
- No dynamic, no argument-bearing, and no internal entries appear in any
  bar. Internal commands (`ApplyEdit`, `OpenPath`, `SaveToPath`, `SortBy`,
  cell editors) are registered but never bound to menus — the discipline
  the `Listing`/placement split must preserve.
- The only argument-bearing menu-like surface anywhere is the table's
  per-row context wiring (`context_rows::<OpenRecord>`,
  control_gallery view.rs:251) — already outside the bar.
- glass_tuner authors **no menu bar at all** (its two menu-styled rows are
  an acrylic paint probe inside a floating panel, view.rs:248-249). A
  command-rich app with no bar remains valid: the derived bar must be
  opt-in, never ambient.

### A-07 — compression witness, quantified

49 authored topology lines + 6 hand separators + 19 conventional entries
(each with a hand-recalled position) collapse to: **one derived-bar call per
app plus one placement declaration per genuinely app-specific command**
(3 in text_editor, 5 in control_gallery — the 2 Controls-category commands
also declare their custom category once). Four literal chord strings and
~12 re-authored conventional labels in registrations also delete
(A-01/A-02). Net: ~55 lines of transcribed culture become ~10 lines of
declared deviation.

### A-08 — menu machinery reusable by the projector

`ui.menu_bar` / `ui.menu(id, label, children)`
([ui.rs:47-57](../../src/widget/ui.rs)) → `MenuBar`/`Menu` widgets →
`Builder::menu_bar()` / `Builder::menu(id, label)` / typed
`Builder::bound::<C>()` ([builder.rs:22-32](../../src/view/node/builder.rs))
→ ordinary `view::Node` trees with `Role::MenuBar/Menu/Binding`. Menu rows
display `command::State` (label, shortcut display via keymap profile,
checked, enabled). Sessions, retargeting, popup realization, generations,
and keyboard navigation all operate on these nodes — the entire Current
Context campaign's machinery is downstream of this seam and is reused
unchanged. The projector's only job is *emitting the same nodes from data*.

### A-09 — surface separation (regression pins)

Context menus order by `responder::Path` ordinals under `Inspection`
(runtime/context_menu.rs:107). The palette's ordering is now precisely
receipted (`runtime/palette.rs:268-280`): **provenance `sort_key` is the
primary key** (`(order, structural_order, name)`, chain.rs:100-102), fuzzy
relevance score is only a tiebreak *within* equal provenance, and
`registration_index` is the final tiebreak. Palette section headers are
derived per-run from provenance kind + `subject::Path` segment labels
(`section_for`, palette.rs:282-299) — sections are *subject* labels, not
vocabulary labels. The palette additionally **drops** disabled and hidden
commands outright (`.filter(is_enabled)`, palette.rs:229) and drops its own
`Listing::Describer`. Neither surface consumes categories; the bar consumes
no responder ordering for topology. Three surfaces, three projections, one
registry underneath — and the bar's stable-disabled membership is the exact
inverse of the palette's enabled-only membership, which is why they must
never share a resolution projection (A-05).

### A-10 — name census (merges and non-merges)

| Existing concept | Location | Verdict |
|---|---|---|
| **Geometric placement family**: `geometry::PlacementRequest`/`PlacementAnchor`, `view::node::FloatingPlacement`, `Builder::with_menu_placement(anchor, available)` | geometry/placement.rs, node/mod.rs:58, builder.rs:317 | **The load-bearing collision.** "Menu placement" *already means geometry* in this codebase, sealed by Current Context doctrine ("`PlacementRequest` is intent…"). The command-vocabulary concept must not create a second doctrinal meaning of bare "placement" — see the Phase D naming amendment. |
| `widget::binding::Placement` (private: Button/Menu) | binding.rs:13-17 | Second collision, minor: means "presentation form." Rename to `Form` during the campaign; private, so free. |
| `view::node::Role` (also text document `Role`, layout `StructuralRole`) | node/role.rs:2 | **Name taken.** Keeping `command::Standard` as the role vocabulary avoids minting `command::Role` entirely. |
| Palette `section: String` + `section_for` | command_palette.rs:15, palette.rs:282 | Existing *derived* grouping label from provenance + subject path — subject labels, not vocabulary labels. Principled non-merge with categories. |
| `responder::Provenance`/`order`/`Kind` | chain.rs:76-114, kind.rs:14 | **The de-facto grouping/ordering authority today** for palette and claims. C-01/C-02 record why it cannot own bar topology. Non-merge. |
| `command::HistoryGroup` | command/mod.rs:49 | Undo-coalescing identity. Menu groups live inside the new vocabulary, namespaced away. Non-merge. |
| `Candidate::registration_index` | surface.rs:20 | Palette final tiebreaker; must not leak into bar ordering. Non-merge. |
| `command::Listing` | spec.rs:129-135 | Self-reference policy, not placement. Non-merge. |
| `Category` / `Slot` / `Priority` / `Rank` | — | **Free.** `Category` has zero occurrences in src/; `Slot` only as locals; `Priority` only a wgpu present-mode const; no `Rank` type exists. |

## Case law (recorded as binding)

### C-01 — Same scope cannot explain semantic grouping
In a focused editor, Undo (focused text history) and Copy (focused text
transfer service) both claim at `Focused`, yet culture requires
Edit → History and Edit → Clipboard with a rule between them. **Menu group
ordering is cultural data, not scope derivation.** Architecture witness:
History and Clipboard must never merge on matching claim provenance.

### C-02 — Implementation altitude is not semantic scope
Text services carry the current responder kind; the system timeline claims
at `Kind::Framework` only as fallback ([system.rs:79](../../src/runtime/services/system.rs)).
`Kind::Framework` is claim provenance, not a menu category.

### C-03 — Sets are installation bundles, not menu groups
`Set` entries are (type, name, spec, install-fn) with `include`/`without`
composition ([set.rs:5-55](../../src/command/set.rs)); `Editing::standard()`
contains internal `ApplyEdit` alongside Cut/Copy/Paste/Delete/SelectAll,
and the authored Edit menu splits set members across visible sections.
Sets may *install* placement-bearing specs; set boundaries generate no
menu sections. (Corrective note to the earlier design exchange: Undo/Redo
live in `timeline::`, not in `Editing::standard()` — the Edit menu's
knowledge is already split across two sets, which is further evidence that
sets cannot own menu grouping.)

### C-04 — Disabled ownership and stable vocabulary are compatible
The hidden/disabled/resolved triple already exists at registry.rs:126-137.
The bar consumes it; the palette's drop-unclaimed projection stays where it
is (registry.rs:243).

## Phase B — standard-role table

Chords are the existing `Profile` results (keymap.rs:197-234). "Show chord"
follows the Microsoft convention that some standard items don't display
their shortcut. Slots are relative positions within groups, shown here as
ordinals for readability only — the public API never exposes numbers.

| Role | Default label | Win/Linux chord | Mac chord | Category | Group | Slot | Show chord | Relocates (Mac) | Existing embodiment |
|---|---|---|---|---|---|---|---|---|---|
| New | "New" | Ctrl+N | Cmd+N | File | Creation | 1 | yes | no | `document::NewFile` |
| Open | "Open…" | Ctrl+O | Cmd+O | File | Creation | 2 | yes | no | `document::OpenFile` |
| Save | "Save" | Ctrl+S | Cmd+S | File | Persistence | 1 | yes | no | `document::SaveFile` |
| SaveAs | "Save As…" | Ctrl+Shift+S | Cmd+Shift+S | File | Persistence | 2 | yes | no | `document::SaveAsFile` |
| CloseWindow | "Close Window" | Alt+F4 | Cmd+W | File | Window lifecycle (terminal group) | 1 | **no** (Win convention) / yes (Mac) | no (stays in File) | `session::CloseWindow` |
| Undo | "Undo" | Ctrl+Z | Cmd+Z | Edit | History | 1 | yes | no | `timeline::Undo` |
| Redo | "Redo" | Ctrl+Y (+Ctrl+Shift+Z) | Cmd+Shift+Z | Edit | History | 2 | yes (primary chord) | no | `timeline::Redo` |
| Cut | "Cut" | Ctrl+X | Cmd+X | Edit | Clipboard | 1 | yes | no | `document::Cut` |
| Copy | "Copy" | Ctrl+C | Cmd+C | Edit | Clipboard | 2 | yes | no | `document::Copy` |
| Paste | "Paste" | Ctrl+V | Cmd+V | Edit | Clipboard | 3 | yes | no | `document::Paste` |
| SelectAll | "Select All" | Ctrl+A | Cmd+A | Edit | Selection | 1 | yes | no | `document::SelectAll` |
| CommandPalette | "Command Palette…" | Ctrl+Shift+P | Cmd+Shift+P | **none** | — | — | yes | — | `session::OpenCommandPalette` |

Resolved cells:

- **CloseWindow** needs no cross-menu relocation on any platform — File in
  both worlds — but its chord, chord display, and group label differ by
  platform. Note honestly: with Alt+F4 semantics on Windows it behaves as
  Exit in single-window applications; a future distinct **Quit** role is
  the relocating one (Mac app menu, Qt `QuitRole`) and stays on the watch
  list until a multi-window caller forces the split.
- **CommandPalette**: standard chord, **no default placement** — apps may
  place it explicitly (View is common) but the platform templates contain
  no palette slot; the "absent unless explicitly placed" witness holds.
- **Default labels**: the framework can honestly supply English defaults
  today — no localization system exists to contradict them, every current
  registration hand-authors the identical strings, and ellipsis policy
  (Open…, Save As… need more input; New, Save do not) is part of the
  cultural table (Microsoft ellipsis guidance). Label override remains via
  the explicit-label form (Phase D).

**Missing-role watch list** (candidates, not additions):

- **Delete — has a live caller today** (`document::Delete`, registered and
  menu-bound in two examples) and should join `Standard` in the campaign's
  first checkpoint. Placement note: the Microsoft standard puts Delete in
  its own group *after* Select All; **both examples currently place it in
  the Clipboard group** (text_editor view.rs:128, control_gallery
  view.rs:71) — and macOS convention agrees with the examples (Delete rides
  the pasteboard group). So Delete's *group membership itself* is
  platform-template data. This is the strongest single proof in the census
  that group composition belongs to the template, not to the role
  hard-coded: same role, different group per platform. Flag for taste:
  Windows template per Microsoft (own group) vs. examples' current shape
  (clipboard group); either is defensible, pick once, in data.
- Find/FindNext/Replace/GoTo (Edit/Search) — no callers.
- Print/PageSetup (File) — no callers.
- Quit distinct from CloseWindow — no caller until a multi-window app.
- About / Options-Preferences / Help — no Help-menu infrastructure yet;
  these are the Qt-relocation trio (About→AboutRole, Preferences→
  PreferencesRole, Quit→QuitRole) and land with the Mac template's real
  hardware arc.
- FullScreen/Zoom/StatusBar/Toolbars (View) — no callers.

## Phase C — conventional topology research

All three load-bearing sources verified live this session (not recalled):

1. **Apple `CommandGroupPlacement`** (developer.apple.com, fetched via the
   documentation JSON): twenty opaque semantic slots — appInfo,
   appSettings, appTermination, appVisibility, systemServices,
   importExport, newItem, printItem, saveItem, pasteboard, textEditing,
   textFormatting, undoRedo, sidebar, toolbar, help, windowArrangement,
   windowList, windowSize, singleWindowList. Placement slots are invisible
   structural positions; ordering is system-owned. Apple gave up deriving
   and canonized the slots — the strongest precedent for culture-as-data.
2. **Qt `QAction::MenuRole`** (doc.qt.io, fetched): NoRole,
   TextHeuristicRole, ApplicationSpecificRole, AboutQtRole, AboutRole,
   PreferencesRole, QuitRole — "how an action should be moved into the
   application menu on macOS"; immediate menubar menus only. Roles owned by
   actions, relocation owned by the platform layer.
3. **Microsoft menu design guidelines** (learn.microsoft.com, fetched):
   - Standard bar: File · Edit · View · Tools · Help; standard categories
     "for programs that create or view documents."
   - Literal Edit order: Undo/Redo ⟨sep⟩ Cut/Copy/Paste ⟨sep⟩ Select all
     ⟨sep⟩ Delete ⟨sep⟩ Find/Replace/Go to.
   - **The stable-membership law, verbatim:** "Disable menu items that
     don't apply to the current context, instead of removing them. Doing so
     makes menu bar contents stable and easier to find."
   - **The context-menu inverse, verbatim:** "Remove rather than disable
     context menu items that don't apply" (with the standard-commands
     exception) — both halves of our constitution in one primary source.
   - "Don't change menu item names dynamically" with the object-name
     exception (recent files) — the dynamic-label policy.
   - Groups of ≤7, separators between groups, ellipsis rules.
4. GTK/GIO `MenuModel` (cited, not fetched): menus are ordered sections;
   separators are projected between adjacent nonempty sections — the
   separator-derivation model the projector should use.
5. Microsoft `StandardUICommand` (cited, not fetched): standard command
   kinds derive label, icon, description, and accelerator — the same
   promotion, shipped by the platform vendor.

### Windows/Linux template (implementable now)

```text
File(1) · Edit(2) · View(3) · [custom categories] · Tools(4) · Window(5) · Help(6)

File:  Creation(New, Open…) ⟨sep⟩ Persistence(Save, Save As…) ⟨sep⟩
       [Recent — dynamic authored section] ⟨sep⟩ Window lifecycle(Close Window)
Edit:  History(Undo, Redo) ⟨sep⟩ Clipboard(Cut, Copy, Paste) ⟨sep⟩
       Selection(Select All) ⟨sep⟩ Deletion(Delete)†
View:  app toggles (no standard residents yet)
```
† or Clipboard-group per the taste decision recorded in Phase B.

Custom categories insert between View and Tools by default; each may
declare a typed anchor to sit elsewhere.

### macOS template (paper design, no hardware claims)

```text
AppMenu(About, ⟨sep⟩ Settings…, ⟨sep⟩ Services, ⟨sep⟩ Hide/Others/All, ⟨sep⟩ Quit)
· File(New, Open…, ⟨sep⟩ Close Window, Save, Save As…) · Edit(History ⟨sep⟩
Cut, Copy, Paste, Delete, Select All — one pasteboard group) · View ·
[custom] · Window(Minimize, Zoom, ⟨sep⟩ window list) · Help
```

Relocations (About/Settings/Quit → app menu) affect only watch-list roles —
**zero current residents relocate**, so the Mac template is pure data with
no machinery pressure until those roles gain callers.

## Phase D — placement vocabulary

**Recommended shape: one truth, two entrances.**

```rust
// Common case — role supplies label, chord, and placement:
Spec::standard(Standard::Copy)

// Label-override case — app authors the label, role supplies the rest:
Spec::new("Copy Frame").role(Standard::Copy)
```

Both set the same `role: Option<Standard>` field on `Spec`;
`Spec::standard(r)` is sugar for `Spec::new(default_label(r)).role(r)`.
Alternative 3 (separate `key_chord(standard)` + `placement(standard)`)
is the duplication baseline and is rejected: it preserves exactly the
repetition the promotion exists to delete. Alternative 1 alone is rejected
because label overrides must not forfeit the role's chord and placement.

Custom commands:

```rust
Spec::new("Load Stress Text").placement(Placement::after(Standard::SaveAs))
Spec::new("Wrap text").placement(Placement::category(Category::VIEW))
Spec::new("Click").placement(Placement::category(Category::of::<Controls>()))
```

- **Naming amendment (post-census):** bare `command::Placement` is the
  wrong name. The A-10 census shows "menu placement" *already means
  geometry* here — `PlacementRequest`, `FloatingPlacement`, and literally
  `with_menu_placement(anchor, available)` — and "Placement is intent" is a
  sealed doctrine sentence about rectangles. A second doctrinal meaning of
  the bare word invites exactly the meaning-drift this framework prosecutes.
  Two honest resolutions, in preference order:
  1. **`command::menu::{Placement, Category}`** — accept the word but make
     the qualified path mandatory vocabulary (`menu::Placement` is where a
     command lives in the menu vocabulary; `geometry::PlacementRequest`
     stays where a popup lives on screen). One extra module, zero new
     nouns.
  2. A distinct noun if taste prefers it at review time.
  Either way: rename the private `widget::binding::Placement` to `Form`
  (A-10) so the surface-form meaning stops squatting.
- `Category`: **the name is completely free** (zero occurrences in src/) —
  framework consts (FILE/EDIT/VIEW/TOOLS/WINDOW/HELP) plus typed custom
  identity via marker types (`Category::of::<T>()` with a declared label) —
  no stringly `"File"` identity anywhere, matching the house pattern of
  typed command identity.
- Anchors are typed and relative (`after`/`before` a `Standard` role or a
  custom command type); numeric ranks never appear in public API. Anchor
  resolution when the anchor is absent: fall to the anchor's *group*
  terminal position, then the category terminal — deterministic by table
  order, witnessed.
- Duplicate standard role across two registered commands: deterministic
  registration error, precedent `Error::AmbiguousShortcut` (a duplicate
  role would already collide chords today).
- Argument-bearing commands with a `Placement` fail registration the same
  way `ShortcutRequiresArgs` works today — placement implies enumerable.
- Internal commands (`ApplyEdit`, `OpenPath`, …) simply carry no role and
  no placement: absent from the bar by construction, exactly as they are
  absent from authored menus today (A-06).

## Phase E — derived-bar projection (design only)

```text
registry (placement-bearing AnyCommands)
  → platform template (role→category/group/slot data)
  → stable topology (categories, groups, ordered entries)
  → per-open state pass: Task traversal → state_any (claims override)
  → Builder::menu_bar/menu/bound — ordinary view::Node trees
  → existing menu sessions, popups, realization, generations
```

1. **Enumeration**: directly from registry metadata (AnyCommand.spec); a
   thin `Candidates<Bar>` phantom marker if the existing candidate plumbing
   wants a type — no new candidate species with new semantics.
2. **Stable membership**: bar rows resolve through the `state_any` path
   (disabled on no-claim), never `resolve_candidates` (drops). Palette and
   context policies untouched (A-05 receipts).
3. **Empty categories** (no registered members): omitted.
4. **Groups with only disabled members**: present, disabled — stability
   wins (Microsoft guideline, verbatim, Phase C).
5. **Separators**: projected between adjacent nonempty groups (GIO model);
   never authored, never stored.
6. **Duplicate roles**: registration error (Phase D).
7. **Topology rebuilds** on registration change, platform/profile change,
   or extension change — all startup-rare; the bar is a pure projection of
   registry + template, so rebuild is recomputation, and retained
   composition diffs it like any view change.
8. **Without topology rebuild**: enabled/disabled, checked, dynamic labels,
   shortcut display, and invocation targets — all per-open/per-frame claim
   resolution through the existing state pipeline.
9. **Keyboard navigation**: nothing new — the projector emits the same
   `Role::MenuBar/Menu/Binding` nodes the authored path emits.
10. **Architecture witness**: the projector module may reference only
    `Builder::menu_bar/menu/bound` node constructors (source-pattern
    witness), and a behavioral witness asserts a derived bar and the
    equivalent authored bar produce node trees of identical roles — no
    parallel widget species can exist.

## Phase F — mixed authored/derived grammar

One grammar, typed anchors, no label-matching merges ever:

| Case | Form |
|---|---|
| Fully authored bar | `ui.menu_bar(…)` — unchanged, forever valid |
| Fully derived bar | one call: `ui.standard_menu_bar()` (name is campaign's) |
| Custom top-level category | registration-only: commands with `Placement::category(Category::of::<Controls>())` appear automatically |
| Static custom command in a standard category | registration-only: `Placement::category(Category::VIEW)` or `after(Standard::…)` |
| Dynamic section (recent files, window list) | view-side extension at a typed anchor: `ui.standard_menu_bar_with(\|ext\| ext.section(Category::FILE, after(Standard::SaveAs), \|ui\| …))` |
| Replace a category wholesale | `ext.replace(Category::FILE, \|ui\| …)` — authored menu under derived bar |
| Argument-bearing entries | authored only (`menu_with_args` in an authored menu or extension section) — placement cannot invent arguments |
| Recent files / open windows | the dynamic-section case; item labels are object names (the Microsoft dynamic-label exception) |

Verdict on extension semantics: extensions **merge into typed standard
categories at typed anchors**; a custom category is just a placement
target; replacement is explicit and whole-category. Nothing merges by
visible label under any circumstance.

## Phase G — reuse map and deletion forecast

**Reused unchanged** (receipts in Phase A): `Spec` + registration,
`Registry` enumeration/`state_any`/erased invocation, typed registration ×
type-erased resolution, `ResolvedAction`/`State::with_command`,
`Binding::menu` and node builders, menu row presentation (label, shortcut
display, checked), keyboard navigation, authored/contextual popup lifecycle
and everything the Current Context campaign sealed, `keymap::Profile` as
the platform-projection pattern, `responder::Traversal::Task` +
chain-for-path for per-item resolution.

**Deletions the campaign can claim** (each with its explicit replacement):

| Deletion | Replacement |
|---|---|
| 49 authored menu-bar lines + 6 hand separators (A-06) | one derived-bar call per app + ~8 placement declarations |
| 4 literal chord strings for standard roles (A-01) | `Spec::standard`/`role()` |
| ~19 re-authored conventional labels across registrations | role default labels |
| `KeyChordKind::Standard` as public role owner | `Spec.role` field; chord projection internal |
| Registration-order relevance to any menu topology | placement table |
| Duplicated Edit-block culture across examples | the one role table |

**Not deletable yet**: authored `ui.menu_bar` (permanent, it's the
deviation path); `KeyChord::standard` may need a deprecation beat if
anything external uses it (nothing in-repo will after migration).

## Witness drafts for the eventual campaign

- `undo_and_copy_share_focused_scope_yet_occupy_distinct_groups` (C-01,
  the same-scope architecture witness)
- `derived_bar_topology_is_independent_of_claim_provenance_and_registration_order`
- `registered_unclaimed_commands_render_disabled_not_absent`
- `unregistered_commands_are_absent_from_the_derived_bar`
- `history_clipboard_separator_survives_focus_changes` (open bar over
  editor, table, and nothing — same topology all three)
- `standard_chords_resolve_per_platform_profile` (extends existing keymap
  tests; Redo two-chord Windows case pinned)
- `close_window_places_in_file_with_platform_chord_and_display_policy`
- `command_palette_absent_without_explicit_placement`
- `apply_edit_and_internal_commands_never_appear` (pins A-06's discipline)
- `custom_placement_after_standard_anchor_stable_when_anchor_absent`
- `dynamic_authored_sections_coexist_with_derived_groups`
- `authored_menu_bars_remain_behaviorally_unchanged` (regression pin over
  the full A-06 inventory)
- `derived_bar_projects_ordinary_menu_nodes` (architecture witness, E-10)
- `context_menu_and_palette_ordering_unchanged` (regression pins on the
  Current Context and palette-scope behaviors)

## Synthesis

1. **Promotion verdict**: promote. `Standard` is currently upside-down
   (role as chord-subtype) and fractured (four roles unused while their
   meanings are re-declared as strings). The concept already exists; the
   campaign turns it right-side up.
2. **Ownership**: role and placement live on `Spec` (registration
   metadata); never on the `Command` trait; template data owned by the
   platform layer beside `keymap::Profile`.
3. **Twelve-role matrix**: Phase B, complete, with CloseWindow and
   CommandPalette resolved and the Delete group question flagged for taste.
4. **Missing roles**: Delete (immediate, has callers); Find-family, Print,
   Quit, About/Options, View standards (watch list, no callers).
5. **Platform templates**: Phase C; Windows/Linux implementable, macOS
   paper-only; zero current residents relocate.
6. **Stable membership**: registration = existence, claims = state;
   hidden/disabled/resolved triple already in the registry.
7. **Relative placement**: typed anchors only; deterministic absent-anchor
   fallback; no public numbers.
8. **Duplicate roles**: deterministic registration error
   (AmbiguousShortcut precedent).
9. **Mixed grammar**: Phase F — registration-only for static deviation,
   typed view-side extensions for dynamic sections, explicit whole-category
   replacement, label-matching banned.
10. **Reuse map**: Phase G — the projector is the only new machinery; it
    emits existing nodes into the existing popup world.
11. **Deletion forecast**: ~55 lines of transcribed culture → ~10 lines of
    declared deviation across the examples, plus the KeyChordKind
    inversion.
12. **Public API flags**: `Spec::standard` / `Spec::role` /
    `command::menu::Placement` (name amended per A-10 — bare `Placement`
    already means geometry) / `Category` are new public vocabulary
    (morning-review items); `binding::Placement`→`Form` rename is internal.
13. **Non-merges**: A-10 table (Listing, HistoryGroup, view Role,
    responder Kind, registration_index) — all recorded with reasons.
14. **Draft campaign** (each checkpoint independently green):
    - **0. Census pins** — authored-bar behavior pinned over the A-06
      inventory; palette/context ordering pinned; keymap standard-chord
      tests extended.
    - **1. Role promotion** — `Spec.role`, `Spec::standard`, label/chord
      derivation, Delete joins `Standard`, KeyChordKind inversion, all
      eight production callsites + four literal-string sites migrate.
      Deletion: literal chords and duplicate labels.
    - **2. Placement vocabulary + Windows template as data** — `Placement`,
      `Category`, typed anchors, duplicate/arg-bearing registration errors,
      binding::Form rename; pure resolved-topology witnesses (no UI yet).
    - **3. Derived bar projection** — `standard_menu_bar()` emitting
      ordinary nodes; stable membership; derived separators; text_editor
      converts and deletes its File/Edit/View recipes.
    - **4. Mixed composition** — custom categories and anchors
      (control_gallery converts: Controls category + View toggles);
      extension/replacement grammar; dynamic-section seam designed, recent
      files deferred until a real caller.
    - **5. Closeout** — doctrine ("a menu orders by what it is about";
      "cultural data is data"), macOS template recorded as paper data,
      gallery comparison, deletions verified, full ritual.
15. **Go/no-go: GO.** Pure framework work, no hardware dependency, every
    load-bearing mechanism already exists, and the music player — the next
    flagship — is the consuming caller for exactly this bar.

## Verification

- Investigation only: no production code changed; working tree untouched
  except this document.
- All repository receipts verified by direct read this session; the
  examples inventory cross-checked by independent sweep.
- Apple, Qt, and Microsoft citations fetched live and quoted from the
  fetched content; GTK and StandardUICommand cited from documentation
  knowledge and marked as such.
- `comparison_open: true` untouched.
