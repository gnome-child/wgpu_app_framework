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
| 2 | Typed `FrameContent` over the existing roles | In progress | Current-tree family census is the next action; no implementation has begun |
| 3 | Uniform virtual region/list | Pending | One million logical rows produce bounded materialization and work |
| 4 | Keyed selection and active item | Pending | Independent long state-machine and virtualization journeys |
| 5 | Read-only record table | Pending | Large gallery table with tracks, headers, resize, sort intent, selection and public cells |
| 6 | Editable cells | Pending | Numeric plus textual/enumerated editors with stable cell identity |

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

## Pending eyes

- Checkpoint 1 (implemented, morning eyes remain non-blocking): ellipsis glyph
  appearance, cut spacing, end/middle appearance in both themes, and logical
  source-order bidi behavior.
- Checkpoint 5: striping, rules, sticky-header behavior, resize feel, selection
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

## Failure and reduction ledger

The first C1 full run found three stale editor-debug expectations. Reduction
showed no behavior or ownership failure: each expected the former single string
that mixed authored diagnostics with file/status world data. Commit `3ef75719`
changed those witnesses to assert the new provenance split and the added author
overflow instrument. All three focused journeys and the repeated full suite
then passed. No unresolved failure crosses the checkpoint boundary.

## Commit ledger

| Checkpoint | Commit | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- |
| Campaign | `0fad3ff2` | 1 | 151 | 0 | Crash-safe ledger established |
| Campaign | `e3f1199f` | 1 | 7 | 6 | Checkpoint 1 marked in flight |
| 1 | `e6ce36b2` | 22 | 539 | 17 | Overflow owner, required declaration, cache identity, diagnostics, and tests |
| 1 | `c8deeb29` | 2 | 32 | 5 | Doctrine and honest text-editor callers |
| 1 | `3ef75719` | 2 | 10 | 3 | Reduced full-gate expectations updated |
| 1 | `dfa728f2` | 6 | 23 | 37 | Caller-local path truncation retired; world policy is sole display owner |

## Final fixed-point sweep

Pending all six independently green checkpoints, cross-checkpoint replay,
public API review, performance comparison, pending-eyes transfer, full ritual,
and clean-worktree proof.
