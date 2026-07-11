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
| 2 | Typed `FrameContent` over the existing roles | Complete | One content discriminant; 825/8 in 0.88 s; three smokes; release gate held |
| 3 | Uniform virtual region/list | In progress | Current-tree provider/viewport/pinning census is the next action |
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

Checkpoint 2 added no public API. `FrameContent`, its family payloads, and the
single-representation witness are crate-internal layout structure.

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

## Checkpoint 2 census — typed `FrameContent`

Current-tree ownership claim: `Frame` has one common envelope for node identity,
path, geometry, active geometry, focus/selection presentation, overlay policy,
background, clip, generic target/binding/action, and the label projection shared
by many roles. Role-specific semantic payload belongs in one `FrameContent`.
The current parallel cluster is `viewport`, `text_area_layout`,
`text_box_layout`, `text_box_text_rect`, `slider_track_rect`, `checkbox`,
`radio`, `text_area`, `text_box`, `slider`, and the three shortcut fields;
`text`, `text_wrap`, and `world_text_overflow` are derived text-family payload.

The existing house pattern is `view::Control`: one enum owns mutually exclusive
typed leaf models and narrow accessors project them. `FrameContent` will use the
same pattern. `Frame::role()` will derive from content, deleting the independent
role discriminant so role/payload disagreement is structurally impossible.
Generic binding/action remains common because `Element` may truthfully bind
structural presentation roles; it is not a leaf-role payload.

### Family migration map

| Migration family | Roles | Truthful payload after migration | Existing downstream consumers |
| --- | --- | --- | --- |
| Structural | Root, Stack, MenuBar, Panel | restricted structural discriminator; no leaf payload | layout algorithm, composition identity, generic background/children paint |
| Choice | Checkbox, Radio | one choice enum carrying exactly one typed model | active rect, shared choice paint, checked/selected state |
| Text | Label, SectionHeader, TextArea, TextBox | label kind/overflow or typed area/field model plus owned layout geometry | measurement, hit/drag caret mapping, runtime text projection, scene text paint |
| Slider | Slider | model plus derived track rect | hit value mapping, capture/gesture action, scene paint, native presentation transform |
| Scroll and floating | Scroll, FloatingPanel | scroll viewport only on Scroll; floating remains a unit semantic variant | reveal/wheel/chrome and overlay/native layer routing |
| Remaining specialized | Menu, Binding, Separator, Button | bound presentation follows any truthfully bound role; Separator alone carries an unbound reserved menu column; roles remain unit variants | menu/palette row paint, shortcuts, menu actions, button paint |

The complete widget-audit downstream index remains the migration navigation
map: `layout/algorithm`, `layout/measure`, `layout/frame`, `layout/chrome`,
`runtime/{pointer,visual,presentation}`, `scene/paint` and its choice/slider/text
modules, and Slider's native-paint branch. No phase-local policy is being
consolidated merely because it also inspects a role.

| Cell | Scenario | Required result | Current mechanism / migration proof | Status |
| --- | --- | --- | --- | --- |
| C2-01 | One discriminant | `Frame` stores `content: FrameContent`; `role()` derives from it | Independent role deleted in first family commit | Held |
| C2-02 | Structural family | Root/Stack/MenuBar/Panel cannot carry leaf payload | Restricted `StructuralRole` inside `FrameContent` | Held |
| C2-03 | Choice family | Checkbox and Radio share geometry/paint family but carry exactly one correct model | `ChoiceContent::{Checkbox, Radio}` | Held |
| C2-04 | Text family | Label/header/area/field payloads are exclusive; user-text layout and world overflow retain behavior | `TextContent` variants own model/layout/geometry; 141 focused text/layout journeys held | Held |
| C2-05 | Slider | Slider model and track geometry travel together and no other role can hold them | `SliderContent`; widget, scene and native-transform slider journeys held | Held |
| C2-06 | Scroll/floating | Viewport mutation is legal only for Scroll; floating behavior stays unit/common presentation | `ScrollContent`; 41 scroll and 8 floating focused journeys held | Held |
| C2-07 | Remaining specialized | Shortcut display/width follows truthful bound roles; Separator alone owns unbound row reservation | `BoundContent` plus `SeparatorContent`; reduced menu/palette journeys held | Held |
| C2-08 | Interaction equivalence | Hit, action, drag, focus, capture and reveal journeys remain byte-for-behavior equivalent | Existing narrow accessors retained; full suite held | Held |
| C2-09 | Presentation equivalence | Paint, overlay, chrome and Slider native transform retain exact branches | Downstream consumers unchanged; full suite and smokes held | Held |
| C2-10 | Single representation | Legacy role and optional payload cluster are absent; one source witness pins this | `frame_content_is_the_single_role_payload_representation` passes | Held |
| C2-11 | Performance/non-scope | Suite gauge and release text benchmark hold; no table role/API/feature appears | 825/8 in 0.88 s; release figures below; no public API or role added | Held |

Expected exclusions: no public API, role, widget, table behavior, capability
table, generic property bag, or phase-policy consolidation. Family commits may
temporarily coexist with unmigrated legacy fields, but every family commit must
be green and the checkpoint boundary may contain only `FrameContent`.

## Checkpoint 3 census — uniform virtual region/list

Ownership claim after inspecting the current tree: virtualization is not a
second widget runtime. A provided list remains a view node and uses the normal
retained composition, command resolution, transient interaction projection,
layout frames, viewport/chrome, scene paint, and runtime pruning sentence. The
new coordination seam is a bounded two-pass fixed point: layout derives a
logical visible range from real viewport geometry; runtime rebuilds the normal
view with only that range plus pins; retained composition reconciles provider
keys; layout repeats once against the requested materialization.

Existing mechanics to reuse:

- `layout::Viewport` already owns clamped scroll offsets, content extent,
  reveal geometry, wheel consumability, clips, and scrollbar projection.
- `composition::Tree` already reconciles explicit keyed siblings and reports
  true removal to session pruning; provider keys extend that existing match
  rule rather than creating a parallel identity map.
- `view::Node` already projects focus, capture-relevant pointer targets,
  TextArea/TextBox interaction, bindings, and focus order through the retained
  tree. Provider rows must become ordinary public nodes before those passes.
- `draft::Input` already retains inactive drafts with a bounded store and
  deletes them only when composition reports actual identity removal.
- runtime rebuild and layout invalidation are already distinct. Virtual range
  convergence may request one ordinary rebuild inside frame preparation; it
  does not ask applications to calculate ranges.

Dematerialization must therefore be a distinct composition outcome from
removal. A materialized provider child leaving the visible/pinned set drops its
view node without contributing removed node/element identities while its key
still exists in the provider. If `Provider::index_of(key)` says the key no
longer exists, normal removed-subtree reporting and session pruning apply.

### Planned public contract

The narrow v1 provider surface is synchronous and flat:

- total logical length;
- stable `virtual_list::Key` for an index;
- efficient reverse lookup from key to current index, needed for reorder,
  pinning, and deletion truth without scanning one million rows;
- public row construction returning an ordinary `view::Node` for a requested
  logical index.

The list constructor requires a stable list id, uniform positive row height,
and a provider. Overscan is a small bounded setting. Variable heights, async or
streaming rows, selection, headers, tracks, and table semantics are excluded.
The exact module/re-export names remain an API flag until the first honest
caller compiles.

| Cell | Scenario | Required result | Existing owner / missing seam | Status |
| --- | --- | --- | --- | --- |
| C3-01 | Structural grammar | One provided-container node derives public row children; no exhaustive app-authored child list | Node/Widget/Composition exist; provider payload and role are missing | Planned |
| C3-02 | Initial viewport | First frame materializes only visible rows plus bounded overscan | Viewport exists; range request/rebuild fixed point is missing | Planned |
| C3-03 | Small and jump scroll | Nearby and million-row jumps request correct ranges with bounded provider calls | Scroll offset/clamping exists; uniform range math is missing | Planned |
| C3-04 | Growth and shrink | Content extent follows count; offsets clamp; stale rows do not survive true shrink | Viewport feedback and provider reverse lookup are reusable | Planned |
| C3-05 | Stable-key reorder | Visible retained row identity follows provider key rather than logical index | Composition keyed-sibling matching exists only for static ids | Planned |
| C3-06 | Viewport resize and scale | Logical range updates for new viewport height; scale-only changes preserve logical identity | Layout rebuild/scale boundary exists; request comparison is missing | Planned |
| C3-07 | Measurement bound | Only materialized visible/pinned rows produce nodes and frames; no provider-wide intrinsic scan | Fixed uniform height permits arithmetic content extent | Planned |
| C3-08 | Paint bound | Paint items and per-scroll work remain proportional to viewport rows plus overscan/pins | Clip and scene projection already consume frames | Planned |
| C3-09 | Focus pin | Focused row remains materialized and may be clipped when scrolled away | Focus projection exists; row-key association/pin collection is missing | Planned |
| C3-10 | Capture pin | Captured row remains materialized until capture ends | Pointer capture target exists; row-key association is missing | Planned |
| C3-11 | Active edit pin | Text row with active draft/preedit remains materialized | Draft input exposes active target; row-key association is missing | Planned |
| C3-12 | Selection non-pin | No selection concept or accidental pin appears in checkpoint 3 | Selection is deliberately deferred to C4 | Planned |
| C3-13 | Dematerialized draft | Inactive TextBox draft survives ordinary scroll out/back | Bounded draft store exists; composition must suppress false removal | Planned |
| C3-14 | Provider deletion | Deleted key ends focus/capture/edit/draft through normal pruning | Removed-subtree path exists; provider key existence decides truth | Planned |
| C3-15 | Focus-before-move | A logical target is materialized before focus transfer | Runtime focus owner exists; keyed pre-materialization seam is missing | Planned |
| C3-16 | Million-row witness | Node/frame/paint/provider-call counts stay bounded across initial, jump, reorder, shrink and resize journeys | Deterministic counter provider and inspection tests are missing | Planned |
| C3-17 | Honest caller | Control gallery exercises a real large provided list made of public widgets | Gallery and external smokes exist | Planned |

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
| E-008 | C2 family gates 1–5 | Held | 90 layout, 5 choice, 23 TextBox, 28 text-input, 11 slider, 41 scroll and 8 floating focused journeys passed |
| E-009 | C2 remaining-family first gate | Failed and reduced | Menu Separator rejected shortcut width; palette Label lost bound shortcut projection |
| E-010 | C2 reduced remaining family | Held | Four exact failures plus complete menu-popup and command-palette families passed |
| E-011 | C2 full library and smoke gate | Held | 833 discovered: 825 passed, 8 deliberately ignored, 0 failed in 0.88 s; three smokes exited 0 |
| E-012 | C2 release text gate | Held | 30.158 ms load; 2.730 / 3.702 / 3.872 / 3.621 µs typing; 35.557 / 37.060 ns clones; 0.67 s benchmark work |

## Failure and reduction ledger

The first C1 full run found three stale editor-debug expectations. Reduction
showed no behavior or ownership failure: each expected the former single string
that mixed authored diagnostics with file/status world data. Commit `3ef75719`
changed those witnesses to assert the new provenance split and the added author
overflow instrument. All three focused journeys and the repeated full suite
then passed. No unresolved failure crosses the checkpoint boundary.

The first C2 remaining-family gate disproved the census claim that shortcut
presentation belonged only to `Role::Binding`. The reduced evidence showed two
existing truths: palette results are bound Labels, and menu Separators reserve
the shared shortcut column without a binding. The final model therefore keeps
typed bound presentation common to any bound role and gives Separator only its
reserved width. Four exact failures and both complete families passed after the
correction. No failed representation crosses the checkpoint boundary.

## Commit ledger

| Checkpoint | Commit | Files | Insertions | Deletions | Outcome |
| --- | --- | --- | --- | --- | --- |
| Campaign | `0fad3ff2` | 1 | 151 | 0 | Crash-safe ledger established |
| Campaign | `e3f1199f` | 1 | 7 | 6 | Checkpoint 1 marked in flight |
| 1 | `e6ce36b2` | 22 | 539 | 17 | Overflow owner, required declaration, cache identity, diagnostics, and tests |
| 1 | `c8deeb29` | 2 | 32 | 5 | Doctrine and honest text-editor callers |
| 1 | `3ef75719` | 2 | 10 | 3 | Reduced full-gate expectations updated |
| 1 | `dfa728f2` | 6 | 23 | 37 | Caller-local path truncation retired; world policy is sole display owner |
| 2 | `873b6c5f` | 1 | 54 | 0 | Current-tree family census recorded |
| 2 | `32c50996` | 1 | 81 | 9 | Independent role deleted; structural content introduced |
| 2 | `23e9a8f8` | 1 | 30 | 17 | Choice models made exclusive |
| 2 | `9afe26dc` | 1 | 110 | 62 | Text payload family migrated |
| 2 | `0acfa2c4` | 1 | 27 | 11 | Slider model and track migrated together |
| 2 | `e0964c13` | 1 | 14 | 7 | Scroll viewport restricted to Scroll content |
| 2 | `35736441` | 2 | 95 | 19 | Bound/specialized family finished; absence witness added |

## Final fixed-point sweep

Pending all six independently green checkpoints, cross-checkpoint replay,
public API review, performance comparison, pending-eyes transfer, full ritual,
and clean-worktree proof.
