# Crate Seams — investigation ledger

Status: investigation complete (Fable 5); execution/refinement open to Codex.
No production code changed. Method per Shea: industry baseline first, then
per-seam "does this seam make sense *here*," then refactor around accepted
seams — decoupling across seams, packing dependent concepts within them,
each crate consuming lower crates as libraries.

## Mission

Break the single `wgpu_l3` crate into a workspace whose crate boundaries are
honest seams: low coupling across, high cohesion within, feature gates where
a seam is optional capability rather than layer. The split must preserve the
constitution — laws must not weaken at crate boundaries — and success is
measured the house way: behavior-identical migration checkpoints, deletions
of private conventions into shared vocabulary, and receipts (compile-time
measurements, dependency isolation) rather than vibes.

## Industry baseline

Verified live this session:

- **Linebender stack** (github.com/linebender/xilem): `kurbo` (geometry) →
  `peniko` (paint types) → `parley`/`fontique` (text) → `vello` (renderer,
  over wgpu) → `masonry` (retained widget tree + passes; "a toolkit for
  building UI frameworks") → `xilem_core`/`xilem` (reactive layer), with
  `winit` and `accesskit` as peer integrations. The cleanest published
  layering in the Rust UI space.
- **iced** (iced.rs): `iced_core` → `iced_runtime` (renderer-agnostic) →
  `iced_wgpu`/`iced_tiny_skia` (swappable backends) → `iced_widget` →
  `iced_winit` (shell) → `iced` facade. The "modular ecosystem split into
  reusable parts" claim is their headline architecture.

Cited from documented knowledge (verify during execution if load-bearing):

- **egui**: `emath`/`ecolor` → `epaint` → `egui` → `egui-winit`/`egui-wgpu`
  → `eframe` facade.
- **Bevy**: crate-per-subsystem (`bevy_ecs`, `bevy_render`, `bevy_text`,
  `bevy_winit`, …) unified by a facade crate whose features re-export
  members. The maximal split; also the cautionary churn tale.
- **Qt**: Core (no GUI) / Gui (windows, painting, fonts — no widgets) /
  Widgets. The classic three-layer cut.
- **GTK**: `glib`/`gio` → `pango` (text) → `gdk` (platform/windowing) →
  `gsk` (render scene) → `gtk` (widgets).
- **Flutter**: engine → rendering → widgets → material, as layered
  libraries with strict downward dependency.

**Convergent seams across all baselines**: (1) geometry/math primitives,
(2) paint/scene grammar, (3) text stack, (4) GPU/render backend,
(5) platform/windowing shell, (6) widget layer, (7) app/reactive runtime,
(8) facade with feature gates, (9) accessibility as a separate integration.

**Where this framework legitimately deviates**: renderer-agnosticism (iced's
organizing goal) is a non-goal — wgpu is in the name; the render seam is
justified purely as dependency isolation. And the proposed text-*editing*
seam is finer than any baseline (parley/pango split at shaping, not
editing); it is justified here by machinery weight, not precedent.

## Repository census (module graph)

Full graph gathered by exhaustive sweep: 46 top-level modules (36 pub),
296 production internal edges, test-only edges separated.

### The bottom layer (extractable today)

Nine modules have zero internal dependencies: `animation`, `color`,
`error`, `feedback`, `geometry`, `icon`, `state`, `subject`, `task`.

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
| `paint ↔ text` | paint/mod.rs:2 \| text/edit/view.rs:3 | text's edit-surface projection consumes paint; either paint's text-agnostic primitives move below text, or the edit-view projection moves up. Census cell for the split campaign. |
| `context ↔ layout` | context/mod.rs:5 \| layout/frame.rs:1 | command context carries a layout frame handle — hoist the needed type down (geometry/interaction) to free the command stack. |
| `layout ↔ scene`, `overlay ↔ scene`, `scene ↔ theme/window` | census | scene is in 6 cycles — mid-stack stays together in v1 (below). |
| `render → diagnostics → session/window` | census | render must not drag session: invert so diagnostics *observes* render through stats types render owns. Also makes `diagnostics` gateable. |
| `command ↔ input/keymap/responder` | census | all three cycles are *within* the proposed command crate — intra-crate cycles are free. No action. |

**The general cycle-breaking tool**: most back-edges are *identity/vocabulary
sharing* (`interaction::Id`, `table::Cell`, `popup::Surface`, selection
membership types) — the classic fix is hoisting shared identity types into a
low vocabulary crate, which converts mutual edges into two downward edges.
This should be the split campaigns' recurring first move, receipts per hoist.

### External dependency ownership (who carries what)

| Would-be crate | Deps it isolates |
|---|---|
| text | cosmic-text, unicode-{bidi,linebreak,segmentation}, lru, glyphon(types), iconflow |
| render | wgpu, bytemuck, glyphon(atlas), (wgpu-hal is platform's) |
| platform | winit, windows, windows-sys, windows-numerics, windows-future, wgpu-hal, rfd, pollster |
| clipboard | arboard (sole owner — clean extraction) |
| theme | serde + toml (sole owner — clean extraction) |
| foundation | thiserror, log |

`env_logger` is examples-only (not a library dep at all — delete from
`[dependencies]` during the split). `windows` proper is platform-only;
`windows-sys` also appears in `document` (file system) and `pointer`
(double-click metrics) — two small relocations or cfg-scoped remnants.

## Proposed workspace (per-seam verdicts)

Ordered bottom-up. Names are placeholders — crate naming is a taste flag.

| Crate | Contents | Verdict & rationale |
|---|---|---|
| **foundation** | the nine leaves: geometry, color, animation, error, state, task, icon, feedback, subject | **Yes, first.** Extractable today with zero edge surgery. Matches kurbo/emath precedent. `state` (the Store) is framework-specific but a pure leaf — it rides here unless taste wants it alone. |
| **text** | text (shaping, layout, unicode, measurement, overflow projection) | **Yes.** Fan-in #1 must sit low; isolates the heaviest pure-Rust deps. Requires the `paint ↔ text` cut first. **Feature `editing`** gates `text::edit` + `draft` (lexical admission, drafts, edit surface) per Shea's seam — display/shaping always on. |
| **command** | command, responder, context, response, target, timeline, notification, keymap, input | **Yes.** The framework's most original layer, and its three internal cycles are intra-crate. Requires the `context ↔ layout` hoist. This is the crate other Rust apps could plausibly consume standalone — the typed command system with claims, chains, menus-from-meaning. |
| **ui** (retained core) | the tangle cluster: scene, view, widget, layout, composition, interaction, session, table, virtual_list, selection, draft(non-text parts), popup, overlay, theme | **Yes, as ONE crate in v1.** The SCC is real; splitting view/widget from scene/layout now would fight 24 cycles for no consumer. Record the identity-hoist strategy as the path to later subdivision *when a consumer exists*. Theme rides here until the scene↔theme knot is hoisted, then may extract with its serde/toml gate. **Feature `tables`** parked: gating table+virtual_list today means cfg-sprawl through session/interaction/scene (fan-in 9); feasible only after identity hoists. Honest verdict: not yet. |
| **render** | render, paint | **Yes.** Isolates wgpu/bytemuck/glyphon-atlas. Requires the diagnostics-edge inversion. |
| **runtime** | runtime, shell, host, ime, document, clipboard(?) | **Yes.** The integrator (fan-out 36). `document` belongs here, not in text — it's file-backed application machinery (and carries windows-sys file IO). `clipboard` either here or its own micro-crate; arboard's sole ownership makes either clean. |
| **platform** | platform (winit runner, native windows, popups, DComp, IME bridge), pointer | **Yes.** The OS sink. **Feature `native-surfaces`** gates the native popup/composition/DWM machinery (windows, windows-numerics, windows-future, wgpu-hal) with in-frame fallback always compiled — per Shea's seam, and the single biggest compile-time isolation in the graph. **Feature `dialogs`** gates rfd. |
| **wgpu_l3** (facade) | re-exports; examples unchanged until the end | **Yes.** The Bevy/iced/eframe pattern; features forward to member gates. |

**Diagnostics** becomes a feature (`diagnostics`) somewhere sensible after
the render-edge inversion — it is dev tooling with mid-stack reach today.

## Feature-gate audit (beyond Shea's two)

| Gate | Mechanism | Verdict |
|---|---|---|
| `native-surfaces` (Shea) | platform crate feature; in-frame path always present | **Yes** — biggest dep isolation; Wayland already proves the in-frame fallback is a first-class citizen. |
| `text-editing` (Shea) | text crate feature over edit/draft | **Yes** — display-only builds for viewers/dashboards. |
| `clipboard` | arboard behind gate; Cut/Copy/Paste registrations absent when off | **Yes — and note the synergy**: meaning-derived menus make gated commands *honestly absent* from every surface (absent registration = absent menu item, by law). Feature gates and the command system compose perfectly here. |
| `dialogs` | rfd behind gate; Open/Save effects degrade to typed errors | **Yes**, same synergy. |
| `theme-files` | serde+toml behind gate; built-in theme compiled in | **Yes** — two deps for a file-loading convenience. |
| `diagnostics` | dev instrumentation | **Yes**, after the render-edge cut. |
| `tables` | table+virtual_list | **Parked** — requires identity hoists first (see tangle). Record as the post-split candidate. |
| `palette` | command palette UI | **No for now** — small, deeply integrated with command core, no dep isolation. Watch. |
| `accessibility` | future AccessKit adapter crate | **Reserve the seam** — every baseline ships a11y as a separate integration crate; roadmap item 11 already reserves the semantic hooks. |

## Anti-seams — laws that cross any cut (become contracts)

1. **The one panel path** crosses ui (overlay store, placement) → platform
   (realization, generations, exposure). Contract: popup identity/geometry
   vocabulary (`popup::Surface/Generation/Realization`, placement types)
   lives low (ui or foundation); **platform depends upward on ui types,
   never the reverse** — the runtime must never know `windows`.
2. **The presentation clock** crosses host/shell/runtime/render. Receipts
   (acquire/present/commit) are vocabulary owned below render; render and
   platform report into it.
3. **The stacking contexts** (the locked, un-ignited campaign): `stack::Key`
   is foundation-crate material; each context's `Stratum` lives with its
   context. The split should land *after* that campaign so the vocabulary
   is born in the right home. Sequencing below.
4. **Architecture witnesses**: 98 `CARGO_MANIFEST_DIR` source-reads in
   architecture.rs assume one `src/`. The split's checkpoint 0 must ship a
   workspace-aware source-reading helper and relocate witnesses to a
   workspace-level test crate — otherwise 98 silent breakages. This is the
   largest single migration cost found.
5. **Doctrine placement**: master_design and the ledgers stay
   workspace-root; each crate gets a doc-comment charter (one paragraph:
   what it owns, what it must never depend on). The compression discipline
   applies per crate: every `pub` in a member crate is real public API —
   the split forcibly cashes the public-compression audit.

## Migration shape (for the eventual campaign(s))

- **Phase 0 — scaffold + pins**: workspace manifest; facade crate = current
  crate unchanged; witness-path helper; compile-time baseline measurements
  (clean + incremental, per feature combination) so gains are receipts;
  full ritual green.
- **Phase 1 — foundation**: move the nine leaves. Behavior-identical.
- **Phase 2 — text**: paint↔text cut, then extract with `editing` feature.
- **Phase 3 — command**: context↔layout hoist, then extract.
- **Phase 4 — render**: diagnostics inversion, then extract.
- **Phase 5 — platform**: extract with `native-surfaces`/`dialogs` gates.
- **Phase 6 — runtime/ui settle**: what remains splits into ui + runtime +
  facade; SPDX headers land per-crate **during** the split (kills roadmap
  item 20's parked sweep naturally — new crate roots need headers anyway).
- Each phase: one campaign checkpoint, ledger-first, ritual green,
  behavior-identical, deletions recorded (private conventions absorbed).

## Risks

- **Over-splitting** (Bevy churn) for a one-person project — the v1 crate
  count above is eight including facade; resist finer cuts until a consumer
  exists (the same admission rule as everything else here).
- **cfg-sprawl** if `tables` (or anything inside the tangle) is gated
  before identity hoists — hence parked.
- **Orphan rule**: app-side std-trait derives (Display/FromStr/Ord) are
  unaffected (app types, framework traits stay in one crate each). Internal
  impls need homes with one of their sides — the census found no blocking
  case, but each phase must re-check.
- **Compile-time claims must be measured**, not assumed — workspace splits
  can *worsen* clean builds if the graph doesn't actually narrow; the
  baseline measurements in Phase 0 are the receipt.
- **Naming** (taste): member crate names (`wgpu-l3-text` vs `l3_text` vs
  short names) — flag for Shea before Phase 0.

## Sequencing recommendation

1. Land **One Resolved Press** (in flight, item 34).
2. Land **Typed Stacking Contexts** (locked) — its `stack` module is born
   foundation-shaped.
3. Then ignite the split campaign(s) at a quiet tree, Phase 0 first.
   The investigation is read-only and this ledger is the census seed;
   Codex may extend/refute per the bake-off pattern before formulation.

## Completion criteria for the eventual split

- Eight-or-fewer member crates, each with a one-paragraph charter.
- Every cross-crate dependency points downward; the graph is a DAG by
  construction (cycles impossible, not just absent).
- Feature matrix compiles: default, `--no-default-features`, each gate
  alone, all gates. CI-shaped script even without CI.
- Behavior identical: full ritual + witnesses green at every phase.
- Compile-time receipts recorded against Phase 0 baselines.
- SPDX/license headers landed per crate; GPL notices intact.
- The 98 witness paths resolved through the workspace helper — zero
  path-literal source reads outside it.
