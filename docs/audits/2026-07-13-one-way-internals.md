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
