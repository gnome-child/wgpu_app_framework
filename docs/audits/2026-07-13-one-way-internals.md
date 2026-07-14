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
- **One canonical spelling through every projection.** A type is declared with
  the simple name exposed by the API. If a re-export would otherwise require
  `CompoundName as Name`, rename the declaration itself to `Name`; do not keep
  the compound declaration behind an alias. The same spelling must survive
  every projection depth, including parent and higher-parent re-exports: write
  `pub use module::Name`, never `pub use module::CompoundName as Name`.
- **Namesake modules flatten only their central type.** When public module
  `module` owns the central public type `Module`, its parent publicly
  re-exports only `Module` from that module. Supporting declarations retain
  simple names inside `module` and are never flattened into a parent namespace.
- **Call sites qualify namesake-module support.** A call site needing the
  module and its central type imports `use parent::{module, Module};`, uses
  `Module` for the central type, and uses `module::Type` for every supporting
  type. Namespace qualification resolves collisions; compound declarations,
  aliased projections, and flattened supporting re-exports do not.
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

### R1-05 — task vocabulary versus queues and execution

Status: **complete; investigation map redrawn**. Correction `d417aab6`
(`Admit task system as an independent seam`).

1. **Question and trace.** `Task::new`, `ready`, and `future` produce deferred
   event work; `Id`, `Status`, and `Outcome` preserve identity and lifecycle;
   `Queue` and `Sink` own pending, completed, canceled, and stale-completion
   transitions; and `Executor` realizes native-host work on a bounded named
   worker pool. The trace covered command spawning, direct/headless
   `run_next`, native dispatch and completion events, no-poll wake accounting,
   cancellation before and during execution, restore, discarded stale
   completions, unchanged outcomes, and worker-thread realization.
2. **Current graph.** The provisional foundation slot housed the whole task
   subsystem while treating `task -> pollster` as misplaced runtime weight.
   A candidate vocabulary/machinery split would put the public values below
   private queues, sinks, and executors despite those parts implementing one
   deferred-work lifecycle and the standing doctrine assigning bounded worker
   execution to that lifecycle owner.
3. **Admission.** The split is rejected. A tenth virtual owner, `tasks`, owns
   deferred work identity, status, outcome, future realization, queuing,
   cancellation, completion, and bounded worker execution. It has independent
   command, runtime, platform, and facade consumers, imports no framework
   owner, and isolates pollster without a callback, service locator, or state
   bag. `Task`, `Id`, `Status`, and `Outcome` remain its crossing vocabulary;
   `Queue`, `Sink`, and `Executor` remain private machinery pending the Rung 6
   symbol-level visibility disposition.
4. **Reduction/rewire.** No production route or intermediate was displaced:
   the live ownership was already coherent. The provisional map now removes
   `task` from foundation, admits `tasks` as a dependency-free slot, and
   assigns pollster to `tasks` and platform—the latter separately realizes
   GPU startup. No module, API, or established name moved or changed.
5. **Proof and ratchet.** The existing architecture witness now also pins
   `Task::future` realization inside task and excludes UI, renderer, and
   platform dependencies. Twenty focused task witnesses passed across every
   traced lifecycle. Full library: 1,061 passed, 10 ignored, 0 failed; all
   examples, formatting, census parser tests, and diff checks passed.
   Presentation clocks, renderer topology, and frame economics were
   inapplicable because the cell changed no production path.
6. **Gauge delta.** Heavy external-boundary questions 3 -> 2. Production and
   test-only module edges remain 325/96; provisional forbidden internal edges
   remain 15. The honest slot raises cross-slot edges 33 -> 37 but remains
   outside the existing SCC. Production `pub(crate)` declarations remain
   1,740; cross-slot test edges and source-root mentions remain 76/103. The
   strengthened witness raises filesystem read calls 295 -> 296.
7. **Fixed point.** One owner now names the complete deferred-work lifecycle;
   pollster is honest implementation weight at that owner; no higher framework
   concept enters task. Cell closed.

### R1-06 — state vocabulary versus model storage

Status: **complete; investigation map redrawn**. Correction `2248151a`
(`Admit model state as an independent seam`).

1. **Question and trace.** `State` constrains the durable application model;
   `Store` owns its current and saved revisions, dirty truth, bounded change
   log, retained transaction snapshot, and model value; `Reason` and `Change`
   describe committed transitions; and `Snapshot` carries restorable model
   truth. The trace covered initial cleanliness, mutation, ignored and changed
   responses, save, load, restore, undo/redo, change-log pruning, retained
   snapshot reuse, and the deliberate independence of model revision from the
   presentation epoch.
2. **Current graph.** The provisional foundation slot housed the complete
   state subsystem. A candidate vocabulary/store split would place `State`,
   `Revision`, `Reason`, `Change`, and `Snapshot` below the only machinery that
   mints, retains, prunes, saves, and restores those facts. The public values
   are receipts of that lifecycle, not a second owner.
3. **Admission.** The split is rejected. The established `state` owner is
   admitted as an independent, dependency-free virtual seam: it owns model
   storage, snapshots, revisions, change reasons, dirty truth, and committed
   transitions exactly as `master_design.md` already states. Command, UI,
   runtime, platform, and facade are independent consumers; state imports no
   higher framework reason for existence and needs no widened API or hidden
   callback.
4. **Reduction/rewire.** No production route or type was displaced. The map
   removes `state` from the provisional foundation bucket and gives the
   existing module its own slot. Public `Store`, `State`, `Snapshot`,
   `Revision`, `Reason`, and `Change` remain the crossing vocabulary;
   `PendingSnapshot`, direct mutation, commit, retention, and pruning remain
   owner-private machinery for Rung 6 visibility disposition. No established
   name changed.
5. **Proof and ratchet.** Seven focused lifecycle witnesses pinned initial
   state, commit/dirty behavior, save, undo/redo, bounded retention, restore,
   and revision-versus-presentation clocks; the broader state filter passed 41
   witnesses. Full library: 1,061 passed, 10 ignored, 0 failed; all examples,
   formatting, and diff checks passed. The standing doctrine plus the explicit
   map are the cheapest ratchet; adding another source-root architecture read
   would only enlarge debt already assigned to Rung 6.
6. **Gauge delta.** Production/test module edges, internal forbidden edges,
   external questions, and the existing SCC remain 325/96, 15, 2, and 1.
   Exposing four real consumer relationships raises cross-slot edges 37 -> 41;
   state itself remains outside the SCC. Visibility, cross-slot test edges,
   source-root mentions, filesystem reads, allowances, panics, and expects are
   unchanged.
7. **Fixed point.** One dependency-free owner contains both model-state
   vocabulary and the machinery that makes its receipts truthful. Runtime
   coordinates state without absorbing it; presentation freshness remains a
   separate clock. Cell closed.

### R1-07 — feedback vocabulary versus ranked stacks

Status: **complete; investigation map redrawn**. Correction `dd59d4ff`
(`Reduce feedback storage to its semantic seam`).

1. **Question and trace.** `Severity` ranks runtime facts; `Stack` eagerly
   formats reports, retains one independent fact per severity, suppresses
   unchanged replacement, and projects the highest current fact. Draft input
   owns target identity and rejection lifetime; window session owns window
   identity and ephemeral lifetime; runtime exposes report/clear operations;
   view maps the winning fact into established hit-transparent presentation.
   The trace covered duplicate reports, priority fallback, per-severity and
   complete clearing, invalid draft mutation/success/cancel/eviction/removal/
   destruction, window destruction, hover explanation from both the invalid
   field and glyph, and nontrapping window feedback.
2. **Current graph.** Feedback was provisionally grouped into foundation.
   Splitting public `Severity` from private ranking/formatting would separate
   the fact vocabulary from its only coexistence law. Inside the stack,
   however, private `Entry { severity, text }` repeated severity already
   encoded by the array slot and merely transported the same pair back to two
   consumers.
3. **Admission and reduction.** The vocabulary/stack split is rejected and a
   dependency-free `feedback` virtual owner is admitted. The redundant
   `Entry` is deleted: `Stack` now stores formatted strings directly and its
   winner projects `(Severity, &str)`. This preserves eager single formatting,
   unchanged suppression, ranked coexistence, and fallback while removing a
   type with no identity, authority, lifecycle, or invalid-state prevention.
4. **Rewire and boundaries.** Draft input and window session consume the
   direct winning projection. The map removes feedback from foundation and
   gives UI, runtime, and facade the honest seam. `feedback::Severity` remains
   the complete public vocabulary; `Stack` remains private crossing machinery
   pending Rung 6 visibility disposition. Typed stores still own identities
   and lifetimes, and view/overlay still own chrome, placement, exposure, and
   hit transparency. No established name changed.
5. **Proof and ratchet.** The owner tests pin one-time `Display` formatting and
   Error/Warning/Info fallback. Focused witnesses pin rejection-at-most-draft,
   inline projection without an eager panel, field-and-glyph hover explanation,
   ranked window truth, both destruction paths, and nontrapping presentation.
   Full library: 1,061 passed, 10 ignored, 0 failed; all examples, formatting,
   census parser tests, and diff checks passed. Renderer topology and frame
   economics are unchanged because the same winning projection reaches the
   same presentation path.
6. **Gauge delta.** Production/test edges, forbidden internal edges, external
   questions, and the SCC remain 325/96, 15, 2, and 1. The two live UI/runtime
   consumer relationships raise cross-slot edges 41 -> 43; feedback remains
   outside the SCC. Deleting `Entry` and four associated methods lowers
   production `pub(crate)` declarations 1,740 -> 1,735. All remaining gauge
   counts are unchanged.
7. **Fixed point.** Feedback has one owner for fact shape, formatting, and
   ranking; typed stores own retention; presentation owns communication.
   Neither a universal anchor nor a second stack/message wrapper is admitted.
   Cell closed.

### R1-08 — command failure versus generic error housing

Status: **complete**. Correction `c863f06a` (`Move command failures to their
semantic owner`).

1. **Question and trace.** The root `Error` flows from command registration,
   shortcut resolution, typed argument/output erasure, responder claims,
   availability, routing, invocation, response transport, shell/host
   propagation, and the platform's explicit framework/backend sum. Every
   variant names command meaning: unknown command, missing/ambiguous/disabled
   target, argument/target/output mismatch, ambiguous shortcut, or shortcut
   arguments. Outcomes traced include absence, disabled claims, ambiguity,
   type mismatch, successful typed output, backend failure, and public error
   formatting/source propagation.
2. **Current graph.** The enum lived in generic top-level `error.rs` and most
   internal consumers imported that root path, although the provisional map
   already assigned the file to command. In contrast, clipboard unavailability,
   document I/O, renderer preparation/presentation, theme parsing, native
   realization, event-loop failure, and backend failure already retain their
   own typed errors and failure models. Merging those operational failures
   would erase owner and recovery distinctions.
3. **Admission.** Command owns the framework error enum and its crate-internal
   `Result` alias. The alias is retained because several command consumers use
   the same standard sum and it names that failure domain; it does not absorb
   any low-level error. The established public `wgpu_l3::Error` and
   `wgpu_l3::error::Error` names remain facade projections of the one command
   type. No rename or compatibility enum is introduced.
4. **Reduction/rewire.** Moved the definition and `thiserror` derive to
   `command::error`, exported `command::Error`, and migrated every production
   consumer from the generic root facade to `command::Error` or
   `command::Result`. Top-level `error.rs` is now a one-line facade reexport and
   the virtual map assigns that compatibility surface to facade rather than
   pretending it owns the type. Root `Error` now reexports from command.
5. **Proof and ratchet.** Focused command coverage passed 110 registration,
   shortcut, state, routing, response, and invocation witnesses; disabled,
   ambiguous-target, missing-target, and facade-type cases passed directly.
   The architecture ratchet pins the command definition, the one-line public
   facade, and absence of internal root-error imports outside the owner/facade
   witness. Full library: 1,063 passed, 10 ignored, 0 failed; all examples,
   formatting, census parser tests, and diff checks passed. No presentation,
   renderer, or performance path changed.
6. **Gauge delta.** Production module edges fall 325 -> 322. Slot edges,
   forbidden internal edges, external questions, and the SCC remain 43, 15,
   2, and 1. The named command `Result` definition/export raises production
   `pub(crate)` declarations 1,735 -> 1,737. The new behavioral and ownership
   witnesses raise test-only edges 96 -> 97, cross-slot test edges 76 -> 77,
   source-root mentions 103 -> 104, and filesystem reads 296 -> 298; Rung 6
   remains responsible for consolidating those reads.
7. **Fixed point.** Registration, routing, and invocation failure has one
   command owner. The root error module is facade only, and lower operational
   failures remain typed at the owners that can define their recovery and
   lifetime. Cell closed.

### R1-09 — Rung 1 hygiene and task worker startup failure

Status: **complete**. Correction `176aa109` (`Handle task worker startup
failure`).

1. **Question and trace.** The required Rung 1 hygiene pass inventoried
   production `pub(crate)`, allowances, panic/expect paths, and compatibility
   structure in animation, pointer, document, icons, tasks, state, feedback,
   command failure, and the immediate command/responder consumers encountered
   by those traces. One site described an operational outcome as an invariant:
   `task::Executor::new` expected every requested OS worker thread to start.
   The trace covered full, partial, and zero worker capacity; queued jobs;
   executor drop; native dispatch rejection; cancellation; event-loop proxy
   completion; and the direct/headless path.
2. **Inventory disposition.** No Rung 1 owner contains an `#[allow(...)]` or a
   production `panic!`. The ten global allowances have named later owners: the
   overlay dead-code allowance and popup/layout argument-shape allowances go to
   the Rung 5 UI examination; native-popup argument shape goes through Rungs 4
   and 5; and the six platform/backend private-interface allowances go through
   Rungs 4 and 6. The nine global production panics likewise route to Rung 2
   shaping cache (one), Rung 4 render-filter shader (two), and the Rung 3/5
   command/UI examination for layout frames, focus, and standard-menu topology
   (six). They are owned debt, not silently accepted residue.
3. **Retained invariants and visibility.** The task sink's downcast follows an
   exact `TypeId` admission check; responder insertion, response-effect
   cardinality, and menu-section insertion are local cardinality proofs. The
   registered-category label expectation is a command state-shape question and
   travels with the complete Rung 3 command trace rather than being hidden as
   indexing or `unreachable!`. Icon expectations are test-only. All-target
   compilation reports no dead-code warning in the touched owners. Their live
   crate-visible crossings have named consumers—task executor/queue/sink from
   platform/runtime/command, state machinery from runtime/timeline, feedback
   stack from draft/session, icon fonts from text, and command internals from
   runtime/UI—while Rung 6 still owes the symbol-level future-workspace
   visibility disposition. The one-line root error facade remains the sole
   intentional compatibility surface admitted by R1-08; no retired Rung 1
   path receives new behavior.
4. **Admission and correction.** OS thread creation failure is absence of
   execution capacity, not proof of a programming invariant. A new public
   error or platform callback would enlarge the seam unnecessarily because the
   existing executor contract already returns whether it accepted a job and
   the native runner already cancels rejected task ids. Executor construction
   now retains every worker that starts, logs individual failures, and removes
   its sender when zero workers exist. Partial capacity continues asynchronously;
   zero capacity rejects immediately and uses the existing cancellation path.
   It never runs deferred work on the UI thread or leaves work buffered without
   a receiver.
5. **Proof and ratchet.** New owner tests simulate zero and partial startup
   capacity. The existing exact worker-thread witness, 22 focused task
   witnesses, and the strengthened architecture witness pin named workers,
   nonpanic startup, zero-capacity rejection, event-loop proxy completion, and
   cancellation after rejection. Full library: 1,065 passed, 10 ignored, 0
   failed; all five examples and all targets compiled without warnings;
   formatting and diff checks passed. The task doctrine now records the
   failure law. Renderer topology, presentation clocks, and frame economics
   are inapplicable; the successful full-capacity execution path is unchanged.
6. **Gauge delta.** Production `.expect(...)` calls fall 103 -> 102. Production
   and test module edges, slot edges, forbidden edges, external questions, the
   SCC, visibility, test-edge counts, source-root reads, filesystem reads,
   allowances, and production panics are unchanged.
7. **Fixed point.** Rung 1's one operational expect is retired; every remaining
   hygiene finding in the examined slices has an owner and disposition. The
   task owner keeps a bounded asynchronous pool under partial capacity and an
   honest rejection result under zero capacity. Cell closed.

### R1-10 — window facts versus scene-owned color housing

Status: **complete; remaining window projections routed**. Correction
`ef0d9363` (`Move semantic color to its lower owner`).

1. **Question and trace.** `window::Id`, `Facts`, `Options`, `Kind`,
   `PresentationEpoch`, and `Departed` were traced through session, shell,
   host, platform, semantic presentation, diagnostics, cleanup publication,
   public defaults, and theme selection. Every layer wraps `Facts`; the fact
   owner remains authoritative for id, title, inner size, canvas color, and
   kind. The direct `window -> scene` edge existed only because the sRGB-byte
   `Color` value was physically declared under scene while all of its transfer,
   packed-byte, and boundary-conversion laws already lived in the lower
   `color` owner.
2. **Current and proposed graph.** Scene construction and window facts were
   peers sharing the same semantic color datum, but housing the value in scene
   forced window upward and helped form the provisional UI/foundation cycle.
   A new color type or renamed public path would create parallel vocabulary.
   The admitted graph instead makes existing `color` own the byte value and
   conversion laws; scene and window consume it. `scene::Color` remains the
   exact established public re-export, so application API and value identity do
   not change.
3. **Reduction and rewire.** Moved the 21-line `Color` declaration from
   `scene/color.rs` into `color.rs`, deleted the old module, and made scene
   re-export the lower value. Window facts and options now name the lower owner
   internally. Constructors, channels, equality, copy semantics, theme tokens,
   scene brushes, renderer conversion, canvas clears, and public
   `scene::Color` call sites are unchanged. No new module, compatibility type,
   or name was introduced.
4. **Remaining window projections.** The two surviving window back-edges are
   not laundered into this correction. `window::DEFAULT_CANVAS_COLOR` and the
   default selected by `Options` intentionally project the theme-owned token;
   lowering the bytes into window would violate settled theme ownership. Their
   physical facade/housing disposition belongs to the Rung 5 UI examination.
   `window::Departed` is a domain past-tense fact implemented through the
   command notification contract; moving it behind a callback or duplicating
   cleanup would be worse. Its trait/realization boundary travels with the full
   Rung 3 notification and responder trace. `Id`, `Facts`, `Kind`, and
   `PresentationEpoch` remain the lower window core. The provisional
   top-level-module map therefore stays explicitly split-pending rather than
   claiming the entire module is purified.
5. **Proof and ratchet.** The lower owner test pins the scene re-export and
   absence of `scene::Color` from window facts/options. Existing default-token,
   window-fact uniqueness, color conversion, theme parsing, canvas, native
   paint, and renderer color witnesses passed: 24 focused color tests and 44
   focused window tests. Full library: 1,066 passed, 10 ignored, 0 failed; all
   targets compiled without warnings; formatting and diff checks passed.
   Renderer topology, shaping, batching, presentation clocks, and frame
   economics are unchanged because the Rust value and every consuming route
   are identical.
6. **Gauge delta.** Provisional forbidden edges fall 15 -> 14 by deleting
   `window -> scene`. Production module edges rise 322 -> 323 because the gauge
   now sees both truthful lower dependencies, `scene -> color` and
   `window -> color`, in place of the one false peer dependency. Slot edges,
   the existing SCC, external questions, visibility, test edges, source-root
   and filesystem reads, allowances, panics, and expects are unchanged.
7. **Fixed point.** Semantic sRGB color has one lower owner and scene is a
   public consumer, not its implementation home. Window facts no longer import
   scene. The theme-default and departure-notification projections remain
   visible, named questions at their proper later rungs. Cell closed.

## Rung 1 closure — lowest vocabulary

Status: **complete**. Production boundary `ef0d9363`; doctrine/ledger boundary
`f01a3f72`. The repository was clean at the boundary, remained 24 commits ahead
of `origin/master` and not behind, and preserved
`comparison_open: true`.

Rung 1 exercised the complete loop across ten cells. It moved winit schedule
projection and OS click metrics to platform adapters; recorded document atomic
replacement as an explicit persistence-owned external exception; admitted
independent icons, tasks, state, and feedback owners; removed redundant
feedback storage; moved command failure to command while preserving facade
paths; corrected worker startup failure through the existing rejection and
cancellation contract; and unified the semantic color datum with its existing
lower owner. No physical crate or feature gate was introduced.

### Boundary gauge

| Metric | Rung 0 | Rung 1 |
|---|---:|---:|
| Top-level production modules | 45 | 45 |
| Unique production module edges | 325 | 323 |
| Unique test-only module edges | 95 | 97 |
| Provisional cross-slot edges | 30 | 43 |
| Provisional forbidden internal edges | 15 | 14 |
| Provisional heavy external-boundary violations | 8 | 2 |
| Provisional slot SCCs | 1 | 1 |
| Production `pub(crate)` declarations | 1,738 | 1,737 |
| Cross-slot test-only edges | 75 | 77 |
| `CARGO_MANIFEST_DIR` mentions | 100 | 104 |
| Filesystem read calls | 288 | 298 |
| `#[allow(...)]` attributes | 10 | 10 |
| Production `panic!` calls | 9 | 9 |
| Production `.expect(...)` calls | 103 | 102 |

The increase in slot edges is truthful map resolution, not added coupling:
icons, tasks, state, and feedback left the false foundation bucket and now show
their actual independent consumers. Those four owners remain outside the
provisional SCC. The source-root and filesystem-read increases are narrow
architecture receipts already assigned to the Rung 6 workspace witness seam;
none is production I/O.

### Updated frontier

- `text -> paint` and the scene `glyphon` use enter Rung 2's complete
  text/geometry/paint trace; neither is pre-judged as a move or exception.
- notification/responder placement, including `window::Departed`, enters Rung
  3 with the whole command/service chain.
- renderer diagnostics and platform `wgpu` ownership enter Rung 4 with
  presentation realization and observation.
- the theme-owned window default, layout/view diagnostics, and widget/document
  projection remain named Rung 5 UI questions.
- all ten allowances, nine remaining production panics, retained invariant
  expects, and concrete crossing visibility have named rung dispositions; Rung
  6 still owes the final symbol-level and workspace-ready audit.

The accepted provisional map now contains foundation plus independent icons,
feedback, tasks, and state owners beneath the still-disputed text, command, UI,
renderer, runtime, platform, and facade territories. The map remains a gauge:
Rungs 2–5 may admit more owners or merge provisional territories according to
their complete traces.

### Boundary proof

- full library: 1,066 passed, 10 ignored, 0 failed;
- all targets and all five examples compiled without warnings;
- 24 focused color and 44 focused window witnesses passed at the final cell;
- all six census parser witnesses passed;
- `cargo fmt --check` and `git diff --check` passed;
- the final census reproduced every metric above.

Rung 2 begins with the direct `text -> paint` edge, tracing each imported type
by coordinate space, clock, shaping authority, renderer consumer, and public
surface before moving anything.

## Rung 2 cell records

### R2-01 — renderer-neutral coordinates versus paint policy

Status: **complete**. Correction `efe641f4` (`Move shared coordinates to
geometry`).

1. **Question and trace.** The direct `text -> paint` edge consisted entirely
   of floating logical area/point facts used by field layout, text-area
   observation, shaping bounds, hit testing, caret projection, reveal, and
   scrolling. The same paint declarations also crossed renderer preparation,
   canvas sizing, filters, native popup geometry, monitor-scale projection, and
   surface realization. Geometry separately owned an integer `Point` and a
   public `LogicalArea` carrying the same width/height facts. The trace covered
   logical/physical conversion, fractional-scale snapping, layout-to-paint
   conversion, native surface minimums, popup scaling, rounded geometry,
   selection/hit paths, renderer batches, and the public text projections.
2. **Current and proposed graph.** Paint already depends on text because its
   flattened display list carries prepared text values, so text importing paint
   coordinates formed a real peer cycle. Moving the entire paint geometry
   vocabulary downward would be equally false: device-grid `Grid`, rounded
   paint `Rect`/`Rounding`, and flattened paint items are renderer-ready policy
   and representation. The admitted graph puts only renderer-neutral area and
   point unit facts in geometry; text and paint both consume them, and paint
   retains its actual rendering vocabulary.
3. **Naming and API ruling.** The coordinate move applies the established
   house law rather than preserving compound compatibility names. Public unit
   species are `geometry::area::Logical` and `geometry::point::Logical`;
   `point::Point` remains the module's same-named central type and is its sole
   parent projection as `geometry::Point`. No `LogicalArea`, `LogicalPoint`,
   `SurfaceArea`, or `SurfacePoint` alias survives. Call sites import the
   supporting modules and qualify their simple types. The campaign constitution
   and master API doctrine now state the same rule for later cells.
4. **Reduction and rewire.** Deleted `paint/area.rs` and `paint/point.rs`, moved
   the live logical/physical area conversions and logical point fact into
   geometry, and rewired text, paint, render, and native consumers. Deleted the
   one-use geometry-to-paint native area translation, unused physical-point
   conversion machinery, unused area clamps, the two text surface aliases, and
   dead paint-rounding constructors/accessors exposed by the move. Text's
   unconsumed public `ObservedArea` no longer names private paint `Rect`; it
   carries its existing text `Viewport` plus a geometry origin without changing
   the observation calculations.
5. **Proof and ratchet.** The architecture witnesses pin the absence of paint
   area/point modules, preserve private paint policy modules, restrict paint
   imports to rendering consumers, require the `point::Point` parent projection,
   forbid compound coordinate aliases, and retain the old whole-bucket
   tombstone. Focused proof passed 131 text tests (2 standing ignores), 56 paint
   tests, and 123 renderer tests (8 standing ignores). Full library: 1,067
   passed, 10 ignored, 0 failed; all targets and all five examples compiled
   without warnings; formatting and diff checks passed. Existing snapping,
   batch/pass topology, presentation clocks, native popup geometry, and frame
   economics are unchanged because every consumer receives the same values in
   the same order.
6. **Gauge delta.** The forbidden `text -> paint` edge is deleted: provisional
   forbidden edges fall 14 -> 13 and slot edges fall 43 -> 42. Truthful shared
   dependencies (`text -> geometry`, `paint -> geometry`, and their existing
   consumers) raise production module edges 323 -> 325. The new architecture
   receipt raises test-only edges 97 -> 98, cross-slot test edges 77 -> 78, and
   filesystem reads 298 -> 299; source-root mentions remain 104. Explicit
   crate-private constructors and the physical area species raise production
   `pub(crate)` declarations 1,737 -> 1,742. External questions, the one
   provisional SCC, allowances, panics, and expects remain 2, 1, 10, 9, and
   102.
7. **Fixed point and next frontier.** Text contains no paint import, geometry
   contains no rendering reason for existence, and paint retains the policy
   that only its renderer/native consumers need. The text/paint coordinate
   cycle is closed. Rung 2 continues with renderer-ready scene grammar and the
   direct scene `glyphon` dependency, followed by the complete read-only text
   projection versus mutation/history/IME trace.

### R2-02 — shaped text ownership versus transit-layer renderer types

Status: **complete**. Correction `04202403` (`Keep shaped buffers behind
text`).

1. **Question and trace.** Scene's sole direct `glyphon` use was the shared
   buffer inside editable `TextSurface`; private paint repeated the same
   concrete field. Text layout creates and shapes that buffer, retains it in
   field and text-area caches, uses it for hit/highlight/reveal projection, and
   emits the same `Rc` through render surfaces. Scene adds semantic placement
   and presentation color, native lowering snaps the placement and converts the
   color, and render alone borrows the buffer for glyphon preparation. The
   trace covered field and area creation, cache hit/miss and color-only change,
   scene translation/grouping, popup lowering, batch ordering, per-batch
   viewports, render borrowing, and test inspection.
2. **Challenge and admission.** Moving shaping into scene or render would
   duplicate text authority and reshape cached surfaces; storing the whole
   `TextAreaSurface` in scene would drag source-line, hit-test, and geometry
   metadata through a consumer that needs none of it. A type alias would merely
   hide the dependency syntactically. A text-owned opaque handle is admitted
   because it makes the concrete renderer type unrepresentable in transit
   layers while preserving the shaped buffer's shared identity and borrow
   lifetime.
3. **Reduction and rewire.** Added crate-private
   `text::layout::ShapedBuffer`, a one-field shared handle whose only downstream
   operation is borrowing at render. Text surfaces project that handle; scene
   and paint retain their existing viewport/surface grammar but store the
   handle instead of `Rc<RefCell<glyphon::Buffer>>`. No buffer is copied or
   reshaped. Removed the now-unused `Rc`/`RefCell` imports from scene and paint,
   and narrowed the unconsumed public `TextAreaSurface::buffer` escape hatch to
   text-internal access. Rectangles, color conversion, text ordering, and
   viewport grouping are unchanged.
4. **Proof and ratchet.** The new architecture witness requires the text-owned
   handle, forbids a public glyphon-buffer accessor, and proves all scene and
   paint sources are free of `glyphon` paths. The existing cache-identity,
   current-color, editable-text scene, native paint conversion, popup, batching,
   per-batch viewport, and renderer witnesses passed in the full library:
   1,068 passed, 10 ignored, 0 failed. All targets and all five examples
   compiled without warnings; formatting and diff checks passed. Renderer
   topology and economics are identical: the same `Rc` clone reaches the same
   batch, is borrowed once at preparation, and is never reshaped at a transit
   boundary.
5. **Gauge delta.** The `scene -> glyphon` heavy-boundary question is closed:
   provisional external violations fall 2 -> 1, and the external-user set is
   now exactly text plus render. Production/test module edges, slot edges,
   forbidden edges, SCCs, and cross-slot test edges remain 325/98, 42, 13, 1,
   and 78. The four explicit handle/crossing declarations raise production
   `pub(crate)` declarations 1,742 -> 1,746. The architecture receipt raises
   source-root mentions 104 -> 105 and filesystem reads 299 -> 302. Allowances,
   panics, and expects remain 10, 9, and 102.
6. **Fixed point and next frontier.** Text owns both shaping mechanics and the
   concrete shaped output representation; scene and paint transport an opaque
   capability; render is the sole downstream concrete consumer. No concrete
   glyphon dependency remains in semantic scene or private paint grammar. Rung
   2 now continues with the complete read-only selection/caret projection
   versus mutation, history, draft, and IME machinery trace.

### R2-03 — read-only text projection versus mutation capability

Status: **complete; four independently green production checkpoints**.
Vocabulary correction `979878ef` (`Separate read-only text vocabulary`);
operation correction `60fff63d` (`Separate selection operations from text
mutation`); action reduction `bf01aef4` (`Remove text action clipboard
intermediate`); composition correction `20d02bb4` (`Separate preedit projection
from view state`).

1. **Question and trace.** Text buffer state, caret motion, selection,
   field-mode capability, surface projection, view state, direct pointer/key
   operations, document commands, draft routing, clipboard actions, history,
   and composition were traced through editable documents, local text-box and
   table-cell drafts, read-only selection, disabled fields, keyboard and
   pointer entrances, visual caret maps, reveal/blink updates, rejection,
   preedit clearing, undo/redo, and example responders. The live code had
   placed always-present caret/selection/surface/view vocabulary under
   `text::edit`, and one `Edit` sum mixed pure selection operations with text
   mutation.
2. **First reduction and naming ruling.** Caret maps, motion, selection state,
   surface capability/projection, and view vocabulary now live at first-class
   text modules that do not depend on mutation. The parent follows the house
   law: `text::Edit`, `text::Surface`, and `text::View` are the sole
   same-named central projections, while supporting concepts remain qualified
   through `text::edit`, `text::surface`, `text::selection`, and `text::view`.
   No compound compatibility declaration or old mutation-owned projection was
   retained.
3. **Operation reduction and rewire.** `text::selection::Operation` now owns
   move, extend, select-all, direct-position, and pointer selection grammar,
   with one selection-owned application path and `PointerKind`. `text::Edit`
   contains mutation only. Input, view actions, keymap, document routing,
   drafts, focused services, runtime, and the text-editor example carry the
   distinction structurally. Visual motion still consumes the existing caret
   map; selection never enters mutation history; mutation retains the exact
   buffer transaction and diagnostic path. The redundant public compound
   spelling was not propagated: framework call sites use parent-projected
   `text::Edit`, while `text::edit::Editor` and `text::edit::History` remain
   namespaced supporting machinery.
4. **Action and clipboard reduction.** The complete action trace found no
   independent text-library owner for `text::edit::Action`, its clipboard
   trait/adapter, `ActionResult`, or `ActionOutcome`. Document commands and
   focused drafts already own the real Copy/Cut/Delete/Paste/SelectAll
   contracts; production history never consumed the action route. Those two
   source files, the adapter, the editor branch, the public
   `Document::apply_action` intermediate, and duplicated low-level command
   tests were deleted. Document command targets now call selection, mutation,
   and framework clipboard contracts directly while preserving confirmed-write
   Cut, empty-versus-failed Paste, availability, and outcome laws. History's
   real result is now `text::edit::history::Outcome`; `History` is its only
   parent projection, the coalescing constant stays namespaced, and the old
   compound `HistoryKind` declaration collapsed to private `history::Kind`.
5. **Composition ownership and duplicate reduction.** `text::view::ViewState`
   no longer stores optional IME state. The immutable composition value now
   lives at same-named `text::preedit::Preedit` with `text::Preedit` as its sole
   parent projection; draft input remains the only owner of target identity,
   replacement, clearing, and retirement. The layout bridge passes the active
   projection separately through six crate-private compose/reveal/hit
   crossings, while ordinary public layout remains committed-text-only. The
   complete producer/consumer trace also found that
   `Surface::presentation_text_for_state` and its Field/Area implementations
   duplicated the real `PreeditProjection` composition algorithm without a
   production consumer. Those methods and their second string-composition
   helper were deleted, including the separately repeated obscured-field path.
6. **Behavior and lifecycle preservation.** Editable mutation, read-only
   selection, disabled-field rejection, stale table-draft rejection,
   single/double/triple click and drag, Unicode motion, selection collapse,
   reveal, caret blink, clipboard behavior, draft validation, feedback
   clearing, undo/redo, IME preedit/commit/disable, popup-to-parent routing,
   cancel, focus/menu transitions, snapshot restore, removed-target pruning,
   and application revision versus document-dirty clocks retain their existing
   paths. Focused owner witnesses prove selection clears active preedit without
   changing text or entering mutation history, numeric policy never filters a
   composition, and obscured fields consume the one composed projection.
7. **Proof and ratchet.** Architecture witnesses require first-class
   always-present modules, the central parent projections, absence of old
   compatibility paths, distinct selection/mutation input and command types,
   a selection-only pointer view action, a private typed keymap sum, and the
   extinction of the text action/clipboard intermediate. The composition
   ratchet additionally requires the same-named preedit owner/projection,
   forbids any preedit state or compatibility path in `text::view`, pins draft
   input as lifecycle owner, and requires explicit layout consumption. Fourteen
   focused preedit witnesses passed. Full library: 1,069 passed, 10 ignored, 0
   failed; all targets compiled without warnings; formatting and diff checks
   passed. Shaping, hit testing, renderer topology, batching/pass fusion,
   presentation clocks, and frame economics are unchanged because the same
   buffers, states, positions, and resolved operation order reach the same
   owners.
8. **Gauge delta from R2-02.** Production module edges fall 325 -> 324 while
   test-only edges rise 98 -> 99 and cross-slot test edges 78 -> 79 from the
   strengthened command-boundary witness. Slot edges, forbidden edges,
   external questions, and SCCs remain 42, 13, 1, and 1. The four checkpoints
   net production `pub(crate)` declarations 1,746 -> 1,756; the final eight
   are the explicit framework-only composition crossings rather than a public
   state bag. Architecture receipts raise source-root mentions 105 -> 108 and
   filesystem reads 302 -> 318; Rung 6 retains their consolidation.
   Allowances, panics, and expects remain 10, 9, and 102.
9. **Fixed point and feature-seam result.** No selection, surface, or view
   owner depends on mutation machinery; no view state carries composition; no
   text action/clipboard route or duplicate composition algorithm survives.
   Draft input owns transient composition lifetime, layout owns its immutable
   projection, and ordinary committed-text layout has no composition in its
   public state or method signatures. A later text-mutation capability can
   therefore gate the upper lifecycle and explicit internal crossings without
   disturbing committed display, selection, shaping, or hit testing. Cell
   closed.

## Rung 2 closure — text, geometry, and paint

Status: **complete**. Production boundary `20d02bb4`; cell-ledger boundary
`bef2a9ed`. The repository was clean at the boundary, remained nine commits
ahead of `origin/master` and not behind, and preserved
`comparison_open: true`.

Rung 2 resolved three bounded ownership questions. Renderer-neutral logical
area and point facts moved to geometry while paint retained snapping and
renderer-ready policy; shaped glyphon buffers became opaque text-owned handles
through scene and paint; and the text subsystem separated selection/view
projection, mutation/history, command/clipboard ownership, and transient IME
lifecycle without a gate. The final re-scan found no text-to-paint import, no
glyphon path in scene or paint, no mutation import from selection/surface/
layout/view, and no higher framework dependency in geometry.

### Boundary gauge

| Metric | Rung 1 | Rung 2 |
|---|---:|---:|
| Top-level production modules | 45 | 45 |
| Unique production module edges | 323 | 324 |
| Unique test-only module edges | 97 | 99 |
| Provisional cross-slot edges | 43 | 42 |
| Provisional forbidden internal edges | 14 | 13 |
| Provisional heavy external-boundary violations | 2 | 1 |
| Provisional slot SCCs | 1 | 1 |
| Production `pub(crate)` declarations | 1,737 | 1,756 |
| Cross-slot test-only edges | 77 | 79 |
| `CARGO_MANIFEST_DIR` mentions | 104 | 108 |
| Filesystem read calls | 298 | 318 |
| `#[allow(...)]` attributes | 10 | 10 |
| Production `panic!` calls | 9 | 9 |
| Production `.expect(...)` calls | 102 | 102 |

Production edges rise by one because the former false text/paint cycle became
truthful shared dependencies on geometry; the forbidden text-to-paint edge and
the scene glyphon boundary question are both deleted. Visibility grew at named
text crossings rather than through a state bag: opaque shaped-buffer transit,
explicit selection application, and the eight framework-only composition
entrypoints. Rung 6 still owns their symbol-level future-workspace disposition.
The four source-root mentions and twenty filesystem-read calls are focused
architecture receipts assigned to the same Rung 6 witness consolidation.

### Boundary proof and next frontier

- full library: 1,069 passed, 10 ignored, 0 failed;
- all targets and all five examples compiled without warnings;
- fourteen focused preedit witnesses and the lower-owner architecture
  witnesses passed;
- `cargo fmt --check`, `git diff --check`, tombstone searches, and the full
  census passed;
- renderer order, batching/pass fusion, shaping identity, hit testing,
  presentation clocks, and frame economics retain their existing routes.

Rung 3 begins with the complete command-context and responder capability trace,
starting at the concrete clipboard/layout crossings but following input,
keymap, targets, focused text, tasks, notifications, and effects through every
consumer before admitting any lower contract.

## Rung 3 cell records

### R3-01 — command invocation environment versus concrete services

Status: **complete; clipboard owner admitted**. Text-capability correction
`019eb330` (`Narrow command text context to caret mapping`); clipboard and
construction correction `f207e488` (`Admit clipboard as an independent
capability`).

1. **Question and trace.** `context::Context` was traced through typed and
   erased command transactions, state resolution, authored bindings,
   conventional bars, context menus, the palette, direct text drops, responder
   and framework-service claims, application targets, document editing, focused
   drafts, observers, and task-producing examples. The trace covered every
   `Source`, sourced state clones, configured and absent clipboards, confirmed
   empty and failed reads, confirmed and failed writes, accepted and rejected
   tasks, missing task sinks, semantic and visual caret motion, missing visual
   mapping, and the retained response/effect/history paths.
2. **Current graph.** Context owned no service engine, but its representation
   said otherwise: it stored the concrete UI `layout::TextService`, stored the
   clipboard capability under the provisional runtime owner, and constructed
   clipboard, task sink, and text layout through one aggregate
   `with_services_source` route. Document selection consumed only
   `text::selection::CaretMap`; clipboard targets cloned the already-shared
   handle for every operation; tasks consumed only the tasks-owned acceptance
   sink.
3. **Admission.** Context remains the command-owned invocation environment and
   may depend on lower capabilities, but it may not contain their engines.
   `text::selection::CaretMap` is an honest narrow contract: it names one text
   question, returns absence coherently, and has the command invocation
   lifetime. Clipboard is admitted as an independent owner of representations,
   synchronization results, shared handle identity, and its optional system
   realization. The previously admitted tasks owner remains intact; `Sink` and
   `Option<Id>` already express acceptance and rejection without exposing the
   queue or executor.
4. **Reduction and rewire.** Context now stores a shared trait object for caret
   mapping rather than `layout::TextService`; the UI layout service projects
   that capability from the identical shared text engine. Document visual
   selection borrows it only for the operation. The redundant
   `clipboard_mut` clone path is deleted and every document/focused-text
   operation borrows the one handle. The aggregate services constructor is
   deleted; invocation contexts compose clipboard, tasks, and caret mapping
   explicitly. No new wrapper, alias, service locator, or callback surface was
   added.
5. **Resistance and optionality.** A narrower clipboard wrapper is rejected:
   `Clipboard` already is the typed capability and another value would repeat
   its failure and lifetime model. Splitting its in-memory contract from the
   arboard realization inside this cell is also rejected; the independent
   owner already isolates that dependency, while an optional system-adapter
   gate belongs to Rung 6 formulation. A new task contract is likewise
   rejected. Existing command behavior that treats clipboard absence as
   unavailable—including the current Select All and Delete paths—was
   deliberately preserved rather than opportunistically corrected.
6. **Behavior and economics.** All state-query and invocation routes retain
   their sources, claims, fallthrough, outputs, effects, history, and
   invalidation. Clipboard still distinguishes absence, empty, failure, and
   confirmed publication; Cut still deletes only after a successful write.
   Visual motion reaches the same `text::layout::Engine` and performs the same
   shaping/cache work. Task acceptance and cancellation are unchanged.
   Clipboard operations now avoid a redundant `Rc` clone; renderer topology,
   batching/pass fusion, presentation clocks, and frame economics are
   otherwise inapplicable or unchanged.
7. **Proof and ratchet.** A direct target witness proves that Context supplies
   visual motion through the caret-map contract, and a default-context witness
   pins clipboard, caret-map, and task-sink absence. Architecture witnesses
   forbid concrete layout service vocabulary and the retired aggregate
   constructor, require explicit task-sink construction, require one borrowed
   clipboard route, and keep the independent clipboard owner free of framework
   imports. Clipboard failure/empty/confirmed-write, focused draft transfer,
   task execution/rejection, and command routing witnesses all passed.
8. **Full verification.** The closing library run discovered 1,082 tests:
   1,072 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; formatting, diff checks, the census,
   and the protected `comparison_open: true` state passed.
9. **Gauge delta from Rung 2.** Production/test module edges remain 324/99.
   Admitting the truthful clipboard consumers raises slot edges 42 -> 44 while
   the independent owner remains outside the SCC. Removing `context -> layout`
   and admitting `context -> clipboard` lower forbidden edges 13 -> 11.
   External questions, SCCs, production `pub(crate)`, cross-slot test edges,
   allowances, panics, and expects remain 1, 1, 1,756, 79, 10, 9, and 102.
   The architecture receipts raise source-root mentions 108 -> 109 and
   filesystem reads 318 -> 323; Rung 6 retains their consolidation.
10. **Fixed point and next frontier.** Context names source plus three explicit
    lower capabilities and no concrete UI/runtime service. Clipboard, tasks,
    and text each retain one owner and one failure model; no aggregate service
    bag survives. Rung 3 continues with responder identity and scope, tracing
    the `responder -> interaction/session/table` crossings before touching
    notification placement or effect consumption.

### R3-02 — responder routing identity versus UI service scope

Status: **complete; shared authored identity lowered**. Correction `43bcd3f0`
(`Separate responder routing from UI scope`).

1. **Question and trace.** Responder identity and scope were traced through
   builder registration, exact and broad claims, path traversal, focused and
   application routing, typed and erased command state/invocation, context-menu
   sections, palette capture/query routing, standard bars, focused text and
   table services, direct text drops, and notification delivery. Focus species
   included text, table cell, and control; scope kinds included Focused,
   Transient, and Captured; contextual paths covered task and inspection
   traversal with exact service routes.
2. **Current and proposed graph.** The lower `responder::Scope` stored both
   routing facts and higher `session::Focus`/table facts, forcing responder to
   import interaction, session, and table. The same authored identity datum was
   declared under interaction even though reconciliation and responder routing
   consume it independently. The admitted graph places the unchanged `Id`
   declaration in private lower housing, leaves `interaction::Id` as its public
   projection, keeps `responder::Scope` route-only, and lets private
   `session::CommandScope` align that route with optional focus and table facts
   for runtime service realization.
3. **Admission.** Stable authored identity is dependency-free vocabulary with
   independent UI and command-routing consumers, so it belongs in the existing
   foundation responsibility; it does not earn another virtual crate. The
   higher command scope is not a transport wrapper: its invariant derives the
   same responder/table identity from focus and prevents the service question
   from drifting from the route used to claim and invoke. Erasing the UI facts
   behind callbacks or keeping them in responder would conceal the upward
   dependency instead of removing it.
4. **Naming and API ruling.** The declaration remains the simple canonical
   `Id`; no compound declaration, alias, or second public root is introduced.
   `interaction::Id` re-exports that exact declaration and the former
   `interaction/id.rs` path is tombstoned. This preserves the established
   application spelling while applying the house law through the private
   ownership move.
5. **Reduction and rewire.** `responder::Scope` now contains only optional
   authored identity plus `responder::Kind`; builder chains accept that identity
   directly. Session constructs `CommandScope` for focused, transient,
   captured, and contextual questions. Runtime services consume the higher
   scope, while responder paths consume its route projection. Context-menu
   section order is still built once from the same broad-to-exact source list;
   text drops and notifications project only the focus identity they need.
6. **Behavior and economics.** Claim precedence, disabled-claim stopping,
   exact service invocation, task/inspection order, captured palette focus,
   context-menu table/text ownership, live standard-menu state, notification
   listener order, and text-drop cleanup are unchanged. No allocation, shaping,
   paint, renderer, presentation-clock, or frame path changed; the new scope is
   a small Copy value replacing the same fields previously embedded in the
   responder value.
7. **Proof and ratchet.** The architecture witness requires one private lower
   declaration with the established interaction projection, tombstones the old
   declaration path, pins route-only responder fields and higher session
   alignment, and recursively forbids interaction, session, and table paths
   under responder. Focused responder, context-menu, palette, notification,
   text-drop, and path-traversal witnesses all passed.
8. **Full verification.** The library discovered 1,083 tests: 1,073 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; census parser witnesses, formatting, diff checks, and the
   protected `comparison_open: true` state passed.
9. **Gauge delta from R3-01.** Top-level modules rise 45 -> 46 because the
   shared declaration now has truthful root housing. Production edges remain
   324; test-only edges rise 99 -> 100. Slot edges rise 44 -> 45 from the now
   visible lower identity dependency, while forbidden edges fall 11 -> 8 by
   deleting all three `responder -> interaction/session/table` crossings.
   External questions and SCCs remain 1/1. Explicit scope crossings raise
   production `pub(crate)` declarations 1,756 -> 1,764 in 191 files;
   cross-slot test edges rise 79 -> 80. The ratchet raises source-root mentions
   109 -> 110 and filesystem reads 323 -> 328. Allowances, panics, and expects
   remain 10, 9, and 102.
10. **Fixed point and next frontier.** Responder now depends only on command and
    lower vocabulary; no UI service fact or focus type enters its directory.
    Session owns the one aligned higher projection and runtime consumes both
    levels explicitly. Rung 3 continues with notification placement and the
    `window -> notification` crossing, tracing departure publication and
    effect consumption before admitting any move.

### R3-03 — window departure fact versus notification binding

Status: **complete; domain fact retained at its owner**. Correction `947eb32b`
(`Move departure binding to notification`).

1. **Question and trace.** `window::Departed` was traced from session close and
   its single-drain queue through started/event/transaction delivery, ten
   runtime cleanup listeners, responder-ordered application listeners, state
   commit and redraw reaction, shell/host closed-window projection, native
   popup/IME/cursor purge, duplicate and missing close, and snapshot restore.
   Document cancellation facts and generic notification absence, ordering,
   effect, and history laws were compared as sibling cases.
2. **Current and proposed graph.** Window owned the correct pure fact and
   payload meaning, but its source file also implemented the command-owned
   `Notification` trait. That one binding created the forbidden
   `window -> notification` edge. Moving the fact itself would make generic
   notification machinery own a window-domain name; a facade impl would fail a
   later cross-crate orphan check. The trait owner can lawfully bind a lower
   domain type, so notification now owns a private window binding and depends
   downward on the unchanged fact.
3. **Admission and naming.** The split is admitted because declaration and
   protocol conformance have different stable owners and the future physical
   command crate may implement its own trait for the foundation-owned type.
   `window::Departed`, its simple declaration name, payload `window::Id`, and
   stable `window.departed` name remain exact; no alias or second public
   projection is introduced.
4. **Reduction and rewire.** `window/departed.rs` is now the pure two-line fact.
   Private `notification/window.rs` contains the sole generic delivery impl.
   The queue, publisher, internal listener registry, responder registrations,
   platform listener, and all call sites are byte-for-byte unchanged.
5. **Resistance.** The centralized runtime listener array is retained: it is
   the one explicit registry over runtime-owned per-window stores, while each
   store still owns its purge implementation. Replacing it with callbacks,
   registration objects, or another cleanup trait would add machinery without
   deleting authority. Snapshot restore remains a distinct whole-runtime reset,
   not a series of false departure events.
6. **Behavior and economics.** One successful close still emits once; missing
   and duplicate closes remain inert; internal cleanup precedes application
   delivery; zero-to-many listeners retain chain order; only changed app
   reactions commit notification reason and request redraw. No allocation,
   render, presentation, or frame path changed.
7. **Proof and ratchet.** The existing per-window lifecycle witness now
   requires the pure window declaration, forbids notification vocabulary there,
   pins the private trait-owner binding, and retains every cleanup listener and
   native purge. Focused notification and departure witnesses passed.
8. **Full verification.** The library discovered 1,083 tests: 1,073 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; all six census parser witnesses, formatting, diff checks,
   and protected example state passed.
9. **Gauge delta from R3-02.** Production/test module edges remain 324/100.
   The reversed module edge joins an existing legal command-to-foundation slot
   relation, so slot edges fall 45 -> 44 and forbidden edges fall 8 -> 7.
   External questions, SCCs, visibility, cross-slot test edges, source-root
   mentions, allowances, panics, and expects remain 1, 1, 1,764, 80, 110, 10,
   9, and 102. The strengthened witness raises filesystem reads 328 -> 330.
10. **Fixed point and next frontier.** Window declares its close fact without
    importing command machinery; notification alone binds that fact to generic
    delivery; runtime remains the one publisher. Rung 3 continues with the two
    remaining command/UI back-edges in `input`, tracing their full focus and
    target semantics before deciding whether the top-level module has one owner
    or split responsibilities.

### R3-04 — keyboard facts versus runtime input ingress

Status: **complete; input responsibility split and reclassified**. Correction
`61162f20` (`Separate keyboard facts from runtime input`).

1. **Question and trace.** The entire public `input` module was traced through
   native event translation, host and shell events, runtime dispatch and view
   routing, session pointer state, interaction targets and menus, focus,
   scrolling, shortcuts, keymap profiles, text selection/mutation/preedit/drop,
   file-dialog outcomes, command registry matching, and public tests/examples.
   The trace separated raw key/modifier facts from runtime ingress, handling
   outcomes, and compound payloads.
2. **Current and proposed graph.** The provisional command placement was false:
   `Input` is the event sum accepted by `Runtime::handle_input`, and therefore
   legitimately names UI focus/targets plus command and text operations. Only
   `Key` and `Modifiers` were independently consumed below runtime by command,
   keymap, and interaction state. Keeping them under input forced command to
   depend upward; moving all input into UI would merely reverse the same leak.
   The admitted graph places dependency-free keyboard facts in foundation and
   the remaining input module in runtime.
3. **Admission.** Keyboard facts have independent command, UI, runtime, host,
   and platform consumers and no higher reason for existence, but do not earn a
   standalone virtual crate; they join foundation responsibility. Runtime input
   has one sentence of ownership—ingress plus its outcome/payload values—and
   its dependencies match the executor that consumes it. No callback, erased
   service, or visibility widening is required.
4. **Naming and API ruling.** `Key` and `Modifiers` keep their simple canonical
   declaration names. Private `keyboard` housing is not a second public API;
   `input::Key` and `input::Modifiers` remain the exact established projections.
   The old `input/key.rs` path is tombstoned, and no compound compatibility name
   or alias survives.
5. **Reduction and rewire.** Command registration/spec matching, keymap policy,
   interaction pointer state, and session pointer projection now consume the
   lower declarations directly. Input publicly projects them while retaining
   `Input`, `Outcome`, and `TextDrop`. The provisional map moves input from
   command to runtime and adds keyboard to foundation. Every higher host,
   platform, runtime, and application call site retains its public spelling.
6. **Behavior and economics.** Key normalization, all platform profiles,
   standard and authored shortcuts, modifier propagation through parent and
   popup events, text motion/edit routing, focus/cancel, pointer gestures,
   scrolling, IME, drops, and file-dialog results are unchanged. The same Copy
   enums/struct cross the same runtime paths; no allocation, renderer,
   presentation-clock, or frame behavior changed.
7. **Proof and ratchet.** The new architecture witness requires one private
   lower declaration, the exact input projections, extinction of the former
   declaration path, retention of runtime-ingress variants, and absence of
   `input::Key`/`input::Modifiers` dependencies in command, keymap, interaction,
   and session pointer sources. Focused keymap, command, text-input, and
   host/shell suites passed.
8. **Full verification.** The library discovered 1,084 tests: 1,074 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; all census parser witnesses, formatting, diff checks, and
   protected example state passed.
9. **Gauge delta from R3-03.** Top-level modules rise 46 -> 47 and production
   edges 324 -> 325 as the one false input owner becomes truthful shared
   keyboard dependencies; test-only edges remain 100. Slot edges fall 44 -> 43
   and forbidden edges fall 7 -> 5, retiring both `input -> interaction/session`
   crossings without creating a command-to-runtime replacement. External
   questions, SCCs, visibility, and cross-slot test edges remain 1, 1, 1,764,
   and 80. The witness raises source-root mentions 110 -> 111 and filesystem
   reads 330 -> 334; allowances, panics, and expects remain 10, 9, and 102.
10. **Fixed point and next frontier.** Command meaning no longer imports runtime
    input or UI state; input's higher dependencies are truthful at runtime; raw
    keyboard facts have one lower owner. Rung 3 now re-scans command context,
    responders, input, keymap, targets, clipboard, tasks, text, notifications,
    and effects for remaining concrete-service, concealed-callback, ownership,
    visibility, and intermediate findings before declaring the rung complete.

### R3-05 — full command-boundary fixed-point audit

Status: **complete; no further correction admitted**.

1. **Question and sweep.** The re-scan walked command, context, responder,
   target, response, notification, keymap, timeline, runtime input, clipboard,
   tasks, text operations, and every runtime construction site in both
   directions. It challenged concrete engines, optional capability state,
   generic services, selectors/callbacks, type erasure, effect transport,
   duplicate routes, failure/absence, visibility, and retained intermediates.
2. **Command dependency result.** No command-owned source imports runtime,
   session, interaction, layout, view, table, or composition. `Context` carries
   source plus exactly clipboard, task acceptance, and caret mapping; each is a
   lower owned contract with an explicit absence/failure model. There is no
   aggregate constructor, concrete layout engine, runtime state bag, or
   arbitrary resource lookup.
3. **Responder-service ruling.** The private `responder::Service` trait is
   retained. It has one runtime implementation, exists only inside `Chain`, and
   accepts exactly the same typed-command erasure tuple used by targets:
   command identity/name, arguments, Context, state store, and typed
   Claim/Response outcomes. Its lifetime is the one chain transaction and its
   failures are command failures. It has no `get<T>`, ambient registration,
   arbitrary capability lookup, or callback-owned policy, so it is the honest
   cross-crate responder extension rather than a service locator.
4. **Other dynamic seams.** Target selectors are authored projections from the
   application model to registered targets, observer closures are the public
   post-command behavior, typed triggers retain their argument builders, and
   task Sink is the previously admitted tasks-owned acceptance capability.
   Each callback preserves an independent lifetime or type invariant; none
   hides an upward framework dependency. Exact `TypeId` checks precede the
   retained downcasts.
5. **Effects and intermediates.** `response::Effect` remains one command result
   contract. Its Batch representation is not transport-only: composition
   flattens nested batches, removes duplicates, preserves non-invalidation
   order, and collapses invalidation to the maximum depth. Runtime is the sole
   operational consumer for dialogs, floating-panel closure, and invalidation.
   Command population's private typed marker/candidate/resolved forms retain
   distinct surface membership and route freshness already pinned by command
   surface case law; merging them would erase those invariants.
6. **Visibility and hygiene.** Every crate-visible command crossing encountered
   has a live runtime/UI consumer; symbol-level future-workspace disposition
   remains explicitly assigned to Rung 6 rather than widened or guessed here.
   Retained expects are exact type/cardinality/registration invariants covered
   by tests; no Rung 3 production panic, allowance, concrete engine, retired
   alias, or compatibility route was discovered.
7. **Proof.** Source scans found one Service implementation and no prohibited
   higher import in command territory. Default/explicit Context absence,
   clipboard outcomes, task rejection, visual motion, responder precedence,
   exact contextual routes, notification ordering, effect associativity, key
   profiles, and all input paths remain covered. The R3-04 full proof is the
   unchanged production boundary: 1,074 passed, 10 ignored, 0 failed; all
   targets/examples, census witnesses, format, and diff checks passed.
8. **Gauge.** No code or map changed: the R3-04 gauge remains 47 top-level
   modules, 325 production edges, 100 test-only edges, 43 slot edges, 5
   forbidden edges, 1 external question, 1 SCC, 1,764 production
   `pub(crate)` declarations in 191 files, 80 cross-slot test edges, 111
   source-root mentions, 334 filesystem reads, 10 allowances, 9 panics, and
   102 expects.
9. **Fixed point.** All Rung 3 seeded questions and the full re-scan are closed.
   No command meaning depends on a higher engine or UI state, every dynamic
   crossing has a named invariant, and no admissible reduction remains within
   the rung's bounds.

## Rung 3 closure — command meaning and service realization

Status: **complete**. Production boundary `61162f20`; cell-ledger boundary
`4e54cdb5`. The repository was clean at the pre-closure boundary, remained 19
commits ahead of `origin/master` and not behind, and preserved
`comparison_open: true`.

Rung 3 resolved four ownership cells and one full fixed-point audit. Command
Context now carries three explicit lower capabilities and no engine; clipboard
is an independent owner; responder routing identity is lower than session
service scope; window departure keeps its domain fact while notification owns
the trait binding; and raw keyboard facts are lower than runtime input ingress.
No physical crate, feature gate, public compatibility alias, generic service
locator, or behavior change was introduced.

### Boundary gauge

| Metric | Rung 2 | Rung 3 |
|---|---:|---:|
| Top-level production modules | 45 | 47 |
| Unique production module edges | 324 | 325 |
| Unique test-only module edges | 99 | 100 |
| Provisional cross-slot edges | 42 | 43 |
| Provisional forbidden internal edges | 13 | 5 |
| Provisional heavy external-boundary violations | 1 | 1 |
| Provisional slot SCCs | 1 | 1 |
| Production `pub(crate)` declarations | 1,756 | 1,764 |
| Cross-slot test-only edges | 79 | 80 |
| `CARGO_MANIFEST_DIR` mentions | 108 | 111 |
| Filesystem read calls | 318 | 334 |
| `#[allow(...)]` attributes | 10 | 10 |
| Production `panic!` calls | 9 | 9 |
| Production `.expect(...)` calls | 102 | 102 |

The module count reflects truthful private housing for shared authored identity
and keyboard facts, not new virtual crates. The one additional production edge
is the net result of exposing those shared lower dependencies while deleting
eight forbidden crossings: concrete layout from Context, all three responder
UI dependencies, window-to-notification binding, and both false command-owned
input edges. The slot-edge increase of one resolves independent clipboard and
lower-vocabulary consumers; clipboard, tasks, state, feedback, and icons remain
outside the SCC. Architecture receipts account for the test/source-read growth
already assigned to the Rung 6 workspace witness seam.

### Boundary proof and next frontier

- full library: 1,074 passed, 10 ignored, 0 failed;
- all targets and all five examples compiled without warnings;
- focused command context, responder, context-menu, palette, notification,
  keymap, input, task, clipboard, text-motion, and host/shell witnesses passed;
- all census parser witnesses, `cargo fmt --check`, `git diff --check`, and
  tombstone scans passed;
- renderer order, batching/pass fusion, shaping, presentation clocks, and frame
  economics are unchanged because no rendering or presentation route moved.

Rung 4 begins with semantic presentation versus physical realization. It first
traces the remaining `render -> diagnostics` observation back-edge and
`platform -> wgpu` external-boundary question together with scene lowering,
surface lifecycles, presentation clocks, and backend consumption before
admitting either inversion.

## Rung 4 cell records

### R4-01 — renderer facts versus diagnostic aggregation

Status: **complete; diagnostics observer seam admitted**. Correction
`eb5d6143` (`Invert render diagnostics ownership`).

1. **Question and paired trace.** The remaining renderer-diagnostics edge was
   traced together with the public Backend receipt, native parent presentation,
   popup presentation, surface acquisition, draw preparation, GPU submission,
   successful and skipped presents, runtime acknowledgement, presented
   geometry, key-to-present sampling, and the direct/headless backend tests.
   The paired `platform -> wgpu` question was also enumerated across backend
   selection, surface formats, alpha capability, safe window targets, and the
   Windows composition-visual target, but was not collapsed into this smaller
   ownership correction.
2. **Current and proposed graph.** Renderer preparation produced
   `DrawStats`, yet that value and the complete render-attempt receipt were
   declared inside diagnostics, forcing render to import its observer. The
   admitted graph makes renderer own its output facts and makes diagnostics an
   explicit observer seam above renderer, text, UI, and command facts. Runtime
   and platform may consume that observer without making diagnostics a
   behavior input.
3. **Naming and API ruling.** The public established path remains
   `diagnostics::RenderReport`, but the declaration is now canonically named
   `RenderReport` at its renderer owner and is re-exported exactly. The former
   `Report as RenderReport` parent alias is deleted in accordance with the
   house naming law. Public constructors and accessors retain their signatures;
   no compatibility declaration or second report type survives.
4. **Reduction and rewire.** Moved the private draw facts and the exact report
   declaration into `render::report`, deleted `diagnostics/draw.rs`, and made
   diagnostics aggregation consume the renderer-owned receipt. Renderer no
   longer imports diagnostics. The native adapter still composes the same
   acquire, batch, draw, encode/submit/present, pool, and presented facts at the
   same boundary, and the public Backend trait still returns the established
   diagnostics projection.
5. **Observer-seam ruling.** A callback or diagnostics-owned trait was rejected:
   either would leave the producer depending on the observer or conceal the
   same fact crossing. Diagnostics is admitted as its own provisional virtual
   owner because it owns counters and sample windows while consuming facts
   declared by their producers. Existing layout/view diagnostic back-edges
   remain visible Rung 5 questions rather than being legalized by the new slot.
6. **Behavior, clocks, and economics.** Successful presents acknowledge the
   same epoch and promote the same candidate layout; skipped, occluded,
   outdated, timeout, validation, and lost attempts retain visible geometry and
   retry behavior. Every duration and renderer count reaches the same sample
   window. Scene order, batch/pass fusion, geometry uploads, filter pools,
   shaping, surface acquisition, submission, and popup presentation paths are
   unchanged; the correction only relocates the unchanged Copy receipt.
7. **Proof and ratchet.** A structural witness recursively forbids diagnostics
   imports under render, requires renderer ownership of `RenderReport` and
   `DrawStats`, requires the exact public diagnostics projection, and
   tombstones the old draw file and alias. The older renderer-boundary witness
   was narrowed by vocabulary: private paint remains confined to rendering
   paths, while diagnostics alone may observe the render module.
8. **Full verification.** The library discovered 1,085 tests: 1,075 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; all census parser witnesses, formatting, diff checks, and
   protected `comparison_open: true` state passed.
9. **Gauge delta from Rung 3.** Production edges remain 325. The architecture
   witness raises test-only edges 100 -> 101. Giving diagnostics its truthful
   observer slot raises slot edges 43 -> 51 and cross-slot test edges 80 -> 82,
   while the admitted inversion lowers forbidden edges 5 -> 4. External
   questions and SCCs remain 1/1. The ten explicit renderer-to-observer fields
   raise production `pub(crate)` declarations 1,764 -> 1,774; source-root
   mentions rise 111 -> 112 and filesystem reads 334 -> 336. Allowances,
   panics, and expects remain 10, 9, and 102.
10. **Fixed point and next frontier.** Renderer output facts have one owner and
    diagnostics is consumption only; neither a duplicate report nor an
    observer import remains in render. Rung 4 continues with the large semantic
    scene-to-paint projection currently housed under the native OS adapter,
    while retaining the separately traced wgpu surface bridge for its own
    bounded cell.

### R4-02 — semantic scene lowering versus native realization

Status: **complete; renderer projection owner established**. Correction
`657c3752` (`Move semantic scene lowering to renderer`).

1. **Question and trace.** The complete semantic-scene-to-private-paint path
   was traced from parent and popup presentation through scale selection,
   clear-color conversion, quads, rules, text, icons, shadows, panes, clips,
   outlines, groups, material resolution, popup visual reach and translation,
   unchanged-scene suppression, composition-region projection, and final
   renderer consumption. The trace covered all four supported scale witnesses,
   parent versus composition-backed popups, exposed/fresh/stale submissions,
   resolved glass and refraction values, native material fidelity, and the
   retained candidate/prepared/submitted/presented/committed clocks.
2. **Current and proposed graph.** A 1,322-line renderer grammar conversion and
   its color bridge were declared under `platform::native` despite containing
   no OS or window-system operation. Platform produced and cached the private
   display list while render only consumed it. The admitted graph makes render
   own the one semantic-to-renderer projection; platform supplies the surface
   scale and consumes the result while retaining native window, material,
   composition, and presentation realization.
3. **Admission and naming.** The correction deletes false platform ownership
   without adding a trait, callback, wrapper, or second representation.
   `render::scene` is the crate-private supporting module and exact
   `render::Scene` is its parent projection of the existing private paint
   scene. Native call sites therefore qualify the supporting module as
   `render::scene` and the central type as `render::Scene`, following the
   established module/type pattern; no compound declaration or alias was
   introduced. The old native paint and color paths are tombstoned, and stale
   test/doctrine wording now names renderer projection.
4. **Reduction and rewire.** Moved `platform/native/paint.rs` to
   `render/scene.rs` and `platform/native/color.rs` to `render/color.rs`.
   Parent and popup surfaces call the renderer scene contract; popup cache
   identity remains the exact same `paint::Scene` type through `render::Scene`;
   surface clears use one renderer color boundary; and Windows composition
   consumes the shared rounded-rectangle projection. Test-only helpers were
   narrowed to private while only the live platform crossings became
   crate-visible.
5. **Retained physical edge.** Windows composition still consumes private
   `paint::Grid` and projected rectangles when converting renderer geometry to
   OS composition regions. That is a real physical-realization boundary, not a
   second semantic lowering algorithm, and it remains visible for the paired
   renderer/platform boundary trace. Popup tests also use `paint::Color` only
   to construct an unequal private scene. Neither receipt is laundered into a
   claim that platform is paint-free.
6. **Behavior, clocks, and economics.** The move preserves the source algorithm
   and all seventeen owner tests. The same scale, snapped coordinates, colors,
   shaped-buffer handles, primitive order, clip/group topology, material
   values, visual bounds, popup translations, and scene equality reach the
   same renderer and caches. No shaping, allocation, invalidation, batching,
   pass fusion, acquisition, submission, acknowledgement, or presentation
   clock changed.
7. **Proof and ratchet.** The new architecture witness requires renderer
   ownership of scene lowering and popup projection, requires native surface
   and popup consumers to use that contract, and tombstones both former native
   files. Existing refraction, generic-filter extinction, popup visual-reach,
   color-delegation, composition geometry, unchanged-scene, fractional-scale,
   clip, focus, and table-rule witnesses moved with their owner and passed.
8. **Full verification.** The library discovered 1,086 tests: 1,076 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; all census parser witnesses, formatting, diff checks, and
   protected `comparison_open: true` state passed.
9. **Gauge delta from R4-01.** Truthful renderer-scene and renderer-popup
   dependencies raise production edges 325 -> 327 and slot edges 51 -> 52;
   forbidden edges, external questions, and SCCs remain 4, 1, and 1. Relocated
   owner tests resolve through their new module and raise test-only edges
   101 -> 107 and cross-slot test edges 82 -> 88. Explicit renderer/platform
   crossings raise production `pub(crate)` declarations 1,774 -> 1,786 in 192
   files. The tombstone witness raises source-root mentions 112 -> 113 and
   filesystem reads 336 -> 339. Allowances, panics, and expects remain 10, 9,
   and 102.
10. **Fixed point and next frontier.** Native platform contains no semantic
    scene-to-paint or color conversion owner; render contains no OS operation;
    and no parallel lowering route survives. Rung 4 continues with the direct
    `platform -> wgpu` external edge, tracing backend choice, format/alpha
    capability, safe window targets, composition visuals, and surface
    realization before admitting an abstraction.

### R4-03 — renderer dependency types versus native surface realization

Status: **complete; final external-boundary violation retired**. Correction
`198461f6` (`Own GPU surface boundary in renderer`).

1. **Question and complete trace.** Every direct `wgpu`/`wgpu-hal` use under
   platform was traced through explicit `WGPU_BACKEND`, Windows DX12-first and
   ordinary fallback attempts, context/device creation, selected adapter
   identity, safe winit targets, unsafe composition-visual targets, surface
   format/alpha choice, renderer-cache identity, popup material fallback,
   offscreen scene-format selection, DX12 HAL access, DXGI swapchain cloning,
   resize, acquire, draw, present, and popup prewarm. Parent and popup surfaces,
   redirected and composition-backed hosts, all acquire outcomes, and failure
   before tenancy completion were included.
2. **Current graph.** Platform named raw backend sets, adapter/backend values,
   texture formats, alpha modes, context options, unsafe surface targets, and
   the wgpu surface/HAL escape hatch. Some were real platform decisions, but
   their representation made the native adapter a second owner of renderer
   dependency types and spread one boundary across four platform files. The
   old parent projections also exposed supporting `Options`/`Outcome` types
   under compound aliases contrary to the established naming law.
3. **Admission.** Renderer owns GPU dependency representation and platform owns
   native attempt sequencing, window/visual lifetimes, fallback lifecycle, and
   COM realization. First-party `render::context::Backends` and `Backend`
   preserve backend-set and selected-backend capability; opaque
   `render::surface::Format` preserves exact renderer-cache identity;
   `surface::WindowsPopupSupport` preserves the alpha/format decision and its
   two distinct failure reasons; and `surface::Target` preserves the unsafe
   native-target lifetime contract. These values make invalid cross-seam raw
   dependency use unrepresentable rather than merely renaming wgpu types.
4. **Context and surface rewire.** Native context creation still owns the one
   explicit/DX12-first/fallback attempt loop, but constructs renderer-owned
   options from first-party backend sets. Canvas options now accept semantic
   scene color and convert only inside render. Surface alone selects and
   exposes opaque render/cache formats and resolved popup support. The deleted
   platform `render_format_for_canvas` recomputation is now one
   `Surface::render_format` decision consumed by parent, popup, prewarm, and
   renderer construction.
5. **Windows interop seam.** `surface::Target::composition_visual` encodes the
   unsafe wgpu target while the native owner supplies and retains the live
   `IDCompositionVisual`. `Surface::dx12` returns the scoped HAL guard whose
   borrow cannot outlive the surface; platform clones the live DXGI swapchain
   and continues to own every WinRT/COM tree operation. No service callback,
   raw-pointer ownership transfer, transmute, renderer import of Windows
   policy, or second surface representation was introduced.
6. **Naming and visibility reduction.** The render parent now projects exactly
   `Canvas`, `Context`, `Frame`, and `Surface` as the same-named central types.
   Supporting `canvas::Options`, `context::Options`/`Backends`/`Backend`,
   `frame::Outcome`, and surface contracts remain namespaced. The compound
   parent aliases `CanvasOptions`, `ContextOptions`, and `FrameOutcome` are
   deleted. Raw context device/instance/adapter/queue access and canvas
   color/alpha access narrowed to render; platform contains no `wgpu::` or
   `wgpu_hal::` spelling.
7. **Behavior, clocks, and economics.** Explicit backend choice remains the
   sole attempt; implicit Windows still tries DX12 then the ordinary set; the
   same device requirements and `DxgiFromVisual` option are used. The popup
   availability matrix remains exactly available, non-sRGB-format unavailable,
   or premultiplied-alpha unavailable with the same diagnostics. The same
   surface format keys the same renderer, the same sRGB offscreen format and
   pack pass are selected, and the same visual and cloned swapchain reach the
   same composition tree. Allocation, pipelines, draw order, batching/pass
   fusion, acquisition, submission, acknowledgement, and every presentation
   clock are unchanged.
8. **Proof and platform scope.** The ratchet recursively forbids `wgpu::` and
   `wgpu_hal::` under platform, pins all first-party contracts and central-type
   projections, and assigns both dependencies to renderer in the gauge. The 48
   native lifecycle tests, nine surface/context tests, eight active renderer
   topology tests, and 109 architecture witnesses passed. The exact Windows
   interop calls compile on the current target; live DX12 composition tenancy
   was not newly hardware-verified, so this cell inherits the standing native
   hardware scope rather than claiming a broader guarantee.
9. **Full verification and gauge.** The library discovered 1,088 tests: 1,078
   passed, 10 standing ignores, and 0 failed. All targets and all five examples
   compiled without warnings; census parser, formatting, diff, and protected
   example checks passed. Production/test edges, slot edges, forbidden edges,
   SCCs, and cross-slot test edges remain 327/107, 52, 4, 1, and 88. External
   violations fall 1 -> 0. Explicit first-party crossings raise production
   `pub(crate)` declarations 1,786 -> 1,804 in 192 files; source-root mentions
   rise 113 -> 115 and filesystem reads 339 -> 345. Allowances, panics, and
   expects remain 10, 9, and 102.
10. **Fixed point and next frontier.** Renderer dependency types and HAL access
    now have one renderer owner; native platform consumes stable first-party
    contracts and retains only OS realization. No raw GPU dependency crossing,
    duplicate capability decision, compatibility alias, or concealed callback
    remains. Rung 4 continues with a bidirectional renderer/runtime/platform
    sweep over presentation clocks, native-popup retirement, remaining paint
    crossings, allowances, panics, and visibility before closure.

### R4-04 — renderer geometry projection versus native composition values

Status: **complete; last production platform-to-paint edge retired**.
Correction `7053ed93` (`Project composition geometry through renderer`).

1. **Question and trace.** The remaining production `platform -> paint`
   crossing was traced through Windows material-region projection, shadow
   spread/offset/blur, all four scale-factor witnesses, rounded-rectangle
   snapping, uniform-radius admission, opacity, keyed region realization,
   unchanged-scene suppression, and the report-after-success path. Native
   composition imported private paint only to clamp a scale and unpack the
   renderer's snapped rectangle into physical COM values.
2. **Current and proposed graph.** Renderer already owned the semantic-to-paint
   projection and paint's device-grid policy. Letting platform construct
   `paint::Grid` and inspect `paint::Rect` exposed a private renderer
   representation without giving platform any independent decision. The
   admitted graph makes renderer project the exact physical rectangle and
   resolved radii; platform consumes those values while retaining COM geometry
   creation, material identity, visual order, failure, and lifecycle.
3. **Admission and type ruling.** `render::scene::Scale` preserves the clamped
   scale invariant and `PhysicalRect` makes the logical-to-physical coordinate
   transition structural. They are not transport aliases: raw scale and a
   logical rectangle cannot be substituted after admission, and the private
   paint grid/rectangle cannot cross the seam. Both supporting concepts remain
   namespaced under `render::scene`; no parent alias or compound compatibility
   name was introduced.
4. **Reduction and rewire.** Renderer now performs the existing grid clamp,
   snapped rounded-rectangle conversion, radius resolution, and physical-scale
   multiplication once. Native shadow projection consumes the admitted scale;
   native material geometry consumes the physical rectangle. The former
   crate-visible paint-rectangle helper narrowed to its owner, and the last
   production paint import under platform was deleted.
5. **Retained test seam.** `platform::native::popup` still has a test-only
   paint import solely to construct an unequal private scene for cache-change
   coverage. It is not a production crossing or a second lowering route; its
   future-owner housing remains an explicit Rung 6 test disposition rather
   than being hidden in this production cell.
6. **Behavior, clocks, and economics.** The same clamped factor, snapped
   origin/area, resolved radii, uniform-radius rejection, opacity, and shadow
   values reach the same COM calls. Scene identity, region keys, visual order,
   invalidation, renderer batches/pass fusion, submission, acknowledgement,
   and candidate/prepared/submitted/presented/committed clocks are unchanged.
   No allocation or duplicate computation was added at the boundary.
7. **Proof and ratchet.** Nine native-composition and seventeen renderer-scene
   tests passed with the architecture witness. The witness now requires the
   physical renderer projection and forbids private paint vocabulary in native
   composition. The full library discovered 1,088 tests: 1,078 passed, 10
   standing ignores, and 0 failed. All targets and all five examples compiled
   without warnings; census parser, formatting, diff, and protected
   `comparison_open: true` checks passed.
8. **Gauge delta from R4-03.** Production edges fall 327 -> 326. Top-level
   modules, test-only edges, slot edges, forbidden edges, external violations,
   SCCs, and cross-slot test edges remain 47, 107, 52, 4, 0, 1, and 88. The
   explicit physical-boundary values raise production `pub(crate)` declarations
   1,804 -> 1,813 in the same 192 files. Source-root mentions, filesystem reads,
   allowances, panics, and expects remain 115, 345, 10, 9, and 102.
9. **Fixed point and next frontier.** No production platform source imports
   private paint. Renderer owns projection through physical values and platform
   owns OS realization from those values. Rung 4 continues with the full
   presentation-clock, popup-retirement, allowance, panic/expect, visibility,
   and naming sweep before its boundary can close.

### R4-05 — external test modules versus the production gauge

Status: **complete; instrument correction with no production change**.
Correction `0a3878fa` (`Count external test modules correctly`).

1. **Question and evidence.** The Rung 4 panic/expect sweep found two reported
   production panics in `render/filter/tests.rs`, a file reachable only through
   `#[cfg(test)] mod tests;`. The gauge partitioned inline cfg-test items but
   forced an external file to test-only only when its top-level path happened
   to begin with `tests`. That made module housing, rather than the declaring
   cfg, decide whether external test helpers counted as production.
2. **Trace.** Every external module declaration inside a cfg-test range was
   resolved through Rust's ordinary `parent/name.rs` and
   `parent/name/mod.rs` housing. The live roots are `tests`,
   `text::acceptance`, `text::tests`, `view::presentation`, and
   `render::filter::tests`; all descendant files inherit the root's test-only
   status. Path-overridden example modules remain outside the source census,
   while their declarations are already excluded from production imports.
3. **Correction.** The census now masks all files first, resolves test-only
   external module roots from the parent declaration, and partitions each file
   according to that resolved module ancestry. It no longer guesses from a
   filename. Two new parser witnesses pin root resolution, descendant
   inheritance, ordinary production siblings, and both Rust file-housing
   forms; the parser suite now has eight witnesses.
4. **Gauge correction.** Production edges remain 326. Test-only edges correct
   107 -> 109; cross-slot test edges 88 -> 90. Production `pub(crate)` corrects
   1,813 in 192 files -> 1,809 in 191 files; production panics 9 -> 7 and
   production expects 102 -> 97. Modules, slot edges, forbidden edges,
   external violations, SCCs, source-root mentions, filesystem reads, and
   allowances remain 47, 52, 4, 0, 1, 115, 345, and 10. Earlier cell gauges
   remain receipts of the then-current instrument; these corrected counts are
   the canonical current baseline and will govern subsequent boundaries.
5. **Proof and fixed point.** All eight parser witnesses and a full census
   passed; the five live external roots were independently enumerated from the
   corrected resolver. No Rust source, behavior, dependency, visibility, test
   execution, presentation clock, renderer topology, or frame economics
   changed. Rung 4 resumes its hygiene and visibility sweep using the corrected
   gauge.

### R4-06 — popup pipeline cache admission versus asserted lookup order

Status: **complete; redundant cache protocol removed**. Correction `ec1ac0f8`
(`Make popup pipeline cache admission structural`).

1. **Question and trace.** The corrected production-expect inventory found the
   popup packer checking a format-keyed pipeline cache, creating/inserting on
   absence, then looking up the same key again and expecting initialization.
   The path was traced through non-sRGB premultiplied popup surfaces, renderer
   construction, composition-target drawing, exact sRGB packing, bind-group
   creation, the final replacement pass, and the ignored GPU readback witness.
2. **Challenge and admission.** Pipeline identity is legitimately cached by
   output format, but the `contains_key -> insert -> get -> expect` sequence
   encoded one map invariant as repeated logic plus a runtime assertion. The
   map entry API represents occupied versus vacant structurally and returns
   the admitted value from both paths. No new wrapper or state is needed.
3. **Reduction and rewire.** `pack_to_view` now consumes one
   `pipelines.entry(output_format)` result. The vacant path creates exactly the
   former pipeline and inserts it; the occupied path reuses the same value.
   The separate ensure method, duplicate hash lookup, and initialization expect
   are deleted. Pipeline creation still precedes bind-group creation and pass
   encoding, and the cache remains lazy and per renderer/format.
4. **Behavior and economics.** Shader source, alpha convention, target format,
   sampler, pipeline layout, render-pass load/store, bind group, draw call, and
   output bytes are unchanged. A cache hit now performs one lookup instead of
   two; no allocation, pass, batch, presentation clock, invalidation, surface
   acquisition, submission, or acknowledgement changed.
5. **Proof and gauge.** Four focused popup-pack witnesses passed with one
   standing ignored GPU readback diagnostic, and the existing alpha-owner
   architecture witness now pins single-entry admission and tombstones the old
   protocol. Full library: 1,078 passed, 10 ignored, 0 failed; all targets and
   all five examples compiled without warnings; parser, census, format, diff,
   and protected-state checks passed. Production expects fall 97 -> 96; every
   other corrected gauge count remains unchanged.
6. **Fixed point and next frontier.** The renderer has one format-keyed popup
   pipeline owner and no asserted cache-order protocol. Rung 4 continues with
   native backend attempt shape, native-popup lifecycle expects, allowances,
   public/private backend crossings, and presentation-clock closure.

### R4-07 — native backend attempt order versus optional error state

Status: **complete; nonempty fallback policy made structural**. Correction
`d50a0d71` (`Make native backend attempts nonempty`).

1. **Question and trace.** Native context creation represented explicit,
   DX12-first, and ordinary backend attempts as a vector, accumulated an
   optional last error, then expected the vector to have been nonempty. The
   trace covered authoritative `WGPU_BACKEND`, Windows tenancy preference,
   non-Windows defaults, first-attempt success/failure, fallback
   success/failure, selected context installation, error logging, material
   fallback logging, and the render error returned when all attempts fail.
2. **Challenge and type admission.** The policy has one mandatory first attempt
   and at most one fallback; an empty sequence and more than one fallback are
   invalid states. Private `surface::Attempts { first, fallback }` preserves
   that distinction directly. It earns its existence by removing the invalid
   empty state and the `Option<Error>` protocol rather than transporting an
   unchanged collection.
3. **Reduction and rewire.** Explicit selection creates one first attempt and
   no fallback. Windows implicit selection creates DX12 first and the ordinary
   backend set as fallback; other targets create the ordinary set first with
   no fallback. `Attempts::initialize` performs the first operation, performs
   the fallback only after failure, returns the first error when no fallback
   exists and the second error when both fail, and records the same diagnostics.
   The vector, loop-index policy, optional last error, and nonempty expect are
   deleted.
4. **Behavior and economics.** Backend sets, environment authority, attempt
   count/order, context options, DX12 tenancy, legacy material fallback,
   adapter/device creation, failure reporting, and successful context cache
   installation are unchanged. No GPU resource, surface, scene, pass/batch,
   acquisition, submission, presentation clock, or frame path changed.
5. **Proof and gauge.** The explicit-choice owner test and both Windows policy
   architecture witnesses passed; the target-gated DX12-first owner test
   remains compiled on Windows. Full library: 1,078 passed, 10 ignored, 0
   failed; all targets and all five examples compiled without warnings;
   parser, census, format, diff, and protected-state checks passed. Production
   expects fall 96 -> 95; every other corrected gauge count is unchanged.
6. **Fixed point and next frontier.** Native backend policy has one platform
   owner and cannot represent an empty attempt ladder. Rung 4 continues through
   native-popup cache/lifecycle invariants, the stale popup allowance, the six
   public/private backend allowances assigned to Rung 6, and the full clock and
   retirement sweep.

### R4-08 — stale native-popup argument allowance

Status: **complete; unowned suppression removed**. Correction `4be10c9e`
(`Remove stale popup argument allowance`).

1. **Question and ruling.** The allowance inventory found
   `popup_needs_concealment` carrying `clippy::too_many_arguments` after prior
   reductions left it with five inputs. Those inputs remain the exact current
   exposure, material, and scale comparison; grouping them would add an
   intermediate with no invariant. The suppression itself had no live lint or
   owner and was stale.
2. **Correction and proof.** Deleted the attribute without changing the
   function or any call site. Both concealment/serial witnesses passed and all
   targets compiled without warnings; format, diff, census, and protected state
   checks passed. Allowances fall 10 -> 9; all other corrected gauge counts and
   every renderer, presentation, and lifecycle path are unchanged.
3. **Fixed point and next frontier.** Native popup concealment has no local
   allowance debt. The remaining six platform allowances all describe the
   public `Backend` trait's use of crate-private overlay/IME contracts and stay
   visible for the Rung 6 symbol-level public-surface ruling; Rung 4 now closes
   its full clocks, retirement, dependency, and retained-invariant sweep.

### R4-09 — native renderer cache admission versus post-insert expects

Status: **complete; one platform cache owner consumed directly**. Correction
`80586f7f` (`Centralize native renderer cache admission`).

1. **Question and trace.** Parent clear/present, popup present, and popup-host
   prewarm each called `ensure_renderer(format)`, then looked up the identical
   format and expected the renderer to exist. Popup first-present diagnostics
   separately queried the same map for warm/cold state. The trace covered
   render-format resolution, parent and popup canvas lifetimes, lazy renderer
   construction, prewarm, clear/draw, renderer reuse, alpha packing, and every
   acquire/present outcome.
2. **Admission.** Platform legitimately owns a renderer-instance cache keyed
   by opaque renderer `surface::Format`; renderer owns construction and draw
   behavior. The cache entry is the complete crossing. A second cache type or
   wrapper would repeat identity, while the existing ensure-plus-get protocol
   repeated lookup and represented impossible postconditions dynamically.
3. **Reduction and rewire.** One private `renderer_for_format` function now
   consumes `renderers.entry(format)`, returns the admitted renderer and its
   warm/cold fact, and constructs only on vacancy. All four consumers use that
   returned renderer directly. The ensure method, separate warm query, four
   follow-up lookups/expects, and the ensure method's context expect are
   deleted.
4. **Behavior and economics.** The same opaque format selects the same cached
   renderer and pipeline set. Cold initialization still precedes clear/draw;
   warm diagnostics retain their exact value and timing; parent, popup, and
   prewarm paths reach the same canvas and renderer. Each use now hashes once
   instead of two or three times. Scene order, batching/pass fusion, shaping,
   invalidation, acquisition, submission, acknowledgement, popup generation,
   and all presentation clocks are unchanged.
5. **Proof and gauge.** All 48 native lifecycle witnesses and the strengthened
   format-cache architecture witness passed. Full library: 1,078 passed, 10
   ignored, 0 failed; all targets and all five examples compiled without
   warnings; parser, census, format, diff, and protected-state checks passed.
   Production expects fall 95 -> 90; every other corrected gauge count remains
   unchanged.
6. **Fixed point and next frontier.** Native platform has one renderer cache
   admission path and no renderer-existence protocol. Rung 4 proceeds to its
   final bidirectional zero-change sweep over dependencies, presentation and
   popup clocks, retirement, retained expects, visibility, and names.

### R4-10 — full presentation-boundary fixed-point audit

Status: **complete; no further Rung 4 correction admitted**.

1. **Question and sweep.** The closing pass walked semantic scene preparation,
   renderer lowering, private paint, GPU context/surface ownership, native
   parent and popup realization, diagnostics observation, runtime
   acknowledgement, presented geometry, popup generation receipts, retirement,
   pooling, and parent departure in both directions. It challenged every
   remaining dependency, translation, cache, presentation clock, failure path,
   expect, allowance, visibility crossing, and parent projection in the slice.
2. **Dependency result.** Render and private paint import neither platform,
   runtime, nor diagnostics. Platform contains no production private-paint
   import and no raw `wgpu`/`wgpu-hal` spelling. Renderer owns GPU dependency
   types, semantic-to-paint lowering, physical geometry projection, draw facts,
   and surface presentation; platform owns attempt sequencing, OS targets,
   native windows, COM realization, renderer-instance caching, and popup host
   lifecycle. Diagnostics consumes renderer facts and is never a renderer
   behavior input. No callback-hidden replacement edge or parallel lowering
   route survives.
3. **Parent presentation clock.** `PreparedFrame` captures one layout, scene,
   overlay set, invalidation, revision, and desired epoch. A skipped surface
   acquisition submits and presents nothing and produces no present timing.
   Native realization derives `presented` only from that timing; runtime
   acknowledges the epoch and promotes `PresentedGeometry` only for a
   successful receipt, while every skipped attempt retains the visible hit
   surface and retries the same invalidation. Older successful receipts cannot
   replace newer presented geometry. Diagnostic attempt samples remain
   distinct from successful-frame and key-to-present samples.
4. **Popup-local clock.** Live and `RetiringPopup` layers take the native-popup
   path while in-frame `Ghost` layers remain semantic scene content. Each popup
   show or reconfiguration carries an exact generation through configured,
   prepared-concealed, acquire, present, synchronization, and exposure states.
   Skipped acquire remains concealed and requests the bounded retry; stale
   generation receipts are inert; geometry/material replacement commits only
   after the current present. A retiring layer retains paint geometry but sets
   `accepts_input` false, so it contributes no native hit target.
5. **Retirement and cleanup.** Stale cleanup is scoped to synchronized parents,
   rehomes cursor and IME state before removal, and returns only compatible,
   ready composition hosts beneath the per-parent capacity. Every other host
   hides before teardown and removes its popup subclass in `Drop`. Parent
   departure removes live popups, raw-id routes, dormant hosts, capacity and
   prewarm state, cursor ownership, and IME ownership. No ghost, host, receipt,
   or native input route can outlive its owner.
6. **Retained invariants and visibility.** The remaining native expects assert
   already-witnessed lifecycle ordering: context after tenancy, popup after
   admitted creation/configuration, due values after their applicator reports
   due, and a composition host after committed material admission. Replacing
   them would require a broad state-machine rewrite without an observed
   failure or a smaller contract, so no behavior-preserving Rung 4 correction
   is admitted. The six `private_interfaces` allowances are the public
   `Backend` trait's overlay/IME crossings and remain assigned to Rung 6's
   symbol-level public-surface ruling. The sole platform paint import is inside
   the popup test module and remains assigned to Rung 6 test housing. No Rung 4
   production panic remains under the corrected test partition.
7. **Naming and economics.** Renderer parent projections are exactly the
   same-named central `Canvas`, `Context`, `Frame`, and `Surface`; supporting
   types remain qualified through their modules, and the retired compound
   aliases stay absent. The same semantic scenes, snapped primitives, cache
   identities, batch/pass order, filter pools, acquisition, submission, and
   acknowledgement routes remain in force. The admitted cache cells reduce
   hash lookups without changing renderer topology or frame work.
8. **Proof and gauge.** The closing library run discovered 1,088 tests: 1,078
   passed, 10 standing ignores, and 0 failed. All targets and all five examples
   compiled without warnings; all eight census parser witnesses, the full
   census, formatting, diff checks, and protected `comparison_open: true`
   state passed. The corrected gauge is 47 top-level modules, 326 production
   edges, 109 test-only edges, 52 slot edges, 4 forbidden edges, 0 external
   violations, 1 SCC, 1,809 production `pub(crate)` declarations in 191 files,
   90 cross-slot test edges, 115 source-root mentions, 345 filesystem reads,
   9 allowances, 7 panics, and 90 expects.
9. **Fixed point.** The full Rung 4 trace admits no additional correction.
   Every renderer/platform crossing names a first-party contract and every
   presentation clock advances only from its proper receipt. The four
   surviving forbidden edges are all explicit UI questions. Rung 5 may now
   reassess that territory without inheriting renderer or platform ownership
   confusion.

## Rung 4 closure — semantic presentation and physical realization

Status: **complete**. Production boundary `80586f7f`; final production-cell
ledger boundary `d89662fc`. The repository was clean before the closure record
and preserved `comparison_open: true`.

Rung 4 inverted diagnostics observation, moved semantic scene lowering and
color conversion from native platform to renderer, introduced first-party GPU
surface contracts, projected physical composition geometry at the renderer
boundary, corrected the census treatment of external test modules, and reduced
three asserted cache/attempt protocols to structural admission. No physical
crate, feature gate, public numeric dependency type, callback bridge, or
user-visible behavior change was introduced.

### Boundary gauge

| Metric | Rung 3 | Rung 4 |
|---|---:|---:|
| Top-level production modules | 47 | 47 |
| Unique production module edges | 325 | 326 |
| Unique test-only module edges | 100 | 109 |
| Provisional cross-slot edges | 43 | 52 |
| Provisional forbidden internal edges | 5 | 4 |
| Provisional heavy external-boundary violations | 1 | 0 |
| Provisional slot SCCs | 1 | 1 |
| Production `pub(crate)` declarations | 1,764 | 1,809 |
| Cross-slot test-only edges | 80 | 90 |
| `CARGO_MANIFEST_DIR` mentions | 111 | 115 |
| Filesystem read calls | 334 | 345 |
| `#[allow(...)]` attributes | 10 | 9 |
| Production `panic!` calls | 9 | 7 |
| Production `.expect(...)` calls | 102 | 90 |

R4-05 corrected the instrument mid-rung: five expects, two panics, four
crate-visible declarations, and two module edges previously counted as
production were actually housed beneath external cfg-test module roots. The
Rung 3 column remains its historical receipt; the corrected Rung 4 column is
the governing baseline from here forward. The added truthful slot edges and
visibility are primarily the diagnostics observer seam and explicit renderer
scene/surface contracts. Test-edge and source-read growth comes from focused
architecture receipts plus the corrected test partition and remains assigned
to Rung 6 consolidation.

The final external-boundary violation and renderer diagnostic back-edge are
gone. Platform has no production private-paint or raw GPU dependency crossing;
renderer has no platform/runtime/diagnostics back-edge. The four remaining
forbidden edges are `window -> theme`, `layout -> diagnostics`,
`view -> diagnostics`, and `widget -> document`, all named inputs to the Rung 5
UI examination rather than residue hidden by the presentation work.

### Boundary proof and next frontier

- full library: 1,078 passed, 10 ignored, 0 failed;
- all targets and all five examples compiled without warnings;
- parent presentation, stale-geometry, popup generation, concealment,
  retirement, cleanup, renderer cache, scene projection, and architecture
  witnesses passed;
- all eight census parser witnesses, the full census, `cargo fmt --check`,
  `git diff --check`, and tombstone searches passed;
- renderer order, batching/pass fusion, shaping, invalidation, parent and popup
  presentation clocks, and frame economics remain equivalent except for the
  recorded cache-lookup reductions.

Rung 5 begins by tracing the UI territory as one knot before breaking any
edge: scene, view, widget, layout, composition, interaction, session, table,
virtualization, selection, draft, popup, overlay, theme, pointer, and the four
surviving forbidden crossings.

## Rung 5 cell records

### R5-01 — layout-produced text facts versus diagnostic aggregation

Status: **complete; producer-owned observer fact established**. Correction
`6a653084` (`Invert layout diagnostics ownership`).

1. **Whole-knot trace before selection.** The required opening census covered
   all 27,250 Rust lines under composition, draft, interaction, layout,
   overlay, pointer, popup, scene, selection, session, table, theme, view,
   virtual-list, and widget ownership. The fifteen modules have 73 internal
   module edges and form one SCC. The live frame path is application view ->
   retained composition -> transient session projection -> layout and virtual
   refinement -> semantic scene and overlay buckets -> renderer/platform
   realization. Session owns per-window focus and interaction; composition owns
   retained node identity; layout owns geometry/hits; scene owns drawing;
   overlay owns floating-entry lifetime. Those cycles express one coordinated
   UI state machine, so no internal virtual-crate split is admitted merely from
   the import graph.
2. **Question and trace.** The first external contradiction was the
   `layout -> diagnostics` edge. Layout's text service owns the author-overflow
   counter, consumes and resets text-engine layout receipts, emits one aggregate
   after composition, and runtime accumulates that fact into the per-window
   diagnostic snapshot supplied to tools, tests, and the text-editor debug
   panel. Successive layout recompositions add facts; cache reuse does not
   manufacture layout work; reset occurs exactly when the aggregate is taken.
3. **Current and proposed graph.** The complete public `Text` fact and its
   aggregation methods were declared under diagnostics, forcing both
   `layout::Engine` and `layout::text::Service` to import their observer.
   Rung 4's observer law applies exactly: the producer declares its receipt and
   diagnostics owns accumulation/storage. The admitted graph makes
   `layout::text::Text` the canonical declaration and
   `diagnostics::Text` its exact established public projection.
4. **Naming and visibility.** The declaration remains simply `Text`; no
   `LayoutText`, `TextDiagnostics`, alias, or compatibility type was added.
   The private layout parent projects its same-named central `Text`, engine
   call sites import `{text, Text}`, and supporting service/layout types remain
   namespaced or private. The text-engine merge operation narrowed from
   crate-visible to module-private because it has one owner; only runtime's
   aggregate-to-aggregate `add` crossing remains crate-visible.
5. **Reduction and rewire.** Moved the unchanged sixteen public fields and
   accumulation law into `layout/text.rs`, deleted `diagnostics/text.rs`, and
   changed diagnostics to re-export the producer-owned type. Runtime, view
   context, examples, tests, field names, reset timing, counter addition, and
   public paths are unchanged. Layout contains no diagnostics import, and no
   duplicate fact or translation survives.
6. **Behavior and economics.** The same text-engine receipt is read and reset
   once, the same author-overflow counter is replaced with zero, and the same
   aggregate is added to the same per-window snapshot. View rebuilding,
   virtual refinement, layout cache reuse, shaping/cache work, scene order,
   batching/pass fusion, invalidation, and presentation clocks are unchanged;
   the correction only relocates the value and narrows one method.
7. **Proof and ratchet.** A recursive architecture witness forbids diagnostics
   vocabulary under layout, requires the canonical layout declaration and
   parent projection, requires the exact diagnostics re-export, and tombstones
   the former file. Debug-panel snapshot and live-render diagnostic witnesses
   passed. Full library: 1,079 passed, 10 ignored, 0 failed; all targets and all
   five examples compiled without warnings; all eight census parser witnesses,
   formatting, diff checks, and protected state passed.
8. **Gauge delta from Rung 4.** Production edges fall 326 -> 325, slot edges
   52 -> 51, and forbidden edges 4 -> 3. Deleting the old crate-visible merge
   method lowers production `pub(crate)` declarations 1,809 in 191 files ->
   1,808 in 190 files. The architecture receipt raises source-root mentions
   115 -> 116 and filesystem reads 345 -> 348. Modules, test edges, cross-slot
   test edges, external violations, SCCs, allowances, panics, and expects remain
   47, 109, 90, 0, 1, 9, 7, and 90.
9. **Fixed point and next frontier.** Layout owns and publishes its facts;
   diagnostics only observes and accumulates them. No other layout diagnostic
   back-edge remains. The cohesive UI ruling stays provisional while Rung 5
   continues through the document projection, view-context diagnostics, and
   window/theme split responsibilities before re-scanning internal state shape.

### R5-02 — document workflow versus runtime housing

Status: **complete; independent document seam admitted**. Correction
`3826a669` (`Admit document workflow as independent seam`).

1. **Question and complete trace.** The remaining `widget -> document`
   violation was traced through `TextArea::from_document`, public document
   construction and mutation, selection, standard editing commands, focused
   draft and table service implementations, command availability and outcomes,
   open/save dialogs and cancellation facts, synchronous and deferred saves,
   identity/version/generation rejection, atomic replacement, application
   targets, runtime dispatch, and facade exports. Widget consumes only a cloned
   buffer plus copied selection state; document imports no UI or runtime owner.
2. **Current and proposed graph.** The provisional runtime slot made document
   look like orchestration because runtime coordinates its commands. The live
   module instead owns a complete workflow and depends only on text,
   command/context/response/target/notification, clipboard, Unicode
   segmentation, and its private OS replacement primitive. Runtime, UI, and
   facade are independent consumers. The admitted graph gives `document` its
   own virtual owner below those consumers.
3. **Admission and resistance.** The seam has one sentence of ownership,
   public state/command/save contracts smaller than its implementation, no
   higher import, independent runtime/widget/application consumers, and
   meaningful persistence dependency weight. Splitting standard editing
   commands from the workflow is not admitted: the established command set,
   document targets, focused-draft targets, and table target share the same
   typed command identities and outcome law already settled by Rungs 2 and 3.
   No callback, service locator, or wrapper is introduced to disguise that
   shared contract.
4. **Map and external-boundary correction.** The virtual map moves `document`
   out of runtime, permits its direct clipboard/text/command dependencies, and
   names UI, runtime, and facade consumers. `unicode-segmentation` now belongs
   to text and document rather than the false runtime bucket. The former
   module-specific `windows-sys` exception is retired: document and platform
   are the two named owners, and document's use remains the target-specific
   private step of its atomic save transaction.
5. **Behavior and API.** No Rust production path, public spelling, command
   identity, buffer clone, selection copy, I/O operation, failure result,
   cleanup path, or runtime route changed. `TextArea::from_document` remains
   the exact named value-semantics projection, while `from_buffer` remains the
   general constructor. No crate or gate was created; document-workflow
   optionality remains a Rung 6 formulation question.
6. **Proof and ratchet.** The existing document-to-widget architecture witness
   now also recursively forbids runtime, session, view, widget, layout, and
   platform imports under document; pins the independent slot; assigns
   `windows-sys` directly; and requires the old exception table to be empty.
   Four focused save/version/temporary-sibling witnesses passed. Full library:
   1,079 passed, 10 ignored, 0 failed; all targets and all five examples
   compiled without warnings; all census parser witnesses, formatting, diff,
   and protected-state checks passed.
7. **Gauge delta from R5-01.** Production/test edges remain 325/109. Exposing
   the truthful document dependencies and consumers raises slot edges 51 ->
   56, while the false `widget -> document` direction lowers forbidden edges
   3 -> 2. The one SCC remains and now truthfully includes document while the
   two remaining back-edges still connect its lower command dependency to UI;
   SCC membership is evidence for the next cells, not a reason to merge the
   admitted owner. The consolidated witness keeps source-root mentions at 116
   and raises filesystem reads 348 -> 349. Visibility, cross-slot test edges,
   external violations, allowances, panics, and expects remain unchanged.
8. **Fixed point and next frontier.** Document has one owner for editable file
   workflow and no runtime/UI reason for existence; runtime coordinates it and
   widget projects it. No external exception or false upward UI dependency
   remains. Rung 5 continues with the diagnostics-bearing view callback
   context, then the theme-selected window default.

### R5-03 — application view callback context versus declarative UI

Status: **complete; split source responsibility made explicit**. Instrument and
map correction `67c5ba6e` (`Model split source responsibilities`).

1. **Question and complete trace.** `view::Context` was traced from the public
   Runtime builder callback type through per-window render/render-all
   invocation, runtime snapshot construction, public window and diagnostics
   accessors, the text-editor instrument panel, and the surrounding declarative
   view module. The context is created only when runtime invokes application
   view construction. No node, style, binding, control, composition, layout, or
   scene path consumes diagnostics.
2. **Current and proposed graph.** The top-level-module gauge assigned all of
   `view` to UI, so the dedicated `src/view/context.rs` callback envelope made
   the honest facade-to-diagnostics contract appear as a forbidden
   UI-to-diagnostics edge. Moving the type to runtime would make orchestration
   own an application API; moving its public spelling would create needless API
   churn; and passing diagnostics separately or through a callback would smear
   the same facade contract. The admitted graph keeps declarative view in UI
   and assigns the already-isolated callback source to facade responsibility.
3. **Instrument correction.** The census now accepts exact Rust-file source
   responsibilities as real virtual owners. It resolves each internal and
   external receipt from that effective owner, while retaining top-level module
   and module-edge counts. Configured paths and slots must exist; ordinary files
   continue to inherit their module owner. This is not an exception table: the
   assigned owner participates in direction checks, slot edges and cycles,
   cross-slot test counts, external-boundary checks, and reports.
4. **Naming and API ruling.** The simple canonical declaration and established
   `view::Context` projection remain exact; no `ViewContext`, callback-context
   alias, second public root, wrapper, or generic resource bag is introduced.
   Runtime still supplies one immutable value containing window identity and a
   cloned diagnostics snapshot. A later physical facade may expose that value
   beside UI types without requiring the monolith to pretend both
   responsibilities have one owner.
5. **Doctrine and ratchet.** Master design now distinguishes the application
   callback envelope from declarative view data and forbids every other view
   source from depending on diagnostics. The architecture witness pins runtime
   construction, the public projection, the exact source assignment, and
   diagnostics confinement. A ninth census witness proves that one assigned
   receipt resolves facade-to-diagnostics while an ordinary receipt from the
   same top-level module remains UI-to-diagnostics and would still be judged on
   that direction.
6. **Behavior and economics.** No Rust production source, public signature,
   callback timing, clone, snapshot, layout, scene, renderer, presentation
   clock, allocation, or frame path changed. The cell changes only the gauge,
   doctrine, and structural witnesses.
7. **Proof and gauge delta from R5-02.** The library discovered 1,090 tests:
   1,080 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; all nine census witnesses, the full
   census, formatting, diff, and protected-state checks passed. Production/test
   module edges remain 325/109. One explicit split responsibility lowers slot
   edges 56 -> 55 and forbidden edges 2 -> 1; the remaining SCC sheds
   diagnostics and renderer and is now command/document/foundation/text/UI.
   Visibility, cross-slot test edges, external violations, allowances, panics,
   and expects remain 1,808 in 190 files, 90, 0, 9, 7, and 90. The new ratchet
   raises source-root mentions 116 -> 117 and filesystem reads 349 -> 355 for
   Rung 6 consolidation.
8. **Fixed point and next frontier.** Facade owns the application callback
   envelope; UI owns declarative view; runtime constructs and invokes the
   contract; diagnostics remains an observer. No false view back-edge or
   broader source assignment survives. Rung 5 continues with the theme-selected
   window default, the sole remaining forbidden edge.

### R5-04 — lower window vocabulary versus facade configuration

Status: **complete; final forbidden edge and slot cycle retired**. Correction
`136d9252` (`Separate window facade configuration`).

1. **Question and complete trace.** The remaining `window -> theme` edge was
   traced through `window::DEFAULT_CANVAS_COLOR`, `Options::new/default`, title,
   size, canvas and kind configuration, session window construction, runtime
   fallback clears, platform application/popup realization, unthemed scene
   clears, examples, and default-value witnesses. Theme owns the one canvas
   token. `Options` selects it for application configuration, while `Kind` is
   lower window vocabulary consumed independently by session, shell, platform,
   backend reports, and tests.
2. **Current and proposed graph.** The window parent declared the public default
   projection and imported theme, while `options.rs` mixed facade configuration
   with the lower `Kind` declaration. Assigning all of window to facade would
   falsely raise identity/facts/presentation vocabulary; copying the color bytes
   downward would create competing theme truth. The admitted graph isolates
   exact default declarations and `Options` as facade sources, isolates `Kind`
   as lower window vocabulary, and leaves the theme token authoritative.
3. **Reduction and rewire.** Added focused `window/defaults.rs` and
   `window/kind.rs` housing, moved the unchanged declarations, and made the
   parent project them exactly. The width/height implementation constants now
   use sibling visibility instead of crate visibility. `options.rs` consumes
   lower `Kind` plus the facade defaults; no lower window source imports theme,
   and no duplicate constant, color value, or conversion survives.
4. **Naming and API ruling.** `window::Kind`, `window::Options`,
   `window::DEFAULT_TITLE`, and `window::DEFAULT_CANVAS_COLOR` remain the exact
   established public spellings and values. Their declarations use those same
   canonical names; the parent performs exact re-exports. No `WindowKind`,
   `WindowOptions`, renamed default, compound alias, or compatibility path is
   introduced. Moving the declarations does not authorize call-site churn.
5. **Doctrine and ratchet.** Master design now distinguishes lower window
   vocabulary from facade configuration. The existing canvas-owner witness now
   pins the dedicated sources, exact parent projections, simple names, facade
   assignments for defaults/options, lower assignment for kind, and theme
   confinement to the default-projection source.
6. **Behavior and economics.** Default title, 800x600 size, canvas bytes,
   application kind, explicit overrides, popup kind, scene clears, runtime
   fallback, platform realization, and every public method are unchanged. The
   same Copy `Kind` and `Color` values reach the same paths. No layout, scene,
   renderer, presentation clock, allocation, or frame work changed.
7. **Proof and gauge delta from R5-03.** The library discovered 1,090 tests:
   1,080 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; focused default/token witnesses, all
   nine census witnesses, the full census, formatting, diff, and protected-state
   checks passed. Production/test module edges remain 325/109. Two additional
   split responsibilities lower slot edges 55 -> 54, forbidden edges 1 -> 0,
   and SCCs 1 -> 0. Narrowing two constants lowers production `pub(crate)`
   declarations 1,808 -> 1,806 in the same 190 files. Cross-slot test edges,
   external violations, source-root mentions, allowances, panics, and expects
   remain 90, 0, 117, 9, 7, and 90; the strengthened witness raises filesystem
   reads 355 -> 359 for Rung 6 consolidation.
8. **Fixed point and next frontier.** Every accepted virtual owner now points
   one way in the gauge: zero forbidden internal edges, zero external boundary
   violations, and zero slot cycles. Theme owns the token, facade owns
   application window configuration, and lower window owns its facts. Rung 5
   now re-scans the cohesive UI state machine for state-shape, intermediate,
   repetition, lifecycle, visibility, naming, and housing findings before
   closure; a clean import graph alone is not the rung exit.

### R5-05 — stale overlay dead-code suppression

Status: **complete; unowned suppression removed**. Correction `0456a0f8`
(`Remove stale overlay allowance`).

1. **Question and trace.** The Rung 5 hygiene sweep found `overlay::Layer::id`
   marked `allow(dead_code)`. The field is produced by live, ghost, and retiring
   popup layers and is read on production scene logging, IME targeting, native
   popup presentation, and backend application paths as well as lifecycle
   witnesses. The suppression no longer described the code it covered.
2. **Correction and proof.** Deleted only the attribute. All nineteen overlay
   lifecycle witnesses, including the 10,000-update law test, passed; all
   targets compiled without warnings; format, diff, census, and protected state
   checks passed. Allowances fall 9 -> 8; every dependency, visibility, panic,
   expect, renderer, presentation, and lifecycle count/path is unchanged.
3. **Fixed point and next frontier.** Overlay layer identity has live consumers
   and no local suppression. The popup-realization and variable-list layout
   argument-count allowances remain visible pending their complete aggregate
   boundary ruling; the UI sweep continues through lifecycle state and
   intermediate types.

### R5-06 — atomic provided-list selection endpoints

Status: **complete; correlated optional state collapsed**. Correction
`ee43ac04` (`Make selection endpoints atomic`).

1. **Question and complete trace.** The state-shape sweep traced provided-list
   selection from pointer and keyboard admission through the virtual-list
   provider operations, session-scoped lookup, snapshot/restore, view
   projection, table active-row lookup, runtime reveal/presentation, provider
   reorder and deletion reconciliation, and the public read-only selection
   facts. Anchor and active each carried one stable key and one last usable
   provider index, but represented that one endpoint as two independent
   `Option`s.
2. **Authority and invariant.** The key remains the authoritative identity.
   Current provider order resolves it when available; the stored index remains
   only the existing navigation fallback when that key has departed or moved.
   Every construction, toggle, extension, select-all, movement, clear, and
   reconciliation path already created, replaced, or removed each key/index
   pair together. No caller observed either index independently.
3. **Reduction and displaced state.** Added the private value `Endpoint { key,
   index }` and reduced four independently optional fields to
   `anchor: Option<Endpoint>` and `active: Option<Endpoint>`. Endpoint resolution
   now has one local implementation. The four parallel fields and every
   duplicated half-pair assignment are deleted; a key-without-index or
   index-without-key state is no longer representable.
4. **Naming, visibility, and API ruling.** `Selection`, `Selection::anchor`,
   `Selection::active`, the root projection, and all call-site spellings remain
   exact. `Endpoint` is a simple private supporting concept in the selection
   owner and receives no parent re-export or compound alias, consistent with
   the canonical module/type and projection law. No visibility widened.
5. **Doctrine and ratchet.** Master design now records that an anchor or active
   endpoint atomically owns its stable key and fallback index. The architecture
   witness requires the two endpoint options and tombstones the four former
   independently optional field spellings.
6. **Behavior and economics.** Membership, selected counts, public facts,
   pointer modifiers, keyboard movement, range anchoring, all-except select-all,
   reorder/deletion fallback, snapshot identity, and change detection are
   unchanged. The same provider lookups occur at the same transitions. There is
   no new allocation, callback, layout, scene, renderer, presentation-clock, or
   platform work; selection state carries two option discriminants rather than
   four.
7. **Proof and gauge delta from R5-05.** The library discovered 1,091 tests:
   1,081 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; focused selection and architecture
   witnesses, all nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Production/test module edges remain
   325/109; split responsibilities, slot edges, forbidden edges, external
   violations, and SCCs remain 3, 54, 0, 0, and 0. Visibility, cross-slot test
   edges, allowances, panics, and expects remain 1,806 in 190 files, 90, 8, 7,
   and 90. The new ratchet raises source-root mentions 117 -> 118 and filesystem
   reads 359 -> 360.
8. **Fixed point and next frontier.** Provided-list endpoints have one atomic
   representation and no parallel optional state. The UI sweep continues
   through overlay lifecycle shape and the two argument-count allowances; the
   clean import graph and this local correction do not close Rung 5.

### R5-07 — valid resolved overlay lifecycle species

Status: **complete; flattened invalid lifecycle states replaced**. Correction
`ecacbc98` (`Make overlay layer lifecycle valid`).

1. **Question and complete trace.** The lifecycle sweep traced live overlay
   entries, in-frame ghosts, retiring native popups, fade sampling and
   scheduling, scene projection, native presentation, diagnostic logging,
   afterlife retention, contextual retarget/reopen, and the 10,000-update law.
   Resolved `Layer` stored species, optional live state, and optional elapsed
   time independently even though its three producers admitted only three
   correlated forms.
2. **Authority and admitted forms.** Live layers alone own `Entering` or `Live`
   state and may use either backend. Ghosts are always in-frame; retiring popups
   are always native. All three species own elapsed time. Backend remains a
   separate realization fact because live layers can legitimately select
   either realization; lifecycle species owns only lifecycle validity.
3. **Reduction and displaced path.** Replaced `kind: LayerKind`,
   `state: Option<State>`, and `elapsed: Option<Duration>` in resolved layers
   with private `Lifecycle::{Live { state, elapsed }, Ghost { elapsed },
   RetiringPopup { elapsed }}`. `LayerKind` and optional state remain exact
   downstream projections, while elapsed is now total. The runtime diagnostic
   path no longer supplies zero for an impossible absent elapsed value.
4. **Naming, visibility, and API ruling.** `Lifecycle` is the simple private
   supporting concept inside overlay; it has no public or parent projection and
   no compound compatibility alias. Existing `LayerKind`, `State`, and their
   call-site spellings remain unchanged. No visibility widened and no native
   platform type crossed into UI.
5. **Doctrine and ratchet.** Master design now states the three disjoint
   resolved lifecycle species, total elapsed time, and live-only state. The
   existing native-fade architecture witness now pins the retiring-native
   lifecycle case and requires the sum while tombstoning the two former
   optional fields.
6. **Behavior and economics.** Opacity, fade curves, schedules, paint order,
   hit transparency, backend choice, native surface retirement, ghost caps,
   context identity, reopening, logging values, scene contents, renderer
   topology, presentation clocks, and allocation paths are unchanged. Species
   projection adds only exhaustive matches over the same stored discriminant;
   no per-frame lookup, callback, allocation, layout, or native operation was
   added.
7. **Proof and gauge delta from R5-06.** The library discovered 1,091 tests:
   1,081 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; all nineteen focused overlay witnesses,
   including the 10,000-update lifecycle law, the strengthened architecture
   witness, all nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Every gauge remains unchanged:
   production/test edges 325/109, split responsibilities 3, slot edges 54,
   forbidden/external/SCC counts 0/0/0, visibility 1,806 in 190 files,
   cross-slot test edges 90, source-root mentions 118, filesystem reads 360,
   allowances 8, panics 7, and expects 90.
8. **Fixed point and next frontier.** Resolved overlay layers have one valid
   lifecycle representation and no absent-time fallback. Rung 5 continues with
   the popup-realization and variable-list argument aggregates, followed by a
   reverse sweep over the full UI owner.

### R5-08 — retained popup realization geometry contract

Status: **complete; flattened boundary aggregated and suppression removed**.
Correction `e6b8af93` (`Aggregate realized popup geometry`).

1. **Question and complete trace.** The remaining popup allowance was traced
   from selected-host placement and scale-resolved renderer projection through
   generation staging, native presentation, visible clipping, panel/visual
   offsets, hit geometry, retained-coordinate event translation, IME and
   retiring paint-only behavior. `Realization::native` accepted nine positional
   values even though six of them are the one resolved geometry fact that every
   downstream path must consume consistently.
2. **Boundary ruling.** Popup identity, parent, and presentation generation are
   realization identity. Local, host, visible-clip and visual bounds, panel
   offset, and host scale are one geometry value resolved once by the selected
   platform host. This is a deliberate popup/platform crossing contract, not a
   generic argument bag or a renderer type; placement intent and the
   platform-private applied-HWND geometry remain distinct.
3. **Reduction and rewire.** Added `popup::Geometry`, nested it in
   `popup::Realization`, and changed `Realization::native` to consume identity,
   generation, and that one value. All four construction paths now form the
   geometry at the resolving host/test boundary. Existing realization accessors
   delegate to the nested fact. The nine-argument constructor and its
   `too_many_arguments` suppression are deleted.
4. **Naming and visibility ruling.** The supporting concept is declared simply
   as `Geometry` inside `popup` and used as `popup::Geometry` at sibling call
   sites. It receives no compound declaration, alias, root projection, or
   parent re-export. The type and its constructor are the two narrowly widened
   receipts required for the real cross-module contract; fields remain private.
5. **Doctrine and ratchet.** Master design now names the one retained
   `popup::Geometry` value consumed by paint, hit testing, event translation,
   IME, accessibility, clipping, and material projection. The native-position
   architecture witness requires the namespaced aggregate, nested realization
   storage, and absence of the former suppression/flattened boundary.
6. **Behavior and economics.** Every coordinate, scale, generation, clip,
   offset, native position, hit rectangle, retained point, and log value is
   unchanged. The same `Copy` fields occupy the realization without heap
   allocation, clone, callback, lookup, layout, renderer, presentation-clock,
   or platform-operation changes.
7. **Proof and gauge delta from R5-07.** The library discovered 1,091 tests:
   1,081 passed, 10 standing ignores, and 0 failed. All targets and all five
   examples compiled without warnings; thirteen focused popup/native witnesses,
   the strengthened architecture witness, all nine census parser witnesses,
   the full census, formatting, diff, and protected-state checks passed. A
   repository-wide `-D warnings` Clippy probe confirmed the popup constructor no
   longer reports `too_many_arguments`; that probe remains non-gating because it
   also exposed the standing unrelated Clippy backlog. Production/test edges,
   split responsibilities, slot edges, forbidden/external/SCC counts, and
   cross-slot test edges remain 325/109, 3, 54, 0/0/0, and 90. The two admitted
   contract receipts raise production `pub(crate)` declarations 1,806 -> 1,808
   and the cross-slot upper bound 1,759 -> 1,761. The strengthened witness raises
   filesystem reads 360 -> 361; source-root mentions remain 118. Removing the
   suppression lowers allowances 8 -> 7; panics and expects remain 7 and 90.
8. **Fixed point and next frontier.** Realized popup geometry now crosses the
   native seam once as one named fact; no flattened constructor or suppression
   remains. Rung 5 continues with the variable-list layout allowance and then a
   complete reverse sweep over the UI owner.

### R5-09 — variable-list layout arity resistance ruling

Status: **complete; exceptional arity retained without a blanket allowance**.
Correction `4d51b881` (`Own variable list layout arity`).

1. **Question and complete trace.** The last UI argument-count suppression was
   traced from the virtual-list role through fixed/variable branch selection,
   scroll viewport derivation, retained measurement identity, sparse height
   refinement, table intrinsic measurement, materialized row placement,
   recursive child layout, viewport/request projection, same-range mode
   transition, rebuild, and bounded-work witnesses.
2. **Ruling and alternatives resisted.** Seven arguments are the established
   recursive layout traversal facts: view node, retained node, path, assigned
   rect, floating state, inherited clip, and layout context. The remaining
   three are already-resolved variable-list facts: viewport rect, provider
   model, and the selected retained measurements handle. Wrapping them would
   create a transport-only argument bag; re-deriving them inside the helper
   would repeat model/viewport policy and add another `Rc` measurement clone;
   inlining the helper would erase the useful fixed/variable algorithm boundary.
   No coherent domain aggregate or ownership seam is missing here.
3. **Correction and future retirement.** Replaced the blanket
   `#[allow(clippy::too_many_arguments)]` with a reasoned
   `#[expect(clippy::too_many_arguments, ...)]`. Clippy now proves that the
   exceptional shape still exists and will emit an unfulfilled-expectation
   warning if a future broader traversal refactor removes it. No parameter,
   type, call site, visibility, lookup, clone, or algorithm changed.
4. **Naming and visibility ruling.** No intermediate type was invented, so no
   compound/simple naming question or new projection exists. The existing
   `virtual_list::Measurements`, `Model`, and layout-local facts retain their
   exact names and visibility.
5. **Behavior and economics.** Fixed and variable selection, retained height
   identity, measurement refinement, table projection, scroll anchoring,
   materialization bounds, child order, layout output, allocation, reference
   counting, renderer topology, and presentation work are byte-for-byte
   unchanged; only lint governance changed.
6. **Proof and gauge delta from R5-08.** Both focused variable-list integration
   witnesses passed; all targets and all five examples compiled without
   warnings; a focused Clippy run satisfied the expectation with no
   `layout/algorithm.rs` or unfulfilled-expectation diagnostic. All nine census
   parser witnesses, the full census, formatting, diff, and protected-state
   checks passed. Every graph, visibility, test, source-root, filesystem, panic,
   and expect metric remains unchanged. Blanket `#[allow(...)]` attributes fall
   7 -> 6.
7. **Fixed point and next frontier.** Both Rung 5 argument-count sites now have
   explicit outcomes: popup geometry became a real crossing contract; variable
   layout arity remains explicit and self-retiring because no aggregate is
   honest. The seeded UI queue is exhausted, so the required reverse sweep now
   starts across every UI module and every cell dimension; queue exhaustion is
   not rung closure.

### R5-10 — atomic contextual path location

Status: **complete; parallel optional location facts collapsed**. Correction
`b94e14ac` (`Make context path location atomic`).

1. **Question and complete trace.** The reverse sweep traced contextual owners
   from retained-node traversal through table, row, cell, text, responder, and
   application frames into command-scope resolution and the white-box path
   witnesses. `ContextOwner` carried `table`, `row`, and `cell` as three
   parallel options even though a row or cell already carries its table key.
2. **Invalid states and repeated policy.** The old representation admitted
   contradictory table/row/cell combinations and forced runtime scope
   resolution to repeat `table.or_else(cell.table)`. Traversal happened to
   construct only four legitimate species: no location, table, row, or cell.
3. **Correction.** A private `Location::{None, Table, Row, Cell}` sum now owns
   that one contextual-path fact. `table()` is the single projection that
   derives table identity from every applicable species; `row()` and the
   test-only `cell()` project their exact species. Traversal constructs the
   species directly, and runtime consumes only `owner.table()`.
4. **Boundary and naming ruling.** `ContextOwner::new` is private to `view` and
   its descendants, while `cell()` leaves the production surface because only
   white-box tests inspect it. `Location` is an implementation-local simple
   name with no parent projection or alias, so the namesake-module and
   compound-name laws require no public spelling.
5. **Behavior and economics.** Context-path order, responder/focus/binding
   selection, table and text service frames, row command population, cell
   identity, application fallback, menu construction, hit behavior, rendering,
   allocation, and presentation work are unchanged. The refactor removes
   impossible state and one repeated fallback without adding a heap object or
   traversal.
6. **Doctrine and witnesses.** Master design now requires exactly one location
   species per contextual frame and forbids parallel table/row/cell facts. The
   architecture witness pins the four species, the absence of the old fields,
   centralized table projection, and retirement of the runtime cell fallback.
7. **Proof and gauge delta from R5-09.** The focused context-menu suite passed;
   the full library discovered 1,092 tests and passed 1,082 with 10 ignored;
   all targets compiled without warnings. All nine census parser witnesses,
   formatting, diff, and protected-state checks passed. Production and
   provisional graph metrics remain clean; the new architecture witness raises
   test-only module edges 109 -> 110. Production `pub(crate)` declarations fall
   1,808 -> 1,806 and the cross-slot upper bound falls 1,761 -> 1,759. All other
   gauges remain unchanged.
8. **Fixed point and next frontier.** Contextual location is now one keyed fact
   at construction and one derived table projection at consumption. The reverse
   sweep continues across the remaining UI state shapes; this cell does not
   close Rung 5.

### R5-11 — valid overlay realization capability species

Status: **complete; contradictory support booleans collapsed**. Correction
`7c7181d6` (`Make overlay capabilities a valid species`).

1. **Question and complete trace.** The reverse capability sweep traced overlay
   realization from headless defaults and native backend detection through
   backend selection, entry/exit animation policy, prepared-frame popup
   collection, runtime realization, native presentation, and the full overlay
   lifecycle witnesses.
2. **Invalid state.** `overlay::Capabilities` carried independent
   `native_popups` and `native_popup_animation` booleans. Its constructors
   happened to admit only three combinations, but the representation also
   admitted animation support without native popup support, a capability that
   has no realization path.
3. **Correction.** `Capabilities` is now the closed sum
   `InFrameOnly | AnimatedNativePopups | ImmediateNativePopups`. Existing
   constructors and queries remain the crossing API, so every caller and
   consumer retains its behavior while the impossible fourth combination is
   unrepresentable.
4. **Boundary and naming ruling.** No helper or intermediate type was added,
   visibility did not widen, and `overlay::Capabilities` remains module
   qualified at crossings. It has no compound declaration/simple projection or
   namesake-module parent re-export to collapse.
5. **Behavior and economics.** Native support detection, DX12 animation policy,
   in-frame fallback, backend resolution, enter/exit fade timing, retirement,
   popup surface collection, logging, rendering, allocation, and presentation
   topology are unchanged. Queries compile to direct enum tests rather than
   field reads and add no storage or traversal.
6. **Doctrine and witnesses.** Master design now names the three capability
   species and forbids animation without native realization. The architecture
   witness pins the sum and tombstones both support-boolean fields.
7. **Proof and gauge delta from R5-10.** The architecture witness and all 19
   overlay lifecycle tests passed; the full library discovered 1,093 tests and
   passed 1,083 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains
   unchanged.
8. **Fixed point and next frontier.** Overlay capability is now one valid
   realization species from platform detection through presentation. The
   reverse sweep continues across hit, interaction-receipt, and remaining UI
   state shapes; this cell does not close Rung 5.

### R5-12 — exclusive resolved-hit action source

Status: **complete; parallel optional hit claims collapsed**. Correction
`82205a6f` (`Make layout hit source exclusive`).

1. **Question and complete trace.** The reverse hit-state sweep traced initial
   acquisition for ordinary frames, late scrollbar chrome, table dividers, and
   input indicators through target projection, cursor admission, gesture
   intent, frame action lookup, text selection, release handling, row/cell
   provenance, and the clipped-chrome witnesses.
2. **Invalid state and repeated policy.** `layout::Hit` carried optional
   `chrome` and `target` fields alongside its frame. Constructors happened to
   populate neither or exactly one, but the representation admitted both, and
   every action lookup repeated the rule that either override suppresses the
   frame action.
3. **Correction.** A private `Kind::{Frame, Chrome, Target}` sum now owns the
   exclusive action source. Target projection and action lookup each match that
   fact once. Table-cell provenance remains an independent optional annotation
   because chrome, synthetic-target, and frame hits may all originate over a
   table cell.
4. **Boundary and naming ruling.** `Kind` is private to `layout::hit`; `Hit`'s
   crate surface and all constructors/queries are unchanged. No parent
   projection, compound/simple alias, or namesake-module export is introduced.
5. **Behavior and economics.** Reverse paint-order acquisition, ancestor clips,
   chrome priority, divider and indicator targeting, pointer cursor and intent,
   text hit mapping, action sequencing, cell gestures, allocation, cloning, and
   presentation work are unchanged. One enum discriminant replaces the two
   option discriminants and introduces no traversal or heap object.
6. **Doctrine and witnesses.** Master design now states that a hit has exactly
   one frame, chrome, or synthetic-target action source. The architecture
   witness pins the private sum and tombstones both optional claim fields;
   existing functional witnesses continue to pin chrome target/cursor behavior.
7. **Proof and gauge delta from R5-11.** The architecture witness and three
   focused scrollbar-chrome tests passed; the full library discovered 1,094
   tests and passed 1,084 with 10 ignored; all targets compiled without
   warnings. All nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains
   unchanged.
8. **Fixed point and next frontier.** Resolved hit ownership is now exclusive at
   construction and consumption. The reverse sweep continues through
   interaction receipts and the remaining UI inventories; this cell does not
   close Rung 5.

### R5-13 — valid interaction-pruning receipt species

Status: **complete; capture implication encoded**. Correction `d5e0be2e`
(`Make interaction pruning receipt valid`).

1. **Question and complete trace.** The reverse receipt sweep traced composition
   removal through hover, press, capture, scroll, draft, and contextual-menu
   pruning; window/session forwarding; prepared-frame interaction refresh;
   gesture cancellation; focus validation; and the removal/capture witnesses.
2. **Invalid state.** `interaction::Pruned` carried independent `changed` and
   `capture_removed` booleans. Its sole producer always included capture removal
   in `changed`, but the representation admitted `capture_removed = true` with
   `changed = false`, contradicting the mutation and its consumer protocol.
3. **Correction.** The receipt now contains one private
   `PruneOutcome::{Unchanged, Changed, CaptureRemoved}` fact. Existing
   `changed()` and `capture_removed()` queries preserve the crossing API;
   capture removal necessarily answers true to both.
4. **Boundary and naming ruling.** `Pruned` remains the crate-visible receipt;
   `PruneOutcome` is private to `interaction`, has no re-export or alias, and
   does not create a parent-module projection. No visibility widened.
5. **Behavior and economics.** Removal matching and mutation order, pointer
   clearing, scroll/draft/menu pruning, interaction cloning, gesture
   cancellation, logging, focus validation, allocation, and presentation work
   are unchanged. One enum discriminant replaces two booleans without a heap
   object or additional traversal.
6. **Doctrine and witnesses.** Master design now names the three receipt species
   and the capture-implies-change law. The architecture witness pins the closed
   outcome and tombstones both receipt booleans; functional witnesses continue
   to prove menu pruning and captured-gesture cancellation.
7. **Proof and gauge delta from R5-12.** The architecture witness and focused
   menu/capture removal tests passed; the full library discovered 1,095 tests
   and passed 1,085 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains
   unchanged.
8. **Fixed point and next frontier.** Pruning now returns one valid receipt from
   producer through runtime consumption. The reverse sweep continues across the
   remaining UI inventories and option/boolean clusters; this cell does not
   close Rung 5.

### R5-14 — valid realized-material parts receipt

Status: **complete; tint implication encoded**. Correction `27231efe`
(`Make realized material parts valid`).

1. **Question and complete trace.** The reverse scene-receipt sweep traced
   material requests from retained pane identity and native capability forecast
   through Windows composition/accent outcomes, popup reports, unique report
   matching, residual material subtraction, fidelity classification, renderer
   lowering, and all material lifecycle witnesses.
2. **Invalid state.** `scene::RealizedMaterialParts` carried independent
   `backdrop_frost` and `surface_tint` booleans. Its constructors admitted only
   no parts, frost, or frost plus tint, but the representation also admitted
   surface tint without backdrop frost, which no platform report can mean.
3. **Correction.** The receipt now contains one private
   `Parts::{None, Frost { surface_tint }}` fact. Existing `none()`, `frost()`,
   `backdrop_frost()`, and `surface_tint()` crossings are unchanged; tint can
   exist only within the frost species.
4. **Boundary and naming ruling.** `RealizedMaterialParts` retains its direct,
   unchanged crate-private projection from `scene::region`; private `Parts`
   does not escape or acquire an alias. No compound declaration is exposed
   under a simpler spelling and no visibility widened.
5. **Behavior and economics.** Platform forecast and realization, report
   identity/order, duplicate rejection, residual subtraction order, fidelity,
   popup material paths, renderer batching, allocation, and presentation work
   are unchanged. One enum replaces two booleans without a heap object or
   additional traversal.
6. **Doctrine and witnesses.** Master design now states that surface tint is a
   fact nested inside realized frost. The architecture witness pins that
   species and tombstones the old parallel receipt fields; the existing 38
   material-path witnesses continue to cover realization and fallback.
7. **Proof and gauge delta from R5-13.** The architecture witness and all 38
   focused material-path tests passed; the full library discovered 1,096 tests
   and passed 1,086 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains
   unchanged.
8. **Fixed point and next frontier.** Material realization now carries only
   reportable part combinations from platform receipt through residual paint.
   The reverse sweep continues across composition identity and remaining UI
   state shapes; this cell does not close Rung 5.

### R5-15 — exclusive structural reconciliation-key species

Status: **complete; parallel optional structural keys collapsed**. Correction
`cd1f3b48` (`Make composition key species exclusive`).

1. **Question and complete trace.** The reverse composition sweep traced
   ordinary positional siblings, explicit element ids, provided rows, table
   rows/cells/headers, dematerialization, sibling movement, removal reporting,
   cell tombstone deduplication, retained/layout identity namespaces, and all
   reconciliation consumers and witnesses.
2. **Co-occurrence and invalid state.** A table row legitimately carries both
   table-row semantics and provided-row identity, so those view facts remain
   separate. The private reconciliation `Key`, however, carried parallel
   optional provided-row, table-cell, and table-header-cell keys. Acquisition
   always prioritized exactly one; overlapping structural keys have no valid
   producer or matching meaning.
3. **Correction.** The existing private `Key` itself is now the sum
   `Ordinary | ProvidedRow | TableCell | TableHeaderCell`; no helper transport
   type was added. Role and axis remain inside every species and therefore
   still participate in derived equality/hash. Focused projections replace
   direct optional-field reads in matching, dematerialization, and removal.
4. **Boundary and naming ruling.** `Key` remains private to
   `composition::tree`, so its simple established name needs no parent
   projection. No compound/simple alias, re-export, or visibility change was
   introduced.
5. **Behavior and economics.** Matching priority, explicit-id behavior,
   positional fallback, provider dematerialization, table identity, removal
   order/deduplication, cloning, hashing, allocation, and presentation work are
   unchanged for every admitted node. One enum discriminant replaces three
   optional key discriminants and adds no heap object or traversal.
6. **Doctrine and witnesses.** Master design now states the four structural-key
   species and their exclusivity. The architecture witness pins the existing
   `Key` as that sum and tombstones all three optional fields; functional
   reconciliation and platform-composition witnesses remain green.
7. **Proof and gauge delta from R5-14.** The architecture witness, 17 focused
   framework composition tests, and nine native composition tests passed; the
   full library discovered 1,097 tests and passed 1,087 with 10 ignored; all
   targets compiled without warnings. All nine census parser witnesses, the
   full census, formatting, diff, and protected-state checks passed. Every
   graph, visibility, test-edge, source-root, filesystem, allowance, panic, and
   expect gauge remains unchanged.
8. **Fixed point and next frontier.** Structural reconciliation identity is now
   one species while legitimate higher-level row co-occurrence remains intact.
   The reverse sweep continues across view/widget/layout and remaining state
   shapes; this cell does not close Rung 5.

### R5-16 — atomic projected text-box caret

Status: **complete; cursor and selection collapsed into one optional unit**.
Correction `71887ea8` (`Make text box caret projection atomic`).

1. **Question and complete trace.** The reverse text-control sweep traced
   `TextBox` construction, draft/input projection, focus projection, cursor and
   selection queries, field shaping, caret reveal/blink scheduling, selection
   paint, pointer placement/drag, table editing, and the full text-box/cursor
   witness sets.
2. **Invalid state.** `view::TextBox` carried optional cursor and optional
   selection independently. Every producer installed selection only alongside
   a cursor and cleared both together, but the representation admitted a
   selection with no cursor to own its active endpoint.
3. **Correction.** One private optional `Caret { cursor, selection }` now owns
   the projected fact. Existing public `cursor()` and `selection()` queries
   remain unchanged; draft projection and focus fallback construct the unit,
   while unfocus or absent projection clears it atomically.
4. **Boundary and naming ruling.** `Caret` is private to the private
   `view::control::text_box` housing. Public `view::TextBox` keeps its established
   namesake projection unchanged; no support type, compound alias, or
   visibility escapes at either parent.
5. **Behavior and economics.** Draft authority, cached text, cursor fallback,
   selection range, field shaping, caret reveal/blink, paint order, pointer
   editing, allocation, and presentation work are unchanged. One optional
   aggregate replaces two options without a heap object or extra traversal.
6. **Doctrine and witnesses.** Master design now states that projected cursor
   and selection are one caret unit and that selection implies cursor. The
   architecture witness pins the private unit and tombstones the two parallel
   fields on `TextBox`.
7. **Proof and gauge delta from R5-15.** The architecture witness, all 26
   text-box tests, and the 16-test cursor slice passed; the full library
   discovered 1,098 tests and passed 1,088 with 10 ignored; all targets compiled
   without warnings. All nine census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed. Every graph,
   visibility, test-edge, source-root, filesystem, allowance, panic, and expect
   gauge remains unchanged.
8. **Fixed point and next frontier.** Text-box caret projection is now one fact
   from interaction draft through layout and paint. The reverse sweep continues
   across layout frame species, view-node role facts, widgets, and remaining UI
   state shapes; this cell does not close Rung 5.

### R5-17 — atomic resolved virtual-frame geometry

Status: **complete; viewport and request collapsed into one optional fact**.
Correction `a9fceac9` (`Make virtual frame geometry atomic`).

1. **Question and complete trace.** The reverse layout-frame sweep traced fixed
   and variable virtual-list layout through viewport resolution, range/overscan
   derivation, materialization requests, frame storage, request collection,
   runtime fixed-point materialization, hit-to-row mapping, scroll projection,
   refinement, jump, pinning, and bounded-work witnesses.
2. **Invalid state and ownership constraint.** `VirtualListContent` carried
   optional viewport and request fields even though both were installed by one
   `with_virtual_list` step and consumed as one resolved frame. Either half
   alone is invalid. Moving viewport into `virtual_list::Request` was rejected
   because it would drag layout-owned geometry into the lower provider seam.
3. **Correction.** A private frame-local optional
   `VirtualGeometry { viewport, request }` now owns the pair. Existing frame
   viewport/request queries project from it; row hit mapping consumes the same
   viewport. Unresolved frame construction carries no geometry.
4. **Boundary and naming ruling.** `VirtualGeometry` is private to
   `layout::frame`, is not re-exported, and crosses no seam. The public and
   crate-private `virtual_list` and `layout` spellings remain unchanged; no
   compound declaration is exposed under a simpler alias.
5. **Behavior and economics.** Fixed/variable range selection, retained
   measurement identity, viewport scroll, request collection, refinement,
   materialization, row hits, allocation, cloning, and presentation work are
   unchanged. One optional aggregate replaces two options without a heap object
   or added traversal.
6. **Doctrine and witnesses.** Master design now states the atomic frame-local
   pair and the lower-seam ownership constraint. The architecture witness pins
   the aggregate and tombstones both parallel fields on `VirtualListContent`.
7. **Proof and gauge delta from R5-16.** The architecture witness and all 15
   virtual-list tests passed; the full library discovered 1,099 tests and passed
   1,089 with 10 ignored; all targets compiled without warnings. All nine census
   parser witnesses, the full census, formatting, diff, and protected-state
   checks passed. Every graph, visibility, test-edge, source-root, filesystem,
   allowance, panic, and expect gauge remains unchanged.
8. **Fixed point and next frontier.** Resolved virtual geometry is now one fact
   without contaminating the provider seam. The reverse sweep continues across
   frame shortcut/popup facts, view nodes, widgets, and remaining UI state
   shapes; this cell does not close Rung 5.

### R5-18 — valid view-binding trigger species

Status: **complete; current slider trigger and factory made inseparable**.
Correction `b299aeaa` (`Make view binding trigger species valid`).

1. **Question and complete trace.** The reverse binding sweep traced typed and
   erased command bindings, resolved menu/bar actions, ordinary controls,
   fixed-command sliders, value-mapped sliders, state resolution, activation,
   history grouping, drag updates, command invocation, and binding/slider
   witnesses.
2. **Invalid state.** `view::Binding` carried one current `AnyTrigger` plus an
   optional slider value-trigger factory. Constructors kept them aligned, but
   the representation admitted a slider factory paired with an unrelated
   current trigger and made fixed-versus-slider meaning indirect.
3. **Correction.** A private `Trigger::{Fixed, Slider { current, factory }}` sum
   now owns the species. All trigger metadata, resolution, and invocation read
   the current member; slider updates can only derive a successor from the
   factory nested beside that member. Existing `Binding` queries and actions are
   unchanged.
4. **Boundary and naming ruling.** The public namesake `view::Binding` remains
   the sole projected type; private `Trigger` stays inside the private binding
   module and is not re-exported. No compound/simple alias or visibility change
   was introduced.
5. **Behavior and economics.** Command identity, state, route, source,
   description, history grouping, menu visibility, slider mapping, gesture
   coalescing, allocation, cloning, and presentation work are unchanged. One
   enum discriminant replaces the optional factory discriminant without a heap
   object or traversal.
6. **Doctrine and witnesses.** Master design now names fixed and slider trigger
   species and requires current/factory atomicity. The architecture witness pins
   the private sum and tombstones the optional factory field.
7. **Proof and gauge delta from R5-17.** The architecture witness, all six
   slider tests, and all five binding tests passed; the full library discovered
   1,100 tests and passed 1,090 with 10 ignored; all targets compiled without
   warnings. All nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains
   unchanged.
8. **Fixed point and next frontier.** View bindings now carry one valid trigger
   species from widget construction through command invocation. The reverse
   sweep continues across view-node roles, remaining layout payloads, session,
   theme, and scene inventories; this cell does not close Rung 5.

### R5-19 — distinct standard-menu projected-entry lifecycles

Status: **complete; catalog and authored entry state separated**. Correction
`f3e4d07a` (`Separate standard menu entry lifecycles`).

1. **Question and complete trace.** The reverse standard-menu sweep traced
   registry topology, platform catalog slots, live bar actions, missing/hidden
   nodes, authored item/section/category insertions and replacements, anchor
   ordering, marker retention, final node filtering, menu construction, and the
   complete command/menu witnesses.
2. **Valid absence and invalid overlap.** Catalog entries legitimately retain
   an optional standard marker even when no live node resolves, so node absence
   cannot simply disappear. Authored entries, by contrast, always carry a node
   and alone may carry an after-anchor. The prior three independent options
   admitted markerless/nodeless authored entries and anchors on catalog entries.
3. **Correction.** The existing private `ProjectedEntry` is now
   `Catalog { standard, node } | Authored { node, after }`; no helper type was
   added. Focused projections provide marker, anchor, and final-node views while
   preserving replacement and ordering algorithms.
4. **Boundary and naming ruling.** `ProjectedEntry` remains private to the
   private standard-menu module; the existing crate-private
   `StandardMenuExtension` housing/projection is untouched. No new re-export,
   compound/simple alias, or visibility change was introduced.
5. **Behavior and economics.** Platform topology, standard markers, hidden
   commands, authored ordering, section/category replacement, shortcut policy,
   final node order, allocation, cloning, and presentation work are unchanged.
   One enum discriminant replaces the mixed option set without a heap object or
   additional traversal.
6. **Doctrine and witnesses.** Master design now names the catalog and authored
   entry lifecycles and their distinct absence rules. The architecture witness
   pins the two species and tombstones the authored-anchor field on the old
   struct shape.
7. **Proof and gauge delta from R5-18.** The architecture witness, focused
   standard-menu tests, and all 30 command tests passed; the full library
   discovered 1,101 tests and passed 1,091 with 10 ignored; all targets compiled
   without warnings. All nine census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed. Every graph,
   visibility, test-edge, source-root, filesystem, allowance, panic, and expect
   gauge remains unchanged.
8. **Fixed point and next frontier.** Standard-menu projection now preserves
   legitimate marker absence without admitting cross-lifecycle state. The
   reverse sweep continues across the remaining view/layout/widget/session/
   theme/scene inventories; this cell does not close Rung 5.

### R5-20 — valid projected table-track species

Status: **complete; axis and column-only resize facts made inseparable**.
Correction `b44ff406` (`Make table track species valid`).

1. **Question and complete trace.** The reverse layout-table sweep traced track
   projection from table/header/row frames through resolved column geometry,
   row and column rule paint, floating-layer ordering, clipped divider hits,
   resize targets/actions, renderer scene inspection, rebuilds, sorting, scroll,
   and the complete table-layout witness set.
2. **Invalid state and ownership.** `layout::table::Track` carried an `Axis` and
   optional `Column` independently. Its two producers admitted only column
   tracks with resize identity/geometry and row tracks without those facts, but
   the representation also admitted both contradictory combinations. Layout
   remains the one owner of the resolved projection; no new seam is needed.
3. **Correction.** A private `Kind::{Column(Column), Row}` now owns the species.
   Axis is derived from it, and header identity, divider target, resize action,
   and hit geometry all consume one private column-fact projection. The parallel
   axis field, optional column field, and their implicit agreement are deleted.
4. **Boundary and naming ruling.** `Kind` remains private to `layout::table` and
   receives no projection or alias. Existing crate-visible `Track`, `Axis`, and
   their parent spellings remain unchanged; no compound declaration is exposed
   under a simpler re-export and no visibility widened.
5. **Behavior and economics.** Track order, boundaries, clipped rule geometry,
   floating-layer placement, header/row paint axes, divider hit zones, resize
   widths, rebuild identity, allocation, cloning, hashing, and presentation work
   are unchanged. One enum discriminant replaces the axis plus option
   discriminants without a heap object, lookup, or additional traversal.
6. **Doctrine and witness.** Master design now states the two table-track
   species and makes axis a projection rather than parallel truth. The
   architecture witness pins the private sum and tombstones the independent
   axis and optional-column fields.
7. **Proof and gauge delta from R5-19.** The focused architecture witness passed;
   the full library discovered 1,102 tests and passed 1,092 with 10 ignored; all
   targets compiled without warnings. All nine census parser witnesses, the
   full census, formatting, diff, and protected-state checks passed. Every
   graph, visibility, test-edge, source-root, filesystem, allowance, panic, and
   expect gauge remains unchanged.
8. **Fixed point and next frontier.** A projected table track now carries exactly
   the facts its row or column species can mean. The reverse sweep continues
   through scene animation, view/layout role state, widget/session/theme facts,
   and the remaining lifecycle and intermediate inventories; this cell does not
   close Rung 5.

### R5-21 — valid runtime visual-scalar species

Status: **complete; moving and resting scalar state separated**. Correction
`910b0518` (`Make visual scalar species valid`).

1. **Question and complete trace.** The reverse scene-animation sweep traced
   slider hover/press desire through runtime transition creation, retargeting,
   eased sampling, schedule continuation, visual storage and sanitization,
   scene transform/scale-motion projection, layout-to-paint snapping, renderer
   movement admission, and the slider and moving-geometry witnesses.
2. **Invalid state and derived facts.** `scene::visual::Scalar` stored value,
   endpoints, progress, and `Motion` independently. Its sole producer admitted
   only a moving transition with all four scalar facts or a resting value whose
   endpoints equal that value, progress is complete, and motion is resting. The
   struct also represented every disagreement among those facts.
3. **Correction.** `Scalar` itself is now
   `Moving { value, from, target, progress } | Resting { value }`. Existing
   value/endpoint/progress/motion queries derive the same downstream values, and
   sanitization preserves the species while sanitizing only its owned fields.
   The parallel motion field and redundant resting endpoint/progress storage are
   deleted without adding an intermediate type.
4. **Boundary and naming ruling.** The existing private-housing `Scalar` and its
   crate-private scene projection remain unchanged; the cell introduces no
   public name, parent projection, alias, or visibility change. In particular,
   no compound declaration is exposed under a simpler spelling.
5. **Behavior and economics.** Desire, transition endpoints, eased progress,
   current scale, sanitization, redraw scheduling, final resting pose, scene
   transforms, device snapping, subpixel motion, allocation, batching/pass
   fusion, and presentation clocks are unchanged. The resting species stores
   one scalar instead of five fields and all projections remain constant-time.
6. **Doctrine and witness.** Master design now names the moving and resting
   scalar species and their derived resting facts. The architecture witness pins
   the sum and tombstones an independently stored motion field on the scalar.
7. **Proof and gauge delta from R5-20.** The focused architecture witness and
   slider hover-animation witness passed; the full library discovered 1,103
   tests and passed 1,093 with 10 ignored; all targets compiled without warnings.
   All nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains unchanged.
8. **Fixed point and next frontier.** Runtime scalar visuals now carry only valid
   transition species from sampling through renderer admission. The reverse
   sweep continues through view/layout role facts, widget/session/theme state,
   popup/overlay lifecycle, and the complete visibility/failure inventories;
   this cell does not close Rung 5.

### R5-22 — valid pointer hover-tip lifecycle

Status: **complete; waiting time and visible anchor separated by lifecycle**.
Correction `ad24f9a3` (`Make hover tip lifecycle valid`).

1. **Question and complete trace.** The reverse pointer-lifecycle sweep traced
   presented-hit eligibility, same-target and changed-target projection,
   admission timing, deadline scheduling, pointer absence, promotion, captured
   reveal position, same-target pointer movement, dismissal on input/effects,
   view rebuild, floating-panel projection, and hover-panel witnesses.
2. **Invalid state and clocks.** Private `HoverTip` carried optional start time,
   a visible boolean, and optional anchor independently. Producers used only
   idle, waiting since an instant, and visible at a captured point, but the
   representation admitted visible-without-anchor, anchor-while-waiting, and
   idle-with-retained-time states. Admission time and reveal geometry are
   successive clocks, not simultaneous optional facts.
3. **Correction.** `HoverTip` is now
   `Idle | Waiting { started_at } | Visible { anchor }`. Eligibility compares
   against idle, deadlines exist only while waiting, promotion consumes the
   waiting instant and current pointer position into one visible anchor, and
   dismissal returns to idle. The three parallel fields and all agreement
   checks are deleted.
4. **Boundary and naming ruling.** `HoverTip` remains a simple private concept in
   `interaction::pointer`; it has no projection, alias, or visibility change.
   Public and crate-visible pointer/session queries retain their exact names and
   absence models.
5. **Behavior and economics.** Delay origin, target-change restart, unchanged
   eligibility, missing-position retry, frozen reveal point, dismissal result,
   schedule timing, view rebuild, placement, hit transparency, allocation, and
   presentation work are unchanged. One enum discriminant replaces two option
   discriminants plus a boolean without a heap object or new traversal.
6. **Doctrine and witnesses.** Master design now states the three hover-tip
   lifecycle species and distinguishes waiting time from visible geometry. The
   architecture witness tombstones the parallel fields; the owner witness now
   also pins the waiting deadline and its retirement upon visibility.
7. **Proof and gauge delta from R5-21.** The focused architecture and retained-
   anchor owner witnesses passed; the full library discovered 1,104 tests and
   passed 1,094 with 10 ignored; all targets compiled without warnings. All nine
   census parser witnesses, the full census, formatting, diff, and protected-
   state checks passed. Every graph, visibility, test-edge, source-root,
   filesystem, allowance, panic, and expect gauge remains unchanged.
8. **Fixed point and next frontier.** Hover-tip admission and revelation now
   advance through one valid lifecycle from presented hit to floating panel.
   The reverse sweep continues through overlay placement/context projections,
   view/layout role state, widget/session/theme facts, and all remaining
   visibility/failure inventories; this cell does not close Rung 5.

### R5-23 — exclusive active command-surface species

Status: **complete; menu and palette coexistence made unrepresentable**.
Correction `efa2168d` (`Make command surface exclusive`).

1. **Question and complete trace.** The reverse interaction-surface sweep traced
   menu open/toggle/switch/close, command-palette capture/open/navigation/close,
   pointer menu switching, key routing, contextual removal, query-draft cleanup,
   focus capture/restoration, view projection, visual state, and the explicit
   bidirectional replacement witnesses.
2. **Invalid state and distinct lifecycles.** `Interaction` stored optional open
   menu and optional command palette independently even though every admission
   path replaces the other. Their payloads and focus-restoration rules remain
   distinct, but simultaneous presence has no routing, projection, or product
   meaning and contradicted the replacement witnesses.
3. **Correction.** One private
   `Surface::{Menu(Menu), CommandPalette(CommandPalette)}` under one option now
   owns the active command surface. Existing menu and palette queries project
   their species; open/toggle/close, mutable palette navigation, and contextual
   pruning consume the same fact. The two parallel options are deleted.
4. **Boundary and naming ruling.** Private `Surface` uses the interaction-module
   namespace and receives no parent re-export or alias. Public `interaction::Menu`
   and all crate-visible `Interaction` queries retain their exact names and
   visibility; internal command-palette state remains unprojected publicly.
5. **Behavior and economics.** Menu switching, palette capture, query focus,
   restoration, text-draft cleanup, contextual pruning, pointer behavior,
   visual active state, overlay creation/retirement, allocation, and
   presentation work are unchanged. One option discriminant replaces two, with
   no heap object, callback, lookup, or traversal.
6. **Doctrine and witnesses.** Master design now states that a window has one
   active menu-or-palette command surface while preserving their distinct
   session focus lifecycles. The architecture witness pins the private sum and
   tombstones both former fields; existing replacement witnesses prove both
   transition directions.
7. **Proof and gauge delta from R5-22.** The focused architecture witness and
   both menu/palette replacement witnesses passed; the full library discovered
   1,105 tests and passed 1,095 with 10 ignored; all targets compiled without
   warnings. All nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Every graph, visibility, test-edge,
   source-root, filesystem, allowance, panic, and expect gauge remains unchanged.
8. **Fixed point and next frontier.** Command-surface interaction state now has
   one active species from admission through projection and dismissal. Overlay
   placement/context remains intentional independent state; the reverse sweep
   continues through view/layout role state, widget/session/theme facts, and
   the remaining visibility/failure inventories. This cell does not close
   Rung 5.

### R5-24 — atomic active text-input composition

Status: **complete; preedit lifetime nested beneath its active target**.
Correction `1465344a` (`Make draft preedit target atomic`).

1. **Question and complete trace.** The reverse draft-input sweep traced active
   targets, retained drafts, IME preedit/commit/disable, ordinary edits and
   selection, undo/redo, caret-blink epochs, TextBox and TextArea projection,
   table and palette text tasks, focus transfer, cancellation, snapshot restore,
   draft eviction, identity pruning, and window destruction.
2. **Invalid state and retained distinctions.** `draft::Input` stored an optional
   active target and optional preedit independently even though every preedit
   producer installed a target and every target-retirement path cleared
   preedit. Composition without a target had no producer or consumer meaning.
   A target without composition remains valid, and independently keyed drafts
   may lawfully outlive their active target, so neither distinction was merged.
3. **Correction.** One private optional `Active { target, preedit }` now owns the
   live text-input unit. Target and preedit queries project from it; activation,
   editing, selection, undo/redo, empty-preedit handling, deactivation, draft
   removal, pruning, and full clearing transition that unit atomically. The
   parallel top-level options and redundant two-field clearing protocol are
   deleted, while the draft store and caret epochs remain separately keyed.
4. **Boundary and naming ruling.** `Active` is a simple private supporting
   concept inside private `draft::input` housing. It receives no parent
   projection, alias, or visibility widening. Existing `draft::Input`, target,
   preedit, draft, feedback, and caret-epoch crossings retain their exact
   spellings.
5. **Behavior and economics.** IME routing, composition replacement and
   clearing, target switching, draft retention/eviction, feedback lifetime,
   selection history exclusion, caret blink resets, layout projection,
   invalidation, allocation, shaping, renderer topology, batching/pass fusion,
   and presentation clocks are unchanged. The same target and optional preedit
   values occupy one nested aggregate without a heap object, lookup, callback,
   or additional traversal.
6. **Doctrine and witnesses.** Master design now states that preedit is nested
   beneath the one active text target while retained drafts remain independent.
   The architecture witness requires the private active unit and tombstones the
   former optional target field. A new owner witness covers both preedit-only
   retirement and draft-backed target retention after composition clears.
7. **Proof and gauge delta from R5-23.** All fifteen preedit-focused journeys
   passed; the full library discovered 1,106 tests and passed 1,096 with 10
   ignored; all targets compiled without warnings. All nine census parser
   witnesses, the full census, formatting, diff, and protected-state checks
   passed. Every graph, visibility, test-edge, source-root, filesystem,
   allowance, panic, and expect gauge remains unchanged.
8. **Fixed point and next frontier.** Active text input now cannot carry
   composition without its target, while inactive retained drafts remain
   untouched. The reverse sweep continues through interaction target capture,
   pointer/session lifecycles, view/layout role facts, widget/theme state, and
   the remaining visibility/failure inventories. This cell does not close
   Rung 5.

### R5-25 — interaction target provenance and capture resistance

Status: **complete resistance ruling; no production correction admitted**.

1. **Question and complete trace.** The reverse target sweep traced every
   `interaction::Target` constructor through retained/node/table-cell identity,
   command binding provenance, view-role projection, pointer down/move/up,
   capture and cursor retention, gesture-history coalescing, removal pruning,
   menus, palettes, scrollbars, text selection, sliders, and hit routing.
2. **Co-occurrence matrix.** Source and capture are independent axes with all
   four live combinations: an ordinary command button has source without
   capture; a slider command has source and capture; a passive label has
   neither; and text, scroll, scrollbar, and divider targets capture without a
   command source. No combination is invalid or unproduced.
3. **Authority ruling.** Source is command provenance and participates in target
   equality/hash so menu and palette identity remain distinct. Capture is
   pointer-routing policy, deliberately excluded from semantic identity and
   consumed separately with the resolved cursor. Deriving capture from kind
   would fail for command buttons versus sliders; nesting it under source would
   fail for text and scrolling; merging the axes into species would encode the
   full valid product less directly.
4. **Reduction and naming ruling.** No field, constructor, helper, visibility,
   name, or call site changed. The simple existing `Target` and its namespaced
   `interaction::Kind` retain their public spellings; no intermediate or alias
   was invented merely to replace a valid boolean.
5. **Behavior and economics.** Target equality, focus keys, command provenance,
   capture admission/release, retained cursor, drag routing, gesture-history
   grouping, pruning, allocation, layout, scene, renderer, and presentation
   work remain byte-for-byte unchanged.
6. **Proof and gauge.** The ordinary command, captured command slider, and
   captured text-area witnesses passed directly; passive target construction is
   covered by the target and hover suites. The R5-24 full proof and every gauge
   remain the current boundary because this cell changes only the ledger.
7. **Fixed point and next frontier.** Target source/capture remains an
   intentional non-merge with a complete product of valid states. The reverse
   sweep continues through pointer/session state, view/layout roles,
   widget/theme facts, and the visibility/failure inventories; this cell does
   not close Rung 5.

### R5-26 — atomic pointer press lifecycle

Status: **complete; target, intent, and conditional capture made one unit**.
Correction `90cfcdae` (`Make pointer press lifecycle atomic`).

1. **Question and complete trace.** The reverse pointer-state sweep traced raw
   and resolved pointer down, hover replacement, captured and uncaptured
   presses, cursor retention, activation-versus-manipulation intent, drag
   demotion, release, pointer leave, cancellation, target removal, gesture
   cancellation, click classification, text selection, sliders, table
   dividers, scroll chrome, menus, palettes, and the session/runtime action
   crossings.
2. **Invalid states and retained species.** `Pointer` stored optional pressed
   target, capture, and press intent independently even though every press has
   one target and intent, and capture—when present—owns that same target plus
   its resolved cursor. The representation admitted target without intent,
   intent without target, capture without a press, and capture of a different
   target. Captured and uncaptured presses both remain valid: pointer leave
   retains the former and retires the latter.
3. **Correction.** One private optional
   `Press::{Captured { capture, intent }, Uncaptured { target, intent }}` now
   owns the lifecycle. Press admission derives the species from the target's
   capture policy; target, capture, and activation queries project from it;
   drag intent changes the nested fact; release, cancellation, leave, and
   removal retire the unit atomically. The three parallel options and duplicate
   captured-target storage are deleted.
4. **Boundary and naming ruling.** In the touched namesake seam,
   `interaction::Pointer` is the sole parent projection. The supporting
   `Capture`, `ClickCount`, and `PressIntent` declarations remain simple names
   inside crate-visible `interaction::pointer` housing and call sites qualify
   them as `interaction::pointer::Type`; no compound declaration, aliased
   projection, or flattened supporting re-export remains. `Press` stays private
   to its owner.
5. **Behavior and economics.** Hover replacement, capture cursor, press
   classification, activation, manipulation, drag routing, leave behavior,
   click counting, gesture grouping/cancellation, target pruning, allocation,
   layout, scene, renderer topology, and presentation work are unchanged. The
   captured species stores its target once rather than in both pressed and
   capture state; one outer option and one species replace three option
   discriminants without a heap object, lookup, callback, or traversal.
6. **Doctrine and witnesses.** Master design now states the one press lifecycle,
   its two species, and their retirement laws. The architecture witness pins
   the sum and tombstones all three parallel fields; the existing resolved-
   cursor witness now consumes the namespaced capture support type. Focused
   leave, drag, capture-removal, and architecture witnesses passed.
7. **Proof and gauge delta from R5-25.** The full library discovered 1,107 tests
   and passed 1,097 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Production/test edges remain 325/110; split
   responsibilities, slot edges, forbidden edges, external violations, and
   SCCs remain 3, 54, 0, 0, and 0. Making the supporting module an explicit
   crate crossing raises production `pub(crate)` declarations 1,806 -> 1,807
   and the cross-slot upper bound 1,759 -> 1,760. Cross-slot test edges,
   source-root mentions, filesystem reads, allowances, panics, and expects
   remain 90, 118, 361, 6, 7, and 90.
8. **Fixed point and next frontier.** Pointer press state now carries only valid
   target/intent/capture combinations from admission through retirement, and
   the touched seam practices the canonical parent-projection law. The reverse
   sweep continues through pointer position/surface, session lifecycles,
   view/layout roles, widget/theme facts, and the remaining visibility/failure
   inventories; this cell does not close Rung 5.

### R5-27 — atomic pointer location

Status: **complete; coordinate and owning surface given one presence
lifetime**. Correction `4113956d` (`Make pointer location atomic`).

1. **Question and complete trace.** The reverse pointer-location sweep traced
   parent and native-popup move/down/up/drag/scroll events, retained-coordinate
   translation, modifier-only reprojection, presented-layout hover refresh,
   hover-tip promotion, pointer leave, capture retention, cursor resolution,
   hit testing, and every position/surface consumer. Every producer supplied a
   point and surface together; every consumer used the surface only with that
   point; leave alone retired presence.
2. **Invalid state and coordinate ownership.** `Pointer` stored optional logical
   position beside an independently total `popup::Surface`. The representation
   admitted absence with stale native-popup identity, while the setter admitted
   an absent point paired with any surface even though no caller produced that
   state. The point is meaningful only in the retained coordinate space named
   by its owning parent or popup surface.
3. **Correction and repeated-path reduction.** One optional
   `pointer::Location { point, surface }` now owns presence. The renamed
   `set_pointer_location` requires both facts, the sole `location()` query
   returns them atomically, and pointer leave removes the unit. Separate
   position/surface fields and queries, the optional setter argument, the
   default-Parent stale-surface convention, and a duplicate session lookup in
   presented hover refresh are deleted.
4. **Boundary and naming ruling.** `interaction::Pointer` remains the sole
   parent projection. `Location` is a simple supporting declaration under
   crate-visible `interaction::pointer`, is consumed through that namespace
   where named, and is not flattened or aliased at the parent. Renaming the
   setter from position to location is admitted by the clarified coordinate-
   space axis; no public application spelling or unrelated name changed.
5. **Behavior and economics.** Parent/popup routing, retained point values,
   surface-qualified hits, modifier refresh, hover projection, tip anchors,
   capture cursor behavior, departure fallback point, allocation, layout,
   scene, renderer topology, and presentation clocks are unchanged. The same
   two Copy facts occupy one optional aggregate; consumers perform one
   interaction lookup rather than independently re-pairing projections, with
   no heap object, callback, extra lookup, or traversal.
6. **Doctrine and witnesses.** Master design now states that pointer presence
   owns point and surface together and absence owns neither. The architecture
   witness pins the aggregate, atomic query, and renamed setter while
   tombstoning split storage and the old setter. A direct owner witness covers
   native surface retention and absence; parent departure, native-popup event,
   popup hit isolation, popup hover paint, and retained-tip-anchor witnesses
   passed.
7. **Proof and gauge delta from R5-26.** The full library discovered 1,109 tests
   and passed 1,099 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Production edges remain 325; the additional
   architecture receipt raises test-only edges 110 -> 111. Split
   responsibilities, slot edges, forbidden edges, external violations, SCCs,
   and cross-slot test edges remain 3, 54, 0, 0, 0, and 90. The explicit
   location contract raises production `pub(crate)` declarations 1,807 ->
   1,809 and the cross-slot upper bound 1,760 -> 1,762. Source-root mentions,
   filesystem reads, allowances, panics, and expects remain 118, 361, 6, 7,
   and 90.
8. **Fixed point and next frontier.** Pointer presence can no longer carry a
   coordinate without its surface or retain a surface after departure. The
   reverse sweep continues through session window clocks and lifecycles,
   view/layout role facts, widget/theme state, and the remaining visibility,
   failure, and intermediate inventories; this cell does not close Rung 5.

### R5-28 — valid session cursor-publication species

Status: **complete; cursor value and pending handoff made one lifecycle**.
Correction `7e008361` (`Make cursor publication state valid`).

1. **Question and complete trace.** The reverse session-window sweep traced
   cursor resolution from pointer hits, capture, modifiers, successful
   presentation, stationary-pointer reprojection, direct test injection, update
   draining, platform deduplication, backend application, open/restore, and
   window destruction. It also tested the neighboring invalidation, projected-
   revision, desired/acknowledged epoch, focus-restoration, dialog, feedback,
   and interaction fields for the same correlation.
2. **Invalid state and resistance matrix.** Current cursor and
   `cursor_changed` described one synced-or-pending publication lifecycle, yet
   were independently representable. The neighboring facts resist merging:
   invalidation may clear while the desired epoch remains; retry invalidation
   deliberately does not mint an epoch; projected revision may precede or lag
   presentation; acknowledgement advances only on successful presentation;
   focus, optional menu restoration, dialogs, feedback, and interaction each
   have separately witnessed absence and cleanup lifetimes.
3. **Correction.** One private
   `Cursor::{Synced(pointer::Cursor), Pending(pointer::Cursor)}` now owns both
   the current value and publication state. Resolving a different cursor enters
   `Pending`; resolving the same value preserves the species; draining returns
   the pending value once and advances it to `Synced`. The parallel dirty flag,
   agreement protocol, and field-level mutations are deleted.
4. **Boundary and naming ruling.** Private `session::window::Cursor` is a simple
   owner-local supporting concept and receives no parent projection or alias.
   Public `session::Window::cursor`, `pointer::Cursor`, and `pointer::Update`
   retain their established names and signatures. No visibility widened and no
   compound declaration is exposed under a simpler spelling.
5. **Behavior and economics.** Cursor semantics, same-value suppression,
   pending replacement by the newest value, one-shot drain, window order,
   platform active-cursor deduplication, backend calls, redraw independence,
   capture, layout, scene, renderer topology, and presentation clocks are
   unchanged. One enum discriminant replaces the boolean beside the same Copy
   cursor value without allocation, lookup, callback, or traversal.
6. **Doctrine and witnesses.** Master design now names synced and pending cursor
   publication species. The architecture witness pins the sum and tombstones
   the dirty flag. A direct owner witness covers initial sync, value visibility
   while pending, one-shot handoff, final sync, and same-value suppression; all
   eighteen cursor-focused behavior witnesses passed.
7. **Proof and gauge delta from R5-27.** The full library discovered 1,111 tests
   and passed 1,101 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every gauge remains unchanged: production/
   test edges 325/111, split responsibilities 3, slot edges 54, forbidden/
   external/SCC counts 0/0/0, production `pub(crate)` declarations 1,809 in
   190 files, cross-slot upper bound 1,762, cross-slot test edges 90,
   source-root mentions 118, filesystem reads 361, allowances 6, panics 7, and
   expects 90.
8. **Fixed point and next frontier.** Session cursor publication now advances
   through one valid state while the independently clocked session facts remain
   explicit resistance outcomes. The reverse sweep continues through menu
   focus restoration, view/layout role facts, widget/theme state, and the
   remaining visibility, failure, and intermediate inventories; this cell does
   not close Rung 5.

### R5-29 — contextual-menu pruning retires captured focus

Status: **complete; cross-owner cleanup made explicit**. Correction `b4e00498`
(`Retire pruned menu focus capture`).

1. **Question and complete trace.** The reverse menu-focus sweep traced authored
   and contextual menu opening, switching, toggling, explicit close, outside-
   surface dismissal, command activation, palette replacement, focus capture
   and restoration, contextual-owner reconciliation, command-scope resolution,
   window restore, and destruction. Every ordinary session path retired
   `menu_restore_focus` with the menu except interaction-owned contextual-menu
   pruning.
2. **Stale authority and reopened evidence.** Removing a contextual owner made
   `interaction::Interaction` close its menu directly, but the session-owned
   captured focus survived. `Session::command_focus` then preferred that stale
   capture over later live focus even though no menu surface remained. This new
   downstream consequence legitimately reopens R5-13's three-species pruning
   receipt: capture removal was not the only cleanup fact that had to cross the
   interaction/session boundary.
3. **Correction.** `PruneOutcome` is now
   `Unchanged | Changed { capture_removed, menu_removed }`. The outer species
   makes every consequence imply change, while the two consequences remain
   independent and may coexist. Interaction reports contextual-menu removal;
   the session wrapper consumes that receipt and clears `menu_restore_focus`
   without restoring a possibly removed owner. The stale capture path is
   deleted.
4. **Boundary and resistance ruling.** Interaction continues to own command-
   surface identity and contextual-owner pruning; session continues to own
   keyboard focus and restoration. Moving `Focus` into interaction, duplicating
   menu activity in a session enum, or inferring cleanup after the fact would
   blur those owners. The narrow cleanup receipt is the honest crossing and
   existing explicit-close, outside-dismissal, palette-replacement, restore,
   and destruction paths retain their distinct policies.
5. **Naming and visibility ruling.** `Pruned` remains the crate-visible central
   receipt and private `PruneOutcome` remains unprojected. The supporting
   `menu_removed()` query is visible only across the interaction/session module
   boundary; no parent re-export, alias, compound declaration, or public
   application spelling was added.
6. **Behavior and economics.** Menu placement, switching, command resolution
   while live, focus restoration on ordinary close, non-restoring outside
   dismissal, palette capture, composition order, scene/overlay paint,
   renderer topology, presentation clocks, allocation, and frame work are
   unchanged. Owner removal still closes the contextual menu without restoring
   focus; it now also retires the capture that had become meaningless.
7. **Proof and gauge delta from R5-28.** The focused architecture witness and
   contextual-owner removal journey passed, including a live-focus replacement
   that would previously have been shadowed. The full library discovered 1,111
   tests and passed 1,101 with 10 ignored; all targets compiled without
   warnings. All nine census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed. Production/test edges remain
   325/111; split responsibilities, slot edges, forbidden edges, external
   violations, SCCs, and cross-slot test edges remain 3, 54, 0, 0, 0, and 90.
   The explicit cleanup query raises production `pub(crate)` declarations
   1,809 -> 1,810 and the cross-slot upper bound 1,762 -> 1,763. Source-root
   mentions, filesystem reads, allowances, panics, and expects remain 118, 361,
   6, 7, and 90.
8. **Fixed point and next frontier.** A contextual menu and its captured focus
   now retire together across the owner boundary, and no other direct menu-
   retirement path leaves a capture behind. The reverse sweep continues through
   the partial focus/target APIs, view/layout role facts, widget/theme state,
   and the remaining visibility, failure, and intermediate inventories; this
   cell does not close Rung 5.

### R5-30 — explicitly optional focus-to-text-target projection

Status: **complete; partial public conversions retired**. Correction `d925281d`
(`Make focus text targets explicitly optional`).

1. **Question and complete trace.** The reverse focus/target sweep traced text,
   table-cell, and control focus from construction through session focus,
   composition target projection, ordinary character input, selection, IME
   preedit and commit, local drafts, focused services, text drop, caret reveal
   and blink, TextBox/TextArea projection, table editing, snapshot tests, and
   every conversion call site. Text and table-cell focus have text targets;
   control focus is a keyboard destination with no text target.
2. **Partial API and reachable failure.** `Focus::target` panicked for table-cell
   and control focus, `Focus::into_target` expected every focus to be textual,
   and `Target::text_area(Focus)` publicly presented that partial conversion as
   total. A printable key or IME preedit while an ordinary control held focus
   could reach the conversion and panic. Safe `Focus::target_id` and
   `Focus::text_target` already represented the two distinct optional
   projections, so the total-looking routes preserved no independent invariant.
3. **Correction and displaced path.** Deleted all three partial public methods
   and migrated runtime input, session draft operations, focused services,
   view/layout target projection, widgets, and tests to `text_target()` or
   `target_id()` according to the fact they need. Every text consumer now
   handles absence through its existing false, `None`, not-attempted, or ignored
   outcome; no compatibility shim, fallback target, panic, or duplicated
   species check survives.
4. **Boundary and naming ruling.** Session continues to own focus species and
   interaction continues to own target representation. The one narrow
   `Option<interaction::Target>` projection is the honest crossing; moving focus
   into interaction or making runtime rediscover the species would blur those
   owners. Existing simple `Focus`, `Target`, `target_id`, and `text_target`
   names remain exact. No compound declaration, aliased projection, supporting
   parent re-export, or unrelated rename was introduced.
5. **Behavior and economics.** Text and table-cell editing, draft retention,
   selection, history, feedback, reveal, caret blink, text drop, table-cell
   identity, view projection, shaping, scene order, renderer topology,
   presentation clocks, allocation, and lookup work are unchanged. Control-
   focused printable input and preedit now take the ordinary unavailable path
   instead of violating the focus species; successful text paths perform the
   same constant-time match that the deleted conversions performed.
6. **Doctrine and witnesses.** Master design now names all three focus species,
   the optional text-target law, and absence behavior. The architecture witness
   requires the optional projection and tombstones all three partial APIs. A
   runtime behavior witness proves both printable input and IME preedit are
   ignored without changing state or focus while a control owns focus; all 26
   text-box, 28 text-input, and 13 focus witnesses passed.
7. **Proof and gauge delta from R5-29.** The full library discovered 1,113 tests
   and passed 1,103 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Production/test edges remain 325/111; split
   responsibilities, slot edges, forbidden edges, external violations, SCCs,
   production `pub(crate)`, cross-slot upper bound, cross-slot test edges,
   source-root mentions, filesystem reads, and allowances remain 3, 54,
   0/0/0, 1,810, 1,763, 90, 118, 361, and 6. Deleting the partial methods lowers
   production panics 7 -> 6 and expects 90 -> 89.
8. **Fixed point and next frontier.** Focus-to-text conversion is now total only
   through an explicit option, and no text ingress assumes all keyboard focus is
   editable. The reverse sweep continues through view/layout role facts,
   widget/theme state, the remaining session lifecycles, and the full
   visibility/failure/intermediate inventories; this cell does not close Rung
   5.

### R5-31 — one command-focus precedence ladder

Status: **complete; repeated semantic fallback centralized**. Correction
`2e015c1a` (`Centralize command focus precedence`).

1. **Question and complete trace.** The reverse focus-policy sweep traced live
   window focus, active text targets, menu opening/switching/toggling/closing,
   contextual pruning, outside dismissal, command-palette replacement and
   capture, command resolution, focused-text preparation, command-scope
   construction, restoration, window restore, and destruction. It compared
   `Session::command_focus` with the separate menu-capture fallback rather than
   assuming their similar syntax shared meaning.
2. **Repeated policy and precedence.** Both paths ask for the focus against
   which commands operate and transient command surfaces later restore. The
   complete precedence is command-palette capture, open-menu restoration
   capture, live window focus, then an active text target projected back to
   focus. The menu helper independently reconstructed only the final two rungs;
   the agreement depended on call order and could drift from command
   resolution after another focus species or capture rule was added.
3. **Correction and displaced path.** Private `Window::command_focus` now owns
   the entire ladder. The session crossing delegates to it, and first menu open
   or toggle captures that same resolved fact before installing the surface.
   The standalone `restore_focus_for_menu` algorithm and its duplicate
   live-focus/text-target fallback are deleted; no callback, cache, new state,
   or compatibility route replaces them.
4. **Boundary and naming ruling.** Session window state already owns live focus,
   menu restoration, and the interaction values needed for palette and text
   projections, so the private window method is the lowest honest policy owner.
   Interaction still owns surface identity and draft target state. Existing
   `command_focus`, `Focus`, and `Window` names remain exact; no compound type,
   alias, flattened supporting export, or public spelling was introduced.
5. **Behavior and economics.** Menu and palette replacement, captured command
   routing, menu switching, restoration, contextual pruning, focused text
   commit/deactivation, live focus, active-draft fallback, view resolution,
   allocation, layout, scene order, renderer topology, and presentation clocks
   are unchanged. The same constant-time option ladder is evaluated at the same
   entrances; one duplicate evaluation body is gone.
6. **Doctrine and witnesses.** Master design now records the four-rung ladder
   and forbids consumers from reconstructing subsets. The architecture witness
   pins the one window owner, both menu consumers, session delegation, and the
   old-helper tombstone. A direct owner witness walks active-draft fallback,
   menu capture over later live focus and restoration, then palette capture over
   later live focus and restoration. The contextual-pruning, fourteen palette,
   and thirteen focus journeys also passed.
7. **Proof and gauge delta from R5-30.** The full library discovered 1,115 tests
   and passed 1,105 with 10 ignored; all targets compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every gauge remains unchanged: production/
   test edges 325/111, split responsibilities 3, slot edges 54, forbidden/
   external/SCC counts 0/0/0, production `pub(crate)` 1,810 in 190 files,
   cross-slot upper bound 1,763, cross-slot test edges 90, source-root mentions
   118, filesystem reads 361, allowances 6, panics 6, and expects 89.
8. **Fixed point and next frontier.** Command focus now has one policy owner and
   no consumer-local fallback algorithm. The reverse sweep continues through
   view/layout role facts, widget/theme state, remaining lifecycle shapes, and
   the complete visibility/failure/intermediate inventories; this cell does not
   close Rung 5.

### R5-32 — one structural view-node content truth

Status: **complete; role and mutually exclusive payload made inseparable**.
Correction `31b62d65` (`Make view node content structural`).

1. **Question and complete trace.** The reverse view-role sweep traced every
   `Node` constructor and builder through widgets, tables, virtual-list
   materialization, standard-menu projection, command palettes, contextual and
   feedback panels, retained composition, layout frame construction, pointer
   targets, text commits, focus projection, scene production, and public model
   inspection. It covered ordinary and table scrolls, committed and
   uncommitted text boxes, standard and ordinary menu bars, interactive and
   hit-transparent floating panels, role-changing layout containers, and
   removal/rebuild lifecycles.
2. **Invalid states and co-occurrence resistance.** `Node` stored `Role` beside
   independent optional control, virtual-list, table-scroll, text-commit,
   standard-menu, scroll-offset, and floating-panel facts. The representation
   admitted payload-less controls, payloads under unrelated roles, table models
   on arbitrary nodes, commit capability outside text boxes, standard-menu
   extensions on ordinary bars, and floating facts on nonfloating nodes. By
   contrast, identity, axis, style, subject/label, binding, focus/selection,
   provided/table identity, participation, context-menu eligibility, and
   children all have witnessed cross-role combinations and remain common.
3. **Correction and displaced paths.** One private `Content` sum is now the
   role truth and owns control models directly, ordinary/table scroll species,
   virtual-list model and offset, text-box commit capability, ordinary/standard
   menu-bar species, and floating-panel state. `Role` is derived. The redundant
   private `Control` sum, copied role, parallel payload fields,
   `with_table_model`, and `with_text_commit` are deleted; table scroll and
   committed text-box constructors admit those capabilities structurally.
4. **Boundary and naming ruling.** Public `view::Node` and each concrete control
   model retain their established simple names. Private `Content`, `MenuBar`,
   `Scroll`, and `Panel` stay in private node housing and receive no parent
   projection, alias, or flattened supporting export. The cell introduces no
   compound declaration under a simpler re-export and does not reopen unrelated
   naming cleanup.
5. **Behavior and economics.** Construction, binding, standard-menu
   projection, virtual materialization, table horizontal scrolling, text
   commit lookup, floating placement/policy, focus, hit testing, layout, scene
   order, renderer topology, batching/pass fusion, invalidation, and every
   presentation clock are unchanged. The same owned models and Copy facts move
   through exhaustive constant-time matches; no heap allocation, callback,
   lookup, traversal, or frame work was added.
6. **Doctrine and witnesses.** Master design now names the common node envelope,
   one private typed content truth, and the justified common annotations. The
   architecture witness pins structural control, scroll, virtual-list,
   text-commit, standard-menu, and floating-panel species; tombstones every
   displaced field and builder; and strengthens the prior control-privacy
   witness to forbid restoration of any parallel `Control` enum.
7. **Proof and gauge delta from R5-31.** The structural witnesses, 39 text-box,
   15 virtual-list, 60 table, two standard-menu, and ten floating-panel tests
   passed. The full library discovered 1,116 tests and passed 1,106 with 10
   ignored; all targets and all five examples compiled without warnings. All
   nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Every gauge remains unchanged: production/
   test edges 325/111, split responsibilities 3, slot edges 54, forbidden/
   external/SCC counts 0/0/0, production `pub(crate)` 1,810 in 190 files,
   cross-slot upper bound 1,763, cross-slot test edges 90, source-root mentions
   118, filesystem reads 361, allowances 6, panics 6, and expects 89.
8. **Fixed point and next frontier.** View role and its exclusive payload now
   have one structural owner from construction through layout projection, while
   genuine cross-role annotations remain explicit resistance outcomes. The
   reverse sweep continues through remaining layout/view projections,
   widget/theme state, session lifecycles, and the complete visibility,
   failure, and intermediate inventories; this cell does not close Rung 5.

### R5-33 — structural floating-panel state through layout

Status: **complete; attachment, communication, and derived frame facts made
valid**. Correction `4ec5958c` (`Make floating panel state structural`).

1. **Question and complete trace.** The continuation of the view/layout role
   sweep traced generic floating widgets, authored menus, contextual menus,
   command palettes, hover tips, window feedback, root placement and nested
   available bounds, retained element anchors, auxiliary measurement/paint,
   focus collection, semantic overlay drafts, native popup preference/context,
   retirement, and in-frame fallback. It followed every floating-only fact from
   node construction through `FrameContent` and scene projection.
2. **Invalid states and independent axes.** View panel state stored an optional
   attachment beside optional available geometry, and stored policy beside an
   optional hint. Available bounds have meaning only for a geometry attachment;
   pointer and element attachment cannot own them. Interactive panels carry no
   auxiliary explanation, while hover-tip and window-feedback panels always
   communicate one `Hint` and never accept input. Layout then copied six
   floating-only facts onto every `Frame`. Placement mode, popup context,
   material preference, force-group diagnostics, and attachment species remain
   independent inside a real floating panel and were deliberately not merged.
3. **Correction and displaced paths.** `PanelAttachment::Geometry` now owns its
   optional available rectangle. `PanelPolicy` is now
   `Interactive | HoverTip(Hint) | WindowFeedback(Hint)`, so communication and
   hit transparency are one fact. The parallel panel availability and hint
   fields, `placement_available`, and `with_auxiliary_hint` are deleted. Layout
   `FrameContent::FloatingPanel` now owns a private payload containing popup
   placement/context, material/group preferences, and the same policy; ordinary
   frames cannot carry any of them.
4. **Boundary and naming ruling.** Existing `Node`, `PanelPolicy`,
   `PanelAttachment`, `Frame`, and public widget spellings remain exact. Private
   `FloatingPanelContent` and feedback's construction-only `Auxiliary` species
   receive no projection or alias. No supporting public re-export, compound
   declaration under a simpler name, compatibility path, or unrelated rename
   was introduced.
5. **Behavior and economics.** Edge flipping, available-bound clamping,
   pointer clearance, element lookup, menu and palette focus, hover and window
   feedback, hit transparency, auxiliary icons, overlay identity, popup
   material/context, native/in-frame selection, scene order, batching/pass
   fusion, invalidation, retirement, and presentation clocks are unchanged.
   The same owned `Hint` and Copy placement values move through exhaustive
   matches without a heap allocation, callback, lookup, traversal, or frame
   operation.
6. **Doctrine and witnesses.** Master design now states the attachment species,
   hint-owning communication policies, and role-local layout payload. The new
   architecture witness follows those facts through view and layout, forbids
   the displaced builders/projections, and requires communication policy to own
   its hint. The existing `FrameContent` witness now also tombstones every
   floating-only common-frame field.
7. **Proof and gauge delta from R5-32.** The two structural witnesses, 17 hover,
   seven feedback, eleven floating-panel, nine context-menu, and 84 popup tests
   passed with one standing ignored GPU diagnostic. The full library discovered
   1,117 tests and passed 1,107 with 10 ignored; all targets and all five
   examples compiled without warnings. All nine census parser witnesses, the
   full census, formatting, diff, and protected-state checks passed. Production/
   test edges remain 325/111; split responsibilities, slot edges, forbidden/
   external/SCC counts, and cross-slot test edges remain 3, 54, 0/0/0, and 90.
   Removing the independent hint builder lowers production `pub(crate)` 1,810
   -> 1,809 and the cross-slot upper bound 1,763 -> 1,762. Source-root mentions,
   filesystem reads, allowances, panics, and expects remain 118, 361, 6, 6,
   and 89.
8. **Fixed point and next frontier.** Floating-panel semantic and derived state
   is now role-local at both owners, and communication cannot block operation
   because its hint-owning species is intrinsically hit-transparent. The reverse
   sweep continues through remaining layout projections, widget/theme state,
   session lifecycles, and the complete visibility/failure/intermediate
   inventories; this cell does not close Rung 5.

### R5-34 — structural residual layout-frame projections

Status: **complete; scroll, text-input, and label facts given their valid
lifetimes**. Correction `133f7bf2` (`Make layout frame projections
structural`).

1. **Question and complete trace.** The continuing view/layout role sweep
   traced ordinary and table scroll construction through viewport resolution,
   table projection, track paint, scrollbar chrome, target lookup, reveal, and
   tests; active and inactive TextBox layout through draft projection, input
   decomposition, indicator paint/hit, shaping, caret/selection, and table-cell
   display; and resolved label overflow through source projection, visible
   measurement, text hit mapping, hover eligibility, and auxiliary-panel
   explanation. Every producer and consumer of `table_projection`,
   `input_parts`, and `overflow_projection` was included.
2. **Invalid states and retained common state.** `Frame` stored a table
   projection independently from common frame content even though only a table
   scroll can own one and it is resolved with that scroll's viewport. Input
   parts occupied every frame even though they are derived only beside a
   TextBox model; text content then represented an inactive TextBox as an Area
   with an optional input, allowing its role truth to collapse back to a text
   area. A selectable overflow projection was duplicated between Area content
   and the common frame, while optional label text, total label width, and that
   projection described one resolved label lifetime. Labels themselves remain
   common because label, button, bound-control, TextArea, and inactive TextBox
   presentation truthfully consume them.
3. **Correction and displaced paths.** `ScrollContent` now distinguishes
   ordinary and table species; one resolved private `TableScroll` owns the
   table viewport and projection together, and one `with_table_scroll` step
   replaces the sequential viewport/projection protocol. `TextContent` now
   distinguishes Area, InactiveField, and Field; both TextBox species own a
   private `TextBoxContent` containing the model and its input-part geometry.
   One common private `LabelContent` owns visible text, measured width, and the
   optional overflow projection. The three common frame fields, the duplicated
   Area projection, the optional inactive input, the independent label-width
   field, and `with_table_projection` are deleted.
4. **Boundary and naming ruling.** `Frame`, `Viewport`, and all existing
   table/text/indicator/overflow queries retain their exact crate spellings.
   `TableScroll`, `TextBoxContent`, and `LabelContent` are private owner-local
   concepts with no parent projection, alias, or flattened supporting export;
   the cell introduces no compound declaration re-exported under a simpler
   name and does not reopen unrelated naming cleanup.
5. **Behavior and economics.** Ordinary and table scroll offsets, viewport
   clips, table widths/tracks, reveal, scrollbar hits, active and inactive
   TextBox shaping, input-indicator geometry and targets, invalid-input hover,
   visible overflow text, source-range mapping, hover-panel content, label
   measurement, allocation, scene order, renderer topology, batching/pass
   fusion, invalidation, and presentation clocks are unchanged. One table
   projection clone was already required for layout's nested table context;
   the frame now stores it beside the same viewport, while the duplicate
   selectable projection clone is removed.
6. **Doctrine and witnesses.** Master design now states the ordinary/table
   scroll species, active/inactive text-field species, TextBox-owned input
   geometry, and the resolved label lifetime. The existing FrameContent
   architecture witness now tombstones every displaced common field and the
   former table builder while requiring the three structural payloads.
7. **Proof and gauge delta from R5-33.** The 154-test layout-scene slice passed,
   including ordinary/table scroll, inactive table editors, input indicators,
   overflow mapping, and hover explanation. The full library discovered 1,117
   tests and passed 1,107 with 10 ignored; all targets and all five examples
   compiled without warnings. All nine census parser witnesses, the full
   census, formatting, diff, and protected-state checks passed. Every gauge is
   unchanged: production/test edges 325/111, split responsibilities 3, slot
   edges 54, forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,809
   in 190 files, cross-slot upper bound 1,762, cross-slot test edges 90,
   source-root mentions 118, filesystem reads 361, allowances 6, panics 6, and
   expects 89.
8. **Fixed point and next frontier.** A table projection cannot inhabit an
   ordinary frame or separate from its viewport; input geometry cannot exist
   without its TextBox; inactive and active fields have explicit species; and
   visible label text, width, and overflow mapping have one owner. The reverse
   sweep continues through widget/theme state, remaining layout/session
   lifecycles, and the complete visibility/failure/intermediate inventories;
   this cell does not close Rung 5.

### R5-35 — valid widget label text species

Status: **complete; mutually exclusive builder policy made structural**.
Correction `0714a744` (`Make widget label text species structural`).

1. **Question and complete trace.** The widget-state sweep traced all public
   `Label::new`, `Label::world`, and `Label::wrapped` entrances through widget
   conversion, view-node construction, author-overflow diagnostics,
   world-text resolution, wrapping, measurement, scene paint, selectable
   source mapping, and overflow hover explanation. The constructors are the
   complete mutation surface; no setter or deserialization path can add a
   fourth policy.
2. **Invalid state.** `widget::Label` stored overflow and wrap as independent
   options. Its constructors admitted exactly author text, single-line world
   text with an explicit overflow policy, or wrapped world text with Clip, but
   the representation also admitted simultaneous ellipsis and wrapping. Node
   conversion retained an `unreachable!` solely to reject that impossible
   builder state.
3. **Correction and displaced path.** One private
   `Content::{Author, World { text, wrap, overflow }}` fact now owns label text
   provenance and policy. Each public constructor selects its exact species,
   and conversion exhaustively projects the same author, world, or wrapped
   node. The two independent options, tuple agreement match, and unreachable
   branch are deleted without adding a wrapper at any crossing.
4. **Boundary and naming ruling.** Public `widget::Label` and its three
   constructors retain their exact established spellings. Private `Content`
   stays inside private `widget::control::label` housing and receives no parent
   projection, alias, or flattened supporting export. The namesake central
   `Label` remains the only public parent projection; no compound declaration
   is re-exported under a simpler name.
5. **Behavior and economics.** Author diagnostics, explicit Clip/end/middle
   overflow, word wrapping, source text, measured dimensions, scene glyphs,
   hover eligibility/content, allocation, shaping/cache identity, renderer
   topology, batching/pass fusion, invalidation, and presentation clocks are
   unchanged. One enum discriminant replaces two option discriminants and no
   heap object, callback, lookup, traversal, or frame work is added.
6. **Doctrine and witnesses.** Master design now names author and world label
   species and requires world wrap/overflow policy to be one fact. A focused
   architecture witness pins the private sum and tombstones both independent
   options and the unreachable branch.
7. **Proof and gauge delta from R5-34.** Both world-text layout witnesses and
   the new structural witness passed. The full library discovered 1,118 tests
   and passed 1,108 with 10 ignored; all targets and all five examples compiled
   without warnings. All nine census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed. Every gauge remains
   unchanged: production/test edges 325/111, split responsibilities 3, slot
   edges 54, forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,809
   in 190 files, cross-slot upper bound 1,762, cross-slot test edges 90,
   source-root mentions 118, filesystem reads 361, allowances 6, panics 6, and
   expects 89.
8. **Fixed point and next frontier.** A widget label now carries only one valid
   text-provenance/policy species and no failure branch remains in conversion.
   The reverse sweep continues through the larger view/session interaction
   projections, theme patch semantics, remaining layout lifecycles, and the
   complete visibility/failure/intermediate inventories; this cell does not
   close Rung 5.

### R5-36 — structural focus-presentation species

Status: **complete; visible focus and focus ownership made one fact**.
Correction `4fc97968` (`Make focus presentation structural`).

1. **Question and complete trace.** The reverse focus-presentation sweep traced
   session focus species and input modality through retained-node projection,
   editable and inactive-display text controls, caret/preedit cleanup, public
   node inspection, layout-frame construction, focus-outline scene production,
   runtime visual state, cursor behavior, and the focused menu, table, text-box,
   and generic-control witnesses.
2. **Invalid state and duplicate authority.** `TextArea`, `TextBox`, `Node`, and
   `Frame` each stored independent `focused` and `focus_visible` booleans even
   though visible focus always implies focus. Text nodes additionally copied a
   generic node-envelope focus fact beside their control-owned projection even
   though editability and inactive-display state make the text control the only
   owner capable of deciding its visible affordance.
3. **Correction and displaced paths.** One closed
   `view::focus::Presentation::{Unfocused, Focused, Visible}` species now flows
   from view projection into layout. TextArea and TextBox own the projection for
   text nodes; the node envelope owns it for every other node. Public focused and
   visible queries derive from that one value, and `Frame` carries the same
   resolved receipt. All eight parallel boolean fields and the duplicated text-
   node envelope truth are deleted.
4. **Boundary and naming ruling.** The supporting type is declared simply as
   `Presentation` inside crate-visible `view::focus` and is consumed as
   `view::focus::Presentation` by layout. The view parent does not flatten or
   alias it. Only the type, its two downstream queries, the supporting module,
   and the node-to-layout receipt cross the owner boundary; construction and
   control projection remain restricted to view.
5. **Behavior and economics.** Pointer-hidden versus keyboard-visible focus,
   editable text focus, inactive table-cell display, caret installation and
   retirement, preedit cleanup, menu and palette focus, focus-outline paint,
   cursor selection, allocation, layout work, scene order, renderer topology,
   batching/pass fusion, invalidation, and presentation clocks are unchanged.
   One enum discriminant replaces each formerly correlated boolean pair without
   a heap object, lookup, callback, or additional traversal.
6. **Doctrine and witnesses.** Master design now names the three presentation
   species, the visible-implies-focused law, and the distinct text-control and
   generic-node owners. The architecture witness follows the namespaced species
   through both controls, node access/traversal, and frame storage; forbids a
   flattened parent projection; and tombstones every parallel boolean field.
7. **Proof and gauge delta from R5-35.** The full library discovered 1,119 tests
   and passed 1,109 with 10 ignored; all targets and all five examples compiled
   without warnings. All nine census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed. Production/test edges
   remain 325/111; split responsibilities, slot edges, forbidden edges, external
   violations, SCCs, cross-slot test edges, source-root mentions, filesystem
   reads, allowances, panics, and expects remain 3, 54, 0, 0, 0, 90, 118, 361,
   6, 6, and 89. The explicit namespaced crossing raises production
   `pub(crate)` declarations 1,809 in 190 files -> 1,814 in 191 files and the
   cross-slot upper bound 1,762 -> 1,767.
8. **Fixed point and next frontier.** Visible focus can no longer exist without
   focus, and text nodes no longer carry competing generic and control-owned
   presentation truth. The reverse sweep continues through text-control active
   state, session scopes, theme patch semantics, remaining layout lifecycles,
   and the complete visibility/failure/intermediate inventories; this cell does
   not close Rung 5.

### R5-37 — valid draft-change implication species

Status: **complete; detailed changes made to imply the broader receipt**.
Correction `2d2ce117` (`Make draft change implications structural`).

1. **Question and complete trace.** The reverse draft-receipt sweep traced the
   sole change producer through ordinary edit, selection, undo/redo, target
   replacement, preedit clearing, cursor and caret-blink updates, submit,
   runtime input outcomes, focused document services, feedback retirement,
   command-palette rebuilding, layout invalidation, and the draft/TextBox/text-
   input witnesses.
2. **Invalid states and retained independence.** `draft::Change` stored
   `text_changed`, `selection_changed`, and the broader `changed` fact as three
   independent booleans even though either detail always implies the broader
   change. A changed receipt without either detail remains valid for cursor,
   target, preedit, operation, or blink work. Submit is also independent: a
   submit request can exist without mutating draft state and is consumed
   separately by commit handling.
3. **Correction and repeated-policy reduction.** One private
   `Kind::{Unchanged, Changed { text, selection }}` now owns the implication.
   The sole constructor accepts the two details plus only the remaining change
   causes and derives the species once. The producer no longer repeats text and
   selection inside its broader-change expression; all existing receipt queries
   project from the sum, while submit remains beside it.
4. **Boundary and naming ruling.** Crate-visible `draft::Change` remains the
   namesake module's sole parent projection. Private `Kind` stays inside
   `draft::change` and receives no re-export or alias. No compound declaration,
   flattened supporting name, visibility widening, or public application
   spelling was introduced.
5. **Behavior and economics.** Text, selection, cursor, target, preedit,
   history, blink, submit, command-palette reset, feedback clearing, response
   effects, document outcomes, allocation, layout work, scene order, renderer
   topology, batching/pass fusion, invalidation, and presentation clocks are
   unchanged. One enum discriminant replaces the correlated broad boolean
   without a heap object, lookup, callback, or traversal.
6. **Doctrine and witnesses.** Master design now records the unchanged/changed
   species, the detailed-change implication, and submit independence. A direct
   owner witness proves both implications and submit-only absence of mutation;
   the architecture witness pins the private sum, the one central parent
   projection, derived admission, and extinction of the repeated producer
   terms.
7. **Proof and gauge delta from R5-36.** The full library discovered 1,121 tests
   and passed 1,111 with 10 ignored; all targets and all five examples compiled
   without warnings. All nine census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed. Every gauge remains
   unchanged: production/test edges 325/111, split responsibilities 3, slot
   edges 54, forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,814
   in 191 files, cross-slot upper bound 1,767, cross-slot test edges 90,
   source-root mentions 118, filesystem reads 361, allowances 6, panics 6, and
   expects 89.
8. **Fixed point and next frontier.** Detailed draft changes can no longer exist
   outside the broader changed species, and the producer no longer maintains
   the same implication manually. The reverse sweep continues through session
   scopes, scene/view action receipts, theme patch semantics, remaining layout
   lifecycles, and the complete visibility/failure/intermediate inventories;
   this cell does not close Rung 5.

### R5-38 — valid resolved-press admission species

Status: **complete; target and intent nested under their admission**.
Correction `a513cb33` (`Make resolved press admission structural`).

1. **Question and complete trace.** The reverse pointer/cursor sweep traced
   last-presented hit acquisition through ordinary, selectable-row, text,
   slider, scrollbar, divider, indicator, menu, palette, parent, popup,
   modifier-change, capture, down, drag, up, leave, task-focus departure, and
   stationary reprojection paths. It included every `ResolvedPress` consumer
   and the pointer/cursor/platform witnesses.
2. **Invalid states and retained axes.** `ResolvedPress` stored optional target,
   `PressAdmission`, and optional press intent independently. The representation
   admitted an inert or selection-only target with an intent, an admitted press
   without one, and an intent whose target could drift from admission. A runtime
   `expect` asserted the intended agreement. Hit presence, selectable-row
   gesture, optional task focus, cursor, and menu/palette surface membership
   remain separate because their combinations have witnessed meanings.
3. **Correction and displaced paths.** `PressAdmission` is now
   `Inert | SelectionOnly(Target) | Target { target, intent }`. Target projection
   derives from that species; admitted cursor selection matches it; pointer-down
   exhaustively obtains the same target and intent before constructing one
   action. The two parallel options, agreement comparison, target/intent
   re-pairing helper, and its assertion are deleted.
4. **Adjacent failure-path reduction.** The same admitted pointer-down path
   checked an optional focus transition for rejection and then expected the
   option to be present. An ownership-preserving match now consumes the rejected
   species directly and retains accepted absence/presence for later composition;
   that second runtime assertion is also deleted without changing transition
   policy.
5. **Boundary and naming ruling.** `ResolvedPress` and `PressAdmission` remain
   private runtime concepts; `interaction::pointer::PressIntent` remains a
   namespaced supporting type and is not flattened. Public input still exposes
   gesture meaning rather than internal admission. No compound declaration,
   alias, visibility widening, or public application spelling was introduced.
6. **Behavior and economics.** Cursor meaning, selection-only defaults, row
   focalization, task-focus rejection, click classification, capture, text and
   slider manipulation, menu/palette dismissal, pointer action order, layout,
   scene order, renderer topology, batching/pass fusion, invalidation, and
   presentation clocks are unchanged. One enum payload replaces two options
   and stores the admitted target once without a heap object, callback, lookup,
   or traversal.
7. **Doctrine and witnesses.** Master design now states exactly which target
   and intent facts each admission species owns. The existing resolved-press
   architecture witness now pins the sum and cursor match, tombstones both
   parallel fields and both assertions, and continues to require the one shared
   resolver for move, down, up, drag, leave, modifiers, and presentation.
8. **Proof and gauge delta from R5-37.** Forty-eight pointer and eighteen cursor
   witnesses passed. The full library discovered 1,121 tests and passed 1,111
   with 10 ignored; all targets and all five examples compiled without warnings.
   All nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Production/test edges remain 325/111; split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, allowances,
   and panics remain 3, 54, 0/0/0, 1,814 in 191 files, 90, 118, 361, 6, and 6.
   Production expects fall 89 -> 87.
9. **Fixed point and next frontier.** Admission can no longer disagree with its
   target or intent, and pointer departure consumes rejection without asserted
   optional presence. The scene target-visual flags resist merging as a full
   valid product across capture, manipulation, menus, hover, and selection. The
   reverse sweep continues through pointer-release actions, contextual
   departure, session scopes, theme patch semantics, remaining layout
   lifecycles, and the complete visibility/failure/intermediate inventories;
   this cell does not close Rung 5.

### R5-39 — valid resolved pointer-release action species

Status: **complete; activation nested beneath its release target**. Correction
`aaa423e1` (`Make pointer release action structural`).

1. **Question and complete trace.** The reverse pointer-action sweep traced
   resolved release from parent and native-popup hit testing through frame,
   scrollbar, divider, indicator, selectable-row, text, slider, menu, command,
   capture, release-away, pointer-leave, and direct view-action test paths. It
   followed target admission, optional activation lookup, pressed-target
   comparison, pointer retirement, gesture completion, command invocation, and
   every producer and consumer of `view::Action::PointerUp`.
2. **Invalid state and retained distinctions.** `PointerUp` stored an optional
   target beside an optional activation action. Layout admits a frame hit only
   when that frame has a target, and an activation action is obtained only from
   that same hit, so an action without a target had no producer or valid routing
   meaning. A targeted release without activation remains valid for text,
   slider, chrome, selection-only, and disabled/inert action cases; a release
   outside any target remains a distinct valid species that still retires press
   and gesture state.
3. **Correction and displaced protocol.** `PointerUp` now owns a total target
   and its optional activation action, while `PointerUpOutside` owns neither.
   Runtime resolves the species before constructing the view action; targeted
   routing compares the total target with the pressed target, while outside
   routing releases with ordinary target absence. The parallel optional target,
   `Some` wrapping at every targeted producer, and target/action invalid state
   are deleted.
4. **Boundary and naming ruling.** Both release species remain variants of the
   private crate-visible `view::Action`; no supporting type, parent projection,
   public re-export, compound declaration, or alias was introduced. The touched
   paths therefore preserve the canonical module/type projection law without
   widening the cell into unrelated naming cleanup.
5. **Behavior and economics.** Target equality, activation gating, captured and
   uncaptured release, release-away cancellation, menu opening, typed command
   invocation, pointer retirement, gesture history, layout, scene order,
   renderer topology, batching/pass fusion, invalidation, and presentation
   clocks are unchanged. One invalid option combination is removed; no heap
   object, callback, lookup, traversal, or frame work is added.
6. **Doctrine and witness.** Master design now names targeted and outside
   release species and the action-implies-target law. A focused architecture
   witness follows both constructors into runtime routing, requires a total
   target on the action-carrying species, and tombstones the optional target.
7. **Proof and gauge delta from R5-38.** Forty-nine pointer-focused witnesses
   passed, including captured release, release-away, menu activation, and typed
   command activation. The full library discovered 1,122 tests and passed 1,112
   with 10 ignored; all targets and all five examples compiled without warnings.
   All nine census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed. Production/test edges remain 325/111; split
   responsibilities, slot edges, forbidden/external/SCC counts, cross-slot test
   edges, source-root mentions, filesystem reads, allowances, panics, and
   expects remain 3, 54, 0/0/0, 90, 118, 361, 6, 6, and 87. The explicit
   outside-release constructor raises production `pub(crate)` declarations
   1,814 -> 1,815 and the cross-slot upper bound 1,767 -> 1,768.
8. **Fixed point and next frontier.** A resolved activation action can no longer
   outlive or disagree with its target, while outside release remains explicit
   and retires the same press lifecycle. `PointerDrag` resists the same shape:
   its target is already total, and hovered target plus optional drag action are
   independently meaningful under capture. The reverse sweep continues through
   contextual departure, session scopes, theme patch semantics, remaining
   layout lifecycles, and the complete visibility/failure/intermediate
   inventories; this cell does not close Rung 5.

### R5-40 — exhaustive contextual text-task departure

Status: **complete; asserted optional-presence protocol removed**. Correction
`3dd48921` (`Consume contextual departure exhaustively`).

1. **Question and complete trace.** The reverse contextual-departure sweep
   traced secondary release and direct context-menu opening through presented
   hit acquisition, semantic context paths, selected and unselected virtual
   rows, active text tasks, accepted and rejected draft commits, row selection,
   section resolution, menu opening, empty-section fallback, outcome merging,
   focus retention, pointer admission, and the corrected follow-up gesture.
2. **Outcome model and redundant assertion.** Departure already has three
   legitimate outcomes: absence when no active text draft exists, an accepted
   `TaskTransition` whose outcome composes with later work, or a rejected
   transition that must return before row selection or menu opening. The code
   tested optional presence plus rejection, then called `expect` to recover the
   value whose presence that test had just established. The assertion preserved
   no invariant beyond the option match.
3. **Reduction and rewire.** One ownership-consuming match now returns the
   rejected transition directly and retains accepted presence or absence for
   the existing later composition. The `as_ref` predicate, second option
   recovery, and production assertion are deleted. `TaskTransition` remains the
   honest accepted/rejected receipt; its outcome payload and the outer optional
   absence have distinct meanings and are not merged.
4. **Boundary and naming ruling.** The correction changes no declaration,
   visibility, module projection, or call-site import. Private runtime
   `TaskTransition` is not re-exported under another name, so no compound/simple
   collapse or namesake parent rule is implicated and unrelated naming cleanup
   remains out of scope.
5. **Behavior and economics.** Rejected drafts still retain focus, active text
   task, focal row, selection, and closed-menu state; accepted departure still
   composes before selection and opening; absence still proceeds directly.
   Context traversal, command resolution, allocation, layout, scene order,
   renderer topology, batching/pass fusion, invalidation, and presentation
   clocks are unchanged.
6. **Doctrine and witness.** Master design now states the absence/acceptance/
   rejection ordering law for contextual row departure. A focused architecture
   witness requires exhaustive rejection consumption and tombstones the
   presence assertion; the existing end-to-end rejection journey proves that no
   later selection, menu, or pointer work crosses the rejected boundary.
7. **Proof and gauge delta from R5-39.** Eleven focused architecture,
   context-menu, host-routing, and rejection witnesses passed. The full library
   discovered 1,123 tests and passed 1,113 with 10 ignored; all targets and all
   five examples compiled without warnings. All nine census parser witnesses,
   the full census, formatting, diff, and protected-state checks passed. Every
   graph, visibility, test-edge, source-root, filesystem, allowance, and panic
   gauge remains unchanged. Production expects fall 87 -> 86.
8. **Fixed point and next frontier.** Contextual departure now consumes each
   outcome once and has no asserted optional-presence path. Other
   `TaskTransition` consumers already use ownership-preserving `if let`, match,
   or option mapping and retain their distinct accepted/rejected/absent laws.
   The reverse sweep continues through session scopes, theme patch semantics,
   remaining layout lifecycles, and the complete visibility/failure/
   intermediate inventories; this cell does not close Rung 5.

### R5-41 — session command-scope and window-state resistance

Status: **complete resistance ruling; no production correction admitted**.

1. **Question and complete trace.** The reverse session sweep traced focused,
   transient, captured, and contextual `CommandScope` construction through
   responder-chain routing, text and table service claims/invocation, command
   palette query and captured worlds, context-menu path frames, standard-menu
   live state, typed and erased command transactions, and programmatic dispatch.
   It also traced window focus, menu restoration, palette capture, active text
   target, invalidation, projected revision, desired/acknowledged presentation
   epochs, dialogs, feedback, selection snapshots, restore, pruning, and
   destruction.
2. **Scope alignment ruling.** `CommandScope` is not a transport-only wrapper.
   Its lower `responder::Scope` is consumed by path traversal, while its focus,
   table, and route kind are consumed repeatedly by higher text/table service
   realization. Focused, transient, captured, and contextual constructors mint
   those facts together; table identity is derived once from table-cell focus
   when an explicit contextual table is absent. Removing the type would force
   every consumer to reconstruct alignment, while moving focus/table into
   responder would restore the Rung 3 upward dependency.
3. **Window-state product ruling.** Live focus, optional menu restoration,
   optional palette-captured focus, and active text target have separately
   witnessed combinations and precedence. A menu or palette may capture no
   prior focus; live focus changes while a command surface remains open; text
   drafts may outlive activation; outside dismissal intentionally does not
   restore. Likewise invalidation, projected revision, desired epoch,
   acknowledged epoch, file-dialog request, feedback stack, and interaction
   state each have independent creation, drain, retry, snapshot, or cleanup
   clocks. No correlated option cluster or boolean protocol is present.
4. **Boundary and naming ruling.** Session remains the honest owner of the
   higher alignment receipt and focus-restoration lifecycles; responder remains
   route-only and interaction remains command-surface identity. The private
   `CommandScope` name is not aliased or re-exported, and no parent projection
   or supporting public type is added. Existing namesake and call-site
   qualification laws therefore remain satisfied without naming churn.
5. **Rejected alternatives.** A generic service context would expose a state
   bag; callbacks would conceal the same route-to-service dependency; an enum
   duplicating all four scope constructors would repeat the same payloads
   without deleting a valid state; nesting session `Focus` inside interaction
   menus would reverse owner direction; and merging the window clocks would
   contradict skipped-presentation, retry, and restoration witnesses. No
   correction passes admission.
6. **Behavior and economics.** Command precedence, exact and broad routing,
   active-text priority, table service selection, palette invocation against
   captured rather than query focus, menu restoration, context pruning,
   snapshots, allocation, layout, scene order, renderer topology, invalidation,
   and presentation clocks remain byte-for-byte unchanged.
7. **Proof and gauge.** The responder-scope architecture witness, both captured-
   focus palette journeys, and pointer-opened menu focus journey passed
   directly. The R5-40 full proof remains the unchanged production boundary:
   1,113 passed, 10 ignored, 0 failed; all targets/examples, census, format,
   diff, and protected-state checks passed. Every gauge remains unchanged,
   including zero forbidden/external/SCC findings and 86 production expects.
8. **Fixed point and next frontier.** Session scope alignment and window state
   remain intentional, evidenced non-merges; no consumer-local fallback,
   competing authority, or invalid combination was found. The reverse sweep
   continues with theme patch absence and material species, then remaining
   layout lifecycles and the complete visibility/failure/intermediate
   inventories; this cell does not close Rung 5.

### R5-42 — theme patch absence and truthful glass round trips

Status: **complete; patch optionality retained and one lossy export corrected**.
Correction `27dcc1d0` (`Preserve theme glass luminosity round trips`).

1. **Question and complete trace.** The reverse theme sweep traced public TOML
   parsing and serialization through variant selection, palette extension and
   reference resolution, every optional section and token, typography-derived
   defaults, solid and glass material admission, recipe/current-material
   overlay, public programmatic material mutation, glass-tuner projection,
   runtime token consumption, unknown-field/color/recipe failures, and full
   export/reparse. All patch and export transport types remain private to
   `theme::toml`; resolved `Theme` stores total runtime tokens.
2. **Absence and material-species resistance.** `Option` in `ThemePatch` and
   its section patches is authored omission: an absent variant selects dark,
   an absent section or field inherits the selected base, and an absent glass
   member inherits its recipe or current material. Glass members are
   intentionally independent partial overrides; tint/opacity, luminosity
   color/opacity, and the refraction components resolve their missing peer from
   current truth. Collapsing those options into runtime species would erase the
   patch language rather than remove invalid state. `MaterialToml::Solid` and
   `Glass` remain the honest exclusive material species.
3. **Discovered information loss.** The public material API can install a
   `Glass` whose `Luminosity` has an arbitrary color and opacity, but TOML
   exported only the opacity and always selected a recipe from the theme
   variant. A pinned public round trip failed before correction: a dark theme
   carrying panel-light glass with luminosity `#123456` reparsed with the dark
   recipe's `#1c1c1e` luminosity. The glass tuner's displayed patch had the same
   mismatch after tint tuning because its rendered luminosity used the tint
   color while its TOML omitted that fact.
4. **Correction and displaced path.** Glass TOML now accepts optional
   `luminosity-color` beside the established `luminosity-opacity`. Supplying
   either preserves the other from the current recipe; export emits both facts
   whenever luminosity exists. The glass tuner emits the same tint as its
   luminosity color, so the displayed patch now describes its rendered
   material. The lossy recipe recovery is deleted without changing existing
   opacity-only patches, variant defaults, or any other token.
5. **Boundary and naming ruling.** The new field is private patch vocabulary;
   no Rust declaration, visibility, parent projection, or call-site import was
   added. `Theme`, `ThemeTomlError`, `Material`, `Glass`, and `Luminosity` keep
   their established canonical names. No compound declaration is projected
   under a simple alias, no namesake module flattens a supporting type, and the
   cell does not widen into unrelated naming cleanup.
6. **Behavior and economics.** Existing theme files retain their inheritance
   semantics; opacity-only glass patches preserve recipe color; unknown fields,
   colors, recipes, and nonuniform rounding retain typed failure. Runtime
   layout, shaping, scene order, material layers, renderer topology,
   batching/pass fusion, invalidation, presentation clocks, allocation, and
   frame work are unchanged. Serialization adds one truthful inline field only
   when the resolved material has luminosity.
7. **Doctrine and proof.** Master design now distinguishes patch absence from
   resolved theme state and requires serialization of every publicly mutable
   material fact. The new witness proves color-only inheritance and a public
   custom-glass round trip; all fifteen theme TOML tests and the glass-tuner
   render/patch journey passed. The full library discovered 1,124 tests and
   passed 1,114 with 10 ignored; all five examples compiled; formatting and
   diff checks passed.
8. **Gauge delta and next frontier.** The full census remains 47 top-level
   modules, 325 production edges, 111 test-only edges, three split
   responsibilities, 54 slot edges, zero forbidden/external/SCC findings,
   1,815 production `pub(crate)` declarations in 191 files, a 1,768 cross-slot
   upper bound, 90 cross-slot test edges, 118 source-root mentions, 361
   filesystem reads, six allowances, six panics, and 86 expects. Theme patch
   absence and material species are now at fixed point. The reverse sweep
   continues through remaining layout lifecycles and the complete visibility,
   failure, intermediate, housing, and naming inventories; this cell does not
   close Rung 5.

### R5-43 — structurally nonempty text line index and total current marks

Status: **complete; false current-document absence removed at its lower
owner**. Correction `8ffdc983` (`Make current text marks total`).

1. **Question and complete trace.** The remaining layout-failure sweep traced
   every current position/cursor-to-mark projection through empty and multiline
   buffer construction, persistent line-index creation and replacement, full
   deletion, grapheme clamping, selection state, edits and history markers,
   TextBox field layout, table TextArea draft projection, scroll anchors, and
   stale mark restoration. It distinguished creation of a mark in the current
   document from later lookup of a stable line identity after edits.
2. **False absence and lower invariant.** `TextDocument::mark_for_position` and
   `mark_for_cursor` returned `Option` even though the source tree always has at
   least one logical line and every position/cursor is clamped into it. The
   persistent `LineIndex` nevertheless stored an optional root because private
   split/concat temporaries could be empty. That leaked splice machinery into
   the stored owner and forced layout, view, edit, selection, marker, and scroll
   paths to carry unreachable absence, fallback-to-end protocols, or runtime
   assertions. Reverse mark lookup remains legitimately optional because an
   older mark's stable line identity can depart after a later edit.
3. **Correction and displaced paths.** Stored `LineIndex` now owns a total
   `Arc<Node>` root; only private split, join, and replacement-middle values may
   be empty. Removing every line installs one fresh empty-document line before
   the index becomes observable. Current position/cursor and selection-range
   mark creation is total, while mark-to-position/cursor projection keeps its
   existing `Option`. Field layout, projected TextArea drafts, selection/edit
   mutation, history recovery, and scroll-anchor construction consume the
   total result directly. Six production assertions and all associated
   fallback/re-pairing paths are deleted.
4. **Boundary, API, and naming ruling.** The public
   `Buffer::mark_for_position` contract now truthfully returns `Mark` rather
   than advertising impossible absence; clamping and affinity are unchanged and
   no compatibility shim preserves the false shape. `text::Buffer` remains the
   namesake module's sole parent projection. Supporting `Mark`, `MarkRange`,
   `Position`, and related values remain simple declarations under
   `text::buffer` and are used through that namespace; no compound declaration,
   flattened support export, alias, or unrelated rename was introduced.
5. **Behavior and economics.** Empty and fully deleted documents still contain
   one logical line; oversized positions and cursors still clamp to the current
   text and grapheme boundary; affinity, stable line identity, stale-mark
   absence, selection, edit history, reveal, shaping, layout, scene order,
   renderer topology, batching/pass fusion, invalidation, and presentation
   clocks are unchanged. The release acceptance witness measured 8 MiB load at
   33.255 ms, long-line typing at 2.501–3.613 microseconds/edit, and 10-byte
   versus 10-MiB clone medians at 33.674/34.225 ns, all within the standing
   bounds.
6. **Doctrine and ratchet.** Master design now states that the stored line index
   is structurally nonempty, current-document mark creation is total, and only
   stale stable-identity lookup is fallible. A typed public witness fails to
   compile if `mark_for_position` regains `Option`; an owner test removes every
   indexed line and proves the replacement remains nonempty; and the existing
   buffer architecture witness pins the total root and signatures while
   tombstoning the optional stored root and old invariant assertion.
7. **Proof.** The 132-test text slice passed 130 with its two standing ignores;
   the text-buffer architecture witness and all 39 TextBox and 55 TextArea
   slices passed. Both ignored deep-tier witnesses were then run explicitly:
   the release economics bound and the 100,000-operation edit/undo reference
   property passed. The full library discovered 1,126 tests and passed 1,116
   with 10 ignored; all five examples compiled; formatting and diff checks
   passed; protected `comparison_open: true` remained unchanged.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, cross-slot test
   edges, source-root mentions, allowances, and panics remain 325/111, 3, 54,
   0/0/0, 90, 118, 6, and 6. Narrowing the end-mark helper lowers production
   `pub(crate)` declarations 1,815 -> 1,814 and the cross-slot upper bound 1,768
   -> 1,767. The strengthened existing architecture witness raises filesystem
   reads 361 -> 362. Removing the six assertions lowers production expects 86
   -> 80. Current-document mark projection is at fixed point; the reverse sweep
   continues through remaining layout lifecycles and the complete visibility,
   failure, intermediate, housing, and naming inventories. This cell does not
   close Rung 5.

### R5-44 — direct namespaced composition identity

Status: **complete; duplicate namespace wrapper and two asserted conversions
removed**. Correction `3c5f0dcc` (`Make composition identity directly
namespaced`).

1. **Question and complete trace.** The composition-identity sweep traced
   retained tree construction, view-only layout construction, reconciliation,
   sibling matching, parent links, addition/removal receipts, subtree pruning,
   retained traversal, layout frames, hit targets, and scene-region ownership.
   It included the standing collision witness that gives view-only layouts a
   separate namespace from installed retained composition.
2. **Duplicate representation and retained law.** `NodeId` already stored the
   authoritative `Retained | Layout` namespace, while private `Identity`
   wrapped the same value in a second `Retained | Layout` sum. The wrapper
   preserved no additional identity, authority, lifetime, or capability and
   could represent disagreement between its outer species and the inner
   `NodeId` space. The layout namespace itself is deliberate doctrine and is
   retained unchanged.
3. **Correction and displaced paths.** Composition nodes and parent links now
   store `NodeId` directly; sibling-use sets, construction, reconciliation,
   lookup, and ancestry consume that same value. Retained construction adds
   the freshly minted retained id directly to `Changes`, deleting both
   `retained_id().expect(...)` conversions. Layout construction still mints
   `NodeId::layout`, and retained projections still return `Option` where the
   shared test-only tree species makes namespace absence meaningful.
4. **Boundary, naming, and resistance ruling.** `NodeId` remains process-
   transient crate vocabulary and is neither public API nor application state.
   No module projection, supporting-type flattening, alias, compound
   declaration, or public call-site spelling changed. The production
   `require_retained_id` traversal guard remains: deleting it would require a
   separately proven retained-tree type boundary rather than assuming every
   composition node has the installed species.
5. **Gauge instrument correction.** Removing the wrapper exposed that the
   census treated `#[cfg(test)]` enum variants and match arms as whole Rust
   items, sometimes masking a later production item. The range scanner now
   ends attributed variants, arms, fields, and statements at their own syntax
   boundary while retaining item/block handling. A tenth parser witness pins
   both the test-only receipts and the following production visibility. The
   corrected governing R5-43 baseline is 109 test-only edges rather than 111,
   1,819 production `pub(crate)` declarations rather than 1,814, and a 1,772
   cross-slot upper bound rather than 1,767; no Rust visibility changed.
6. **Behavior and economics.** Stable retained ids, positional and explicit-id
   reconciliation, cross-parent removal/addition, subtree cleanup, parent
   ancestry, view-only layout ids, hit-target isolation, allocation, hashing,
   layout, scene order, renderer topology, batching/pass fusion, invalidation,
   and presentation clocks are unchanged. One enum discriminant and two
   asserted conversions disappear from retained construction.
7. **Doctrine and proof.** Master design now states that `NodeId` itself owns
   the namespace and nodes/parents store it directly. The architecture witness
   failed against the old `Identity` representation, then passed while
   tombstoning the wrapper. All fourteen composition tests passed, including
   retained hit identity and view-only namespace separation. The full library
   discovered 1,126 tests and passed 1,116 with 10 ignored; all five examples,
   all ten census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed.
8. **Gauge delta and next frontier.** Production edges, split
   responsibilities, slot edges, forbidden/external/SCC findings, cross-slot
   test edges, source-root mentions, filesystem reads, allowances, and panics
   remain 325, 3, 54, 0/0/0, 90, 118, 362, 6, and 6. On the corrected
   instrument, test-only edges and visibility remain 109 and 1,819 in 191
   files with a 1,772 upper bound. Removing the two assertions lowers
   production expects 80 -> 78. Direct namespaced composition identity is at
   fixed point; the reverse sweep continues through retained-tree admission,
   remaining layout lifecycles, and the complete visibility, failure,
   intermediate, housing, and naming inventories. This cell does not close
   Rung 5.

### R5-45 — total sorted table-header geometry

Status: **complete; false indicator absence removed at the geometry owner**.
Correction `676ee193` (`Make sorted header geometry total`).

1. **Question and complete trace.** The remaining layout-geometry assertion was
   traced from table header presentation and sort direction through label
   measurement, trailing-indicator reservation, scene icon paint, header hit
   routing, divider resize, ascending/descending application order, expanded
   overflow, and all four standing scale witnesses.
2. **False absence.** The shared header helper returned an optional indicator
   because unsorted headers legitimately have none. The explicitly sorted
   `table_sort_indicator_rect` call then invoked that helper with `true` and
   asserted the requested indicator existed. Absence belongs only to the
   unsorted/sorted decision; it is impossible after the sorted species has been
   selected.
3. **Correction and displaced path.** A private
   `sorted_table_header_parts -> (Rect, Rect)` now resolves label and indicator
   geometry together. The optional shared projection wraps that total pair only
   when a caller supplies the sort-presence boolean, while the explicit
   indicator request consumes the total result directly. The production
   assertion and its false optional recovery are deleted.
4. **Boundary and naming ruling.** Layout remains the sole geometry owner and
   scene remains a consumer. The helper is private, introduces no transport
   type, crosses no seam, and receives no projection or alias. Existing
   namespaced layout functions and every public table/header spelling remain
   exact; no compound declaration, supporting parent export, or unrelated
   naming cleanup was introduced.
5. **Behavior and economics.** Content padding, indicator extent, centered
   geometry, label gap, ellipsis width, sort icon choice, header target,
   divider hit zone, resize behavior, table order, allocation, scene order,
   renderer topology, batching/pass fusion, invalidation, and presentation
   clocks are unchanged. Sorted requests perform the same arithmetic once and
   no longer construct and unwrap an `Option`.
6. **Doctrine and ratchet.** Master design now states that sorted-header label
   and indicator rectangles are one total pair. A focused architecture witness
   failed against the asserted optional path, then pinned the total helper and
   tombstoned the assertion. Existing end-to-end witnesses continue to compare
   painted label and chevron rectangles against the layout owner.
7. **Proof.** The architecture witness, the control-gallery sort/resize
   journey, and the expanded single-line sort-header journey passed directly.
   The full library discovered 1,127 tests and passed 1,117 with 10 ignored;
   all five examples, all ten census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Every graph, visibility, test-edge,
   source-root, filesystem, allowance, and panic gauge remains unchanged:
   production/test edges 325/109, split responsibilities 3, slot edges 54,
   forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,819 in 191
   files, cross-slot upper bound 1,772, cross-slot test edges 90, source-root
   mentions 118, filesystem reads 362, allowances 6, and panics 6. Production
   expects fall 78 -> 77. Sorted-header geometry is at fixed point; the reverse
   sweep continues through group/ghost projection, retained tree/layout
   admission, frame construction, and the complete visibility, failure,
   intermediate, housing, and naming inventories. This cell does not close
   Rung 5.

### R5-46 — invariant-preserving ghost group projection

Status: **complete; asserted reconstruction replaced by structural
projection**. Correction `b51f4472` (`Preserve ghost group invariants
directly`).

1. **Question and complete trace.** The remaining scene-group assertion was
   traced from in-frame overlay retirement through scene cloning, recursive
   primitive projection, nested groups, pane material downgrade, material-
   region retirement, outer ghost opacity, semantic scene order, renderer
   lowering, and the existing ghost-overlay witnesses.
2. **Invariant and false fallibility.** An admitted `Group` is nonempty and
   owns a clamped positive opacity. Ghost projection maps every primitive
   one-for-one and preserves that opacity, so an existing group cannot become
   empty or invalid. Re-entering `Group::new` made the preserved invariant look
   fallible and required `expect("existing group is visible")` to recover it.
3. **Correction and displaced path.** Recursive backdrop-sampling removal now
   belongs to `Primitive` and `Group`. A group maps its children one-for-one
   and constructs the same private fields directly; scene invokes that owner
   projection, clears material regions, and applies the established outer
   opacity. The free recursive helper, fallible reconstruction, and assertion
   are deleted. Material-resolution filtering still uses `Group::new` because
   its `filter_map` can legitimately remove every child.
4. **Boundary and naming ruling.** Primitive shape and group validity remain
   owned by scene primitive housing; scene orchestration retains ghost-region
   and overlay-opacity policy. The new operation is visible only within
   `crate::scene`. No declaration is re-exported, no supporting type is
   flattened, and no compound name or alias is introduced; the canonical
   parent-projection and call-site qualification law remains untouched.
5. **Behavior and economics.** Pane body, tint, grain, geometry, nested group
   membership, child order, admitted opacity, outer ghost opacity, material-
   region absence, hit transparency, scene order, renderer topology,
   batching/pass fusion, invalidation, and presentation clocks are unchanged.
   Projection still performs one clone-or-transform per primitive and now
   avoids validation and optional recovery for already-valid groups.
6. **Doctrine and witnesses.** Master design now states that the one-for-one
   projection preserves group validity directly. The architecture witness
   failed against the asserted reconstruction, then pinned primitive-owned
   projection and tombstoned the assertion. A new nested-group witness proves
   inner membership and opacity, outer ghost opacity, retained pane content,
   and absent material regions.
7. **Proof.** Both ghost-overlay tests and the focused architecture witness
   passed directly. The full library discovered 1,128 tests and passed 1,118
   with 10 ignored; all five examples, all ten census parser witnesses, the
   full census, formatting, diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, allowances,
   and panics remain 325/109, 3, 54, 0/0/0, 1,819 in 191 files, 90, 118, 362,
   6, and 6. Production expects fall 77 -> 76. Ghost group projection is at
   fixed point; the reverse sweep continues through retained-tree admission,
   frame construction, remaining layout lifecycles, and the complete
   visibility, failure, intermediate, housing, and naming inventories. This
   cell does not close Rung 5.

### R5-47 — direct typed view-content projection into layout

Status: **complete; duplicated role/content agreement and fourteen assertions
removed**. Correction `7d20ba15` (`Project typed view content into layout`).

1. **Question and complete trace.** The frame-construction sweep traced every
   view content species through node builders, public model accessors, layout
   measurement, text overflow and inactive-field projection, control geometry,
   table and virtual-list specialization, floating-panel state, frame role
   queries, scene production, hit testing, and the complete layout/control
   witness slices.
2. **Duplicate agreement and failure shape.** R5-32 had made private
   `view::node::Content` the node's structural role/payload truth, yet layout
   queried the derived role, recovered each payload through independent
   optional accessors, rebuilt a second agreement in `FrameContent::for_node`,
   and used fourteen `expect`s to assert that the two representations still
   matched. Those assertions preserved no invariant beyond the source sum.
3. **Boundary and naming ruling.** The existing namesake seam is the honest
   crossing: `view::node` is crate-visible, `view::Node` remains the central
   parent projection, and supporting `Content`, `MenuBar`, `Panel`, and
   `Scroll` remain qualified beneath `view::node`. Layout imports
   `view::{node, Node}`, uses `Node` centrally, and matches `node::Type` for
   support. No support type is flattened at `view`, no compound declaration or
   aliased re-export is introduced, and application-facing paths are unchanged.
4. **Correction and displaced paths.** Frame construction now exhaustively
   matches `Node::content()` and enriches each semantic species directly with
   layout-owned geometry, shaping, overflow, and resolved panel facts.
   `FrameContent::for_node`, role-first payload recovery, all fourteen
   agreement assertions, and three displaced panel accessors are deleted.
   Shared label projection remains one helper and preserves the established
   shaping and diagnostic sequence.
5. **Visibility and resistance.** Eight net crate-visible declarations are the
   measured price of the real view-to-layout library contract: the namespaced
   module/content species, the narrow content projection, and the panel fields
   layout consumes, offset by deleted accessors. Making layout depend on a
   second transport enum, callback, or flattened facade would hide the same
   crossing and restore duplicated agreement, so the explicit typed seam is
   admitted for Rung 6's later symbol-level disposition.
6. **Behavior and economics.** Every role, text-area and active/inactive
   TextBox layout, overflow projection, label width/diagnostic, input geometry,
   slider/choice active rectangle, scroll species, virtual model, panel policy,
   scene order, hit route, renderer topology, batching/pass fusion,
   invalidation, and presentation clock is unchanged. Models are cloned and
   text engines invoked in the same order; no heap object, callback, lookup,
   traversal, or frame pass was added.
7. **Doctrine and proof.** Master design now names the direct typed crossing
   and the namespaced parent law. The ownership witness failed against the old
   private/role-recovery boundary, then pinned `view::{node, Node}`, direct
   content matching, and extinction of `for_node` and its assertions. The four
   structural witnesses, 154 layout-scene tests, 26 TextBox tests, 13 focus
   tests, and six slider tests passed. The full library discovered 1,128 tests
   and passed 1,118 with 10 ignored; all five examples, all ten census parser
   witnesses, the full census, formatting, diff, and protected-state checks
   passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, cross-slot
   test edges, source-root mentions, filesystem reads, allowances, and panics
   remain 325/109, 3, 54, 0/0/0, 90, 118, 362, 6, and 6. The admitted crossing
   raises production `pub(crate)` declarations 1,819 in 191 files -> 1,827 in
   192 files and the cross-slot upper bound 1,772 -> 1,780. Removing the false
   agreement assertions lowers production expects 76 -> 62. Direct
   view-content projection is at fixed point; the reverse sweep continues
   through two-phase frame specialization, retained-tree admission, remaining
   layout lifecycles, and the complete visibility, failure, intermediate,
   housing, and naming inventories. This cell does not close Rung 5.

### R5-48 — two-phase frame specialization resistance

Status: **complete resistance ruling; no production correction admitted**.

1. **Question and complete trace.** The post-R5-47 sweep traced ordinary and
   table scroll viewport resolution, fixed and variable virtual-list requests,
   parent-measured menu shortcut columns, floating-panel recursive placement,
   frame publication, every specialized frame query, and all five
   `with_*`/placement call sites. It followed the surrounding child layout,
   clipping, table projection lifetime, materialization fixed point, overlay
   attachment, scene consumption, and hit testing.
2. **Construction-phase invariant.** Generic `layout_node` excludes Scroll and
   VirtualList before publishing a frame. Their dedicated algorithms resolve
   geometry and immediately specialize the frame before its one push. Menu
   rows receive the parent-resolved maximum shortcut width before publication.
   Floating placement is the one recursive exception: the parent resolves the
   request, lays out the child panel, then updates that exact child frame by
   retained node identity. No consumer observes a half-specialized frame.
3. **Challenge and rejected reductions.** Replacing the panics/assertion with
   no-ops, defaults, or `Option` returns would hide caller drift and manufacture
   false fallibility. Adding a generic resolution enum would merely represent
   every invalid node/resolution pairing in another type. A structurally total
   solution requires a draft-frame typestate sum plus a second common envelope,
   or broadening every recursive layout entrance with role-specific state;
   neither deletes an owner, translation, traversal, or observed defect.
4. **Ruling.** Retain the four local specialization invariants and the popup
   debug assertion. They guard one cohesive layout owner's private construction
   protocol, not a cross-seam agreement or operational failure. Reopen only if
   another consumer observes incomplete frames, a second specialization path
   appears, or a broader layout-state refactor supplies the draft type without
   duplicating the common frame envelope.
5. **Boundary and naming ruling.** The protocol remains entirely under layout;
   no public or crate-crossing type, visibility, projection, alias, compound
   declaration, or call-site import changes. The newly admitted
   `view::{node, Node}` seam is not widened with layout staging vocabulary.
6. **Behavior and economics.** Viewport geometry, table projection, virtual
   requests, shortcut alignment, floating placement, frame order, clipping,
   scene order, renderer topology, batching/pass fusion, invalidation,
   allocation, recursive work, and presentation clocks remain byte-for-byte
   unchanged.
7. **Proof.** The one-way call-site census found one ordinary-scroll, one
   table-scroll, two virtual-list, one menu-row, and one popup-placement path,
   each paired with its immediate publication or exact-node update. R5-47's
   unchanged full boundary remains the proof: 1,118 passed, 10 ignored, all
   five examples and all ten census parser witnesses green, with format, diff,
   census, and protected-state checks passing.
8. **Gauge and next frontier.** No code or map changed: production/test edges
   remain 325/109, split responsibilities 3, slot edges 54, forbidden/
   external/SCC counts 0/0/0, production `pub(crate)` 1,827 in 192 files,
   cross-slot upper bound 1,780, cross-slot test edges 90, source-root mentions
   118, filesystem reads 362, allowances 6, panics 6, and expects 62. The
   reverse sweep continues with retained-tree identity/child admission, then
   remaining layout lifecycles and the full visibility, failure, intermediate,
   housing, and naming inventories. This cell does not close Rung 5.

### R5-49 — retained tree species and namespaced support surface

Status: **complete; false retained-identity absence removed and the touched
namesake seam normalized**. Correction `93a16d52` (`Make retained tree identity
structural`).

1. **Question and complete trace.** The retained-tree admission sweep traced
   retained construction, reconciliation, subtree removal, ancestry, view-only
   layout construction, layout frame identity, retained view traversal, hit
   targets, collision isolation, and both child-order admission helpers. It
   followed every `NodeId`, `Node`, `Changes`, and `Tree` crossing and every
   call site affected by the composition parent projection.
2. **False absence and structural boundary.** `Space::Layout`, layout-id minting,
   and `Tree::layout` existed only under `cfg(test)`, yet attaching that
   constructor to the same `Tree` type made production retained operations
   recover identity through `Option` and one `expect`. Subtree removal and
   ancestry also filtered identity despite being reachable only through
   retained construction or reconciliation. The alternate namespace is real;
   its apparent presence inside retained production is not.
3. **Correction and displaced paths.** `composition::Tree` now has only retained
   construction and reconciliation entrances. Test-only view layout uses the
   distinct `composition::tree::Layout` species while sharing the private node
   grammar and preserving layout-namespace ids. Retained subtree removal,
   ancestry, and view traversal consume `NodeId` directly. The retained-id
   conversion methods, filtering protocol, `Tree::layout`, and traversal
   assertion are deleted.
4. **Namesake projection ruling.** Because the cell touched the `tree` module's
   boundary, it applied the canonical law rather than preserving a flattened
   support surface. `composition` projects only the central `Tree`; supporting
   `NodeId`, `Node`, `Changes`, and test-only `Layout` remain beneath
   crate-visible `composition::tree`, and all affected call sites use that
   namespace. No compound declaration, aliased re-export, flattened supporting
   export, or compatibility path survives.
5. **Child-admission resistance.** Declarative view children and composition
   children remain parallel by construction and reconciliation. The two local
   child-order assertions are retained: truncating `zip`, optional propagation,
   or defaults would conceal drift. Making the pairing structural today would
   require a second common node envelope, a recursive wrapper, or generic tree
   species spread through view and layout, adding machinery without deleting an
   owner or evidenced defect. Reopen when a paired traversal can replace that
   envelope rather than duplicate it.
6. **Behavior and economics.** Retained id stability, sibling reconciliation,
   removal receipts, ancestry, view-only collision isolation, hit identity,
   layout geometry, focus and interaction projection, scene order, renderer
   topology, batching/pass fusion, invalidation, allocation, and presentation
   clocks are unchanged. The test-only layout tree retains the same node values;
   retained paths remove one option recovery and identity filtering.
7. **Doctrine and proof.** Master design now names the distinct tree species,
   total retained entrance, namespaced support surface, and child-admission
   resistance. The architecture witness failed against the shared constructor
   and flattened projection before correction, then pinned the new boundary and
   tombstones. All fourteen composition tests and the 221-test layout slice
   passed. The full library discovered 1,128 tests and passed 1,118 with ten
   ignored; all targets and all five examples compiled without warnings. All ten
   census parser witnesses, the full census, formatting, diff, and protected-
   state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, cross-slot test
   edges, source-root mentions, allowances, and panics remain 325/109, 3, 54,
   0/0/0, 90, 118, 6, and 6. Removing the flattened/false crossings lowers
   production `pub(crate)` declarations 1,827 -> 1,825 and the cross-slot upper
   bound 1,780 -> 1,778; the strengthened architecture witness raises filesystem
   reads 362 -> 363. Production expects fall 62 -> 61. Retained-tree identity is
   at fixed point; the reverse sweep continues through remaining layout
   lifecycles and the complete visibility, failure, intermediate, housing, and
   naming inventories. This cell does not close Rung 5.

### R5-50 — direct typed view-content dispatch through recursive layout

Status: **complete; duplicate derived-role dispatch and two asserted payload
recoveries removed**. Correction `7535cda7` (`Dispatch layout from typed view
content`).

1. **Question and complete trace.** The post-frame-construction sweep traced
   every view content species through recursive layout selection, ordinary and
   table scrolling, fixed and variable virtual lists, root floating-panel
   placement, clipping, frame publication, table/virtual specialization, and
   all downstream scene and hit consumers. It followed every remaining role
   query and content-payload recovery in `layout::algorithm` after R5-47 had
   established typed content as the frame-construction boundary.
2. **Duplicate agreement and false failure.** Recursive layout still selected
   algorithms through derived `Role`, then recovered table-scroll and
   virtual-list models through optional accessors and two `expect`s. The source
   `view::node::Content` sum already makes both models structural and had just
   been matched directly by frame construction. The role/payload protocol was
   therefore a second, weaker representation of the same agreement.
3. **Correction and displaced paths.** `layout_node` now captures and
   exhaustively matches `Node::content()` for recursive dispatch as well as
   ordinary frame recursion. `node::Scroll::Table` passes its owned table model
   directly to table layout; virtual-list content passes its model directly to
   virtual layout. Floating-panel detection also matches content. Every role
   query, both optional model recoveries, and both assertions are deleted from
   the algorithm.
4. **Boundary and naming ruling.** The touched seam practices the canonical
   namesake law: `view::Node` remains the sole central parent projection,
   layout imports `view::{self, node}`, and supporting `Content` and `Scroll`
   are named as `node::Content` and `node::Scroll`. No supporting type is
   flattened, no compound declaration is re-exported under a simple alias at
   any parent, and no compatibility spelling survives.
5. **Topology resistance.** Table surface/header children, materialized
   virtual rows, and declarative/composition child order remain independently
   stored topology. Their local assertions are retained: defaults, truncating
   zips, or optional recovery would conceal drift, while making them structural
   requires the broader paired-tree envelope already resisted in R5-49. This
   cell removes only duplicated content agreement.
6. **Behavior and economics.** Algorithm selection, table and virtual models,
   viewport geometry, floating placement, clipping, child order, frame order,
   scene order, renderer topology, batching/pass fusion, invalidation,
   allocation, and presentation clocks are unchanged. The same borrowed models
   enter the same algorithms without optional recovery or a derived-role
   branch.
7. **Doctrine and proof.** Master design now requires both frame construction
   and recursive layout dispatch to consume typed content directly. The
   architecture witness failed against the role/recovery path, then pinned the
   direct content match, namespaced support types, and displaced assertions.
   The 221-test layout slice and the full library passed: 1,118 passed with ten
   standing ignores. All targets and all five examples compiled without
   warnings; all ten census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, allowances,
   and panics remain 325/109, 3, 54, 0/0/0, 1,825 in 192 files, 90, 118, 363,
   6, and 6. Removing the two payload assertions lowers production expects
   61 -> 59. Recursive typed-content dispatch is at fixed point; the reverse
   sweep continues through the remaining table/virtual topology lifecycles and
   the complete visibility, failure, intermediate, housing, and naming
   inventories. This cell does not close Rung 5.

### R5-51 — table and virtual-child topology resistance

Status: **complete resistance ruling; no production correction admitted**.

1. **Question and complete trace.** The remaining layout-topology sweep traced
   the sole table widget producer through its outer table, horizontal scroll,
   surface, header, virtual body, retained composition, table projection,
   layout, scene, and hit consumers. It also traced virtual-list model
   construction, request/refinement, the sole materialization assignment,
   provider-row stamping, reconciliation keys, selection projection, pinning,
   fixed and variable layout, frame projection, scene paint, and runtime
   reveal. Every surface/header and provided-row assertion or optional
   projection was included.
2. **Table topology ruling.** A table scroll has one surface and that surface
   begins with its header, but both relationships currently live in the same
   recursive `Node::children` tree consumed by composition, generic stack
   layout, scene, hits, and public inspection. Passing a surface to a revised
   constructor would still leave header presence dynamic; separately storing
   header or surface would duplicate the node and its identity; flattening the
   surface would change frame and stack topology. The two local assertions
   therefore remain honest guards over the one private producer.
3. **Virtual topology ruling.** `Model::materialize` is the sole producer of
   virtual children and stamps every provider key/index before one assignment
   to the virtual-list node. The row fact semantically belongs to that
   parent-child edge, yet `Node` exposes one homogeneous child slice and the
   same child may also carry table-row meaning. A local row wrapper, checked
   iterator, or centralized assertion would not prevent an unstamped child;
   filtering or defaulting would corrupt layout, selection, reconciliation,
   and stable identity rather than represent absence truthfully.
4. **Required structural threshold.** Making these laws unrepresentable
   requires one broader recursive-child design: parent-specific child species
   that carry edge metadata while preserving generic traversal, retained-tree
   alignment, public `children()` behavior, and table-row co-occurrence. That
   entails a new recursive envelope and migration of every composition/view/
   layout traversal. No second consumer or observed defect currently pays for
   that campaign inside this bounded cell, and a second common node envelope
   repeats the resistance already recorded by R5-49.
5. **Boundary and naming ruling.** No intermediate, visibility, projection,
   alias, or call-site import was added. `view::Node` remains the sole parent
   projection and support remains namespaced under `view::node`; inventing a
   flattened `VirtualRowNode`, `TableSurfaceNode`, or aliased compound type
   would violate the canonical naming law without making topology structural.
6. **Behavior and economics.** Table geometry, virtual measurement/refinement,
   provider identity, selection, pinned rows, reconciliation, child order,
   frame and scene order, renderer topology, batching/pass fusion,
   invalidation, allocation, and presentation clocks remain byte-for-byte
   unchanged. Defaults, truncating zips, and silent omission remain rejected.
7. **Proof, gauge, and next frontier.** The source census found one table-scroll
   producer, one virtual-list materialization assignment, and one provider-row
   stamping route; all consumers resolve through those paths. R5-50's unchanged
   full boundary remains the proof: 1,118 passed with ten ignored; all targets,
   examples, census witnesses, formatting, diff, and protected state green.
   Every gauge remains unchanged, including 59 production expects. The reverse
   sweep continues with post-insertion collection recovery and the remaining
   standard-menu, layout, visibility, failure, intermediate, housing, and
   naming inventories; this cell does not close Rung 5.

### R5-52 — total post-insertion UI collection recovery

Status: **complete; two false post-push absences removed**. Correction
`3a0bd574` (`Borrow inserted UI entries directly`).

1. **Question and complete trace.** The failure inventory traced composition
   creation, replacement, lookup, reconciliation, removal, and window cleanup,
   then traced selectable-list lookup, insertion, mutation, reconciliation,
   snapshots, restore, and every selection consumer. Both stores used ordered
   vectors for stable iteration and recovered an element immediately after
   pushing it through `last[_mut]().expect(...)`.
2. **False absence.** In each absent-key branch, the vector length is known,
   exactly one value is pushed, and no intervening operation can remove or
   reorder it. General `Vec::last` optionality therefore leaked a collection
   query into an operation whose postcondition is total. Allocation failure
   retains Rust's ordinary process behavior and is not an application-level
   absence outcome.
3. **Correction and displaced paths.** Each owner captures the pre-push length,
   pushes the unchanged value, and borrows the inserted slot at that index.
   Both option recoveries and assertions are deleted. Existing-key branches,
   vector storage, uniqueness checks, order, and every consumer are unchanged.
4. **Repetition and boundary ruling.** The two owners share a mechanical
   postcondition, not domain meaning, so no common helper, collection wrapper,
   trait, or lower seam is admitted. Composition still owns installed views and
   retained trees; interaction still owns per-list selection state. Each local
   three-step operation is the smallest truthful form.
5. **Naming and visibility ruling.** No type, method, visibility, projection,
   alias, or import changed. The touched namesake modules keep their existing
   central projections and introduce no supporting name, compound declaration,
   or flattened parent export.
6. **Behavior and economics.** Window ordering, composition identity and
   replacement, selection ordering, snapshot order, reconciliation, retained
   rows, allocation count, layout, scene order, renderer topology,
   invalidation, and presentation clocks are unchanged. Both operations avoid
   constructing and branching on an impossible `Option` after insertion.
7. **Proof.** Seventeen composition-focused and forty-one interaction-focused
   witnesses passed. The full library passed 1,118 tests with ten standing
   ignores; all targets and all five examples compiled without warnings. All
   ten census parser witnesses, the full census, formatting, diff, and
   protected-state checks passed.
8. **Gauge delta and next frontier.** Every graph, visibility, test-edge,
   source-root, filesystem, allowance, and panic gauge remains unchanged:
   production/test edges 325/109, split responsibilities 3, slot edges 54,
   forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,825 in 192
   files, cross-slot upper bound 1,778, cross-slot test edges 90, source-root
   mentions 118, filesystem reads 363, allowances 6, and panics 6. Production
   expects fall 59 -> 57. Post-insertion recovery is at fixed point; the reverse
   sweep continues through standard-menu topology and the remaining layout,
   visibility, failure, intermediate, housing, and naming inventories. This
   cell does not close Rung 5.

### R5-53 — single-pass standard-menu topology and namespaced extension support

Status: **complete; duplicate anchor recovery, parallel section state, and two
parent aliases removed**. Correction `e8deaee8` (`Make standard menu topology
single pass`).

1. **Question and complete trace.** The standard-menu sweep traced every
   platform slot and virtual marker through command registration, category
   catalogs, blueprint grouping, live action resolution, standard-bar view
   projection, all seven authored extension species, hidden commands,
   replacement/ordering, final menu nodes, and public mixed-bar builders. It
   distinguished framework topology agreement from invalid author requests.
2. **Duplicate agreement and asserted recovery.** Command templates stored the
   current section beside a vector of sections, pushed a group on transition,
   then asserted that group existed. View extension projection first located a
   section containing an anchor, then rescanned that section and asserted the
   same anchor existed to recover its entry index. Both assertions represented
   agreement already established by the immediately preceding operation.
3. **Correction and displaced paths.** Command templates now group
   `(Section, Vec<Standard>)` in one pass and discard the grouping key only
   after construction. View projection resolves each anchor once to one private
   `Location { category, section, entry }` and consumes those aligned indices
   for item or section mutation. The parallel current-section option, post-push
   recovery, second entry scan, helper chain, and both assertions are deleted.
4. **Failure resistance.** `CommandPalette` remains an unplaceable standard and
   an unregistered custom category remains absent from the registry-derived
   catalog. Those are public author-contract violations detected at their
   existing construction/projection boundaries; silently omitting them or
   manufacturing a category would change behavior and hide configuration
   errors. The two explicit panics therefore remain.
5. **Canonical naming correction.** The touched support seam no longer aliases
   `standard_menu::Extension` as `StandardMenuExtension` at `node` and then
   again at `view`. `view::Node` remains the central parent projection;
   `view::node::standard_menu` is the crate-visible support module; and widget,
   content, builder, and accessor callsites name
   `node::standard_menu::Extension`. No flattened supporting re-export or
   compound alias survives at either parent.
6. **Behavior and economics.** Platform category/section/item order, virtual
   markers, missing live actions, authored before/after ordering, section and
   category replacement, hidden commands, shortcuts, menu focus/activation,
   allocation, layout, scene order, renderer topology, invalidation, and
   presentation clocks are unchanged. Anchored item extensions now scan the
   projected topology once rather than twice.
7. **Doctrine and proof.** Master design now requires one template grouping,
   one extension-anchor location, explicit author failures, and namespaced
   support vocabulary. The architecture witness failed before correction, then
   pinned the aligned location, extinct assertions/helpers, and absent parent
   aliases. Forty-three command-focused witnesses passed. The full library
   passed 1,118 tests with ten standing ignores; all targets and all five
   examples compiled without warnings. Census, formatting, diff, and protected
   state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, cross-slot
   test edges, source-root mentions, filesystem reads, allowances, and panics
   remain 325/109, 3, 54, 0/0/0, 90, 118, 363, 6, and 6. Removing the flattened
   parent support projection lowers production `pub(crate)` 1,825 -> 1,824 and
   the cross-slot upper bound 1,778 -> 1,777. Removing the two false recoveries
   lowers production expects 57 -> 55. Standard-menu topology is at fixed
   point; the reverse sweep continues through remaining layout/session failure,
   visibility, intermediate, housing, and naming inventories. This cell does
   not close Rung 5.

### R5-54 — structural transaction-history preparation plan

Status: **complete; parallel history policy and optional snapshot collapsed**.
Correction `23395a65` (`Make transaction history planning structural`).

1. **Question and complete trace.** The transaction-history sweep traced typed
   and erased command dispatch, missing targets, dispatch failures, observer
   failures, changed and unchanged responses, automatic gestures, history-group
   coalescing, framework-service commits, ignored commands, notifications,
   revision repair, retained snapshots, timeline recording, undo/redo, and
   departure delivery. Every preparation and completion callsite was included.
2. **Invalid state and duplicate agreement.** Runtime passed
   `command::History` beside `Option<PendingSnapshot>` even though Automatic
   always prepared one and Committed/Ignored never did. The representation
   admitted all mismatches, made unchanged completion rediscover snapshot
   presence, and made changed Automatic completion assert it. Public policy and
   its runtime preparation result were parallel truths.
3. **Correction and displaced paths.** Private
   `history::Plan::{Automatic(PendingSnapshot), Unrecorded}` now owns the
   completion species. Preparation converts public policy once; every typed,
   erased, early-return, observer-error, and notification path carries that one
   value; completion consumes it exhaustively. The independent history/snapshot
   parameters, optional recovery, and assertion are deleted.
4. **Semantic resistance.** Public `command::History::{Committed, Ignored}`
   remain distinct author policy: one says handling commits through framework
   services and the other says the command is nonundoable. Runtime completion
   legitimately treats both as no-runtime-snapshot work, so they share only the
   internal `Unrecorded` species rather than being merged publicly.
5. **Boundary and naming ruling.** Runtime remains the owner of transaction
   preparation/completion while state owns prepared snapshots and timeline owns
   undo history. `Plan` is supporting vocabulary inside the existing `history`
   module and receives no parent projection, alias, or flattened export. No
   compound declaration is re-exported under a simpler spelling and no public
   command name changes.
6. **Behavior and economics.** Automatic unchanged work restores the retained
   snapshot; changed work records, coalesces, or commits exactly as before;
   committed/ignored work clears grouping and repairs revision exactly as
   before. Snapshot clone elision, model revisions, notifications, departure
   ordering, allocation, layout, scene order, renderer topology, invalidation,
   and presentation clocks are unchanged.
7. **Doctrine and proof.** Master design now requires one prepared history plan
   whose Automatic species owns its snapshot. The architecture witness failed
   against the parallel state, then pinned both species and the removed
   assertion. Nineteen history-focused and thirty-seven runtime-focused tests
   passed. The full library discovered 1,129 tests: 1,119 passed with ten
   standing ignores. All targets/examples, census, formatting, diff, and
   protected-state checks passed.
8. **Gauge delta and next frontier.** Every graph, visibility, test-edge,
   source-root, filesystem, allowance, and panic gauge remains unchanged:
   production/test edges 325/109, split responsibilities 3, slot edges 54,
   forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,824 in 192
   files, cross-slot upper bound 1,777, cross-slot test edges 90, source-root
   mentions 118, filesystem reads 363, allowances 6, and panics 6. Production
   expects fall 55 -> 54. Transaction-history preparation is at fixed point;
   the reverse sweep continues through command/response collection cardinality
   and the remaining layout/session visibility, failure, intermediate, housing,
   and naming inventories. This cell does not close Rung 5.

### R5-55 — total command collection cardinality and namespaced support

Status: **complete; two false collection absences and two flattened support
projections removed**. Correction `0cb0c624` (`Make command collection
cardinality total`).

1. **Question and complete trace.** The command/response collection sweep
   traced responder builder registration through object target/listener
   attachment, ordered exact and broad routing, focus and application
   precedence, and every chain consumer. It also traced response-effect
   composition through nested batches, duplicate removal, invalidation-depth
   collapse, command/notification outcomes, runtime side-effect consumption,
   and renderer invalidation scheduling.
2. **False absence and retained semantics.** Responder registration pushed one
   spec and immediately recovered it with `last_mut().expect(...)`; effect
   normalization checked a vector length of one and then recovered that element
   with `pop().expect(...)`. Neither option represented an application outcome.
   Ordered responder registration, effect deduplication, noninvalidation order,
   and maximum invalidation depth remain real owner laws and are unchanged.
3. **Correction and displaced paths.** Responder builder now records the
   pre-push index, pushes exactly one spec, and lends that inserted slot
   directly. Effect normalization pops once: absence becomes `None`, an empty
   remainder yields the sole effect, and a nonempty remainder receives the
   popped tail again before becoming `Batch`. Both assertions and both false
   optional recoveries are deleted while the existing vector allocation and
   normalized order are preserved.
4. **Repetition and naming ruling.** The two sites share a collection
   postcondition, not domain meaning, so no helper, trait, wrapper, or lower
   seam is admitted. Because both namesake seams were touched, the canonical
   projection law applies fully: `responder::Builder` is the builder module's
   only parent projection and support remains
   `responder::builder::Object`; `response::Effect` is the effect module's only
   parent projection and support remains
   `response::effect::Invalidation`. The former flattened `responder::Object`
   and `response::Invalidation` projections and every latter callsite are
   retired without aliases.
5. **Behavior and economics.** Responder identity, insertion and traversal
   order, exact/broad claim precedence, target/listener attachment, effect
   associativity, duplicate removal, invalidation strength, dialog/panel
   effect order, allocation, layout, scene order, renderer topology,
   batching/pass fusion, invalidation, and presentation clocks are unchanged.
   The responder path avoids an impossible option branch; multi-effect
   normalization reuses the same vector allocation and order.
6. **Doctrine and witnesses.** Master design now names both direct cardinality
   operations and both namespaced support surfaces. The architecture witness
   pins the sole parent projections, public support modules, direct inserted
   slot borrow, structural effect collapse, and extinction of the flattened
   exports and assertions. A focused owner test proves zero, one, and multiple
   effect cardinalities without reordering.
7. **Proof.** The three effect-owner tests, four responder-chain tests, and the
   architecture witness passed directly. The full library discovered 1,131
   tests and passed 1,121 with ten standing ignores. All targets and all five
   examples compiled without warnings; all ten census parser witnesses, the
   full census, formatting, diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, allowances,
   and panics remain 325/109, 3, 54, 0/0/0, 1,824 in 192 files, 90, 118, 363,
   6, and 6. Production expects fall 54 -> 52. Command collection cardinality
   and the touched naming surfaces are at fixed point; the reverse sweep
   continues through required versus optional erased-command invocation and
   the remaining layout/session visibility, failure, intermediate, housing,
   and naming inventories. This cell does not close Rung 5.

### R5-56 — structural erased-command invocation cardinality

Status: **complete; required and optional invocation results separated over one
prepared transaction**. Correction `964ef2ee` (`Make erased command
cardinality structural`).

1. **Question and complete trace.** The erased-command transaction sweep
   traced resolved view bindings, menu and slider activation, palette selection,
   keyboard shortcuts, registry absence, disabled shortcuts, missing targets,
   pre-dispatch failure, command and observer mutation, history preparation and
   completion, timeline restore, text-input cleanup, and every transaction
   effect consumer. All three erased invocation entrances and every early or
   normal completion path were included.
2. **Distinct cardinalities and false absence.** Palette and shortcut lookup
   legitimately produce no response when no registered or enabled command is
   available. A resolved `view::Binding`, however, invokes one typed erased
   trigger and always returns `AnyResponse`; unknown command, target, or output
   failures are values inside that response rather than absence. The shared
   optional transaction return weakened binding activation and forced an
   assertion to recover the total result.
3. **Correction and displaced paths.** Runtime now exposes private required
   and optional erased-command transaction entrances. Required invocation
   accepts and returns total response/outcome values; optional invocation alone
   accepts and returns `Option`. The old ambiguous entrance, binding's
   `Ok(Some(...))` promotion, and its post-transaction assertion are deleted.
   Palette and shortcut callsites name their optionality explicitly.
4. **Prepared-token admission and centralization.** One private
   `Prepared<M>` token owns the `AnyInvocation`, structural history plan, and
   pre-dispatch revision from context construction until exactly one finish.
   It earns its existence by making one-shot transaction completion structural
   across pre-dispatch error, optional absence, observer failure, and normal
   response. Both cardinality entrances share one context/responder invocation
   setup and one response/observer/effect completion; repeated finish argument
   lists and history-group clones are deleted rather than moved into wrappers.
5. **Boundary and naming ruling.** Runtime remains the transaction owner;
   command registry and responders own invocation, state/timeline own history,
   and response owns the erased result. `Prepared` is private support in the
   existing transaction command housing and receives no projection, alias, or
   visibility widening. No compound declaration is re-exported under a simple
   name and no namesake parent surface is touched.
6. **Behavior and economics.** Palette and shortcut absence, binding failures,
   target routing, command source, context-menu scope, observer ordering,
   changed-state calculation, history coalescing/restoration, timeline text
   cleanup, effects, allocation, layout, scene order, renderer topology,
   invalidation, and presentation clocks are unchanged. Required activation
   removes one impossible option construction and branch; transaction facts
   are moved once instead of cloning the history group for failure paths.
7. **Doctrine and proof.** Master design now states the two erased invocation
   cardinalities and one prepared completion lifecycle. The architecture
   witness pins both named entrances, their required/optional callsites, the
   prepared token, shared completion, and extinction of the ambiguous method
   and assertion. Command, palette, pointer-binding, interaction, and slider
   slices passed. The full library discovered 1,132 tests and passed 1,122 with
   ten standing ignores. All targets and all five examples compiled without
   warnings; all ten census parser witnesses, the full census, formatting,
   diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, allowances,
   and panics remain 325/109, 3, 54, 0/0/0, 1,824 in 192 files, 90, 118, 363,
   6, and 6. Production expects fall 52 -> 51. Erased-command cardinality is at
   fixed point; the reverse sweep continues through the remaining layout/
   session failure, visibility, intermediate, housing, and naming inventories.
    This cell does not close Rung 5.

### R5-57 — structural text shaping and LRU cache admission

Status: **complete; shaping cardinality and nonzero cache capacity encoded at
their owner**. Correction `7e05017a` (`Make text cache admission structural`).

1. **Question and complete trace.** The remaining text-layout cache assertions
   were traced through area-line display shaping, field surfaces, inline text
   and icons, height indices, render buffers, measurement, committed and preedit
   retention, cache hits/misses/eviction, diagnostics, and every constructor.
   All `ShapingCache`, direct `LruCache`, and measurement-FIFO capacity entrances
   were included rather than treating the assertions as isolated syntax.
2. **Distinct cardinalities and capacity semantics.** Area-line and field
   preparation are total after their keys are admitted; neither preparation
   function has an absence outcome. Inline text and icon preparation remains
   legitimately optional because empty, multi-run, or unavailable glyph
   preparation can decline. Every LRU cache requires positive capacity, while
   the separate measurement FIFO deliberately supports zero as a disabled-cache
   mode. One optional shaping entrance and one `usize` capacity policy could not
   truthfully represent all three distinctions.
3. **Correction and displaced paths.** `ShapingCache` now exposes
   `shape_required` and `shape_optional`; both share one cache-hit projection and
   one insertion path. Area and field consume the total entrance directly, and
   field preparation returns its value rather than wrapping it. Inline text and
   icon callsites name optionality explicitly, and absence never enters the
   cache. The two total-result assertions and their false `Option` paths are
   deleted.
4. **Typed cache admission.** `ShapingCache::new` and both direct LRU
   constructors now accept `NonZeroUsize`. Their fixed retention constants carry
   that type from declaration to construction, deleting the shaping-cache panic
   and two capacity assertions. The measurement FIFO stays `usize` and retains
   its existing zero-disabled branch; the correction does not falsely erase
   that independent policy.
5. **Boundary and naming ruling.** Text layout remains the one shaping/cache
   mechanics owner; area, field, and inline modules retain their domain keys and
   retention limits. The two entrance names describe result cardinality and no
   wrapper, alias, parent projection, support re-export, or visibility change is
   introduced. No compound declaration is exposed under a simpler name and no
   namesake parent surface is touched.
6. **Behavior and economics.** Cache keys, capacity values, LRU/FIFO eviction,
   committed versus transient retention, shaped-buffer identity, hit/miss and
   shaping diagnostics, allocation, layout, scene order, renderer topology,
   batching/pass fusion, invalidation, and presentation clocks are unchanged.
   Required preparation removes an impossible option construction/branch; cache
   lookup and insertion remain centralized and occur in the same order.
7. **Doctrine and proof.** Master design now states required versus optional
   shaping cardinality, typed positive LRU admission, and the measurement FIFO
   exception. An owner witness covers required miss/hit and optional absence/
   miss/hit; the architecture witness pins both entrances, typed constructors,
   total callsites, optional inline callsites, and all displaced assertions.
   The focused text slice passed 131 tests with two standing ignores. The full
   library discovered 1,134 tests and passed 1,124 with ten standing ignores.
   All targets and all five examples compiled without warnings; all ten census
   parser witnesses, the full census, formatting, diff, and protected-state
   checks passed.
8. **Gauge delta and next frontier.** Production/test edges, split
   responsibilities, slot edges, forbidden/external/SCC counts, visibility,
   cross-slot test edges, source-root mentions, filesystem reads, and allowances
   remain 325/109, 3, 54, 0/0/0, 1,824 in 192 files, 90, 118, 363, and 6.
   Removing the cache panic lowers production panics 6 -> 5; removing the four
   runtime assertions lowers production expects 51 -> 47. Text shaping/cache
   admission is at fixed point; the reverse sweep continues through the
   remaining layout/session visibility, failure, intermediate, housing, and
   naming inventories. This cell does not close Rung 5.

### R5-58 — borrowed table-service model admission

Status: **complete; admitted provider model lent through typed service work**.
Correction `78b2a5e7` (`Lend admitted table service models`).

1. **Question and complete trace.** The remaining runtime table-service
   assertion was traced from contextual and focused table scopes through
   composition lookup, typed provider claim/state/invoke, exact and broad
   responder routing, canonical Select All, session selection, all-except
   representation, provider deletion, and contextual target enumeration. The
   generic typed-target adapter and the surrounding service precedence path were
   included.
2. **Duplicate lookup and false absence.** Service admission proved that the
   scoped window/table still resolved one virtual-list model, but stored only
   the two keys. Each provider-target construction then repeated the composition
   lookup, cloned the model, and asserted presence. Typed invocation constructs
   a target for state and another for invoke, so one admitted service value
   performed the same lookup and clone twice even though no intervening service
   operation can replace its borrowed composition.
3. **Correction and displaced paths.** One `table_model` projection resolves
   window plus borrowed model. `table_for` constructs the service from that
   admitted value; `Table` and `SelectionTarget` lend the same model reference
   through state and invocation. Claim, invoke, and contextual enumeration use
   that projection. The retained composition/table-key fields, repeated lookup,
   two model clones, false optional recovery, and assertion are deleted.
4. **Revalidation resistance.** Responder-chain claim and later invocation
   remain separate live command operations and continue to revalidate their
   route. This cell does not cache command availability or let a model borrow
   escape the service transaction. Whether the generic service claim receipt
   should carry more typed invocation identity is a broader responder-protocol
   question kept visible for the next sweep rather than presumed here.
5. **Boundary and naming ruling.** Composition remains model-projection owner,
   runtime services own contextual command realization, and session owns keyed
   selection mutation. The existing private `Table` and `SelectionTarget`
   declarations are not re-exported or aliased; no public name, compound-to-
   simple projection, supporting parent export, or visibility changes.
6. **Behavior and economics.** Table versus focused-text precedence, exact
   routes, command state, disabled/missing outcomes, Select All membership,
   million-row all-except behavior, contextual ordering, selection
   reconciliation, allocation, layout, scene order, renderer topology,
   invalidation, and presentation clocks are unchanged. Each admitted table
   service value now performs one model lookup and no model clone.
7. **Doctrine and proof.** Master design now requires the table service to lend
   its admitted model. The architecture witness pins the borrowed model, one
   projection/construction path, and extinct key/relookup/clone/assertion path.
   Sixty-two table-focused tests passed, including table context precedence and
   bounded million-row Select All. The full library discovered 1,135 tests and
   passed 1,125 with ten standing ignores. All targets and all five examples
   compiled without warnings; all ten census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed.
8. **Gauge delta and next frontier.** Every graph, visibility, test-edge,
   source-root, filesystem, allowance, and panic gauge remains unchanged:
   production/test edges 325/109, split responsibilities 3, slot edges 54,
   forbidden/external/SCC counts 0/0/0, production `pub(crate)` 1,824 in 192
   files, cross-slot upper bound 1,777, cross-slot test edges 90, source-root
   mentions 118, filesystem reads 363, allowances 6, and panics 5. Production
   expects fall 47 -> 46. Table-model admission is at fixed point; the reverse
   sweep continues through the responder service claim/invoke protocol and the
   remaining layout/session visibility, failure, intermediate, housing, and
   naming inventories. This cell does not close Rung 5.

### R5-59 — claimed responder-service identity through broad invocation

Status: **complete; discarded claimant and replayed service precedence
removed**. Correction `78e9cc49` (`Carry responder service claimant identity`).

1. **Question and complete trace.** The responder-service protocol sweep traced
   broad and exact state resolution and invocation through ordinary responders,
   active focused text, table selection, inactive focused text, system
   undo/redo and window commands, palette query versus captured document,
   context-menu routes, typed provider targets, focus handoff, disabled and
   missing outcomes, and every `responder::Service` implementation and
   construction site.
2. **Discarded identity and repeated policy.** Broad `Chain::invoke_any`
   resolved one service `Claim` and then discarded it. The sole runtime service
   implementation replayed the complete active-text -> table -> text -> system
   ladder to rediscover the claimant, after which the selected typed service
   re-claimed its target a third time before invocation. The middle replay
   duplicated precedence, repeated composition/session work, and contained two
   `.ok().flatten()` paths that could discard a repeated claim error. It
   preserved no identity, lifecycle, or freshness beyond the immediately
   adjacent claim.
3. **Correction and displaced paths.** `Service::invoke` now receives the
   adjacent responder `Claim`, and the chain passes the exact value it just
   resolved. Runtime dispatches focused text, table selection, or system work
   from that claim's provenance name. One private contextual invocation helper
   is shared by broad and exact table/text routes. The replayed ladder,
   `text::state`, the generic typed-service state-only adapter, repeated claim
   error handling, and all associated optional recovery are deleted.
4. **Revalidation and exact-route ruling.** The claim receipt chooses only the
   subservice; it is not an availability lease and carries no cached typed
   target. `service_target::invoke` still calls `claim_target` and checks live
   hidden/disabled/ambiguous state immediately before invoking. Focused text
   still resolves its live base text, performs the established focus handoff,
   and only then revalidates the typed target. `Route::Service` still performs
   its separate exact re-claim before exact invocation, so a departed contextual
   owner remains missing rather than falling through to a broader service.
5. **Boundary and naming ruling.** Responder continues to own claim provenance
   and the one chain transaction; runtime services continue to own text/table/
   system realization and typed target adapters. Provenance exposes its existing
   simple `name` fact to the service consumer; no public API, parent projection,
   supporting re-export, compound declaration, alias, or call-site spelling was
   introduced. The touched seams therefore satisfy the canonical naming law
   without unrelated cleanup.
6. **Behavior and economics.** Ordinary responder priority, active text over
   table, table over inactive text, system fallback, disabled stopping, exact
   route identity, palette query ownership, table Select All, draft history,
   focus timing, command effects, allocation, layout, scene order, renderer
   topology, batching/pass fusion, invalidation, and presentation clocks are
   unchanged. A broad service invocation now performs one claimant resolution
   plus the required typed-target revalidation instead of replaying the whole
   semantic ladder between them.
7. **Doctrine and proof.** Master design now distinguishes adjacent claimant
   identity from live typed-target availability. A responder-owner witness
   proves the chain invokes from one exact claim; the architecture witness pins
   claimant dispatch, extinction of the middle replay and state-only adapters,
   focus-before-revalidation, typed availability checks, and retained exact
   route re-claiming. Focused active-text, exact text, table-only, palette, and
   system branches passed. The full library discovered 1,137 tests and passed
   1,127 with ten standing ignores. All targets and all five examples compiled
   without warnings; all ten census parser witnesses, the full census,
   formatting, diff, and protected-state checks passed.
8. **Gauge and next frontier.** Every gauge remains unchanged: production/test
   edges 325/109, split responsibilities 3, slot edges 54, forbidden/external/
   SCC counts 0/0/0, production `pub(crate)` 1,825 in 192 files, cross-slot
   upper bound 1,778, cross-slot test edges 90, source-root mentions 118,
   filesystem reads 363, allowances 6, panics 5, and expects 46. Broad service
   invocation is at fixed point; the reverse sweep continues through the
   remaining layout/session visibility, failure, intermediate, housing, and
   naming inventories. This cell does not close Rung 5.

### R5-60 — typed-table projection and ordering assertions

Status: **resistance; private heterogeneous boundary retained without
correction**.

1. **Question and complete trace.** The assertion inventory traced free and
   typed table construction, every text/Boolean/custom column builder, lazy
   `Source::new` and bounded `Source::records`, derived sortable and custom
   headers, projected sort state, ascending/descending/tied ordering, stable
   key lookup, compact/expanded row materialization, one-record-per-row
   caching, and every public `Provider` implementation and consumer. The four
   production expects in the slice are the two record downcasts performed by
   an ordering projection, the just-populated projected-record cache, and the
   cell projection selected by a declared column id.
2. **Owner and invariant.** Typed construction captures one homogeneous record
   type while producing heterogeneous columns for the record-agnostic `Table`
   and public free-provider path. One optional ordering projection is the sole
   capability truth: its presence derives the header affordance and the same
   value orders bounded records. `TypedColumn` fields are private, and
   `Table::typed` creates the column list, cell map, and ordering map in one
   pass, so no external caller can pair a projection with another record type,
   omit a declared cell projection, or address the cache before the cell path
   populates it.
3. **Challenge and rejected rewrites.** A generic comparator in
   `TypedProvider<R>` removes the downcasts only by storing a second sortable
   flag or id set for record-agnostic headers; that creates competing retained
   truth forbidden by standing doctrine. Querying the typed map through a
   closure or new private adapter trait hides the same dependency behind a
   callback/helper boundary and adds dispatch plus another intermediate. A
   default method on public `Provider` spends permanent public surface on an
   implementation detail. Making `Table`, `Column`, or the free provider path
   generic broadens and couples the established escape hatch. Deferring typed
   assembly until node conversion adds a second builder lifecycle merely to
   move the same proof in time. None deletes more machinery or yields a
   smaller contract.
4. **Ruling.** Retain `Any` at this one private erased boundary and retain the
   four expects as construction-invariant assertions. They do not translate an
   operational absence into a panic: record type, declared projection, and
   populated-cache membership are all established immediately upstream by
   the same private constructor/path. Lazy providers remain unenumbered and
   application-ordered; bounded providers alone consume the comparator. This
   is the same typed-capture-then-private-erasure law already practiced by the
   command, task, notification, and table systems, not a public type-erasure
   API.
5. **Naming, visibility, and behavior.** No declaration, projection, parent
   re-export, alias, call-site spelling, visibility, module housing, public
   API, or dependency edge changes. The canonical namesake-module law is
   therefore satisfied without opportunistic naming work. Selection, active
   cells, editing, validation, stable identity, sorting, virtualization,
   allocation, layout, paint order, renderer topology, batching/pass fusion,
   invalidation, and presentation clocks remain byte-for-byte on their prior
   paths.
6. **Proof.** All five table owner tests passed, including std capability
   admission, one parse/validation pass, Boolean projection, shared header and
   bounded-record ordering, ties, reverse order, key lookup, replacement, and
   one record projection across visible cells. The architecture witness pins
   extinction of framework capability mirrors and a single optional ordering
   projection with no sortable flag. All four table doctests passed, including
   the three negative capability cases. The immediately preceding full
   R5-59 boundary remains the repository-wide green proof because this cell
   changes no production or doctrine path.
7. **Gauge and next frontier.** Every gauge remains unchanged: production/test
   edges 325/109, split responsibilities 3, slot edges 54, forbidden/external/
   SCC counts 0/0/0, production `pub(crate)` 1,825 in 192 files, cross-slot
   upper bound 1,778, cross-slot test edges 90, source-root mentions 118,
   filesystem reads 363, allowances 6, panics 5, and expects 46. The table
   assertion slice is at fixed point; the reverse sweep continues through the
   remaining layout/session/runtime visibility, failure, intermediate,
   housing, and naming inventories. This zero-change cell does not close
   Rung 5.

### R5-61 — fixed-cardinality panel-placement fallback

Status: **complete; discarded nonempty proof removed**. Correction
`211e9069` (`Preserve placement cardinality structurally`).

1. **Question and complete trace.** The production-assertion sweep traced the
   one pure geometry placement request from point-anchored context menus and
   hover tips, rectangle-anchored panels, layout projection, overlay retention,
   native work-area projection, in-frame realization, and the Windows native
   popup position adapter. It covered first-fitting right/down preference,
   every horizontal and vertical flip, pointer clearance, oversized desired
   panels, negative available origins, no-resize fallback, equal intersection
   areas, and native versus in-frame consumers.
2. **Discarded proof.** `candidates` already returns `[Rect; 4]`, making
   cardinality and nonemptiness structural. `Request::resolve` converted that
   fixed array into an iterator, used the optional result of `max_by_key`, and
   immediately recovered the proof with
   `expect("panel placement always has four candidates")`. The `Option` added
   no absence state, failure model, identity, or lifecycle; it merely forgot a
   fact the type had already established.
3. **Correction.** The fallback now destructures the four rectangles and folds
   the remaining three from the first value. `>=` deliberately preserves
   `Iterator::max_by_key`'s established last-wins behavior for equal
   intersection areas. The first-contained short circuit, candidate order,
   intersection law, saturating arithmetic, origin clamp, desired size, and
   clearance are unchanged. No helper, wrapper, collection, callback, trait,
   or new type replaces the deleted optional protocol.
4. **Ownership and boundary ruling.** Geometry continues to own the pure
   anchor/desired/available placement law. Layout supplies logical anchors and
   desired size; platform supplies work-area facts and projects the resolved
   logical origin physically; overlay owns panel lifecycle. No consumer learns
   candidate ordering or performs a parallel fit/clamp decision. The
   correction therefore strengthens the existing lower seam without moving or
   widening it.
5. **Naming and visibility.** No declaration, public or parent re-export,
   call-site spelling, visibility, module housing, feature surface, or public
   API changes. The canonical namesake-module and projection laws are
   satisfied without unrelated naming cleanup.
6. **Behavior and economics.** The four-element stack array, first-fit scan,
   fallback comparisons, and clamp remain allocation-free and bounded. Equal
   areas still choose the final candidate; a focused witness uses a panel wider
   than the available rect but vertically movable so a changed tie rule would
   produce observably different geometry. Layout invalidation, overlay
   retention, native realization, paint order, renderer topology,
   batching/pass fusion, and presentation clocks are untouched.
7. **Proof.** All five geometry-placement owner tests passed, including all
   edge flips, clearance, oversized no-resize clamp, rectangle anchors, and the
   new equal-area tie receipt. The full library discovered 1,138 tests and
   passed 1,128 with ten standing ignores. All targets and all five examples
   compiled without warnings; all ten census parser tests, the full census,
   formatting, diff hygiene, and protected `comparison_open: true` check
   passed.
8. **Gauge and next frontier.** Production expects fall 46 -> 45. Every other
   gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   production `pub(crate)` 1,825 in 192 files, cross-slot upper bound 1,778,
   cross-slot test edges 90, source-root mentions 118, filesystem reads 363,
   allowances 6, and panics 5. Placement fallback is at fixed point; the
   reverse sweep continues through remaining layout/session/runtime
   visibility, failure, intermediate, housing, and naming inventories. This
   cell does not close Rung 5.

### R5-62 — total shadow reach versus optional item bounds

Status: **complete; synthetic item round-trip and recovered totality removed**.
Correction `3a8fbc2f` (`Centralize shadow visual bounds`).

1. **Question and complete trace.** The production-assertion sweep traced
   scene shadow recipes through scene-to-paint conversion, group bounds,
   popup visual-envelope resolution, panel-local paint translation, HWND and
   swapchain sizing, composition material-region translation, pointer
   translation, and physical popup placement. It also traced every paint item
   species through generic group bounds, including the legitimate absence of
   bounds for clip push/pop commands.
2. **Repeated decision and false optionality.** `item_bounds` already contained
   the authoritative blur, spread, offset, and one-physical-pixel fringe
   calculation for a shadow. The crate-visible `shadow_visual_bounds` path
   wrapped the value in a synthetic `Item::Shadow`, invoked the generic
   optional item query, and recovered totality with an expect. Optionality was
   truthful for clip commands but impossible for the concrete `Shadow` input;
   the round-trip obscured the lower semantic owner and made its only direct
   consumer handle a state that could not occur.
3. **Correction and deletion.** The existing `shadow_visual_bounds` function
   now owns the calculation directly. Generic `item_bounds` delegates its
   shadow arm to that function and wraps the total result in `Some` for the
   heterogeneous collection protocol. The synthetic enum construction,
   generic redispatch, duplicated ownership direction, and expect are deleted.
   No new helper or representation replaces them.
4. **Ownership and non-merge ruling.** Paint remains the sole owner of visual
   reach because it owns physical-pixel snapping and the exact raster recipe.
   Renderer scene projection consumes that fact for popup envelopes; platform
   consumes the resolved geometry and never re-derives a margin. Generic item
   bounds remain optional because clip commands alter the stream without
   owning pixels. Total shadow reach and optional heterogeneous item reach are
   therefore related projections, not one falsely optional concept.
5. **Naming and visibility.** The already-established simple function name and
   crate-visible crossing are retained. No declaration, public/parent
   re-export, alias, call-site spelling, module housing, feature surface, or
   public API changes, so the canonical naming laws require no unrelated
   cleanup in this cell.
6. **Behavior and economics.** Blur/spread clamping, offset order, fringe,
   snapping, group union, popup surface area, panel offset, native physical
   origin, material-region translation, and hit exclusion remain unchanged.
   The direct path avoids one enum construction, match, optional branch, and
   panic edge. Allocation, scene order, renderer topology, batching/pass
   fusion, invalidation, and presentation clocks are unchanged.
7. **Proof.** The paint owner witness now proves direct shadow reach equals the
   shadow-only group bound. Both popup-projection witnesses passed across four
   scales, including native material resolution that strips the painted shadow
   only after retaining its authored envelope. The full library discovered
   1,138 tests and passed 1,128 with ten standing ignores. All targets and all
   five examples compiled without warnings; all ten census parser tests, the
   full census, formatting, diff hygiene, and protected
   `comparison_open: true` check passed.
8. **Gauge and next frontier.** Production expects fall 45 -> 44. Every other
   gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   production `pub(crate)` 1,825 in 192 files, cross-slot upper bound 1,778,
   cross-slot test edges 90, source-root mentions 118, filesystem reads 363,
   allowances 6, and panics 5. Shadow visual reach is at fixed point; the
   reverse sweep continues through remaining layout/session/runtime
   visibility, failure, intermediate, housing, and naming inventories. This
   cell does not close Rung 5.

### R5-63 — canonical placement support namespace

Status: **complete; flattened compound aliases removed**. Correction
`58a487c6` (`Namespace placement support types`).

1. **Question and complete trace.** The rung-closing naming sweep followed the
   placement vocabulary from its geometry declarations through layout panel
   attachment and frame retention, overlay drafts/live/retiring entries, view
   node construction, context menus, hover feedback, native work-area
   projection, Windows popup positioning, integration witnesses, and the
   context-menu architecture contract.
2. **Naming contradiction.** The owner declared simple support types
   `placement::Anchor` and `placement::Request`, then the geometry parent hid
   their namespace and changed both spellings with
   `Anchor as PlacementAnchor` and `Request as PlacementRequest`. Every
   downstream layer consequently used compound names to recover context that
   the module already supplied. The two aliases were not compatibility API;
   they were crate-visible private convention and violated one canonical
   spelling through every projection.
3. **Correction and displaced surface.** `geometry::placement` is now
   crate-visible and the parent re-exports neither support type. All production
   and test callsites use `geometry::placement::Anchor` and
   `geometry::placement::Request`; declaration and consumption therefore share
   one spelling at every depth. The compound parent aliases and every callsite
   spelling are deleted without replacement aliases.
4. **Namesake and visibility ruling.** `placement` owns no central `Placement`
   type, so there is no central-type parent projection to preserve. `Anchor`
   and `Request` are peer support declarations and remain qualified by their
   module. Changing `mod placement` plus a crate-visible re-export into one
   `pub(crate) mod placement` does not widen the effective crate boundary; it
   removes the alias projection and exposes exactly the namespace already
   consumed across the crate. No public API or external path changes.
5. **Ownership and behavior.** Geometry still owns anchor species, desired
   size, clearance, candidate ordering, fit, intersection, and clamping.
   Layout, overlay, view, and platform retain their prior responsibilities and
   values; only their Rust paths change. Panel placement, native/in-frame
   realization, pointer clearance, visual bounds, hit testing, allocation,
   invalidation, paint order, renderer topology, batching/pass fusion, and
   presentation clocks are unchanged.
6. **Ratchet and proof.** The context-menu architecture witness now requires
   the namespaced request path, the crate-visible placement module, and
   structural absence of both compound aliases. All five placement owner tests
   and the focused architecture witness passed. The full library discovered
   1,138 tests and passed 1,128 with ten standing ignores. All targets and all
   five examples compiled without warnings; all ten census parser tests, the
   full census, formatting, diff hygiene, and protected
   `comparison_open: true` check passed.
7. **Gauge and next frontier.** Every gauge remains unchanged: production/test
   edges 325/109, split responsibilities 3, slot edges 54, forbidden/external/
   SCC counts 0/0/0, production `pub(crate)` 1,825 in 192 files, cross-slot
   upper bound 1,778, cross-slot test edges 90, source-root mentions 118,
   filesystem reads 363, allowances 6, panics 5, and expects 44. Placement
   naming is at fixed point. The rung-closing alias inventory continues through
   remaining touched UI parent projections before Rung 5 can close.

### R5-64 — central layout chrome without a one-variant support surface

Status: **complete; redundant species and flattened support projections
removed**. Correction `4ce4c644` (`Collapse scrollbar chrome support`).

1. **Question and complete trace.** The touched-parent naming sweep traced
   layout chrome production from viewport/frame eligibility through vertical
   and horizontal geometry, owner clips, hit testing, drag offsets, runtime
   activity/fade state, scene late-chrome projection, theme thickness, table
   gutters, nested text areas, palettes, and every integration-test inspection
   of projected scrollbar geometry.
2. **Compensating surface.** Private `chrome` declared central `Chrome`, a
   single-variant `Kind::Scrollbar`, and `Scrollbar`. The layout parent
   re-exported `Chrome`, renamed `Kind as ChromeKind`, and flattened
   `Scrollbar`; runtime and scene then matched the only possible variant to
   recover scrollbar facts. The compound alias and flattened support type were
   symptoms of a redundant species wrapper, not evidence that the private
   layout-inspection module should become a crate namespace.
3. **Challenge and privacy ruling.** A first namespace-only sketch made
   `layout::chrome` crate-visible. The full suite correctly rejected it through
   the standing layout-inspection privacy witness. The live consumers already
   need only resolved scroll and projected track/thumb geometry, not the
   internal support types. The campaign therefore retained the private module
   and reduced the boundary instead of weakening the witness or widening a
   module to satisfy naming mechanically.
4. **Correction and deletion.** `Chrome` now contains its private scrollbar
   geometry directly and projects resolved scroll, hit/drag behavior, and
   thickness-adjusted track/thumb rectangles through central-type methods.
   Runtime and scene consume those total projections without matching a
   species. `Kind`, `Chrome::kind`, crate-visible `Axis`/`Scrollbar`,
   `ChromeKind`, the flattened `Scrollbar` export, every external match, and
   the associated support methods are deleted. Private `Scrollbar` remains a
   bounded construction/geometry value inside its owner.
5. **Naming and visibility.** The namesake parent now re-exports only
   `chrome::Chrome`; supporting declarations do not leave the private module.
   No alias, alternate spelling, or public API remains. Effective visibility
   narrows: production `pub(crate)` declarations fall by four, and the
   existing ban on crate-visible layout inspection submodules stays intact.
6. **Behavior and economics.** Chrome creation order, vertical/horizontal
   eligibility, clip scope, targets, scroll offsets, fade activity, theme
   policy, track/thumb resizing, hit precedence, late paint order, allocation,
   layout invalidation, renderer topology, batching/pass fusion, and
   presentation clocks are unchanged. Runtime and paint each avoid one
   exhaustive match and no longer copy a support value across the owner
   boundary.
7. **Ratchet and proof.** A new architecture witness pins a private chrome
   module, the sole central parent projection, extinction of `Kind` and
   `ChromeKind`, and retention of private scrollbar construction. The standing
   layout-inspection privacy witness and all twelve focused scrollbar tests
   passed. The full library discovered 1,139 tests and passed 1,129 with ten
   standing ignores. All targets and all five examples compiled without
   warnings; all ten census parser tests, the full census, formatting, diff
   hygiene, and protected `comparison_open: true` check passed.
8. **Gauge and next frontier.** Production `pub(crate)` declarations fall
   1,825 -> 1,821 and the cross-slot-provider upper bound falls 1,778 -> 1,774.
   Every other gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   192 visibility-bearing files, cross-slot test edges 90, source-root mentions
   118, filesystem reads 363, allowances 6, panics 5, and expects 44. Layout
   chrome is at fixed point; the touched-parent naming sweep continues through
   table tracks, layout text, scene visuals, and command-palette support before
   Rung 5 closes.

### R5-65 — namespaced layout table-track species

Status: **complete; compound parent aliases removed and projector narrowed**.
Correction `1a703f29` (`Namespace layout table tracks`).

1. **Question and complete trace.** The touched-parent sweep traced table-track
   construction from resolved header/row frames through column boundaries,
   resize hit zones/actions, clipping, floating-layer ownership, table-node
   identity, scene grid-rule projection, shared clips, compact/expanded tables,
   horizontal scroll, pinned rows, resize/rebuild, and every track-inspection
   witness.
2. **Naming contradiction.** Private `layout::table` declared meaningful
   `Axis::{Column, Row}` and `Track`, while the layout parent changed their
   spellings to `TableTrackAxis` and `TableTrack`. Scene paint and tests used
   those compounds to reconstruct context already supplied by the module.
   Unlike chrome, the axis is a real two-species distinction and the track is
   a coherent crossing value; deleting either would replace a structural
   distinction with booleans or duplicate paint policy.
3. **Correction and visibility guard.** `layout::table` is now crate-visible;
   its consumers use `layout::table::Axis` and `layout::table::Track`, and the
   parent aliases are deleted. The projector function, which has only the
   layout parent as a consumer, narrows from `pub(crate)` to `pub(super)` so
   namespacing the real crossing types does not expose construction. The
   existing `Projection` visibility is deliberately retained because
   crate-visible `Frame::table_projection` and integration witnesses consume
   its inferred type and `content_width`; Rung 6 owns that explicit test/
   visibility disposition rather than this naming cell inventing adapters.
4. **Ownership and non-merge ruling.** Layout table owns resolved column/row
   track species, boundaries, rule geometry, clips, resize identity, and hit
   zones. Scene paint consumes the structural axis to select an ordinary rule
   orientation; table widget/provider state remains a separate semantic owner.
   No scene type moves into layout, no table provider learns paint, and no
   callback or facade module conceals the crossing.
5. **Naming law.** Declaration and callsite spellings are now identical;
   support is qualified by its owning module and no alternate parent spelling
   survives. Because the module has no namesake central `Table` type, it exports
   no central type at the parent. No public API or external application path
   changes.
6. **Behavior and economics.** Track construction/order, column/row identity,
   resize precedence, clipping, rule rects and thickness, layer ownership,
   table scroll, allocation, layout invalidation, paint order, renderer
   topology, batching/pass fusion, and presentation clocks are unchanged. The
   change is path-only plus one visibility narrowing.
7. **Ratchet and proof.** The existing track-species witness now also pins the
   namespaced module, extinction of both compounds, and the parent-only
   projector. All targets compiled without warnings. The full library
   discovered 1,139 tests and passed 1,129 with ten standing ignores. All five
   examples, all ten census parser tests, the full census, formatting, diff
   hygiene, and protected `comparison_open: true` check passed.
8. **Gauge and next frontier.** Production `pub(crate)` declarations fall
   1,821 -> 1,820 and the cross-slot-provider upper bound falls 1,774 -> 1,773.
   Every other gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   192 visibility-bearing files, cross-slot test edges 90, source-root mentions
   118, filesystem reads 363, allowances 6, panics 5, and expects 44. Table
   track naming is at fixed point; the sweep continues through layout text,
   scene visuals, and command-palette support before Rung 5 closes.

### R5-66 — central layout text projection without flattened area support

Status: **complete; flattened support projection removed without widening the
private layout-text seam**. Correction `ff1604a2` (`Project layout text through
its central frame`).

1. **Question and complete trace.** The touched-parent naming sweep traced
   layout text-area construction from the view model through the layout engine,
   cached `text::Area` storage in `Frame`, interaction and rendering accessors,
   TextArea and TextBox scene branches, render-surface projection, and every
   call site that named the support type outside its declaring module.
2. **Naming contradiction.** Private `layout::text` declared diagnostic
   `Text` and staging value `Area`, while the layout parent projected the
   latter under the compound spelling `TextArea`. The only named production
   consumer was the text-area painter; one test used the alias solely as a
   method pointer. The parent spelling added context already supplied by the
   module and exposed a private implementation value beside the namesake
   module's central type.
3. **Ownership and challenge.** `Area` is layout-owned cached text geometry.
   It is constructed and retained by `Frame`; scene paint needs the frame's
   resolved surfaces, not independent authority to name or construct that
   support value. Making `layout::text` crate-visible would widen an inspection
   seam to repair a naming violation mechanically, while moving paint policy
   into layout or adding a facade would conceal the same dependency.
4. **Correction and deletion.** The layout parent no longer re-exports
   `text::Area as TextArea`. The text-area painter now accepts the central
   `layout::Frame` and borrows its private cached area internally; callers no
   longer transport the support value across the paint boundary. The lone test
   method pointer uses type inference. No replacement alias, compatibility
   path, adapter, or callback remains.
5. **Naming and visibility.** The namesake parent projects only `text::Text`.
   The `text` module and `Area` remain private/crate-private at their existing
   owner, and no call site can spell a flattened `layout::TextArea`. A new
   architecture witness pins the central-only parent projection, alias
   extinction, and painter boundary.
6. **Behavior and economics.** Text-area construction, caching, selection and
   surface order, TextBox inactive-field behavior, clipping, paint order,
   renderer topology, batching/pass fusion, invalidation, allocation, and
   presentation clocks remain unchanged. The painter performs the same single
   optional frame lookup that its caller previously performed.
7. **Proof.** The full library discovered 1,140 tests and passed 1,130 with ten
   standing ignores. All targets and all five examples compiled without
   warnings; all ten census parser tests, the full census, formatting, diff
   hygiene, and protected `comparison_open: true` check passed.
8. **Gauge and next frontier.** Production `pub(crate)` declarations fall
   1,820 -> 1,819 and the cross-slot-provider upper bound falls 1,773 -> 1,772.
   Every other gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   192 visibility-bearing files, cross-slot test edges 90, source-root mentions
   118, filesystem reads 363, allowances 6, panics 5, and expects 44. Layout
   text naming is at fixed point; the touched-parent sweep continues through
   scene visuals and command-palette support before Rung 5 closes.

### R5-67 — scene visual support retained behind its central owner

Status: **complete; compound projections deleted and scene-only support
narrowed**. Correction `497c398f` (`Keep scene visual support behind Visuals`).

1. **Question and complete trace.** The touched-parent sweep traced target
   hover/press/active/selection facts, slider transition sampling, sanitization,
   caret visibility, scrollbar animation, runtime scheduling and cleanup,
   scene tint/transform/scrollbar projection, renderer motion admission, and
   every named `VisualScalar` and `TargetVisual` call site.
2. **Naming and boundary contradiction.** Private `scene::visual` declared
   `Visuals`, `Target`, `Scalar`, and `Scrollbar`, while the scene parent
   changed two support spellings to `TargetVisual` and `VisualScalar`. Runtime
   used those aliases only to construct values immediately handed back to
   `Visuals`; paint consumed returned support values within the scene subtree.
   The aliases therefore exposed construction details rather than a deliberate
   crossing contract.
3. **Challenge and stronger ruling.** Making `scene::visual` crate-visible and
   qualifying both support types would obey spelling law but unnecessarily
   publish the whole staging namespace. Runtime owns transition sampling and
   interaction resolution; scene-owned `Visuals` owns validation, storage, and
   paint projection. The smaller truthful contract is semantic admission on
   `Visuals`, with support species retained below it.
4. **Correction and deletion.** Runtime now admits resolved target facts and
   moving/resting slider samples through `Visuals` methods. `Visuals` constructs
   and sanitizes its private support values, exposes the one hover-or-press
   query needed by runtime, and supplies the same inferred scalar default to scene
   paint. Both aliased parent projections and both external support constructors
   are deleted; no compatibility alias, facade, callback, or duplicate state
   replaces them.
5. **Visibility and naming.** The scene parent projects only central `Visuals`.
   `Target`, `Scalar`, and `Scrollbar`, their scene-consumed queries, and their
   accessors narrow from `pub(crate)` to `pub(super)`. Declaration spellings no
   longer change at any boundary, the private module is not widened, and a new
   architecture witness pins the central projection, alias extinction, and
   subtree-only support visibility.
6. **Behavior and economics.** Target sparsity/defaulting, slider desire and
   transition endpoints, sanitization, caret blinking, scrollbar opacity and
   thickness, scene order, transforms, renderer topology, batching/pass fusion,
   redraw scheduling, allocation, cleanup, and presentation clocks remain
   unchanged. The runtime-to-scene crossing transports the same scalar facts
   without first materializing a support value at the caller.
7. **Proof.** The focused architecture and slider-animation witnesses passed.
   The full library discovered 1,141 tests and passed 1,131 with ten standing
   ignores. All targets and all five examples compiled without warnings; all
   ten census parser tests, the full census, formatting, diff hygiene, and
   protected `comparison_open: true` check passed.
8. **Gauge and next frontier.** Production `pub(crate)` declarations fall
   1,819 -> 1,803 and the cross-slot-provider upper bound falls 1,772 -> 1,756.
   Every other gauge remains unchanged: production/test edges 325/109, split
   responsibilities 3, slot edges 54, forbidden/external/SCC counts 0/0/0,
   192 visibility-bearing files, cross-slot test edges 90, source-root mentions
   118, filesystem reads 363, allowances 6, panics 5, and expects 44. Scene
   visual naming and visibility are at fixed point; command-palette support is
   the final known touched-parent projection before the full Rung 5 sweep.

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
