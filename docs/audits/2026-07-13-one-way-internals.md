# One-Way Internals — Seams Before Crates (umbrella ledger)

Status: arc open. Campaign destination from the
[crate-seams investigation](2026-07-13-crate-seams-investigation.md);
method is the ten-step loop; this document is the operating ledger for the
whole arc — gauge, wave map, virtual-crate table, forbidden-edge allowlist,
cell queue, and cell records. No file moves into crates during this arc.

## Destination and exit theorem

Make the monolith behave as if the crates already existed. The arc reaches
fixed point when:

- the virtual crate graph is a DAG (zero forbidden edges);
- text and ui do not depend on renderer or platform;
- command retains no concrete UI/runtime services;
- runtime and renderer are peers; platform is the only OS dependency sink;
- every future cross-crate API is already deliberate (named consumer,
  disposition for every crossing);
- no unresolved test or visibility workaround remains.

Then the crate split is packaging, not architecture.

## Scales

| Scale | Purpose |
|---|---|
| Virtual-crate graph | Architectural destination |
| Campaign wave | Bounded subsystem territory |
| Ledger cell | Unit of investigation |
| Loop | Method of correction |
| Ratchet | Durable evidence of progress |
| Exit theorem | Arc completion criterion |

Two repetitions: the **inner loop** works one cell to fixed point; the
**outer loop** re-censuses the graph and selects the next cell until the
exit theorem holds.

## The loop

**Select → Trace → Model → Challenge → Admit → Reduce → Rewire → Prove →
Ratchet → Re-scan.**

1. **Select** one cell: a forbidden edge, disputed owner, duplicated
   policy, visibility leak, or candidate seam. One named seam question.
2. **Trace** every relevant entrance, outcome, lifecycle transition,
   backend, and failure path within the slice — success, absence,
   rejection, cancellation, staleness, replacement, retry.
3. **Model** ownership, identity, clocks, coordinate spaces, state
   authority, fallibility, dependencies, witnesses, intended virtual crate.
4. **Challenge** intermediates, duplicated decisions, misplaced ownership,
   dependency direction — with the standing questions below.
5. **Admit** only on concrete evidence. Structural taste alone is
   insufficient. A proven forbidden dependency or cross-seam cycle is
   sufficient evidence even without a user-visible bug (the broadened
   Examen authorization). Zero-change outcomes are valid and recorded.
6. **Reduce** — remove machinery that carries no invariant.
7. **Rewire** — relocate machinery that is real but owned by the wrong
   seam. One coherent correction.
8. **Prove** — behavior, architecture, dependency-graph, renderer-topology,
   and performance witnesses proportional to the change. Behavior-preserving
   means failure, absence, ordering, and cleanup identical — not merely the
   happy path.
9. **Ratchet** — encode the result so the old structure cannot silently
   return: allowlist entry burned, forbidden-import witness, unique-owner
   assertion, tombstone, charter, narrowed visibility, relocated test,
   ledger receipt (old path, new owner, deletion).
10. **Re-scan** the slice; repeat to fixed point; return to the queue.

### Standing Challenge questions

A type survives when it encodes identity, lifecycle, units, coordinate
space, authority, or failure — when it makes invalid states
unrepresentable. If it has no invariant, no lifecycle/clock/space/identity
distinction, no multiple real producers or consumers, and removing it does
not increase coupling or visibility, it is transport scaffolding: Reduce.

Repeated code centralizes only when it is the same semantic decision with
the same inputs and failure rules — never for shared syntax. Deliberate
non-merges (roadmap item 22 and every campaign-ledger non-merge) enter this
step as already-answered case law; a cell may not re-litigate one without
new evidence.

A seam is admitted per the
[seam admission law](2026-07-13-crate-seams-investigation.md) — and
withdrawn if its contract would be larger or less coherent than the
coupling it replaces. No generic service locator; no trait whose only
purpose is concealing a dependency arrow; an edge hidden behind
`Box<dyn Fn>` is still an edge.

## Constitutional rails

- No user-visible behavior changes; no feature gating during the arc.
- No generic `core`/`common`/`types`/`util`/`manager` buckets.
- No renaming established concepts merely because they move.
- No blanket visibility widening; hoists keep `pub(crate)`.
- One ownership correction per commit where practical; land on main; no
  long-lived refactor branch.
- Every correction retires or narrows a ledger entry; every accepted
  boundary gains a witness.
- One cell fits one session, Prove step closed — a cell that cannot close
  in-session is two cells.
- Cells yield to in-flight behavioral campaigns: a cell whose slice
  intersects a hot campaign re-queues.
- Success metrics: forbidden edges, ambiguous ownership, unjustified
  visibility, services crossing semantic seams, direct witness path-reads.
  Non-goals: total modules, imports, types, intra-slot cycles, line count.

## Gauge

Updated per cell. Initial values from the census; Wave 0 regenerates
mechanically.

| Metric | Now | Target |
|---|---|---|
| Forbidden virtual-crate edges (allowlist size) | 13 seeded (see below; Wave 0 recount is authoritative) | 0 |
| Cross-slot cycles | 4 named knots + SCC seam crossings | 0 |
| Concrete services crossing semantic seams | 3 known (context clipboard/task/text-layout) | 0 |
| Unresolved cross-seam `pub(crate)` sites | uncounted — Wave 0 counts | 0 |
| White-box tests depending on another slot's internals | uncounted — Wave 0 counts | 0 |
| Architecture-test direct path reads | 98 | 0 (one workspace helper) |
| `#[allow(...)]` without owner / expiry | uncounted — Wave 1 inventory | 0 |

## Wave map

- **Wave 0 — virtual crates and the ratchet** (infrastructure; acceptance
  below).
- **Wave 1 — purify the bottom**: animation/winit, pointer/double-click
  metrics, document/file-replacement FFI, icon identity vs pack, task and
  state vocabulary vs machinery, error back to command. Plus the
  inventories: `panic!`/`expect` (witnessed invariant or typed
  fallibility), `#[allow]` audit (the Menus dead-code allowance carried an
  expiry — check first), dead `pub(crate)`.
- **Wave 2 — text/geometry/paint ownership**: renderer-neutral coordinates
  down; text layout off paint vocabulary; read-only selection/caret
  projection separated from mutation/history; renderer keeps
  `paint::Scene`; text-editing boundary established, not gated.
- **Wave 3 — command meaning freed from concrete services**: smallest
  honest contracts for clipboard/task/text-layout access; reject the seam
  rather than manufacture an ugly abstraction. Also the command-slot
  upward edges: `input → {session, interaction}`,
  `responder → {session, interaction, table}`.
- **Wave 4 — semantic presentation vs physical realization**:
  scene-to-paint lowering out of `platform::native::paint` into renderer
  ownership; renderer and runtime become peers (already true by import
  graph — keep it true); diagnostics inversion; presentation clock
  preserved exactly.
- **Wave 5 — reassess the UI knot**: `{scene, view, table, interaction,
  session, composition, virtual_list}` stays together unless a back-edge
  crosses a seam, creates competing authority, drags heavy deps downward,
  forces exposure, or blocks a feature boundary. Zero intra-slot cycles is
  a non-goal.
- **Wave 6 — visibility and test readiness**: every crossing has a
  disposition; white-box tests have owning future crates; journeys use
  real contracts; the workspace source-reading helper replaces all 98
  path reads; no public test-support escape hatch. Then — and only then —
  the workspace split ignites as its own campaign.

Stacking sequencing: Typed Stacking Contexts flies after Wave 1's
foundation purification and becomes that seam's first real consumer
(`stack::Key` vocabulary born low, strata with their owners).

## Wave 0 seed

### Virtual crate slots

| Slot | Modules |
|---|---|
| `foundation` | geometry, color, animation†, subject, feedback†, icon†, state†, task† |
| `text` | text (editing boundary marked within) |
| `command` | command, responder, context†, response, target, timeline, notification, keymap, input, error |
| `ui` | scene, view, widget, layout, composition, interaction, session, table, virtual_list, selection, draft, popup, overlay, theme, pointer† |
| `render` | render, paint† |
| `runtime` | runtime, shell, host, ime, document†, clipboard, diagnostics |
| `platform` | platform (and every OS/FFI projection relocated by Wave 1) |
| `facade` | lib.rs re-exports; `tests/` pending Wave 6 disposition |

† split-pending per Codex's review rulings: the module's *vocabulary* and
its *machinery* may take different slots (icon identity vs iconflow pack;
task/state/feedback vocabulary vs executor/store/stacks; animation minus
its winit projection; pointer minus its windows-sys metrics; document
semantics vs file-replacement FFI; paint coordinates vs render-ready
scene; context contract vs service realizations). Each † is a queue cell.

### Allowed directions

`foundation` → nothing. `text` → foundation. `command` → foundation, text.
`ui` → foundation, text, command. `render` → foundation, text (paint).
`runtime` → foundation, text, command, ui, render-as-peer-forbidden.
`platform` → everything. `runtime` and `render` are peers: neither imports
the other (already true; the witness keeps it true). Facade re-exports all.

### Forbidden-edge allowlist (seed — Wave 0's witness recount is authoritative)

| # | Edge | Wave | Receipt |
|---|---|---|---|
| 1 | animation → winit (`ControlFlow`) | 1 (pilot) | census: animation returns winit type |
| 2 | pointer → windows_sys (double-click metrics) | 1 | pointer/mod.rs:47 |
| 3 | document → windows_sys (file replacement) | 1 | census TASK 4 |
| 4 | icon → iconflow (pack realization in identity module) | 1 | census |
| 5 | error → (command vocabulary living in foundation position) | 1 | Codex review ruling |
| 6 | text → paint | 2 | paint/mod.rs:2 \| text/edit/view.rs:3 |
| 7 | context → layout (`TextService`) | 3 | context/mod.rs:5 |
| 8 | context → clipboard, context → task (concrete services) | 3 | census edges |
| 9 | input → session | 3 | input/mod.rs:5 |
| 10 | input → interaction | 3 | census |
| 11 | responder → session / interaction / table | 3 | responder/builder.rs:4 etc. |
| 12 | render → diagnostics | 4 | census |
| 13 | layout → diagnostics | 4 | census |
| 14 | widget → document (`TextArea::from_document`) | 3/5 | census; the compression constructor needs an owner ruling |

This list is **gauge information, not a suite gate** (ruling: an
allowlist-as-failing-test would moderate exploration and reward laundering
edges past the parser through erasure — the failure mode worse than
re-tangling). The census parser runs as a report at cell and wave
boundaries; the ledger records the count. Enforcement is per-cell: each
completed cell's Ratchet step adds a narrow structural-absence tombstone
for the specific structure it burned (the established house pattern —
pinning the past, never gating the future). If an area demonstrably
re-tangles across audits, that evidence admits a narrow witness there —
enforcement obeys the same admission law as everything else.

### Wave 0 acceptance

- The census parser is productionized as a **gauge tool** (same sweep
  rules validated this session: `crate::`/`super::` resolution, grouped
  uses, `cfg(test)` separation), keyed to the slot and direction tables
  above, producing the forbidden-edge report on demand — not wired into
  the suite.
- The recount replaces the seed list above (order-of-15 expected; the
  metric is forbidden edges, not the 24 raw mutual pairs — intra-slot
  cycles are lawful).
- Cross-slot `pub(crate)` count and white-box-test count measured and
  entered in the gauge.
- Compile-time baselines recorded (clean + incremental) for later receipt.
- The queue below is confirmed against in-flight campaign territory.

## Cell record template

1. traced slice; 2. current owner graph; 3. candidate; 4. admission
evidence; 5. displaced path; 6. correction commit; 7. dependency/API
delta; 8. verification; 9. fixed-point result (including zero-change).

## Queue (initial)

1. **Pilot: `animation::Schedule → winit::ControlFlow`** — small,
   unambiguous, exercises all ten steps, burns allowlist #1, produces the
   first dependency-direction ratchet.
2. Allowlist #2–#5 (Wave 1 leaks), then the Wave 1 inventories.
3. Flag-sourced candidates: Examen unresolved R-flags; roadmap 25/26
   (Panel contract, semantic leaf openness) as ownership questions;
   pending-eyes entries where a trace is cheap while the slice is open.
4. Wave 2 knot (`text ↔ paint`) once Wave 1 closes.

Cell records append below as the arc runs.

---
