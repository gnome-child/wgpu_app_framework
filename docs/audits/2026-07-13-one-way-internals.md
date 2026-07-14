# One-Way Internals — Seams Before Crates

Status: **in flight**. Ignited from clean production baseline `1d7278c1`; the
only pre-ignition changes were this finalized formulation and its roadmap sync.
This is the canonical operating ledger
for the full cleanup campaign. The campaign runs its examination loop until
the exit theorem is satisfied. It does not stop after one cell, one rung, or
the exhaustion of the initial queue.

The [crate-seams investigation](2026-07-13-crate-seams-investigation.md) is a
useful census and source of hypotheses. It is not the architectural direction.
The live framework, `master_design.md`, established case law, and evidence
produced by this campaign decide which seams survive, move, merge, or disappear.

No files move into member crates during this campaign.

## Mission

Make the current monolith practice an honest one-way internal architecture
before Cargo is asked to enforce one.

Trace the framework broadly enough to find misplaced ownership, upward
dependencies, unnecessary translations and intermediate types, repeated
semantic decisions, hidden service coupling, unjustified visibility, and
module housing that contradicts the Examen. Correct each admitted finding at
its lowest honest owner, delete the displaced path, and ratchet the result.

At closure, the accepted virtual-crate graph is already a DAG; its crossings
are narrow, deliberate library contracts; and a later physical split is
packaging rather than architectural discovery.

## Authority

In descending order:

1. observed user-visible behavior and already-settled product decisions;
2. `docs/master_design.md` and narrower standing doctrine;
3. practiced case law in behavioral and architecture witnesses;
4. the live ownership, dependency, state, and lifecycle graph;
5. findings admitted by this campaign;
6. prior audits, proposed crate maps, industry precedent, and aesthetic
   preference.

The lower items may reveal a question. They may not overrule the higher items
without evidence that the higher account is stale or internally contradictory.

## Constitution

### Seams are discovered, not imposed

A candidate seam is a question until it survives tracing and admission. The
campaign may confirm the investigation's proposed foundation, text, command,
UI, renderer, runtime, platform, and facade seams; it is equally authorized to
move, merge, split, or reject them when the resulting contract would otherwise
be broader or less truthful than the coupling it replaces.

The campaign goal is not eight crates. It is a coherent one-way graph whose
eventual crates each have a stable sentence of ownership.

### One truth has one lowest honest owner

Every authoritative fact has one owner. Other layers consume a projection or
receipt; they do not recompute, shadow, or retain competing truth. Runtime may
coordinate several owners without absorbing their domain decisions.

### Types earn their existence by preserving meaning

A type survives when it makes a distinction structural: identity, authority,
lifecycle, clock, coordinate space, unit, fallibility, capability, or an
otherwise invalid state. A type that only transports the same facts between
adjacent owners is a reduction candidate.

Removing a type is not progress when its invariant merely reappears as
booleans, conventions, widened visibility, or duplicated checks.

### Repetition is judged by meaning

Centralize repeated semantic decisions only when they have the same owner,
inputs, outcomes, failure rules, and lifecycle. Similar syntax is not enough.
An admitted common concept must let the old local decisions be deleted; a
miscellaneous helper does not qualify.

Standing intentional non-merges remain case law. Reopening one requires new
evidence, not a fresh preference.

### Dependency inversion must remain honest

No generic service locator, callback smuggling, or trait whose sole purpose is
concealing a dependency arrow. Erasure does not erase ownership. A lower
contract is admitted only when it names a real capability and has a coherent
failure and lifetime model.

### Public surface is a budget

Rust's lack of workspace visibility makes every eventual cross-crate `pub` a
real design decision. No blanket visibility widening, public test-support
escape hatch, or broad state bag is permitted to make a proposed seam compile.
Reject or redraw the seam instead.

### Naming follows the established house law

- Names describe the domain concept, not its implementation trick.
- No new `core`, `common`, `types`, `util`, `helper`, or `manager` bucket.
- Moving or re-housing a concept does not authorize renaming it.
- Existing overloaded-name cleanup remains under its established census and
  is not opportunistically folded into this campaign.
- A rename is admitted only by a concrete collision, stale meaning, or newly
  clarified axis—not by crate fashion.
- Package and feature names are deferred until their charters and scopes are
  proven. Feature names, when later admitted, describe positive capability.

### Behavior and optionality are frozen

This is a behavior-preserving internal campaign. It introduces neither Cargo
member crates nor feature gates. It may identify and prepare honest feature
seams, including native realization and text mutation, but optional behavior
belongs to a later campaign after default-on extraction is proven.

## Operating mandate

The loop is the campaign's engine, not a planning exercise.

After a cell is independently green, append its receipt, commit it where
authorized, select the next highest-value cell, and continue. A zero-change
cell updates the ledger and continues. Completing a rung triggers its full
re-census and then the next rung. Do not yield merely because one correction,
one module, or the seeded queue is complete.

The campaign stops only when:

- the exit theorem is proven by the final fixed-point sweep; or
- progress requires a product decision, external authority, unavailable
  hardware, or other genuine blocker that safe in-scope investigation cannot
  resolve.

Feature campaigns such as Typed Stacking Contexts may consume the resulting
seams after closure. They are not inserted between rungs unless the user
explicitly reprioritizes the campaign.

## Scales

| Scale | Purpose |
|---|---|
| Campaign goal | One-way internals and split readiness |
| Rung | A dependency frontier to purify and re-census |
| Ledger cell | One bounded ownership or seam question |
| Loop | The method applied to every cell |
| Ratchet | Durable evidence preventing restoration of the displaced shape |
| Fixed-point sweep | Proof that no further admissible correction remains |

There are two repetitions. The inner loop works one cell to fixed point. The
outer loop re-censuses the framework and selects cells until the campaign exit
theorem holds.

## The loop

**Select → Trace → Model → Challenge → Admit → Reduce → Rewire → Prove →
Ratchet → Re-scan.**

1. **Select** one bounded cell: a forbidden dependency, disputed owner,
   repeated policy, suspicious intermediate, visibility leak, optional-field
   state cluster, bridge leak, module-housing question, test seam, or feature
   candidate. State one named seam question.
2. **Trace** every relevant entrance, outcome, lifecycle transition, consumer,
   backend, and failure path within the slice. Exhaustiveness is bounded by
   the selected question, not by an arbitrary file count.
3. **Model** the current owners, identities, authorities, clocks, coordinate
   spaces, units, fallibility, external dependencies, visibility, witnesses,
   and candidate virtual homes. Name both the current and proposed dependency
   graph.
4. **Challenge** every translation, intermediate type, recomputation,
   duplicate decision, upward import, optional field, boolean protocol,
   compatibility path, broad API, and misplaced test encountered by the
   trace. Consult the sweep lenses below.
5. **Admit** a correction only on concrete evidence. A user-visible defect is
   sufficient but not required: a proven forbidden dependency, cross-seam
   cycle, competing authority, false public surface, or dependency-weight leak
   is also evidence. Structural taste alone is insufficient. Record resistance
   and intentional non-merges as real outcomes.
6. **Reduce** machinery that carries no invariant. Delete unnecessary wrappers,
   translations, cached copies, states, aliases, compatibility paths, and
   repeated computation without erasing a real distinction.
7. **Rewire** the machinery that is real but owned at the wrong seam. Make one
   coherent ownership correction and delete or demote the old route.
8. **Prove** behavior, state transitions, absence, failure, cleanup, dependency
   direction, visibility, renderer topology, and performance in proportion to
   the correction. Behavior preservation includes negative and stale paths,
   not only the happy path.
9. **Ratchet** the finding with the cheapest durable mechanism justified by
   its recurrence risk: type/privacy boundary, narrowed API, unique-owner
   assertion, focused forbidden-import witness, tombstone, relocated test,
   charter, or ledger receipt. Do not build a global bureaucracy merely to
   count debt.
10. **Re-scan** the changed slice and its immediate consumers. Repeat the cell
    until fixed point, update the global gauge, and select the next cell.

### Trace matrix

For each cell, explicitly consider:

- entrances: public API, internal callers, event paths, commands, tests;
- outcomes: success, absence, decline, rejection, failure, cancellation;
- time: creation, steady state, replacement, retry, staleness, retirement;
- identity: creation, lookup, reuse, removal, eviction, destruction;
- hosts: in-frame, native, headless/test, platform-specific paths;
- presentation: candidate, prepared, submitted, presented, committed;
- performance: invalidation, allocation, shaping, batching/pass fusion,
  rebuild, and compile dependency cost where relevant.

Not every axis applies to every cell. The record states which were inapplicable
rather than silently omitting them.

## Sweep lenses

The loop examines more than import arrows:

1. **Ownership and authority** — duplicated truth, state at the wrong altitude,
   runtime answering instead of coordinating.
2. **Dependencies and cycles** — upward imports, peer cycles, callback-hidden
   edges, heavy external dependencies pulled downward.
3. **State shape** — boolean protocols, related `Option` clusters, invalid
   combinations, authority split across stores.
4. **Intermediates and translations** — wrappers or erased forms with no
   invariant, repeated conversion, bridge details escaping their edge.
5. **Repeated logic** — common semantic decisions waiting for a lower honest
   owner; deliberate duplication whose meanings only look alike.
6. **Failure and lifecycle** — `panic!`/`expect` standing in for agreement,
   ignored absence, incomplete cleanup, stale receipts, cancellation gaps.
7. **Identity, time, and space** — raw/stringly identity, collapsed clocks,
   logical/physical or owner/local coordinate confusion, repeated boundary
   conversions.
8. **Visibility and API compression** — unjustified `pub(crate)`, future
   cross-crate surfaces, broad data bags, APIs with no named consumer.
9. **Tests and witnesses** — white-box tests housed with the wrong owner,
   architecture reads tied to one source root, duplicated production
   algorithms in tests, missing negative witnesses.
10. **External dependencies and optional capability** — single-owner
    dependencies, platform/renderer leakage, credible feature boundaries and
    their honest feature-off behavior.
11. **Renderer and runtime economics** — paint order, pass/batch fusion,
    presentation clocks, invalidation, cache ownership, and compile-time cost.
12. **Housing, names, and doctrine** — modules organized around owners,
    declared charters matching live code, stale docs, and naming debt routed to
    its existing owner rather than casually renamed.
13. **Dead and compatibility structure** — retired aliases, parallel paths,
    allowances without owners/expiry, and scaffolding receiving new behavior.

## Admission law

A virtual crate seam survives only when all of these are true:

1. it has one sentence of ownership and a short forbidden-dependency list;
2. its crossing API is smaller and more stable than its implementation;
3. the resulting graph is acyclic without concealed callbacks or services;
4. it has an independent consumer, isolates meaningful dependency/compile
   weight, or establishes an honest optional-capability boundary;
5. its tests can live with the owner or observe the same contract as
   production; and
6. proving it deletes coupling, competing policy, or a private convention.

A seam that fails admission is merged, redrawn, or recorded as resistance. The
campaign does not preserve it to match the investigation ledger.

## Gauge and ratchet

Global dependency census is a gauge, not a suite gate. It guides selection and
measures progress without encouraging import laundering or freezing a
provisional map. Rung 0 makes the gauge trustworthy and establishes its
baseline.

Track at least:

- accepted-seam forbidden edges;
- cross-seam cycles and concealed service edges;
- concrete services crossing semantic seams;
- unresolved cross-seam `pub(crate)` uses;
- white-box tests depending on another proposed owner;
- direct architecture-witness source-root reads;
- external dependencies owned below their honest boundary;
- `#[allow(...)]`, panic paths, and compatibility structures lacking a named
  owner and disposition.

The global counts may rise when a more truthful map reveals debt. Progress is
the retirement of admitted confusion, not a monotonically flattering number.
Each completed cell receives a narrow ratchet only when justified by the
finding and its recurrence risk.

## Rungs

Rungs establish search order, not predetermined conclusions. Each ends with a
full re-census, updated candidate map, receipts, and a green repository. New
evidence may reorder later cells.

### Rung 0 — Establish the instruments and provisional map

- Verify the clean boundary and protected user state.
- Pin the current behavior and full verification ritual.
- Build or validate a workspace-aware dependency/source census as a reporting
  tool, including grouped imports, `crate::`/`super::` resolution, cfg/test
  separation, external dependencies, visibility, and test ownership.
- Record clean and incremental compile baselines.
- Assign every module and split-pending responsibility a provisional owner,
  explicitly marking disputed placements.
- Turn the investigation's proposed edges into revalidated candidates, not an
  inherited allowlist.
- Establish the cell queue and ledger record format.

This rung changes no production behavior.

### Rung 1 — Purify the lowest vocabulary

Start with high-confidence leaks that exercise the whole loop cheaply:

- platform-neutral scheduling versus winit `ControlFlow`;
- pointer grammar versus OS double-click metrics;
- document semantics versus platform file replacement;
- icon identity versus icon-pack realization;
- task/state/feedback vocabulary versus stores, stacks, and executors;
- command-owned failure versus generic low-level values.

Also inventory dead visibility, unowned allowances, panic/expect paths, and
compatibility scaffolding encountered in these slices. Do not bulk-move every
syntactic leaf into a foundation bucket.

### Rung 2 — Untangle text, geometry, and paint

- place renderer-neutral geometry at the lowest honest owner;
- remove renderer paint vocabulary from text layout and shaping;
- distinguish read-only selection/caret projection from mutation, history,
  draft, and IME machinery without gating it;
- keep renderer-ready scene grammar with its actual consumer;
- preserve shaping, hit testing, selection, overflow, snapping, and renderer
  topology exactly.

The rung establishes whether a low text library and a later text-mutation
capability are honest seams. It does not assume either result.

### Rung 3 — Free command meaning from concrete services

Trace command context, responders, input, keymap, target, clipboard, task, text
layout, timeline, notification, and effects end to end.

Find the smallest honest capability contracts, relocate service realization,
and delete compensating routes. Reject a standalone command seam if it requires
a service locator, an incoherent callback surface, or UI/runtime state exposed
as command vocabulary.

### Rung 4 — Separate semantic presentation from physical realization

- place semantic-scene lowering with the renderer rather than the OS adapter;
- keep renderer and runtime as peers if the live graph supports that law;
- make platform the top adapter for OS windows, event loops, native surfaces,
  dialogs, clipboard bridges, and physical cursor/IME realization;
- invert diagnostics so owners publish their own facts to observers;
- preserve candidate/prepared/submitted/presented/committed clocks and every
  native-popup lifecycle state;
- verify unchanged batching, pass fusion, invalidation, and frame economics.

### Rung 5 — Reassess the UI knot

Trace scene, view, widget, layout, composition, interaction, session, table,
virtualization, selection, draft, popup, overlay, theme, and pointer as one
connected territory before deciding its internal seams.

Break only edges that cross an admitted owner boundary, create competing
authority, drag heavy dependencies downward, force unjustified exposure, or
block a real capability seam. Intra-owner collaboration and cycles are not
failures. Organize modules around the resulting owners in accordance with the
Examen; do not split a state machine merely to improve browsing.

### Rung 6 — Visibility, tests, and feature-seam readiness

- give every accepted crossing a named consumer and visibility disposition;
- house white-box tests with their future owner;
- make cross-layer journeys use production contracts;
- replace direct source-root assumptions with one workspace-aware witness
  seam;
- prove no public test-support escape hatch is needed;
- audit external dependency ownership and compile isolation;
- formulate, but do not activate, positive feature seams and feature-off
  semantics for text mutation, native runner, native popup realization,
  system clipboard, dialogs, theme files, accessibility integration, and any
  additional capability admitted by evidence.

### Rung 7 — Final fixed-point sweep and closure

Re-run the loop bidirectionally across every accepted owner:

- doctrine to code;
- code to doctrine;
- each crossing in both directions;
- every external dependency to its owner;
- each public and cross-seam item to a named consumer;
- tests and witnesses to their owning contract;
- all recorded resistance, flags, intentional non-merges, and feature seams.

Reopen any cell contradicted by the final graph. Continue correcting and
re-sweeping until no admissible finding remains.

## Rung 0 record — instruments and provisional map

Status: **complete**. Production baseline `1d7278c1`; formulation/ignition
checkpoint `b7a9a317`. The branch was three commits ahead of `origin/master`
before ignition and not behind. Protected glass-tuner state remained
`comparison_open: true`.

### Behavioral and build baseline

- `cargo test --lib`: 1,065 discovered; 1,055 passed, 10 deliberately ignored,
  0 failed. Test-profile preparation took 44.43s; test execution took 4.74s.
- `cargo check --examples`: all five examples passed in 6.80s on the warm
  ordinary target.
- `cargo fmt --check` and `git diff --check`: passed.
- Isolated-target clean `cargo check --lib`: 47.934s.
- Same-target no-change incremental `cargo check --lib`: 0.926s (Cargo-reported
  work 0.80s).

The isolated target is `target/one-way-rung0-baseline`, already covered by the
repository's target ignore. These timings are local receipts, not promises
about another machine.

### Gauge instrument

Rung 0 added:

- `tools/one_way_census.py` — workspace-root discovery, Rust comment/literal
  masking, grouped `crate::{...}` and relative `super::{...}` resolution,
  cfg/test separation, module/slot edges, external-dependency users, SCCs,
  visibility and witness counts, and Markdown/JSON reports;
- `tools/one_way_slots.json` — the explicitly provisional slot/direction and
  heavy external-boundary hypotheses; and
- `tools/test_one_way_census.py` — six differently shaped parser witnesses
  for grouped crate paths, direct/grouped relative paths, masked receipts, and
  test-only separation.

The tool is a gauge only. It is not wired into the Rust suite and its slot map
is campaign data, not architecture law. Manual probes independently confirmed
the 100 `CARGO_MANIFEST_DIR` mentions, the 1,881 raw production-plus-test
`pub(crate)` spellings, and every high-priority external receipt before the
map was accepted as a starting instrument.

### Initial mechanical snapshot

| Metric | Rung 0 |
|---|---:|
| Top-level production modules | 45 |
| Unique production module edges | 325 |
| Unique test-only module edges | 95 |
| Provisional cross-slot edges | 30 |
| Provisional forbidden internal module edges | 15 |
| Provisional heavy external-boundary violations | 8 |
| Provisional slot SCCs | 1 (command/foundation/renderer/runtime/text/UI) |
| Production `pub(crate)` declarations | 1,738 in 183 files |
| Cross-slot-provider `pub(crate)` upper bound | 1,738 |
| Unique cross-slot test-only module edges | 75 |
| `CARGO_MANIFEST_DIR` mentions | 100 |
| Filesystem read calls | 288 |
| `#[allow(...)]` attributes | 10 |
| Production `panic!` calls | 9 |
| Production `.expect(...)` calls | 103 |

The visibility upper bound is intentionally labeled: every provisional slot
currently provides something across a slot boundary, so a module-level census
cannot truthfully identify which of its 1,738 declarations would cross. Cells
disposition concrete surfaces as their consumers are traced; Rung 6 closes the
remaining symbol-level budget. No false exact count is manufactured.

### Revalidated candidate edges

The mechanical gauge found more current production edges than the older hand
census because grouped relative imports are now expanded. These are questions,
not inherited violations:

| Provisional direction | Candidate module edges |
|---|---|
| command -> runtime/UI | `context -> clipboard/layout`; `input -> interaction/session`; `responder -> interaction/session/table` |
| foundation -> command/UI | `window -> notification/scene/theme` |
| renderer -> runtime | `render -> diagnostics` |
| text -> renderer | `text -> paint` |
| UI -> runtime | `layout/view -> diagnostics`; `widget -> document` |

Heavy external-boundary questions are:

- `animation -> winit`;
- `document -> windows-sys`;
- `pointer -> windows-sys`;
- `icon -> iconflow` and `text -> iconflow`;
- `task -> pollster`;
- `scene -> glyphon`;
- `platform -> wgpu`.

The full receipts are reproducible with
`python tools/one_way_census.py --format markdown` (or `json`). Each edge is
admitted, redrawn, or rejected only after its own trace.

### Provisional owner rulings and queue

All 45 modules have a provisional slot in `one_way_slots.json`; every slot
remains revisable. The dagger-like split questions from the investigation are
preserved as cells rather than silently assigned: animation scheduling versus
winit projection; icon identity versus pack realization; task/state/feedback
vocabulary versus machinery; window facts versus UI policy; text geometry
versus paint; command contracts versus service realization; diagnostics facts
versus aggregation; semantic scene versus renderer payloads.

Rung 1 begins with `animation::Schedule -> winit::ControlFlow`: the smallest
unambiguous downward dependency, already behaviorally witnessed, and therefore
the pilot for the complete cell loop. The remaining Rung 1 cells are ordered by
dependency weight unlocked and independent provability, not by the old census
table.

## Rung 1 cell records

### R1-01 — platform-neutral schedule versus winit realization

Status: **complete**. Correction `754c5a54` (`Move event-loop schedule
projection to platform`).

1. **Question and trace.** `animation::Schedule` is produced by overlay,
   caret, visual-animation, runtime, shell, host, and platform work paths. Its
   merge and due-time laws are consumed throughout those paths. Exactly one
   operation crossed into winit: `Schedule::control_flow`, called only by the
   native runner immediately before `ActiveEventLoop::set_control_flow`.
   Outcomes traced: idle wait, future deadline wait, due deadline poll,
   next-frame poll, exiting event loop, backend-forced poll, and normal merged
   runtime schedule.
2. **Current graph.** A foundation-shaped time vocabulary imported and returned
   `winit::event_loop::ControlFlow` solely for a platform consumer. This was an
   external dependency leak, not a shared invariant.
3. **Admission.** The event-loop type and policy have one consumer and one
   physical effect. Relocating the conversion to that adapter makes the
   crossing smaller (`Schedule` in, `set_control_flow` locally) and deletes
   winit from the lower module. Admitted.
4. **Reduction/rewire.** Deleted the winit import, `Schedule::control_flow`, and
   its misplaced test from `animation.rs`. Added the private conversion beside
   `sync_control_flow` in `platform::runner::native`; no producer, merge, due,
   exit, or polling behavior changed.
5. **Proof and ratchet.** Six schedule/transition tests, the four-outcome native
   projection witness, and the new structural-absence architecture witness
   passed. Full library result: 1,056 passed, 10 ignored, 0 failed; all examples,
   formatting, and diff checks passed.
6. **Gauge delta.** Heavy external-boundary questions 8 -> 7;
   production `pub(crate)` declarations 1,738 -> 1,737. Production module edges
   and provisional internal back-edges remained 325 and 15. The new ratchet
   increased test-only module edges 95 -> 96, cross-slot test edges 75 -> 76,
   source-root mentions 100 -> 101, and filesystem read calls 288 -> 290; Rung
   6 will collapse those witness paths behind the workspace helper.
7. **Fixed point.** No `winit` or `ControlFlow` reference remains in animation;
   the native runner is the only Schedule-to-ControlFlow owner. Cell closed.

### R1-02 — pointer click grammar versus OS metrics

Status: **complete**. Correction `b08b5970` (`Inject platform multi-click
thresholds`).

1. **Question and trace.** The click chain is interaction truth keyed by target,
   point, instant, and prior count. Its only platform input is the system's
   interval and x/y distance thresholds. The trace covered parent and native
   popup presses, primary versus ignored buttons, selection/rejection chain
   cancellation, single/double/triple cycling, target changes, exact threshold
   boundaries, direct/headless Runtime callers, and native settings changes
   between presses.
2. **Current graph.** `pointer::MultiClickSettings` was a useful platform-neutral
   value, but its `system()` constructor imported Windows FFI and interaction
   queried it directly during classification. Grammar and realization were
   therefore housed together despite different owners.
3. **Admission.** The value has an invariant and multiple consumers; it stays.
   OS acquisition has one adapter owner and moves. Because custom backends need
   to provide the same fact, `pointer::MultiClickSettings` and the Runtime
   builder configuration are a deliberate namespaced contract rather than a
   hidden callback or platform import. Admitted.
4. **Reduction/rewire.** Deleted Windows/non-Windows system acquisition from
   pointer. The native event adapter now refreshes thresholds before every raw
   press—the same clock as the former per-classification query—and injects them
   into Session. Interaction consumes the retained value. Direct/headless hosts
   receive deterministic defaults and may configure the public Runtime builder.
5. **Proof and ratchet.** Added exact threshold and target-identity behavioral
   witnesses plus structural absence of Windows details from pointer. Parent
   and popup raw presses share the one runner translation path. Full library:
   1,059 passed, 10 ignored, 0 failed; all examples, format, and diff checks
   passed.
6. **Gauge delta.** Heavy external-boundary questions 7 -> 6; pointer no longer
   appears as a `windows-sys` user. Production edges and provisional internal
   back-edges remained 325 and 15. Deliberate retained configuration raised
   production `pub(crate)` declarations 1,737 -> 1,739. The architecture
   ratchet raised source-root mentions 101 -> 102 and filesystem reads
   290 -> 293; test-only edge counts remained 96/76.
7. **Fixed point.** Pointer/interaction contain policy and state only; platform
   alone queries OS click metrics; native settings remain live per press; a
   custom host has an explicit value contract. Cell closed.

### R1-03 — document atomic replacement versus native-window platform

Status: **resistance; no production correction admitted**. Gauge/configuration
checkpoint follows this record; the live persistence path is deliberately
retained.

1. **Question and trace.** `SaveSnapshot::write_to` streams a versioned buffer
   into a uniquely created sibling, syncs contents, atomically replaces the
   destination, and removes the sibling on every failure. It is called by
   synchronous `Document::save_to`, direct snapshot callers, and deferred
   worker tasks. Windows needs `MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH)`
   because an open source file cannot use the ordinary rename path. Non-Windows
   uses `std::fs::rename`. Version/generation rejection occurs after the write
   and remains separate from filesystem success.
2. **Current graph.** The only `windows-sys` use below the native platform is a
   private implementation step inside the document persistence owner. No UI,
   event-loop, window, renderer, or runtime-session policy enters the path.
3. **Challenge.** Moving replacement to the native-window platform would make
   public `SaveSnapshot` and background tasks call upward through a callback,
   service locator, or trait with one real implementation. A new filesystem
   crate/module would likewise have one consumer and a contract no smaller than
   the private function. Either move would conceal or enlarge coupling without
   deleting competing authority.
4. **Ruling.** Reject the candidate seam. Atomic replacement stays with the
   document persistence guarantee. `document -> windows-sys` is an explicit
   module-level exception to the platform-slot external boundary, not blanket
   permission for the runtime slot. A future document-workflow feature owns
   this dependency if that capability is admitted. A separate filesystem seam
   requires a second consumer or a smaller proven contract.
5. **Proof.** Existing witnesses cover synchronous round-trip, CRLF preservation
   while the source is open, snapshot identity/revision, deferred generation
   rejection, replacement of an existing destination, and temporary-sibling
   cleanup. The four `document_save` cases, the CRLF case, and the architecture
   ownership witness passed unchanged.
6. **Gauge correction and delta.** The external scanner now requires a
   dependency name at the root of a Rust path. A sixth parser witness proves
   `use windows::...` is counted while `std::os::windows::...` is not; this also
   stops treating `glyphon::cosmic_text` as a direct cosmic-text import. The
   corrected Rung 0 external baseline is 8, R1-01 is 8 -> 7, and R1-02 is
   7 -> 6. Recording the narrow document exception changes the current count
   6 -> 5 without pretending code was deleted.
7. **Fixed point.** One document owner retains the complete atomic write
   transaction and one target-specific private primitive. No smaller honest
   seam is evidenced. Cell closed as resistance.

### R1-04 — icon identity and pack-backed realization

Status: **complete; investigation map redrawn**. Correction `b5ad7720`
(`Admit icon pack as an independent seam`).

1. **Question and trace.** `icon::Id`, `Style`, `Icon`, and `Glyph` flow through
   view hints, semantic scene icons, paint items, shortcut chrome, text inline
   layout, and renderer batches. `Icon::glyph` resolves the established
   phosphor identity/style to a family and codepoint; text also loaded the same
   pack's embedded fonts directly from iconflow. Missing icons remain absence,
   style fallbacks remain pack policy, and text/renderer consume the resulting
   glyph without owning pack selection.
2. **Current graph.** The old provisional foundation placement made iconflow a
   dependency leak, while splitting identity from realization would break the
   existing public `Icon::glyph` sentence and create two mutually dependent
   concepts. Text's second iconflow import duplicated pack knowledge rather
   than proving a separate text owner.
3. **Admission.** A dedicated `icons` virtual owner is smaller and more honest:
   it owns icon identity, style, selected-pack resolution, glyph facts, and the
   selected pack's embedded font sources. It has independent text, UI, and
   renderer consumers, isolates iconflow, imports no other framework owner, and
   preserves the established API. The campaign is not constrained to the
   investigation's eight slots; the ninth seam is admitted.
4. **Reduction/rewire.** Added one icon-owned `font_bytes` projection and made
   text consume it, deleting text's direct iconflow knowledge. The provisional
   map now places `icon` in `icons`; text/UI/renderer/platform/facade may depend
   on that slot. No files moved and no feature gate was introduced.
5. **Proof and ratchet.** Glyph resolution, missing glyph, style selection,
   nonempty embedded fonts, inline icon-cache behavior, shortcut-icon behavior,
   and unique iconflow ownership passed. Full library: 1,061 passed, 10 ignored,
   0 failed; all examples, formatting, and diff checks passed.
6. **Gauge delta.** Heavy external-boundary questions 5 -> 3: both
   `icon -> iconflow` and `text -> iconflow` are resolved by one admitted owner.
   Production edges and internal forbidden edges remain 325/15. The new honest
   slot raises cross-slot edges 30 -> 33 without joining the existing SCC.
   Production `pub(crate)` declarations 1,739 -> 1,740; source-root mentions
   102 -> 103 and filesystem reads 293 -> 295 for the new ratchet.
7. **Fixed point.** One icon owner names identity and selected-pack realization;
   no other module imports iconflow. Alternate/glyph-less packs remain a later
   capability question, not a split forced without a caller. Cell closed.

## Initial hypotheses and queue

The investigation suggests foundation, text, command, UI, renderer, runtime,
platform, and facade as a useful first map. The campaign owes none of them
survival. Its first question for each is whether the live crossing contract is
smaller and more coherent than keeping the concepts together.

Initial cells, subject to Rung 0 revalidation:

1. pilot: platform-neutral scheduling versus `winit::ControlFlow`;
2. the remaining high-confidence lower-boundary leaks in Rung 1;
3. cross-seam visibility, test, allowance, and panic findings encountered by
   those traces;
4. the text/paint/geometry knot;
5. concrete services crossing the command boundary;
6. semantic-scene lowering and diagnostics observation;
7. the UI knot only after the lower contracts are stable.

Queue priority is: clearest ownership contradiction, highest downward
dependency weight, smallest independently provable correction, then the
correction that unlocks the most later cells.

## Cell record

Every cell records:

1. question and selected bounds;
2. traced entrances/outcomes/lifecycles/backends;
3. current owner and dependency graph;
4. proposed owner or resistance ruling;
5. admission evidence;
6. displaced or deliberately retained path;
7. implementation and deletion;
8. dependency, visibility, API, test, and performance delta;
9. verification and commit receipt;
10. fixed-point result and next unlocked cells.

## Verification and commit discipline

- Preserve unrelated working-tree changes and protected example state.
- One coherent ownership correction per commit where practical.
- Each production cell is independently green and reviewable.
- Run focused behavioral and architecture witnesses during the cell.
- Run the full library/examples/format/diff ritual at every rung boundary and
  whenever a cross-cutting owner changes.
- Run deep-tier GPU/native/performance witnesses when the changed seam can
  affect their law; unavailable hardware remains an explicit caveat, never a
  fabricated guarantee.
- Record hashes, counts, deleted paths, graph deltas, compile receipts, and
  zero-change rulings in this ledger.
- No mid-campaign push unless explicitly requested.

## Non-goals

- physically creating workspace member crates;
- introducing feature gates or changing default capability;
- changing user-visible behavior;
- minimizing module, type, import, or line counts;
- eliminating lawful cycles within one cohesive owner;
- conforming to the Fable 5 candidate map;
- opportunistically completing unrelated naming or feature campaigns;
- preserving a proposed seam by widening or obscuring its API.

## Exit theorem

The campaign is complete only when all of the following are evidenced:

1. Every module and split responsibility has one admitted virtual owner and a
   short charter stating what it owns and must not depend on.
2. The accepted virtual-crate graph is a DAG with zero prohibited crossings or
   concealed service/callback back-edges.
3. Every crossing is a deliberate, named library contract with an identified
   consumer, failure model, lifecycle, and visibility disposition.
4. Lower owners do not import higher reasons for existence. Renderer-specific
   dependencies live with rendering; OS/window-system dependencies live at the
   platform edge; any exception is explicitly justified by the accepted graph.
5. Authoritative facts, identities, clocks, coordinates, and cleanup lifetimes
   have one owner; displaced computations and parallel paths are deleted or
   reduced to witnesses.
6. No proposed seam depends on a generic service locator, callback smuggling,
   broad state exposure, blanket visibility widening, or public test-support.
7. Tests and architecture witnesses are housed for the future workspace and
   observe real contracts; direct single-crate source-root assumptions are
   retired behind one workspace-aware seam.
8. Optional-capability candidates have evidence-backed boundaries and honest
   absent-capability semantics recorded, without gates being introduced here.
9. Behavior, renderer topology, presentation clocks, and measured performance
   remain equivalent to the campaign baseline except for explicitly recorded
   and authorized internal economics improvements.
10. A complete final bidirectional sweep finds no further admissible ownership,
    dependency, visibility, state-shape, repetition, housing, or seam
    correction.

At that point, and not before, a separately authorized workspace campaign may
choose package names, create member crates, preserve the facade, and introduce
feature gates one at a time. The architecture will already have been decided
and practiced inside the monolith.
