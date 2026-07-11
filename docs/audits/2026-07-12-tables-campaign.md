# Tables Campaign — 2026-07-12

This is the crash-safe ledger for the six-checkpoint campaign that turns the
tables arc into real editable record tables. The campaign begins at pushed
commit `4083ad33` (`Record tables arc constitution: six slices, pinning rule,
identity tuple`) with a clean worktree and `HEAD == origin/master`.

## Constitution

Only one checkpoint may be in progress. Each checkpoint begins with a current
tree census, ends independently green, deletes any displaced path, updates this
ledger and the roadmap, and records public API flags and pending human-eyes
work. No red state, unexplained behavior, or parallel representation crosses a
checkpoint boundary.

Tables are a provided-container species. Their public children are derived
from a provider and a visible range rather than exhaustively declared. Identity
is fixed for the campaign:

```text
row identity    = provider key
column identity = column id
cell identity   = (table id, row key, column id)
```

Only visible rows participate in measurement. Dematerialization is not
removal. Focused, captured, or actively edited rows pin; selected rows do not.
Pinned rows may remain clipped and unpainted. Actual provider deletion ends the
row and its transient state.

Public API follows the house naming rule: central concepts share and are
re-exported from their module; supporting concepts remain namespaced with
simple names. No compatibility aliases are added around provisional names.

An unresolved constitution-level product, public-API, visual, or accessibility
decision halts at the last green checkpoint. It does not authorize improvising
through the remaining campaign or redefining completion.

## Baseline

- Starting and remote commit: `4083ad33`.
- Worktree: clean.
- Library: 815 passed, 8 deliberately ignored, 0 failed; harness work 0.81 s.
- Example smoke executables: text editor, control gallery, and glass tuner
  exited successfully.
- Protected state: `examples/glass_tuner/app/state.rs` contains
  `comparison_open: true`.
- Release text acceptance benchmark:
  - 8 MiB load: 33.812 ms;
  - typing at 10 B / 2.5 MB / 5 MB / 10 MB:
    2.726 / 3.557 / 3.744 / 3.587 µs per edit;
  - clone at 10 B / 10 MB: 35.590 / 37.097 ns;
  - benchmark test work: 0.72 s.
- Historical comparison from the Practiced Constitution: 29.531 ms load,
  3.344–3.898 µs/edit at large sizes, and 35.329/35.647 ns clones. The fresh
  measurements above are the authoritative same-checkout baseline; both sets
  remain in the ledger so noise is not mistaken for a checkpoint regression.

## Checkpoint ladder

| Checkpoint | Contract | Status | Boundary proof |
| --- | --- | --- | --- |
| 1 | Text overflow and three text kinds | Complete | 824 passed + 8 ignored in 0.87 s; three smokes; release text gate held |
| 2 | Typed `FrameContent` over the existing roles | Complete | One content discriminant; 825/8 in 0.88 s; three smokes; release gate held |
| 3 | Uniform virtual region/list | Complete | Stable provider keys, bounded two-pass materialization, pin/deletion laws, 832/8 and three smokes |
| 4 | Keyed selection and active item | Complete | Window/list-scoped stable keys, all-except select-all, bounded reveal, 840/8 and three smokes |
| 5 | Read-only record table | Complete | Public-node cells over selectable VirtualList; weighted tracks, resize/sort/grid/scale matrices, 847/8 and three smokes |
| 6 | Editable cells | In progress | Numeric plus textual/enumerated editors with stable cell identity |

## API flags

Checkpoint 1 added no public module, provider, callback, trait, or compatibility
alias. It added:

- re-export `text::Overflow` with variants `Clip`, `EllipsisEnd`, and
  `EllipsisMiddle`;
- constructor `view::Node::world_text(text, overflow)`;
- constructor `widget::Label::world(text, overflow)` on the existing widget;
- diagnostic field `diagnostics::Text::author_text_overflows`;
- read-only scene projection `scene::Text::overflow()`.

The names follow the house idiom: `Overflow` is supporting vocabulary
namespaced under `text`; the existing `Label` and `Node` concepts gain narrow
constructors rather than an alias or a new widget. No naming uncertainty is
carried into checkpoint 2.

Checkpoint 2 added no public API. `FrameContent`, its family payloads, and the
single-representation witness are crate-internal layout structure.

Checkpoint 3 added one public module, one re-exported central concept, and no
compatibility aliases:

- module `virtual_list`;
- root re-export `VirtualList`;
- supporting `virtual_list::Key` with `new`, `value`, and `From<u64>`;
- synchronous `virtual_list::Provider` with `len`, `key`, `index_of`, `row`,
  and default `is_empty`;
- `VirtualList::new(id, row_height, provider)` plus bounded `overscan`, and the
  ordinary `width`, `height`, `max_height`, and `background` style builders.

`Provider::row` returns an ordinary public `view::Node`; there is no parallel
row-widget species. Runtime materialization and logical focus preparation are
crate-internal. The names follow the module doctrine and compiled in the honest
control-gallery caller without aliases; no checkpoint-3 API flag remains.

Checkpoint 4 added one public module and root re-export:

- module and root type `selection::Selection` / `Selection`;
- `Selection::{new, len, is_empty, contains, is_all, anchor, active}` and
  `Default`;
- `VirtualList::selectable()`;
- read-only `Session::selection(window, list_id)`;
- `view::Node::is_active_item()` for projected-view inspection;
- modifier-aware `Runtime::pointer_down_at_with_modifiers` and
  `Shell::pointer_down_with_modifiers`;
- current modifiers on host/shell `PointerDown`, so native input reaches the
  same toggle/range state machine as headless input.

Mutation remains input-owned and there is no public setter, duplicate app
model, selection callback, prefixed alias, or table-specific selection type.
The host event field is a deliberate source-level contract addition required
to make modifier selection real on the native path; it remains a morning API
review flag because downstream event constructors must now supply modifiers.

Checkpoint 5 added one public module and root concept, plus two narrow shared
vocabulary additions:

- module `table` and root re-export `Table`;
- synchronous `table::Provider::{len, key, index_of, cell, is_empty}`, where
  `cell` returns an ordinary public `view::Node`;
- `table::Column::{new, header, id, label, width}`;
- `table::Width::{Fixed, Weight}` plus `fixed` and `weight` constructors;
- `Table::new(id, row_height, columns, provider)` and ordinary
  `header_height`, `width`, `height`, `max_height`, and `background` builders;
- public stable `table::Cell` with `table`, `row`, and `column` accessors;
- read-only `Session::active_table_cell(window, table)`;
- weighted shared layout vocabulary `view::Dimension::Weight` and
  `Dimension::weight`;
- `pointer::Cursor::ResizeHorizontal`.

Header dividers, row/header metadata, track projection, and mutation of active
columns/resized widths remain crate-internal. The public provider does not own
sorting, resizing, selection, or edit state. Morning review flags are whether
`Width` should remain a table-facing declaration beside public weighted
`Dimension`, and whether every v1 column being resizable by default is the
right minimal policy. No compatibility aliases were added.

## Pending eyes

- Checkpoint 1 (implemented, morning eyes remain non-blocking): ellipsis glyph
  appearance, cut spacing, end/middle appearance in both themes, and logical
  source-order bidi behavior.
- Checkpoint 3 (implemented, morning eyes remain non-blocking): scrollbar feel
  and row density in the one-million-row control-gallery witness.
- Checkpoint 5 (implemented, morning eyes remain non-blocking): striping,
  rules, sticky-header behavior, resize feel, selection
  visuals, and density.
- Checkpoint 6: editor placement, rejection presentation, focus transitions,
  and keyboard commit/cancel feel.

## Checkpoint 1 census — text overflow

Ownership claim: overflow is text-layout meaning. View declares the kind and
policy; layout supplies constraints; the independent text engine resolves
grapheme-safe displayed text and cache identity; scene paints only the resolved
text. Author overflow reporting belongs to diagnostics, never paint policy.

The v1 bidi policy will be chosen from shaped/layout evidence and recorded
before the checkpoint closes. No caller-local substring operation may satisfy
these cells.

| Cell | Scenario | Required result | Existing mechanism / missing part | Status |
| --- | --- | --- | --- | --- |
| C1-01 | Vocabulary and structural absence | `Clip`, `EllipsisEnd`, and `EllipsisMiddle` are text-layout concepts; world text cannot be constructed without one | `text::Overflow`; required policy in `Node::world_text` and `Label::world`; no role or widget added | Held |
| C1-02 | Fits within width | All three policies preserve byte-identical visible text and normal measurement | Resolver returns source before policy branching when real metrics fit | Held |
| C1-03 | End ellipsis | Long world text yields a real `…`, preserves the maximal fitting head, and never splits a grapheme | Grapheme candidate search in `text::layout::overflow`; ZWJ witness | Held |
| C1-04 | Middle ellipsis | Long path-like text preserves fitting head and tail around one `…` without splitting graphemes | Balanced head/tail search; path and combining/ZWJ witnesses | Held |
| C1-05 | Tiny/zero constraints | Resolution is deterministic when even the ellipsis cannot fit; no negative or looping search state | Zero and NaN resolve to empty; Clip preserves source | Held |
| C1-06 | Cache identity | Same text/metrics/constraints with different overflow modes cannot share a cached result; unchanged mode still hits | `Overflow` added to inline `TextKey`; mode miss and ordinary-hit tests pass | Held |
| C1-07 | Author diagnostic | Width-constrained author text that does not fit reports a diagnostic condition rather than silently becoming world text | Public counter aggregates through runtime diagnostics and appears in the editor panel | Held |
| C1-08 | User text non-regression | TextArea scroll/wrap and TextBox reveal/preedit/caret remain unchanged | Full 824-test matrix, including existing area/field journeys, passes | Held |
| C1-09 | Honest world caller | At least one programmer-uncontrolled value declares its overflow at construction | Full file path uses middle ellipsis; status uses end ellipsis in text-editor debug panel | Held |
| C1-10 | Bidi | End and middle behavior follow one documented v1 visual/logical policy with deterministic mixed-direction witnesses | Logical source-order policy recorded in `master_design.md`; RTL end witness passes | Held |

### Existing text path inventory

- `text::layout::inline::TextKey` owns cached inline shape identity: document
  metrics, width, height, and `WrapKey`.
- `text::layout::inline` prepares glyphon buffers and is the lowest existing
  owner that can measure candidate visible text with the real font system.
- `layout::text` and `layout::Engine` are the framework bridge for label width
  and constrained label size; they must consume text-engine overflow results,
  not reproduce truncation.
- `view::Node` has private role/label/control fields and narrow constructors,
  so an author/world distinction can be structurally enforced without adding
  a widget or a new view role.
- `scene::Text` currently carries already-resolved value, bounds, style, wrap,
  and alignment. Scene must not learn truncation policy.
- `view::TextArea`/`TextBox` use separate user-text layout paths and must not be
  routed through world-text overflow.
- Existing cache tests distinguish color-insensitive hits, metric misses,
  bounded eviction, and repeated measurement. Existing widget/layout tests
  witness label wrap and user-text behavior.

## Checkpoint 2 census — typed `FrameContent`

Current-tree ownership claim: `Frame` has one common envelope for node identity,
path, geometry, active geometry, focus/selection presentation, overlay policy,
background, clip, generic target/binding/action, and the label projection shared
by many roles. Role-specific semantic payload belongs in one `FrameContent`.
The current parallel cluster is `viewport`, `text_area_layout`,
`text_box_layout`, `text_box_text_rect`, `slider_track_rect`, `checkbox`,
`radio`, `text_area`, `text_box`, `slider`, and the three shortcut fields;
`text`, `text_wrap`, and `world_text_overflow` are derived text-family payload.

The existing house pattern is `view::Control`: one enum owns mutually exclusive
typed leaf models and narrow accessors project them. `FrameContent` will use the
same pattern. `Frame::role()` will derive from content, deleting the independent
role discriminant so role/payload disagreement is structurally impossible.
Generic binding/action remains common because `Element` may truthfully bind
structural presentation roles; it is not a leaf-role payload.

### Family migration map

| Migration family | Roles | Truthful payload after migration | Existing downstream consumers |
| --- | --- | --- | --- |
| Structural | Root, Stack, MenuBar, Panel | restricted structural discriminator; no leaf payload | layout algorithm, composition identity, generic background/children paint |
| Choice | Checkbox, Radio | one choice enum carrying exactly one typed model | active rect, shared choice paint, checked/selected state |
| Text | Label, SectionHeader, TextArea, TextBox | label kind/overflow or typed area/field model plus owned layout geometry | measurement, hit/drag caret mapping, runtime text projection, scene text paint |
| Slider | Slider | model plus derived track rect | hit value mapping, capture/gesture action, scene paint, native presentation transform |
| Scroll and floating | Scroll, FloatingPanel | scroll viewport only on Scroll; floating remains a unit semantic variant | reveal/wheel/chrome and overlay/native layer routing |
| Remaining specialized | Menu, Binding, Separator, Button | bound presentation follows any truthfully bound role; Separator alone carries an unbound reserved menu column; roles remain unit variants | menu/palette row paint, shortcuts, menu actions, button paint |

The complete widget-audit downstream index remains the migration navigation
map: `layout/algorithm`, `layout/measure`, `layout/frame`, `layout/chrome`,
`runtime/{pointer,visual,presentation}`, `scene/paint` and its choice/slider/text
modules, and Slider's native-paint branch. No phase-local policy is being
consolidated merely because it also inspects a role.

| Cell | Scenario | Required result | Current mechanism / migration proof | Status |
| --- | --- | --- | --- | --- |
| C2-01 | One discriminant | `Frame` stores `content: FrameContent`; `role()` derives from it | Independent role deleted in first family commit | Held |
| C2-02 | Structural family | Root/Stack/MenuBar/Panel cannot carry leaf payload | Restricted `StructuralRole` inside `FrameContent` | Held |
| C2-03 | Choice family | Checkbox and Radio share geometry/paint family but carry exactly one correct model | `ChoiceContent::{Checkbox, Radio}` | Held |
| C2-04 | Text family | Label/header/area/field payloads are exclusive; user-text layout and world overflow retain behavior | `TextContent` variants own model/layout/geometry; 141 focused text/layout journeys held | Held |
| C2-05 | Slider | Slider model and track geometry travel together and no other role can hold them | `SliderContent`; widget, scene and native-transform slider journeys held | Held |
| C2-06 | Scroll/floating | Viewport mutation is legal only for Scroll; floating behavior stays unit/common presentation | `ScrollContent`; 41 scroll and 8 floating focused journeys held | Held |
| C2-07 | Remaining specialized | Shortcut display/width follows truthful bound roles; Separator alone owns unbound row reservation | `BoundContent` plus `SeparatorContent`; reduced menu/palette journeys held | Held |
| C2-08 | Interaction equivalence | Hit, action, drag, focus, capture and reveal journeys remain byte-for-behavior equivalent | Existing narrow accessors retained; full suite held | Held |
| C2-09 | Presentation equivalence | Paint, overlay, chrome and Slider native transform retain exact branches | Downstream consumers unchanged; full suite and smokes held | Held |
| C2-10 | Single representation | Legacy role and optional payload cluster are absent; one source witness pins this | `frame_content_is_the_single_role_payload_representation` passes | Held |
| C2-11 | Performance/non-scope | Suite gauge and release text benchmark hold; no table role/API/feature appears | 825/8 in 0.88 s; release figures below; no public API or role added | Held |

Expected exclusions: no public API, role, widget, table behavior, capability
table, generic property bag, or phase-policy consolidation. Family commits may
temporarily coexist with unmigrated legacy fields, but every family commit must
be green and the checkpoint boundary may contain only `FrameContent`.

## Checkpoint 3 census — uniform virtual region/list

Ownership claim after inspecting the current tree: virtualization is not a
second widget runtime. A provided list remains a view node and uses the normal
retained composition, command resolution, transient interaction projection,
layout frames, viewport/chrome, scene paint, and runtime pruning sentence. The
new coordination seam is a bounded two-pass fixed point: layout derives a
logical visible range from real viewport geometry; runtime rebuilds the normal
view with only that range plus pins; retained composition reconciles provider
keys; layout repeats once against the requested materialization.

Existing mechanics to reuse:

- `layout::Viewport` already owns clamped scroll offsets, content extent,
  reveal geometry, wheel consumability, clips, and scrollbar projection.
- `composition::Tree` already reconciles explicit keyed siblings and reports
  true removal to session pruning; provider keys extend that existing match
  rule rather than creating a parallel identity map.
- `view::Node` already projects focus, capture-relevant pointer targets,
  TextArea/TextBox interaction, bindings, and focus order through the retained
  tree. Provider rows must become ordinary public nodes before those passes.
- `draft::Input` already retains inactive drafts with a bounded store and
  deletes them only when composition reports actual identity removal.
- runtime rebuild and layout invalidation are already distinct. Virtual range
  convergence may request one ordinary rebuild inside frame preparation; it
  does not ask applications to calculate ranges.

Dematerialization must therefore be a distinct composition outcome from
removal. A materialized provider child leaving the visible/pinned set drops its
view node without contributing removed node/element identities while its key
still exists in the provider. If `Provider::index_of(key)` says the key no
longer exists, normal removed-subtree reporting and session pruning apply.

### Shipped public contract

The narrow v1 provider surface is synchronous and flat:

- total logical length;
- stable `virtual_list::Key` for an index;
- efficient reverse lookup from key to current index, needed for reorder,
  pinning, and deletion truth without scanning one million rows;
- public row construction returning an ordinary `view::Node` for a requested
  logical index.

The list constructor requires a stable list id, uniform positive row height,
and a provider. Overscan is a small bounded setting. Variable heights, async or
streaming rows, selection, headers, tracks, and table semantics are excluded.
The honest control-gallery caller compiles against `VirtualList` and
`virtual_list::{Provider, Key}`. No alternate or prefixed naming path exists.

| Cell | Scenario | Required result | Existing owner / missing seam | Status |
| --- | --- | --- | --- | --- |
| C3-01 | Structural grammar | One provided-container node derives public row children; no exhaustive app-authored child list | `VirtualList` materializes ordinary provider-built nodes | Held |
| C3-02 | Initial viewport | First frame materializes only visible rows plus bounded overscan | Bounded bootstrap converges to the layout-requested range in one rebuild | Held |
| C3-03 | Small and jump scroll | Nearby and million-row jumps request correct ranges with bounded provider calls | Uniform range arithmetic and 600,000-row jump witness | Held |
| C3-04 | Growth and shrink | Content extent follows count; offsets clamp; stale rows do not survive true shrink | Provider count/reverse lookup drives extent and deletion | Held |
| C3-05 | Stable-key reorder | Visible retained row identity follows provider key rather than logical index | Provided keys extend local retained sibling matching | Held |
| C3-06 | Viewport resize and scale | Logical range updates for new viewport height; scale-only changes preserve logical identity | Resize witness updates bounds; platform keeps layout in logical units across scale | Held |
| C3-07 | Measurement bound | Only materialized visible/pinned rows produce nodes and frames; no provider-wide intrinsic scan | Uniform height computes extent without provider row construction | Held |
| C3-08 | Paint bound | Paint items and per-scroll work remain proportional to viewport rows plus overscan/pins | Scene/frame/provider-call counters remain bounded | Held |
| C3-09 | Focus pin | Focused row remains materialized and may be clipped when scrolled away | Retained focus target maps to provider key | Held |
| C3-10 | Capture pin | Captured row remains materialized until capture ends | Retained pointer capture maps to provider key; deletion releases it | Held |
| C3-11 | Active edit pin | Text row with active draft/preedit remains materialized | Draft input target pins independently after focus clears | Held |
| C3-12 | Selection non-pin | No selection concept or accidental pin appears in checkpoint 3 | Selection remains absent and deferred to C4 | Held |
| C3-13 | Dematerialized draft | Inactive TextBox draft survives ordinary scroll out/back | Dematerialization suppresses removal; rematerialization restores the draft | Held |
| C3-14 | Provider deletion | Deleted key ends focus/capture/edit/draft through normal pruning | Missing reverse lookup emits ordinary retained/element removal | Held |
| C3-15 | Focus-before-move | A logical target is materialized before focus transfer | Keyed pin is installed and presented before runtime focus changes | Held |
| C3-16 | Million-row witness | Node/frame/paint/provider-call counts stay bounded across initial, jump, reorder, shrink and resize journeys | Six deterministic virtual-list journeys bound all new work | Held |
| C3-17 | Honest caller | Control gallery exercises a real large provided list made of public widgets | One-million-row gallery provider builds ordinary Element/Label rows | Held |

## Checkpoint 4 census — keyed selection and active item

Current-tree ownership decision: generic row selection is window-local runtime
interaction state. It exists because a user is operating a presented list, does
not dirty application data, must not enter application undo, and must not leak
between two windows or two list ids. The existing command-palette selected
index proves the session/interaction owner, while the master doctrine already
separates active item from keyboard focus. Unlike the palette's small filtered
index, the reusable model keys every durable fact by `virtual_list::Key`.

The framework stores one selection per `(window, list id)`. Applications may
inspect a read-only public `selection::Selection` through `Session`; mutations
come from pointer/keyboard input, not a second app-owned copy. Membership uses
explicit keys for ordinary selections and all-except for select-all, so one
million selected logical rows require constant state and do not materialize
rows. Anchor and active are optional stable keys. Provider reverse lookup and
key construction remain the only order oracle.

Selection reuses the landed virtual-list path: provider rows remain ordinary
nodes; selected/active presentation projects through existing view/frame visual
state; range and navigation use provider indices without scanning nodes;
offscreen selected rows never enter the pin set; an offscreen active target is
materialized before reveal or any later focus transfer. No text-selection
mechanic, app history entry, alternate provider identity, or table role is
admitted.

| Cell | Scenario | Required result | Existing owner / missing seam | Status |
| --- | --- | --- | --- | --- |
| C4-01 | Empty | New selectable list has no members, anchor, or active key | Present installs an empty session selection | Held |
| C4-02 | Single | Plain click replaces membership and sets anchor/active | Uniform coordinate or retained row ancestry resolves the key | Held |
| C4-03 | Modifier toggle | Primary-modified click toggles one stable key without index identity | Host pointer modifiers reach the shared state machine | Held |
| C4-04 | Range | Shift selection spans current provider order from stable anchor to target | Provider order constructs the stable-key range | Held |
| C4-05 | Select all | Primary+A selects one million rows without one million stored keys or views | All-except membership is constant state | Held |
| C4-06 | Anchor and active | Anchor survives ordinary extension; active follows the navigation endpoint | Public stable-key facts are independent from membership | Held |
| C4-07 | Keyboard navigation | Arrows/Home/End/Page move active by provider order and optionally extend | List focus scope plus real viewport page size | Held |
| C4-08 | Reorder persistence | Membership, anchor, and active follow keys while indices change | Reverse lookup refreshes cached indices without changing keys | Held |
| C4-09 | Growth/shrink | Growth preserves state; deleted selected/anchor/active keys reconcile deterministically | Explicit/all-except membership and nearest selected fallback | Held |
| C4-10 | Departure/restoration | Window/list scoping survives runtime snapshot restore and clears on actual window departure | Window snapshots carry selections; departure owns the window | Held |
| C4-11 | Unmaterialized movement | Navigation can target a logical offscreen key without scanning or constructing intervening rows | One active key is temporarily materialized for reveal | Held |
| C4-12 | Focus ordering | Any focus transfer happens only after target materialization; active item itself does not steal focus | List retains focus; C3 focus-before-move witness remains green | Held |
| C4-13 | Selection non-pin | Large offscreen selection remains unmaterialized; only focus/capture/edit pin | C3 collector excludes selection; pending reveal pins only active once | Held |
| C4-14 | Large complexity | Million-row select-all and navigation keep state, view, frame, paint, and provider work bounded | Constant all state and bounded frame/provider counters | Held |
| C4-15 | Independence | Two list ids and two windows never share membership/anchor/active | Per-window interaction store keyed by list id | Held |
| C4-16 | Visual projection | Materialized selected/active rows use existing selected visual truth without a parallel paint system | Transient projection feeds existing Frame/Visual row tint | Held |

## Checkpoint 5 census — read-only record table

Current-tree composition claim: Table is a provided-container composition, not
a monolithic semantic leaf. Its sticky header is an ordinary horizontal
container outside the existing selectable `VirtualList`; each materialized row
is an ordinary horizontal container; each cell is the public `view::Node`
returned by the table provider. Table owns track declaration and cell identity,
while existing layout, composition, selection, overflow, focus, scene, runtime,
and provider mechanics keep their current jobs.

The missing reusable primitive is weighted grow allocation. Add weight to the
existing `Dimension`/flow allocator rather than implementing separate table
width arithmetic. Explicit columns use fixed dimensions; weighted columns use
the same row allocator for header and materialized rows. Window-local resized
width overrides belong to session interaction presentation, keyed by table and
column id, and are projected into the table before rows materialize. Provider
data is never mutated.

Stable identities are exactly the campaign tuple. The virtual-list provider key
owns the row. A column owns a stable `interaction::Id`. Table tags each public
cell with `(table id, row key, column id)` so retained composition can match
cells independently of current row/column indices. Headers use `(table id,
column id)`. Sorting is an ordinary typed command bound by the application to a
public header widget; Table emits that intent and never reorders provider data.

| Cell | Scenario | Required result | Existing owner / missing seam | Status |
| --- | --- | --- | --- | --- |
| C5-01 | Public composition | Table provider returns ordinary public cell nodes; no table-only cell widget hierarchy | `table::Provider::cell` returns Node; rows compose through VirtualList | Held |
| C5-02 | Identity | Table, row, column and cell retain `(table, row key, column)` truth through reorder | Composition keys include cell/header tuples under provider-keyed rows | Held |
| C5-03 | Explicit tracks | Fixed widths align header and every visible row | Header and row use the same fixed Dimension declarations | Held |
| C5-04 | Weighted tracks | Remaining width distributes by declared weights with deterministic remainder | Shared flow allocator owns weighted remainder distribution | Held |
| C5-05 | Resize | Captured divider drag changes one window/table/column presentation width without provider mutation | Existing capture route updates session table presentation before materialization | Held |
| C5-06 | Sticky header | Vertical body scroll never moves the header | Ordinary header is outside the VirtualList viewport | Held |
| C5-07 | Sort intent | Header activation emits an application command and provider order changes only if the app changes it | Gallery Button emits `SortRecords`; app state reverses provider mapping | Held |
| C5-08 | Striping and rules | Visible rows stripe deterministically and cell boundaries use scene/layout truth | Scene reads row index and cell/header frame metadata; Rule remains raster owner | Held |
| C5-09 | Cell overflow | Provider world text declares and obeys overflow within the allocated track | C1 overflow passes unchanged through public cell nodes | Held |
| C5-10 | Public widget cells | Buttons, choices, labels and other public nodes retain normal action/focus behavior in cells | Gallery and tests use Button, Checkbox and world Label cells | Held |
| C5-11 | Keyboard grid | Logical row/column active position navigates without scanning unmaterialized rows | C4 owns row; session stores one column id; End witness stays bounded | Held |
| C5-12 | Visible measurement | No intrinsic scan of unmaterialized provider rows occurs | Million-row provider-call witness remains bounded to visible rows/columns | Held |
| C5-13 | Reorder/shrink | Row selection and cell identity follow keys; deleted rows reconcile normally | Reorder emits no retained churn; shrink follows C3/C4 reconciliation | Held |
| C5-14 | Independence | Two tables and two windows never share selection or resized tracks | Runtime witnesses isolate identical columns by window and table id | Held |
| C5-15 | Scale matrix | 1.0, 1.25, 1.5 and 2.0 preserve logical track alignment and stable device snapping | Integer logical alignment plus shared Rule matrix at all four scales | Held |
| C5-16 | Honest large caller | Control gallery exercises large data, sort intent, resize, selection, widget cells, truncation and sticky header | Real million-record Table is the gallery caller and sort journey | Held |

## Execution ledger

| Run | Checkpoint / cells | Result | Evidence |
| --- | --- | --- | --- |
| E-000 | Campaign baseline | Held | 815/8 library result in 0.81 s; three smokes green; release benchmark recorded above |
| E-001 | C1 focused mechanics | Held | 6 overflow, 5 inline-cache, layout/scene, author-diagnostic, and editor projection witnesses passed |
| E-002 | C1 first full library gate | Failed and reduced | 821 passed, 3 failed, 8 ignored; all failures asserted the displaced combined debug string |
| E-003 | C1 reduced failures | Held | All three updated provenance/diagnostic witnesses passed independently |
| E-004 | C1 full library gate | Held | 832 discovered: 824 passed, 8 deliberately ignored, 0 failed; harness work 0.87 s |
| E-005 | C1 external smoke gate | Held | `text_editor`, `control_gallery`, and `glass_tuner` all exited 0 |
| E-006 | C1 release text gate | Held | 31.167 ms load; 2.537 / 3.317 / 3.495 / 3.651 µs typing; 37.450 / 37.632 ns clones; 0.70 s benchmark work |
| E-007 | C1 caller-local truncation fixed point | Held | Removed the old path substring helper; repeated full suite 824/8 in 0.88 s and all three smokes exited 0 |
| E-008 | C2 family gates 1–5 | Held | 90 layout, 5 choice, 23 TextBox, 28 text-input, 11 slider, 41 scroll and 8 floating focused journeys passed |
| E-009 | C2 remaining-family first gate | Failed and reduced | Menu Separator rejected shortcut width; palette Label lost bound shortcut projection |
| E-010 | C2 reduced remaining family | Held | Four exact failures plus complete menu-popup and command-palette families passed |
| E-011 | C2 full library and smoke gate | Held | 833 discovered: 825 passed, 8 deliberately ignored, 0 failed in 0.88 s; three smokes exited 0 |
| E-012 | C2 release text gate | Held | 30.158 ms load; 2.730 / 3.702 / 3.872 / 3.621 µs typing; 35.557 / 37.060 ns clones; 0.67 s benchmark work |
| E-013 | C3 bounded mechanics | Held | Six virtual-list journeys cover initial frame, jump, resize, reorder, growth/shrink, all pin species, draft dematerialization/deletion, and focus-before-move |
| E-014 | C3 deletion reduction | Failed and reduced | Deleted id-less TextBox row emitted retained removal but not its focus element, leaving the draft orphaned |
| E-015 | C3 first full library gate | Failed and reduced | Empty materialization state compared `None` with an empty map, falsely requesting an endless second rebuild in ordinary apps |
| E-016 | C3 repeated full boundary gate | Held | 840 discovered: 832 passed, 8 deliberately ignored, 0 failed in 0.91 s; three smokes, fmt, diff, and protected state held |
| E-017 | C4 state-machine and integration matrix | Held | Four pure keyed-state witnesses plus click/toggle/range, million select-all, offscreen End reveal, reorder/delete, snapshot, two-window and two-list journeys |
| E-018 | C4 full boundary gate | Held | 848 discovered: 840 passed, 8 deliberately ignored, 0 failed in 0.88 s; three smokes, fmt, diff, and protected state held |
| E-019 | C5 focused composition matrix | Held | Million-row tracks/calls, sticky scroll, reorder/shrink, capture resize, two-window/two-table isolation, keyed grid navigation, sort intent and four-scale Rule snapping passed |
| E-020 | C5 full boundary gate | Held | 855 discovered: 847 passed, 8 deliberately ignored, 0 failed in 0.98 s; three smokes, fmt, diff, and protected state held |

## Failure and reduction ledger

The first C1 full run found three stale editor-debug expectations. Reduction
showed no behavior or ownership failure: each expected the former single string
that mixed authored diagnostics with file/status world data. Commit `3ef75719`
changed those witnesses to assert the new provenance split and the added author
overflow instrument. All three focused journeys and the repeated full suite
then passed. No unresolved failure crosses the checkpoint boundary.

The first C2 remaining-family gate disproved the census claim that shortcut
presentation belonged only to `Role::Binding`. The reduced evidence showed two
existing truths: palette results are bound Labels, and menu Separators reserve
the shared shortcut column without a binding. The final model therefore keeps
typed bound presentation common to any bound role and gives Separator only its
reserved width. Four exact failures and both complete families passed after the
correction. No failed representation crosses the checkpoint boundary.

The C3 deletion journey exposed an existing composition seam: id-less
TextBoxes use their focus element as draft identity, but retained composition
reported only explicit node ids as removable elements. The reduced ordinary
conditional-TextBox witness failed for the same reason. Composition now treats
text focus elements as reconciliation/removal identity, so real deletion prunes
drafts while ordinary dematerialization remains silent.

The first C3 full gate exposed an absence-normalization bug in the new runtime
coordinator: a window with no virtual lists represented state as `None`, while
the derived request set was an empty map. Treating those as unequal caused the
debug fixed-point assertion to fail across ordinary applications. The reduced
ordinary TextArea witness held after normalizing absence and empty state. The
repeated 832/8 gate then passed. No unexplained failure crosses checkpoint 3.

Checkpoint 4 produced no framework contradiction. Its first pointer-range test
attempted to click logical row 5 at y=110 in a 100-pixel viewport; reduction
showed the expected clip boundary correctly rejected the hit. Moving the
witness wholly inside the viewport held without a framework change. No failure
or parallel selection representation crosses the checkpoint boundary.

Checkpoint 5's first sort-intent journey found that the gallery declared and
targeted `SortRecords` but had not registered the command/responder in its
runtime. Command resolution therefore correctly hid the unsupported bound
header while the other headers and rows rendered. Registering the existing
typed intent made the ordinary Button header visible and the reduced journey
proved that only application state changes provider order. The first resize
assertion also assumed a center click while the shared helper chooses one pixel
inside a frame; expressing expected width as pointer x minus header x proved
the actual geometry contract. No unexplained failure crosses checkpoint 5.

## Commit ledger

| Checkpoint | Commit | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- |
| Campaign | `0fad3ff2` | 1 | 151 | 0 | Crash-safe ledger established |
| Campaign | `e3f1199f` | 1 | 7 | 6 | Checkpoint 1 marked in flight |
| 1 | `e6ce36b2` | 22 | 539 | 17 | Overflow owner, required declaration, cache identity, diagnostics, and tests |
| 1 | `c8deeb29` | 2 | 32 | 5 | Doctrine and honest text-editor callers |
| 1 | `3ef75719` | 2 | 10 | 3 | Reduced full-gate expectations updated |
| 1 | `dfa728f2` | 6 | 23 | 37 | Caller-local path truncation retired; world policy is sole display owner |
| 2 | `873b6c5f` | 1 | 54 | 0 | Current-tree family census recorded |
| 2 | `32c50996` | 1 | 81 | 9 | Independent role deleted; structural content introduced |
| 2 | `23e9a8f8` | 1 | 30 | 17 | Choice models made exclusive |
| 2 | `9afe26dc` | 1 | 110 | 62 | Text payload family migrated |
| 2 | `0acfa2c4` | 1 | 27 | 11 | Slider model and track migrated together |
| 2 | `e0964c13` | 1 | 14 | 7 | Scroll viewport restricted to Scroll content |
| 2 | `35736441` | 2 | 95 | 19 | Bound/specialized family finished; absence witness added |
| 3 | `1010098d` | 1 | 70 | 0 | Current-tree virtual-list census and provider contract recorded |
| 3 | `df35e116` | 23 | 1228 | 10 | Uniform keyed virtualization, bounded fixed point, pins, deletion laws, and deterministic witnesses |
| 3 | `f223d454` | 2 | 62 | 3 | Doctrine and honest one-million-row control-gallery caller |
| 4 | `acd87094` | 1 | 45 | 0 | Current-tree selection ownership and scenario census |
| 4 | `116cde39` | 32 | 1306 | 18 | Keyed selection, active item, native modifiers, snapshot scoping, bounded reveal and tests |
| 4 | `cd91ad95` | 2 | 20 | 0 | Selection doctrine and honest selectable gallery caller |
| 5 | `83aab550` | 1 | 48 | 0 | Current-tree table census and ownership decision |
| 5 | `da8e0c41` | 6 | 139 | 7 | Shared weighted flow allocation landed before Table composition |
| 5 | `d7d8cd98` | 39 | 1731 | 90 | Read-only Table composition, identities, resize/sort/grid/scale matrices, gallery, doctrine and full gate |

## Final fixed-point sweep

Pending all six independently green checkpoints, cross-checkpoint replay,
public API review, performance comparison, pending-eyes transfer, full ritual,
and clean-worktree proof.
