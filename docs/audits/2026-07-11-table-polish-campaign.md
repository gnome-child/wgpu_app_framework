# Table Polish Campaign — 2026-07-11

This is the crash-safe ledger for the six-checkpoint campaign that follows the
table grammar close at `8d595d63`. It corrects the first human-eyes findings
and establishes the baseline interaction grammar of an editable desktop data
grid without admitting spreadsheet semantics.

## Constitution

Only one checkpoint may be in progress. Every checkpoint begins with a current
tree census, consumes lower mechanics instead of reproducing them in `table`,
ends independently green, deletes displaced paths, and records its tests,
public API effect, non-merges, pending-eyes work, and commit receipt here.

Measurement uses projected geometry, allocation uses measured geometry, and
chrome uses visible geometry. Each projection is derived once and consumed by
every layout, paint, clip, hit-test, and interaction path in its domain.
Interaction may target a projection, but it mutates the projection's sources.

Display, current-cell focus, text selection, and editing are distinct session
truths. The existing read-only text surface owns selection and copy. Only the
editable text surface can express mutation. Application commands remain the
sole owners of provider order and value changes.

No replacement track solver, duplicate width model, new naming channel, new
read-only text mode, dormant display editor, paint-only scrollbar correction,
range/TSV spreadsheet behavior, or push is permitted.

## Baseline

- Starting commit: `8d595d63` (`Close table grammar campaign ledger`).
- Branch: `master`, 11 local commits ahead of `origin/master`.
- Worktree: clean.
- Library: 875 passed, 8 deliberately ignored, 0 failed; harness work 0.92 s.
- Formatting: `cargo fmt --all -- --check` passed.
- All targets: `cargo check --all-targets` passed.
- Example smokes: `text_editor`, `control_gallery`, and `glass_tuner` passed.
- Protected state: `examples/glass_tuner/app/state.rs` retains
  `comparison_open: true`.
- No push.

## Checkpoint ladder

| Checkpoint | Contract | Status | Boundary proof |
| --- | --- | --- | --- |
| 1 | Expanded rows consume intrinsic content at resolved track widths | Complete | `5af6d17b`; 876 passed, 8 ignored; exact 28/128 px rows, resize to 108 px, scroll invariance, generic variable-list preservation, three smokes and all boundary checks green |
| 2 | Internal table naming consumes the existing subject channel | Complete | `c401ebe9`; 877 passed, 8 ignored; retained subject and painted-text absence witnesses; three smokes and all boundary checks green |
| 3 | Active sort indication is trailing header structure, never label text | Complete | `503bd6d3`; 878 passed, 8 ignored; active-only caret geometry, both directions, wrapping, target and resize precedence, three smokes and all boundary checks green |
| 4 | Vertical scrollbar chrome consumes the visible body projection | Complete | `c8fbc548`; 879 passed, 8 ignored; overlay and gutter anchors, stable horizontal-scroll geometry, hit ownership, page extent, fully-clipped absence, three smokes and all boundary checks green |
| 5 | Read, select, navigate, then deliberately edit | Complete | `ab8b51fe`; 883 passed, 8 ignored; read-only selection/copy, deliberate edit activation, canonical keyboard movement, native bool control, virtualization pruning, three smokes and all boundary checks green |
| 6 | Resistance audit and campaign close-out | Complete | Structural absences held, complete six-commit stack reviewed, native Compact/Expanded eyes pass held, 883/8 final boundary green |

## Boundary ritual

At every checkpoint boundary record:

- focused geometry and behavior witnesses;
- full `cargo test --lib` result and ignored count;
- all three example smokes;
- `cargo fmt --all -- --check`;
- `cargo check --all-targets`;
- `git diff --check`;
- `comparison_open: true` preservation;
- compact million-row bounded-work preservation;
- API changes, non-merges, pending-eyes notes, commit hash, and diff statistics.

Checkpoint commits are not pushed.

## Checkpoint 1 census — expanded measurement

- Proven defect: `layout_variable_virtual_list` calls
  `resolved_height_for_width` with `i32::MAX`. A row is an implicit vertical
  grow stack, so allocation height can become the row measurement.
- Proven second derivation: the generic horizontal-stack intrinsic measure
  resolves each flexible child against the whole row width, while actual table
  placement consumes `table::Projection` tracks.
- Required owner: the existing table projection must provide the same cell
  widths to intrinsic row measurement and placement. The configured row height
  remains the floor.
- Weak witness to delete: expanded rows currently need only be `> 24`, so an
  absurd measurement passes.

## Checkpoint 1 verdict and boundary

- `layout_variable_virtual_list` now measures table rows intrinsically and
  applies the configured table row height only as a table-local floor. Generic
  variable lists retain their independent estimate-versus-explicit-height
  contract; the full suite caught and rejected an attempted global floor.
- `table_stack_intrinsic_height` consumes column widths from the same
  `table::Projection` used by placement. The helper also measures the Expanded
  header before the surface column allocates it.
- Fixed child heights remain explicit constraints through the promoted
  `intrinsic_or_fixed_height_for_width` helper; implicit grow is never treated
  as intrinsic content.
- Exact witnesses pin a short row at 28 px and a wrapped row at 128 px for a
  100 px track. Resizing the track to 140 px moves both cell geometry and
  measurement, reducing the wrapped row to 108 px; horizontal scrolling leaves
  heights unchanged. The gallery pins all four visible rows at 68 px and all
  Expanded header cells at 30 px instead of accepting any value above 24.
- No public API change. `Projection::column_width` and the promoted measurement
  helper remain crate-internal.
- Pending eyes: re-check Expanded density and wrapping after all chrome changes
  land; no table-specific padding or second solver was introduced.
- Commit receipt: `5af6d17b`; 5 files changed, 282 insertions, 37 deletions.

## Checkpoint 2 census and boundary — subjects, not captions

- Existing owner: `Node::with_subject`, public `Element::subject`, and retained
  composition subjects already separate semantic naming from painted labels.
  No new channel or paint exception is warranted.
- Production census: the table was the only production caller putting an
  internal name into a scroll label. Command-palette and platform scroll nodes
  do not. Test fixtures still use visible labels such as `Outer Scroll`,
  `Inner Scroll`, and `Audit scroll`; they remain because those tests exercise
  the public visible-label recipe and do not prove an application misuse.
- The table horizontal scroll now carries `Subject("Table columns")` and no
  label. Layout lookup uses its table projection rather than text identity.
- Exact witnesses inspect the constructed node's retained subject and absent
  label, then prove the scene contains no `Table columns` text while the
  horizontal viewport and scrollbar still exist.
- No public API change and no global scroll-label behavior change.
- Commit receipt: `c401ebe9`; 3 files changed, 66 insertions, 9 deletions.

## Checkpoint 3 census and boundary — sort state is not label text

- Proven duplication: the sortable header concatenated `↑`, `↓`, or neutral
  `↕` into the button label. That collapsed semantic name, sort state, label
  layout, and indicator placement into one string and made trailing alignment
  impossible.
- Existing pattern: controls such as choices already remain one retained target
  while layout and paint derive multiple host-owned subparts. Sort headers now
  follow that pattern rather than creating a child control or second target.
- `HeaderPresentation` carries only projected wrap and active sort direction.
  Layout derives one leading label rect and one optional trailing indicator rect
  from the header cell. Measurement, paint, and tests consume those same rects.
- The button label remains exactly the column name. Unsorted sortable columns
  reserve no indicator and paint no glyph. Active ascending/descending states
  paint Phosphor `caret-up`/`caret-down` at the trailing edge.
- Clicking the decorative caret routes through the existing header target;
  resize boundaries keep precedence in their authoritative hit strips.
- Expanded sortable headers measure and wrap in the label subpart around the
  pinned caret. Compact headers use no wrapping. Custom headers retain their
  escape-hatch behavior and do not receive an automatic sort target.
- Structural absence: source and gallery code contain no `Record ↑`,
  `Record ↓`, neutral `↕` branch, or glyph-formatted header label.
- No public API change; the presentation and subpart helpers are crate-internal.
- Commit receipt: `503bd6d3`; 12 files changed, 357 insertions, 30
  deletions.

## Checkpoint 4 census and boundary — chrome belongs to visible space

- Proven duplication: a table body's canonical viewport and frame retain the
  full track extent, while its inherited clip carries the visible table extent.
  Scrollbar bounds previously read the former and hit testing separately read
  the latter. The resulting scrollbar was real but horizontally offscreen.
- Existing owner promoted: `Viewport` now carries both its canonical scroll
  rectangle and one derived visible projection. The projection separates the
  visible frame (including any gutter) from visible content (after gutter
  allocation); canonical content and maximum offsets remain source truth.
- Ordinary scrolls, table scrolls, and fixed- and variable-row virtual lists
  derive visible geometry once from their frame, ancestor clip, axis, and
  existing theme policy. Child clipping, table projection bounds, scrollbar
  tracks and thumbs, reveal calculations, hit/drag geometry, and virtual-list
  page extent consume that projection.
- The table witness pins the vertical track to the visible right edge and proves
  it remains unchanged while every header, body cell, and divider moves through
  the horizontal viewport. The projected track is the target that wins hit
  testing; no paint-only overlay was introduced.
- `GutterAlways` proves visible-frame anchoring and visible-content allocation
  are two views of the same projection, and keyboard page size consumes the
  latter. `OverlayAuto` retains its existing fade policy. A fully ancestor-
  clipped viewport now emits no phantom scrollbar chrome.
- No public API or theme-policy change. The visible projection and its geometry
  helpers remain crate-internal; generic scroll chrome is the owner rather than
  table-specific scrollbar code.
- Pending eyes: verify overlay fade and gutter density in the comparison example
  after the interaction checkpoint; horizontal and vertical scroll ownership
  intentionally remain separate sources joined only by visible geometry.
- Commit receipt: `c8fbc548`; 6 files changed, 237 insertions, 71 deletions.

## Checkpoint 5 census and boundary — deliberate editing

- Existing lower owners were retained. `FieldMode` owns editable, read-only,
  and disabled capabilities; document editing owns selection, clipboard,
  history, validation, caret, and IME behavior; retained `table::Cell` identity
  owns the current cell; `interaction::Tables` owns the sole active edit
  identity; providers and application commands remain the only value owners.
- `TextEditor` and `NumberEditor` now project a read-only text surface at rest
  and construct the editable projection only while the canonical cell is in
  the edit session. The display surface keeps table-cell chrome. It permits
  selection and copy while `FieldMode::allows_edit` centrally rejects cut,
  paste, deletion, history mutation, text input, and IME mutation.
- The edit descriptor is capability data, not a dormant editor. It retains the
  source text, input and validation policy, presentation, and commit intent so
  a rebuild can derive exactly one editor when the session changes mode. The
  display and editor never coexist as active text owners.
- Existing text pointer behavior was generalized to accept the platform click
  class. Windows double-click timing and distance come from the native system
  metrics; the pointer session retains one sequence and cancels it on drag.
  Single-click and selection drag remain display operations. Only a double
  click on an editable table cell emits the begin-edit transition; double and
  triple click on non-editable/read-only text retain word/all selection.
- Grid navigation consumes provider keys and column identities rather than
  frame positions. Arrows, row Home/End, table Ctrl+Home/Ctrl+End, Page,
  Tab/Shift+Tab, Enter, Shift+Enter, and F2 converge on one `CellMove`
  vocabulary and the existing vertical/horizontal reveal paths. Direct child
  controls receive focus and keep native button/toggle key grammar.
- Successful commit routes through the existing validated commit binding,
  closes the edit session, and establishes the destination as current. Failed
  validation retains the editor and blocks movement. Escape clears draft,
  rejection, and edit identity without application mutation. Deliberate cell
  movement commits first; window deactivation, overlay opening, measurement
  refinement, and unrelated rebuilds do not accidentally commit.
- Retained reconciliation prunes an edit when its row or column disappears.
  Compact and Expanded presentations carry the same cell identity and
  interaction state; no mode-specific edit session was introduced.
- Boolean typed cells now derive their checked state from the existing
  `EditToggle` capability and retain native single-click/Space behavior. This
  deletes the gallery's affordance lie without teaching table navigation a
  private toggle mechanism.
- Exact witnesses prove display drag selection and copy without editing;
  mutation commands rejected while read-only; a platform-classified second
  click constructs exactly one editor with word selection; F2/Enter entry;
  Escape cancellation; Enter/Shift+Enter/Tab/Shift+Tab validated movement;
  horizontal reveal through overflow; bool click/Space behavior; and safe row
  removal during an active edit.
- Public API effects are limited to two demonstrated external capabilities:
  `input::Key::F2`, needed by application/platform key handling, and
  `EditToggle::is_on`, needed by external typed boolean columns to project
  application state. Table edit-session, read-only bridging, click sequence,
  and navigation vocabulary remain internal.
- Principled non-merges: value-only/custom cells without a canonical text
  projection keep their own behavior; no table export or synthetic text owner
  was invented. The reserved accessibility seam has no implemented data-grid
  vocabulary to consume, so this checkpoint preserves roles/subjects and
  records semantic grid position and sort/edit metadata as follow-up rather
  than minting table-only accessibility state.
- Explicit absences: no printable type-to-replace, range selection, TSV,
  paste, fill, column reorder, multi-sort, formulas, frozen columns, or
  Shift+Wheel remapping entered the tree.
- Commit receipt: `ab8b51fe`; 42 files changed, 1,515 insertions, 145
  deletions.

## Checkpoint 6 resistance audit and close-out

### Structural audit

- Variable table rows no longer measure through an `i32::MAX` viewport-height
  surrogate. Remaining integer maxima are numeric saturation or generic scroll
  limits, not table intrinsic measurement.
- Table cell measurement and placement have one width source:
  `table::Projection`. There is no `table::Width`, `TrackSpec`, replacement
  solver, or downstream private table-width model.
- `Table columns` appears only as the retained subject and its two exact
  witnesses. No production `.with_label("Table columns")` call exists and the
  phrase is absent from scene text.
- Source contains no `Record ↑`, `Record ↓`, neutral `↕`, or other
  glyph-formatted table header label. The active caret remains a decorative
  header subpart with the header as its only target.
- `Viewport::visible_frame` and `visible_content` are the shared inputs for
  clipping, page extent, chrome paint, and chrome hit geometry. No table-only
  scrollbar clamp or parallel track rectangle exists.
- Pointer source contains exactly one edit-entry condition: a platform-classed
  `Double` click on a frame with table edit capability. Single click only makes
  the cell current or participates in selection.
- Read-only mutation gating has one owner, `FieldMode::allows_edit`; direct
  runtime edits and focused document services both consume it. Table code does
  not privately enumerate clipboard or mutation commands.
- Current cell, row selection, text selection/draft, and active edit identity
  remain respectively owned by retained cell focus, virtual-list selection,
  document editing, and `interaction::Tables`.
- Source-wide absence searches found no TSV, range selection, fill handle,
  frozen-column, type-to-replace, multi-column sort, or Shift+Wheel table path.
  Compact and Expanded still construct one table species and share identity.

### Manual eyes and interaction pass

- At the representative native Windows raster scale, Compact retained its
  original density. Expanded produced bounded rows sized to wrapped content,
  not large empty allocations; the Expanded header remained ordinary table
  chrome.
- The active Record sort projected one trailing up/down caret. An inactive
  header projected none. Single-click sorting changed application order to
  descending and retained the same header target.
- Widening Detail moved its rule and produced horizontal overflow. Scrolling
  horizontally moved columns beneath the body while the vertical overlay thumb
  appeared at the visible right edge. Vertical scrolling, resizing, and direct
  checkbox activation remained usable.
- A single click on Count established restrained current-cell chrome and did
  not construct an editor. F2 and double-click each constructed the temporary
  editor. Escape restored `0`; entering `8` and pressing Enter committed
  through the application command and moved current-cell focus down one row.
  Entering invalid `1000` retained the editor and left application state and
  destination unchanged.
- Display selection/copy, Tab/Shift+Tab, page navigation, row removal during
  edit, four-scale rule/track geometry, and clipped virtual-row behavior are
  additionally pinned by deterministic whole-runtime witnesses. The native
  driver does not expose custom table accessibility nodes yet, matching the
  recorded accessibility non-merge rather than hiding a failed semantic claim.

### Concept verdict

No new highest-level table species emerged. The six defects were violations of
existing lower concepts or missing bridges between them:

- resolved tracks own table measurement and placement;
- subjects own internal semantic names;
- sort state projects decorative header structure;
- visible viewport geometry owns scroll chrome;
- existing read-only text owns display selection and copy;
- the table session owns only current-cell navigation and one edit identity;
- application commands own data mutation.

The only public additions, `Key::F2` and `EditToggle::is_on`, have demonstrated
platform and external typed-column callers. No speculative public table
vocabulary was admitted. The accessibility seam and spreadsheet behaviors
remain explicit future arcs rather than partial features.

### Final worktree and stack

The reviewed campaign stack is:

1. `816271b8` — open the crash-safe ledger.
2. `5af6d17b` — resolved-track intrinsic measurement.
3. `c401ebe9` — subject-channel internal naming.
4. `503bd6d3` — structural sort-state projection.
5. `c8fbc548` — visible-space scroll chrome.
6. `ab8b51fe` — deliberate table editing grammar.

The close-out commit changes only this ledger and the roadmap. No unrelated
worktree changes remain, protected `comparison_open: true` is unchanged, and
the campaign is not pushed.

## Execution ledger

| Entry | Scope | Result |
| --- | --- | --- |
| E-000 | Campaign baseline | Held: 875 passed, 8 ignored; three smokes, formatting, all targets, clean worktree, and protected state green |
| E-001 | Checkpoint 1 focused geometry | Held: exact resolved-track row heights, resize remeasurement, horizontal-scroll invariance, and generic variable-list regression witness |
| E-002 | Checkpoint 1 full boundary | Held: 876 passed, 8 ignored; three smokes, formatting, all targets, diff check, compact million-row witness, and protected state green |
| E-003 | Checkpoint 2 naming witnesses | Held: retained `Table columns` subject, absent node label, absent scene text, structural projection lookup |
| E-004 | Checkpoint 2 full boundary | Held: 877 passed, 8 ignored; three smokes, formatting, all targets, diff check, compact million-row witness, and protected state green |
| E-005 | Checkpoint 3 focused interaction and geometry | Held: active-only caret, exact label/indicator rects, caret click activation, divider precedence, descending transition, Expanded wrap |
| E-006 | Checkpoint 3 full boundary | Held: 878 passed, 8 ignored; three smokes, formatting, all targets, diff and absence checks, compact million-row witness, and protected state green |
| E-007 | Checkpoint 4 focused projection and interaction | Held: visible-edge overlay and gutter anchors, stable track through horizontal scroll, chrome hit target, visible page extent, and fully-clipped absence |
| E-008 | Checkpoint 4 full boundary | Held: 879 passed, 8 ignored; three smokes, formatting, all targets, diff check, compact million-row witness, and protected state green |
| E-009 | Checkpoint 5 focused interaction grammar | Held: read-only drag selection/copy, platform double-click entry, canonical edit-key movement, horizontal reveal, native bool click/Space, and removal pruning witnesses |
| E-010 | Checkpoint 5 full boundary | Held: 883 passed, 8 ignored; three smokes, formatting, all targets, diff check, compact million-row witness, and protected state green |
| E-011 | Checkpoint 6 resistance and manual-eyes audit | Held: defective source shapes absent; Compact/Expanded density, trailing sort caret, resize/overflow, pinned vertical chrome, current/edit distinction, commit/move, cancel, validation retention, and native checkbox behavior observed |
| E-012 | Campaign final boundary | Held: 883 passed, 8 ignored; three smokes, formatting, all targets, diff check, protected comparison state, commit stack, and final worktree green |

## Public API flags

Checkpoint 5 added public `input::Key::F2` and required `EditToggle::is_on`.
Both have demonstrated callers at the platform/application boundary. All other
corrections consume existing `view::Dimension`, `table::Projection`, `subject`,
text surface modes, focus, clipboard, and viewport chrome concepts.

## Watch lines and explicit deferrals

- Printable type-to-replace.
- Range and discontiguous selection; row/column shortcuts.
- Multi-cell TSV copy/paste, cut, delete, fill, and bulk mutation.
- Column drag reorder and double-click divider auto-fit.
- Multi-column sorting and third-click sort clearing.
- Frozen columns, formulas, and sheet semantics.
- Context-menu integration.
- Shift+Wheel axis remapping belongs to generic viewport input, not tables.

## Commit ledger

| Boundary | Commit | Files | Insertions | Deletions | Receipt |
| --- | --- | ---: | ---: | ---: | --- |
| Campaign open | `816271b8` | 2 | 115 | 1 | Ledger and roadmap opened from a clean baseline |
| Checkpoint 1 | `5af6d17b` | 5 | 282 | 37 | Resolved-track intrinsic row and header measurement |
| Checkpoint 2 | `c401ebe9` | 3 | 66 | 9 | Existing subject channel replaces painted internal scroll label |
| Checkpoint 3 | `503bd6d3` | 12 | 357 | 30 | Active sort state projects into header-owned label and caret subparts |
| Checkpoint 4 | `c8fbc548` | 6 | 237 | 71 | Scroll chrome consumes one visible viewport projection |
| Checkpoint 5 | `ab8b51fe` | 42 | 1,515 | 145 | Display selection and deliberate editing consume existing text and table-session owners |
| Checkpoint 6 | this close-out commit | 2 | 98 | 13 | Resistance audit, roadmap pruning, and campaign close-out |
