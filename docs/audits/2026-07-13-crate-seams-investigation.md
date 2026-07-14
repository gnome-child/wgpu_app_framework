# Crate Seams — investigation ledger

Status: investigation complete (Fable 5 census, Codex review and rulings);
campaign formulation not started. No production code changed. Method per
Shea: industry baseline first, then per-seam "does this seam make sense
*here*," then refactor around accepted seams — decoupling across seams,
packing dependent concepts within them, each crate consuming lower crates as
libraries.

## Mission

Break the single `wgpu_l3` crate into a workspace whose crate boundaries are
honest seams: low coupling across, high cohesion within, feature gates where
a seam is optional capability rather than layer. The split must preserve the
constitution — laws must not weaken at crate boundaries — and success is
measured the house way: behavior-identical migration checkpoints, deletions
of private conventions into shared vocabulary, and receipts (compile-time
measurements, dependency isolation) rather than vibes.

## Industry baseline

Primary sources were rechecked during Codex review. These are dependency
graphs, not linear layer cakes; arrows below mean only that higher facilities
consume lower contracts.

| Framework | Published seam | Lesson admitted here |
|---|---|---|
| [Linebender/Xilem workspace](https://github.com/linebender/xilem/blob/main/Cargo.toml) | Geometry/paint/text/renderer libraries sit below `masonry_core`, `masonry_winit`, `xilem_core`, and the Xilem facade. | Small reusable vocabularies and platform adapters deserve separate owners; the current workspace also cautions against pretending the whole graph is one chain. |
| [iced](https://docs.iced.rs/) | `iced_core`, renderer-agnostic [`iced_runtime`](https://docs.iced.rs/iced_runtime/), graphics/render backends, widgets, winit, facade. | Core vocabulary, runtime, renderer, widgets, and OS integration are real seams. Swappable rendering is not automatically our goal. |
| [egui](https://github.com/emilk/egui) | Platform-agnostic `egui`/`epaint`, separate winit and renderer integrations, `eframe` facade. | Keep heavy platform and renderer dependencies out of the interaction/widget library. |
| [Bevy](https://github.com/bevyengine/bevy/blob/main/Cargo.toml) | Subsystem crates (`bevy_text`, `bevy_ui`, `bevy_render`, `bevy_winit`, and many more) are composed by additive facade features. | Facade feature forwarding works, but maximal crate granularity is a churn warning, not a target. |
| [Qt](https://doc.qt.io/qt-6/qtmodules.html) | Core, GUI (windowing, events, painting, fonts/text), Widgets, plus optional add-ons. | Stable low graphical vocabulary can be useful without widgets; optional integrations should be add-ons. |
| [GTK](https://docs.gtk.org/gtk4/overview.html) | Pango text, GDK window-system abstraction, GSK render nodes/backends, GTK widgets. | Text, semantic render grammar, platform, and widgets can have distinct owners. |
| [Flutter](https://docs.flutter.dev/resources/architectural-overview) | Independent downward-dependent libraries over an engine/embedder boundary. | Optional libraries and a narrow platform embedder are stronger than platform conditionals scattered through UI code. |

The recurring seams are lower value vocabulary, text, semantic paint/scene
grammar, rendering, widgets/interaction, runtime, platform integration, and a
facade. Accessibility commonly enters through a separate adapter, as
[`accesskit_winit`](https://docs.rs/crate/accesskit_winit/latest/source/README.md)
does for winit. No baseline says every module deserves a crate.

Two deviations are honest here. Renderer interchangeability is not an
organizing goal: wgpu is a chosen engine, although renderer dependency
isolation still matters. Text *mutation* as an optional capability is finer
than most baseline crate maps; it earns consideration because this framework
already has substantial buffer, draft, history, IME, transaction, and runtime
machinery. Industry precedent alone does not prove that cut.

## Seam admission law

A module group becomes a crate only when all of these are true:

1. it has one sentence of ownership and a short list of forbidden dependencies;
2. its cross-crate API is smaller and more stable than its implementation;
3. the proposed dependency graph remains acyclic without callback smuggling or
   type-erased service locators;
4. it either has an independent consumer, isolates meaningful dependencies or
   compile work, or creates an honest optional capability boundary;
5. its tests can live with the owner or observe it through the same contract as
   production; and
6. moving it deletes coupling or a private convention. A path move by itself is
   not a receipt.

Crate dependency order is a DAG, not a global numeric layer scale. Peer crates
may share lower vocabulary and meet in one integrator without becoming
comparable or mutually dependent.

## Repository census (module graph)

Full graph gathered by exhaustive sweep: 46 top-level modules (36 pub),
296 production internal edges, test-only edges separated.

### Syntactic leaves are not yet a foundation crate

Nine modules have zero production imports from other top-level framework
modules: `animation`, `color`, `error`, `feedback`, `geometry`, `icon`,
`state`, `subject`, `task`. That is useful extraction evidence, but Fable's
initial "extractable today" ruling was too strong:

- `animation` directly returns winit `ControlFlow`; the platform projection
  must move to the winit adapter before platform-neutral animation can sit low;
- `error` is command dispatch vocabulary and belongs with the command owner;
- `icon` owns the `iconflow` pack lookup as well as icon identity, so identity
  and pack realization may need different homes; and
- `feedback`, `state`, `subject`, and `task` are not cohesive merely because
  they are leaves. Their public vocabulary may need to sit low, while stores,
  stacks, executors, and service machinery stay with UI, command, or runtime.

The first extraction is therefore a responsibility/API census, not a bulk
move of nine files.

### The hubs

Fan-in: `text` 20 (the #1 hub), `interaction` 18, `geometry`/`window`/
`session` 13, `scene`/`context`/`state`/`response` 12, `command` 11.
Fan-out (integrators): `runtime` 36, `platform` 24, `session`/`view` 20,
`shell` 19.

### The tangle — the load-bearing finding

Twenty-four mutual module pairs exist, and they cluster:

> **`{scene, view, table, interaction, session, composition, virtual_list}`
> is one strongly-connected component.** Splitting any single member into
> its own crate requires breaking back-edges first.

Additional knots outside the cluster, each with receipts in the census:

| Knot | Receipt (one per direction) | Cut strategy |
|---|---|---|
| `paint ↔ text` | paint/mod.rs:2 \| text/edit/view.rs:3 plus text/layout's many paint-coordinate imports | `paint::Scene` contains shaped text while text layout consumes paint-space rectangles. Move renderer-neutral coordinate/rect vocabulary down, keep selectable view projection with text/UI, and let the renderer own the render-ready paint scene. Do not make text depend on the wgpu crate. |
| `context ↔ layout` | context/mod.rs retains `layout::TextService`; layout/frame.rs and layout/typography.rs consume `context::Source` | This is not a frame-handle hoist. Split command invocation context from concrete text-layout service realization: the command contract owns semantic service access; runtime/UI supplies the implementation. No generic service locator. |
| `layout ↔ scene`, `overlay ↔ scene`, `scene ↔ theme/window` | census | scene is in 6 cycles — mid-stack stays together in v1 (below). |
| `render → diagnostics → session/window` | census | render must not drag session: invert so diagnostics *observes* render through stats types render owns. Also makes `diagnostics` gateable. |
| `command ↔ input/keymap/responder` | census | all three cycles are *within* the proposed command crate — intra-crate cycles are free. No action. |
| semantic scene → platform lowering → paint/render | platform/native/paint.rs owns the entire semantic-to-render projection | Lowering is renderer work, not Windows/winit policy. Move it behind the renderer contract so platform consumes a renderer rather than owning the renderer's grammar conversion. |

**The general cycle-breaking tool** remains identity/vocabulary hoisting when
the type is genuinely shared (`interaction::Id`, `popup::Surface`, selection
membership types). It is not a universal answer: concrete services such as
`layout::TextService`, transaction stores, renderer lowering, and platform
effects need dependency inversion or relocation, not a lower type bucket.
Every hoist needs two named consumers and a forbidden-dependency receipt.

### The Rust visibility and test wall

The present crate uses `pub(crate)` 1,881 times across 187 source files, plus
1,359 `pub(super)` / `pub(in crate::...)` restrictions. Rust has no
`pub(workspace)`: crossing a crate seam requires `pub`, an owned public trait,
or a relocation that keeps the call private. The [Rust visibility rules](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
make this an architectural cost, not a mechanical rename.

Checkpoint 0 must inventory the exact cross-seam subset and assign each use one
of four dispositions: make it a deliberate member-crate API, replace it with a
narrow contract, relocate the caller with the owner, or reject the seam. A
blanket `pub(crate)` → `pub` rewrite is forbidden.

Architecture tests now contain 99 `CARGO_MANIFEST_DIR` roots and 271 filesystem
read/read-dir sites. The earlier count of 98 predated One Resolved Press. In
addition, many end-to-end tests reach crate-private state. Root path repair and
test ownership are separate problems: keep the facade package at the workspace
root so one workspace-source helper can replace path literals; move white-box
tests with their owner; retain cross-layer journeys at the facade through real
contracts. Do not add a public `test-support` feature to bypass privacy.

### External dependency ownership (who carries what)

| Would-be crate | Deps it isolates |
|---|---|
| text | cosmic-text, unicode-{bidi,linebreak,segmentation}, lru, glyphon text types |
| render | wgpu, bytemuck, glyphon atlas/renderer |
| platform | winit, windows, windows-sys, windows-numerics, windows-future, wgpu-hal, rfd |
| runtime task execution | pollster |
| clipboard | arboard (sole owner — clean extraction) |
| theme | serde + toml (sole owner — clean extraction) |
| icon realization | iconflow (sole owner today; identity need not carry the pack dependency) |
| command/platform errors | thiserror in their respective owners |

`env_logger` is examples-only (not a library dep at all — delete from
`[dependencies]` during the split). `pollster` is used by task execution as
well as platform startup and should follow execution rather than being assumed
platform-only. `windows` proper is platform-only; `windows-sys` also appears
in `document` (file system) and `pointer` (double-click metrics) — move those
OS projections behind platform/runtime contracts before claiming that the
lower graph is platform-free.

## Candidate workspace DAG (per-seam rulings)

Eight product crates including the facade remain a reasonable ceiling, but
this is a branching DAG, not an extraction order. Names are descriptive slots,
not approved package names.

| Slot | Depends on | Ownership and ruling |
|---|---|---|
| **foundation** | none | **Accepted after surgery.** Geometry/color, platform-neutral time/animation, and only the stable identities/value vocabulary proven by two higher owners. Split winit `ControlFlow` projection, icon identity from pack lookup, public task/state vocabulary from executor/store machinery, and command errors from generic values. No leaf sweep and no `common`/`types` bucket. |
| **text** | foundation | **Accepted.** Text content, Unicode, shaping, measurement, layout, overflow, caret/selection geometry. Read-only selection stays in the always-present contract. Text mutation becomes an additive capability only after the mixed `text::edit` module is decomposed. Text must not depend on wgpu, winit, UI layout, or renderer paint types. |
| **command** | foundation | **Accepted conditionally.** Typed commands, specs/registration/population, key grammar, responders/routes, responses, notification and timeline semantics. It earns a standalone crate only after concrete clipboard/task/text-layout services leave `Context` behind narrow semantic service contracts and interaction identity is hoisted. No generic service locator. |
| **ui** | foundation, text, command | **Accepted as one crate in v1.** Keep the seven-module SCC plus view/widget/layout/semantic scene, selection, popup/overlay, theme, table and virtualization together. Subdivide only after a second consumer or a materially smaller contract appears. |
| **renderer** | foundation, text, ui | **Accepted.** Own renderer-ready paint grammar, semantic-scene lowering, wgpu/glyphon preparation, batching and surfaces. First move coordinate vocabulary needed by text down; move lowering out of `platform::native::paint`; invert diagnostics observation. UI and text never import this crate. |
| **runtime** | foundation, text, command, ui | **Accepted.** Store/task execution, runtime transactions/services, shell/host, document workflow and configured clipboard. It produces semantic presentations and requests; it does not own OS windows or GPU resources. Renderer and runtime are peers that meet above, not a cycle. |
| **platform** | foundation, ui, renderer, runtime | **Accepted as the top integration crate.** Generic backend orchestration plus winit/native adapters, native popup realization, OS dialogs, system clipboard/IME bridges and physical cursor host. Optional native facilities live here; lower crates contain no winit/windows types. |
| **wgpu_l3 facade** | all enabled members | **Accepted.** Preserve established application paths by re-export and forward additive features. Keep the root package as facade/workspace root so examples, doctrine, ledgers and source witnesses retain one anchor. |

The graph intentionally has peers: renderer does not depend on runtime, and
runtime does not depend on renderer. Platform consumes both. This is the
strongest available expression of presentation preparation versus physical
realization.

Member crates begin `publish = false`. Rust still requires their cross-crate
surface to be `pub`, so it remains an explicit internal API budget; the flag
only prevents accidental registry promises. Publishing the facade later
requires a separate ruling: either publish the members with supported APIs or
keep distribution path/git-based. Do not quietly turn internal crates into
public ecosystem commitments during the move.

## Feature-seam audit

[Cargo features are additive and unified](https://doc.rust-lang.org/cargo/reference/features.html).
Facade features therefore forward into every participating member; a feature
cannot mean "disable" and cannot assume that a same-named dependency feature
stays local.

| Capability seam | Scope | Ruling |
|---|---|---|
| text mutation/editing | text selection core + UI controls + runtime services/transactions + platform IME | **Accepted after decomposition.** Always keep display, hit testing, caret geometry and read-only selection. Gate insertion/deletion, editable controls, draft/history mutation, commit grammar and IME mutation. This is a facade capability fan-out, not merely `text::edit` behind cfg. Exact feature name follows the established naming pass. |
| native platform runner | platform/winit and OS event-loop adapter | **Accepted as a distinct question.** Feature-off leaves embeddable runtime/renderer APIs and custom/headless backends. It isolates winit and base OS integration. |
| native popup/material realization | platform native popup/DComp/DWM machinery; in-frame overlays always remain | **Accepted conditionally.** This is narrower than the native runner. First separate ordinary window/surface code from optional popup realization. Do not use the ambiguous `native-surfaces` name until these two scopes are ruled. |
| system clipboard | arboard adapter + command registrations/effects | **Accepted.** Keep framework clipboard payload/contract available if useful; gate the system adapter. No system adapter means Cut/Copy/Paste registrations that require it are honestly absent from derived menus. Read-only selection remains independent. |
| system dialogs | rfd adapter + corresponding effects/registrations | **Accepted.** Gate the adapter; absent capability is absent registration or an explicit unsupported effect, not a runtime panic. |
| theme files | serde/toml parsing only | **Accepted.** Built-in and programmatic themes remain; only file serialization/parsing and its dependencies disappear. |
| accessibility adapter | future semantic projection + platform adapter | **Reserved.** Keep semantics in UI and platform exposure in a separate optional integration, following AccessKit's adapter model. |
| diagnostics | instrumentation spans UI/runtime/render/platform but isolates no external dependency | **Parked pending measurement.** First invert render observation and measure code/compile/runtime cost. Prefer stable no-op sinks over cfg spread if savings are negligible. |
| tables/virtualization | deep within the UI SCC | **Parked.** No dependency isolation and high cfg spread today. Revisit after identity/service cuts, not during the initial split. |
| document workflow | document commands, file I/O and runtime effects | **Watch, not admitted.** It has one strong caller but little independent dependency cost after dialogs are gated. A second consumer or a clearly smaller runtime API must justify it. |
| icon pack realization | iconflow phosphor lookup | **Watch.** Sole dependency ownership is clean, but standard UI currently assumes the pack. Admit only with a witnessed glyph-less or alternate-pack path. |
| command palette | command/UI/session integration | **Rejected for v1.** Small, deeply integrated, and no dependency isolation. |

## Anti-seams — laws that cross any cut (become contracts)

1. **The one panel path** crosses ui (overlay store, placement) → platform
   (realization, generations, exposure). Contract: popup identity/geometry
   vocabulary (`popup::Surface/Generation/Realization`, placement types)
   lives with UI or admitted foundation vocabulary; platform consumes it and
   UI never imports platform. Runtime must never know `windows`.
2. **The presentation clock** crosses runtime/renderer/platform. Candidate,
   submitted, acquired, presented and committed remain distinct receipts;
   renderer and platform report physical outcomes through runtime-owned
   semantic contracts. Moving a type cannot collapse the clocks.
3. **The stacking contexts** (the locked, un-ignited campaign): `stack::Key`
   is foundation-crate material; each context's `Stratum` lives with its
   context. The split should land *after* that campaign so the vocabulary
   is born in the right home. Sequencing below.
4. **One Resolved Press** stays runtime interaction law. UI contributes
   semantic hit/target data; runtime resolves admission and execution;
   platform receives only the resolved cursor projection. A crate cut must not
   recreate cursor policy in UI or platform.
5. **Architecture witnesses**: 99 `CARGO_MANIFEST_DIR` roots and 271
   filesystem read sites assume one `src/`. Checkpoint 0 ships one
   workspace-aware source helper while the facade remains the root package;
   white-box tests move with owners and end-to-end tests stay at the facade.
6. **Doctrine placement**: master_design and the ledgers stay
   workspace-root; each crate gets a doc-comment charter (one paragraph:
   what it owns, what it must never depend on). The compression discipline
   applies per crate: every `pub` in a member crate is real public API —
   the split forcibly cashes the public-compression audit.

## Migration shape (for the eventual campaign(s))

- **Phase 0 — pins and budgets**: full baseline ritual; clean and incremental
  compile timings; dependency/build-artifact sizes; one workspace-source
  witness helper; cross-seam `pub(crate)` and white-box-test disposition
  tables; proposed crate DAG and forbidden-edge architecture witness. No
  workspace split yet.
- **Phase 1 — prove seams inside the monolith**: remove winit from animation;
  separate renderer-neutral coordinates from paint; decompose selectable text
  truth from mutation; invert command context services; hoist only admitted
  identities; move scene lowering toward renderer ownership; invert diagnostics
  observation. Every cut is behavior-identical and has an absence witness.
- **Phase 2 — workspace shell and foundation**: keep the root package as the
  facade, add workspace members `publish = false`, extract the proven
  foundation surface, and keep all application paths re-exported.
- **Phase 3 — text and command**: extract independently proven peers. All
  existing capability remains unconditional/default-on; no feature behavior
  changes are mixed with moves.
- **Phase 4 — UI and renderer**: move the retained SCC as one UI crate, then
  extract renderer-ready paint/lowering/wgpu behind its narrow input/output
  contracts.
- **Phase 5 — runtime and platform**: extract semantic runtime, then the top
  platform integration. Lower crates must compile without winit/windows in
  their dependency trees.
- **Phase 6 — introduce feature seams one at a time**: after the default build
  is behavior-identical, add text mutation, native runner, native popup
  realization, system clipboard, dialogs, and theme-file gates in separate
  checkpoints with explicit absent-capability behavior.
- **Phase 7 — closure**: facade/docs/examples settle, compile-time receipts,
  feature matrix, API/visibility audit, source-witness audit, and SPDX headers
  per member (retiring roadmap item 20's parked sweep).

Each phase is independently revertible, ends green, and records deleted
coupling. Extraction and optionality never land in the same checkpoint.

## Risks

- **Over-splitting** (Bevy churn) for a one-person project — the v1 crate
  count above is eight including facade; resist finer cuts until a consumer
  exists (the same admission rule as everything else here).
- **Workspace-public API expansion** — 1,881 current `pub(crate)` sites make
  this the largest structural risk. Reject a seam rather than exporting broad
  stores, frame internals, or transaction machinery merely to satisfy Cargo.
- **Test weakening** — moving a white-box test to the facade by exposing
  internals is a regression. Keep white-box tests with owners and keep
  cross-layer laws black-box or architecture-based.
- **cfg-sprawl** if `tables` (or anything inside the tangle) is gated
  before identity hoists — hence parked.
- **Feature unification** — member and facade features are additive. Every gate
  needs `cargo tree -e features`, `cargo tree --duplicates`, default,
  no-default, individual, pairwise load-bearing combinations, and all-features
  receipts.
- **Orphan rule**: app-side std-trait derives (Display/FromStr/Ord) are
  unaffected (app types, framework traits stay in one crate each). Internal
  impls need homes with one of their sides — the census found no blocking
  case, but each phase must re-check.
- **Compile-time claims must be measured**, not assumed — workspace splits
  can *worsen* clean builds if the graph doesn't actually narrow; the
  baseline measurements in Phase 0 are the receipt.
- **Naming and path churn** — package names are not decided here. A crate name
  must state its owner, not become `core`, `common`, `types`, `util`, or another
  miscellaneous bucket. Established domain type names do not change merely
  because their file moves; existing overloaded-name cleanup remains a
  separate campaign. The two native feature scopes must be distinguished
  before either receives a name.

## Naming and publication ruling before formulation

- Keep `wgpu_l3` as the application facade and preserve its established public
  paths through re-exports during migration.
- Use one consistent package family only after every crate charter is written;
  package spelling is a Shea taste decision, not an architectural premise.
- Do not introduce generic replacement nouns while moving code. Existing
  `Scene`, `Presentation`, `Surface`, `State`, and `Frame` cleanup remains
  governed by its own naming census.
- Member packages start non-publishable. Cross-crate `pub` is admitted for a
  named workspace consumer, not treated as stable facade API by default.
- Feature names describe positive capability. Never name a feature by what it
  disables, and never use one name for both the native runner and native popup
  realization.

## Sequencing recommendation

1. **One Resolved Press** is complete at `7ac61c5a`.
2. Land **Typed Stacking Contexts** (locked) — its `stack` module is born
   foundation-shaped.
3. Then ignite the split campaign(s) at a quiet tree, Phase 0 first.
   This reviewed ledger is the formulation seed.

## Completion criteria for the eventual split

- Eight-or-fewer product crates including the facade, each with a one-paragraph
  charter and forbidden-dependency list.
- Every cross-crate dependency points downward; the graph is a DAG by
  construction (cycles impossible, not just absent).
- No lower crate depends on winit/windows/wgpu except renderer wgpu and platform
  OS integration; `cargo tree` witnesses the dependency isolation.
- Feature matrix compiles: default, `--no-default-features`, each gate alone,
  load-bearing pairs, and all gates. CI-shaped script even without CI.
- Behavior identical: full ritual + witnesses green at every phase.
- Compile-time receipts recorded against Phase 0 baselines.
- SPDX/license headers landed per crate; GPL notices intact.
- All 99 current witness roots resolve through the workspace helper — zero
  direct `CARGO_MANIFEST_DIR` source construction outside it.
- No blanket visibility widening: every cross-crate `pub` has a named consumer;
  no public test-support escape hatch exists.
- Default-on extraction lands before any optional capability is introduced;
  feature commits are independently revertible.
