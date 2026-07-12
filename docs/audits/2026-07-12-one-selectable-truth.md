# One Selectable Truth campaign

Opened from local `5c8b0ec5`, five commits ahead of `origin/master` at
`6b480ccc`. The prior One Text Truth close remains an honest receipt of its
automated boundary; this campaign begins from the subsequent field report:
selection and caret paint drifted from Count glyphs and remained in previously
visited cells, compact ellipsized cells exposed inert horizontal scrollbars,
and those late-painted scrollbars escaped their table viewport.

Status: active. Checkpoint 1 must seal the field regressions before the user-
authorized push of accumulated history. Checkpoints 2 and 3 remain local unless
separately authorized.

## Doctrine under test

- Rectangle containment is not glyph coincidence.
- Inactive retained state does not own presentation.
- Ellipsis is presentation, never hidden scrolling.
- Viewport-scoped late chrome carries paint and hit-test provenance together.
- A cell species is selected by its presentation medium.
- Forward std conversion supplies presentation; reverse std conversion
  optionally supplies deliberate interaction; sorting is independent `Ord`.
- `From<bool>` is honest only for a value genuinely equivalent to the Boolean
  medium.

## Protocol

- Census and reduce before implementation; prefer deletion and existing text
  mechanics.
- One independently green commit per checkpoint, plus ledger open and close.
- Every checkpoint runs focused witnesses, formatting, all-target compilation,
  the full library suite, all three external smokes, diff hygiene, and protected
  `comparison_open: true` verification.
- No speculative choice/progress vocabulary, runtime trait detection, second
  table solver, or application-overlay generalization.

## Baseline

Synchronized boundary from `5c8b0ec5`:

- `cargo fmt --all -- --check` — pass.
- `cargo check --all-targets` — pass.
- `cargo test --lib` — 894 passed, 8 ignored.
- `text_editor`, `control_gallery`, and `glass_tuner` external smokes — pass.
- `git diff --check` — pass.
- `examples/glass_tuner/app/state.rs` retains `comparison_open: true`.

## Checkpoint 1 — one resolved selectable text projection

One inactive textual-cell projection must own source text, visible shaping,
omission mapping, hit testing, visible selection, clipboard source ranges,
overflow, and visible extent. Compact ellipsis cannot produce scroll, reveal,
or scrollbar behavior. Only the active text target may project cursor,
selection, scroll, preedit, or caret epoch; inactive draft storage may remain.

### Opening census

- `Frame::new` shapes a full-source read-only TextArea and separately resolves
  an overflow String for visible inline paint. The paths share a rectangle but
  not shaping or source mapping.
- `TextArea::project_layout_interaction` projects any retained `draft_for`
  target without requiring that target to be active, so previously visited
  cells repaint stale selections and carets.
- TextArea viewport extent comes from the full source layout. Generic chrome
  therefore truthfully exposes a scrollbar for a hidden layout while visible
  ellipsized inline text ignores its offset.
- Overflow currently returns only a String. The selectable projection needs
  omission/source mapping rather than another agreement-by-rectangle bridge.

### Outcome

- Overflow now resolves a retained source/visible projection at grapheme
  boundaries. It maps pointer positions and source selections in both
  directions, while clipboard commands continue to consume source ranges.
- Selectable table cells shape and paint the projected TextArea buffer once;
  the parallel inline-label paint path is deleted. Glyphs, selection, hits,
  visible extent, and viewport chrome therefore consume one layout.
- Compact ellipsis produces no hidden text scroll extent or scrollbar.
  Expanded wrapping preserves the source and its measured row behavior.
- Table draft presentation is gated by the active text target. Moving to a
  second cell clears the first cell's selection/scroll/caret projection while
  retaining its useful draft storage.
- Named witnesses cover stale retained selection paint, phantom cell
  scrollbars, end/middle omission mapping, and dragging across an ellipsis to
  copy an omitted source tail. Public API surface is unchanged.
- Synchronized boundary: formatting and all-target checks pass; 899 library
  tests pass with 8 intentional ignores; all three external smokes pass; diff
  hygiene passes; `comparison_open: true` is preserved.

## Checkpoint 2 — viewport-scoped late chrome

Generalize only the two proven late-paint tenants: focus outlines and
scrollbars. The projection carries owner, layer, viewport-visible geometry,
ancestor/viewport clip intersection, and a hit scope derived from the same
geometry. Selection remains inline; application overlays and table rules remain
separate unless their census proves identical semantics.

### Opening census

- Deferred focus paint has an ad hoc `FocusOverlay { outline, clip }` record.
- Scrollbar `Chrome` carries targets and geometry but no inherited clip; late
  paint occurs after frame clips are popped, and hit testing uses its unclipped
  track.
- This is the recorded second caller required by the Five Truths admission gate.

## Checkpoint 3 — zero capability traits; species by medium

Delete `table::{Value, EditText, Sort, EditToggle}`. Text species use `Display`,
optional `FromStr<Err: Display>`, and optional `Ord`; Boolean species use forward
`Into<bool>`, optional reverse `From<bool>`, and optional `Ord`. Alignment and
input are column configuration. `Column::custom` remains the node escape hatch.

### Opening census

- `Value::text()` is immediately converted into an owned String, so its Cow
  does not currently avoid allocation.
- `EditText::{parse,input}` and `Sort::order` redeclare std meanings.
- `EditToggle::{is_on,toggled}` is a Boolean-medium projection plus reverse
  interaction; passive Boolean presentation is independently legitimate.
- `TypedColumn::order` has no tree caller outside tests and requires a deletion
  census before public storage survives.
- `Display` is necessary for honest zero-adapter foreign citizenship: an
  application cannot implement a framework trait for a foreign type.

## Commit receipts

| Boundary | Commit | Files | Insertions | Deletions | Result |
| --- | --- | ---: | ---: | ---: | --- |
| Ledger open | `eaab8b53` | 2 | 118 | 3 | Baseline, doctrine, census, and protocol |
| Checkpoint 1 | this commit | 10 | 704 | 112 | One resolved selectable projection |
| Checkpoint 2 | pending | pending | pending | pending | Viewport-scoped late chrome |
| Checkpoint 3 | pending | pending | pending | pending | Std capability boundary and explicit species |
| Close | pending | pending | pending | pending | Resistance audit, API review, final boundary, clean tree |
