# View And Command Architecture Boundary

This note applies `docs/master_design.md` to the seam where declarative view
data, runtime interaction state, and command contracts meet.

## North Star

```text
Node tree is data.
Composition is retained identity.
Layout is derived geometry.
Interaction is runtime state.
Commands are capability contracts.
Runtime wires them together.
```

The important boundary is not file placement alone. It is which layer answers
which question.

## Ownership

### `widget`

Widgets are builders. They turn ergonomic app calls into `view::Node` data.
They may project command-backed controls into generic view bindings, but they do
not execute commands, mutate app state, own focus, or perform routing.

### `view`

View owns declarative node data:

- roles and children;
- style, labels, subjects, and ids;
- focus affordances and active rect declarations;
- command-agnostic bindings and actions;
- surface metadata needed by layout and paint.

View may describe that something can be acted on. It must not decide whether a
command can currently run against app state.

### `composition`

Composition owns the installed view and retained node identity. It answers
"which node is this across frames?" and "what ancestry does this node have?"
It does not execute behavior. Runtime uses composition identity to key
ephemeral UI state, prune removed nodes, resolve containment, and derive subject
paths for presentation.

### `layout`

Layout owns measurement, frame construction, viewport geometry, projected
chrome, clips, and hit testing. It answers "where is it?" and "what was hit?"
It should not ask the command registry or app model whether an action is
available.

### `session` and `interaction`

Session and interaction own runtime UI state:

- focus and active item;
- hover, press, and pointer capture;
- text draft state;
- scroll offsets and reveal requests;
- open menus and command palette state.

They may produce intents and targets. They do not execute app behavior.

### `command`

Command owns contracts:

- typed args and output;
- names, labels, shortcut roles, and history policy;
- registry metadata;
- claim state and triggers.

A command definition says what can be asked. It does not know which node is
focused, which popup is open, or which concrete app object should answer today.

### `target` and `responder`

Targets execute typed commands. Responder chains describe current routing
participation. They bridge command contracts to concrete app/framework
capabilities while keeping execution typed at the edges.

### `runtime`

Runtime is the orchestrator. It is allowed to know about view callbacks,
composition, layout, session, commands, responders, text services, clipboard,
timeline, tasks, diagnostics, theme, and platform-facing work because its job is
coordination.

Runtime should coordinate the seam, not absorb every domain question. If a
question has a lower owner, runtime asks that owner and composes the result.

## Current Data Flow

```text
model + view context
  -> widget builders produce view::View
  -> runtime resolves command state through responders
  -> composition reconciles retained identity
  -> layout measures and places frames
  -> scene paints from layout, theme, and visual interaction state
  -> platform/native adapts scene to render
```

Command availability is resolved before installation by runtime using responder
chains and the command registry. Layout and scene receive generic presentation
state; they do not import command routing.

Input follows the opposite direction:

```text
platform event
  -> shell/host input
  -> runtime hit tests layout
  -> session updates interaction state
  -> runtime invokes view actions or command triggers
  -> responders execute typed targets
```

Hit testing may identify a retained target. It does not dispatch commands by
itself.

## Boundary Rules

- `widget` may import command types only where it is explicitly projecting a
  command-backed builder into command-agnostic view data.
- `view` stores binding/action data, not command registry state.
- `composition` stores identity and ancestry, never behavior.
- `layout` may see targets and active rects, but not command availability.
- `scene` paints visual state, but does not ask why that state exists.
- `command::Registry` must not ask for a view tree, layout frame, or focus.
- `runtime` may ask all of these layers for their owned facts and then route.

## Palette And Menu Proofs

Menus and the command palette are the highest-pressure consumers of this seam.
They exercise command enumeration, shortcuts, subject ancestry, retained
identity, floating panels, viewport clipping, active item reveal, and row
highlight paint.

The intended split is:

- command registry and responders decide which commands are visible and enabled;
- keymap formats resolved shortcuts before layout and paint;
- composition ancestry gives user-facing subject sections;
- layout owns popup/palette placement, result viewport geometry, and chrome;
- scene paints row highlights and shortcut text from shared presentation tokens;
- runtime invokes selected commands against captured focus.

If a menu or palette fix requires command knowledge inside layout or scene, the
fix is in the wrong layer.

## Text Editing Boundary

The text engine is a lower engine, not a widget. `text::Buffer` owns document
text. `text::selection::State` owns the persistent cursor and selection marks
used by both selection and editing. `text::view::ViewState` owns ordinary
surface presentation facts: scroll, caret blink, reveal intent, and preferred
caret position. `text::Preedit` is the immutable composition projection shared
with layout; draft input owns its target and lifetime and passes it separately
only while composition is active. `text::edit::Editor` applies mutations
against an explicit buffer and selection state.

Framework widgets and document commands should pass explicit edit state through
the text engine. They should not reintroduce hidden buffer-owned cursor state,
store composition inside ordinary view state, or add widget-local text editing
logic.

## Remaining Pressure Points

- Keep command-backed widget builders as projection boundaries. Do not let
  generic view nodes become command registry clients.
- Keep palette/menu row presentation shared only at the row-highlight and flow
  math level. Menus and palettes are different surfaces with different meaning.
- Keep renderer reports out of the core API until render diagnostics have a real
  runtime consumer. Statistics may remain where render preparation actually
  consumes or returns them.
- Continue deleting stale compatibility paths after each unification pass. A
  refactor is not complete while the old route still works.

## Design Test

A layer is probably wrong if it needs to answer a question outside its
ownership:

- Layout asks "can this command run?" Wrong.
- Scene asks "what is in the command registry?" Wrong.
- Command registry asks "what node is focused?" Wrong.
- Runtime asks all of those owners and coordinates the answer. Correct.
