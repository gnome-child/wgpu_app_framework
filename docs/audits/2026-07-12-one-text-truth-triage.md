# One Text Truth table triage

Crash-safe ledger opened from local `25cfab90`, six commits ahead of
`origin/master` at `c73a90e3`. The closed Five Truths campaign remains an
honest receipt of its automated boundary; this campaign begins from the later
release field report: a table click terminated `control_gallery`, Compact text
combined ellipsis with a hard tail cut, editable numeric display lost its
alignment, expanded headers changed height, and a held divider did not follow
the pointer.

Status: active. Campaign commits after Fix 1 remain unpushed unless separately
authorized.

## Doctrine under test

One resolved text layout supplies measurement, overflow, paint, hit testing,
and selection. Editability adds behavior without replacing inactive display
truth.

Declarative weights allocate unclaimed space. They do not countermand a
boundary the user is actively holding.

The admission companion is narrowness: a proven instance does not authorize a
general class. Specifically, this campaign does not make every missing command
target inert, freeze every weighted column permanently, or mint a public table
text-layout API without a second caller.

## Protocol

- Census owners and reduce each defect before changing behavior.
- Prefer deletion and existing mechanics; preserve unrelated work and
  `comparison_open: true`.
- Every fix closes with focused witnesses, formatting, all-target compilation,
  the full library suite, the three smoke examples, and diff hygiene.
- Each fix is independently green and committed with its receipt below.
- Once Fix 1 is fully green, the user authorizes pushing the prior Five Truths
  stack plus the crash fix. Fixes 2-5 remain local without new authorization.

## Fixes

### Fix 1 — input ownership

Reduce the `document.apply_edit` process exit in `control_gallery`. A focused
table cell owns its local draft interaction and never falls through to document
editing. Only a specifically proven stale interactive target may become inert
with a diagnostic at the final input boundary; programmatic `MissingTarget`
and unrelated defects remain visible.

Closed. The live release gallery survived ordinary, sorted, expanded, edited,
selected, resized, and cross-cell click sequences. The deterministic reduction
is the ownership boundary beneath those gestures: text input with a table-cell
focus but no current local draft fell through to `document::ApplyEdit` and
reproduced the field report's exact `MissingTarget`. This includes display-only
cells and a late table identity whose materialized cell has departed.

The repair is deliberately narrow. `handle_text_edit` recognizes table-cell
focus after the local-draft path declines, emits a debug diagnostic, and returns
an ignored input outcome. It does not change command dispatch, responder lookup,
or `MissingTarget`; the same programmatic `ApplyEdit` invocation remains an
error witness. Existing reconciliation already clears stale focus, so no second
stale-target framework layer was admitted.

Boundary: 901 discovered; 893 passed, 8 deliberately ignored, 0 failed.
Formatting, all-target compilation, all three smokes, diff hygiene, and the
protected comparison flag were green.

### Fix 2 — one approved width

Census every width used by typed display, editable display, overflow,
measurement, paint, hit testing, and selection. They consume one canonical
padded content rectangle. At four scales, approved overflow output fits the
paint rectangle and Compact never combines ellipsis with an unapproved hard
cut.

Closed. `table_content_rect` remains the body-cell geometry owner and now
delegates its width to one `table_content_width` calculation. World-text
overflow approval, wrapped intrinsic measurement, inactive TextArea shaping,
pointer/drag mapping, selection and caret clipping, and visible table paint all
consume that padded rectangle. The TextArea frame retains the rectangle beside
its shaped layout so late interaction cannot reconstruct it differently.

Witnesses pin EllipsisEnd and EllipsisMiddle output inside the exact painted
rectangle at logical scales 1.0, 1.25, 1.5, and 2.0; selection quads are bounded
by the same rectangle; and expanded row measurement responds monotonically to
the padded resolved track width. `Clip` remains honest residual clipping rather
than pretending the source fits. No public API or second table width solver was
introduced.

Boundary: 901 discovered; 893 passed, 8 deliberately ignored, 0 failed.
Formatting, all-target compilation, all three smokes, diff hygiene, and the
protected comparison flag were green. This commit remains local.

### Fix 3 — one inactive display recipe

Ordinary and editable values share inactive typography, alignment, placement,
measurement, identity, and glyph geometry in both presentations. `V::align()`
survives edit capability; selection and copy consume the visible glyph layout;
editor machinery exists only while editing. Boolean toggles and custom actions
remain deliberately distinct.

Closed. The census found two inactive species: ordinary typed values built a
Label (and an alignment Stack for numeric values), while `.editable()` replaced
that projection with a read-only TextArea and discarded both `V::align()` and
the column's overflow policy. Both paths now call one typed-value projection:
a read-only, selectable TextArea carrying the same world-text wrap, residual
overflow, and alignment facts. Only an active edit replaces it with TextBox;
native boolean toggles and arbitrary provider nodes remain their own honest
species.

The retained frame carries alignment beside the already-shared world-text
policy. Visible glyphs and selection geometry consume that fact together;
single-line numeric selection is anchored to the same trailing content edge as
its painted text, and all inactive text is vertically centered inside the
canonical padded cell rectangle. Draft discovery was widened only to table
cells, so ordinary application TextAreas retain document ownership.

Witnesses prove ordinary Record, editable Note, and editable Count cells share
the same inactive role; Count stays end-aligned with the same node identity in
Compact and Expanded; read-only selection/copy remains local; and Expanded row
height is the measured wrapped requirement rather than a brittle fixed
constant. Boundary: 901 discovered; 893 passed, 8 deliberately ignored, 0
failed. Formatting, all-target compilation, all three smokes, diff hygiene, and
the protected comparison flag were green. This commit remains local.

### Fix 4 — constant headers

Reverse expanded-header wrapping intentionally. Headers remain single-line and
fixed-height in both presentations, use canonical padded overflow geometry,
and retain the trailing active sort indicator without resize or toggle jitter.

Closed as the intentional product reversal. The duplicate presentation branch
was literal: Compact chose a fixed header container and no-wrap text, while
Expanded chose fit height and word wrap. Header presentation no longer owns a
wrap mode, and the table always allocates its configured fixed header height.
Derived sortable and ordinary headers declare single-line EllipsisEnd world
text; custom header nodes remain the escape hatch rather than being rewritten.

Overflow approval now measures against `table_header_label_rect`, the same
canonical padded geometry painted by the scene and shortened by the trailing
chevron reservation. Button frame content retains the world-text policy just
as Label and TextArea content already did, so a header's resolved string,
painted overflow contract, and measurement cannot diverge by role.

Witnesses prove an intentionally long active sort header remains single-line,
ellipsizes inside the chevron-safe rectangle at scales 1.0, 1.25, 1.5, and
2.0, and keeps identical height through a divider drag. The gallery proves all
header heights remain identical across Compact/Expanded while body rows still
change flow. Boundary: 901 discovered; 893 passed, 8 deliberately ignored, 0
failed. Formatting, all-target compilation, all three smokes, diff hygiene, and
the protected comparison flag were green. This commit remains local.

### Fix 5 — held boundaries

Reduce the Count/Enabled seam: its Count override becomes `Fixed` before flex
allocation, so Detail and Note surrender width and the held boundary stays
stationary. Count retains trailing-edge ownership. The boundary follows the
clamped pointer, left widths remain, right tracks translate, total extent
floats, and existing horizontal overflow absorbs growth. Census post-resolution
override against minimum resolved-width materialization before choosing state.

Closed with the post-resolution mechanism. The reduction pinned the reported
Count/Enabled seam: feeding Count's override back as a Fixed declaration let
the allocator lawfully redistribute the two weighted columns, leaving the held
boundary stationary while earlier seams moved. The alternative of materializing
every resolved width at drag start would have fixed the gesture by freezing all
flex; it was rejected because it introduced a permanent table-wide mode switch.

Column declarations now allocate their ordinary base projection first. Session
resize overrides replace only their matching resolved widths afterward, before
placement consumes the projection. Origins are accumulated once from those
resolved widths, so the overridden trailing edge follows the pointer, left
tracks remain identical, right tracks translate rigidly, and the scroll extent
grows by exactly the drag delta. The layout surface still spans at least the
visible viewport so vertical viewport chrome keeps its visible-edge owner; grid
rules and cells consume the independently floating track extent.

The named regression drags Count/Enabled through four intermediate positions
and proves each pointer equals the rule center, resize-zone anchor, header edge,
and body edge; Record, Detail, and Note do not move; Enabled and Action retain
width and translate; horizontal max scroll grows by the same delta. A wider
viewport then proves Detail and Note weights still re-resolve while Count keeps
its manual width. Existing final-column clamping, minimum width, capture
removal, scrolling, variable rows, and table-local session witnesses remain
green. Native projection repeats the resized table at scales 1.0, 1.25, 1.5,
and 2.0 with aligned one-physical-pixel rules.

Boundary: 902 discovered; 894 passed, 8 deliberately ignored, 0 failed.
Formatting, all-target compilation, all three smokes, diff hygiene, and the
protected comparison flag were green. This commit remains local.

## Census receipts

- Crash: `document::Editing` registers `ApplyEdit`, but the gallery has no
  `Document` responder. The invalid path is table-draft miss followed by the
  generic document-edit fallback; the exact interactive transition still needs
  reduction.
- Text: overflow approval uses full frame width while table paint subtracts
  canonical horizontal padding. Inactive editable text also maintains an
  invisible TextArea layout beside its visible world-text layout.
- Alignment: `.editable()` replaces `value_node`, so integer end alignment is
  lost in both presentations rather than only Expanded.
- Headers: presentation currently chooses both header wrap and fixed-versus-fit
  height, and a test explicitly approves expanded growth.
- Resize: `Column::effective_width` converts a session override to
  `Dimension::Fixed`; the flex-fill branch then reallocates the remaining
  viewport width. Existing resize witnesses use cases where the dragged edge
  moves and do not cover a fixed column preceded by weighted tracks.

## Commit receipts

| Boundary | Commit | Files | Insertions | Deletions | Result |
| --- | --- | ---: | ---: | ---: | --- |
| Ledger open | this commit | 2 | 112 | 3 | Field report, doctrine, reductions, and protocol |
| Fix 1 | this commit | 3 | 81 | 1 | Table-cell input remains local; programmatic missing targets remain errors |
| Fix 2 | this commit | 7 | 133 | 23 | One padded body-text rectangle across measure, paint, and interaction |
| Fix 3 | this commit | 9 | 253 | 43 | One inactive display recipe; typed alignment and identity survive presentation |
| Fix 4 | this commit | 6 | 145 | 89 | Constant single-line headers with shared overflow geometry |
| Fix 5 | this commit | 5 | 275 | 13 | Post-resolution manual width; held boundary follows pointer without freezing flex |
| Close | pending | pending | pending | pending | Laws, resistance audit, final boundary, clean tree |
