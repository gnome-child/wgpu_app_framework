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

Paint-space geometry belongs to the private `paint` module. It is the f32,
device-grid-aware vocabulary used by text layout, paint conversion, and GPU
preparation internals; it is not public framework geometry and should not leak
into widget, view, layout, or app APIs. `paint::area::Logical` and
`paint::area::Physical` remain distinct types so DPI unit safety stays enforced
by the compiler.

The layout-to-paint boundary is a geometry boundary. Layout frames use integer
logical coordinates. Paint uses floating logical coordinates because a
device-pixel-aligned edge may be fractional in logical space at scale factors
such as 125%. Boundary conversion multiplies by the active scale factor, rounds
in device-pixel space, and divides back to floating logical coordinates. Do
not snap by rounding layout coordinates in integer logical space.
Exact half-device-pixel ties round toward zero. This is a deliberate style
choice for thin geometry: a 1 logical px line at 150% should stay 1 device px
rather than becoming heavier. The same midpoint rule applies to edge positions
and snapped distances. Use `Rule` when an axis-aligned UI hairline needs exact
physical-pixel thickness.

Snapping has two decoration species. Positional boxes snap absolute edges for
closure with neighboring boxes. Relative decorations snap the base first, then
snap their own distances from that base for symmetry. External focus rings use
this rule: snap the base rect, snap `offset`, snap `width`, then derive inner
and outer ring edges. Do not snap `offset + width` as one expanded rect. The
internal ring path already follows the same distance-first law by deriving its
inset from a snapped base edge and a snapped width.

`window`

Owns `Facts`: id, title, inner size, canvas color, and kind. Session, shell,
host, and platform window types remain layer projections that wrap this core;
they must not redeclare its fields.

`color`

Owns transfer functions and named byte conventions. Scene RGB bytes are sRGB,
paint RGB floats are linear, glyphon color bytes are sRGB, and Windows accent
gradients are packed as `AABBGGRR`; conversion happens once at these boundaries.

`text`

Owns document, buffer, edit, surface, layout, and unicode concepts. The text
engine should be usable without the framework runtime. Editing state belongs to
explicit edit/session values, not secretly to a shared buffer when multiple
views or surfaces can exist.

Text layout owns shaped-buffer cache mechanics through `ShapingCache`; area
lines, field surfaces, and inline text/icons supply domain keys and retention
limits, while the shared owner mediates lookup, insertion, and `FontSystem` use.

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

Hit testing v1 reads layout truth. At fractional scale, painted device-snapped
edges and integer layout hit edges may differ by less than one device pixel;
that tolerance is accepted until a real spatial-presentation animation needs
presentation-space hit acquisition. When that caller exists, pointer hits
should follow the visible presentation position, while keyboard focus, reveal,
caret geometry, and scroll remain layout-space concepts.

The OS pointer cursor is a promise about what a click would do. Editable text
surfaces use the text cursor only where a click can place or drag a caret or
selection; painted labels, menu rows, buttons, palette rows, chrome, and
disabled fields keep the default cursor. Cursor resolution consumes the same
clip-aware hit truth as pointer clicks, wheel targeting, and paint, so hidden
or occluded text must not leak an I-beam through overlays.

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

The renderer may lower scene primitives into the private `paint` vocabulary for
GPU batching. `paint` is the flattened display-list seam between retained scene
and backend rendering. It is not a second public scene API; apps and framework
features should speak in `scene` terms unless they are inside the native
renderer adapter.

Renderer queue writes apply before encoded passes execute. A shared GPU buffer
written once per batch is therefore last-write-wins for the whole submitted
frame, not a sequence of interleaved "update then draw" operations. Per-batch
values, such as glyphon viewport resolution for base text versus promoted
overlay text, must ride per-batch buffers.

Scene clips are paint primitives. Paint applies every resolved frame clip; it
does not decide that a role or layer should ignore clipping. Filters inside
clipped or promoted content sample from the accumulated backdrop beneath the
current layer, then write their result into the current local target. A filter
inside a layer must not silently skip itself because it is no longer drawing
directly to the main target.

Scene is also the presentation-space boundary. The doctrine is: layout is
snapped, presentation is continuous, and animation is presentation. Resting
layout geometry is snapped once when integer layout rects become floating
paint rects at the active device scale. Presentation transforms may then move
or scale that snapped geometry continuously while their `Motion` is `Moving`.
When motion stops, the pose is geometry again: a `Resting` transform is baked
into the paint rect and snapped, even if that resting pose is a held hover
plateau rather than the base state. Paint call sites forward motion state from
runtime visuals; they do not invent their own answer to whether a transition
is still moving.

Resting targets are chosen in snapped terms before motion begins. Moving
presentation interpolates between snapped resting truths, so the final moving
frame and the first resting frame describe the same paint-space pose. Runtime
visuals may carry current value, target value, progress, and motion, but device
scale and snapping remain owned by the layout-to-paint boundary.

Renderer-local snapping is witness vocabulary, not a second source of truth.
Resting quad geometry is asserted aligned during render preparation because it
should already have been snapped at the layout-to-paint boundary. Axis-aligned
UI hairlines are `Rule` primitives, not quads with special snapping. A rule
snaps its span edges for closure and keeps its declared physical-pixel
thickness because thickness is its meaning: menu separators are horizontal
rules, text carets are vertical rules. Explicit moving presentation remains
continuous; do not add another primitive-local snap policy when the boundary
snap should own the fact. A permanent unsnapped transform is not a default
behavior; a future caller must earn and name that variant explicitly.

`overlay`

Owns floating UI entries above the main scene. An overlay entry is live UI:
it has retained identity, order, scene primitives, opacity, and input semantics
through the normal layout and interaction systems. Per-entry buckets are the
contract boundary for overlay backends: `InFrame` paints the entry inside the
parent scene, while `NativePopup` paints the same entry bucket into a separate
OS popup window when the backend capability probe supports it. Native popup is
a backend realization, not a framework window in app/session state; parent
window focus and command routing remain authoritative. Unsupported platforms
fall back to `InFrame`. Wayland is fallback-only in v1 because arbitrary global
popup placement is not a portable winit capability.

Native popup windows are popup shell surfaces: undecorated, transparent,
initially hidden until positioned and sized, absent from taskbar/dock-style
shell presence where the platform allows it, and invisible to framework
app/session window state. Mixed-DPI correctness is per-window: a native popup
uses the popup window's own scale factor for paint conversion, not the parent
window's scale. Overlay bounds are parent client-area coordinates, so native
popup placement anchors to the parent window's client-area screen origin
(`inner_position`) and only falls back to outer window origin when the platform
cannot report the client origin.

Popup shell semantics belong to `window::Kind::Popup`, not ad hoc overlay
call sites. A popup is owned by its parent where the platform supports
ownership, is non-activating after creation as well as during creation, and is
shown or positioned through no-activate paths when the platform exposes them.
Popup geometry is event-driven: creation, anchor/bounds changes, parent
movement, parent resize, and scale changes may configure position or size, but
opacity/fade redraws are draw-only. Parent close/minimize/focus-loss may close
or suspend popup entries through overlay policy, not by turning the popup into
a framework application window. Per-frame popup repositioning is forbidden.
Native diagnostics may report desired, applied, and observed shell geometry,
but window-manager disagreement must not create a correction loop. Some tiling
window managers may still need user ignore rules; the framework's duty is to
emit correct popup/tool-window signals before documenting that limitation.

Native popup input is a coordinate adapter into the parent interaction truth,
not a second interaction model. Pointer movement, buttons, and wheel events
from a popup window are converted from popup-local physical coordinates to the
parent window's logical overlay coordinates by using the popup window scale
factor and the entry bounds. The parent window remains authoritative for focus,
commands, keyboard routing, diagnostics, and session state. Pointer cursor
application currently targets framework windows; popup-hosted text fields will
need the cursor side effect to target the physical window under the pointer.
Native popup lifetime is synchronized only by an authoritative overlay
presentation pass: no popup presentation statement means leave existing popups
alone, while an authoritative empty popup set means close stale popups.
Native popup text editing has two named v1 seams: IME candidate windows still
need popup-local caret anchoring, and pointer cursor application still needs to
target the physical popup window under the pointer. Until those seams are
implemented, popup-hosted text input is not considered complete.

Intent is portable; realization is native. `Material::Glass` means "glasslike
panel material"; an in-frame backend realizes it by sampling the parent
composition, a native popup backend realizes it with OS window material when
the platform and surface alpha support it, and native fallback realizes it as
an opaque readable body when neither backdrop path is available. Theme/config
should express portable intent and platform-scoped realization choices, never a
flat optional cluster of OS-specific fields.

Overlay backend choice follows window capability, not material identity. A
floating panel that prefers `NativePopup` uses it whenever the platform probe
supports native popup windows; unsupported platforms fall back to `InFrame`.
Material realization is backend-local: in-frame panes may sample parent
composition, while native popup windows own backdrop material, corner shape,
and shadow. Native popup scenes must not contain framework glass panes; the
framework renders content and interaction visuals into a transparent
premultiplied popup surface so OS material can show through. If the popup
surface cannot support that alpha mode, the backend logs the downgrade and
renders an opaque native-safe fallback scene, still without framework glass.
All floating panels therefore follow the same backend path, with material
differences handled below the backend seam.
On Windows, documented DWM system backdrop tracks activation state. A
`NOACTIVATE` popup is permanently inactive for that material and receives the
solid fallback color even when its owner window is focused. System backdrop may
still be valid for future activation-capable utility windows, but not for
nonactivating popup overlays. Windows popup glass therefore uses the
focus-independent accent policy (`SetWindowCompositionAttribute` with
`ACCENT_ENABLE_ACRYLICBLURBEHIND`) behind the native sys seam. The accent
`GradientColor` is ABGR/AABBGGRR and comes from the popup material tint, so tint
alpha remains a theme/material dial rather than a platform constant.
OS-side realizations are settle-rate, not event-rate. Geometry, accent material,
future border color, and similar native attributes are desired state with an
applied snapshot; they coalesce to the latest value and cross into the OS only
after a meaningful geometry change, material-presence change, or short settled
quiet period. Drag-rate parameter changes must not build a queue of native
compositor calls.
Windows popup acrylic is not tied to the DX12 DirectComposition Visual path:
Vulkan redirected popups can realize accent acrylic when the surface reports
premultiplied alpha. The backend mask therefore stays `wgpu::Backends::all()`
and `WGPU_BACKEND` remains the A/B lever. DX12 `DxgiFromVisual` stays available
for future composition-backed windows and targeted diagnostics, but it is not
the default requirement for popup acrylic. `CompositionBacked` still means the
DX12 visual path plus `WS_EX_NOREDIRECTIONBITMAP`; `RedirectedFallback` keeps the
redirection bitmap, requests premultiplied alpha, and may use OS acrylic if the
reported alpha mode supports it. Premultiplied surfaces require premultiplied
content: alpha diagnostics must use a real half-alpha primitive or
premultiplied clear, never a straight-alpha clear as evidence. The authoritative
alpha witness is a standalone primitive over a transparent clear with readback that proves both alpha and premultiplied RGB; clear-only witnesses and visuals
nested inside panel body content are contaminated evidence.

Windows premultiplied popup surfaces use a different final pass than ordinary
opaque app windows. The scene renders into an sRGB offscreen target using the
normal linear renderer. The final popup pack pass samples that scene, converts
straight RGB with the exact piecewise sRGB transfer function, re-multiplies by
alpha, and writes with `REPLACE` into a non-sRGB premultiplied popup surface.
The legacy composition-texture blit remains for opaque/default app windows; it
must not be reused as the Windows popup handoff. This replaced the earlier
direct-surface pin: that pin was correct before the sRGB/premultiplied boundary
was understood, but it is now obsolete.

`native_alpha_probe` is the
permanent Windows instrument for backend, accent, and popup attribute bisection:
start with a boring transparent window, compare Vulkan against DX12
`DxgiFromVisual`, test single popup attributes first, and only then test
suspicious pairs such as owner+toolwindow or no-redirection+backdrop.

Native popup foreground defects must be partitioned before fixing: alpha
convention, color-space/gamma, and scale/stretch can all look like "crusty"
foreground pixels but require different repairs. Foreground witnesses must
include fractional coverage from antialiased quads and glyph masks, not only
solid interior pixels. Visual comparison starts with the same foreground over
`OpaqueFallback`, transparent/no-accent, and OS acrylic; if the opaque fallback
is also crusty, scale and surface sizing are the first suspects. Native popup
scale diagnostics report the whole chain: scene logical bounds, requested
popup bounds, observed inner size, canvas physical area, surface config size,
and popup scale factor. The foreground clarity fixture compares the same content
over an opaque backing strip that uses the in-frame panel surface color and over
the unbacked native material. The backed row must match the in-frame reference
before unbacked crust can convict native-boundary blending. The six-cell manual
matrix is `OpaqueFallback`, transparent/no-accent, and acrylic, each checked in
backed and unbacked form; crust in no-accent and acrylic unbacked rows points to
general DWM/native boundary blending, while crust only under acrylic points to
the accent layer. The foreground rows must include the real states where defects
are easiest to see: disabled menu bindings with shortcut glyphs and live sliders
whose hover/drag animation exercises the actual slider paint path.

Native popup enter-fade stays disabled until the premultiplied-alpha/group
blend audit. Menus can ship before that because their content uses the safe
alpha extremes: opaque rows/text over transparent window gaps. Fade makes
ordinary content semi-transparent and therefore depends on the same
premultiplied convention that future local blur also requires.

Overlay ghosts are paint-only afterlife. When a live in-frame entry is
dismissed, runtime may retain its final scene bucket briefly as a `Ghost` for
fade-out, but the ghost is not layout, hit testing, wheel targeting, cursor
resolution, focus routing, dismissal containment, semantics, or command
routing. Native popup entries do not allocate ghosts in v1; the native surface
closes rather than teleporting a fading afterimage back into the parent
window. Focus restoration and key routing update when the live entry is
dismissed, not when the ghost expires. Ghost fade frames are presentation work
and must not imply model revision changes. If an in-frame ghost contains a
material pane, the pane is downgraded to paint-only material layers; ghosts
keep body, tint, and grain, but they do not backdrop-sample the world.

Ghosts capture scene primitives, not paint primitives, so DPI and scale changes
during a fade still pass through the normal layout-to-paint boundary. Ghost
paint order is the departed entry's original order; a freshly reopened entry
receives a newer order and paints above its own fading ghost. Dismiss and
immediate reopen may therefore show a non-interactive fading ghost behind a
fresh live entry, which is intended.

Overlay opacity is group compositing, not per-primitive alpha. A fully opaque
entry renders inline and costs no offscreen target. An entry whose opacity is
between 0 and 1 is promoted into an offscreen group: render the entry's panes,
panel chrome, text, icons, shadows, and rounded edges into a local transparent
target, then composite that target back once with the group opacity. Opacity 0
skips rendering.

A material is a visual recipe; a pane is shaped material. Glass is a UI
material operation, not a list of unrelated blur, luminosity, tint, and noise
draw items that happen to agree. `scene::Pane` and `paint::Pane` carry the
pane rect, rounding, and material recipe until render, where one material
context owns backdrop color, local material state, shape coverage, scratch, and
group opacity. Renderer-internal generic filter chains remain available for
non-material image operations and future local blur, but floating-panel glass
does not flatten into generic filter primitives or expose a generic filter
display-list front.

Group bounds are paint-space visual extents, not just the retained entry rect:
they include shadows, filter spreads, blur radii, and other pixels the entry
owns. The group source rect is local texture space while the destination rect
is the global entry bounds; confusing those is a coordinate bug that can stretch
or zoom the accumulated scene during fades.

Backdrop source space and local output space are separate axes. The first
backdrop sample may read from the global accumulated composition, but every
intermediate filter scratch target after that is local to the current target.
This is another `Axis Splitting` case: full-window composition is accumulated
scene truth, while target-local ping/pong scratch is filter-chain workspace.
Material anchoring is a separate axis again: backdrop material layers sample
source space, while surface layers such as noise use panel-local material space
so grain rides with the glass instead of the world.

Material and filter pass parameters derive from one context authority that owns
the backdrop source, local source, target scratch, and global-to-local mapping;
individual passes must not re-thread bare source and target rects as independent
truths. A material/filter chain has one world; a pass that derives source,
target, or coverage from outside the context is a migration bug.

Local target dirtiness is also a separate axis from backdrop truth: inside a
promoted group, a prior local primitive such as a shadow does not make the local
transparent target the backdrop for glass. Backdrop layers sample the
accumulated parent scene; local surface layers sample the material built inside
the target so far.

Material coverage is the panel shape, not sampled source alpha; source alpha
belongs to layer and group composite-back. Shape-mode material filters use
source RGB independently from source alpha so a transparent group target cannot
erase backdrop blur, luminosity, or other glass material.

Backdrop blur needs target-local scratch padding for the kernel reach; group
bounds reserve that spread before the filter chain writes into local ping/pong
textures. Temporary filter layer and scratch texture pools are retention-capped
at eight entries each and report their current sizes through render diagnostics.
Temporary group targets use the renderer alpha convention
consistently: primitives draw into a transparent target, and the group composite
samples and re-applies opacity as one image so text, rounded edges, shadows, and
backdrop effects do not separate. A full premultiplied-alpha/group-blend audit
is scheduled follow-up work, not an optional maybe, because group compositing has
now exposed multiple alpha-convention seams. Local blur is future work on the
same filter-chain context seam, but it must wait for that audit because blurring
transparent local content under straight alpha creates dark halos.

Reduced motion and accessibility policy can set zero exit duration to skip
ghost allocation entirely.

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
of each feature inventing local undo semantics. Runtime scopes a command's
coalescing declaration to its window and focused target before timeline reuse.

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

`diagnostics`

Owns framework-visible counters and sample windows that turn performance and
interaction reports into numbers. Diagnostics are not behavior inputs; they are
instrumentation read by tools, tests, and debug panels.

The text editor debug panel is the current full instrument panel. It shows text
layout work (paint calls, metric calls, visible and shaped logical lines,
layout plus overscan segments, interaction surfaces, highlight scans), text
caches (line hits/misses, render-surface calls and cache hits/misses, render
source lines and bytes), scroll work (wheel events, offset changes, redraw
requests, committed frame scrolls, text area viewports), frame work (full
redraws, view rebuilds, layout recomposes and reuses, text surfaces), and
render work (presented frames, frame-interval p95, surface-acquire p95, draw
p95, `key->present` p95, pending key samples, promoted group composites, and
retained filter layer/scratch pool entries).

The instrument map is:

| Instrument | Owner | Signals |
| --- | --- | --- |
| `Text layout` | `text::layout`, `text::edit` | paint calls, metric calls, visible and shaped lines, overscan segments, overlay and highlight work |
| `Text caches` | text layout caches | line hits/misses, render-surface calls, render-cache hits/misses, render source lines and bytes |
| `Scroll` | interaction and text viewport services | wheel events, offset changes, redraw requests, committed frame scrolls, text area viewport work |
| `Frames` | runtime presentation | full redraws, view rebuilds, layout recomposes/reuses, text surfaces |
| `Render` | native renderer and `diagnostics::Render` | frames, interval/acquire/draw p95, `key->present` p95, pending key samples, promoted groups, filter pool sizes |
| `wgpu_l3::render::filter_params` | filter encoder | filter pass uniforms and source/target rects |
| `wgpu_l3::render::material` | pane material path | pane source/target facts and material layer sequence |
| `wgpu_l3::overlay::fade` | overlay runtime | entry opacity, schedule, frame number, and demotion timing |
| `wgpu_l3::overlay::backend` | overlay runtime | entry material realization, backend preference, resolved backend, and fallback capability flags |
| `wgpu_l3::native_popup` | native platform | popup shell style, geometry, routing, and native-window lifecycle decisions |

Render latency samples are revision-tagged: a key/input sample records only
when the presented frame revision includes the state change it produced.
`key->present` means input-to-present-call, not input-to-glass.

Compositor investigations use narrow debug log targets and must stay quiet
under the example default `RUST_LOG=info`. Current targeted debug channels are
`wgpu_l3::render::filter_params` for filter pass uniforms,
`wgpu_l3::render::material` for pane material source/target facts, and
`wgpu_l3::overlay::fade` for overlay opacity, schedule, and demotion timing.
Native popup and backend-choice questions use `wgpu_l3::native_popup` and
`wgpu_l3::overlay::backend`.
Use targeted `RUST_LOG=wgpu_l3::render::material=debug` style filters for
diagnosis instead of raising the whole app to debug.

`platform`, `host`, and native/render adapters

Own the boundary with the operating system, window system, GPU, renderer,
clipboard, dialogs, and native event loop. Renderer dependencies belong at this
edge unless a lower rendering vocabulary is explicitly being defined.
Native paint adapters carry the window scale factor into layout-to-paint
conversion so monitor moves and fractional DPI changes re-snap the same layout
truth to the new device grid.

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

## Answer Catalog

Use these as repair moves during design and review. They are companions to the
smells above: a smell names what is wrong, while an answer names the shape of a
good fix.

`One Truth, One Owner`

Fallacy: each subsystem may compute its own copy of a shared fact. Answer:
move the fact to the lowest honest owner and make every other subsystem
consume it. Layout owns viewport clips for paint, hit testing, and wheel
targeting. Viewport geometry owns scrollbars, reveal, and scroll consumption.
The paint `Grid` owns device-scale snapping.

Enforce by deleting parallel computations. Add tests only where a duplicate
has already returned or where the owner boundary is easy to regress.

Render `Surface` owns surface configuration epochs: a suboptimal acquire records
deferred reconfiguration, and render completion applies it only after presenting
and releasing the outstanding surface texture.

`Witness Demotion`

Fallacy: a displaced mechanism should keep doing part of the old job. Answer:
when a lower truth replaces a computation, the old mechanism either disappears
or becomes a checker for the new truth. Render preparation may assert that
resting geometry is aligned; it must not re-decide how layout-to-paint snapping
works. Architecture tests may guard the deletion of retired modules and aliases.

Enforce by asking of every survivor: is this a witness, or is it dead code?

`Axis Splitting`

Fallacy: one concept or word can straddle two independent questions. Answer:
split the axes so each type, name, or field answers one question. Labels are
visible text; ids are invisible identity. Commands are imperative requests;
notifications are past-tense facts. Theme metrics affect measurement;
appearance affects paint. Logical and physical paint areas stay distinct types.
Positional boxes and relative decorations use different snapping rules because
closure and symmetry are different goals.
Backdrop color, local material, shape coverage, and group opacity are separate
axes owned by the pane material context, not one filter alpha channel or dirty
flag.

Enforce with type separation, module placement, and names that state the axis.
Repeated words are not automatically wrong; they become naming debt when the
meanings co-occur in one scope and force aliases or reader ambiguity. Current
known overloaded terms include `Presentation`, `Frame`, and `Surface`; rename
them only when a concrete import-scope collision proves the ambiguity.

`Structural Absence`

Fallacy: an invalid question can be answered with a default or ignored value.
Answer: remove the slot entirely. Notifications do not have availability,
history, registry specs, output, or failure channels. Rules do not pretend to
be freeform quads with special dimensions. A concept that must not answer a
question should lack the field, method, or trait item.

Enforce by making the wrong question unrepresentable instead of documenting
that a value should be ignored.

`Exceptions Become Citizens`

Fallacy: an exception list can grow without changing the concept. Answer:
when exceptions become patterned, name the missing concept and move behavior
there. Hairline quads became `Rule`; dialog-cancel commands became
notifications. Glass-panel filter stacks became `Pane` plus `Material`, because
blur, luminosity, tint, and noise are layers of one shaped material, not
unrelated primitives. A growing exception list is often a concept announcing
itself.

Enforce by watching for enum variants, booleans, or special cases whose names
describe patches rather than things.

`Endpoints Are Truth`

Fallacy: motion, reveal, or conversion can aim anywhere and be corrected after
arrival. Answer: choose endpoints in the owning truth space before moving.
Reveal computes from real viewport and descendant rects. Resting animation
poses are snapped paint-space truths, and moving presentation interpolates
between them. Boundary conversions happen once, at named seams.

Enforce with endpoint tests: final moving frame equals first resting frame,
already-visible reveal is a no-op, and settled scenes match static scenes.

`Findings Graduate Into Invariants`

Fallacy: a bug fix is complete when the symptom disappears. Answer: every
important finding becomes a durable invariant through tests, architecture
guards, or explicit doctrine. The test suite is accumulated case law for the
framework's design.

Enforce by pinning failures that have already escaped once, while avoiding
architecture-test bureaucracy for unproven risks.

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
- Which fallacy is this change tempting us into?
- Which answer-pattern does this change use?
- Is the concept in the lowest honest layer?
- Did any lower layer learn about a higher layer's reason for existence?
- Did runtime orchestrate, or did it absorb domain behavior?
- Did a bridge remain at the edge?
- Did repeated logic move into a real concept instead of a helper bucket?
- Did the type system rule out the bad state where practical?
- If a displaced mechanism remains, is it a witness or dead code?
- If a term is overloaded, do the meanings co-occur in one import scope?
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
