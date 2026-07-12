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
| 1 | Expanded rows consume intrinsic content at resolved track widths | Green; commit pending | 876 passed, 8 ignored; exact 28/128 px rows, resize to 108 px, scroll invariance, generic variable-list preservation, three smokes and all boundary checks green |
| 2 | Internal table naming consumes the existing subject channel | In progress | Census pending |
| 3 | Active sort indication is trailing header structure, never label text | Pending | — |
| 4 | Vertical scrollbar chrome consumes the visible body projection | Pending | — |
| 5 | Read, select, navigate, then deliberately edit | Pending | — |
| 6 | Resistance audit and campaign close-out | Pending | — |

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
- Commit receipt: pending checkpoint commit; the next ledger boundary records
  its hash and diff statistics.

## Execution ledger

| Entry | Scope | Result |
| --- | --- | --- |
| E-000 | Campaign baseline | Held: 875 passed, 8 ignored; three smokes, formatting, all targets, clean worktree, and protected state green |
| E-001 | Checkpoint 1 focused geometry | Held: exact resolved-track row heights, resize remeasurement, horizontal-scroll invariance, and generic variable-list regression witness |
| E-002 | Checkpoint 1 full boundary | Held: 876 passed, 8 ignored; three smokes, formatting, all targets, diff check, compact million-row witness, and protected state green |

## Public API flags

None opened. Internal corrections must use existing `view::Dimension`,
`table::Projection`, `subject`, text surface modes, focus, clipboard, and
viewport chrome concepts before proposing public vocabulary.

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
| Checkpoint 1 | pending | pending | pending | pending | Resolved-track intrinsic row and header measurement |
