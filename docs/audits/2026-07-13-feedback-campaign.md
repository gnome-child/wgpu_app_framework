# Feedback campaign

Status: complete (2026-07-13). `comparison_open: true`. No push during the
campaign.

## Constitution

- `command::Spec::description` is stable meaning; `command::State::hint` is a
  contextual claim-time explanation. Neither populates the other.
- Feedback is a retained runtime fact: severity plus eagerly formatted text.
  Its typed store owns anchor identity and lifetime.
- The element-level auxiliary-content resolver consults independent owners in
  this order: Error, Warning, Info, hint, description, confirmed overflow.
- Retained severity and auxiliary chrome are separate axes. Overflow
  revelation uses the shared panel recipe with plain, glyphless chrome rather
  than impersonating informational feedback.
- A tooltip is a hover policy, not stored semantic data.
- Every panel uses the established placement request, host realization,
  generation receipt, and exposure path. Species are policies, not paths.
- Content measures completely before the shared placement resolver asks where
  it fits.

## Preflight census

| Cell | Receipt | Verdict |
|---|---|---|
| Command contextual text | `src/command/state.rs`; three registry error arms and `runtime/dispatch.rs` produce `tooltip` text | Proven truth with no presenter; rename to hint. |
| Command defaults | `State::with_command` supplies label and shortcut from `AnyCommand`; `Spec` has no description | Description is a separate default source and must not be copied into hint. |
| Table rejection | `interaction::Tables` retains `Cell -> String`; focus departure checks only existence | Identity and text already exist; presentation and lifetime witnesses are missing. |
| Rejection clearing | cancel, successful commit, input dispatch, and provider removal have separate clearing sites | Consolidate under rejection-at-most-draft law and cover every exit. |
| Overflow | `OverflowProjection` retains source, visible text, and source mapping; Clip returns identity | Add explicit overflow truth at resolution; never remeasure at hover. |
| Panel measurement | `root_floating_panel_rect` measures width, then height-for-width, then constructs `PlacementRequest` | Existing content-first path is the required owner. |
| Placement | `PlacementRequest::resolve` supplies context-menu flip/clamp and native popup placement metadata | Auxiliary panels consume it; no new fit arithmetic. |
| Panel realization | scene painting emits `overlay::Draft`; overlay/native popup code owns host selection, generations, fades, and retirement | All new panel policies enter this path. |
| Identities | table cells, interaction targets, composition nodes, and windows already have typed stable identities | No universal feedback anchor is admitted. |

## Named reductions

| ID | Reduction | Status |
|---|---|---|
| F-01 | Rejected table edit traps focus without explaining why. | Closed: rejection immediately rebuilds an anchored Error panel while retaining draft, edit session, and focus. |
| F-02 | Runtime command explanations carry truth but produce no pixels. | Closed: eligible resolved command elements project hint first, then description, after the themed dwell. |
| F-03 | Ellipsized table text cannot reveal its complete source. | Closed: confirmed overflow projects the complete source through the shared hover panel. |
| F-04 | Fully visible text must never produce an overflow tip. | Closed: `OverflowProjection::overflowed` is explicit and eligibility consumes it without remeasurement. |
| F-05 | Competing feedback, hint, description, and overflow produce one winner. | Closed: Error > Warning > Info > hint > description > overflow, with live rejection suppressing even the lower-priority timer. |
| F-06 | Auxiliary panels never focus, capture, or outlive their anchors. | Closed: noninteractive policy is hit-transparent through native realization, omitted from focus isolation, and pruned with typed owners. |
| F-07 | Panel content cannot resize after placement is selected. | Closed: constrained wrapped content is measured before one `PlacementRequest` resolves. |

## Accessibility seams

- Description -> AccessKit `Description`.
- Node-supplied description -> `DescribedBy`.
- Rejected validation -> `Invalid` plus `ErrorMessage`.
- Runtime feedback may later project through `Live` when its caller proves the
  announcement policy.

These mappings are recorded now; the complete accessibility tree remains a
separate campaign.

## Checkpoint receipts

### 1. Description and hint are separate command truths

- `command::Spec::description` retains stable meaning and is carried through
  registry resolution into bindings without entering `command::State`.
- `command::State::hint` replaces the old tooltip-named channel at every
  contextual producer.
- `State::with_command` continues to default only label and shortcut; it does
  not copy description into hint.
- Focused tests independently pin description and hint, and representative
  gallery and editor commands declare descriptions.
- The accepted 12-pixel menu-title insets are pinned by a behavior-shaped
  regression instead of the stale square-title assumption found at preflight.

### 2. First-party feedback truth

- `feedback::Severity::{Info, Warning, Error}` is the entire public
  vocabulary. Reporting accepts `Display` and eagerly retains one formatted
  string; there is no framework error trait or public message wrapper.
- Typed owners remain typed: table cells retain their own feedback stacks and
  windows retain theirs in ephemeral session state.
- A severity stack preserves independent runtime facts while projecting the
  highest-priority current fact (`Error`, then `Warning`, then `Info`).
- Runtime context exposes report and clear operations without requiring an
  application to construct a panel. Window destruction destroys its feedback.
- Existing table rejection strings now occupy the Error slot without changing
  focus behavior; presentation and complete draft lifetimes remain checkpoint
  3 work.

### 3. Table rejection is a complete contract

- The typed cell store now retains a severity stack beside the active draft;
  rejection keeps the existing cell identity, draft, edit session, and focus.
- Failed commit requests `Rebuild`, not merely `Layout`, because the anchored
  panel is derived while rebuilding the view. This closes the field report in
  which the reason waited for unrelated input before appearing.
- Draft mutation, cancel, successful commit, provider removal, and window
  destruction all clear the rejection no later than their owning draft/cell.
  Command-driven edit, undo, and redo use the same clearing rule.
- Deliberate departure continues through the one commit-and-deactivate owner;
  platform window deactivation is explicitly not translated into such a
  departure.
- A gallery warning proves that severity alone never traps keyboard focus.

### 4. Auxiliary panels are policies, not paths

- `PanelPolicy` names interactive, hover-tip, anchored-feedback, and
  window-feedback behavior while the existing overlay store still owns the
  only host/realization/fade/receipt path.
- Noninteractive panels carry `accepts_input = false` through draft, live
  layer, native-popup projection, and retirement. Windows receive an empty
  native hit region; in-frame hit testing receives no target.
- Floating-panel focus isolation now applies only to interactive panels, so a
  warning can remain visible while Tab traverses underlying controls.
- `View` always has one stable Root. Adding or removing an auxiliary child can
  no longer replace the content root and accidentally prune focused or edited
  state.
- Hover dwell is armed only after a successfully presented layout proves that
  the current target has hint, description, or confirmed overflow. Ordinary
  controls therefore pay no timer or redraw cost.

### 5. One auxiliary anatomy

- `AuxiliaryChrome::{Info, Warning, Error}` uses the shared Phosphor icon path
  (`info`, `warning`, `x-circle`) with theme-owned extent, gap, and colors.
- The post-constitution product amendment is explicit: overflow revelation
  uses `Plain` chrome with no glyph. It reuses the same anatomy and panel path
  without being misclassified as retained informational feedback.
- Auxiliary text uses the same interface typography as menus, table cells,
  and controls. The icon is vertically centered against the complete wrapped
  text block.
- Theme/TOML owns the dwell delay, maximum width and height, icon geometry,
  and severity colors; no producer carries presentation numbers.

### 6. Content first, placement second

- The context-menu-only `menu_anchor`/`menu_available` fields were promoted to
  generic panel-placement fields. Context menus and every auxiliary policy now
  submit the same `PlacementRequest`.
- Table feedback resolves its typed cell rectangle; hover tips resolve their
  current interaction target; window feedback uses an ordinary point anchor.
  No second flip/clamp implementation exists.
- Intrinsic width is capped, text is remeasured with wrapping after icon/gap
  reservation, height is capped by declared clipping policy, and only then is
  placement resolved. A bounded-content witness pins 140x64 behavior and the
  structural absence of a nested Scroll species.

### 7. Overflow truth gains its consumer

- `OverflowProjection` now retains an explicit `overflowed` fact alongside
  source, visible text, and source mapping for Clip and ellipsis alike.
- Frame projection carries that fact to hover eligibility; neither the pointer
  path nor the auxiliary resolver measures text again.
- Compact overflow reveals the complete source in a wrapped, glyphless panel.
  Fully visible compact cells and expanded/wrapped cells return no tip.
- Measurement now reads the same node type style that paint uses. This removed
  an old table-interface/body-font disagreement and restored the configured
  24-pixel expanded-row floor for short content.

### 8. Accessibility and closeout

- The design doctrine records Description, DescribedBy, Invalid,
  ErrorMessage, and future Live mappings without making semantics conditional
  on panel visibility. The roadmap's AccessKit seam carries the same list.
- The text editor's asynchronous `last_status` save error was not migrated:
  the current window store is severity-scoped rather than producer-scoped, so
  clearing a successful save could erase an unrelated Error. This is recorded
  as a principled non-merge until producer-scoped lifetime has a second caller.
- `feedback` owns runtime facts; the panel path owns presentation; no public
  message wrapper, error trait, universal anchor enum, tooltip host, or
  auxiliary placement solver was added.

## Final verification

- Formatting and diff whitespace checks pass.
- `cargo check --all-targets` passes.
- Library: 1,041 passed, 10 intentional deep-tier ignores, 0 failed.
- Doctests: 4 passed (1 ordinary, 3 compile-fail), 0 failed.
- Release deep tier: all 10 ignored acceptance/GPU witnesses passed, including
  premultiplied alpha, popup packing, group opacity, material transparency,
  and silhouette shader compilation.
- `text_editor`, `control_gallery`, and `glass_tuner --smoke` all exited 0.
- Existing four-scale popup projection, placement-edge, receipt, fade, and
  native hit-region witnesses remain green; the new policies reach them only
  through the shared path.
- `comparison_open: true` remains unchanged. No temporary diagnostics or
  alternate presentation mechanisms remain.
