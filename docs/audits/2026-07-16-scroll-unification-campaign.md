# Scroll unification campaign — one admitted position, one transform, bounded work

Status: **active** on `codex/scroll-truth-campaign`.

This is a new audit. The 2026-07-15 scroll-truth document and the handoff from
the prior agent are evidence indexes only. No prior checkbox, checkpoint name,
or green test is inherited as closure. Every boundary below is re-opened until
its own closeout record proves the field behavior, owner correction, negative
controls, work bounds, full-suite result, and required native observation.

## Field baseline

The campaign begins from the user's 2026-07-16 observations:

- text document scrolling is much better and now behaves consistently with a
  table;
- disabling wrap exposes the horizontal scrollbar at the proper content width;
- character input remains slow in large unwrapped documents;
- table rules move ahead of cell/background/text geometry;
- horizontal table scrolling can leave geometry stationary until the following
  update snaps it into place; and
- table scrolling remains choppy after the correctness defects are separated.

The first two observations are retained as field improvements, not as proof
that their underlying paths are fully audited.

## Constitution

> One interaction owner holds requested and admitted scroll position. One
> admitted property snapshot feeds content, rules, text, clips, scrollbars,
> hit-testing, caret/IME geometry, and presentation. A descendant consumes each
> ancestor scroll transform exactly once.

The concrete ownership rules are:

1. `interaction::Scroll` alone owns desired and admitted integral offsets.
   Wheel, precision delta, thumb/track, keyboard, reveal, programmatic, and
   geometry correction requests enter the same mutation/admission transition.
2. Layout owns content extent, viewport geometry, clamping, reveal proposals,
   and residency proof. It may resolve a proposal; it may not become a second
   scroll-position owner.
3. `scene::Properties` is the sole admitted presentation snapshot. Semantic
   commit, residency revision, property serial, and presentation epoch remain
   different facts.
4. A scroll scope translates a render target or direct descendant content, not
   both. Effect/group surfaces inherit the ancestor translation at composite
   time; their local members remain local unless they enter a real nested
   scroll scope.
5. Fixed viewport clips remain fixed in target space. Rules, cell fills, text,
   selection, caret, and hit geometry beneath the clip use the same admitted
   transform.
6. Warm movement inside active residency is property-only. It performs no view
   rebuild, layout recompose, semantic commit, text shaping, primitive rebuild,
   or retained-resource creation.
7. Residency crossing and editing work are proportional to the newly exposed
   or edited region. No warm operation may copy, scan, shape, hash, or allocate
   in proportion to total document/table length.
8. A frame is visible only as one complete accepted snapshot. No rule-only,
   clip-only, or scrollbar-only pixel change can acknowledge a content scroll.

## Closeout protocol

A boundary may be marked **closed** only in the same commit that contains its
closeout entry. That commit is then pushed before work begins on the next
boundary. A closeout records all of the following:

- the reproduced failure and a negative control that would fail for the old
  behavior;
- the earliest incorrect owner or assumption;
- the displaced path or assumption, not an added synchronization copy;
- deterministic state/pixel/work witnesses;
- release timing and memory receipts where performance participates;
- the full proportional test tier;
- native observation when cadence, interaction feel, or platform presentation
  participates;
- the closeout commit subject and pushed branch.

A passed legacy test is not closure when its oracle traverses the same faulty
path, observes only one member of a grouped result, or permits a later frame to
repair the first one.

## Industry comparison floor

These systems are comparison witnesses, not authorities to imitate:

| System | Primary-source property | Local challenge |
|---|---|---|
| [GTK `Scrollable`](https://docs.gtk.org/gtk4/iface.Scrollable.html), [`Adjustment`](https://docs.gtk.org/gtk4/class.Adjustment.html), and [`Scrollbar`](https://docs.gtk.org/gtk4/class.Scrollbar.html) | One adjustment per axis carries value, range, increments, and visible page; scrollable controls update it and contents react to its value | Keep one per-axis owner/projection across text, table, and generic controls; update range and value atomically rather than deriving chrome from another state |
| [Qt `QAbstractScrollArea`](https://doc.qt.io/qt-6.8/qabstractscrollarea.html) and [`QPlainTextEdit`](https://doc.qt.io/qt-6/qplaintextedit.html) | Scrollbar value selects viewport content; `contentOffset()` maps document to viewport; plain text exposes first-visible-block and block geometry for large documents | Keep viewport, chrome, geometry, and hit mapping on one offset; edits and paint must remain block/local rather than flattening the document |
| [Iced `Scrollable` source](https://github.com/iced-rs/iced/blob/master/widget/src/scrollable.rs) | One widget state stores axis offsets; the same translation is supplied to operations, events, drawing, and viewport projection | Preserve axis symmetry and one translation through draw and interaction; do not let widget species select different clocks |
| [COSMIC/libcosmic](https://pop-os.github.io/libcosmic/cosmic/) and its [Iced scrollable re-export](https://pop-os.github.io/libcosmic/cosmic/iced/widget/struct.Scrollable.html) | COSMIC is based on Iced and exposes the same scrollable interaction model while composing higher-level table and text widgets | A themed/composed control must not fork the lower scroll owner or translation |
| [Chromium cc architecture](https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/how_cc_works.md) and [cc input](https://chromium.googlesource.com/chromium/src/+/HEAD/cc/input/README.md) | Active/pending trees separate readiness from responsive active scrolling; property trees make updates proportional to interesting nodes; compositor scrollbars move with compositor scroll | Separate semantic/residency/property clocks, keep active pixels responsive while preparation continues, and upload only changed scroll properties |
| [Firefox APZ/WebRender](https://firefox-source-docs.mozilla.org/gfx/AsyncPanZoom.html) | Input produces composite-time transforms; grouped display items and scroll metadata express what moves together; bounded displayports trade memory against checkerboarding | One scroll transform must move every member of a table group together, while resident runway remains finite and complete |
| [Flutter `Scrollable`](https://api.flutter.dev/flutter/widgets/Scrollable-class.html) and [`scrollCacheExtent`](https://api.flutter.dev/flutter/widgets/TwoDimensionalViewport/scrollCacheExtent.html) | Interaction position is separated from viewport construction; finite before/after cache extent bounds near-viewport layout | Keep admission independent of control layout and make runway bounds explicit, finite, and measurable |

The local design deliberately remains integral through admission and property
truth and rejects incomplete resident pixels. Those stronger guarantees do not
excuse extra clocks, whole-document edit work, or a transform applied twice.

## Census matrix

Every fixed-point sweep covers:

| Dimension | Required cells |
|---|---|
| Entrances | wheel lines/pixels, precision diagonal deltas, thumb drag, track press, keyboard/page, programmatic absolute, reveal/caret, resize/scale correction |
| Controls | generic scroll, wrapped/unwrapped editable/read-only text, fixed/virtual table, nested scroll, floating panel, native popup |
| Consumers | content, fills, rules, text, selection/caret, fixed and local clips, scrollbar chrome, hit test, IME/accessibility geometry |
| Lifecycles | warm property tick, residency boundary, large jump, reversal, resize, scale/backend change, edit, provider shrink/reorder, pending activation, occlusion/device loss |
| Economics | rebuild/shape/upload/resource counts, CPU p50/p95/p99/max, draw/pass count, allocation and CPU/GPU high-water, input-to-present cadence, missed/no-progress frames |

## Boundary ledger

| Boundary | State | Exit condition |
|---|---|---|
| U-001 large unwrapped edit locality | **Closed** | Character edit shapes and allocates bounded local data, shares untouched 64 MiB-scale index blocks, preserves exact width/pixels, and passes official receipt |
| U-002 table transform unity | Open | Grouped cells/fills/text and ungrouped rules move by the same requested displacement on the first property tick at all supported scales |
| U-003 table property economics and cadence | Open | Correct table warm ticks upload only changed scroll data, avoid false passes/resources, and meet native cadence without no-progress frames |
| U-004 pending/active and local clocks | Open | Newer desired/admitted offsets cannot regress or wait behind unrelated window/popup work; every presentation receipt names one complete snapshot |
| U-005 entrance/control fixed-point census | Open | The full matrix finds no duplicate owner, species branch, float narrowing, unbounded work, false invalidation, or uncovered transition |
| U-006 final native and release closure | Open | Full deterministic/GPU/native matrix is green and field behavior is reconfirmed before the final closeout commit/push |

## U-001 closeout — persistent sparse horizontal edit index

### Reproduction and violated law

The inherited 4 MiB edit splice bounded shaping to a guarded region but stored
every revision as two complete `Vec` checkpoint arrays. A release 8-warmup,
32-sample baseline at pushed commit `75f980bf` recorded p50/p95/max
`506/631/659 us` and grew horizontal-index residency to `7,566,468` bytes after
40 alternating insert/backspace revisions. A newly added 64 MiB smoke recorded
p50 `2,004 us` and `32,477,412` resident bytes after only ten revisions. The
visible glyph window stayed 1,432 by 640, proving that total-index copying—not
visible shaping—owned the growth.

This violated constitution rule 7 and Qt's block-local large-plain-text floor.

### Earliest owner and correction

`text::layout::horizontal::LineIndex` owned flat absolute checkpoint arrays and
`splice_edit` rebuilt both arrays while shifting the complete suffix. It now
stores immutable `Arc` checkpoint blocks of at most 256 segments with block-
local source/x coordinates. A splice shares every untouched block, replaces
only blocks intersecting the guarded edit, and derives later absolute
coordinates from bounded block summaries. Cache accounting counts newly owned
block storage instead of charging a shared full index to every revision.

This removes the whole-array path. It does not add a second edit index or a
background synchronizer.

### Deterministic witnesses

- `single_line_edit_splices_only_a_guarded_horizontal_index_region` still
  requires exact equality with a cold stable-index width, zero full rebuilds,
  zero full-width source visits, and at most 4,096 shaped source bytes. It now
  also caps storage added by consecutive edits.
- `sixty_four_mib_edit_reuses_all_untouched_checkpoint_blocks` constructs the
  production 246,041-checkpoint scale without invoking the known expensive full
  glyph admission, performs an interior edit, requires all but at most four
  original blocks to remain pointer-shared, and caps new storage at 32 KiB.
- Existing precision, complex-LTR, independent-clone identity, insert/delete,
  and bounded-window witnesses remain required.

### Release receipts

The exact 8/32 comparison after the correction reports p50/p95/p99/max
`392/483/509/530 us`, a 22.5% median reduction, and `308,628` bytes of index
residency, 95.9% below the baseline. Cold admission remains comparable at
`1,098,390 us`; the render window remains 1,432 by 640; all 32 edits take the
incremental path; full index builds and full-width source visits remain zero.

The official version-8 64-warmup/1,024-sample receipt reports
p50/p95/p99/max `348/369/458/550 us`, `3,540,660` maximum index residency over
1,088 revisions, 1,024 incremental updates, zero rebuilds/evictions/full-width
visits, and the same bounded render surface.

The full 64 MiB glyph-admission benchmark is not claimed by this boundary. On
this 32 GiB machine it peaks near 24 GiB and can enter paging; that pre-existing
cold-admission owner remains open under U-005. The synthetic 64 MiB-scale test
proves splice structure without laundering that separate defect into a timing
closeout.

### Closeout

`cargo fmt --all` is clean. `cargo test --workspace --all-targets
--all-features` passes with 1,237 library tests and four hardware tests ignored,
three renderer-debug tests and eighteen GPU tests ignored, and two example
tests; there are no failures. The example-only unused-import warnings predate
this boundary and are outside its scope. The table-transform worktree remains
unstaged and is not part of this boundary.

This entry and its correction close together in commit
`perf(text): share sparse horizontal edit checkpoints`, pushed to
`origin/codex/scroll-truth-campaign` before U-002 work resumes.
