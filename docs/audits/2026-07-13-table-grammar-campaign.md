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
| 1 | One sizing truth: shared `view::Dimension`, minimum-preserving overflow pressure, delete `table::Width` | Green; commit pending | 855 passed, 8 ignored; three smokes and all boundary checks green |
| 2 | Resolve the track projection before placement; one horizontal scroll owner | Pending | — |
| 3 | Host-derived participation and truthful table chrome | Pending | — |
| 4 | General whole-draft `text::Input` policies | Pending | — |
| 5 | Typed columns from `table::{Value, Sort, EditText, EditToggle}` | Pending | — |
| 6 | Measurable read-only world-text wrapping | Pending | — |
| 7 | Independently proven variable-height virtual region | Pending | — |
| 8 | Compact/expanded table presentation and gallery toggle | Pending | — |

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
- Commit receipt: pending checkpoint commit; the next ledger boundary records
  its hash and statistics.

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
