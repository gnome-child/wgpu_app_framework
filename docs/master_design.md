# Master Design

This document is the governing design standard for this project. It does not
replace narrower architecture notes such as `docs/ui_command_architecture.md`
or `docs/command_module_organization.md`; it gives those documents their common
rule of judgment.

Every implementation, bug fix, feature, refactor, and module move should be
tested against this document. When code and this document disagree, either the
code is wrong or this document has exposed a belief that must be revised
explicitly.

## First Principles

### Some Things Must Be Clear

Software always rests on basic beliefs. In this project, those beliefs must be
stated clearly enough to examine before they are built on.

The current north star is:

```text
Core concepts are data and contracts.
View trees are declarative data.
Commands are capability contracts.
Targets execute commands.
Runtime orchestrates engines.
Rendering adapts scenes to the platform.
```

The design is wrong when a lower layer has to know a higher layer's reason for
existence. A view tree should not know why an app wants a command. A command
registry should not know which node is focused. A text buffer should not know
which widget is editing it. A renderer should not know application intent.

### Meaning Precedes Implementation

Do not build a shared abstraction until the shared meaning has a name.

Good names in this project are conceptual names: `History`, `Undoable`,
`Retention`, `Payload`, `Target`, `Response`, `Presentation`, `Snapshot`,
`Surface`, `Context`. They are not merely labels for code. They state what kind
of thing exists in the program.

If a repeated shape cannot compress to one or two clear words, keep it local
and gather more evidence. A premature abstraction is usually an unnamed
presupposition with an API.

### Contradictions Should Be Unrepresentable

The preferred fix is not a guard against a bad state. The preferred fix is to
make the bad state not be a state.

Use Rust's type system, module privacy, ownership, enums, typed commands, typed
targets, and narrow constructors to encode the distinction. If a value cannot
be both a command identity and a target identity, model those as different
types. If an operation is only valid after resolution, represent unresolved and
resolved forms differently. If a service is private to runtime, keep the module
private to runtime.

Runtime checks are still useful at IO and dynamic boundaries. Inside the
framework, contradictions should move toward the type system.

### Dependencies Must Tell The Truth

Some parts of the system are conceptually independent. They should compile and
make sense without higher framework layers. Other parts are dependent and
should say so through their imports and module placement.

The text engine should remain an engine, not a widget. Geometry should remain
basic spatial vocabulary. Layout should measure and hit-test. Scene should be
paint data. Runtime is allowed to depend on many concepts because orchestration
is its job.

Do not let a dependent layer pretend to be independent. Do not let an
independent layer import the thing it should be serving.

## Module Organization

The module tree should read as a conceptual map, not as a filing cabinet.

### Governing Shape

The promoted root framework modules are the architectural direction. The old
parallel `src/scratch` namespace and legacy compatibility surface have been
retired; new framework concepts should live in the root module tree according
to the ownership boundaries below.

Example and smoke-test applications live under `examples/`. They may be
included as test fixtures, but they are not framework concepts and must not be
exported as public crate modules. They should compile through the same public
APIs a real external app would use.

The intended dependency direction is:

```text
basic vocabulary
  geometry, theme, text document/buffer/layout/edit primitives

declarative description
  widget builders -> view nodes -> presentation/action data

derived structure
  composition, layout frames, scene primitives, snapshots

runtime state
  session, interaction, diagnostics, clipboard, tasks, timeline

contracts and routing
  command, target, responder, response

orchestration
  runtime, shell, host, platform/native
```

Higher layers may use lower layers. Lower layers must not reach upward to ask
questions owned by higher layers.

### Layer Ownership

`geometry`

Owns spatial facts: points, rects, sizes, areas. It should not know about
widgets, commands, layout policy, scenes, windows, or renderers.

Renderer-space paint geometry is a private adapter vocabulary below the native
renderer. It may support text and GPU preparation internals, but it is not
public framework geometry and should not leak into widget, view, layout, or app
APIs.

`text`

Owns document, buffer, edit, surface, layout, and unicode concepts. The text
engine should be usable without the framework runtime. Editing state belongs to
explicit edit/session values, not secretly to a shared buffer when multiple
views or surfaces can exist.

`widget`

Owns ergonomic builders for view data. Widgets produce nodes. Widgets do not
execute behavior. A widget may project an app-facing concept into declarative
view/action data, but it should not become the runtime for that concept.

`view`

Owns declarative node data, bindings, presentation, style, focus affordances,
and action metadata. View answers "what is being presented?" It should not own
input dispatch, command execution, mutation history, platform rendering, or
task execution.

Visible naming has separate meanings. `interaction::Id` is invisible identity
for reconciliation, hit targets, tests, and runtime lookup. `label` is visible
presentation text and should be painted when the role presents labels. A node
that must be named but not painted should use an id or an explicit
`subject::Segment`, not a hidden label. Subject segments are user-facing
ancestry/grouping vocabulary. Future accessible-name support is additive and
must not collapse these fields back together.

`composition`

Owns the installed view for a window, the retained composition tree derived
from each fresh view description, and the frame-to-frame identity runtime needs
to coordinate. Composition answers "what declarative interface is active for
this window?" and "which presented node is the same node as last frame?"
Retained nodes may hold identity, ancestry, subject segments, and addition or
removal facts for pruning and future accessibility diffs. They must never own
behavior, execute commands, mutate the app model, expose lifecycle hooks, or
perform platform rendering.

`NodeId` is process-transient and never serialized or stored in app state.
Reconciliation v1 is local to each parent: explicit ids survive sibling
reordering, id-less nodes are positional, and cross-parent moves are remove plus
add until a later keyed reparenting design exists. View-only layout helpers use
layout-namespace composition identities, so their node-backed hit targets cannot
collide with retained composition identity. Subject segment names are strings
for grouping, display/debug output, and future serialization, not routing
identity.

`layout`

Owns measurement, frame construction, text measurement integration, and
hit-testing. Layout answers "where is it?" and "what was hit?" It should not
answer "can this command run?" or "what side effect should happen?"
Its measurement contract is constraints down and size hints up: hints are
advisory, constraints are law, and parents place children.
Padding and gap are separate layout concepts: padding is edge inset inside a
container, while gap is spacing between placed children. A style max size is a
layout-visible constraint on measurement and placement; it must not be treated
as a fixed size when content is smaller.

Layout also owns viewports: clipped windows over larger measured content.
Viewport geometry resolves requested scroll offsets into clamped offsets,
content extent, max scroll, and per-axis consumability. Runtime may feed the
resolved offset back into session state, but the geometry that decides whether
a wheel delta can be consumed belongs to layout. Scrollbars are projected
chrome derived from viewport geometry and interaction state; they are not
semantic view-tree widgets. Layout also owns clip propagation: floating
overlays escape ancestor viewport clips by not inheriting those clips, while a
viewport introduced inside a floating panel still clips its own content.
Viewport is internal layout vocabulary; apps ask for scrollable content through
public widget builders, not by constructing viewport geometry directly.
Resolved rectangular clips govern paint, initial hit acquisition, and wheel
targeting. Captured drags may continue outside those clips after capture.
Chrome is projected above its owner's viewport content, but initial chrome
hits still respect the owner's inherited ancestor clip. Rounded panel corners
remain rectangular for both paint clipping and hit testing until rounded clip
support exists.

Viewport reveal is rect-shaped. A reveal request names a viewport and layout
resolves the actual descendant frame rect after composition. The operation is
minimal displacement: a fully visible rect does not scroll, a rect below or
above the viewport aligns its nearest edge with the viewport edge, and an
oversized rect aligns its top or left edge. Reveal margins are viewport
metrics. Nested reveal through ancestor viewports is a later design, not an
implicit side effect.

`scene`

Owns paint primitives and visual presentation data. Scene answers "what should
be drawn?" It should not know the application model, command registry,
interaction routing, or renderer internals.

The renderer may lower scene primitives into a private paint vocabulary for GPU
batching. That vocabulary is not a second public scene API; apps and framework
features should speak in `scene` terms unless they are inside the native
renderer adapter.

Scene clips are paint primitives. Paint applies every resolved frame clip; it
does not decide that a role or layer should ignore clipping. Filters inside
clipped content must be covered by integration tests before being treated as a
stable rendering contract.

`theme`

Owns visual and metric tokens. Theme metrics may affect layout and measurement;
theme appearance affects paint only. Keep those domains distinct when adding
new theme concepts: a scrollbar thickness is layout-visible, while a scrollbar
thumb color is paint-only. Interaction adornments such as hover-thickened
scrollbars are paint-only and must never change measurement or relayout.
Typography follows the same rule: type size, weight, and future tracking are
metrics because text measurement depends on them; text color and opacity are
appearance. Presentation transforms such as command palette section uppercasing
render a label without mutating the stored label, subject, or future accessible
name.

Typography has two anchors. `interface` is compact system/widget text: buttons,
text boxes, checkbox and radio labels, sliders, menus, palette rows, and other
control chrome. `body` is app/content/document text: labels, prose, document
surfaces, and text areas. `caption` and `hint` belong to the interface family
unless explicitly overridden. The word `chrome` is deliberately not the
typography name because layout chrome already means projected surfaces such as
scrollbars. The compact defaults target Win32/iTunes-style desktop menu density
at 96dpi; touch or comfortable density presets are a named future, not a hidden
scale factor.

Shortcut display is presentation. Theme may choose whether shortcuts render as
symbolic controls or text, but it must not change which key event a shortcut
means. Symbolic display uses real scene icon primitives for recognizable keys
such as Control, Shift, Option/Alt, and Command, separated by `+`; weak or
platform-specific keys without a clear icon stay textual. The default shortcut
display is house grammar on every platform: icon modifiers with separators are
a deliberate departure from OS-native menu grammar, while key resolution stays
platform-truthful.

`session` and `interaction`

Own runtime UI state: focus, hover, pressed state, pointer state, open menus,
text input focus, scroll state, and other state that exists because a user is
interacting with a running app. They may produce intents or targets. They
should not execute app behavior directly.

Visual interaction state is paint input. Hover, press, active/selected row
tint, animation phase, and caret blink should be resolved from session/visuals
by retained target identity during paint, not projected into view data as a
reason to rebuild or relayout.

Focus is the keyboard input destination. An active item is a highlighted
descendant operated on by navigation while focus remains elsewhere, such as a
command palette result while the query text box keeps focus. Active items may
request viewport reveal; they do not become focus.

Target labels are debug and presentation data, not identity. Target identity
is its kind, stable id or retained node identity, and routing source. Changing
a target label or capture behavior must not fork hover, scroll, draft, or
focus identity.

`command`

Owns command contracts: typed args, typed output, names, key chords,
availability, history policy, observers, registry metadata, and triggers.
Commands describe what can be asked. They do not decide which concrete target
is currently meant by focus, capture, or app state.

Commands are imperative requests. Past-tense facts are notifications, not
commands. A past-tense command is a classification error because facts have no
availability, no history policy, no registry spec, and no advertised command
surface.

Command shortcuts are semantic data. `Primary+S` means the platform command
modifier, not the physical Control key, and standard roles such as Undo, Redo,
Copy, Save, CloseWindow, and CommandPalette name user intent before platform
resolution. A standard role may resolve to multiple concrete chords; the first
is the display chord and all chords match input.

`notification`

Owns typed framework facts that have already happened. A notification has a
stable name and payload, and a responder may listen to it. Delivery is
zero-to-many: silence is a valid response, and multiple listeners all hear the
fact in responder-chain order.

Notifications are not undoable. Listener mutations should be peripheral state
such as status, caches, and session bookkeeping. If a fact requires a
history-bearing content change, the listener should speak an imperative
command. Internal framework notifications are also distinct from future
OS-facing toasts or user-visible system alerts.

Document dialog-cancel notifications live under `document` in v1 because the
current dialog kinds are document-shaped. Future generic dialogs should let the
opener declare which fact a dialog outcome emits instead of treating that
placement as load-bearing.

`keymap`

Owns platform keymap profiles, shortcut resolution, shortcut formatting, and
text-edit key motion defaults. Keymap answers "which concrete keys does this
semantic shortcut mean on this platform?" and "how should that resolved chord
be displayed?" Profiles are runtime data so macOS, Windows, and Linux behavior
can be tested headlessly on any host.

Resolve and format are separate stages. Matching turns semantic command chords
into concrete key/modifier sets using the active profile. Presentation formats
the resolved chord through the active theme. Menu and palette layout must
measure the same formatted string that paint draws; semantic declarations such
as `Primary+S` and `Standard::Undo` must never leak directly to pixels.

`state`

Owns model storage, snapshots, revisions, change reasons, and committed state
transitions. State answers "what changed?" and "which model value is current?"
It should not know how a widget painted, which native event arrived, or which
renderer will draw the result.

`target` and `responder`

Own executable capability and routing participation. A target says "this value
can perform this command." A listener says "this value wants to hear this
notification." A responder chain says "these are the current places where a
request may be answered or a fact may be delivered." They should stay typed at
the edges and erase only inside routing machinery.

`response`

Owns the result vocabulary for command handling: changed state, effects,
follow-up work, and output. Responses describe what happened and what must be
scheduled next. They should not perform platform work by themselves.
Invalidation effects merge by maximum depth: `Paint < Layout < Rebuild`.
`Paint` changes pixels only, `Layout` recomposes frames from retained
composition, and `Rebuild` rebuilds/projects the view and reconciles
composition.

`timeline`

Owns undo and redo history. History is a framework concept, not incidental app
bookkeeping. Command history policy should route through the timeline instead
of each feature inventing local undo semantics.

`runtime`

Owns orchestration. Runtime may know about state, timeline, session,
composition, layout, diagnostics, clipboard, tasks, command registry,
responders, gestures, theme, view callbacks, and platform-facing work because
coordinating those engines is its purpose. Runtime should delegate actual
domain questions back to the owning engine.

Runtime owns coarse invalidation scheduling. Paint-only frames reuse the cached
window layout when size and theme still match; layout invalidation refreshes
transient projection and recomposes without rebuilding the view; rebuild
invalidation runs the full view projection and composition reconciliation path.

`platform`, `host`, and native/render adapters

Own the boundary with the operating system, window system, GPU, renderer,
clipboard, dialogs, and native event loop. Renderer dependencies belong at this
edge unless a lower rendering vocabulary is explicitly being defined.

### Public API Rule

Central concepts are re-exported. Supporting concepts stay namespaced.
Internal routing details stay private.

This rule keeps call sites readable while preserving context for narrower
types. A type should move toward a module root only when it is part of that
module's public meaning, not merely because many files mention it.

## The Implementation Protocol

Use this sequence for every non-trivial change.

1. State the ownership claim.

   Before coding, be able to say which concept owns the behavior and why. If
   two layers both seem to own it, the concept is not yet clear.

2. Name the meaning.

   The name should describe the thing, not the current implementation trick. If
   the name sounds like a utility, helper, manager, data bag, or catch-all, keep
   looking.

3. Place it at the lowest honest layer.

   Put the concept where all real consumers can use it without importing a
   higher reason for existence. Do not move it lower than its meaning allows.

4. Make invalid states impossible.

   Prefer types, privacy, constructors, enum variants, and ownership boundaries
   over scattered boolean checks and "remember to" comments.

5. Keep feature work and unification work separate.

   A feature pass may be local and concrete. A unification pass extracts the
   shared concept after a second real caller proves it exists.

6. Delete the old shape.

   Refactors are incomplete while the previous mechanism still exists as an
   alternative path. Deletion is the acceptance test for clarity.

7. Practice the belief with tests.

   Add behavioral tests for user-visible semantics and architecture tests for
   import boundaries or forbidden stale concepts when the boundary is important
   enough to preserve.

## Repetition Is A Design Signal

Repeating logic is usually not just duplication. It often means a more basic
concept was placed too high, too late, or under the wrong name.

When repetition appears, ask:

- Are these sites solving the same conceptual problem or only looking similar?
- Which layer can own the common meaning without importing a higher layer?
- Would moving the concept lower remove knowledge from the callers?
- Can the old local forms be deleted after the move?
- Can the type system make the repeated mistake impossible?

Do not deduplicate by creating a miscellaneous helper. A helper removes lines.
A concept removes confusion.

Keep duplication temporarily when the meaning is not yet known. But mark the
pressure clearly and revisit it after a second real caller proves the shape.

## Boundary Tests

The fastest design review is to ask what question a layer is answering.

Correct questions:

- Geometry asks: what are the spatial facts?
- Text asks: what is the document, edit state, surface, and text layout?
- View asks: what declarative interface is being presented?
- Layout asks: where are the presented things?
- Scene asks: what primitives should be drawn?
- Interaction asks: what is the user currently doing?
- Command asks: what capability contract exists?
- Target asks: can this value perform this command?
- Timeline asks: how is state restored?
- Runtime asks: which engines must be coordinated for this event?
- Platform asks: how does this leave or enter the process?

Wrong questions:

- Geometry asks which widget owns a rect.
- Text buffer asks which control is focused.
- View asks whether a command can execute against current app state.
- Layout asks what command should be dispatched.
- Scene asks what the command registry contains.
- Command registry asks what node is under the pointer.
- Timeline asks how a text glyph should be shaped.
- Renderer asks what app command produced a primitive.

The runtime may ask many questions, but it should not answer all of them
itself. It should ask the owning engine, compose the answers, and commit the
result.

## Smell Catalog

Use these as prompts during review.

`Repeated logic`

A basic concept is probably too high, unnamed, or split across callers.

`Bridge leak`

A bridge type or adapter detail has crossed into a core layer. Move it back to
the edge or name the missing core concept.

`Upward import`

A lower layer imports a higher layer to answer a question outside its ownership.
This is almost always architectural debt.

`Boolean protocol`

Multiple booleans encode modes that should be an enum or state-specific type.

`Stringly identity`

Strings or raw ids are standing in for typed identity. Give the identity a
domain type unless the boundary is truly external.

`Utility bucket`

A module or type named as a helper, manager, common, misc, or util usually
means the concept has not been named yet.

`Optional field cluster`

A struct with many related `Option` fields often hides variants that should be
separate enum cases.

`Runtime state in declarative data`

Focus, hover, pressed, capture, scroll, text session, and open popup state
belong to runtime/session/interaction unless deliberately projected for one
frame.

`Paint with policy`

Paint code should render resolved visual facts. It should not decide command
availability, app behavior, or mutation policy.

`Compatibility hardening`

Temporary compatibility scaffolding starts to receive new features. Instead,
finish the migration or keep the new behavior in the intended architecture.

## Refactor Standard

A refactor is successful only when it improves the truthfulness of the design.
Moving files without changing ownership clarity is churn.

For each refactor, record or be able to explain:

- The old belief the code was practicing.
- The new belief the code should practice.
- The concept that owns the behavior now.
- The imports or public API that prove the boundary.
- The obsolete path that was deleted.
- The test that will fail if the old confusion returns.

If the refactor cannot delete anything, cannot narrow an API, cannot remove an
upward dependency, and cannot make a bad state harder to represent, it may not
be a refactor worth doing yet.

## Tests As Practiced Architecture

Tests are not only output checks. They are executable integrity.

Use tests to preserve:

- User-visible behavior.
- Command dispatch and availability contracts.
- Undo, redo, coalescing, retention, and transaction semantics.
- Text editing and layout invariants.
- Accidental restoration of `src/scratch` or the retired legacy root surface.
- Renderer and platform dependency boundaries.
- Private runtime services that should not become public vocabulary.
- Deletion of stale concepts after unification.

Architecture tests should be used sparingly but decisively. If a boundary has
already failed once and matters to the design, encode it.

## Design Review Checklist

Before merging a substantial change, answer these questions:

- What concept did this change add, remove, or clarify?
- Does the concept have a clear name?
- Is the concept in the lowest honest layer?
- Did any lower layer learn about a higher layer's reason for existence?
- Did runtime orchestrate, or did it absorb domain behavior?
- Did a bridge remain at the edge?
- Did repeated logic move into a real concept instead of a helper bucket?
- Did the type system rule out the bad state where practical?
- Was the obsolete path deleted?
- Is the boundary practiced by tests?

## Relationship To Existing Docs

`docs/ui_command_architecture.md` applies this document to the view, runtime,
and command boundary. Its north star remains authoritative for that seam:

```text
Node tree is data.
Composition is retained identity.
Layout is derived geometry.
Interaction is runtime state.
Commands are capability contracts.
Runtime wires them together.
```

`docs/command_module_organization.md` applies this document to command module
API shape:

```text
Central concepts are re-exported.
Supporting concepts stay namespaced.
Internal routing details stay private.
```

Future architecture notes should be written the same way: state the basic
belief, name the concepts, define ownership, show dependency direction, list
forbidden questions, and identify the tests that practice the boundary.
