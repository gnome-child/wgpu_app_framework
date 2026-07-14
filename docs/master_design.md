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
  session, interaction, clipboard, tasks, timeline

contracts and routing
  command, target, responder, response

observation
  diagnostics aggregates owner-published facts and samples

orchestration
  runtime, shell, host, platform/native
```

Higher layers may use lower layers. Lower layers must not reach upward to ask
questions owned by higher layers.

### Layer Ownership

`geometry`

Owns renderer-neutral spatial facts: integer layout `Point`, `Rect`, and `Size`,
plus the floating coordinate and unit species shared by text observation,
layout-to-paint conversion, GPU preparation, surfaces, and native realization.
`area::Logical` and `area::Physical` remain distinct types so DPI unit safety
stays enforced by the compiler; floating logical points are `point::Logical`.
Geometry should not know about widgets, commands, layout policy, scenes,
windows, paint lists, or renderers.

The private `paint` module owns renderer-ready policy and representation: its
flattened display list, device-grid `Grid`, rounded `Rect`/`Rounding`, brushes,
clips, groups, and material primitives. Paint consumes geometry-owned area and
point facts; text does not import paint vocabulary. Renderer-neutral
coordinates must not be duplicated under paint merely because paint consumes
them.

`render::scene` owns the one semantic-scene-to-paint projection, including
scale snapping, color transfer, renderer material values, and the shared popup
visual-reach projection. Native platform code consumes this renderer contract;
it does not declare the renderer's grammar conversion or retain a second color
bridge.

The layout-to-paint boundary is a geometry boundary. Layout frames use integer
logical coordinates. Paint consumes floating logical coordinates because a
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

Window facts consume the lower semantic color value directly. The
theme-selected default in `Options` and the `Departed` notification are
higher projections around that core; neither authorizes the fact owner to
depend on scene construction or to duplicate theme and notification policy.

`color`

Owns the semantic sRGB-byte `Color` value, transfer functions, and named byte
conventions. `scene::Color` is the established public presentation re-export
of that lower value; scene construction does not own the datum. Paint RGB
floats are linear, and glyphon color bytes are sRGB. Platform-packed formats
are named at their platform boundary; conversion happens once through `color`
rather than being re-derived by native adapters.

`identity`

Owns the stable authored `Id` datum shared by reconciliation, interaction, and
command routing. It is lower vocabulary, not an interaction state machine:
it carries no focus, table, target, session, or responder policy.
`interaction::Id` is the established public projection of that one declaration;
the lower owner is private implementation housing rather than a second public
spelling.

`text`

Owns document, buffer, edit, surface, layout, and unicode concepts. The text
engine should be usable without the framework runtime. Editing state belongs to
explicit edit/session values, not secretly to a shared buffer when multiple
views or surfaces can exist.

Caret affinity belongs to the caret position. `Buffer` owns Position-to-Cursor
conversion, and pointer edits, cursor clamping, obscured-field projection, and
per-line layout projection preserve that affinity. A cursor mutation that
cannot preserve it must choose an affinity explicitly; default construction is
not an allowed conversion.

Hit mapping follows each glyph's own bidi level, including LTR glyphs embedded
in an RTL paragraph. Obscured fields emit exactly one dot per source grapheme;
an empty source has one boundary and therefore renders no phantom dot.

Text layout owns shaped-buffer cache mechanics through `ShapingCache`; area
lines, field surfaces, and inline text/icons supply domain keys and retention
limits, while the shared owner mediates lookup, insertion, and `FontSystem` use.
The concrete glyph buffer remains text implementation vocabulary. Scene and
private paint surfaces may carry a cloned text-owned `ShapedBuffer` handle, but
they do not name glyphon types or expose the buffer through public scene/text
APIs. Render is the only downstream layer that borrows the concrete buffer.
This preserves zero-copy shaping identity without making transit layers owners
of the shaping implementation.

Text has three provenance contracts. Author text is written by the program and
must fit; layout reports an `author_text_overflows` diagnostic instead of
silently turning it into world text. World text comes from files, providers,
users, or other unbounded sources and its view constructor requires an explicit
`text::Overflow`: clip, end ellipsis, or middle ellipsis. User text lives in an
editable surface and scrolls or reveals through the existing TextArea and
TextBox machinery. World-text ellipsis is resolved by text layout against real
font metrics before scene projection, at Unicode grapheme boundaries, using the
real `…` glyph. Scene and paint carry the resolved text and overflow cache
identity; they do not invent truncation. Callers must not pre-truncate strings.

Overflow v1 defines end and middle in logical source order. End ellipsis keeps a
logical prefix; middle ellipsis keeps logical head and tail segments. Bidi
shaping then presents that resolved string in visual order. This policy is
deliberately deterministic; visually directed truncation remains future work
that requires its own accessibility and product decision.

Document saves capture a `document::Version` containing document identity and
buffer revision. Deferred completion carries that version plus a monotonically
newer save generation; only the latest generation for the same identity may
update the document. A completion for an older revision records what reached
disk but leaves newer edits dirty. `SaveSnapshot` writes through a sibling
temporary file and atomic replacement, so memory never calls a partial write a
saved document.

`document`

Owns the file-backed editable-document workflow: document identity, buffer and
selection state, path and saved revision, save snapshots, atomic replacement,
standard editing/file commands, their outcomes, and document-shaped dialog
facts. It consumes text, command, and clipboard contracts but no runtime, UI,
platform, or application state. Runtime coordinates its commands; widgets may
project its buffer and selection by value; neither makes document part of
runtime or UI. The target-specific atomic replacement primitive remains a
private part of the save transaction and is document-owned dependency weight.

`widget`

Owns ergonomic builders for view data. Widgets produce nodes. Widgets do not
execute behavior. A widget may project an app-facing concept into declarative
view/action data, but it should not become the runtime for that concept.
`TextArea::from_document` is the named value-semantics projection of a
`document::Document`; `from_buffer` remains the general constructor. Document
code does not depend back on widgets.

`view`

Owns declarative node data, bindings, presentation, style, focus affordances,
and action metadata. View answers "what is being presented?" It should not own
input dispatch, command execution, mutation history, platform rendering, or
task execution.

The application view callback is a facade contract around that declarative
territory. Its `view::Context` input is not node data: it is a per-window
callback envelope containing the window identity and an immutable diagnostics
snapshot. Runtime constructs the envelope when it invokes the callback, and
`src/view/context.rs` is the one facade-owned source responsibility housed under
the established `view` API. No other view source may depend on diagnostics.
This split preserves the public spelling without making the whole view module
own application instrumentation or forcing diagnostics through declarative UI.

`task`

Owns deferred job execution through a bounded worker pool. `Task<E>` describes
work that eventually produces an application event; native `Runner` moves the
job to the executor, returns its completion through the event-loop proxy, and
only then dispatches the event on the UI thread. Pending work never requests a
UI poll wake. Cancellation and runtime restore keep the task id authoritative,
so a late worker completion for a no-longer-pending id is inert. Headless test
helpers may execute a task deterministically, but they are not the native
production path.

Worker-pool realization retains every worker the operating system admits. If
none can start, the executor rejects work and the native runner cancels the
authoritative task id. Work is never buffered without a consumer, moved onto
the UI thread, or allowed to panic application startup merely because a worker
thread could not be created.

Suite-runtime measurements distinguish Cargo wall time from test-harness work.
The Loop III suite-runtime audit at 785 tests measured five warm `cargo test --lib`
runs at 2.054s average, the already-built test binary at 1.169s average, and
the harness-reported test work at 1.08s. The executor's exact test measured the
same ~96.5ms process floor as an empty filtered harness, and removing that test
did not materially change the suite. Its two-second channel timeout is a
failure ceiling around immediately dispatched work, not a scheduled wait;
executor tests must not sleep. The apparent ~1.05s -> ~2.07s doubling was a
measurement-boundary mismatch, not executor runtime debt.

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

Provided containers derive a bounded public child composition from application
data rather than requiring the application to declare every logical child.
`virtual_list::Provider` is the first species: it supplies a logical length,
stable `virtual_list::Key`, efficient reverse lookup, and an ordinary public
`view::Node` for a requested row. Provider keys extend retained sibling
reconciliation; they do not create a second identity runtime. Uniform row
height lets layout derive the visible range arithmetically from the existing
viewport. Runtime reaches a bounded fixed point by materializing that range
plus overscan and pins, then laying out once more.

Dematerialization is not removal. A row outside the range still exists while
its key remains in the provider, so composition does not emit removal facts and
bounded inactive drafts may survive. Focused, pointer-captured, and actively
edited rows pin and may remain clipped; selection never pins. When reverse
lookup no longer finds a key, ordinary composition removal prunes its focus,
capture, active edit, and draft state. Logical focus movement first
materializes the keyed row and only then transfers focus. V1 virtualization is
synchronous, flat, uniform-height, and bounded to visible rows plus overscan
and pins; it has no variable-height, streaming, or async provider policy.

`Table` is the record-table species of provided container. It composes one
ordinary sticky header with one selectable `VirtualList`; its provider returns
ordinary public cell nodes, so buttons, choices, labels, overflow, focus, and
commands keep their existing owners. A `table::Column` supplies a stable column
id and either fixed or weighted width. The shared horizontal flow allocator
distributes weighted tracks for both headers and visible rows; table code does
not scan provider data for intrinsic widths or own a parallel track solver.

Table identity is `(table id, provider row key, column id)`. Retained cell
matching, active-cell presentation, and future editing/accessibility use that
tuple rather than row or column indices. Selection remains keyed row state;
table navigation adds only a window-local active column id. Column-resize
widths are also window-local session presentation keyed by table and column,
projected into the table model before visible rows materialize, and never
mutate provider data. Header widgets emit ordinary application commands and
the application retains `SortState`. Lazy `Source::new` providers continue to
own their order without enumeration; bounded `Source::records` providers
apply the selected column's derived ordering projection automatically. Table
paint derives striping and rules from layout row/cell facts, and the shared
`Rule` rasterizer owns physical-pixel snapping across scale factors.

Typed columns select a presentation medium explicitly. `Column::text` accepts
any `Display` value, adds editing only for `FromStr` values, and defaults to
sortable construction for `Ord` values. `.unsortable()` is the explicit
escape hatch and removes both the `Ord` requirement and header affordance.
`Column::boolean` projects any value with an honest forward `Into<bool>`
conversion; reverse `From<bool>` is required only when `.toggle()` adds
interaction. One optional ordering projection drives both the header and
bounded-record ordering; there is no separate sortable flag. Alignment and
input filtering are column policy, not hidden type behavior. The framework
owns the cell species and geometry; std owns display, parsing, conversion, and
ordering meanings.
Types with state beyond the Boolean medium expose a Boolean field rather than
round-tripping the whole value and silently discarding information.

Editable record cells reuse TextBox editing, draft history, command mapping,
focus, and virtual-row pinning. `table::TextEditor` keeps text validation and
typed application commit mapping distinct. `table::NumberEditor` additionally
owns integer parsing before an independent domain validator; it does not imply
a universal value/editor trait. Both use `table::Cell` itself as text focus,
target, draft, rejection, and command identity. No synthesized string id or
index-derived identity participates.

Enter commits the active editor; Tab commits before leaving; invalid input
keeps focus and draft with a reason; Escape cancels and rebuilds from provider
truth. A focused edited row follows the existing virtual pin law, reorder
retains its tuple identity, and actual provider deletion prunes focus, draft,
and rejection together. Successful commits travel through ordinary typed
commands and therefore existing application history. Rejection presentation is
session/window-local, exposed read-only to callers, and projected as an editor
outline; provider data changes only when the application handles the command.

`layout`

Owns measurement, frame construction, text measurement integration, and
hit-testing. Layout answers "where is it?" and "what was hit?" It should not
answer "can this command run?" or "what side effect should happen?"
`Frame` is one common geometry/identity/clip/presentation envelope around a
typed `FrameContent`. The content discriminant is the frame's role truth and
owns mutually exclusive choice, text, slider, scroll, and unit-role payloads;
there is no independent copied role or parallel optional leaf-payload cluster.
Bound presentation remains a common typed optional because ordinary Elements,
controls, menu bindings, and palette Labels can all truthfully carry commands.
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

The OS pointer cursor is a promise about what an ordinary primary press would
do now. Runtime resolves one private `ResolvedPress` from last-presented,
clip-aware hit truth, retained pointer point/surface/modifiers, capture, target
meaning, task focus, and selectable-row pre-gesture focality. Its
`PressAdmission` determines whether the exact target is inert, selection-only,
or admitted; cursor projection and pointer-down consume that same answer.
Selection-only members keep the default cursor. After a first click focalizes a
row, only successful presentation of that new truth may re-resolve a stationary
pointer to the member cursor.

Selectable text surfaces use the text cursor only where the admitted press can
place or drag a caret or selection. Read-only selectable text qualifies;
painted labels, menu rows, buttons, palette rows, chrome, indicators, and
disabled fields keep the default cursor. Hidden or occluded text cannot leak an
I-beam through overlays. Capture retains the cursor meaning resolved at press
time rather than reconstructing it from target identity. Modifier changes
re-resolve the stationary pointer and may emit a deduplicated cursor update,
but never require redraw or view reconstruction. Hover resolution is purely
deterministic: it does not parse, validate, resolve commands, or preflight the
fallible task departure that the real gesture attempts once.

Cursor vocabulary follows interaction meaning, not the platform's catalog. A
new `pointer::Cursor` variant requires a real resolved-press species with a
precise semantic criterion plus execution, capture, and platform witnesses.
Applications do not assign cursors to nodes or primitives; a new application
need earns a semantic framework interaction slot rather than a numeric or raw
cursor override.

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

Scene material values own their semantic constraints. In particular,
`scene::Refraction::clamped` is the one refraction constraint computation;
the renderer scene bridge applies it before projecting into the private display
list, and paint/render forward the resolved values without reclamping them.

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
not a second interaction model. Pointer movement, buttons, modifiers, and wheel events
from a popup window are converted from popup-local physical coordinates to the
parent window's logical overlay coordinates by using the popup window scale
factor and the entry bounds. The parent window remains authoritative for focus,
commands, keyboard routing, diagnostics, and session state.
Native popup lifetime is synchronized only by an authoritative overlay
presentation pass and is scoped per rendered parent: no popup presentation
statement means leave existing popups alone, an authoritative empty popup set
closes stale popups for the synchronized parents, and popups owned by parents
absent from that pass remain untouched.
Native cursor routing keeps the parent window's logical cursor value separate
from the physical window currently under the pointer. Raw parent/popup
enter-move-leave events switch that host immediately, reset the old host, and
apply the stored value to the new host even when logical cursor dedup observes
no value change. Layout's focused text caret rectangle is the one geometry used
by caret paint and IME placement. Runtime projects that rectangle onto the
physical host declared by overlay ownership: parent coordinates for in-frame
content, popup-local coordinates for a native floating panel. Platform applies
the IME update only after popup synchronization; native routing disables the
previous popup geometry host, keeps the parent context enabled as the logical
keyboard authority, and gives the declared parent or popup host the cursor
area. IME preedit, commit, and disable events received by a popup remain
coordinate-free input events and adapt back into the parent's logical
focus/session truth.

Intent is portable; realization is reported. `Material::Glass` means
"glasslike panel material" and is retained once as an ordered, keyed scene
request. A request carries declaring identity, logical geometry and rounding,
inherited clip provenance, effective opacity, material recipe, and independent
scene order. Identity comes from the retained declaring frame; traversal and
primitive order never become identity.

Forecast, platform outcome, residual paint, and final fidelity are separate
facts. A platform reports only the material parts it actually realized. One
scene resolver combines the original request, uniquely matching reports, and
renderer context; only reported parts may be removed from residual paint.
Missing, stale, duplicate, failed, or identity-mismatched reports consume
nothing. `Full`, `Frost`, and `Fallback` summarize the final visual result;
they are diagnostics, not assembly instructions. In-frame material is the
all-platform-`None` case and can still reach `Full` through renderer sampling;
platform `None` is not itself fallback.

Overlay backend choice follows window capability, not material identity. A
floating panel that prefers `NativePopup` uses it whenever the platform probe
supports native popup windows; unsupported platforms fall back to `InFrame`.
Windows DX12 can earn per-region host frost in one composition tree beneath a
transparent premultiplied wgpu tenant surface. Non-tenancy native paths may use
the legacy whole-window accent bridge for its supported single-region case.
Anything unreported remains in the resolver's explicit renderer/fallback plan,
so one unsupported region cannot demote its siblings.

Material-region geometry remains scene truth. Device conversion and snapping
consume the same `paint::Grid` and rounded-rectangle projection as renderer
paint; inherited clips are realized only when the platform can represent them.
The platform retains region visuals by declaring `NodeId`, while refreshing
their order from each scene projection. Interaction with projected geometry
routes back to retained sources; projection storage is never application
truth.

OS-side state crosses at the narrowest honest rate. Region presence, removal,
and ordering are immediate because they change what exists. Stable scalar
parameters may settle where the platform call is expensive. `SysApplicator<T>`
continues to own desired/applied snapshots for geometry, the legacy accent
bridge, and redirected-path DWM border color; composition-region retention is
its own keyed collection owner rather than another scalar applicator.
Drag-rate parameter changes must not build a queue of native calls.

`FloatingPanel.border` remains the one popup border datum. In-frame and native
residual paint use it as framework chrome. Composition-backed Windows popups
suppress the DWM border and rounding because the painted border, transparent
corners, host-frost clip, and composition shadow consume one framework
silhouette. Redirected fallback still encodes the same sRGB bytes in DWM's
border format and applies later theme changes through the settle-rate path.

Native surface context creation owns the one cross-platform backend selection.
Target-specific presentation policy belongs in render backend options, not
identical cfg-specific default functions.

The native adapter consumes first-party renderer contracts, never `wgpu` or
`wgpu-hal` vocabulary directly. `render::context::Backends` carries explicit,
DX12-first, and ordinary backend sets while platform retains attempt order and
fallback lifecycle. `render::surface::Format` is the opaque renderer-cache
identity; `render::surface::Target` encodes unsafe native target association;
and `render::surface::WindowsPopupSupport` is the resolved format/alpha
capability consumed by popup material policy. Renderer parents project only
their same-named central types (`Context`, `Canvas`, and `Surface`); supporting
contracts remain qualified through their modules.

#### Windows Platform Map

This is the single reference for Windows popup shell, compositor, color, and
diagnostic policy. Portable overlay intent remains in the surrounding `scene`
contract; Win32 realization details stay here and behind the native `sys` seam.

**Shell and activation.** A popup HWND is an owned, topmost, nonactivating tool
window. Winit creation attributes are inputs, not the postcondition: immediately
after creation the style purge sets `WS_POPUP`, removes caption, system-menu,
resizable-frame, min/max, border, and dialog-frame bits, adds
`WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, removes `WS_EX_APPWINDOW`, and commits any
change with `SWP_FRAMECHANGED | SWP_NOACTIVATE`. The installed subclass answers
`WM_MOUSEACTIVATE` with `MA_NOACTIVATE`; configure and show use no-activate paths.
This is shell correctness, not an optional acrylic tweak.

**Material and packed color.** DX12 tenancy creates one WinRT desktop target
and tree per popup. A host-backdrop region container sits below the live wgpu
content sprite; keyed rounded visuals report frost only after brush, geometry,
clip, update, and order operations succeed. Surface tint remains renderer-owned
residual paint, so bare host frost does not claim exact tint, blur sigma,
refraction, luminosity, saturation, or noise. The legacy non-tenancy bridge
uses `SetWindowCompositionAttribute` with
`ACCENT_ENABLE_ACRYLICBLURBEHIND` only for its supported single full-window
region. Accent `GradientColor` is `AABBGGRR`/ABGR. Tenancy neither applies nor
disables that legacy accent policy. `FloatingPanel.border` converts once to
`COLORREF` (`0x00BBGGRR`) for redirected `DWMWA_BORDER_COLOR`; composition
instead applies `DWMWA_COLOR_NONE`, disables independent DWM rounding and
shadow, and keeps the painted edge beneath the popup root opacity.

**Silhouette and reach.** One scale-resolved popup projection consumes the
retained panel geometry and the existing paint shadow-reach calculation. Panel
bounds remain placement, input, containment, and accessibility truth; visual
bounds size and position the composition-backed HWND/swapchain. The projection
also supplies the panel offset used by paint, material regions, and popup event
translation. The platform may rasterize the declared rounded shadow, but its
mask, color, blur, spread, offset, and surface envelope remain framework truth.
One visible edge has one painter.

**Presentation causality.** Window existence, OS presentation eligibility,
GPU presentation, compositor pickup, and user visibility are separate facts.
Every popup show cycle begins concealed. On Windows the application sets
`DWMWA_CLOAK`, shows the HWND through the existing no-activate path so its
redirected or composition surface can present, acquires and presents the
current scene, crosses the `DwmFlush` compositor-pickup barrier, and only then
removes the cloak. The first user-visible pixels therefore follow a current
present; stale or empty swapchain content is never the mechanism used to make
the window presentable.

Birth, resize, retarget, host reuse, scale change, and material change are all
show cycles. Each native popup configuration receives a fresh monotonic
generation. Its content and material receipts are valid only for that exact
generation; earlier receipts are inert. A visible host keeps its complete
current generation while replacement pixels and resolved geometry are staged;
after the replacement present, its HWND geometry and hit region commit before
the compositor-pickup barrier promotes the pending realization. Same-geometry
content changes use the same generation discipline without moving or cloaking
the host. Material and scale changes retain the concealed reconfiguration gate
because one present cannot make those contracts atomic. Geometry is resolved
once by the selected host into a retained popup realization; paint, hit testing,
event translation, IME, accessibility, clipping, and material projection
consume that same fact. Placement intent is never reused as realized geometry.

Windows are different presentation clocks. Parent-window presents, hover
frames, and fades cannot make a popup stale; popup currentness advances only
from popup-local content, geometry, scale, material, or semantic identity.
Parameter animation does not mint a content generation. Concealment is reserved
for transitions one whole present cannot make atomic; ordinary same-geometry
content changes present directly under a new serial. Absence is a lifecycle
state as well: a retiring popup may keep visual geometry for its exit, but owns
no hit region, input route, IME authority, or semantic session.

The latency epoch begins at retained overlay appearance, not at native-window
construction. Popup scene preparation reports rebuild, layout, and assembly
separately from HWND, surface, material, draw, acquire, synchronization, and
exposure work. An independently presentable popup is submitted before its
parent window's frame: popup visibility does not pay for unrelated parent
rendering. Composition prepares an entering root at imperceptible opacity while
concealed and starts the full entrance animation only after exposure. Until
then, the prepared entrance owns root opacity: a logical transition to stable
cannot overwrite it merely because material readiness outlived the nominal fade
duration. A retained transition to stable state that requests identical popup
pixels updates the compositor timeline without another swapchain submission.

`PopupFirstPresentTrace` records timestamped created, configured,
prepared-concealed, acquire outcome, present, synchronization, and exposed
stages under `wgpu_l3::native_popup`. A skipped acquire remains concealed and
earns a retry. Explicit synchronization failure remains concealed through one
bounded confirmation present; the second freshly presented frame ends the
fallback even if synchronization reporting fails again. This is not an
unbounded retry budget. OS-requested popup redraws remain legitimate redraws.
Accent maintenance that has no immediate draw requests one parent redraw.
Platforms without an implemented concealment primitive retain their existing
show path; they do not inherit the Windows guarantee by assertion.

**Backend and alpha handoff.** An explicit `WGPU_BACKEND` is authoritative and
attempts only that backend. Without an override, Windows attempts DX12 first so
composition tenancy can be earned, then falls back through the ordinary backend
set if DX12 initialization fails. `CompositionBacked` means DX12
`DxgiFromVisual`, `WS_EX_NOREDIRECTIONBITMAP`, and one framework-owned WinRT
target/tree. wgpu receives an unattached classic visual, exposes its live DXGI
swapchain through the hal escape hatch, and becomes a sprite tenant in that
tree. `RedirectedFallback` keeps redirection, requests premultiplied alpha, and
may use the legacy accent bridge. Failure before tenancy completion drops the
partial tree and remains a truthful non-tenancy popup.

The unsafe surface target encoding and scoped DX12 HAL surface borrow belong to
`render::surface`; the native Windows composition owner supplies the live
visual, consumes the cloned DXGI swapchain, and owns every COM tree operation.
This keeps renderer dependency types out of platform without moving Win32 or
WinRT realization policy into render.

The normal renderer writes the scene to an sRGB offscreen target. A Windows
premultiplied popup pack then samples associated linear RGB, unassociates it,
applies the exact piecewise sRGB transfer, re-associates it, and uses `REPLACE`
to write the non-sRGB premultiplied surface. Opaque/default windows keep the
legacy composition blit; it is not a popup handoff. Alpha evidence must use a
real half-alpha primitive or premultiplied clear. The authoritative witness is a
standalone primitive over a transparent clear. Its readback that proves both alpha and premultiplied RGB is the test.
The result rejects clear-only witnesses and visuals nested inside panel body content as contaminated evidence.

**Diagnostics.** `native_alpha_probe` owns backend, accent, and attribute
bisection: begin with a plain transparent window, compare Vulkan with DX12
`DxgiFromVisual`, test individual attributes, then suspicious pairs such as
owner+toolwindow or no-redirection+backdrop. Foreground defects are partitioned
before fixing into alpha convention, color/gamma, and scale/stretch. Witnesses
include fractional quad coverage and glyph masks. The manual matrix compares
`OpaqueFallback`, transparent/no-accent, and acrylic with both backed and
unbacked content. A backed mismatch first convicts scale/rendering; matching
no-accent and acrylic defects implicate the general native boundary; acrylic-only
defects implicate accent. Scale logs carry scene bounds, requested bounds,
observed inner size, canvas physical area, surface size, and popup scale. Real
fixtures include disabled menu shortcuts and live hover/drag sliders.

Native popup fades have one opacity owner per realization. Tenancy animates the
common composition root, so composition shadow, host frost, wgpu content, and
framework residual chrome share one compositor timeline without per-frame
application redraw or repeated `DwmFlush`. First presentation earns exposure;
it is not an animation clock. Reopen retargets from the prior compositor
timeline's mathematically current opacity, never from a freshly sampled zero.
At retirement completion every path cloaks/hides the HWND before its tree,
surface, subclass, or window is dismantled. Legacy non-tenancy has native
placement but no native animator: it exposes the fresh first frame at full
opacity, hides immediately on dismissal, and allocates no pseudo-fade deadline
or retiring surface. On composition-backed dismissal, a noninteractive
`RetiringPopup` keeps the same native
surface and material facts alive through the exit timeline, then native
synchronization closes it. It never becomes a parent-window ghost.

Overlay ghosts are paint-only afterlife. When a live in-frame entry is
dismissed, runtime may retain its final scene bucket briefly as a `Ghost` for
fade-out, but the ghost is not layout, hit testing, wheel targeting, cursor
resolution, focus routing, dismissal containment, semantics, or command
routing. Native popup entries do not allocate parent ghosts; their retiring
native layer fades on the same popup surface. Focus restoration and key routing
update when the live entry is dismissed, not when either kind of afterlife
expires. Ghost and retiring-popup fade frames are presentation work and must
not imply model revision changes. If an in-frame ghost contains a
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

The alpha pipeline has one convention at each stage:

- Quad fragments and glyph mask fragments emit straight linear RGB plus alpha;
  straight source-over associates them as they enter a target. Glyphon's color
  atlas follows the same straight-output blend contract.
- Main scene, clip, group, and filter textures store associated (premultiplied)
  linear RGBA. An sRGB texture format changes only the stored RGB encoding;
  blending and shader sampling still see linear associated RGB.
- Blur and blit passes sample and replace associated RGBA unchanged. Refraction,
  luminosity, and noise scratch passes explicitly return associated RGBA and
  replace a cleared target. A shape-alpha material operation may unassociate a
  sample before replacing source alpha with shape coverage; source-alpha paths
  remain associated throughout.
- Clip, group, overlay, and ghost composite-back shaders emit associated RGBA
  and use premultiplied source-over. Group opacity multiplies RGB and alpha once;
  it never unassociates a low-alpha sample and asks straight blending to restore
  it. Native-popup retiring content follows this same group path.
- Platform surface handoffs that require a different encoded association use an
  explicit replace pass; platform-specific packing belongs in the platform map.

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
Temporary group targets follow the alpha pipeline map above; the one composite
keeps text, rounded edges, shadows, and backdrop effects together. The
renderer-owned fragment-output blend states close the premultiplied-alpha audit
and make the filter chain arithmetically ready for future local blur:
blur scratch preserves associated RGB through transparent edges, and
source-alpha composite-back no longer creates straight-alpha dark halos. Local
blur remains a separate product/API feature; readiness is not implementation.

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

Theme also owns the framework default canvas color. Unthemed scene clears,
window defaults, and examples that choose the framework default consume that
one token; `window::DEFAULT_CANVAS_COLOR` remains its public projection. The
theme `root` surface is a separate token even when a variant currently assigns
it the same bytes as `canvas`.

Window's lower vocabulary is identity, facts, kind, presentation epoch, and
departure. Application-facing `window::Options` and its framework default
projections are facade responsibilities: their sources may select the
theme-owned canvas token, while lower window vocabulary must not depend on UI
policy. The parent keeps the established exact `window::Options`,
`window::DEFAULT_TITLE`, and `window::DEFAULT_CANVAS_COLOR` spellings; it does
not duplicate their declarations or preserve compound aliases.

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
such as Control, Shift, Option/Alt, Command, and Delete, separated by `+` where
a chord has multiple parts; weak or platform-specific keys without a clear icon
stay textual. A Unicode symbol may be an accessibility or plain-text fallback,
but it is not a substitute for the icon primitive in rendered shortcut chrome.
The default shortcut display is house grammar on every platform: icon
modifiers with separators are a deliberate departure from OS-native menu
grammar, while key resolution stays platform-truthful.

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

Provided-list selection is window-local interaction state keyed by list id and
`virtual_list::Key`; it is not application data, does not dirty documents, and
does not enter application history. `selection::Selection` exposes read-only
membership, anchor, and active facts through `Session`. Plain pointer input
replaces membership, Primary-click toggles, Shift extends from the stable
anchor, and list-scoped keyboard navigation moves the active key in current
provider order. Host pointer-down events carry current modifiers so native and
headless paths execute the same state machine.

Select-all uses all-except membership, so selecting one million logical rows is
constant state. Selected offscreen rows do not pin and do not become view or
layout nodes. A pending active-item reveal may materialize its one target for a
single rebuild; after viewport feedback includes it in the ordinary visible
range, that temporary pin disappears. Provider reorder preserves keys;
deletion reconciles selected membership and deterministically moves deleted
anchor/active facts to the nearest remaining selected key. Each anchor and
active endpoint owns its stable key and last usable provider index atomically;
the index is only a navigation fallback while the key remains authoritative.
Selections are
scoped independently by window and list and participate in runtime snapshot
restore; window departure deletes them with the window.

Keyboard input belongs to the palette scope first and is consumed there; the
list describes the captured world beneath it. The query is an ordinary text
box in that transient scope, so text commands resolve through the standard
focused-text service, while a selected row invokes its command against the
captured scope. Command descriptions omit commands marked `Listing::Describer`:
a description does not include the act of describing.

A command surface has three separate stages: candidate discovery, live command
resolution, and presentation. The palette discovers globally and presents
search results. A context menu captures one semantic `responder::Path`, stored
broad-to-exact, and presents each nonempty layer as an ordinary menu section.
Both consume one private erased command projection. Public construction remains
typed, and candidate-provider types keep global and contextual discovery
unsubstitutable.

One command-population owner supplies discovery, erased triggers, registry
metadata lookup, responder resolution, and state composition. Surface policy is
an explicit input to that owner, not duplicated orchestration and not a claim
that all surfaces mean the same thing. A conventional bar keeps registered
unclaimed commands visible but disabled and resolves against the live Task
chain. A context menu consumes claims from its captured semantic path and keeps
the captured invocation route. The palette keeps enabled captured-task claims
and orders them by provenance, relevance, then registration. Sharing mechanics
must never collapse these membership, traversal, ordering, or freshness laws.

A menu orders by what it is about. An inspection menu is about an object, so
its `responder::Path` orders sections broad-to-exact (or Task traversal from an
active editor). A conventional menu bar is about a culturally familiar command
vocabulary, so platform topology data orders its categories and groups.
Responder scope determines who acts; conventional role determines where people
look. Undo and Copy may both resolve at Focused scope while remaining separated
as History and Clipboard, proving that scope does not contain cultural
topology.

Traversal names the question rather than a geometric direction. `Task` serves
an active task exact-to-broad; keyboard input and the command palette use it.
`Inspection` examines a containing object broad-to-exact; a resting context
menu uses it. The first claim consumes a command identity in either traversal,
including a disabled claim. Absence permits fallthrough. Claim precedence,
menu ordering, separators, and invocation routing therefore consume one path
instead of parallel ranks. An active table editor makes its text task frame the
context root, so its Select All consumes before the table; the same cell at rest
is inspected table-to-row-to-facet and the table consumes Select All.

Context identity is not keyboard focus. The lower `responder::Scope` contains
only an optional responder identity and the routing kind. The higher
`session::CommandScope` aligns that route with optional focus and table facts
for runtime service realization. A contextual scope may therefore name an
exact responder and an optional text focus independently; opening a context
menu never manufactures focus to discover a target. Responder never imports
session, interaction, or table state. The route that advertised an automatic
responder or service action is also the route revalidated for invocation, so a
disappearing local owner cannot fall through to a broader target. Sessions
retain owner and anchor, never command availability; state is re-resolved while
the menu remains open.

Tables contribute their existing keyed selection domain, the focal provider
row, and the exact cell facet to that path. Secondary-clicking an unselected row
makes it the sole selection; secondary-clicking a selected row preserves the
whole multiselection. Canonical Select All is owned by the table's bounded
provider domain and uses the existing all-except representation. The focal row
is distinct from selection anchor, active key, and membership. It remains
pinned while merely dematerialized and dismisses the context session when the
provider deletes it.

Standard bindings, text boxes, table rows, and Boolean cells derive contextual
participation from the semantic path they already contribute. Generated rows
may still capture an explicit typed row action from their stable key, but no
wrapper is required merely to expose ordinary text or control commands.

An ordinary binding becomes an automatic context action only when its source is
an honest button-like interaction; text-input commit bindings are editing
mechanics, not menu candidates. Explicit context bindings remain available when
an application deliberately chooses a different contextual action.

Authored menu-bar panels and contextual panels share one overlay lifecycle and
z-order grammar. Retargeting retires the prior visual-only panel and enters a
fresh interactive panel; host infrastructure may be reused, but popup sessions,
captured paths, generation receipts, input authority, and fades are never
reused as infrastructure.

Menu placement is one geometry projection shared by authored and contextual
menus. An anchor and intrinsic size resolve against availability supplied by
the realization host: inherited viewport visibility for in-frame menus and
the anchor monitor's work area for native popups. Layout and platform code
consume the same request; neither owns separate flip or clamp arithmetic.

Target labels are debug and presentation data, not identity. Target identity
is its kind, stable id or retained node identity, and routing source. Changing
a target label or capture behavior must not fork hover, scroll, draft, or
focus identity.

`command`

Owns command contracts: typed args, typed output, names, key chords,
availability, history policy, observers, registry metadata, and triggers.
Commands describe what can be asked. They do not decide which concrete target
is currently meant by focus, capture, or app state.

`command::Set` is an enumerable bundle of command specs, not an editing mode.
The standard document editing set owns its members, labels, and chords; apps
install it in one line and may decline members individually. Sets never attach
responders, choose focus, create editing state, or install observers.

Commands are imperative requests. Past-tense facts are notifications, not
commands. A past-tense command is a classification error because facts have no
availability, no history policy, no registry spec, and no advertised command
surface.

Command shortcuts are semantic data. `Primary+S` means the platform command
modifier, not the physical Control key, and standard roles such as Undo, Redo,
Copy, Save, CloseWindow, and CommandPalette name user intent before platform
resolution. A standard role may resolve to multiple concrete chords; the first
is the display chord and all chords match input.

`command::Standard` is conventional meaning, not merely a shortcut alias.
Label, chord, menu category, section, slot, shortcut visibility, and
platform-specific relocation are projections of that meaning. Registration is
the persistent vocabulary of an opt-in conventional bar; the current responder
chain supplies live state and actor without reordering the cultural topology.
An application requests that projection with `ui.standard_menu_bar()` and
declares static deviations through typed placement metadata. Dynamic or
argument-bearing deviations use the typed mixed-bar builder. Fully authored
`ui.menu_bar(...)` remains the escape hatch; registration alone never creates
ambient UI. All three forms terminate in the same ordinary MenuBar, Menu,
Separator, and Binding nodes and therefore share layout, focus, paint, popup,
and activation behavior.

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

`feedback`

Owns runtime facts that should be communicated now. Reporting accepts
`Display` and eagerly snapshots its text at the reporting boundary; retained
feedback is severity plus formatted text, while its typed store owns identity
and lifetime. Severity orders and dresses facts (`Error`, `Warning`, `Info`)
but never implies focus trapping, persistence, dismissal, or interaction.

Command description, contextual command hint, runtime feedback, and resolved
text overflow remain separate truths on separate clocks. An element's
auxiliary-content resolver consults them without copying one into another:
feedback by severity, then hint, description, and confirmed overflow. Retained
severity is also distinct from presentation chrome: descriptive content may
use informational chrome, while overflow revelation is plain and glyphless.

Every auxiliary panel consumes the ordinary floating-panel path: complete
content measurement, one placement request, host selection, realization,
generation receipt, and exposure. Hover tips and noninteractive feedback are
policies on that path, not alternate overlay species; they are unfocusable,
hit-transparent, and absent from keyboard traversal. Content size is resolved
before placement, and the measured paint, host, and hit-transparent geometry
are one rectangle.

Panel attachment is independent of meaning, severity, and lifetime. A hover
revelation captures the pointer's retained-layout position when it becomes
visible and keeps that snapshot for the panel generation; it does not borrow a
live pointer clock. Context menus consume the same point-anchor solver without
hover clearance, while persistent validation consumes its typed subject
rectangle. The shared placement request alone applies clearance, edge flips,
and final clamping.

Accessibility consumes the semantic truths independently of panel visibility.
`Spec::description` maps to direct AccessKit Description; node-supplied
description may use DescribedBy; validation uses Invalid plus ErrorMessage;
runtime dialogue may later use Live when its caller proves an announcement
policy. `State::hint` remains contextual and never replaces description.

`window::Departed` is the single past-tense close fact. Window owns the pure
fact declaration and payload meaning; the notification owner supplies its
generic `Notification` binding so lower window vocabulary does not import
command routing. Runtime publication is only a listener registry: layout
caches, overlay entries and ghosts, animation schedules, visual animations,
composition, diagnostics, pointer gestures, and the native popup manager each
own their own purge.
Per-window state subscribes to `window::Departed` or documents why not.
Close paths must not grow local
cleanup checklists; a new per-window store joins the notification instead.

Document dialog-cancel notifications live under `document` in v1 because the
current dialog kinds are document-shaped. Future generic dialogs should let the
opener declare which fact a dialog outcome emits instead of treating that
placement as load-bearing.

`keyboard`

Owns the dependency-free `Key` and `Modifiers` facts received from a keyboard.
They contain no shortcut profile, focus, target, session, command, or runtime
policy. The lower housing is private; `input::Key` and `input::Modifiers` are
the established exact public projections of those declarations.

`input`

Owns runtime ingress: the public `Input` event sum, handling `Outcome`, and
`TextDrop` payload. Those values may name session focus, interaction targets,
commands, and text operations because Runtime is their consumer and executor.
Command, keymap, and interaction state consume the lower keyboard facts
directly; they do not depend on the runtime ingress module merely to name a key
or modifier set.

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

`text::Buffer`

Owns text bytes as one persistent tree of spans into immutable source buffers:
one owned original source plus immutable add chunks. Tree nodes summarize bytes
and logical line breaks; clones share the root, and edits path-copy only the
split/insert route. A separate persistent line-index tree owns stable
`LineLayoutIdentity` values so editing one line does not invalidate layout
caches for its siblings. Grapheme and word segmentation are line-local and
lazy; the buffer stores no whole-file character or grapheme index.

**Owned sources, never retained mappings.** A file load reads UTF-8 into owned
immutable source storage. A mapping must not survive loading: external file
truncation can turn a later mapped read into SIGBUS or an access violation, and
Windows keeps mapped files locked against the atomic-rename save path. Save
snapshots therefore Arc-share the persistent tree and stream its spans into the
sibling temporary file without flattening the document.

Original line-ending bytes remain source truth. The line index treats LF and
CRLF uniformly as logical boundaries and excludes CRLF bytes from line content.
The dominant loaded ending (count, then first-seen tie break; LF for a file with
none) owns newly inserted logical line breaks. Programmatic multiline
construction retains its legacy canonical-LF policy; file loading does not
normalize bytes.

`target` and `responder`

Own executable capability and routing participation. A target says "this value
can perform this command." A listener says "this value wants to hear this
notification." A responder chain says "these are the current places where a
request may be answered or a fact may be delivered." They should stay typed at
the edges and erase only inside routing machinery.

`context`

Owns the command invocation environment: its source and the narrow
capabilities available for that query or invocation. Runtime supplies the
capabilities and retains their engines. A command that needs visual text
motion receives the text-owned `selection::CaretMap` contract; the concrete UI
layout service never crosses into command context. Capability absence keeps
its established domain meaning rather than becoming a generic service-locator
failure.

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
`HistoryGroup` carries its coalescing window: generic groups use the command
default, while document typing supplies `text::edit::TYPING_UNDO_COALESCE_WINDOW`
so the runtime timeline and text buffer consume the same typing-pause fact.

`clipboard`

Owns clipboard representations and the outcome of synchronizing them with the
configured backend. Public reads return `Result<Option<T>>`: `Ok(None)` means
the clipboard was read and was empty, while `Err` means it could not be read.
Writes stage representations and publish them only after a system write is
confirmed. Copy reports that result, Cut deletes only after `Ok(())`, and Paste
keeps empty distinct from failed. No adapter may log an OS failure and then
report success to its caller.

Clipboard is an independent capability owner. Runtime configures one shared
handle and command context transports access to it; neither layer rewraps its
result model or clones a second handle for an individual operation. The
optional system adapter is clipboard realization, not evidence that clipboard
belongs to runtime orchestration.

The `Paste` command also owns its availability policy. No configured clipboard
or a confirmed empty clipboard disables it; confirmed text enables it. A probe
failure keeps it enabled so invocation can report the unavailable outcome
instead of each text target silently inventing a different fallback.

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
Snapshot restore has one runtime-owned reset for transient composition,
animation, overlay, task, gesture, history-group, and layout-cache state.

Frame preparation is one runtime recipe. A prepared frame carries its layout,
base scene, overlay layers, IME geometry inputs, and animation consequences;
realization capabilities decide whether each layer joins the parent scene or
becomes a native popup. Headless and native callers supply capabilities rather
than selecting recipe behavior with mode booleans.

**Presentation clock.** Events and frames are separate clocks. Input applies
model mutations, cumulative deltas, discrete commands, and window-local
session changes immediately and in order. It strengthens one pending
invalidation and requests a native redraw; it does not prepare or present a
frame. `RedrawRequested` samples the latest truths once. Coalescing therefore
removes obsolete candidate frames, never semantic input.

Application `state::Revision` and per-window `PresentationEpoch` name different
facts. Revision is model truth. The epoch advances presentation freshness for
scroll, resize, focus, hover, animation, and other session or visual changes
that need not mutate the model. A prepared frame captures an epoch and
candidate layout, but prepared is not presented: only a successful platform
receipt acknowledges the epoch and promotes that layout to
`PresentedGeometry`. Skipped, lost, occluded, or otherwise unsuccessful
attempts leave the previously visible geometry authoritative.

Pointer input is interpreted through last-presented geometry, because the user
cannot target a private candidate they never saw. Interaction retains logical
pointer position, physical surface, and modifiers as truth; hover is a
projection of that point through visible geometry, while cursor is a projection
of the one prospective primary press. Both are rederived before changed
geometry paints. Capture continues to route gestures to retained identity and
preserves its resolved cursor independently of ordinary hover. With no
presented geometry, geometry-dependent input is inert.

Direct manipulation updates its session sources. In particular, table column
width overrides project into retained composition before layout, so divider
movement requests layout rather than rebuilding application view structure.
Width-sensitive shaping and virtual measurement are paid once for each width
selected by a frame, not once for every raw pointer message.

`diagnostics`

Owns framework-visible counters and sample windows that turn performance and
interaction reports into numbers. Diagnostics are not behavior inputs; they are
instrumentation read by tools, tests, and debug panels.

Diagnostics is an observer seam, not the declaration owner for facts produced
by another subsystem. The renderer owns `RenderReport` and its private draw
facts; `diagnostics::RenderReport` is an exact public projection of that one
declaration, and diagnostics consumes it into counters and sample windows.
Renderer code does not import diagnostic aggregation.

Layout likewise owns the public `Text` fact assembled from its author-overflow
counter and the text engine's layout receipts. `diagnostics::Text` is the exact
public projection of that declaration; diagnostics accumulates it without
making layout import its observer. The public projection's name is the
declaration's canonical name—there is no compound layout-diagnostics alias.

The text editor debug panel is the current full instrument panel. Its one
instrument map is:

| Instrument | Owner | Signals |
| --- | --- | --- |
| `Text layout` | `text::layout`, `text::edit` | author overflows, paint calls, metric calls, visible and shaped lines, overscan segments, overlay and highlight work |
| `Text caches` | text layout caches | line hits/misses, render-surface calls, render-cache hits/misses, render source lines and bytes |
| `Scroll` | interaction and text viewport services | wheel events, offset changes, redraw requests, committed frame scrolls, text area viewport work |
| `Frames` | runtime presentation | full redraws, view rebuilds, layout recomposes/reuses, text surfaces |
| `Render` | native renderer and `diagnostics::Render` | frames, interval/acquire/draw p95, `key->present` p95, pending key samples, promoted groups, filter pool sizes |
| `wgpu_l3::render::filter_params` | filter encoder | filter pass uniforms and source/target rects |
| `wgpu_l3::render::material` | pane material path | pane source/target facts and material layer sequence |
| `wgpu_l3::overlay::fade` | overlay runtime | entry opacity, schedule, frame number, and demotion timing |
| `wgpu_l3::overlay::backend` | overlay runtime | entry material realization, backend preference, resolved backend, and fallback capability flags |
| `wgpu_l3::native_popup` | native platform | popup shell style, geometry, routing, and native-window lifecycle decisions |

Render latency samples are presentation-epoch-tagged: an input sample records
only when a successfully presented frame acknowledges the epoch it requested.
This covers session-only scroll, hover, focus, and resize changes without
overloading model revision. `key->present` means input-to-present-call, not
input-to-glass.

The log-target rows above are the compositor diagnostic catalog. They stay quiet
under the examples' default `RUST_LOG=info`; enable only the narrow target under
investigation instead of raising the whole app to debug.

`platform`, `host`, and native/render adapters

Own the boundary with the operating system, window system, GPU, renderer,
clipboard, dialogs, and native event loop. Renderer dependencies belong at this
edge unless a lower rendering vocabulary is explicitly being defined.
Native paint adapters carry the window scale factor into layout-to-paint
conversion so monitor moves and fractional DPI changes re-snap the same layout
truth to the new device grid.

`platform::launch(app)` is the ordinary application ceiling: it folds a view
runtime through Shell and the native Runner and supplies the system clipboard
only when the runtime still carries its untouched default. `with_clipboard`
records an explicit choice that launch preserves. Shell, Host, Platform,
Runner, and `platform::run(shell)` remain public lower-level seams for tests and
advanced adapters.

### Public API Rule

Central concepts are re-exported. Supporting concepts stay namespaced.
Internal routing details stay private.

This rule keeps call sites readable while preserving context for narrower
types. A type should move toward a module root only when it is part of that
module's public meaning, not merely because many files mention it.

When a public module and its central type have the same name, the parent
re-exports only that type. Call sites import the pair as `{module, Module}`;
supporting concepts keep simple declarations inside the module and are spelled
`module::Type`. Do not retain a compound declaration only to re-export it under
a shorter name, including at a parent module: rename the declaration to the
exported name and let callers use the module namespace when distinction is
needed.

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

Late viewport chrome carries its owner, ordered paint layer, and originating
clip stack. Focus outlines and scrollbars replay that provenance after inline
content; scrollbar hit testing consumes the same scope rather than recovering
an owner clip through a parallel path. Selection remains inline geometry, not
late chrome.

Derived geometry is projected once from its owning truths and consumed
everywhere, placement included. Interaction may target a projection, but its
effects mutate the projection's sources. A table track is projected from the
application dimension, session resize override, and available layout extent;
cells, rules, resize zones, editors, and horizontal scroll extent consume that
projection rather than re-deriving it.

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
The current census records legitimate layer twins: view/scene/shell
`Presentation` types are qualified projection stages, while animation/layout/
render/diagnostics `Frame` types mean a time sample, derived node, acquired
texture, and counter set respectively. No import scope aliases either word;
the first required `Presentation as ...` or `Frame as ...` alias reopens the
rename decision.

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
