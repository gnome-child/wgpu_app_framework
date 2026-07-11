# Table Grammar Campaign — 2026-07-13

This is the crash-safe ledger for the eight-checkpoint campaign that polishes
record tables into a typed, horizontally scrollable, compact-or-expanded
grammar. The campaign begins after the rule-seam prerequisite at pushed commit
`c7c3a341` (`Derive table rules and resize seams from one track projection`),
with a clean worktree and `HEAD == origin/master`.

## Constitution

Only one checkpoint may be in progress. Every checkpoint begins with a current
tree census, consumes lower mechanics instead of reproducing them in `table`,
ends independently green, deletes displaced paths, and records its public API,
performance evidence, pending-eyes work, and commit receipt here.

The application owns sizing declarations and provider order. Session state owns
resize overrides. Layout owns available extent. A resolved track is an
ephemeral projection of those truths and is authoritative geometry only for its
layout snapshot. Placement, rules, hit zones, editors, and scroll extent must
consume that one projection. Interaction may target projected geometry, but it
mutates the projection's sources.

The column schema owns meaning and capability. A trait means a type can supply
a capability; a column verb means the table does expose it. Sorting emits
application-owned intent. Text edits use the existing TextBox draft store;
toggle edits create no text draft. `text::Input` evaluates proposed whole
drafts and never interrupts IME preedit. Rejection remains `String`.

Presentation owns ellipsis versus wrapping. Row metrics own vertical
placement. The compact arithmetic virtual-list path remains intact and bounded.
No compatibility aliases or parallel sizing, draft, rejection, or edit
vocabularies are admitted.

## Baseline

- Starting and remote commit: `c7c3a341`.
- Worktree: clean.
- Library: 851 passed, 8 deliberately ignored, 0 failed; harness work 0.98 s.
- Formatting: `cargo fmt --all -- --check` passed.
- Diff check: `git diff --check` passed.
- Protected state: `examples/glass_tuner/app/state.rs` contains
  `comparison_open: true`.
- Rule-seam prerequisite: one post-layout track projection owns rules, resize
  hit zones, resize targets, and drag geometry; checkpoint 2 must move that
  projection before placement rather than create stored track state.

## Checkpoint ladder

| Checkpoint | Contract | Status | Boundary proof |
| --- | --- | --- | --- |
| 1 | One sizing truth: shared `view::Dimension`, minimum-preserving overflow pressure, delete `table::Width` | Complete | `9f5e73d7`; 855 passed, 8 ignored; three smokes and all boundary checks green |
| 2 | Resolve the track projection before placement; one horizontal scroll owner | Complete | `555ef0a8`; 857 passed, 8 ignored; focused horizontal-scroll/resize/scale witnesses and full ritual green |
| 3 | Host-derived participation and truthful table chrome | Complete | `637109ef`; 858 passed, 8 ignored; host-dress census, chrome witnesses, and full ritual green |
| 4 | General whole-draft `text::Input` policies | Complete | `cbd7aeea`; 864 passed, 8 ignored; policy, paste, history, IME, benchmark, and full ritual green |
| 5 | Typed columns from `table::{Value, Sort, EditText, EditToggle}` | Complete | `d1c55dd7`; 867 passed, 8 ignored; compile-fail capability, typed gallery, bounded projection, three smokes, release benchmark, and all boundary checks green |
| 6 | Measurable read-only world-text wrapping | Complete | `1a452f7e`; 868 passed, 8 ignored; standalone measure/paint fixture, cache witnesses, three smokes, release benchmark, and all boundary checks green |
| 7 | Independently proven variable-height virtual region | Complete | `1309e3ea`; 874 passed, 8 ignored; sparse-index and mixed-row runtime witnesses, three smokes, and all boundary checks green |
| 8 | Compact/expanded table presentation and gallery toggle | Complete | 875 passed, 8 ignored; shared-track gallery comparison, three smokes, release benchmark, and all boundary checks green |

## Boundary ritual

At every checkpoint boundary record:

- focused scenario results;
- full `cargo test --lib` result and ignored count;
- `text_editor`, `control_gallery`, and `glass_tuner` smoke results;
- `cargo fmt --all -- --check`;
- `cargo check --all-targets`;
- `git diff --check`;
- `comparison_open: true` preservation;
- compact million-row bounded-work witnesses;
- API flags, pending-eyes notes, commit hash, and diff statistics.

The release text acceptance benchmark runs after checkpoints 4, 6, and 8.
Checkpoint commits are not pushed.

## API flags

Checkpoint 1 begins with one standing deletion requirement: `table::Width`
must disappear rather than coexist with a third track-sizing vocabulary.
`view::Dimension` remains the sole public sizing declaration and gains whatever
general minimum expression the census proves fits its existing callers. No
compatibility alias is permitted.

Checkpoint 1 resolved the flag by replacing the raw `Grow` and `Weight`
variants with `Dimension::Flexible { weight, minimum }` while preserving the
`grow()` and `weight()` constructors. The fluent `minimum()` operation applies
to flexible dimensions; fixed, fit, and percentage declarations retain their
old meanings. `layout::flow::Pressure` is internal because it describes
allocator policy rather than application sizing intent. `table::Width` and its
conversion helper were deleted without aliases.

Further flags append here as public names are proposed and resolved.

## Census and case law

### Checkpoint 1 — opening questions

- Inventory every `view::Dimension` declaration, match, measurement path, and
  allocator consumer.
- Inventory every `table::Width` caller and session resize projection.
- Pin current fit-pressure behavior before adding minimum-preserving overflow
  pressure.
- Reuse `flow::{SizeHint, Item, Row, Column}`; do not add table width
  arithmetic.
- Keep declared column sizing distinct from session resize overrides even when
  both feed the effective projection.

### Checkpoint 1 — verdict and boundary

- Census: all table widths were fixed or weighted declarations immediately
  converted into `view::Dimension`; there was no independent table sizing
  policy worth preserving. All declarations now use `view::Dimension`
  directly.
- Reuse: `flow::{SizeHint, Item, Row, Column}` remains the only allocator.
  `Pressure::Fit` retains ordinary emergency compression, while
  `Pressure::Overflow` stops after shrinking to declared minima. Both horizontal
  and vertical scroll axes select overflow pressure through the existing stack
  path.
- Ownership: `Column::width()` remains the application declaration. A separate
  internal resize override is refreshed from session state and only
  `effective_width()` combines the two; resize no longer overwrites the
  declaration.
- Focused witnesses: 10 allocator tests passed, including deterministic
  weighted remainder, minimum-first surplus, fit deficit, and overflow deficit.
  The public dimension witness pins clamping and confirms that non-flexible
  dimensions are unchanged.
- Absence witness: `rg` finds no `table::Width`, `Width::fixed`,
  `Width::weight`, `enum Width`, `Dimension::Grow`, or `Dimension::Weight` in
  Rust sources.
- Full library: 855 passed, 8 ignored, 0 failed in 0.94 s. The compact
  million-row table witness passed inside that suite.
- Smokes: `text_editor`, `control_gallery`, and `glass_tuner` all passed.
- Checks: formatting, all-target compilation, diff whitespace, and protected
  `comparison_open: true` all passed. No unrelated worktree changes were
  present or absorbed.
- Pending eyes: none added; this checkpoint changes declaration and deficit
  law without introducing new table visuals.
- Commit receipt: `9f5e73d7` (`Unify table sizing with shared dimensions`),
  10 files, 282 insertions, 145 deletions.

### Checkpoint 2 — projection and horizontal-scroll verdict

- Census: the seam prerequisite projected `Track` values only after every frame
  was placed. Generic `Scroll`, `Viewport`, scrollbar chrome, scroll targeting,
  clipping, and axis-aware delta consumption already supplied the horizontal
  mechanics; only the table surface and early projection were missing.
- Structure: table view data now composes one horizontal `Scroll` owner around
  one surface containing the sticky header and vertical virtual body. The
  ordinary scroll target, viewport, clipping, scrollbar projection, wheel and
  drag paths remain the mechanics owners.
- Projection: the horizontal owner resolves every declaration and session
  override once through `flow::Row` under overflow pressure. Its ephemeral
  projection supplies surface extent and the exact x/width used by header and
  body placement. Rules, divider targets, resize math, editors, compact text,
  and hit zones consume the resulting boundaries. Frame-derived geometry is a
  debug witness, not a second allocator.
- Overflow: a 240-pixel viewport with 100 fixed, 120 minimum-flex, and 90 fixed
  tracks projects 310 pixels of content and a 70-pixel maximum offset. The
  rightmost track remains 90 pixels wide offscreen, then ends exactly at the
  viewport after scrolling. Its offscreen divider does not clamp into a false
  edge target; once revealed, its final hit zone clamps inside the viewport.
- Interaction: one horizontal delta moves headers, body cells, rules, editor
  bounds, and divider seams by the same 70 pixels. A divider resize while
  scrolled moves header edge, body edge, rule, and hit anchor by the full
  20-pixel delta. Existing vertical sticky-header, selection, edit pinning,
  removal-during-capture, and table-local resize witnesses remain green.
- Scale: the shared `Rule` raster path now pins one physical pixel for vertical
  separators at 1.0, 1.25, 1.5, and 2.0; the existing horizontal witness covers
  the same scales. Logical track center remains the input seam.
- Doctrine: `docs/master_design.md` now records projection-before-consumption
  and source-directed interaction under `One Truth, One Owner`.
- Public API: none. `layout::table::Projection` and all resolved geometry remain
  internal; the existing public `Table` and `view::Dimension` contracts suffice.
- Focused tests: 24 table-filtered tests plus the four-scale vertical-rule
  witness passed. The compact million-row table stayed bounded.
- Full library: 857 passed, 8 ignored, 0 failed in 0.84 s.
- Smokes and checks: all three examples, formatting, all-target compilation,
  diff whitespace, and `comparison_open: true` passed. No unrelated changes
  were present or absorbed.
- Pending eyes: horizontal thumb discoverability and the visual balance of the
  fully revealed right edge remain explicit manual checks during checkpoint
  3's chrome comparison; geometry and hit behavior are pinned here.
- Commit receipt: `555ef0a8` (`Project table tracks before horizontal
  scrolling`), 8 files, 607 insertions, 28 deletions.

### Checkpoint 3 — participation census and verdict

- Menu census: menu `Binding` nodes carried `Source::Menu`; layout and paint
  repeatedly re-read that behavioral provenance to infer menu-row sizing,
  floating layer, fill, highlight, typography, and content layout. Separators
  remain an intentionally structural menu species.
- Palette census: palette result labels carried `Source::Palette`; layout and
  paint independently inferred palette-row sizing, floating layer, highlight,
  typography, and shortcut layout from that source.
- Verdict: table dress is the third caller of one host-derived participation
  concept. Internal `view::Participation` now records menu row, palette row, or
  a typed table part. Command `Source`, role, target, binding, focus, and action
  remain behavior owners. The marker deletes appearance inference from command
  provenance without becoming public API.
- Table parts: ordinary header, interactive header control, cell, editor,
  passive toggle, interactive toggle, and explicit action are classified when
  the table hosts their existing nodes. No provider control is replaced and no
  binding is added or removed.
- Recipe: internal `Theme::table()` derives a coherent header surface,
  hover/press tints, transparent cell/editor surface, alternating-row tint,
  passive indicator, and cell padding from existing theme truths. It adds no
  parallel TOML vocabulary before a public customization caller exists.
- Appearance witnesses: a sortable `Button` retains its target and action but
  paints as a square, start-aligned table header with no button plate; a
  `TextBox` editor retains draft, caret, selection, error, and focus behavior
  but rests on a transparent cell and focuses with an inset square outline; a
  triggerless checked boolean has no target or checkbox plate and paints a
  passive check; an explicit `Open` action retains ordinary button fill and
  rounding.
- Behavior witnesses: header-center sorting, divider precedence, table-local
  resize/capture, edit commit/reject/cancel, focus, keyboard navigation,
  selection, scrolling, and deletion tests all remained green. Interactive
  toggle classification is present; checkpoint 5 supplies its first honest
  gallery caller and activation witness rather than arming the existing
  display-only checkbox here.
- Public API: none. `Participation`, `TablePart`, and the table theme recipe are
  crate-internal supporting concepts.
- Full library: 858 passed, 8 ignored, 0 failed in 0.90 s. Compact million-row
  work remained bounded.
- Smokes and checks: all three examples, formatting, all-target compilation,
  diff whitespace, and `comparison_open: true` passed. No unrelated changes
  were present or absorbed.
- Pending eyes: scene-level witnesses cover idle/focused editor, sortable
  header, passive boolean, action button, row striping, and focus/error chrome.
  Ascending/descending glyphs and a live interactive boolean wait for their
  typed checkpoint-5 callers; compact/expanded side-by-side review remains a
  checkpoint-8 comparison.
- Commit receipt: `637109ef` (`Derive table chrome from host participation`),
  13 files, 438 insertions, 20 deletions.

### Checkpoint 4 — whole-draft input policy

- Census: every TextBox mutation converges on the existing per-target
  `draft::Input`/`draft::State` history store. Runtime typing and IME commit use
  it directly; clipboard cut/paste services use the same session edit method;
  selection replacement, deletion, and pointer/keyboard motion are ordinary
  `text::edit::Edit` values. IME preedit has a separate projection-only path.
- Public concept: `text::Input` owns `unrestricted()`, `signed_integer()`, and
  `unsigned_integer()` policies. `widget::TextBox::input(...)` and the view
  model's corresponding builder carry the declaration to the one draft seam.
  No integer TextBox species, character filter, table error, or parser was
  added.
- Policy law: the existing draft applies an edit to a candidate snapshot, then
  evaluates the proposed complete single-line draft. Accept applies the
  original edit to the real history, normalize applies one whole replacement,
  and reject leaves text/history untouched. The initial implementation briefly
  installed the candidate clone and broke typing coalescence; the full suite
  caught it, and the final path deliberately applies accepted edits to the
  original history owner.
- Numeric law: empty is valid for both; `-` is a valid signed intermediate and
  invalid unsigned input; only ASCII digits and at most one leading minus are
  accepted. Surrounding whitespace is normalized once. Syntax conversion and
  domain validation remain outside this policy.
- Focused witnesses: insertion, backspace deletion, whole-candidate rejection,
  selection replacement, normalized paste, unsigned rejection, undo/redo,
  signed intermediate drafts, and unrestricted legacy typing all pass. A paste
  of ` 42 ` becomes `42` as one undoable change.
- IME: arbitrary `composition` preedit remains present under signed-integer
  policy and never reaches evaluation; committing ` -42 ` evaluates once,
  produces `-42`, and clears preedit through the established commit path.
- Release text acceptance: passed in 0.68 s. Witnesses: 8 MiB load 31.874 ms;
  10-byte typing 2.547 us/edit; 2.5/5/10 MB typing 3.165/3.556/3.340 us/edit;
  10 B / 10 MB clone 36.114 / 36.029 ns.
- Full library: 864 passed, 8 ignored, 0 failed in 0.89 s. Compact million-row
  table work remained bounded.
- Smokes and checks: all three examples, formatting, all-target compilation,
  diff whitespace, and `comparison_open: true` passed. No unrelated changes
  were present or absorbed.
- Public API flags: `text::Input` is the simply named supporting concept in its
  owning namespace; its representation and decision enum remain private. The
  standard TextBox builder consumes it. No crate-root re-export is warranted.
- Commit receipt: `cbd7aeea` (`Add whole-draft text input policies`), 13
  files, 430 insertions, 17 deletions.

### Checkpoint 5 — typed value and capability tier

- Resistance retracted: the first census looked only at the checkbox model and
  incorrectly required `EditToggle` to own both current orientation and next
  value. Existing `command::State::checked`, projected through `Binding` and
  `Frame`, already carries application-owned current truth for any bound
  control. `EditToggle::toggled()` therefore retains its exact declared meaning:
  it produces the next domain value used to construct command arguments. Paint
  now prefers resolved binding state and falls back to the checkbox model for
  unbound controls. No trait amendment, runtime type test, or second state store
  exists.
- Public traits: exactly `Value`, `Sort`, `EditText`, and `EditToggle` were
  added under `table`. `String`, signed and unsigned integers, `bool`, and
  floats implement only their declared capabilities. `Option<T>`, float sort,
  and generalized choice policy remain deferred. The gallery's `RecordNumber`
  proves an application type can implement the open display/sort traits.
- Typed tier: `Column::value` retains a borrowing accessor while capability
  verbs remain available; `ValueColumn::build` erases it into a heterogeneous
  `TypedColumn`. `.sortable()`, `.editable::<C>()`, and `.toggle::<C>()` exist
  only under their corresponding trait bounds. `Column::custom` is the explicit
  node escape hatch, and the original free `Provider` path is unchanged.
  Compile-fail doctests prove float sorting and bool text editing are absent.
- Sort law: derived header controls project application-owned `SortState`, emit
  canonical `table::SortBy`/`SortIntent`, and never reorder records. The erased
  comparator retains `Sort::order` for product policy; the gallery target owns
  its million-row ascending/descending projection.
- Edit law: typed text cells reuse `TextEditor`, the TextBox draft/focus/history
  lifecycle, checkpoint-4 `text::Input`, syntax parsing, then column domain
  validation. The legacy `NumberEditor` remains unrestricted and behaviorally
  unchanged; the full suite caught and rejected an attempted implicit filter.
  Typed integer columns explicitly consume signed/unsigned input policy.
- Gallery deletion proof: the application schema switch, hand-built labels,
  sort button, `TextEditor`, `NumberEditor`, and inert checkbox were deleted.
  Record/count headers are derived and sortable; note/count editors are
  derived; enabled values live in application state and change through
  `SetRecordEnabled`; `Open` remains a deliberate custom button cell.
- Bounded work: `table::Source` keeps key/index/record projection
  application-owned. The erased provider caches only the currently requested
  row, so all visible columns consume one record projection; a focused witness
  proves one projection per row rather than one per cell. The million-row
  compact witness, scrolling, keyed identity, resize, edit, focus, deletion,
  and history laws remain green.
- Focused witnesses: three typed-capability unit tests, two compile-fail
  doctests, canonical sort activation with divider precedence, live checked
  toggle painting and activation, and table participation chrome all passed.
- Full library: 867 passed, 8 ignored, 0 failed in 0.94 s. All three example
  smokes, formatting, all-target compilation, diff whitespace, and protected
  `comparison_open: true` passed.
- Release text acceptance: passed in 0.68 s. Witnesses: 8 MiB load 31.489 ms;
  10-byte typing 2.674 us/edit; 2.5/5/10 MB typing
  3.617/3.528/3.922 us/edit; 10 B / 10 MB clone 35.824/35.572 ns.
- API flags: `ValueColumn`, `TypedColumn`, and `Source` are supporting public
  nouns required for compile-time capability followed by heterogeneous erasure
  and bounded records. `SortDirection`, `SortState`, `SortIntent`, and `SortBy`
  name application-owned sorting without granting table-owned order. No crate
  root re-exports or compatibility aliases were added.
- Pending eyes: compare neutral/ascending/descending header glyph balance,
  numeric right alignment, editor idle/focus/error states, and live toggle hit
  affordance during checkpoint 8's compact/expanded gallery review.
- Commit receipt: `d1c55dd7` (`Derive typed table columns from value
  capabilities`), 9 files, 959 insertions, 195 deletions.

### Checkpoint 6 — read-only world-text wrapping

- Census: bounded label measurement already uses the document shaping engine's
  `WordOrGlyph` path and width-keyed metric cache; scene text already carries
  `TextWrap`. The missing link was a truthful world-text declaration connecting
  those existing owners. No table line breaker, alternate shaper, or wrapped
  text cache was added.
- Public API: `widget::Label::wrapped(text)` is the one new constructor. It
  selects the existing public `view::Wrap::Word`; no `text::Wrap` alias or
  fluent combination with ellipsis exists. `Label::world(text, overflow)`
  remains the single-line omission constructor and ordinary authored labels
  retain their previous behavior.
- Representation: internal `WorldText::{SingleLine, Wrapped}` makes ellipsis
  and wrapping mutually exclusive. Layout preserves the original wrapped
  source, suppresses author-overflow diagnostics, measures intrinsic height at
  allocated width, and records the wrap decision on the frame. Paint consumes
  that same decision and the same frame bounds as `TextWrap::WordOrGlyph`.
- Focused fixture: a provider-authored sentence remained byte-for-byte intact
  in node, frame, and scene; narrowing from 240 to 92 logical pixels increased
  or retained height, produced multiple-line height, painted with clipping
  rather than omission, and emitted no author-text diagnostic. The existing
  ellipsis and authored-overflow fixtures remained green.
- Cache law: existing deterministic witnesses prove repeated measurement reuses
  one metric entry, width/bounds changes create distinct keys, and color-only
  changes reuse metrics. Existing Unicode, bidi, grapheme, line-break, and
  non-overlap text witnesses remain the mechanics owners.
- Full library: 868 passed, 8 ignored, 0 failed in 0.92 s. All three example
  smokes, formatting, all-target compilation, diff whitespace, and protected
  `comparison_open: true` passed.
- Release text acceptance: passed in 0.71 s. Witnesses: 8 MiB load 32.606 ms;
  10-byte typing 2.823 us/edit; 2.5/5/10 MB typing
  3.380/3.891/4.303 us/edit; 10 B / 10 MB clone 38.012/37.654 ns.
- Pending eyes: checkpoint 8 will exercise wrapped table headers and values at
  real track widths; this checkpoint intentionally adds no table caller.
- Commit receipt: `1a452f7e` (`Add measurable wrapped world labels`), 8 files,
  159 insertions, 10 deletions.

### Checkpoint 7 — variable-height virtual region

- Census and non-merge: uniform virtualization remains a direct
  `index × row_height` calculation with no sparse-index branch, lookup, or
  allocation added to it. Variable rows use a separate general path selected
  by `VirtualList::variable`; both share providers, stable keys,
  materialization, overscan, pins, selection, focus, capture, and draft
  retention rather than duplicating those mechanics.
- Projection: unseen rows use one estimated height. Measured heights are keyed
  by stable row key and reconciled into a sorted index plus prefix deltas. An
  index offset is arithmetic estimate plus a binary-searched measured prefix;
  distant offset lookup binary-searches logical indices and never scans
  preceding rows.
- Anchoring: each request records the visible anchor key, relative offset, and
  fallback index. Refinement above, within, or below the viewport rebuilds the
  sparse projection and routes to an anchor-preserving scroll offset. Reorder,
  insertion, deletion, and shrink reconcile measured keys through provider
  `index_of`; a deleted anchor falls back deterministically.
- Runtime ownership: the variable region is carried by the existing persisted
  materialization record, so measurements survive ordinary application view
  rebuilds. Layout measures only materialized rows, refines the projection,
  places each row from cumulative offsets, and emits another bounded request.
  Width is the measurement-generation token; a change clears stale measured
  heights while preserving the visible anchor.
- Standalone witnesses: mixed deterministic heights; refinement above, within,
  and below the viewport; stable reorder; measured and anchor deletion;
  out-of-range focus/capture/selection/draft-shaped pins with deduplication;
  logarithmic jump past row 800,000 with fewer than 64 provider lookups;
  bounded visible range and measurement count; and width invalidation all
  passed. A real 10,000-row variable widget converged to exact 18/32/47-pixel
  rows with bounded runtime work.
- Full library: 874 passed, 8 ignored, 0 failed in 0.87 s. Existing uniform
  million-row first-frame, jump, resize, table, focus, capture, selection, and
  draft witnesses remained green. All three smokes, formatting, all-target
  compilation, diff whitespace, and protected `comparison_open: true` passed.
- Public API flag: `VirtualList::variable` is the sole general constructor;
  sparse region, entries, prefix deltas, anchors, measurement generation, and
  persistence plumbing remain internal. No table metric vocabulary was added.
- Commit receipt: `1309e3ea` (`Add anchored variable-height virtualization`),
  7 files, 718 insertions, 16 deletions.

### Checkpoint 8 — compact and expanded table presentation

- Public concept: `table::Presentation::{Compact, Expanded}` is defaulted to
  `Compact` and selected by `Table::presentation`. No parallel table type,
  schema, track model, row-height declaration, or overflow vocabulary exists.
- Shared structure: both presentations consume the same columns, typed value
  accessors, provider/source, early track projection, horizontal scroll owner,
  rules, divider hit zones, command bindings, selection, and cell identities.
  The typed provider receives presentation as a projection rather than owning a
  second schema.
- Compact law: the existing fixed header and uniform arithmetic virtual-list
  path remain unchanged. Typed values stay single-line with `EllipsisEnd` or
  their per-column `EllipsisMiddle` override. The million-row bounded witnesses
  remain green.
- Expanded law: typed values and ordinary headers select checkpoint-6 wrapped
  world text; ellipsis is absent. Header height is fit from the maximum hosted
  cell height. Rows use checkpoint-7 variable virtualization and measure the
  maximum intrinsic hosted-cell height at resolved track width. Table theme
  padding contributes to intrinsic hosted height. Custom actions, text editors,
  toggles, and sortable headers retain their existing behavior and chrome.
- Gallery: one visible `Expanded rows` checkbox, one application field, and one
  `ToggleExpandedRows` command select presentation over the same six-column
  typed schema. Default remains false.
- Focused comparison: compact rows were uniformly 24 pixels and detail used
  middle ellipsis; expanded detail preserved full source and painted
  `WordOrGlyph`, at least one row grew, and the header grew beyond 28 pixels.
  All six column boundary coordinates were identical between modes.
- Full library: 875 passed, 8 ignored, 0 failed in 0.90 s. Existing horizontal
  reveal, resize, four-scale seam, sorting/reorder, edit/reject/cancel, toggle,
  selection, focus, capture, draft pin, and million-row witnesses remained
  green. All three smokes, formatting, all-target compilation, diff whitespace,
  and protected `comparison_open: true` passed.
- Release text acceptance: passed in 0.68 s. Witnesses: 8 MiB load 30.006 ms;
  10-byte typing 2.576 us/edit; 2.5/5/10 MB typing
  3.263/3.593/3.468 us/edit; 10 B / 10 MB clone 36.858/36.169 ns.
- Pending eyes: use the gallery toggle to compare header glyph balance, editor
  focus/rejection chrome, boolean hit affordance, row selection, far-right
  reveal, and divider feel in both modes. Geometry and behavior are pinned;
  visual taste remains manual.

## Pending eyes

- Checkpoint 2: horizontal thumb discoverability, far-right reveal, and seam
  alignment while scrolled.
- Checkpoint 3: idle/focused/rejected editors; neutral/ascending/descending
  headers; passive and interactive booleans; action buttons.
- Checkpoint 8: compact versus expanded wrapping, anchoring, selection,
  editing, and resizing.

## Watch lines

- Resolved tracks remain internal projections until a second non-table caller
  proves a public layout concept.
- `table::Value` remains table vocabulary until another surface needs the same
  display projection.
- Float sorting, `Option<T>` policy, `EditChoice`, line clamping, sheet models,
  column virtualization, async providers, locale collation, and generalized
  checkbox value triggers remain excluded without callers and doctrine.
