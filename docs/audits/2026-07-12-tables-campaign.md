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
| 1 | Text overflow and three text kinds | In progress | Scenario census C1-01 through C1-10 below |
| 2 | Typed `FrameContent` over the existing roles | Pending | No optional payload cluster remains; behavior and text benchmark hold |
| 3 | Uniform virtual region/list | Pending | One million logical rows produce bounded materialization and work |
| 4 | Keyed selection and active item | Pending | Independent long state-machine and virtualization journeys |
| 5 | Read-only record table | Pending | Large gallery table with tracks, headers, resize, sort intent, selection and public cells |
| 6 | Editable cells | Pending | Numeric plus textual/enumerated editors with stable cell identity |

## API flags

No campaign public API has been added yet.

Every new public module, re-export, type, trait, constructor, method, provider
shape, callback shape, and naming uncertainty will be listed here before its
checkpoint closes.

## Pending eyes

- Checkpoint 1: ellipsis glyph choice, cut spacing, end/middle appearance in
  both themes, and the documented bidi policy.
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
| C1-01 | Vocabulary and structural absence | `Clip`, `EllipsisEnd`, and `EllipsisMiddle` are text-layout concepts; world text cannot be constructed without one | `view::Wrap`, text `glyphon::Wrap`, and private Node fields demonstrate typed declaration; no overflow exists | Planned |
| C1-02 | Fits within width | All three policies preserve byte-identical visible text and normal measurement | Inline shaping cache already measures constrained text | Planned |
| C1-03 | End ellipsis | Long world text yields a real `…`, preserves the maximal fitting head, and never splits a grapheme | `unicode-segmentation` and the text engine already own grapheme boundaries; no truncation owner exists | Planned |
| C1-04 | Middle ellipsis | Long path-like text preserves fitting head and tail around one `…` without splitting graphemes | Same grapheme owner; two-anchor fitting is missing | Planned |
| C1-05 | Tiny/zero constraints | Resolution is deterministic when even the ellipsis cannot fit; no negative or looping search state | Existing measurement accepts bounded width/height | Planned |
| C1-06 | Cache identity | Same text/metrics/constraints with different overflow modes cannot share a cached result; unchanged mode still hits | `text::layout::inline::TextKey` already keys metrics, bounds, and wrap; overflow is absent | Planned |
| C1-07 | Author diagnostic | Width-constrained author text that does not fit reports a diagnostic condition rather than silently becoming world text | Diagnostics owns counters; no author-overflow signal exists | Planned |
| C1-08 | User text non-regression | TextArea scroll/wrap and TextBox reveal/preedit/caret remain unchanged | Separate area/field layout engines and existing whole journeys already own behavior | Planned |
| C1-09 | Honest world caller | At least one programmer-uncontrolled value declares its overflow at construction | Text-editor file/status/debug values are candidates; ordinary authored labels remain author text | Planned |
| C1-10 | Bidi | End and middle behavior follow one documented v1 visual/logical policy with deterministic mixed-direction witnesses | Per-glyph bidi hit truth exists; overflow policy is absent | Planned |

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

## Failure and reduction ledger

No campaign failure has been observed yet.

## Commit ledger

| Checkpoint | Commit | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- |
| Campaign | First ledger commit | — | — | — | Pending commit |

## Final fixed-point sweep

Pending all six independently green checkpoints, cross-checkpoint replay,
public API review, performance comparison, pending-eyes transfer, full ritual,
and clean-worktree proof.
