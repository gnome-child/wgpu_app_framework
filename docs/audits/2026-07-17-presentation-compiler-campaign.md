# Presentation Compiler campaign — The Last Layer Takes the Class

**Status:** HALTED BY RECEIPT — PC-005 FAILED THE FLAT-COMMIT ARCHITECTURAL GATE

**Formulation date:** 2026-07-17

**Formulation snapshot:** `master` at `1fe6af199b501163e1f5ccf0d8065e9b079a43ea`, with the inherited SE-009 worktree listed below

**Execution base:** `master` at `1fe6af199b501163e1f5ccf0d8065e9b079a43ea`; frozen by the PC-000 ignition receipt on 2026-07-17 with the inherited SE-009 worktree below protected in place

**Campaign-owned production changes:** none at PC-000 ignition

**Campaign-owned artifacts:** this ledger and the [Presentation Compiler source census](2026-07-17-presentation-compiler-source-census.md)

**Predecessors:** [Retained Renderer campaign](2026-07-14-retained-renderer-campaign.md), [Scrolling Engine campaign](2026-07-17-scrolling-engine-campaign.md)
**Branch / commit / push authority:** none implied by formulation; execution must record any later authority explicitly

The Scrolling Engine campaign is retained as evidence and history. This campaign supersedes it only as the execution authority for the CPU presentation architecture identified by that work. It does not retroactively rewrite its receipts.

The formulation worktree already contains uncommitted SE-009 changes. They belong to the earlier campaign and are protected:

- `docs/audits/2026-07-17-scrolling-engine-campaign.md`
- `docs/audits/2026-07-17-scrolling-engine-source-census.md`
- `src/composition/tree.rs`
- `src/diagnostics/{mod,render,residency}.rs`
- `src/layout/frame.rs`
- `src/list.rs`
- `src/render/{debug,renderer,retained,text_renderer}.rs`
- `src/runtime/presentation.rs`
- `src/scene/{primitive,spatial}.rs`
- `src/table.rs`
- `src/tests/{architecture,composition_tests,layout_scene,residency_tests}.rs`
- `src/view/node/{builder,mod,traversal}.rs`
- `tools/renderer_debug/{README.md,src/lib.rs,src/main.rs}`

PC-000 must re-census this set before any production edit. No checkpoint may overwrite, revert, stage, reformat, or silently absorb inherited work.

---

## Opening constitution

> Structure belongs to the commit; values belong to property state.

> Each field has exactly one mutation clock.

> Laws are inviolable; shapes are candidates.

> No property-clock value may participate in a key whose reuse claim spans property ticks: semantic content, local geometry, text shaping, presentation fragments, or GPU content resources. A property-stage cache may compare property values only to reuse explicitly property-derived output and may not promote that comparison into semantic identity, content revision, or geometry revision.

The last law is the campaign tombstone. It is deliberately narrower and stronger than “remove rectangles from cache keys.” Local geometry is legitimate cache currency when it describes local geometry. The present violation is that viewport translation is subtracted into each row rectangle and the contaminated rectangle is then treated as reusable presentation identity. The repair is to restore content-local geometry and apply scroll through the existing spatial/property grammar.

This is the borrowed-clock family, third arrest. A property value borrowed as semantic identity manufactures change without a semantic mutation. The same disease previously appeared when a parent presentation epoch was borrowed as popup content identity. The visible symptom differs; the constitutional failure is identical.

The campaign has two mandatory architectural outcomes:

1. scrolling inside prepared coverage must be a property tick with literal-zero semantic presentation compilation;
2. changing residency must preserve every unchanged overlapping row and perform work proportional to the keyed delta.

Everything heavier—CPU active/pending preparation, immutable provider snapshots, worker compilation—is conditional. It enters only through a measured residual and an explicit admission receipt.

---

## Mission

The mission is to reduce CPU work in the renderer’s upstream presentation compiler until large interactive tables can scroll without monopolizing the UI event path, while preserving the framework’s full semantic contract.

“Presentation compiler” is the campaign name for the CPU derivation that turns application/composition facts into layout-owned local geometry and scene-owned renderer-independent presentation. It names a responsibility, not a preselected type or public subsystem.

The target is stated in currencies:

- an in-runway property tick performs zero row materialization, composition reconciliation, row measurement, text shaping/preparation, row paint, row-fragment assembly, and GPU content upload;
- a pure scroll-driven residency advance, with model membership/order stable, performs row work only for entering, departing, or content-revised logical rows, plus bounded container bookkeeping;
- unchanged overlapping rows preserve composition identity, local geometry, text artifacts, scene fragments, hit/accessibility projection, and GPU content resources;
- scroll offset remains property-clock data and cannot advance semantic, content, geometry, residency, or presentation-generation identity;
- input, selection, caret, IME, accessibility, and pixels continue to derive from one successfully presented generation;
- the native event path never contains an unbounded or resident-window-sized preparation slice.

The campaign may make major architecture changes and break APIs. API breakage is permitted when it repairs ownership or value semantics; it is not evidence that any particular replacement has been admitted.

### Campaign-level verdict

The GPU renderer is no longer the dominant table-scroll failure. The retained-renderer work reduced GPU draw p95 from roughly 8.4 ms to roughly 1.7–2.1 ms, collapsed approximately 88 passes to about 8, and centralized text preparation. The remaining observed bottleneck is synchronous CPU presentation construction: layout/scene work around 12–15 ms p95 and native event passes around 30–63 ms p95 on a 240 Hz display whose frame interval is 4.17 ms.

Those phase percentiles are not additive. They establish owner and scale, not a synthetic total.

---

## Authority

When sources disagree, use this order:

1. constitutional ownership and mutation-clock laws;
2. observed semantic behavior and successful-present truth;
3. `docs/master_design.md` and completed campaign decisions;
4. live owner APIs and their architecture ratchets;
5. reproducible native receipts from this campaign;
6. external renderer precedent;
7. current implementation shape and convenience.

The attached table-campaign diagnosis is an ignition brief, not architectural authority. Its load-bearing facts must be re-received at PC-000. The current source census records which claims are verified, inferred, or still open.

External precedent can suggest mechanisms and negative space. It cannot waive local ownership laws, semantic behavior, or the admission gate.

### Independent execution law

Each checkpoint is a separately green boundary. It must record:

- the exact question it owns;
- allowed and forbidden scope;
- positive witnesses and deliberate negative controls;
- counters, workload, environment, and thresholds;
- implementation and displaced-path deletion;
- source and test evidence;
- the residual handed to the next checkpoint.

No checkpoint closes “mostly.” The only closing states are `COMPLETE`, `REJECTED BY RECEIPT` for an optional mechanism, or an explicit blocker.

Generated receipts may live under ignored `target/` paths. The ledger must copy their environment, workload, and load-bearing values so the campaign never depends on an ignored file continuing to exist.

---

## Protected seams

These are presumptively retained unless a checkpoint produces a direct contradiction:

- `composition` owns `NodeId`, semantic lifetime, and the root `Changes` stream;
- `interaction::scroll::AxisAdjustment` owns each canonical axis value and revision; layout authors its range/page configuration;
- `layout` owns local measurement, geometry, and hit meaning;
- `scene` owns typed content, spatial topology, `Commit`, `Residency`, and compatible sampled `Properties` snapshots—not canonical scroll;
- runtime presentation owns active/pending selection, activation, and presented receipts;
- `scene::Stack` remains the sealed native renderer handoff;
- the renderer owns GPU realization and borrows upstream identity/revisions;
- surface tenancy and surface acquisition stay below renderer realization;
- the retained renderer’s shared text cache/atlas and batched realization remain protected;
- `scene::SpatialTopology` remains the sole scroll-ancestry compiler;
- presented spatial evaluation remains shared by pixels, hit testing, selection, caret, IME, and accessibility;
- desired/candidate state never becomes submitted or visible truth without the existing receipt grammar.

This campaign improves the compiler that feeds the retained renderer. It must not reopen solved GPU batching, clipping, glyph ownership, alpha/material, popup, or surface laws without a new causal receipt.

---

## Current indictment

### Verified synchronous path

The current residency path is effectively:

```text
native event / redraw
  → canonical scroll and residency request
  → FrameNeed::Residency becomes Invalidation::Rebuild
  → remove the relevant layout cache
  → clone/rematerialize the installed view
  → bind bounded entering list/table controls
  → reconcile the composition tree
  → project interaction and focus state
  → construct fresh layout frames for the resident subtree
  → scan/assemble scene presentation
  → prepare retained GPU realization
  → submit and present
```

Provider calls and new widget construction are already bounded to entering rows. The defect is downstream: stable logical rows are repeatedly walked, laid out, painted, and assembled.

### Verified concrete clock fault

In `src/layout/algorithm.rs`, virtual-row placement computes:

```text
y = viewport.y + row_index × row_height − resolved_scroll.y
```

In `src/layout/frame.rs`, the resulting absolute `rect` participates in `SceneKey`. The key also contains a complete `Viewport` whose fields include requested and resolved offsets. A one-pixel scroll can therefore change the presentation key of every resident row while their semantic content revisions remain unchanged.

This explains why stable slot identity and text-buffer reuse did not produce stable scrolling: the cache consults a key whose geometry is married to the viewport.

### Verified receipt bracket

The current retained receipt records, among other facts:

| Measure | Observed p95 / count |
|---|---:|
| GPU draw | 1.745 ms |
| property draw | 2.007 ms |
| semantic draw | 1.942 ms |
| surface acquire | 0.042 ms |
| batch preparation | 0.329 ms |
| encode/submit/present | 1.335 ms |
| scene assembly | 14.825 ms |
| layout | 12.489 ms |
| view rebuild | 6.187 ms |
| composition reconciliation | 4.006 ms |
| native event pass | 30.819 ms |
| full redraws | 312 |
| layout reuses | 131 |
| scene nodes reused / rebuilt | 43,200 / 52,146 |
| wheel samples | 768 |
| samples needing residency | 750 |

Residency traces report approximately 163–170 painted nodes and 81–86 primitives prepared per advance despite small row deltas. These numbers are formulation evidence only; PC-000 must freeze a reproducible baseline and establish attribution.

### Control case

Large native text documents scroll cleanly. That control weakens explanations based on common adjustment handling, surface acquisition, basic text rendering, or input plumbing. The campaign must compare the large-text path directly with the table/list path at every causal measurement boundary.

---

## Root-cause model

The failure has four layers:

1. **Clock contamination.** Property translation participates in semantic/local-presentation keys.
2. **Missing retained derivation.** Composition identity survives, but layout frames and scene fragments do not have an equally stable row-level lifetime.
3. **Coarse residency invalidation.** A keyed set delta is converted into a resident-subtree rebuild.
4. **Latency placement.** The resulting resident-sized work runs synchronously inside the UI event/redraw path.

The ordering matters. Fixing latency placement before repairing identity and delta work risks moving waste to another thread and creating a second truth. Damage tracking before stable fragments merely tracks damage to objects that are reconstructed anyway.

### Complexity target

Let:

- `R` be resident rows;
- `D_s` be the union of entering, departing, and content-revised rows for a pure scroll residency advance whose model order is stable;
- `A_m` be the declared affected set for a model mutation: inserted, removed, replaced, moved/order-changed keys, plus keys whose local position or height-prefix dependency truly changes;
- `C` be bounded container/chrome work;
- `P` be a property-only scroll tick.

The current presentation path behaves materially closer to `O(R)` on residency changes. The required shape is:

| Event | Required row work |
|---|---:|
| `P` inside prepared coverage | `0` |
| pure scroll residency advance | `O(D_s) + O(C)` |
| one row content revision | `O(1)` plus affected declared dependencies |
| insert/remove/replace/move | `O(A_m)` plus named bounded/logarithmic order/height-index maintenance |
| column metric/style change | explicitly classified global/column-dependent work |
| broad semantic rebuild | `O(affected semantic subtree)` |

The campaign does not claim all table mutations are constant time. It requires that each cause pay only for facts it owns.

---

## Clock and ownership contract

| Currency / fact | Sole owner | Advances for | Forbidden causes or keys |
|---|---|---|---|
| application/model revision | application/provider | actual model mutation | scroll, residency, presentation attempts |
| `list::Key` and item revision | `list::Model` | logical membership/content change | slot reuse, viewport position, presentation generation |
| canonical axis value/revision | `interaction::scroll::AxisAdjustment` | relative, absolute, scrollbar, reveal, and programmatic scroll requests; clamp caused by new configuration | scene sampling, residency attempts, presentation attempts |
| axis range/page/step configuration | `layout` authors; `AxisAdjustment` stores/applies atomically | content/viewport geometry or scroll-policy change | current property sampling, GPU work |
| desired offset | derived projection of the two canonical adjustments | either canonical axis changes | semantic/content/geometry identity |
| `NodeId`, semantic lifetime, `Changes` | `composition` | reconciled semantic structure/content | materialization alone, scroll, GPU work |
| local measurement/bounds/hit geometry | `layout` | constraints, content, metrics/style, actual local placement | current offset within declared topology/runway |
| typed content and spatial topology | `scene` | local content, order, bounds, declared effects | property samples, residency membership, presentation attempt |
| residency interval/revision | `scene::Residency` | bounded membership/order/coverage change | semantic content, current offset, presentation epoch |
| compatible property serial/value snapshot | `scene::Properties` | sampling canonical scroll/transform/opacity values for a scene handoff | becoming canonical input, content/geometry revision, topology, activation |
| resident-accepted / candidate / active / activation / present-submitted / visible receipts | presentation owner, with residency facts carried from `scene::Residency` | coverage admission, selection, readiness, submission, successful presentation | mutating canonical adjustment, upstream identity, or property snapshot |
| GPU resource reuse | `render` with carried currencies | relevant `NodeId`, content/geometry/topology/target changes | property serial, residency revision, presentation epoch, renderer ID |
| compiler task/slice token | private scheduler | cancellation, priority, progress | semantic/cache identity or visibility |
| popup generation | popup owner | popup-local source/geometry/material changes | parent presentation epoch |

### Forbidden marriages

- property offset, resolved viewport, or `PropertySerial` in semantic, text, local-geometry, row-fragment, or GPU content keys;
- `PresentationEpoch` in commits, residency, popup content identity, or content-resource keys;
- residency revision in semantic or GPU content identity;
- whole-commit pointer identity as a retained-subtree key;
- recycled slot or row index as logical cell identity;
- desired/candidate offset as input geometry before successful presentation;
- a compiler “dirty” bitstream competing with `composition::Changes`;
- task IDs or worker generations used as semantic freshness;
- a renderer-minted identity that replaces carried `NodeId` and owner revisions.

### Cache-key admission table

| Candidate field | Semantic / fragment key | Property-stage key | Reason |
|---|---|---|---|
| `NodeId` / provider row key / column id | admit | carry if needed | stable owner identity |
| content revision | admit | carry if needed | owner-declared content change |
| local constraints and local geometry revision | admit | carry | true derivation input |
| typography/style/material revision | admit | carry | true content/realization input |
| local clip/effect topology revision | admit | carry | true structural input |
| absolute viewport-space rect containing scroll | reject | compare only for property output | contaminated by property clock |
| canonical/requested/resolved offset | reject | admit only for explicitly property-derived output sampled from the canonical adjustment | property value, not `scene::Properties` authority |
| residency revision | reject | carry for coverage selection, not content reuse | membership clock |
| presentation epoch | reject | presentation routing only | activation clock |
| renderer allocation ID | reject | internal address only | not upstream truth |

### Cell identity

The logical table-cell address remains:

```text
(table identity, provider row key, column identity)
```

Slot index, visual row index, and resident position are replaceable addresses, never semantic identity.

---

## Required dataflows

### In-runway property scroll

```text
input
  → interaction::scroll::AxisAdjustment canonical value/revision
  → layout-authored range/clamp configuration already applied
  → desired/resident admission projections
  → compatible scene::Properties snapshot
  → presented spatial evaluation
  → existing retained GPU realization
  → submit / present receipt
```

No materialization, composition, measurement, text preparation, fragment assembly, or content upload is permitted.

### Residency advance

```text
desired interval
  + list::Model membership revision / ordered Change journal
  → keyed residency-and-order delta
  → bind entering / retire departing / revise changed / reorder moved keys
  → composition::Changes for actual semantic delta
  → retained local derivation for changed keys only
  → splice scene-owned row fragments into stable container topology
  → complete candidate using existing generation grammar
  → atomic activation and successful-present receipt
```

Unchanged overlap is not “reused after recomputation.” It is not visited by expensive phases.

For pure scroll, the membership/order journal is unchanged and the delta is `D_s`. For model mutation, `Change::{Insert, Remove, Replace, Move}` and item revisions define `A_m`; a same-key move is work even when entering and departing sets are empty.

### Broad semantic change

```text
model/view mutation
  → owner revision and composition::Changes
  → affected retained derivation frontier
  → layout/scene changes
  → complete presentation candidate
  → activation / receipt
```

### Semantic projection

Selection, caret, IME, accessibility, and hit testing consume the same presented local geometry plus presented spatial properties used by pixels. Candidate geometry cannot leak into active input semantics.

---

## Presentation compiler charter

`PresentationTree` is a candidate internal shape. The campaign does not require that exact type, one monolithic tree, or a new public module.

If admitted, its charter is:

> `PresentationTree` is private UI-derived structure. It owns the bounded lifetime, structural sharing, and resumable preparation state of node-local layout and scene derivations keyed by carried composition identity and owner-published revisions.
>
> It does not mint semantic identity, model/provider truth, composition change facts, layout meaning, scene meaning, scroll/property values, residency truth, presentation epochs, activation, visible receipts, GPU identity/resources, input state, or platform work.
>
> Composition remains the sole owner of `NodeId`, semantic lifetime, and the root change stream. Layout remains the sole author of local geometry and hit meaning. Scene remains the sole author of typed content, spatial topology, commit/residency/property handoffs, and renderer-independent fragments. Presentation remains the sole owner of active/pending selection, property sampling, activation, and successful-present receipts. Render remains the sole GPU realization owner.
>
> Dirty/readiness/frontier state is derived bookkeeping for one candidate from carried changes and declared global causes. It cannot be externally marked as an independent invalidation truth and cannot escape as another revision currency.
>
> If CPU active/pending preparation is later admitted, an immutable presentation snapshot becomes payload within the existing presentation generation. It does not acquire a separate generation clock, activation path, queue, or receipt grammar.

The current UI territory is one coordinated strongly connected component. Retention begins as a private internal stage, not a new crate or public architectural root. A physical seam requires its own One-Way admission receipt.

### One-way owner graph

```text
provider / view / session facts
    ↓
composition::Tree + composition::Changes
    ↓
private presentation compiler / retained derivation
    ├── layout-owned local geometry and hit meaning
    └── scene-owned fragments, topology, Commit, Residency, Properties
    ↓
runtime presentation owner
    active / pending / activation / receipts
    ↓
scene::Stack
    ↓
render retained realization
    ↓
Canvas / Surface
    ↓
platform
```

Enforced edges:

- `composition` imports no compiler, layout, scene, runtime, render, or platform;
- the compiler consumes `composition::{Tree, Changes}` and publishes no competing upstream change stream;
- layout knows no runtime/presentation/render policy;
- scene remains renderer-independent;
- the compiler may coordinate layout and scene but cannot recompute either owner’s decisions;
- runtime may own progress and scheduling; the compiler cannot call back into runtime through a service locator;
- render receives `scene::Stack` only, never providers, layout frames, `PresentationTree`, or worker snapshots;
- input consumes `present_submitted` truth only;
- diagnostics observes owner-published receipts; production compiler code cannot depend on diagnostics;
- no callback-smuggled reverse edge or blanket visibility widening is allowed.

### Spatial ownership

Content-local rows reuse the existing spatial grammar:

- layout authors local row bounds and fixed viewport coverage;
- `scene::SpatialTopology` compiles ancestry once;
- `interaction::scroll::AxisAdjustment` owns the canonical scroll value/revision, layout authors its bounds/configuration, and `scene::Properties` owns only the compatible sampled translation for a scene handoff;
- the presented evaluator applies it consistently to pixels and semantics.

No new transform evaluator may appear in the compiler, table, renderer, or worker protocol.

---

## Requirements-first ledgers

### Required facts and disposition

| Required fact | Current state | Required disposition |
|---|---|---|
| stable logical row/cell identity | present in model/composition | carry through all retained derivations |
| ordered model-mutation journal | `list::Change::{Insert, Remove, Replace, Move}` exists; table `Rows` currently reports revision 0/empty changes | repair provider gap before claiming incremental order mutation |
| row-local geometry | contaminated by viewport offset | repair to content-local |
| fixed viewport coverage/clip | exists in scene doctrine | preserve independently of rows |
| row text layout/preparation | reusable pieces exist but resident walk persists | retain by owner inputs |
| row scene presentation | bodies cached but commit assembly scans broadly | introduce renderer-independent fragment lifetime |
| keyed residency delta | entering binds bounded; downstream rebuild coarse | preserve overlap end to end |
| active/pending CPU generation | not independently established | conditional PC-007 only |
| worker-safe provider facts | live `Rc`/`RefCell`/callbacks | conditional immutable snapshot PC-008 only |
| cancellation/budgets | coalescing exists; selected candidate can monopolize | measure, then admit bounded scheduling |
| successful-present semantics | existing generation/receipt grammar | preserve and extend, never fork |

### Change-species admission

| Cause | Composition | Layout | Scene fragment | Residency | Properties | GPU content |
|---|---|---|---|---|---|---|
| canonical offset inside runway | 0 | 0 | 0 | 0 | sample adjustment into compatible property update | 0 |
| interval crosses runway with stable model order | keyed `D_s` only | entering/revised | entering/revised | update | sample/update | entering/revised |
| insert/remove/replace/move | affected semantic keys/order | `A_m` local/order/height dependencies | `A_m` fragments | update if coverage changes | sample if canonical clamp changes | affected resources |
| one row text change | affected cell/row | affected local dependency | affected fragment | usually 0 | 0 | affected resource |
| selection/caret state | owner-declared affected controls | only if geometry changes | affected semantic/visual fragment | 0 | possibly property if admitted | affected resource only |
| column width change | declared affected table dependency | affected rows/columns | affected fragments | 0 | 0 | affected geometry/content |
| theme/font metric change | declared broad revision | affected tree | affected fragments | 0 | 0 | affected resources |
| scroll-container resize | container + coverage | affected constraints | affected topology/fragments | interval may change | update | affected resources |
| device/surface event | 0 | 0 | 0 | 0 | target fact only | rebuild target-owned realization |

Zero means literal zero, not “fast.”

### Lifetime and cleanup

| Object | Born | Reused while | Retired |
|---|---|---|---|
| list/table slot | logical row enters/recycles | keyed resident binding valid | departure/rebind |
| retained row derivation | owner inputs first compiled | identity/revisions/constraints valid | departure beyond cache policy or invalidation |
| scene row fragment | scene facts compiled | content/local geometry/topology valid | owner revision or bounded eviction |
| residency candidate | desired interval accepted | source facts current | activation, supersession, cancellation |
| canonical axis adjustment | first scroll target configuration/input | until owner processes a request or new configuration clamps it | next adjustment revision |
| scene property sample | canonical input sampled for compatible scene | until newer compatible sample | next property serial |
| GPU resource | renderer realizes carried key | key/target valid | bounded eviction/target loss |
| optional worker snapshot | UI authority freezes pure value | source revisions current | completion, cancellation, stale rejection |

All caches require explicit bounds. “Retained” never means immortal.

### Failure and activation

- allocation or preparation failure cannot partially activate a candidate;
- stale or cancelled work cannot update active semantic, layout, scene, or input truth;
- active remains drawable while an admitted pending candidate prepares;
- activation is atomic across pixels and submitted semantic geometry;
- successful `present` advances visible receipts; preparation and submission attempts do not;
- device loss may discard target-owned realization without mutating semantic identities;
- rollback reuses the last complete active generation rather than mixing old/new subtrees.

---

## Semantic constraints

Optimization cannot turn table cells into decorative labels.

- read-only text remains selectable and copyable;
- editable cells retain caret, draft, deletion, focus, commit, and cancellation behavior;
- IME placement and composition agree with presented text geometry;
- accessibility bounds, hit testing, selection, caret, and pixels share submitted projection;
- shift-range selection may make multiple resident rows editable;
- focus survives valid recycling and is retired intentionally when its semantic owner leaves;
- no visual cache becomes a second model or input authority;
- variable-height rows, sticky/table chrome, nested scroll, scale, and clipping remain correct;
- reverse scrolling, direction changes, stopping/restarting, and thumb jumps cannot expose stale bindings.

The previous lightweight-cell experiment is a prohibited shape because it broke semantic parity.

---

## Industry precedents and local rulings

The research set is deliberately mixed. The campaign imports principles, not brands.

| System | Official evidence | Useful principle | Local ruling |
|---|---|---|---|
| GTK 4 | [`GtkListView` creates rows for visible items](https://docs.gtk.org/gtk4/class.ListView.html), while [`GtkSnapshot` builds `GskRenderNode` trees](https://docs.gtk.org/gtk4/class.Snapshot.html) and can append an existing node | bounded visible widgets, immutable renderer-independent render nodes, explicit transform/clip structure | supports retained row fragments; does not justify per-row GPU surfaces |
| Qt Quick | [scene graph nodes](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html), [`TableView` delegate recycling](https://doc.qt.io/qt-6/qml-qtquick-tableview.html), [default renderer](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html) | recycled visible delegates, retained node tree, transform nodes, renderer detached from items | mandatory “Qt class”: retained local presentation and transform scroll on-thread before workers |
| Flutter | [Inside Flutter](https://docs.flutter.dev/resources/inside-flutter) | persistent render tree, clean-subtree cutoff, stable constraints, sliver/on-demand children | supports sublinear update and row-local retention; tree surgery still needs local ownership |
| Jetpack Compose | [Compose phases](https://developer.android.com/develop/ui/compose/phases) | phase-local read tracking; defer frequent scroll reads to placement/draw instead of composition | direct precedent for clock divorce; no general reactive dependency engine is admitted |
| Chromium | [compositor architecture](https://www.chromium.org/developers/design-documents/compositor-thread-architecture/), [tree activation](https://www.chromium.org/developers/design-documents/gpu-accelerated-compositing-in-chrome/) | active/pending isolation, property scroll, complete activation, scheduler grammar | optional ceiling only after PC-006; extend the existing generation, never create parallel truth |
| Firefox | [Async Pan/Zoom](https://firefox-source-docs.mozilla.org/gfx/AsyncPanZoom.html) | async spatial relationships and hit-test tree distinct from content production | reinforces shared spatial truth; does not admit a second hit/access evaluator |
| Iced | [`Widget` / `Tree` API](https://docs.rs/iced_core/latest/iced_core/widget/trait.Widget.html) | retained widget state can coexist with repeated layout/draw calls | warning: stable widget state is not retained presentation |
| egui | [`ScrollArea::show_rows`](https://docs.rs/egui/latest/egui/containers/scroll_area/struct.ScrollArea.html#method.show_rows) | bounded visible-row construction can make an immediate model viable | warning: the label “retained” is insufficient; counters must prove bounded work |

### Named deviations

- Unlike Qt, this framework keeps semantic controls and renderer-independent scene facts explicitly separated by `scene::Stack`.
- Unlike Chromium, this campaign does not presume a second CPU tree or worker thread. The synchronous retained ceiling is tested first.
- Unlike Compose, invalidation truth remains explicit owner revisions and `composition::Changes`, not arbitrary runtime state-read capture.
- Unlike GTK, row fragments must carry semantic hit/access projections as well as pixels through the existing owners.
- Unlike egui, resident interactive controls can outlive one frame; bounded rebuilding alone is not enough.

---

## Territory claim and One-Way cleanup

PC-000 records exact ownership before editing. Expected semantic territory:

- `src/list.rs` and `src/table.rs` for keyed residency and provider boundaries;
- `src/composition/**` for semantic identity/change facts;
- `src/layout/**` for content-local geometry and retained layout inputs;
- `src/scene/**` for fragments, topology, residency, and properties;
- `src/runtime/presentation.rs` for orchestration, activation, and optional progress;
- `src/view/node/**` only where installed-view materialization remains causally involved;
- `src/render/**` only for counters or proving carried reuse; no new renderer entrance;
- `src/tests/**` and `tools/renderer_debug/**` for witnesses and native receipts;
- `docs/master_design.md` and prior campaign ledgers at closeout only.

The cleanup loop is:

```text
Select → Trace → Model → Challenge → Admit → Reduce
       → Rewire → Prove → Ratchet → Re-scan
```

Each `OW-PC-*` cell records:

1. question and source trace;
2. current and target owner graph;
3. admission or resistance;
4. displaced path and deletion point;
5. implementation;
6. One-Way gauge delta;
7. semantic/architecture/native proof;
8. fixed-point result.

Initial cells:

| Cell | Question | Expected disposition |
|---|---|---|
| `OW-PC-001` | Why does residency request a full rebuild? | replace with typed keyed delta or record irreducible global cause |
| `OW-PC-002` | Which viewport/property fields enter reusable row keys? | divorce all property-clock inputs while retaining legitimate local geometry |
| `OW-PC-003` | Where can row derivations live without minting authority? | private UI-internal retained stage with carried identity |
| `OW-PC-004` | Can scene fragments splice without renderer knowledge? | scene-owned immutable/reusable fragments; renderer still sees `Stack` |
| `OW-PC-005` | Which view/composition walks remain after keyed delta? | delete broad path or prove bounded container dependency |
| `OW-PC-006` | Does optional scheduling create reverse callbacks? | runtime owns progress; pure downward inputs/results |
| `OW-PC-007` | Do snapshots require provider API repair? | admit immutable values only if residual requires them |
| `OW-PC-008` | Does any old full-rebuild species remain? | delete and tombstone at PC-010 |

No new public root, crate, trait-object service locator, or broad `pub` exposure is admitted merely to make imports compile.

---

## Measurement and receipt law

### Receipt vocabulary

- **Ignition receipt:** territory, base, protected worktree.
- **Freeze/baseline receipt:** reproducible environment, workload, counter distributions, causal bracket.
- **Contract receipt:** clocks, ownership, keys, negative space.
- **Evidence receipt:** implementation plus deterministic witnesses.
- **Admission/rejection receipt:** whether a larger mechanism earned entry.
- **Burn-down receipt:** obsolete species absent and tombstones planted.
- **Closeout receipt:** full matrix, doctrine/census synchronization, exit theorem.

### Stage vocabulary

These facts must never be collapsed:

- requested;
- coalesced;
- selected;
- prepared;
- ready;
- activated;
- submitted;
- `present` called;
- successfully presented/visible.

Similarly, “reused” means the expensive owner phase did not execute. Recompute-and-compare is not reuse.

The campaign extends the existing integrated native renderer/scroll receipt. It does not mint a competing “CPU presentation receipt.” One record binds input, canonical scroll, semantic/residency/property currencies, CPU preparation, GPU realization, activation, submission, and successful `present` acknowledgement. It makes no scanout claim.

Every receipt records commit/checkpoint, workload/phase, OS and architecture, adapter/backend/driver, display/refresh/scale, surface format/present mode, build profile, warmup and sample counts, all owner serials, memory high-water marks, and an explicit terminal state for every candidate: activated, cancelled, superseded, stale-rejected, incompatible, or failed.

The admission clerk rejects a receipt with mismatched commits/environments/workloads/schemas, included warmup samples, missing terminal states, unexplained non-converging serials, incomplete process closure, or global percentiles substituted for required property/residency phase distributions. Timing comparison uses the median p95 of three matching native runs; literal-zero and semantic laws must pass every individual run.

### Required counters

PC-000 freezes exact names and types for at least:

| Family | Counters / timings |
|---|---|
| intent | scroll inputs, selected/coalesced/cancelled/stale candidates |
| residency/order | old/new interval, entering/departing/overlap/revised/moved rows, membership revision and `Change` events |
| provider/view | provider calls, row/cell constructs and binds, slots rebound/moved, view nodes cloned/visited |
| composition | nodes visited/reused/rebuilt and change species |
| layout | nodes visited/measured/placed/reused; row keys hit/missed |
| text | buffers created/reused, layouts, shapes, prepares |
| scene | frames scanned, bodies/fragments hit/missed/painted/spliced, primitives copied |
| presentation | preparation slices, max slice, ready/activated/discarded candidates |
| renderer | content resource hit/miss/upload, batch prep, passes, GPU draw |
| event loop | event-pass duration, time-to-input-service, redraw latency |
| receipts | submitted, present-called, visible success, deadline miss |
| memory | live slots/fragments/layouts/candidates/resources and high-water marks |

Each frame records one mutually exclusive primary `FrameNeed`/scheduling reason and an orthogonal set of property, residency, semantic, device, and diagnostic change species. A frame may legitimately carry both a residency reason and a property sample; the literal-zero gates consult the change-species counters rather than pretending the clocks are mutually exclusive.

### Workloads

Every causal boundary uses native Control Gallery workloads as well as deterministic owner-level fixtures:

1. 5 s warmup, then at least 30 s steady forward wheel/trackpad scroll;
2. stop/restart and rapid reversal;
3. scrollbar thumb jumps and alternating distant targets;
4. fixed-height and variable-height rows;
5. native row windows/runways of 20, 40, and 80 with identical one-row deltas;
6. fixed-residency/fixed-delta model sizes of 10,000, 100,000, and 1,000,000 rows to test model-size independence;
7. selection drag, copy, edit, caret, deletion, commit/cancel;
8. shift-range editing across multiple resident rows;
9. IME composition and accessibility inspection;
10. narrow, medium, and wide tables with 3, 12, and 48 columns; column resize; theme/font change; row mutation; insertion/removal/reorder;
11. nested scrolling; scale 1.0, 1.25, 1.5, 1.75, and 2.0; window resize/minimize/restore; device loss where supported;
12. large-text control under the same input and duration.

Record machine, OS, build profile, display scale, refresh rate, table dimensions, row/column counts, provider mode, input source, warmup, duration, sample count, and receipt path.

The deterministic complexity fixture separately crosses resident populations of 32, 64, 128, and 256 with column counts 3, 12, and 48 where the fixture can bypass native runway limits, using identical one-row and four-row deltas. It requires zero visits to unchanged row and cell bodies at every size; equal per-delta fragment rebuild, text-layout, paint, and primitive-preparation counts; entering-cell work equal to declared columns × entering rows plus revised cells; only separately counted height/order indexes may grow logarithmically; and no flat-vector/commit/primitive copy proportional to resident rows × columns.

A native run is invalid unless it includes at least 128 property frames, 32 completed residency advances, forward and reverse crossings, four stop/restart transitions, four disjoint thumb jumps, one complete selection/copy case, and one complete edit/draft/commit-or-cancel case.

### Architectural acceptance gates

These are literal:

| Gate | Required result |
|---|---|
| in-runway property scroll | zero materialize, reconcile, measure, text prepare, fragment paint/assembly, content upload |
| residency overlap | zero measure, text prepare, paint, fragment rebuild, GPU content upload for unchanged overlapping rows and cells |
| delta proportionality | increasing native runway 20→40→80, deterministic resident population 32→64→128→256, and column count 3→12→48 with constant `D_s` changes only declared entering/revised cell work and named bounded/logarithmic container/index work |
| model-size independence | 10k→100k→1m logical rows at fixed residency/`D_s` produces equal row/cell owner work |
| ordered model mutation | same-key move and insert/remove/replace are observed from the owner journal and pay only for `A_m` plus named index effects |
| property-key purity | no property-clock field in any reuse key spanning property ticks |
| one spatial truth | pixels/hit/selection/caret/IME/access use submitted topology/properties |
| one generation grammar | no parallel activation, visibility, or freshness clock |
| bounded lifetime | memory plateaus according to documented runway/recycle/cache bounds |

### Performance gates

The architectural zeros decide correctness. Time gates decide whether optional machinery is admitted.

For a recorded refresh interval `T`, define:

```text
B = min(1,000 µs, T / 4)
```

On the formulation 240 Hz display, `T ≈ 4,167 µs` and `B = 1,000 µs`.

| Currency | Final synchronous gate |
|---|---:|
| input-event pass p95 | ≤ `B` |
| input-to-redraw-request p95 | ≤ `B` |
| scroll-specific layout p95 | ≤ `B` |
| scroll-specific scene/commit assembly p95 | ≤ `B` |
| property or active-state renderer draw p95 | ≤ `T` |
| complete property redraw event-pass p95 | ≤ `T` |
| input-to-`present`-call p95 for active scroll | ≤ `2T` |
| property renderer deadline misses | 0 |
| residency renderer deadline misses | 0 |
| admitted UI preparation quantum | configured ≤ `B`; overshoot bounded to one measured row/chunk |
| admitted activation p95 | ≤ `B` with zero row/node traversal |
| renderer/property/large-text median-p95 regression | ≤ 10% from PC-000 unless explicitly accepted |

p99 and maximum are recorded and must show no repeated starvation plateau. PC-000 may tighten these gates but cannot loosen them without an explicit machine-calibrated receipt. If minimized delta-bounded residency preparation repeatedly exceeds `B`, pushes the event path beyond `T`, or delays active-scroll submission beyond `2T`, that is admission evidence for PC-007; it is not permission to weaken the gate.

The protected renderer rail is frozen at PC-000. Until a stricter replacement receipt is accepted, the isolated table workload remains at most four text prepares, four glyph batches, ten draw passes, sixteen clip batches, and fifty-six draw calls; the post-crossing property tick performs zero content upload/resource churn/render-plan rebuild; and exact output passes all five scales.

Release owner-work fixtures use 64 warmups and 1,024 measured samples. Release GPU equivalence uses all five scales and five repetitions with exact-first comparison. Native evidence uses three matching 30-second sessions after a five-second warmup and explicit process closure.

---

## Checkpoint board

Only one checkpoint may be `IN PROGRESS`.

| Checkpoint | State | Required outcome |
|---|---|---|
| PC-000 — Claim territory and freeze the CPU bill | SEALED | protected base, census, workloads, counters, baseline, thresholds |
| PC-001 — Ratify clocks, keys, and compiler charter | SEALED | constitutional contract and ratchets before shape |
| PC-002 — Divorce scroll from row geometry | SEALED | content-local rows plus parent property transform |
| PC-003 — Retain row presentation fragments | SEALED | unchanged overlap retains layout/text/scene/semantic projections |
| PC-004 — Re-receipt the concrete fault | SEALED | mandatory measurement boundary after clock/fragment repair |
| PC-005 — Make residency a keyed delta | FAILED BY RECEIPT | row-owner delta is bounded, but flat commit/topology/residency/plan work remains `O(R)` |
| PC-006 — Prove Qt class and decide the ceiling | BLOCKED BY PC-005 | final synchronous receipt cannot legalize resident-proportional work |
| PC-007 — Optional upstream active/pending generation | PROHIBITED | workers/generations may not hide an incomplete PC-005 |
| PC-008 — Optional immutable provider snapshots | PROHIBITED | no worker boundary was admitted |
| PC-009 — Optional worker compilation | PROHIBITED | total CPU remains `O(R)` |
| PC-010 — Burn down the old species | NOT OPENED | requires a successful PC-005/PC-006 architecture ceiling |
| PC-011 — Close out and teach master design | NOT OPENED | campaign stopped at its mandatory architectural gate |

PC-004 is the explicit gate demanded by the countersign. PC-006 is a second gate: it decides the architecture ceiling after keyed delta is complete. Rejection of PC-007 through PC-009 is a successful result.

---

## PC-000 — Claim territory and freeze the CPU bill

**Owned question:** What exact work, on what exact base and machine, produces the table-scroll stall?

**Allowed scope:** diagnostics, receipt tooling, deterministic tests, documentation, and the minimum instrumentation required to attribute owner phases.

**Forbidden scope:** cache repairs, new retention, scheduling, workers, provider redesign, or performance edits mixed into baseline instrumentation.

**Required work:**

1. record HEAD, branch, full status, inherited ownership, and authorized write set;
2. reconcile this census against live source without absorbing earlier dirty changes;
3. pin counter names, stage semantics, overflow behavior, and diagnostic overhead;
4. make diagnostic-off behavior observationally and architecturally neutral;
5. run every standard workload and the large-text control;
6. record distributions, not one screenshots or hand-selected fast runs;
7. attribute the longest event passes to materialization, composition, layout, text, scene, render, or platform;
8. run negative controls that deliberately trigger semantic, residency, property, and device causes.

**Positive witnesses:**

- repeated runs reproduce the same owner ranking;
- every frame has exactly one primary `FrameNeed` plus an orthogonal change-species set, and mixed residency+property cases preserve both facts;
- existing GPU receipt values remain within expected noise;
- the large-text control separates table-specific resident-tree work.

**Negative controls:**

- intentionally misclassify the primary `FrameNeed` or erase a concurrent property species from a residency frame and ensure the oracle fails;
- disable one phase timer and ensure receipt completeness fails;
- inject a fake property-only row rebuild counter and ensure literal-zero gates fail.

**Exit receipt:**

- ignition and baseline ledgers complete;
- receipt files and load-bearing facts recorded;
- exact thresholds ratified;
- no production repair has begun.

**Evidence ledger:**

### PC-000 ignition receipt — 2026-07-17

- branch: `master`;
- HEAD: `1fe6af199b501163e1f5ccf0d8065e9b079a43ea`;
- execution authority: the active goal names this campaign file; no commit, branch, stage, push, or unrelated cleanup authority is implied;
- writable campaign territory: the files named by each checkpoint plus this ledger and source census;
- protected inherited territory: the complete SE-009 dirty set listed in the opening of this ledger;
- worktree reconciliation: `git status` contains that protected set plus the two untracked Presentation Compiler documents and no unexplained path;
- inherited change size at ignition: 26 tracked files, 1,793 insertions, 232 deletions; these deltas remain SE-009 input and are not attributed to PC-000;
- deterministic inherited oracle: `cargo test -p renderer_debug` passed on 2026-07-17 with 3 passed, 0 failed, and 27 GPU-dependent tests intentionally ignored;
- baseline discipline: the historical Control Gallery receipt remains causal formulation evidence only. Fresh timing evidence is not admitted until the PC-000 schema can reject missing phases, missing terminal states, and erased mixed change species.

No performance or architecture repair had begun at this boundary.

### PC-000 counter and receipt-schema receipt — 2026-07-17

The integrated renderer receipt now carries
`wgpu_l3.presentation_compiler.v1`. Its cumulative counters use saturating
`usize`/`u64` currencies, its phase timings are microsecond `u128` samples in
bounded 128-entry queues, and its resident/live values are explicit gauges
rather than inferred from cumulative totals. Production owners publish facts;
no layout, scene, list, composition, renderer, or scheduling decision reads a
diagnostic counter.

The schema records:

- exactly one primary reason per prepared frame: idle, property, residency,
  paint, layout, or rebuild;
- orthogonal property, residency, semantic, device, and diagnostic species,
  including a mixed property+residency counter;
- keyed old/new resident intervals, entering/departing/overlap/revised/moved
  rows, membership revision/events, provider binds, slot rebinds, cloned view
  nodes, and reused text buffers;
- composition change species and reconciliation timing;
- layout candidate visits/reuse, scene frame scans/paints/reuse, and total CPU
  candidate time;
- receipt completeness tied to the integrated frame-prepared count and bounded
  phase sample counts.

Negative controls prove that erasing the residency species from a residency
frame or forging a second primary reason makes
`presentation_receipt_complete=false`. The keyed-list witness separately
proves that a same-key move reports four overlaps/four moves, zero
enter/depart, and one owner-journal event without rebinding.

Verification at this boundary:

- `cargo check --all-targets --features renderer-debug` — pass;
- `cargo test diagnostics::presentation` — 3 passed;
- `cargo test keyed_slots_reuse_moves_rebind_revisions_and_teardown_exactly` — pass;
- `cargo test renderer_receipt_includes_upstream_scene_scroll_and_text_work` — pass;
- pre-instrumentation `cargo test -p renderer_debug` — 3 passed, 27 GPU-only
  witnesses intentionally ignored.

### PC-000 isolated residency baseline — 2026-07-17

Environment: Windows x86_64, NVIDIA GeForce RTX 4070 Ti SUPER, DX12, discrete
GPU, driver `32.0.15.9636`, scale 1.25, release profile. Each owner fixture
constructs an initial resident candidate, resets the integrated diagnostics,
then measures one forward guard crossing and the following property-only tick.
Every receipt was complete and the post-crossing property tick performed zero
node realization, primitive/text preparation, shaping, content upload, GPU
resource churn, or render-plan rebuild.

| Payload | Entering rows | Provider calls | Candidate CPU | Layout | Scene assembly | Scene scanned / painted / reused | GPU node realizations |
|---|---:|---:|---:|---:|---:|---:|---:|
| table, run 1 | 12 | 36 cells | 2,944 us | 994 us | 1,037 us | 89 / 82 / 7 | 81 |
| table, run 2 | 12 | 36 cells | 3,093 us | 1,079 us | 1,076 us | 89 / 82 / 7 | 81 |
| table, run 3 | 12 | 36 cells | 3,041 us | 1,087 us | 1,020 us | 89 / 82 / 7 | 81 |
| virtual list control | 12 | 12 rows | 572 us | 133 us | 197 us | 22 / 21 / 1 | 21 |
| large-text control | 0 | 0 | 2,246 us | 1,375 us | 152 us | 2 / 1 / 1 | 1 |

The matching table median is 3,041 us input-to-candidate, 2,932 us recorded
CPU candidate time, 1,079 us layout, and 1,037 us scene assembly. With the
machine's 240 Hz interval (`T ≈ 4,167 us`, `B = 1,000 us`), both the layout and
scene owner slices miss `B`, while the complete table candidate consumes about
70% of `T` before native event-loop and real-present costs. The identical
12-row list control completes in 505 us recorded CPU time. The table therefore
pays roughly 5.8× the row-only presentation bill and scans/paints approximately
four times as many scene frames. The large-text control is layout-heavy but
does not reproduce the table's 82-frame paint storm.

This receipt verifies the concrete causal bracket without yet closing PC-000:
the required real-window Control Gallery distribution and interaction matrix
remain outstanding. The interrupted Windows automation attempt performed no
gallery input and contributes no evidence.

### PC-000 native Control Gallery freeze receipt — 2026-07-17

The release Control Gallery ran in a real Windows window on the same Windows
x86_64 / NVIDIA GeForce RTX 4070 Ti SUPER / DX12 / driver `32.0.15.9636` /
1.25-scale environment as the owner fixtures. After the first automation
attempt was interrupted by a machine stop, the process was reopened and the
operator exercised steady wheel scrolling, forward/reverse crossings,
stop/restart, deliberate settled runway advances, drag scrolling, disjoint
scrollbar motion, selection/copy, and editable-cell commit-or-cancel behavior.
The process was then closed and a read-only process census found no remaining
`control_gallery.exe` instance.

The admitted cumulative receipt is
`target/release/examples/renderer-receipts/control-gallery-500px-idle-1784331081993.txt`.
The filename retains the gallery's stale default operator label (`idle`); the
ledger does not rewrite it. Actual input/change species are established by the
receipt's direct counters and the witnessed interaction sequence. Future
comparison runs must set a truthful workload label before warmup, and the
admission clerk may not pair this receipt with differently labelled runs.

Load-bearing native values:

| Fact | Value |
|---|---:|
| attempted / present-submitted / skipped frames | 634 / 634 / 0 |
| property / semantic frames | 374 / 260 |
| virtual guard crossings / replenishment commits | 40 / 40 |
| accepted residency advances / property ticks | 32 / 32 |
| wheel/scroll inputs | 2,972 |
| coalesced residency candidates | 2,958 |
| candidate / attempted / GPU-submitted / present-submitted property serial | 443 / 443 / 443 / 443 |
| presentation receipt | `wgpu_l3.presentation_compiler.v1`, complete |
| provider binds / cloned view nodes | 1,238 / 3,279 |
| entering / departing / overlapping rows | 1,237 / 1,165 / 2,042 |
| layout and scene frames scanned | 254,413 / 254,413 |
| scene frames painted / reused | 14,739 / 239,674 |
| presentation total p95 | 70,375 us |
| layout p95 | 60,752 us |
| scene assembly p95 | 22,529 us |
| native event pass p95 | 46,439 us |
| property renderer draw p95 | 2,323 us |
| property renderer deadline misses | 0 |

The run clears the literal native coverage floor of 128 property frames and 32
completed residency advances. It also exposes the CPU indictment more strongly
than the formulation receipt: the current compiler visits every one of 254,413
layout frames again at scene assembly, while 94.2% of those scene-frame visits
end in cache reuse. The direct event input path stays below `B` at 558 us p95,
but layout, scene assembly, and the complete native event pass miss their
campaign budgets by large margins. GPU property draw remains below `T`, and
all property serials converge with zero skipped or property-deadline-missed
frames. The longest bill is therefore upstream presentation work, not the
retained renderer.

The deterministic admission clerk now validates the integrated presentation
schema, exact primary-reason exhaustiveness, concurrent property/residency
species, frame/sample completeness, serial convergence, and required summary
fields. Its eight-test suite includes negative controls for a missing phase,
erased residency species, and a forged second primary reason. Verification at
seal:

- `cargo fmt --all -- --check` — pass;
- `cargo check --all-targets --features renderer-debug` — pass;
- `cargo test diagnostics::presentation` — 3 passed;
- `cargo test keyed_slots_reuse_moves_rebind_revisions_and_teardown_exactly` — pass;
- `tools/test_renderer_receipts.py` using the bundled Python runtime — 8 passed.

PC-000 is sealed. Its thresholds remain `T = 4,167 us` and
`B = min(1,000 us, T / 4) = 1,000 us`. No presentation repair entered the
baseline checkpoint.

---

## PC-001 — Ratify clocks, keys, and the compiler charter

**Owned question:** Which owner and mutation clock author every field that can affect row presentation?

**Allowed scope:** contract types/tests/docs, cache-key census, architecture ratchets, and a private shape sketch.

**Forbidden scope:** blessing `PresentationTree` storage, threading, adding another revision, or changing pixels before the contract is testable.

**Required work:**

1. enumerate every row/cell identity, revision, constraint, geometry, clip, viewport, residency, property, presentation, and GPU key field;
2. classify each by sole owner and clock;
3. ratify the tombstone law and local-geometry qualification;
4. define renderer-independent row-fragment inputs/outputs and bounds;
5. define cleanup, failure, activation, and stale-result behavior;
6. ratify the Presentation Compiler charter and one-way graph;
7. plant architecture tests for forbidden clock marriages.

**Required ratchets:**

- `row_presentation_keys_are_content_local_and_property_free`;
- `presentation_tree_borrows_composition_identity_and_changes`;
- `presentation_tree_is_private_and_renderer_invisible`.

**Negative controls:**

- add resolved offset to a semantic/fragment key and prove a test fails;
- mint a `PresentationNodeId` or externally mutable dirty API and prove the architecture gauge rejects it;
- expose compiler state to render/platform and prove the handoff guard fails.

**Exit receipt:** every field has one owner, every key has an admission decision, and the proposed first repair requires no new clock.

**Evidence ledger:**

### PC-001 owner/clock and key-contract receipt — 2026-07-17

The live-source census ratifies one owner and one mutation clock for every
row-presentation currency:

| Currency | Sole owner / clock | Key ruling |
|---|---|---|
| logical row key, item revision, membership revision and ordered change journal | list/table model semantic commit | admitted as borrowed semantic facts |
| provider record and cell projection | UI-authority provider bind for the item revision | value may enter content; callback/capability identity may not |
| active/recycled slot | list materialization lifetime | bookkeeping only; never logical cell or presentation identity |
| `NodeId`, `ContentRevision`, root `Changes` | composition reconciliation | admitted; no second presentation identity or dirty API |
| row order/index and fixed/variable height facts | model plus layout's local geometry commit | admitted only as declared local geometry inputs |
| column widths, font/theme/style and local constraints | layout/text semantic commit | admitted with the owning revision/constraint value |
| local row/cell rect, active rect, local clip and hit/access bounds | layout commit in scroll-content coordinates | admitted after PC-002 proves the coordinate qualification |
| viewport rect, visible coverage, content extent and maximum | layout geometry commit | admitted only through a property-free geometry key |
| desired/requested/resolved scroll and `scene::Properties` sample | canonical `AxisAdjustment` property clock and submitted property receipt | forbidden from semantic, row, cell, fragment, text, and content keys |
| requested/candidate/accepted resident interval | residency candidate/activation receipt | coverage only; forbidden from unchanged content identity |
| active/pending/activated/submitted/present-success generation | existing runtime presentation grammar | freshness/visibility only; no parallel CPU epoch |
| GPU resource, batch, target and allocation identity | retained renderer realization | renderer-private and forbidden upstream |

The tombstone is literal: no property-clock value may participate in a cache
key that claims validity across property ticks. Geometry may participate only
when its coordinate space and authoring revision are explicit. Recompute and
compare is not reuse.

The first private contract shape is
`layout::frame::ContentLocalRowSceneKey`. It borrows composition identity and
content revision, local layout geometry/clip, semantic presentation state, and
logical provided-row/table facts. It deliberately has no `Viewport`,
`interaction::Offset`, resolved scroll, residency clock, presentation epoch,
or renderer identity. It is a contract sketch, not a new storage owner or
public architecture root.

The renderer-independent row fragment contract is likewise fixed before
storage: composition identity/revision; logical row/cell identity; declared
local constraints and geometry revision; text-owner artifacts; scene-owned
primitive bodies and local clip/effect topology; local hit/access projection;
dependency set; and a runway/recycle-bounded lifetime. Its output remains an
ordinary scene fragment entering the existing `scene::Stack` handoff. It owns
no model truth, property value, residency decision, activation, receipt, or GPU
allocation. Stale/missing/incompatible derivations are discarded before
activation; provider failure leaves the active presentation untouched; row
departure retires the fragment and all hit/focus/access projections through
the existing composition change stream.

Three executable architecture ratchets now enforce the contract:

- `row_presentation_keys_are_content_local_and_property_free`;
- `presentation_tree_borrows_composition_identity_and_changes`;
- `presentation_tree_is_private_and_renderer_invisible`.

Negative control strings cover `Viewport`/offset/residency fields in the row
key, `PresentationNodeId`/`CpuPresentationEpoch`/external dirty APIs, and any
`PresentationTree` or compiler/provider handoff into render/platform. Each
ratchet passes individually. PC-001 is sealed: the first repair needs no new
clock, identity, activation path, queue, public module, or renderer handoff.

---

## PC-002 — Divorce scroll from row geometry

**Owned question:** Can the same prepared row geometry remain valid across property scroll ticks?

**Allowed scope:** virtual-row coordinate systems, fixed coverage clips, spatial declarations/properties, submitted projection, and focused architecture/semantic tests.

**Forbidden scope:** simply deleting all geometry from keys, renderer-side offset reconstruction, visual-only transforms, row-fragment retention beyond what is needed to prove the divorce, or workers.

**Required shape:**

```text
row local y = logical content position
viewport coverage = fixed target-local clip
scroll = parent spatial/property translation
presented projection = shared pixels + input + access evaluator
```

**Positive witnesses:**

- two candidates differing only by scroll property have identical overlapping-row local geometry and fragment-key inputs;
- downstream of the canonical `AxisAdjustment` update, property scroll samples `scene::Properties` and submitted spatial output without semantic/compiler work;
- fixed viewport clips remain fixed while row content moves;
- pixels, hit testing, selection, caret, IME, and accessibility remain aligned at scale and nested scroll;
- variable-height and table-chrome cases declare their actual local dependencies.

**Negative controls:**

- reintroduce offset subtraction into row rectangles and ensure key-purity/behavioral tests fail;
- apply translation in renderer only and ensure semantic projection tests fail;
- remove a legitimate local geometry revision and ensure resize/column tests fail.

**Cleanup:** delete the old viewport-contaminated row-placement species at the same boundary. Do not leave dual absolute/local paths.

**Exit receipt:** `property_scroll_bypasses_the_presentation_compiler` is true for geometry/key work, with exact semantic equivalence.

**Evidence ledger:**

### PC-002 content-local geometry and split-clock receipt — 2026-07-17

Fixed- and variable-height virtual rows now author `y` as viewport origin plus
logical content position. Neither path subtracts the requested/resolved scroll
property. The row subtree no longer inherits viewport coverage as a layout
clip; fixed coverage is projected once as an outer scene/spatial clip. This
keeps nested text/control viewport keys stable even while an overlapping row
moves offscreen inside the resident runway.

`ScrollDeclaration` now distinguishes:

- `property_origin`, used only by spatial projection; content-local virtual
  rows use zero so the complete submitted property translates them;
- `baseline`, the current resident offset used only for coverage admission and
  property fallback.

Baseline-relative ordinary/text/table-horizontal scrolls retain the prior
constructor and semantics. Content-local virtual scrolls use
`new_content_local`; no second property owner or receipt was introduced.
`ViewportSceneKey` contains only rect, visible coverage, content extent, and
maximum. Requested/resolved offsets are absent.

The exact overlap witness constructs two table residency candidates with a
different scroll baseline and proves every shared table-cell `NodeId` retains
identical local rect and identical scene key. Negative architecture ratchets
ban offset subtraction in both virtual-row algorithms, property fields in the
viewport key, and renewed conflation of spatial origin with resident baseline.
The property-only witness proves resident scroll performs no provider bind,
semantic commit, layout recomposition, text work, primitive preparation,
content upload, resource churn, or render-plan rebuild.

GPU and semantic verification:

- `cargo test residency` — 59 passed;
- `property_scroll_bypasses_the_presentation_compiler` — pass;
- `residency_candidates_preserve_overlapping_table_cell_local_geometry_and_scene_keys` — pass;
- the three new PC-002 architecture ratchets — pass;
- all 27 GPU-only `renderer_debug` oracles — pass, including exact pixels at
  all five scales, nested/group/table property movement, slow-scroll coverage,
  horizontal entering pixels, pending activation, and negative controls;
- `cargo check --all-targets --features renderer-debug` — pass.

Release 1.25x table crossing on the PC-000 machine:

| Currency | PC-000 | PC-002 | Delta |
|---|---:|---:|---:|
| entering rows / provider cell calls | 12 / 36 | 12 / 36 | unchanged cold delta |
| candidate CPU | 3,041 us median | 3,209 us single causal run | timing not yet admitted |
| scene frames scanned | 89 | 89 | unchanged residual scan |
| scene frames painted / reused | 82 / 7 | 48 / 41 | 34 misses removed |
| primitive preparations | 81 | 20 | 75.3% removed |
| text prepares | up to 4 protected rail | 1 | within rail |
| post-crossing cold/content work | 0 | 0 | exact |

The causal verdict is confirmed: viewport-contaminated row geometry and keys
were responsible for a large part of the paint/primitive storm. PC-002 is
sealed. It intentionally does not claim the final CPU win: layout still visits
178 frames across the crossing/property pair, scene scans all 89 candidate
frames, and cache hits still clone bodies and build fresh fragment vectors.
Those residuals are owned by PC-003.

---

## PC-003 — Retain row presentation fragments

**Owned question:** What is the smallest renderer-independent lifetime that lets unchanged rows skip layout, text, paint, and scene assembly?

**Allowed scope:** private retained derivation, row/cell fragment contracts, structural sharing/splicing, bounded caches, hit/access projections, and carried owner revisions.

**Forbidden scope:** per-row GPU surfaces/layers, renderer-owned semantic fragments, a second model, broad reactive dependency capture, unbounded caching, active/pending CPU generations, or worker APIs.

**Fragment contract must include or reference:**

- logical identity and owner revisions;
- local constraints/geometry revision;
- text layout/preparation artifacts as owned by current text boundary;
- scene-owned primitives and local clip/effect topology;
- local hit/access geometry;
- declared dependencies and cleanup policy;
- no current scroll offset, residency revision, presentation epoch, or renderer allocation identity.

**Positive witnesses:**

- unchanged row and cell overlap executes zero measurement, text preparation, paint, and fragment construction;
- selection/edit changes rebuild only declared affected fragments;
- a row revision invalidates its own dependent artifacts;
- style/column/global causes intentionally invalidate the declared set;
- cache memory plateaus under long forward/reverse scroll;
- renderer receives the same `scene::Stack` contract and reuses carried GPU resources.

**Negative controls:**

- mutate content without revision and ensure semantic oracle detects stale output;
- add property offset to a fragment key and ensure the clock ratchet fails;
- retain beyond bound and ensure the memory witness fails;
- attempt renderer access to the private tree and ensure architecture tests fail.

**Cleanup:** remove superseded ephemeral row-paint caches and duplicate hit/access projections. One owner per artifact.

**Exit receipt:** stable row identity now produces stable prepared presentation, not merely stable widget state.

**Evidence ledger:**

### PC-003 retained row-fragment receipt — 2026-07-17

The private presentation lifetime now lives across the existing owners rather
than beside them. `layout::VirtualRowFragment` retains each virtual row's flat
frame derivation behind shared storage and borrows `composition::Changes` plus
the existing node/content revisions for invalidation. The scene owner retains
an immutable `Arc<[Fragment]>` per row/layer/panel and borrows that layout
storage as its lifetime token. The row key contains only root identity, layer,
and panel identity. It contains no offset, viewport value, residency revision,
property serial, presentation epoch, or renderer allocation identity. Theme
changes and departure clear/retire the bounded scene entries; a broad
presentation still discards the prior layout candidate.

The renderer API did not change: `scene::Stack` remains the only retained GPU
entrance. The new presentation stage is crate-private, and architecture tests
reject renderer knowledge of `RowFragmentKey` or `CachedRowFragment`.

The deterministic table crossing witness proves both lifetimes at once:

- overlapping candidate rows share the exact `Arc<[Frame]>` storage of the
  prior layout;
- the candidate reports reused frames and constructs fewer frames than its
  total flat population;
- 8 overlapping row scene fragments are spliced and 12 entering row fragments
  are built;
- scene paint is confined to 48 entering nodes while 41 unchanged nodes reuse
  their bodies;
- the 8 overlapping rows retain identical content-local cell rectangles and
  scene keys.

Release owner receipt on the frozen Windows/RTX 4070 Ti SUPER/DX12/scale-1.25
fixture:

| Counter | PC-000 | PC-002 | PC-003 |
|---|---:|---:|---:|
| entering / overlap rows | 12 / 8 | 12 / 8 | 12 / 8 |
| layout nodes constructed / reused | not separated | not separated | 57 / 121 across candidate + property tick |
| scene frames scanned | 89 | 89 | 89 |
| scene frames painted / body-reused | 82 / 7 | 48 / 41 | 48 / 41 |
| row fragments spliced / built | not measured | not measured | 8 / 12 |
| primitive prepares | 81 | 20 | 20 |
| text prepares | 1 | 1 | 1 |
| candidate CPU | 2,944–3,093 us | 3,209 us | 3,221 us |
| layout p95 | 994–1,079 us | 1,141 us | 1,153 us |
| scene assembly p95 | 1,037–1,076 us | 1,073 us | 1,146 us |

The stable-artifact law is therefore repaired, but this receipt deliberately
does **not** claim the latency win. The flat layout vector is still rebased and
the scene/commit builder still scans and appends all 89 resident frames.
PC-004 opens with that honest `O(R)` residual; PC-005 remains barred until the
mandatory measurement gate classifies it.

Verification at this boundary:

- `cargo check --all-targets --features renderer-debug` — pass;
- exact retained-overlap witness — pass;
- `cargo test residency --features renderer-debug` — 59 passed;
- retained-tree privacy/property-key architecture witness — pass;
- presentation diagnostics — 3 passed;
- receipt-clerk suite — 8 passed.

---

## PC-004 — Re-receipt the concrete fault

**Owned question:** After content-local geometry and retained row fragments, what work remains and who owns it?

This checkpoint is a hard measurement boundary. PC-005 may not begin until it closes.

**Allowed scope:** measurement, attribution, targeted counter correction, and cleanup needed to make receipts truthful.

**Forbidden scope:** keyed-delta implementation, scheduling, provider redesign, active/pending CPU state, or workers hidden inside “instrumentation.”

**Required comparisons:**

- the exact PC-000 workloads and machine;
- pre/post distributions for view, composition, layout, text, scene, renderer, and event loop;
- in-runway property frames versus residency frames;
- resident windows 20/40/80 and columns 3/12/48 at constant `D_s`;
- table versus large-text control;
- forward, reversal, stop/restart, and thumb jumps.

**Required verdict:**

1. confirm or reject that the contaminated key caused the 163–170 repainted-node / 81–86 primitive behavior;
2. identify any remaining `O(R)` walk;
3. classify each residual as materialization, composition, layout, scene splice, presentation orchestration, GPU, or platform;
4. amend PC-005 scope only from evidence;
5. explicitly prohibit unmeasured worker escalation.

**Exit receipt:** a causal re-receipt with exact counter deltas and an owner-ranked residual. “Faster” is insufficient.

**Evidence ledger:**

### PC-004 causal owner re-receipt — 2026-07-17

The mandatory measurement boundary confirms the concrete diagnosis and rejects
early scheduling/worker escalation.

On the frozen release table/scale-1.25 crossing, the three matching PC-003
runs recorded candidate CPU values of 3,221, 3,395, and 3,492 us (median
3,395 us). Their presentation p95 values were 3,074, 3,254, and 3,348 us
(median 3,254 us); layout p95 was 1,153, 1,249, and 1,321 us (median
1,249 us); scene/commit assembly p95 was 1,146, 1,138, and 1,173 us (median
1,146 us). The later counter-correction run was consistent at 3,423 us total,
1,205 us layout, and 1,218 us scene assembly.

The repaired-key verdict is **CONFIRMED**:

| Causal counter | PC-000 | After PC-002/PC-003 | Verdict |
|---|---:|---:|---|
| scene nodes painted | 82 | 48 | the 34-node overlap paint storm is gone |
| scene nodes body-reused | 7 | 41 | stable keys now retain unchanged bodies |
| primitive prepares | 81 | 20 | GPU content work is confined to entering content |
| row fragments spliced / built | unmeasured | 8 / 12 | overlap/entering boundary is explicit |
| post-crossing content upload | 0 | 0 | property rail remains clean |

The remaining CPU bill is not GPU batching and not stale row identity. The
corrected counters rank the residual owners:

| Owner | One 12-enter / 8-overlap crossing | Classification |
|---|---:|---|
| materialization | 20 resident view nodes cloned; 12 provider binds | provider work is `O(D_s)`, installed-view projection is `O(R)` |
| composition | 89 nodes visited and reconstructed; 41 identities reused; 48 added | semantic identity survives, but reconciliation still rebuilds the flat retained tree |
| layout | 57 frames constructed and 32 overlap frames rebased/copied for the residency candidate | entering work plus an `O(R)` flat-frame reconstruction |
| scene/commit | 89 frames scanned; 8 row fragments spliced; 12 built | immutable bodies survive, but commit assembly still walks/appends the resident scene |
| renderer | 48 entering nodes realized; 20 primitive prepares; zero overlap content upload | delta-bounded and no longer the primary owner |

The resident-population cross-check holds `D_s` to the runway transition while
scale changes the resident population:

| Scale | Resident / entering / overlap rows | Scene scans | Painted / reused | Fragment splice / build | Layout p95 | Scene p95 |
|---:|---:|---:|---:|---:|---:|---:|
| 2.0 | 13 / 8 / 5 | 61 | 32 / 29 | 5 / 8 | 940 us | 812 us |
| 1.25 | 20 / 12 / 8 | 89 | 48 / 41 | 8 / 12 | 1,205–1,321 us | 1,138–1,218 us |
| 1.0 | 23 / 14 / 9 | 101 | 56 / 45 | 9 / 14 | 1,551 us | 1,317 us |

Scene scans follow exactly `4 × resident rows + 9` for this three-column table;
paint/build follows only entering rows. The virtual-list control at 20 rows
likewise scanned 22 frames while splicing 8 and building 12. The large-text
control had no row work (2 layout/scene nodes, zero paint) and placed its
2,569-us presentation p95 chiefly in the independent large-text layout owner
(1,653 us), so it does not explain the table's resident-tree slope. A disjoint
table jump correctly had no overlap to retain: 25 entering rows, 109 scans,
100 paints, and 25 fragment builds.

PC-005 scope is therefore amended by evidence: it must carry the typed keyed
delta through materialized view order, composition, layout fragment order, and
scene/commit order. Repairing only provider binding would be insufficient.
Scanning overlap to validate reuse or flattening shared fragments back into a
new resident vector is explicitly counted as old-species work. PC-007–PC-009
remain prohibited because the residual is still `O(R)`; moving it to slices or
workers would preserve the wrong complexity class.

### PC-004 native causal receipt — 2026-07-17

The post-repair native receipt is
`target/release/examples/renderer-receipts/control-gallery-500px-idle-1784337144796.txt`.
The filename retains the Control Gallery's stale default `idle` label, so it is
not paired with a differently labelled rail. Its integrated common-schema
clerk passes on the same Windows/RTX 4070 Ti SUPER/DX12/240-Hz environment.
The Windows controller produced ordinary table wheel input only and refreshed
the target window after every step; the gallery itself wrote the receipt.

The quantitative causal population is complete:

- 327 attempted / 327 present-submitted / 0 skipped frames;
- 146 property frames and 181 semantic frames;
- 190 scroll inputs;
- 32 virtual guard crossings and exactly 32 replenishment commits;
- candidate, attempted, GPU-submitted, and present-submitted property serials
  all converged at 273;
- 0 property renderer deadline misses and property draw p95 of 2,834 us;
- 1,266 entering, 1,195 departing, and 3,849 overlapping rows;
- 1,266 provider binds versus 5,115 cloned resident view nodes;
- 81,430 layout constructions/visits and 40,001 reported frame reuses;
- 115,016 scene scans, 8,887 paints, and 106,129 body reuses;
- 10,724 row-fragment splices and 3,731 row-fragment builds.

This is the native version of the deterministic verdict: unchanged row bodies
and GPU content survive, but view/composition/layout/commit owners continue to
walk resident overlap. Presentation p95 was 61,563 us, layout p95 34,970 us,
scene assembly p95 17,492 us, and native event-pass p95 81,528 us. These global
native distributions include large residency jumps and diagnostic receipt
work, so they are not substituted for the phase-isolated owner medians; they
are a starvation receipt and make the need for PC-005 stronger.

The clerk itself exposed and received one truthful-counter correction: timing
queues retain at most 128 samples, so long receipts now require
`draw_us_sample_count == min(present_submitted, 128)` rather than the impossible
unbounded equality. A new long-receipt negative control brings the clerk suite
to 9 passing tests.

PC-004 is sealed. Its admission verdict is **PC-005 REQUIRED; PC-007–PC-009
PROHIBITED** until the named `O(R)` paths are removed.

---

## PC-005 — Make residency a keyed delta

**Owned question:** Can a pure scroll residency change preserve every overlap and pay only for `D_s`, while model insert/remove/replace/move pays only for its declared `A_m`?

**Allowed scope:** typed residency/order deltas, the existing `list::Change` journal and any required table-provider API repair, slot binding/retirement/move, targeted composition changes, retained-derivation splices, container indexes, bounded runway/recycle policy, and cancellation before work begins.

**Forbidden scope:** rebuilding the installed view or resident subtree as a convenience, scanning all resident rows to prove they are unchanged, changing property identity, worker threads, or a second activation grammar.

**Required delta:**

```text
old ordered keys + desired interval + membership revision / ordered Change journal + item revisions
  → entering keys
  → departing keys
  → retained overlap
  → revised overlap
  → moved/order-changed keys and declared position/height-prefix effects
  → bounded order/coverage/index edits
```

**Positive witnesses:**

- every overlapping row retains composition identity, layout/text/fragment artifacts, semantic projections, and GPU content;
- provider and widget work remains bounded to entering/revised rows;
- for fixed `K`, row binds equal entering + revised rows and cell binds equal `K × entering rows + revised cells`, with zero overlap binds;
- a same-key `Move` is observed even when entering/departing sets are empty, and only its declared `A_m` is re-ordered/re-derived;
- composition visits only changed roots plus bounded ancestry;
- scene splices changed fragments without rebuilding unchanged bodies;
- work counters remain flat when resident window doubles at constant `D_s`;
- reversal and thumb-jump candidates retire or reuse cleanly;
- focus/edit state follows logical identity, not slot address.

**Negative controls:**

- reorder keys with equal row count and ensure identity/order tests catch incorrect index reuse;
- revise one overlapping row and ensure exactly its declared dependencies rebuild;
- deliberately scan all overlap and ensure proportionality counters fail;
- activate an obsolete candidate and ensure freshness tests fail.

**Cleanup:** delete `FrameNeed::Residency → Invalidation::Rebuild` and every broad downstream path it displaced, unless a narrowly named non-table client still owns it.

**Exit receipt:** `residency_delta_preserves_every_overlap`, `O(D_s)+O(C)` for pure scroll, and `O(A_m)` plus named index work for model order mutation are demonstrated by counters and semantic oracles.

**Evidence ledger (terminal failed-gate receipt):**

The first keyed-delta cut is implemented and remains inside PC-005. `list::Slots`
now retains ordered keys and the installed interval, and a residency-only
materialization emits front/back removals and insertions without representing
the retained middle. `view::Node` and `composition::tree::Node` use end-editable
child sequences; `present_residency` mutates the installed view and reconciles
only the named virtual-list root plus entering subtrees. It no longer clones the
installed `View`, rebuilds its resident `Vec<Node>`, or calls broad composition
reconciliation as a convenience.

The provider repair is capability-gated. `list::Model::residency_revision` and
`table::Provider::residency_revision` are explicit snapshot contracts. `None`
keeps the full correctness fallback; `Some(revision)` admits the keyed path only
when the same value proves key order and every bind-visible item value are
unchanged. `table::Source::residency_revision` makes that proof explicit for
closure-backed sources, while immutable `Source::records` snapshots derive it
from their generation. Control Gallery and the closed diagnostic table declare
their immutable snapshot generation.

Two literal proportionality tests now ratchet the owner boundary:

- `residency_delta_preserves_every_overlap_without_querying_it` advances one
  row with 16 and 64 residents and observes exactly one key query, one inverse
  query, one item-revision query, and one provider bind in both populations;
- `keyed_residency_composition_work_is_flat_when_population_doubles` preserves
  every overlap `NodeId` and reports `(visited=2, reconstructed=1,
  identities_reused=1, added=1)` for the same one-row advance at both resident
  populations.

Layout row paths now use the stable logical row key rather than the current
child ordinal. Layout frames are stored as immutable ordinary chunks plus a
persistent keyed row sequence; edge edits structurally share the retained
middle and construct only entering row fragments. The scene owner has a keyed
residency path which scans/validates entering frame bodies and ordinary ancestry
only; an overlap row reuses its already-proven cached row scene fragment without
re-reading its visual/frame body.

The first qualified release table crossing at scale 1.25 recorded `D_s=12`,
overlap 8, and the following exact work:

```text
view_nodes_cloned=12
composition_nodes_visited=49       # 1 + 4 × D_s
composition_nodes_reconstructed=48 # 4 × D_s
layout_nodes_visited=57            # 9 + 4 × D_s
scene_frames_scanned=57            # 9 + 4 × D_s
provider_binds=12
scene_frames_painted=48
primitive_prepare_calls=20
```

Two samples at that boundary measured candidate CPU 3.167/3.180 ms,
presentation-total 3.011/3.016 ms, materialization 0.152/0.116 ms,
reconciliation 0.104/0.101 ms, layout 1.104/1.095 ms, and scene assembly
1.369/1.418 ms. This was below the PC-004 isolated median (~3.395 ms), but
layout and scene still missed `B`.

The residual was then assigned exact currencies instead of inferred from the
phase timer. The presentation receipt now separately counts row roots, layout
frames considered by commit construction, registered commit nodes, appended
fragments, lowered draw operations, cache entries swept, semantic candidate
node/draw visits, and residency layout/node/draw/snapshot visits. The v1 clerk
accepts those fields when present without retroactively making them mandatory
for historical v1 receipts.

Two surgical owner repairs followed:

- a typed non-reset residency delta now reuses the previous semantic commit
  directly. Its provider revision already proves stable membership, order, and
  bind-visible content, so drawable coverage cannot become semantic identity;
- residency membership and draw order are compiled once in scene spatial
  topology and consumed by `scene::Residency`. The earlier derivation rescanned
  8,398 layout frames, 8,530 drawable nodes, and 15,500 draw operations for one
  89-node crossing. The repaired crossing records zero layout visits, 148 node
  visits, and 167 draw visits.

`layout::project_scroll_projections` now groups descendant frames and table
tracks by nearest scroll owner in one pass rather than rescanning the complete
frame/track collections per projection. Three release samples of the standard
crossing after these cuts measured:

| Sample | Candidate CPU | Presentation total | Layout | Scene assembly |
|---:|---:|---:|---:|---:|
| 1 | 2,246 us | 2,116 us | 930 us | 680 us |
| 2 | 2,539 us | 2,380 us | 995 us | 825 us |
| 3 | 2,771 us | 2,588 us | 1,084 us | 868 us |

The medians put both layout (995 us) and scene (825 us) inside `B`. Exact
currencies were identical across those samples: 40 row-root visits, 89 commit
nodes, 175 fragments, 250 draw operations, 227 cache entries swept, zero
semantic visits, zero residency layout visits, 148 residency node visits, and
167 residency draw visits. That stability identifies the remaining work but
does not make it delta-bounded.

The runway policy was tested rather than enlarged. Requiring 1.5 forward
viewports produced a steady 180-physical-pixel crossing with 26 resident / 15
entering / 9 departing rows at 3.016 ms, and a 720-pixel crossing with 80 / 42 /
27 rows at 14.708 ms. An experiment requiring only one complete forward
viewport while retaining the two-viewport maximum and half-viewport reverse
runway reduced the isolated receipts to:

| Physical height | Resident / entering / departing / overlap | Candidate CPU | Layout | Scene |
|---:|---:|---:|---:|---:|
| 180 | 23 / 12 / 6 / 11 | 2,903 us | 1,084 us | 975 us |
| 720 | 68 / 30 / 15 / 38 | 10,065 us | 3,874 us | 2,878 us |

The full matrix initially exposed five stale oracles, not five behavior
regressions: they still expected pinned row rectangles to be translated in
layout and capped every transition at the former visible-only row count. Those
assertions contradicted the PC-002 content-local law and the already-protected
80-row transition bound. They now require the pin to remain materialized while
its rectangle is offscreen under the submitted `SpatialSnapshot`, and require
predictive transitions to remain within the explicit 80-row cap. With those
ownership-correct oracles, all focused residency laws remain green. The
one-forward-viewport minimum is therefore admitted as a bounded policy repair,
while the 720-pixel result still identifies both legitimate `D_s` cost and the
resident flat-commit residual.

Native receipt
`target/release/examples/renderer-receipts/control-gallery-500px-idle-1784342501060.txt`
was written at 21:41:41 from the 21:41:21 executable. Source/executable mtimes
prove it contains the semantic fast path but predates the later diagnostic,
spatial-membership, layout-grouping, and runway cuts. It records 348 inputs, 90
residency candidates, 106 property ticks, 3,939 entering and 3,868 departing
rows, and zero semantic commits for ordinary typed residency traces. Disjoint
thumb jumps dominate its 35.874 ms scene and 34.930 ms layout p95s, and the
last jump did not converge before close. It is accepted as a partial native
behavior witness, not as the final PC-005 timing receipt.

While verifying compiled residency, the nested Control Gallery ratchet exposed
two correctness defects: table-track fragments consumed their row-local clip
instead of fixed submitted viewport coverage, and nearest-first scroll paths
were read from the wrong end. Both repairs are now covered by the native nested
table oracle and the dedicated compiled-membership oracle.

A later native scroll exposed a third correctness defect before the source cut
could be offered for another receipt:

```text
retained base commit must satisfy the scene contract:
InvalidSpatialTopology(UnknownNode(NodeId { space: Retained, value: 73 }))
```

The keyed scene cache had selected the latest cached row sequence for a list,
not the exact layout generation named by the typed delta. Active/pending
candidate interleaving can legitimately prepare generation `N+2` while `N+1`
is still retained, so that shortcut spliced row fragments from the wrong
predecessor and left an ordered draw naming a node absent from the candidate.
`RowSequence` now carries an immutable generation identity, `Layout` preserves
the exact predecessor sequence for every non-reset delta, and scene lookup
requires `Weak::ptr_eq` with that identity. No "latest" fallback remains.

The deterministic witnesses
`fast_residency_burst_coalesces_before_candidate_construction`,
`stale_presented_table_click_during_large_absolute_jump_converges`, and
`stale_presented_table_click_after_reversed_large_jump_converges` all pass,
as does the complete 62-test residency suite. The source cut therefore repairs
the reported `UnknownNode(73)` failure without asking a native operator to
reproduce it again.

The current release executable was then exercised by Windows automation rather
than a human operator. Five large page advances moved through the first several
hundred records, a scrollbar-thumb jump landed around row 579,000, and a
reverse thumb jump landed around row 153,000. The process remained responsive,
closed normally, and wrote
`target/release/examples/renderer-receipts/control-gallery-500px-idle-1784350719696.txt`
(23,720 bytes). It records 282 attempted / 282 present-submitted / zero skipped
frames, 580 entering / 508 departing / 656 overlapping rows, and converged
candidate/present-submitted property serial 251. No scene-contract panic or
unknown spatial node recurred.

That receipt is admitted only as a current-binary crash and behavior witness.
The workload field remained `control-gallery-500px-idle` and its formal guard
counter is zero, so the admission clerk may not treat the automation gestures
as the campaign's prescribed guard workload. Its 49,905 us presentation p95,
37,234 us layout p95, nine renderer deadline misses, 135,112 flat commit-frame
visits, 269,808 lowered draw operations, and 325,345 cache-entry sweeps instead
reinforce the terminal complexity verdict under disjoint jumps.

The table-track census also removed one accidental superlinear species:
`layout::table::project` now indexes frames once and walks bounded parent depth
for the nearest table/projection owner instead of rescanning the full resident
frame list for every row/header track. The dedicated table-track projection
oracle remains green. The projection still visits every resident table row,
however, so this is a reduction of `O(R^2)`-shaped work to `O(R × depth)`, not
the required delta boundary.

An immutable scene-row commit-fragment prototype was deliberately removed
before this receipt. It proved the missing seam, but accepting it in isolation
would have created two incomplete presentation representations: table tracks
append content to the same row nodes in a later global order phase, spatial
scope compilation currently assigns flat draw indices, residency snapshots
copy flat membership/order, and render plans batch from one flat order. Merely
wrapping the existing fragments in another persistent sequence would still
flatten at all four consumers, while moving tracks into each row would alter
z-order and multiply clip/scroll batches. The prototype therefore did not earn
production admission. A future campaign must change commit nodes/order,
spatial topology, residency membership, and render-plan sections as one
generation-preserving representation, with table tracks represented as a
separate structurally shared row-order phase.

The terminal cut passes all 62 focused residency tests and all 189 architecture
ratchets under `renderer-debug`; the nested table clip/membership oracle,
`cargo check --all-targets --features renderer-debug`, and
`cargo fmt --all -- --check` are green and warning-free. The full library and
doctest matrix passes 1,432 tests with four intentional ignores plus four
doctests; the ownership-correct pin/runway oracles described above are included.

Receipts through `control-gallery-500px-idle-1784337144796.txt` remain rejected
as PC-005 evidence because they came from the 20:04 executable. The automated
current-binary receipt above supersedes them only as a crash/behavior witness;
it does not satisfy the PC-005 proportionality workload or reopen the failed
gate.

**PC-005 fails its architectural exit gate.** Unchanged row bodies, text, paint, fragments, semantic
projection, and GPU content are literal zero, but scene assembly still visits
every cached row root, reconstructs flat commit registration/order, sweeps flat
caches, and copies a resident residency snapshot. Those exact counters grow
with resident population at constant row shape. A persistent commit/render-plan
representation must remove that residual. This campaign stops here by its own
explicit failed-gate rule: PC-006 does not open, PC-007–PC-009 are prohibited,
and no millisecond result is used to pardon the remaining `O(R)` work.

---

## PC-006 — Prove Qt class and decide the ceiling

**Owned question:** Does the corrected synchronous retained compiler meet latency budgets without a second CPU generation or worker boundary?

**Allowed scope:** release measurement, truthful counter correction, nonbehavioral receipt cleanup, and admission analysis.

**Forbidden scope:** implementing PC-007–PC-009 before their individual verdicts, or changing scheduling/slicing inside the court. Any behavioral correction returns to PC-005 and reruns its receipt before PC-006 restarts.

**Mandatory gates:**

- all architectural zeros green;
- pure-scroll residency work proportional to `D_s` and model-order work proportional to `A_m` plus named index maintenance;
- input-event, scroll-layout, and scroll-scene/commit p95 ≤ `B`;
- property/active renderer and complete property redraw p95 ≤ `T`;
- input-to-`present`-call p95 ≤ `2T`;
- maximum UI preparation slice ≤ `B`;
- no repeated p99/max starvation plateau;
- GPU and semantic controls within accepted regression bands;
- bounded memory and stable long-run behavior.

**Residual classification table:**

| Measured residual | Mandatory verdict |
|---|---|
| all work and timing gates pass | reject PC-007–PC-009; synchronous Qt/GTK class is sufficient |
| any unchanged overlap is visited, measured, painted, copied, prepared, or uploaded | PC-005 incomplete; no scheduling/worker admission |
| work is delta-bounded but a synchronous candidate repeatedly exceeds `B` or delays active output beyond `T` | PC-007 may be admitted |
| churn comes from destroyed/insufficient runway or direction policy | repair residency policy; do not mask it with generations/workers |
| candidate work is small but activation traverses nodes or exceeds `B` | repair activation to an `O(1)` swap |
| live provider/callback state blocks an already-admitted pure preparation boundary | PC-008 may be admitted |
| UI snapshot/export creation itself exceeds `B` | redesign/bound export first; workers cannot repair UI callback cost |
| immutable export is bounded but pure delta layout/text/fragment work still exhausts runway | PC-009 may be admitted |
| total CPU remains `O(R)` | return to PC-005; workers are rejected |
| renderer draw exceeds `T` while CPU gates pass | renderer-specific follow-up requires its own receipt |
| GPU fill/copy dominates | damage/partial present remains rejected until timestamps or a code-owned experiment prove that owner |
| acquire/present wait dominates | surface/backend follow-up; not a presentation-compiler mechanism |
| hit/access/caret/IME generation differs from visible pixels | correctness failure; no performance mechanism is admissible |

**Mechanism admission table:**

| Proposed mechanism | Admit only if | Reject when |
|---|---|---|
| PC-007 CPU active/pending | a complete residency candidate still needs multiple UI slices and atomic old/new isolation is measurably required | corrected delta completes within slice/event budgets |
| PC-008 immutable provider snapshots | live provider/callback ownership blocks pure bounded preparation or moving admitted work | provider work is already entering-row bounded and not causal |
| PC-009 worker compilation | pure retained layout/scene work remains over budget after PC-002–PC-005 and UI slicing cannot meet input latency; execution still requires PC-007 and the explicit PC-008 boundary verdict | on-thread compiler meets budgets |

Every row receives `ADMIT` or `REJECT BY RECEIPT` with counter evidence. Precedent is not evidence.

**Negative control:** run the acceptance logic against a deliberately slow injected row compiler and prove the correct optional mechanism is admitted for the correct owner rather than by elapsed time alone.

**Exit receipt:** a final Qt-class receipt and signed architecture ceiling. Rejecting all optional mechanisms is success.

**Evidence ledger:** _to be filled during execution._

---

## PC-007 — Optional upstream active/pending generation

**Existence gate:** execute only if PC-006 admits it. Otherwise close this checkpoint `REJECTED BY RECEIPT` and record why.

**Owned question:** Can admitted multi-slice CPU preparation preserve one complete active generation until one complete pending generation is ready?

**Allowed scope:** extend the existing runtime presentation active/pending mechanism upward, bounded resumable progress, latest-intent replacement, activation, stale retirement, and diagnostics.

**Forbidden scope:** `CpuPresentationEpoch`, `PresentationTreeGeneration`, a second activation queue, a second visible receipt, partial semantic activation, renderer access to compiler state, or workers.

**Required law:** candidate CPU facts, `scene::Stack`, GPU readiness, and submitted spatial/input geometry participate in one existing presentation generation.

**Positive witnesses:**

- active remains complete and drawable during pending preparation;
- pending work is sliced within the admitted budget and can be superseded;
- obsolete candidates cannot activate;
- activation swaps pixels and semantic geometry atomically;
- attempts, readiness, activation, submission, and visibility remain distinct;
- memory is bounded to one active, one preparing candidate, at most one latest successor, and documented scratch/recycle;
- resize, scale, surface/device loss, popup teardown, and window close cannot strand a generation.

**Negative controls:**

- attempt partial row activation and ensure generation consistency fails;
- finish an older candidate after a newer one and ensure stale rejection;
- inject preparation failure and ensure active remains intact.

**Cleanup:** remove any prior ad hoc residency candidate queue or freshness token displaced by the one-generation grammar.

**Exit receipt:** conditional `one_presentation_generation_grammar` ratchet green.

**Evidence / rejection ledger:** _to be filled during execution._

---

## PC-008 — Optional immutable provider snapshots

**Existence gate:** PC-006/PC-007 must issue an explicit boundary verdict before workers. Execute the API repair only if the live provider boundary is causal; otherwise close `REJECTED BY RECEIPT` with proof that existing preparation inputs are already immutable/pure or that provider work is noncausal. PC-009 may proceed only after either result is recorded.

**Owned question:** What immutable, generation-tagged values must cross the UI authority boundary for pure preparation?

**Allowed scope:** breaking provider/table APIs, immutable batch snapshots, stable row keys/revisions, serializable/value-semantic style and interaction descriptions, validation and stale rejection.

**Forbidden scope:** sending live `Rc`, `RefCell`, callbacks, model mutators, session state, active/pending authority, diagnostics handles, or GPU resources to a worker boundary.

This API break is constitutional repair, not rupture, only if the new boundary makes provider facts citizens of the existing value/revision physics.

**Required snapshot properties:**

- immutable after publication;
- complete for its declared derivation;
- carries owner revisions, never a worker-minted freshness identity;
- carries logical keys, content and template/compatibility revisions, and stable action/control identifiers rather than captured callbacks;
- provider invocation and application mutation stay on the UI authority boundary;
- stale results are rejected against current owner revisions;
- callback effects remain messages/commands, not captured execution capability;
- batch size and lifetime are bounded.

**Positive witnesses:**

- equivalent snapshot values produce equivalent row derivations;
- changed row revision invalidates only affected work;
- stale snapshot completion cannot activate;
- architecture tests prove forbidden interior mutability/callback/GPU fields absent.

**Negative controls:** attempt to include `Rc<RefCell<_>>`, a non-`Send` callback, or presentation authority and ensure compile/architecture tests reject the shape.

**Cleanup:** remove the displaced live-callback preparation path at the admitted boundary. Do not maintain both APIs indefinitely.

**Exit receipt:** conditional `worker_snapshots_are_pure_candidates` ratchet green.

**Evidence / rejection ledger:** _to be filled during execution._

---

## PC-009 — Optional worker compilation

**Existence gate:** execute only if PC-006 admits residual pure work, PC-007 is `COMPLETE` so worker results are pending payloads in the one presentation generation, and PC-008 is either `COMPLETE` or `REJECTED BY RECEIPT` with proof the existing inputs are already immutable/pure. Otherwise close `REJECTED BY RECEIPT`.

**Owned question:** Can pure retained row preparation leave the UI path without creating another source of truth?

**Allowed scope:** bounded worker pool, pure snapshot-to-candidate work, priority, cancellation, result validation, runtime-owned scheduling, and deterministic single-thread oracle.

**Forbidden scope:** model/provider callbacks on workers, GPU/surface work, input or activation authority, locks that block the UI thread, task IDs as identity, unbounded queues, or worker-owned diagnostics policy.

**Scheduling contract:**

- latest required coverage has priority;
- active visible coverage outranks speculative runway;
- cancellation is cooperative and checked at bounded intervals;
- the PC-007 bound remains authoritative: one preparing candidate and at most one latest successor per owner;
- results carry source revisions and are validated on UI authority;
- worker failure or panic cannot corrupt active state;
- deterministic single-thread execution remains an equivalence oracle.

**Positive witnesses:**

- UI event work remains within slice/latency gates under slow injected compilation;
- stale/out-of-order results never activate;
- rapid reversal and thumb jumps bound queued/worked obsolete candidates;
- shutdown, device loss, window destruction, and provider replacement retire work safely;
- semantic/pixel equivalence matches the synchronous oracle;
- memory and thread counts plateau;
- UI snapshot/export p95 stays within `B`;
- discarded work is bounded to one declared chunk per cancellation;
- total CPU for equal entering-row work is no more than 110% of the optimized on-thread PC-005 result unless an explicit energy/latency tradeoff receipt is accepted.

**Negative controls:**

- complete old/new candidates out of order;
- cancel at every yield boundary;
- inject worker failure;
- mutate owner revisions during work;
- saturate queue with alternating distant targets.

**Cleanup:** remove synchronous duplicate compilation and any provisional bridge queue once the worker path is admitted and proven.

**Exit receipt:** worker work is pure, cancellable, bounded, and subordinate to the existing presentation grammar.

**Evidence / rejection ledger:** _to be filled during execution._

---

## PC-010 — Burn down the old species

**Owned question:** Which obsolete paths, clocks, caches, and explanations can still resurrect resident-sized work?

**Allowed scope:** deletion, visibility reduction, naming cleanup, architecture tests, dead diagnostic removal, docs/census correction.

**Required burn-down census:**

- viewport/property fields in semantic/local/text/fragment/GPU content keys;
- absolute scrolled row geometry used as reusable local identity;
- full residency-to-rebuild path;
- resident-window composition/layout/scene walks;
- duplicate spatial/hit/access evaluators;
- parallel dirty/revision/freshness authorities;
- renderer entrances other than `scene::Stack`;
- duplicate active/pending or visible receipt grammars;
- obsolete caches that retain recomputed resident work;
- provisional bridge APIs from admitted optional checkpoints;
- visual-only table-cell shortcuts.

**Tombstones:** source-string architecture guards are acceptable as resurrection alarms only when paired with behavioral/counter witnesses.

**One-Way fixed point:** rerun every `OW-PC-*` cell until no admitted reduction or reverse edge remains. Record intentional SCCs and rejected splits.

**Negative control:** locally reintroduce each principal species and prove at least one narrow ratchet fails before reverting the deliberate fault.

**Exit receipt:** burn-down table names every removed path and remaining justified path, with no dual architecture.

**Evidence ledger:** _to be filled during execution._

---

## PC-011 — Close out and teach master design

**Owned question:** Is the new architecture the sole documented, tested, and measured framework truth?

**Required work:**

1. run the complete verification matrix from a cleanly attributable tree;
2. record actual test counts, warnings, receipt paths, hardware, and distributions;
3. prove the exit theorem item by item;
4. synchronize `docs/master_design.md` with the admitted clock, fragment, delta, and optional scheduling laws;
5. close or supersede relevant SE-009 roadmap/campaign items without erasing history;
6. fix the source census to final ownership and remove resolved open questions;
7. record API breaks and migration notes;
8. record rejected mechanisms as successful receipts;
9. inspect final diff for unrelated or inherited changes;
10. record commit/push state only if separately authorized.

**Closeout is forbidden** while any checkpoint evidence ledger is empty, any optional checkpoint lacks accept/reject status, or any literal-zero gate is inferred from time rather than counted.

**Exit receipt:** this campaign becomes the crash-safe evidence archive and `master_design.md` becomes the concise standing doctrine.

**Evidence ledger:** _to be filled during execution._

---

## Verification discipline

Each production checkpoint runs the narrowest proof first, then expands:

1. owner-level unit and semantic tests;
2. architecture ratchets and deliberate faulty controls;
3. formatting and warning-free all-target/workspace compilation;
4. full library, doctest, example, and relevant release/deep tiers;
5. native Control Gallery interaction smoke;
6. standard receipt workload and causal counter analysis;
7. ownership/source census and One-Way gauge;
8. diff hygiene and protected-worktree audit.

Native interaction is mandatory whenever topology, coordinate systems, scheduling, activation, hit/access projection, text/editing, or provider boundaries change.

Required named ratchets:

- `row_presentation_keys_are_content_local_and_property_free`;
- `presentation_tree_borrows_composition_identity_and_changes`;
- `property_scroll_bypasses_the_presentation_compiler`;
- `residency_delta_preserves_every_overlap`;
- `presentation_tree_is_private_and_renderer_invisible`;
- conditional `one_presentation_generation_grammar`;
- conditional `worker_snapshots_are_pure_candidates`.

Existing witnesses for composition-owned identity, `scene::Stack`-only render handoff, carried GPU identity/revisions, residency freshness, canonical property lookup, and sole spatial ancestry remain green.

---

## Rejected approaches

Rejected unless a future receipt reopens them:

- more GPU batching as the primary table-scroll fix;
- surface/acquire work as the primary explanation;
- replacing glyphon/custom text rendering without a new text-owner receipt;
- table cells reduced to non-semantic labels;
- damage tracking before presentation retention;
- memoizing the contaminated absolute `SceneKey`;
- deleting all geometry from keys instead of repairing coordinate ownership;
- larger runways as a substitute for bounded preparation;
- per-row GPU textures, layers, or surfaces as entitlement;
- moving the current `Rc`/`RefCell` object graph wholesale to workers;
- a second visual, hit-test, accessibility, or scroll truth;
- another presentation epoch or activation queue;
- blanket dirty flags or runtime dependency capture competing with `composition::Changes`;
- parallel old/new APIs left “temporarily” without a deletion checkpoint;
- public `PresentationTree` or a new crate justified only by code organization;
- Chromium-class complexity before the synchronous ceiling receipt.

---

## Non-goals

- redesign the application model or table feature set;
- weaken selection, editing, IME, accessibility, hit testing, or focus semantics;
- guarantee constant time for true global constraint/theme/font changes;
- replace the retained GPU renderer;
- redesign surface tenancy, popup, alpha, material, or device lifecycle law;
- solve remote/asynchronous data fetching unless an independently measured provider problem is admitted;
- promise zero allocations everywhere; the contract is owner-bounded work and explicit lifetimes;
- choose a worker/threading library during formulation;
- preserve source compatibility at the expense of ownership repair.

---

## Exit theorem

The campaign is complete only when all statements are proven:

1. property scroll within prepared coverage changes no semantic/local/text/fragment/GPU content key;
2. in-runway scroll records literal zero upstream presentation work;
3. row geometry is content-local; `AxisAdjustment` remains canonical; and current scroll is sampled through the existing property/spatial system;
4. pixels and every semantic projection consume one submitted spatial truth;
5. unchanged residency overlap preserves row identity, geometry, text, scene fragments, semantic projections, and GPU resources without expensive owner visits;
6. pure-scroll residency work scales with `D_s`, not resident-window/column/model size, and unchanged overlapping cells are literal zero;
7. insert/remove/replace/move and one-row revisions scale with declared `A_m`/dependencies, and the table provider exposes truthful membership/order changes;
8. memory, candidates, caches, and optional queues are bounded;
9. input service meets the pinned latency gates and the large-text control does not regress;
10. renderer still consumes only `scene::Stack` and carries upstream identities/revisions;
11. there is one canonical scroll adjustment, one composition change authority, one compatible scene property/spatial projection, and one presentation/visibility grammar;
12. every optional mechanism has an explicit acceptance or rejection receipt;
13. obsolete full-rebuild, contaminated-key, duplicate-truth, and bridge species are deleted and tombstoned;
14. architecture, semantic, native, performance, One-Way, and diff-hygiene matrices are green;
15. the final source census and `master_design.md` describe the same architecture the code executes.

Milliseconds cannot substitute for items 1–8 and 10–13. Literal-zero and ownership claims require direct witnesses.

---

## Resume protocol

An agent resuming this campaign must:

1. read this ledger and the source census completely;
2. inspect HEAD, branch, status, and inherited changes before editing;
3. compare live source with the protected formulation snapshot;
4. select the first unmet checkpoint exit; never skip PC-004 or PC-006;
5. rerun the narrow baseline/oracle relevant to that checkpoint;
6. keep exactly one checkpoint `IN PROGRESS`;
7. update its evidence ledger as facts are obtained;
8. delete displaced paths at the same green boundary;
9. run deliberate negative controls before closing;
10. hand the measured residual—not the planned mechanism—to the next checkpoint.

The governing arc is:

```text
clock divorce
  → retained local fragments
  → mandatory re-receipt
  → keyed residency delta
  → synchronous ceiling receipt
  → only then, receipt-admitted generation / snapshot / worker machinery
```

The edges have taught inward. This campaign teaches the compiler that feeds them.
