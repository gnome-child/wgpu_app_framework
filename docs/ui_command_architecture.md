# UI and Command Architecture Boundary

This document captures the intended architecture for a rewrite of the tangled `ui`/tree and command integration layers.

## North Star

```text
Node tree is data.
Snapshot is derived structure.
Interaction is runtime state.
Commands are capability contracts.
Runtime wires them together.
```

The stable parts of the current system already point in this direction:

- `ui::layout` owns layout primitives, the generic engine, and the `ui::Node` adapter.
- `ui::layout` has no app or command knowledge.
- command definitions work best as invocation/effect contracts, not UI path bindings.
- runtime should decide context, focus, scope, and capture.
- examples become clearer when manual UI IDs are unnecessary.

## Layer Ownership

### `ui::Node`

Declarative view data only:

- layout
- style
- content
- intrinsic roles such as focusable, hit-testable, text input, scroll, menu
- generic action affordances such as routes, responder keys, target categories, and action subject policy
- children

It should not own command registry state, command execution, command context resolution, or app behavior.
Action affordances are command-agnostic data. A command-backed widget projects a
`command::binding::Route` or `command::binding::Binding` into `ui::ActionRoute` /
`ui::ActionBinding`; the UI tree does not import command.

### `action`

Shared affordance vocabulary between UI and app/runtime:

- `action::Key`
- `action::Target`
- `action::Route`
- `action::Binding`
- `action::State`
- `action::Subject`

This layer is intentionally smaller and more generic than `command`. It lets UI
describe "this thing can be acted on" without knowing whether the action came
from a command, a menu, a tool, or a future runtime concept.

### `ui::Snapshot`

Derived immutable tree index for one frame:

- structural paths
- action routes, bindings, target categories, subjects, and scopes
- text surfaces
- menu surfaces
- interactivity and cursor declarations

Snapshot construction is side-effect free. `ui::Composition` owns the snapshot
alongside layout, focus order, widget metrics, and projected visual state. Layout
and focus order remain composition concerns because they depend on measurement
and visual availability, while snapshot remains the structural index.

### `ui::Interaction`

Mutable UI runtime state:

- hover
- focus
- press
- pointer capture
- text session
- scroll offsets
- open menus and popups

It may produce intents, but it should not execute commands.

### `command::Registry`

Command definitions and command state:

- command metadata
- shortcuts
- target kinds
- configured state
- running state
- typed call preparation

It should not need a UI tree. A registered definition does not make a command
globally available by itself. Presentation availability comes from projected
target state or explicit responder binding state, while execution still checks
the resolved target state before invoking.

### `app::command::Layer`

Explicit bridge between UI and commands:

- reads `ui::Snapshot`
- reads `ui::Interaction`
- reads command affordance metadata
- resolves current/focused/captured/window/path context
- creates `command::Call`
- projects `command::State` into generic UI visual state
- decides whether a UI-originated action can resolve to a non-disabled command request

This is the correct place for coupling.

### `app::Runtime`

Owns and coordinates engines:

- view tree
- composition snapshot
- interaction
- command registry
- command layer
- text engine
- paint scene

Runtime is allowed to know about everything because its job is orchestration.

## Target Flow

```rust
let tree = app.view();

let composition = ui::Composition::build(&tree, viewport, &mut text_engine);

interaction.apply(input, &composition);

let command_projection = commands.project(&composition, &interaction, &registry);

let scene = ui::paint(
    &composition,
    &interaction,
    command_projection.visuals(),
    &mut text_engine,
);
```

The important property is that UI composition and paint do not take `command::Registry`.

## Boundary Rules

No layer should ask for a thing it does not own.

- `ui::Snapshot` should not ask for `command::Registry`.
- `ui::Paint` should not ask for `command::Registry`.
- `command::Registry` should not ask for `ui::Tree`.
- `app::command::Layer` may ask for both UI/action and command data, because it is explicitly the bridge.

## Current Coupling To Remove

The original main smell was `Tree::compose(..., commands: &mut command::Registry, ...)`.

That made UI composition responsible for command state projection, menu command availability,
and registry mutation. Those belong in a command bridge layer.

Current progress:

- `Tree::compose` no longer receives a `window::Id` or concrete `command::Registry`.
- menu popup construction receives a `widget::Presenter` instead of a registry.
- command-backed menu presentation is implemented in `app::command`, not `widget`.
- command visual state projection is owned by `app::command`.
- action visual state projection only publishes explicit command overrides and running state; an unprojected registered command is not treated as enabled or disabled by UI paint.
- `ui::Composition` stores `ui::VisualState`, not `command::State`.
- `ui::paint` consumes `ui::VisualState` and does not import command state or registry APIs.
- `ui::floating::Surface` is visual-only; command context/source lives in `app::floating`.
- `ui::Interaction` carries UI interaction state only; command subject and scope captures live in `WindowState` and `app::command`.
- popup command scope contexts are derived by `app::command` from `app::floating`, not stored on `ui::Composition`.
- `Id`/`Path` are shared routing primitives re-exported by `ui`, so `command::call` no longer imports `ui` just to represent scope paths.
- `Key`/`Modifiers` are shared input primitives re-exported by `ui`, so `command::shortcut` no longer imports `ui`.
- `ui::Node` and `ui::Composition` store generic action routes, bindings, target categories, subjects, and scopes instead of command routes, responders, target kinds, and command scopes.
- widgets are the command-to-action projection boundary for command-backed controls.
- `app::command` is the action-to-command decoding boundary for registry state projection and command request resolution.
- `widget::menu::Item` stores `action::Route`; command-facing menu builders project command routes into actions immediately.
- `widget::Presenter` receives action keys/routes and remains command-agnostic; `app::command::Layer` decodes actions back into command routes when presenting menu labels, shortcuts, and availability.
- menu checkmarks are active-only visual content: popup rows carry an active visual state, and paint decides whether to render the active glyph.
- `Application::command_targets` is the single app hook for command execution and target state projection; runtime runs the same declaration in dispatch mode and projection mode.
- first-party text commands use the same app hook through `CommandDispatch::text_buffer`, which dispatches text calls and publishes text command state depending on the runtime mode.

Some path coupling may remain temporarily, but it should become an app/runtime routing detail rather than a core command concept.

## Better Paint Boundary

Paint should consume generic visual state, not command state:

```rust
struct VisualState {
    available: bool,
    active: bool,
    running: bool,
}
```

The command bridge maps:

```text
command::State -> ui::VisualState
```

Then paint only asks, "what is this node's visual state?"

## Feature And Unification Loop

Feature passes optimize for the caller. Unification passes optimize for the
shared concept. Keep those mindsets separate.

1. Build concrete, twice. Ship the new behavior with local machinery against
   one real consumer. Do not extract a general concept until a second real
   caller appears.
2. Hunt for the twin. After the local behavior works, grep for older ad hoc
   machinery that solves the same need. Existing examples are often already in
   the tree: typed text coalescing, command history policy, identity derivation,
   and responder projection all started as local mechanisms.
3. Name the concept first. If the shared shape does not compress to one or two
   words, keep both local mechanisms and wait for more evidence.
4. Retrofit in its own pass. Leave a clear marker when local machinery should
   be reconsidered. Move both callers onto the shared mechanism in a dedicated
   unification pass, not while landing the feature.
5. Make deletion the acceptance test. The old local shape should be removed, and
   preferably become unrepresentable. If the type system cannot enforce that,
   pin the invariant with a regression test.
6. Close with the ownership check. After the move, rerun the design test below:
   no layer should now answer a question outside its ownership.

Current local markers:

- `interaction::TextDraft` is local text-box editing state. Its likely
  twin is document/text-area editing state in the production text engine, but it
  should not be unified until a second control needs the same draft,
  caret, and commit semantics. Scratch text boxes now route pointer clicks to
  draft caret placement through layout hit-testing, but this remains local
  widget state rather than the document editing model.
- The text-engine structural priority from the GTK/Qt comparison is explicit:
  cursor and selection should move out of `text::Buffer` into per-view edit
  session state, and maintained marks should be added as the durable anchor
  substrate for bookmarks, diagnostics, find results, multiple cursors, and
  split-pane editing. Do not deepen framework text widgets around buffer-owned
  cursors before that ownership is corrected. The first migration step is in
  place: `text::edit::State` now groups the cursor mark and selection mark range
  as one value, and `text::edit::Editor` has explicit edit-state entry points
  (`apply_edit`, `apply_edit_with_caret_map`, and `apply_command`). Scratch
  document commands use those explicit entry points. Buffer marker, selection,
  movement, transaction, and mutation helpers now have explicit-state variants,
  and `text::edit::CaretMap` receives the edit state for visual motion. `Buffer`
  still mirrors that state only as compatibility scaffolding for older
  `Buffer`/`Surface` callers; the remaining removal pass is to move those legacy
  callers onto owned edit state and delete `BufferInner::edit_state`.
  Edit transaction vocabulary also lives under `text::edit`: `edit::Transaction`,
  `edit::Delta`, `edit::Kind`, `edit::Change`, `edit::Impact`, `edit::Outcome`,
  and `edit::CommandOutcome`. This is the intended feature-gate boundary for
  applications that need text viewing/layout without compiling text editing.
- `text::Buffer` is now a direct value over its `BufferInner`, not a shared
  mutable handle. Clones share immutable line text, unchanged line-tree blocks,
  copy-on-write add-buffer storage, and stable per-line layout identity. Text
  area shaped caches are keyed by `LineLayoutIdentity`, so undo-restored cloned
  buffers keep warm line display, render, and height measurements despite fresh
  buffer ids. Text-local undo history has moved out of `text::view` and into
  production app compatibility state (`app::text::Driver`);
  framework `History` remains the real framework timeline and already
  coalesces typed text commits and pointer gestures. `text::buffer` temporarily
  mirrors `text::edit::State` for compatibility; new document editing
  and visual motion route through explicit `text::edit::State` instead.
- `text::edit::CaretMap` is the visual-motion seam. `layout::Engine` owns the
  shaped caret fallback now, and command context carries a
  `layout::TextService` handle so document targets can use it without knowing
  about runtime internals. Glyphon input-adapter helpers are gone from
  `text::edit` and glyphon-buffer conversion helpers are gone from
  `text::buffer`; `edit` and `buffer` have no glyphon dependency.
- `text::surface` is now surface data only. Legacy command target/responder
  impls for text surfaces live in `text::command`, and `AreaWrap` to glyphon
  wrap conversion lives in `text::layout`. The legacy adapter module still
  exists for the production framework, but the core text surface no longer
  imports command or glyphon.
- `document` owns its command output shape. Framework edit commands
  return `document::Outcome`; `text::edit::CommandResult` is converted at the
  document boundary and no longer leaks into command registration,
  responder targets, or text-editor observers.
- `clipboard::{Payload, Text}` is the typed clipboard seam with only
  one payload. Representation negotiation should wait for the second real
  payload caller.

## Rewrite Strategy

Start from the data flow, not from file moves.

1. Sketch the new `ui::Snapshot` and `app::command::Layer` APIs.
2. Move current tree indexing into a side-effect-free snapshot builder.
3. Remove registry mutation from snapshot/composition.
4. Move command state projection into `app::command::Layer`.
5. Make paint consume `VisualStateMap`, not `command::Registry`.
6. Move menu command availability into `app::command::Layer`.
7. Remove direct command imports from core `ui::node`, `ui::tree`, and `ui::painting`.
8. Keep command projection at widget/app edges by using `action` primitives inside UI.

## Design Test

A layer is probably wrong if it needs to answer a question outside its ownership:

- Layout asks "can this command run?" Wrong.
- Paint asks "what is in the command registry?" Wrong.
- Command registry asks "what node is focused?" Wrong.
- Runtime asks all of those and delegates to the right engine. Correct.
