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
- When a public module and its central type have the same name, the parent
  publicly re-exports that type and no sibling types. Call sites import the
  module and central type together as `{module, Module}`; every supporting
  concept keeps its simple name inside the module and is spelled
  `module::Type` at the use site.
- A public re-export's name is the declaration's canonical name. If a compound
  declaration is exposed under a simpler name, collapse the declaration to
  that simpler name rather than preserving an alias. This law holds through
  parent-module re-exports as well as direct ones; callers resolve collisions
  by qualifying supporting concepts through their module, not by retaining a
  compound declaration.
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
