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
| 1 | One sizing truth: shared `view::Dimension`, minimum-preserving overflow pressure, delete `table::Width` | In progress | Census pending |
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

