# Conventional Command Topology Investigation — 2026-07-13

Status: **complete; campaign-ready**. Investigation only. No production code
changed. `comparison_open: true` remains unchanged. No push.

## Executive verdict

Promote `command::Standard` from a shortcut-only discriminator into the
canonical vocabulary for conventional command meaning.

The promotion is justified by both repository evidence and platform practice:

- the repository already uses each `Standard` value as meaning when it asks
  `keymap::Profile` for a platform chord, but stores that meaning inside
  `KeyChordKind::Standard` (`src/command/spec.rs:9-22, 25-28, 51-58`);
- conventional labels and separators are consequently repeated by command
  registrations and authored menu bars (`src/document/command.rs:86-105`,
  `src/timeline/command.rs:64-73`, `examples/text_editor/app/view.rs:110-136`,
  `examples/control_gallery/app/view.rs:59-80`);
- Apple exposes standard command-group placements, Microsoft exposes standard
  commands with projected labels/icons/chords, Qt relocates actions by semantic
  menu role, and GIO derives separators from ordered semantic sections;
- runtime responder scope cannot contain menu-group meaning. In the same
  focused text task, Undo and Copy are both claimed through the same focused
  text service at the same `Focused` kind, yet culture requires separate
  History and Clipboard groups.

The recommended factorization is:

```text
registered command type + Spec(Standard role / explicit placement)
        |
        +-- stable topology --> category / section / slot
        |
        +-- platform profile --> label / chord / relocation
        |
        '-- Task traversal ----> current claimant / state / target
                                  |
                                  '--> ordinary menu Binding nodes
```

The bar must not reuse `Registry::resolve_candidates` unchanged. That resolver
correctly drops unclaimed candidates for the palette and context menus, and its
`ResolvedAction` requires a real `Claim` (`src/command/registry.rs:223-261`,
`src/command/surface.rs:31-40`). A conventional menu bar instead needs a new
typed candidate policy that retains registered unit commands, resolves their
current state through `Traversal::Task`, and represents a missing claimant as
a disabled ordinary menu binding.

## Constitution and binding case law

### One meaning, several projections

`command::Standard` names conventional command meaning. A role may project a
default English label, a platform chord, menu participation, category, semantic
section, relative slot, platform relocation, and eventually an icon and
accessibility description. A shortcut is one projection of the role; the role
is not a subtype of shortcut.

Role belongs in registration metadata (`command::Spec`), not the `Command`
trait. Registration is where the application already supplies presentation and
discovery policy; putting role on `Command` would make execution identity own a
surface decision and would prevent an application from deliberately overriding
or suppressing conventional presentation.

### Placement and resolution are independent

Placement asks where people culturally expect to find a command. Task
traversal asks which responder handles it now. Focus changes the latter and
must not reorder the former.

> A menu orders by what it is about. An inspection menu is about an object, so
> `responder::Path` orders its sections. A menu bar is about a vocabulary, so a
> role table orders its groups. Scope determines who acts; role determines
> where people look; the two meet only during resolution.

### C-01 — same scope cannot explain semantic grouping

The focused text service exposes both `document::Copy` and `timeline::Undo`
(`src/runtime/services/text/mod.rs:184-193`) and wraps either claim with the
incoming scope kind (`src/runtime/services/text/mod.rs:69-100`). A focused
scope is `Kind::Focused` (`src/responder/scope.rs:14-22`). Therefore Undo and
Copy can have identical `Focused` provenance at rest.

They still belong to different conventional sections: Edit/History and
Edit/Clipboard. GIO's own section example uses Undo/Redo, a separator, then
Cut/Copy/Paste. Scope does not merely make the order unstable; it lacks the
information necessary to produce the separator at all.

**Verdict:** menu grouping is cultural data, not scope derivation. An eventual
architecture witness must pin this exact same-scope case.

### C-02 — implementation altitude is not menu scope

When focused text does not claim Undo, the system service supplies the broader
timeline fallback and labels that claim `Kind::Framework`
(`src/runtime/services/system.rs:14-23, 58-80`). Services try table, text, then
system (`src/runtime/services/mod.rs:60-103`). Framework code can therefore
implement both a focused interaction and a framework fallback.

**Verdict:** `Kind::Framework` is claim provenance, not a menu category.

### C-03 — sets are installation bundles, not sections

`document::Editing::standard()` installs internal `ApplyEdit`, transfer
commands, Delete, and Select All in one `command::Set`
(`src/document/command.rs:83-106`). The authored Edit menus split those visible
commands into History, Clipboard, and Selection sections, while `ApplyEdit`
must never appear (`examples/text_editor/app/view.rs:121-131`,
`examples/control_gallery/app/view.rs:64-74`). `Set` itself is only an ordered
vector of typed installers (`src/command/set.rs:5-14, 22-54`).

**Verdict:** a set may install placement-bearing specs, but set boundaries do
not generate categories or separators.

### C-04 — stable vocabulary and disabled state are compatible

The registry already has the right three-state semantics:

- unknown registration returns `State::hidden()`
  (`src/command/registry.rs:99-107, 118-128`);
- registered with no claim returns `State::disabled()`
  (`src/command/registry.rs:109-115, 130-136`);
- a claim supplies enabled/disabled/check/label state, then registration
  defaults fill missing label and shortcut (`src/command/state.rs:94-105`).

**Verdict:** registration determines stable bar membership; Task traversal
determines current state and target. Context menus and palette results retain
their omit-unclaimed policies.

## Phase A — current-truth census

| Cell | Verdict | Repository receipts |
| --- | --- | --- |
| A-01 | `Standard` has twelve residents, but only six editing roles plus Undo, Redo, Close Window, and Command Palette have production role callsites. New/Open/Save/SaveAs exist only in the chord match table; the text editor registers literal `Primary+...` strings instead. | Enum: `src/command/spec.rs:9-22`. Editing: `src/document/command.rs:89-105`. History: `src/timeline/command.rs:64-73`. Session: `src/session/service.rs:94-106`. Raw file chords: `examples/text_editor/app/runtime.rs:17-23`. Platform table: `src/keymap.rs:197-233`. |
| A-02 | `KeyChordKind::Standard` currently owns the only stored path from a registration to role meaning. `Profile::chords` then projects it to one or more concrete chords. Minimal inversion: `Spec` stores `Option<Standard>`; shortcut lookup treats an absent explicit override as `KeyChord::standard(role)`. The `KeyChordKind` variant may remain as the chord projection, but no longer owns the role. | `src/command/spec.rs:25-28, 51-58, 80-82`; `src/keymap.rs:137-143, 197-233`. |
| A-03 | `Spec` owns static display name, optional shortcut, and palette listing. `AnyCommand` erases command/args/history and embeds the `Spec`; `Registry` owns the type map, shortcut map, and stable first-registration order. Role and static placement fit `Spec` without entering `Command`. | `src/command/spec.rs:122-164`; `src/command/registry.rs:17-31, 71-97`. |
| A-04 | Registry enumeration is stable first-registration order and restricted to unit-argument candidates. It creates erased unit triggers; state and invocation later downcast through existing erased paths. Re-registering a type replaces metadata without moving its order. | `src/command/registry.rs:50-60, 71-97, 161-179, 322-353, 405-435`; `src/command/trigger.rs:58-87, 109-191`. |
| A-05 | Global discovery, local discovery, and erased claim resolution are reusable policy pieces, but `resolve_candidates` is intentionally wrong for stable menu-bar membership because it drops `None` and errors and produces a claim-bearing `ResolvedAction`. Authored `Binding::menu` is the correct final presentation species. | `src/command/registry.rs:161-261`; `src/command/surface.rs:6-40`; `src/widget/binding.rs:19-63`; `src/view/binding.rs:104-112, 151-168`. |
| A-06 | Two actual authored bars exist: control gallery and text editor. Glass tuner only uses menu-dressed bindings as a rendering diagnostic, not a bar. Palette and context menus are derived menu-like surfaces with separate policies. Probe examples have no command menus. | `examples/control_gallery/app/view.rs:59-80`; `examples/text_editor/app/view.rs:110-136`; `examples/glass_tuner/app/view.rs:225-265`; `src/runtime/palette.rs:203-240`; `src/runtime/context_menu.rs:95-158`. |
| A-07 | The text editor's 27-line authored bar block contains 12 conventional command rows and four hand-authored separators; only Load Stress Text and two View toggles are application-specific. A derived declaration plus three explicit extensions can remove the whole conventional recipe. The control gallery contains seven conventional Edit rows and two separators; its Controls and View contents are genuine application vocabulary. | Physical blocks above; registrations at `examples/text_editor/app/runtime.rs:16-27` and `examples/control_gallery/app/runtime.rs:16-31`. |
| A-08 | Menu identity is explicit `interaction::Id`; labels are separate strings. Authored separators are leaf nodes. Binding state already projects label, checked state, shortcut, enabled state, and invocation route. Layout gives all menu rows a shared shortcut column, paint renders checks and separators, and existing focus traversal treats the resulting menu nodes generically. | `src/widget/menu.rs:47-71`; `src/interaction/menu.rs:13-65`; `src/view/binding.rs:84-113, 143-168`; `src/layout/algorithm.rs:1319-1366`; `src/scene/paint/mod.rs:639-668`; `src/tests/widget_focus_tests.rs:59-128`. |
| A-09 | Context menus traverse a captured semantic path using Inspection, or Task for an active table editor, and consume first claims per section. The palette captures one task scope, resolves globally, keeps enabled claims, then relevance-sorts inside provenance order. Neither policy is a menu-bar topology. | `src/runtime/context_menu.rs:10-23, 95-168`; `src/runtime/palette.rs:203-240, 253-279`; `src/responder/path.rs:3-56`. |
| A-10 | No existing command role/category/group/slot vocabulary exists. Existing names are principled non-merges: `view::Role` is node structure, `widget::binding::Placement` is Button-vs-Menu dress, geometry placement is popup positioning, paint `Group` is compositing, responder order is claimant provenance, and `subject::Path` is descriptive ancestry. Existing `interaction::Id` can continue to identify authored custom menus, but standard category identity needs typed command/menu vocabulary. `keymap::Platform` becomes a second platform-policy caller and should be reused or promoted, never duplicated. | `src/view/node/role.rs:1-23`; `src/widget/binding.rs:13-17`; `src/geometry/placement.rs:1-48`; `src/paint/mod.rs:302-308`; `src/responder/chain.rs:76-118`; `src/subject.rs:14-76`; `src/keymap.rs:5-22, 55-65`. |

### Authored-surface classification

| Surface | Conventional | Application-specific static | Dynamic / argument-bearing | Internal |
| --- | --- | --- | --- | --- |
| Text editor File | New, Open, Save, Save As, Close Window | Load Stress Text | none | none |
| Text editor Edit | Undo, Redo, Cut, Copy, Paste, Delete, Select All | none | none | `ApplyEdit` is registered but correctly absent |
| Text editor View | none | Wrap Text, Debug Panel | none | none |
| Control gallery Controls | none | Click, Reset | none | none |
| Control gallery Edit | Undo, Redo, Cut, Copy, Paste, Delete, Select All | none | none | `ApplyEdit` absent |
| Control gallery View | none | Wrap Text, Show Grid, Advanced | none | none |
| Glass tuner foreground sample | none | two diagnostic menu-dressed commands | none | not a menu-bar recipe |
| Command palette | all registered unit commands that claim and are enabled, except describer | relevance-generated | none | `OpenCommandPalette` excluded by listing policy |
| Context menu | nearest local unit commands that claim | owner/path generated | exact bound trigger may carry args | first claim consumes |

The compression witness should be measured in deleted recipe operations, not
merely formatter-dependent line count. The text editor repeats 12 conventional
bindings and four separators; the control gallery repeats seven conventional
bindings and two separators. Those 25 operations are deletion candidates. The
three text-editor deviations and five control-gallery category commands remain
explicit either as placement-bearing registrations or authored extensions.

## Phase B — current standard-role matrix

`Primary` below means Control on Windows/Linux and Command on macOS. Windows
Redo has both `Ctrl+Y` and `Ctrl+Shift+Z`, with `Ctrl+Y` displayed first
(`src/keymap.rs:214-220`). Every current embodied command has `Args = ()`.

### Meaning, labels, and chords

| Standard role | Default English label verdict | Windows/Linux chord | macOS chord | Existing command type | Automatic bar participation |
| --- | --- | --- | --- | --- | --- |
| Undo | `Undo` is honest | Primary+Z | Primary+Z | `timeline::Undo` | yes |
| Redo | `Redo` is honest | Primary+Y; Primary+Shift+Z | Primary+Shift+Z | `timeline::Redo` | yes |
| Cut | `Cut` is honest | Primary+X | Primary+X | `document::Cut` | yes |
| Copy | `Copy` is honest | Primary+C | Primary+C | `document::Copy` | yes |
| Paste | `Paste` is honest | Primary+V | Primary+V | `document::Paste` | yes |
| SelectAll | `Select All` is honest | Primary+A | Primary+A | `document::SelectAll` | yes |
| New | `New` is an honest generic default; apps may override `New File`/`New Window` | Primary+N | Primary+N | `document::NewFile` | yes |
| Open | default `Open…`; the unit command requests a path before completion | Primary+O | Primary+O | `document::OpenFile` | yes |
| Save | `Save` is honest | Primary+S | Primary+S | `document::SaveFile` | yes |
| SaveAs | default `Save As…`; the command requests a destination | Primary+Shift+S | Primary+Shift+S | `document::SaveAsFile` | yes |
| CloseWindow | default `Close Window`; never call it `Exit` or `Quit` | Alt+F4 | Command+W | `session::CloseWindow` | yes |
| CommandPalette | `Command Palette` is honest | Primary+Shift+P | Primary+Shift+P | `session::OpenCommandPalette` | **no default placement** |

The framework can honestly supply these English defaults before a localization
system exists because it already requires English `&'static str` labels for
every registration. The role must remain stored separately from the resolved
label so a later locale projection replaces framework defaults without changing
application registrations. Explicit label override remains available now.

### Platform topology, slots, and relocation

| Role | Windows/Linux category → section → slot | macOS category → section → slot | Relocation / special verdict |
| --- | --- | --- | --- |
| Undo | Edit → History → first | Edit → Undo/Redo → first | none |
| Redo | Edit → History → after Undo | Edit → Undo/Redo → after Undo | none |
| Cut | Edit → Clipboard → first | Edit → Pasteboard → first | none |
| Copy | Edit → Clipboard → after Cut | Edit → Pasteboard → after Cut | none |
| Paste | Edit → Clipboard → after Copy | Edit → Pasteboard → after Copy | none |
| SelectAll | Edit → Selection → first | Edit → Selection/Text Editing → first | remains separated from Clipboard |
| New | File → Document → first | File → New Item → first | standard role is a virtual anchor even when absent |
| Open | File → Document → after New | File → New/Open → after New | Apple exposes New placement but Open is scene/capability-derived; this mapping is framework policy |
| CloseWindow | File → Document → after Open | File → Save Item → first | **not** Window menu and **not** application menu; Quit is a different missing role |
| Save | File → Document → after Close Window | File → Save Item → after Close | none |
| SaveAs | File → Document → after Save | File → Save Item → after Save | none |
| CommandPalette | none | none | chord/description role only; explicit placement required |

For every row, automatic placement is valid only when the registered command's
argument type is `()`. Registration already knows the erased args `TypeId`
(`src/command/registry.rs:24-31, 50-60`). A placement-bearing `Spec` on an
argument-bearing command must be rejected, not silently turned into a unit
trigger. `OpenPath` and `SaveToPath` remain authored argument actions; Recent
Files remains an authored dynamic section.

Role collision policy is uniform: one registered command type may own a
standard role. Re-registering the same type may replace its spec while keeping
its order; a different type claiming the same role is a configuration error
rejected before projection. The established architecture expects multiple
responders to claim one command type, rather than multiple command types to
represent the same semantic command. Exact error delivery (startup panic under
the current chaining API versus a new fallible registration boundary) is a
public-API flag for the campaign; last-wins and registration-order wins are
rejected as silent topology lies.

### Missing-role decisions

| Candidate | Verdict |
| --- | --- |
| Delete | **Admit in the campaign.** `document::Delete` is an existing unit caller, both authored Edit menus expose it, Microsoft `StandardUICommand` includes it, and current `Standard` omission forces it outside the role table. Default placement: Edit/Deletion after Selection on Windows/Linux; platform template decides macOS text-editing position. |
| About | Watch line. No registered command caller. Required before a complete macOS application-menu projection. |
| Settings / Preferences | Watch line. No caller. Must support macOS application-menu relocation when admitted. |
| Quit | Watch line. No caller. Must be distinct from `CloseWindow`; macOS app menu and Windows/Linux File/Exit are different semantics. |
| Print | Watch line. No caller. Apple and Microsoft both have conventional placement. |
| Find | Watch line. No caller. Conventional Edit placement, but no search command exists. |
| Help | Watch line. No caller. Both a category anchor and a command role may eventually be needed; do not conflate them. |

## Phase C — primary-source research and platform templates

### Research findings

- Apple [`CommandGroupPlacement`](https://developer.apple.com/documentation/swiftui/commandgroupplacement)
  exposes semantic slots such as app info/settings/termination, new item, save
  item, print, undo/redo, pasteboard, text editing, windows, and help. The names
  are not UI strings; they are placement identities. Its
  [`saveItem`](https://developer.apple.com/documentation/swiftui/commandgroupplacement/saveitem)
  group includes Close, Save, Save As/Duplicate, and Revert. This directly
  supports typed cultural slots and the CloseWindow-not-Quit verdict.
- Apple's [menu guidance](https://developer.apple.com/design/human-interface-guidelines/menus)
  says unavailable regular-menu items remain visible and dimmed, a menu remains
  available even if all its items are unavailable, related commands form
  groups, and separators distinguish groups. Apple's
  [context-menu guidance](https://developer.apple.com/design/human-interface-guidelines/context-menus)
  separately says unavailable contextual items should usually be hidden. That
  is the exact stable-bar versus contextual-surface policy split required here.
- Apple's
  [SwiftUI menu-bar customization](https://developer.apple.com/documentation/swiftui/building-and-customizing-the-menu-bar-with-swiftui)
  derives default menus from scene capability, updates command state from focus,
  inserts custom top-level menus after View, and supports typed before/after and
  replacement of standard groups. It demonstrates that topology and current
  claimant are separate inputs.
- Microsoft's [Windows menu guidelines](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus)
  publish File/Edit/View/Tools/Help conventions and the exact Edit sequence:
  Undo/Redo, Clipboard, Select All, Delete, Find, with separators between
  semantic groups. The same page distinguishes stable standard menus from
  concise contextual menus.
- Microsoft
  [`StandardUICommand`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.input.standarduicommand)
  derives label, icon, shortcut, and description from a standard command kind.
  Its role catalog includes Delete, reinforcing the missing-role verdict.
- Qt [`QAction::MenuRole`](https://doc.qt.io/qt-6/qaction.html) and
  [`QMenuBar`](https://doc.qt.io/qt-6/qmenubar.html) relocate About,
  Preferences, and Quit into the macOS application menu by semantic role. Qt's
  legacy text heuristic is evidence against label-based merging: the typed role
  is the reliable mechanism and visible-string matching is merely fallback.
- GIO [`MenuModel`](https://docs.gtk.org/gio/class.MenuModel.html) represents
  ordered sections and derives a separator between nonempty sections. Its
  [`new_section`](https://docs.gtk.org/gio/ctor/MenuItem.new_section.html)
  example explicitly separates Undo/Redo from Cut/Copy/Paste. `GActionGroup`
  separately supplies action enabled/state, again splitting topology from live
  command state.

### Platform templates

| Platform profile | Category order | Standard sections / policy | Relocations | Custom insertion |
| --- | --- | --- | --- | --- |
| Windows | File · Edit · View · custom · Tools · Window · Help, omitting categories with no registered or authored entries | File/Document; Edit/History · Clipboard · Selection · Deletion; later View/Tools/Window/Help groups as roles arrive | Quit, when admitted, belongs at File/Exit; CloseWindow remains File/Document | after View and before Tools/Window/Help by default; explicit authored order may override |
| Linux | Same starting framework policy as Windows, pending toolkit/desktop integration | Same semantic groups; GIO-style sections generate separators | no OS relocation claimed | same insertion region; this is framework policy, not a claim of one universal Linux HIG |
| macOS | Application · File · Edit · View · custom · Window · Help | File/New-Open · Save Item; Edit/Undo-Redo · Pasteboard · Selection/Text Editing | About, Settings, and Quit eventually move to Application; CloseWindow remains File/Save Item | SwiftUI precedent: custom categories after View; typed group extension/replacement remains available |

Category identity, visible label, category order, section identity, section
order, role slots, and relocation are distinct fields in the template. A
platform may render two semantic groups contiguously or rename a visible label
without changing identity. No visible label participates in merging.

No macOS hardware or native global-menu integration was exercised. The macOS
table is a pure mapping backed by primary documentation, not a claim of visible
correctness in this framework.

## Phase D — placement vocabulary and public API verdict

### Alternatives

#### Alternative 1 — promoted role (**recommended**)

```rust
Spec::standard(Standard::Copy)
```

This is the only shape that makes the role the owner of all defaults. It removes
the current label/chord duplication and gives placement one semantic source.
Explicit projection overrides remain possible:

```rust
Spec::standard(Standard::Open)
    .display_name("Open Project…")
    .key_chord(KeyChord::new("Primary+Shift+O"))
    .placement(Placement::after(Standard::New))
```

An explicit `.unplaced()` suppresses automatic bar participation while keeping
the standard role's other projections. `CommandPalette` is unplaced by default.

#### Alternative 2 — role attached to an ordinary spec (supported as sugar)

```rust
Spec::new("Copy").role(Standard::Copy)
```

This is useful as migration/override sugar, but it should not be the primary
constructor because it keeps conventional label repetition at every callsite.
Semantically it must produce the same internal state as
`Spec::standard(...).display_name(...)`.

#### Alternative 3 — independent shortcut and placement (**rejected**)

```rust
Spec::new("Copy")
    .key_chord(KeyChord::standard(Standard::Copy))
    .placement(Placement::standard(Standard::Copy))
```

This repeats the same semantic role into two projections and permits them to
drift. It is the duplication baseline, not a viable owner model.

### Recommended placement vocabulary

Static unit commands may carry placement in `Spec`:

```rust
Spec::new("Load Stress Text")
    .placement(Placement::section_after(Standard::SaveAs))
```

Required public operations:

- `Placement::before(Standard)` and `Placement::after(Standard)` for the same
  semantic section;
- `Placement::section_before(Standard)` and
  `Placement::section_after(Standard)` when a separator boundary is intended;
- `Spec::unplaced()` for a standard role deliberately absent from the bar;
- explicit standard-role placement override without changing label or chord.

Standard roles occupy **virtual slots** even when their commands are
unregistered. An extension after `SaveAs` therefore remains at the SaveAs
boundary when SaveAs is absent; it does not drift to whichever registered item
happens to be nearby. No public numeric ranks and no registration-order fallback
participate in conventional placement.

Standard category identity should be a typed `command::Category` (Application,
File, Edit, View, Tools, Window, Help). Existing `interaction::Id` remains the
identity of an authored custom menu; its visible label remains a separate
string. Do not merge a custom category into a standard category by comparing
labels.

`keymap::Platform` is currently the only platform enum and the menu template is
its second policy consumer. The campaign must either promote that enum to a
general platform profile or deliberately let menu topology consume it; it must
not mint a parallel Windows/Mac/Linux enum.

### Duplicate and argument policies

- Same command type re-registered: replace metadata, retain first-registration
  position, as today.
- Different command types with one `Standard` role: reject as configuration
  error before a derived bar is exposed.
- Multiple responders for the one command type: expected; Task traversal picks
  the current claim.
- Placement-bearing non-unit command: reject at registration. It cannot produce
  an honest trigger without arguments.
- Dynamic/argument-bearing menu entries: authored through the mixed grammar and
  ordinary `Binding::menu_with_args`; never synthesized from registry metadata.

## Phase E — derived menu-bar projection

### Recommended pipeline

```text
Registry::menu_candidates() -> Candidates<Bar>
        |
        | stable registration metadata; unit args only
        v
platform topology template + authored extension recipe
        |
        v
categories / sections / virtual role slots
        |
        | for each registered entry
        v
state_any(..., responder::Traversal::Task)
        |
        | claimant present -> claimed state
        | no claimant       -> disabled state
        v
ordinary view::Binding(Source::Menu, Route::Chain)
        |
        v
ordinary Menu / Separator / Binding nodes
        |
        v
existing layout, focus, paint, popup, and lifecycle machinery
```

`Candidates<Bar>` should be a third typed discovery policy beside `Global` and
`Local`. It reuses `Candidate`, `AnyTrigger`, registration indices, unit-args
gating, and type erasure. It must not call the existing `resolve_candidates`
because that method's contract is “claimed actions only.” This is a documented
non-merge, not a parallel command resolver: candidate discovery and state
resolution remain registry-owned, while only the surface membership policy
differs.

`ResolvedAction` is reusable for entries with claims but cannot be the bar's
sole intermediate because its `claim: responder::Claim` cannot represent a
registered unclaimed entry. Do not weaken that type to `Option<Claim>` merely
to accommodate a different surface policy. Instead, add one internal binding
projection from erased trigger + resolved `State` + `Route::Chain`; invocation
then resolves the current Task chain exactly as authored menu bindings do.

### Required behavior answers

1. **Enumeration:** directly from registry metadata through typed
   `Candidates<Bar>`, not global/palette candidates.
2. **Unclaimed:** retain the registered item and project disabled state.
3. **Empty categories:** omit standard categories with no registered or
   authored entries. Fully authored categories retain existing behavior.
4. **All-disabled category/group:** keep it and keep its title/section; disabled
   is state, not absence.
5. **Separators:** derive one separator between adjacent nonempty sections.
   Disabled entries make a section nonempty; absent entries do not. Never emit
   leading, trailing, or doubled separators.
6. **Duplicate roles:** reject during registry configuration, before topology
   projection.
7. **Topology rebuild:** only registration metadata, platform profile, or
   authored-extension topology changes. Focus, enabled/check state, and dynamic
   label changes do not rebuild topology.
8. **Live state:** resolve label override, shortcut override, enabled/disabled,
   checked state, and target on ordinary view rebuilds through Task traversal.
9. **Keyboard navigation:** the derived projection creates the same `Menu`,
   `Separator`, and `Binding` nodes consumed by existing focus/navigation tests;
   no new keyboard system.
10. **Architecture witness:** assert that the derived projection contains only
    ordinary `Role::MenuBar`, `Role::Menu`, `Role::Separator`, and
    `Role::Binding` nodes and enters the same authored popup lifecycle.

Topology may be cached by `(registry metadata revision, platform profile,
authored extension revision)`, but caching is not required for the first
campaign. The ownership distinction is required: topology is stable metadata;
per-item state is live resolution.

## Phase F — mixed authored/derived composition grammar

Keep the current fully authored `ui.menu_bar(|ui| ...)` API behaviorally
unchanged. Add a distinct specialized builder for conventional composition so
ordinary `Ui` does not acquire topology methods:

```rust
ui.command_menu_bar(|bar| {
    bar.conventional();

    bar.section_after(Standard::Open, |ui| {
        // Dynamic recent files: ordinary argument-bearing bindings.
    });

    bar.extend(Category::View, |ui| {
        ui.add(Binding::<ToggleWrapText>::menu());
        ui.add(Binding::<ToggleDebugPanel>::menu());
    });

    bar.custom(TOOLS_MENU, "Tools", |ui| {
        // Authored custom category, identity separate from label.
    });
});
```

The exact builder name is a public naming flag; the grammar is the verdict.

| Use case | Grammar verdict |
| --- | --- |
| Fully authored bar | Existing `ui.menu_bar` unchanged. |
| Fully derived conventional bar | `command_menu_bar` with only `bar.conventional()`. |
| Custom top-level category | `bar.custom(interaction::Id, label, authored children)`; never label-merged. |
| Static command after a standard role | Prefer `Spec::placement(Placement::after/section_after(role))`; no bar recipe. |
| Dynamic section | `bar.section_before/after(Standard, closure)`; closure emits ordinary bindings, including args. |
| Extend standard category with authored items | `bar.extend(Category, closure)` at a documented semantic edge; explicit before/after role when position matters. |
| Replace conventional group | `bar.replace_group_containing(Standard, closure)`, so public code anchors the role rather than a platform-dependent numeric group. |
| Replace category | `bar.replace(Category, closure)`. |
| Recent files | Dynamic section after Open; `Binding::menu_with_args(path)`. |
| Open-window list | Dynamic authored Window-category section. |
| Argument-bearing entry | Authored binding only; never registry-enumerated as a unit command. |
| Application command relative to standard anchor | Static `Spec::placement` or dynamic builder section. |

Authored extension order is explicit authored order inside its declared slot.
Conventional role order is platform-template order. These are two honest
owners; registration order is neither.

## Phase G — reuse, deletion, and non-merges

### Reuse map

| Existing concept | Verdict |
| --- | --- |
| `command::Spec` | Promote with role and placement metadata; correct owner. |
| `Registry::order` and command map | Reuse for deterministic enumeration and metadata lookup; do not use order as topology. |
| Typed `Registry::register<C>` → `AnyCommand` erasure | Reuse exactly; role remains data on the erased spec. |
| `AnyTrigger::unit` and candidate typestate | Reuse; add `Candidates<Bar>` policy marker. |
| `Registry::state_any` / Task chain | Reuse for live state. |
| `ResolvedAction` | Partial reuse only; principled non-merge for unclaimed stable entries because it requires a claim. |
| `view::Binding` / `Binding::menu` | Reuse as final row species; add only an internal erased state constructor if needed. |
| Menu row layout, checked/shortcut paint, focus order | Reuse unchanged. |
| Authored/contextual popup lifecycle | Reuse unchanged; derived menus must become ordinary nodes before layout. |
| `keymap::Profile` | Reuse as platform-projection pattern and chord owner. Reuse or promote its Platform enum. |
| `responder::Traversal::Task` | Reuse per item for current claimant; never use it for topology order. |
| `command::Set` | Installation only; rejected as group owner. |
| Palette/context resolver policies | Keep unchanged; documented non-merge with stable bar membership. |

### Deletion forecast

The proposed API makes these deletions explicit:

- repeated conventional labels in timeline, document editing, session, and
  text-editor file registrations, replaced by `Spec::standard` defaults;
- raw standard file shortcut strings in the text editor, replaced by role
  projection;
- 19 repeated conventional binding rows across the two example bars;
- six hand-authored conventional separators across those bars;
- conventional category/group recipes in examples where the derived template
  owns them;
- registration-order dependence for conventional topology (registration order
  remains only a deterministic discovery tiebreaker for non-topology surfaces);
- any future label-string relocation heuristic, structurally prevented by
  typed roles and categories.

`KeyChordKind::Standard` need not be deleted as a representation of a projected
standard chord. What is deleted is its accidental ownership of the only stored
role meaning. The variant becomes a consumer of `Spec.standard`.

The following do **not** count as deletions:

- authored custom categories and dynamic sections;
- menu popup/layout/paint/focus machinery;
- context-menu Inspection ordering;
- palette Task capture and relevance ordering;
- `command::Set` as an installer;
- registration order itself, which still serves palette tiebreaking.

## Stable membership and live resolution policy

The derived menu bar has two deliberately different update domains. Stable
topology comes from registration metadata, the platform profile, and authored
extension declarations. Live presentation comes from resolving each existing
topology entry through `responder::Traversal::Task`.

| Registration | Current Task claim | Bar membership | Presentation | Invocation |
| --- | --- | --- | --- | --- |
| absent | irrelevant | absent | none | none |
| present | absent | present | disabled, using registration defaults | inert until a claim exists |
| present | disabled claim | present | disabled, with resolved label/check/shortcut overrides | disabled |
| present | enabled claim | present | enabled, with resolved state | route through the current Task chain |

Focus and responder changes may update label, shortcut override, check state,
enabled state, and target. They must not move the row, merge groups, remove a
standard category, or change separators. A registered all-disabled category is
still a category. An unregistered virtual standard slot is still a stable
anchor for authored relative placement, but it emits no row.

This policy is specific to persistent conventional bars. Context menus continue
to omit unclaimed actions because they inspect the current object; the palette
continues to omit unclaimed or disabled results because it describes actionable
commands in the captured task. Neither surface receives placeholder rows.

## Required witnesses for the implementation campaign

| Witness | Required proof |
| --- | --- |
| Same-scope grouping | Focused text claims both Undo and Copy at `Kind::Focused`; the projected bar still renders History and Clipboard as distinct sections with one separator. |
| Provenance-independent topology | Moving Undo between focused history and framework timeline changes its claimant/state but not File/Edit/View order, row position, or separators. |
| Stable unclaimed membership | A registered standard command with no claimant remains in its standard slot and is disabled. |
| Absent registration | An unregistered role emits no row, while its virtual slot remains a deterministic anchor for extensions. |
| Separator stability | Focus changes and enable/check changes never create, remove, double, lead, or trail a semantic separator. |
| Chord projection | Every current role retains the existing Windows, Linux, and macOS `keymap::Profile` results, including both Windows Redo chords. |
| Close Window | Windows/Linux place it in File/Document after Open; macOS places it at the front of File/Save Item; no profile treats it as Quit. |
| Command Palette | It retains its standard chord and label but is absent from a derived bar unless explicitly placed. |
| Internal command exclusion | `document::ApplyEdit` never becomes a bar candidate despite membership in `Editing::standard()`. |
| Stable authored anchor | An authored section after Save As remains at that virtual boundary when New, Open, Save, or Save As registrations are independently absent. |
| Dynamic coexistence | A Recent Files argument-bearing section coexists after Open without being synthesized as a unit command. |
| Authored compatibility | Existing fully authored `ui.menu_bar` behavior, identity, focus order, and popup lifecycle remain unchanged. |
| Ordinary-node projection | Derived output contains only the existing menu-bar, menu, separator, and binding node species and uses the existing popup lifecycle. |
| Context isolation | Context-menu Inspection ordering and first-claim consumption remain byte-for-byte policy-equivalent. |
| Palette isolation | Palette Task capture, omit-unclaimed/disabled behavior, relevance ordering, and self-exclusion remain unchanged. |
| Duplicate-role rejection | Two distinct registered command types claiming one `Standard` role fail before any partial bar is exposed; re-registering the same type remains deterministic. |
| Unit-args boundary | Placement-bearing non-unit registration is rejected; authored argument bindings remain valid. |
| Compression witness | The migrated examples delete the 19 conventional bindings and six separators identified in Phase A while retaining every genuine deviation. |

The same-scope witness is doctrine-grade rather than merely a snapshot test. It
must exercise actual focused claims, not construct two synthetic entries with
matching enum values, so a future attempt to group by responder provenance
fails against the real service chain.

## Public API flags for morning review

1. **Role constructor naming.** The recommendation is
   `Spec::standard(Standard)`, with `.role(Standard)` only as migration sugar.
   Decide whether keeping both improves migration enough to justify two
   spellings for one internal fact.
2. **Placement vocabulary location.** `command::Placement` is the leading
   namespace, but it must not collide conceptually with geometry placement or
   `widget::binding::Placement`. A more explicit `command::MenuPlacement` is a
   naming fallback, not a new meaning.
3. **Conventional-bar builder name.** `ui.command_menu_bar` communicates the
   specialized grammar; `ui.conventional_menu_bar` communicates the result.
   The specialized builder must remain distinct from ordinary `Ui` either way.
4. **Duplicate-role error delivery.** `Registry::register` currently chains
   infallibly. Decide between an eager configuration panic with a precise
   diagnostic and a new fallible registration boundary. Silent last-wins and
   registration-order wins are forbidden.
5. **Non-unit placement rejection.** Decide whether the unit-argument boundary
   is enforced when building the `Spec`, at typed registration, or both. It
   must fail before menu projection and name the offending command and role.
6. **Platform vocabulary altitude.** Menu topology is the second consumer of
   `keymap::Platform`. Decide whether to promote that enum or let the command
   topology module consume it without renaming; do not duplicate it.
7. **Missing Delete role.** Adding `Standard::Delete` is an enum/API expansion
   within the campaign. It is justified by an existing caller and both platform
   conventions, but deserves its own changelog/API flag.
8. **Default label overrides.** `Spec::standard` should derive the English
   label and chord only when the corresponding explicit override is absent.
   Confirm builder-call order cannot accidentally change precedence.
9. **Category extension names.** The exact names for `extend`, `replace`, and
   section-relative authored closures should be reviewed as one grammar rather
   than independently accumulated methods.
10. **Localization future.** Standard role must remain the persisted semantic
    input so default English labels can later become locale projections without
    changing application registrations. No localization system is built here.

## Documented non-merges

- **Role is not shortcut.** `KeyChordKind::Standard` remains a chord
  representation; it no longer owns standard meaning.
- **Role is not responder scope.** Task provenance selects the current actor;
  it does not encode category, group, or slot. Undo/Copy is the binding case.
- **Role is not command type.** One command type may be assigned one standard
  presentation role by registration metadata; execution identity remains the
  command type.
- **Role is not a command set.** `command::Set` installs commands, including
  internal ones, and cannot generate visible sections.
- **Category is not a visible label.** Typed standard categories and explicit
  authored identities may share or change labels without merging.
- **Command placement is not geometry placement.** One locates vocabulary in a
  menu; the other locates surfaces in space.
- **Command placement is not binding dress.** `widget::binding::Placement`
  chooses Button versus Menu presentation and stays unchanged.
- **Semantic group is not paint `Group`.** The latter owns compositing.
- **Menu-bar membership is not palette discovery.** A persistent bar retains
  registered unclaimed entries disabled; palette discovery intentionally drops
  them and relevance-sorts the remainder.
- **Menu-bar topology is not context inspection.** Context menus continue to
  traverse `responder::Path` using Inspection (or Task for the active editor)
  and consume claims coarse-to-fine.
- **Stable bar candidate is not always `ResolvedAction`.** The latter requires
  a real claim. Do not weaken it to accommodate an unclaimed persistent row.
- **Custom categories are not standard-category extensions by label.** The
  application must opt into a typed category extension or create a separately
  identified custom category.
- **Close Window is not Quit/Exit.** Closing the active window and terminating
  the application have different platform placements and future roles.
- **Registration order is not cultural order.** It remains useful for
  deterministic enumeration and palette tiebreaking only.
- **Platform template is not native integration.** The macOS mapping is
  designed from primary sources; visible global-menu integration remains
  hardware/platform work.

## Synthesis

1. **Promotion verdict:** promote `command::Standard`. The current enum already
   names semantic commands; storing it only under shortcut metadata is the
   duplication source.
2. **Ownership:** `command::Spec` owns optional standard role and explicit
   placement. Platform policy projects defaults and topology. The registry
   owns uniqueness and enumerable membership. Task traversal owns current
   actor/state.
3. **Twelve-role matrix:** all current residents have label, chord,
   participation, category, group, slot, relocation, argument, command-type,
   and collision verdicts in Phase B.
4. **Missing-role watch list:** admit Delete now because it has a caller;
   record About, Settings/Preferences, Quit, Print, Find, and Help until real
   callers arrive.
5. **Platform templates:** Windows and the initial Linux policy use
   File/Edit/View/custom/Tools/Window/Help; macOS additionally projects an
   Application category and platform relocations. Identity and visible text
   remain separate.
6. **Stable membership:** registration creates the persistent vocabulary;
   Task traversal supplies live state. Unclaimed is disabled, unregistered is
   absent, all-disabled categories remain.
7. **Relative placement:** typed before/after and section-before/after anchors
   target virtual `Standard` slots; no strings, numeric ranks, or registration
   order.
8. **Duplicate roles:** a second command type claiming one standard role is a
   configuration error before exposure; re-registering the same type and
   multiple responders for one type retain their existing meanings.
9. **Mixed grammar:** authored bars remain; a specialized conventional builder
   composes a derived template with typed category/group extensions,
   replacements, dynamic sections, and separately identified custom menus.
10. **Reuse:** preserve typed registration, `AnyCommand`/`AnyTrigger` erasure,
    registry state resolution, Task traversal, ordinary Binding/Menu nodes,
    navigation, paint, and popup lifecycle. Add only the persistent-bar
    candidate policy and erased binding projection it genuinely lacks.
11. **Deletion forecast:** 19 conventional rows, six separators, repeated
    standard labels/chords, and authored standard group recipes disappear from
    the examples; future label heuristics are prevented.
12. **Public API flags:** constructor/builder names, error delivery, placement
    namespace, platform-enum altitude, Delete admission, and localization
    readiness require morning review.
13. **Non-merges:** the preceding section records every nearby concept that
    must remain distinct, especially responder scope, `command::Set`, palette,
    context inspection, and claim-bearing `ResolvedAction`.
14. **Implementation shape:** five independently green checkpoints follow.
15. **Verdict:** **go**. The ownership model is complete enough to implement;
    remaining flags affect API spelling or error delivery, not architecture.

## Draft implementation campaign — One Conventional Vocabulary

### Mission

Make standard command meaning the single source for conventional labels,
chords, and menu topology, then derive ordinary menu bars through the existing
typed-registration, erased-resolution, and menu-presentation pipeline. Preserve
authored surfaces and the distinct context/palette policies.

### Checkpoint 1 — promote role meaning without changing menus

- Add standard role and explicit menu placement metadata to `command::Spec`.
- Add `Standard::Delete` and assign existing timeline, document, and session
  registrations their honest roles.
- Project default labels and existing `keymap::Profile` chords from the role;
  explicit label/chord overrides win independently.
- Correct `CloseWindow`'s registration default from `Exit` to `Close Window`;
  do not add Quit.
- Enforce one standard role per registered command type and reject distinct
  command types colliding on a role.

Independent green boundary: all existing authored menus behave unchanged;
cross-profile chord tests and the twelve-plus-Delete role-default matrix pass.

### Checkpoint 2 — pure platform topology and placement algebra

- Implement typed categories, semantic sections, virtual standard slots, and
  Windows/Linux/macOS templates as a pure projection over registration
  metadata.
- Implement typed before/after and section-before/after placement plus
  unplaced/override behavior.
- Reject placement-bearing non-unit commands before exposure.
- Pin empty/all-disabled policy, virtual-anchor stability, category order,
  CloseWindow relocation, CommandPalette absence, and ApplyEdit exclusion.

Independent green boundary: topology is testable without layout or runtime
claims and is invariant under shuffled registration order and responder state.

### Checkpoint 3 — persistent bar candidates through ordinary menu nodes

- Add the typed `Candidates<Bar>` registry policy over existing erased unit
  triggers and metadata.
- Resolve each entry through `Traversal::Task`; project missing claims as
  disabled state without changing `ResolvedAction` or other candidate policies.
- Produce ordinary `Binding`/Menu/Separator nodes and reuse existing navigation,
  paint, and popup lifecycle.
- Derive separators only between nonempty semantic sections.

Independent green boundary: same-scope Undo/Copy stays separated, focus changes
update state without topology movement, and the architecture witness proves no
parallel menu widget or popup path exists.

### Checkpoint 4 — one mixed authored/derived grammar

- Add the specialized conventional-menu-bar builder while leaving
  `ui.menu_bar` unchanged.
- Support typed standard-category extension, relative dynamic sections,
  category/group replacement, and separately identified custom categories.
- Prove Recent Files and open-window lists use ordinary argument bindings and
  never enter unit-command enumeration.
- Prohibit label-based merging.

Independent green boundary: fully authored, fully derived, and mixed examples
coexist; existing authored behavior and identity tests remain green.

### Checkpoint 5 — migration, deletion, and constitutional closeout

- Migrate the text editor and control gallery, preserving every genuine custom
  command and deleting the 19 conventional rows and six separators counted in
  this investigation.
- Run every witness in the required matrix, including context/palette isolation
  and all platform profiles.
- Record actual deletions, API review decisions, migration notes, and doctrine:
  *a menu orders by what it is about; cultural topology and live resolution are
  separate truths*.
- Run formatting, all-target compilation, the full library/doctest suite, all
  application smokes, and comparison-example protection.

Independent green boundary: examples demonstrate maximum conventional behavior
with no topology recipe, deviations remain explicit, and no old standard-menu
recipe survives in the migrated surfaces.

### Campaign rails

- Ledger/census before each API expansion; no role admitted without a caller or
  platform obligation.
- One independently green commit per checkpoint.
- No changes to context-menu Inspection ordering or palette Task/relevance
  ordering.
- No label heuristics, public numeric ranks, silent collision resolution, or
  parallel menu presentation species.
- Platform mapping tests are policy tests; macOS visible integration remains
  hardware-gated.

## Completion audit

- [x] Every Phase A cell has repository receipts.
- [x] Same-scope History/Clipboard grouping is binding case law.
- [x] All twelve current roles have complete placement verdicts.
- [x] Registry membership and live state are distinguished.
- [x] `command::Set` is rejected as a group owner with evidence.
- [x] Mixed authored/derived composition has one grammar.
- [x] Platform claims cite primary sources and macOS limitations are explicit.
- [x] Reuse, deletion, API flags, non-merges, witnesses, and draft checkpoints
  are recorded.
- [x] No production code changed.
- [x] Formatting/diff hygiene and `comparison_open: true` are verified; this
  document is the only path staged for its independent commit, and no push is
  performed. The commit receipt lives in repository history.

**Campaign-readiness verdict: GO.** The campaign should not relitigate whether
scope can order conventional groups: C-01 proves that it cannot. Implementation
may refine names and error delivery, but the semantic owner, projection
boundaries, surface policies, and acceptance witnesses are settled.
