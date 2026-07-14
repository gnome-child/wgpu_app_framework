# Input validity and status projection audit

Status: campaign in flight (2026-07-13). Checkpoint 0 changes no production
behavior.
`comparison_open: true` remains unchanged.

## Campaign execution ledger

Ignition is `bc860866` (`pre text edit refactor`). The canonical audit was
committed at that boundary, the worktree was clean, and the 19-commit local
pre-campaign stack was pushed to `origin/master` before ignition. No campaign
commit may be pushed. Roadmap item 32 is the sole in-flight owner.

Checkpoints 1 through 6 are one interlocked core. The core may not close before
the inline validity indicator and its inspectable cursor-snapshot explanation
are both present. Checkpoint 7 is the severable status tail.

| Rung | State | Independent boundary |
|---|---|---|
| 0 — reductions and absence pins | Complete | Named reductions, current source census, baseline ritual, roadmap ignition |
| 1 — accepted task transitions | Complete | Rejected departure admits no dependent action or click-chain contribution |
| 2 — one TextBox and row participation | Complete | Pre-gesture focal row gates every member; table edit identity is deleted |
| 3 — one responder path | Complete | Semantic table/row/cell/member layers replace editing-scope suppression |
| 4 — one current commit recipe | Complete | One current draft produces one trigger or one formatted rejection |
| 5 — draft-owned validity | Complete | Rejection lifetime is structurally bounded by its draft entry |
| 6 — inline validity and explanation | Pending | One indicator geometry owns reservation, paint, hit, hover, and accessibility |
| 7 — passive status projection | Pending | General status atom plus thin, virtualized `Column::status` sugar |
| 8 — doctrine and closeout | Pending | Deletion census, public API review, full/deep ritual, roadmap close |

### Checkpoint 0 named reductions

These names are the acceptance-test contracts. A checkpoint may refine the
fixture, but it may not weaken the stated postcondition or repair the failure
with a table-local exception.

| ID | Named witness | Current reduction receipt | Required postcondition |
|---|---|---|---|
| OTT-R01 | `text_commit_state_uses_current_draft_arguments` | `Binding::with_text_value` substitutes draft arguments while cloning state resolved for the base text. | `valid` enables and invokes from an invalid base; the inverse retains one issue and invokes zero times. |
| OTT-R02 | `typed_table_commit_parses_and_validates_once` | Typed columns validate before building an action and repeat parse/validation inside the mapper, with panic as agreement. | Parse and domain validation each execute exactly once per attempt. |
| OTT-R03 | `rejected_departure_blocks_other_cell_activation` | `Action::Sequence` continues after rejected focus transfer into text pointer edit and `BeginTableEdit`. | Old draft/task remain; the other cell receives no activation. |
| OTT-R04 | `rejected_departure_blocks_row_selection` | Pointer selection mutates before the focus transition is attempted. | A rejected departure leaves membership, focal row, and active column unchanged. |
| OTT-R05 | `rejected_departure_blocks_member_controls` | Checkbox/button press and slider manipulation are sequenced independently of accepted departure. | No member press, toggle, command, or manipulation occurs. |
| OTT-R06 | `rejected_gesture_does_not_advance_click_chain` | Click classification runs before the rejecting focus action. | The first corrected click starts a fresh global chain. |
| OTT-R07 | `selection_click_does_not_enter_text_click_chain` | A non-focal row selection click is currently classified and routed into its text surface. | Rapid select -> activate -> repeat yields row focalization, caret, then word/chunk; never select-all. |
| OTT-R08 | `resting_inspection_and_active_task_order_are_path_derived` | `editing_table_scope` suppresses table targets to manufacture active-editor precedence. | Rest uses `Inspection`; active input uses `Task`; first claim alone consumes conflicts. |
| OTT-R09 | `rejection_cannot_outlive_its_draft` | Table feedback and draft state are separate stores with a clearing checklist. | Cancel, success, eviction, removal, pruning, and destruction retire issue no later than draft. |
| OTT-R10 | `rejection_projects_inline_without_opening_a_panel` | Validation paints a cell outline and immediately builds an anchored floating panel. | Failed commit paints the input-owned glyph and opens zero panels. |
| OTT-R11 | `indicator_and_overflow_hover_are_exact_independent_targets` | A subtree-wide feedback block suppresses ordinary hint/overflow resolution. | Indicator hover explains rejection; remaining text hover explains confirmed overflow. |
| OTT-R12 | `status_projection_is_bounded_under_virtualization` | No status species exists; the typed-column erasure seam and virtualized cell boundary are available. | `Some(Status)` projects passively; `None` is blank; work remains bounded to materialized rows. |

### Checkpoint 0 structural-absence contract

The opening Rust-source census is the deletion baseline:

| Retired shape | Opening footprint |
|---|---:|
| `BeginTableEdit` | 3 lines / 2 files |
| `begin_table_edit` | 7 lines / 7 files |
| `editing_table_cell` | 19 lines / 9 files |
| `project_table_edit` | 2 lines / 2 files |
| `editing_table_scope` | 8 lines / 1 file |
| `table_edit_error` | 28 lines / 12 files |
| `table_edit_feedback` | 1 line / 1 file |
| `reject_table_edit` | 2 lines / 2 files |
| `clear_table_edit_error` | 4 lines / 3 files |
| `first_table_rejection` | 3 lines / 2 files |
| `blocked_by_feedback` | 5 lines / 1 file |
| `PanelPolicy::AnchoredFeedback` | 2 lines / 2 files |
| public `table::TextEditor` | 5 direct references / 2 Rust files |
| public `table::NumberEditor` | 5 direct references / 2 Rust files |
| stale `with_text_value` substitution | 2 lines / 1 file |

Checkpoint 8 activates one architecture witness over Rust production sources
that fails if any retired edit-mode, forced-mode, table-validity,
table-feedback-panel, context-suppression, duplicate-parse, or stale-state
substitution shape returns. Broad domain names such as `TableCell` are not
forbidden; the witness names the retired owner or projection exactly. It also
requires doctrine for current-argument commit, selection-before-participation,
draft-owned validity, one TextBox surface, and the one panel path.

### Checkpoint 0 receipts

- Pre-campaign `master` was fetched and verified 19 commits ahead / 0 behind;
  `bc860866` was pushed before ignition. No campaign commit is pushed.
- The worktree was clean and the canonical audit was already tracked at the
  ignition boundary. Roadmap item 31 remains the completed Feedback campaign;
  this campaign occupies new item 32.
- `git diff --check` and `cargo fmt --all -- --check` passed.
- `cargo check --all-targets` passed without warnings.
- `cargo test --lib` passed: 1,043 passed, 10 intentional deep-tier ignores,
  0 failed.
- `cargo test --doc` passed: 1 ordinary and 3 compile-fail doctests.
- `text_editor`, `control_gallery`, and `glass_tuner --smoke` exited 0.
- `examples/glass_tuner/app/state.rs` still declares
  `comparison_open: true`.

### Checkpoint 1 — accepted task transitions

- Private `TaskTransition` makes acceptance independent from the ordinary
  handled/changed/effect outcome. `CommitAttempt` reports accepted versus
  rejected directly; the transition gate performs no table-feedback lookup.
- Pointer routing determines the requested text/control task before applying
  virtual-row selection, classifying a click, dismissing overlays, pressing a
  control, or invoking a slider action. Rejection cancels the one general
  click sequence and admits none of those continuations.
- Table Enter/Tab navigation, ordinary Tab focus, menu/palette activation, and
  shortcuts consult the same accepted transition. `Action::Sequence` retains
  its unconditional meaning for independent actions.
- Table dividers and projected chrome remain manipulation rather than task
  transfer; the existing active-editor resize witness caught and pinned that
  non-merge.
- `rejected_departure_blocks_other_cell_activation_selection_and_click_chain`
  pins cell activation, row selection, focus/task retention, pointer press,
  and the fresh post-correction click chain.
- `rejected_task_transition_blocks_controls_shortcuts_and_tab` pins Tab,
  click-away, button, checkbox, slider, and shortcut continuations. Existing
  platform coverage continues to prove `Focused(false)` is not translated
  into a deliberate framework departure.
- Formatting, diff hygiene, all-target compilation, 1,045 library tests with
  10 intentional deep-tier ignores, all 4 doctests, and all three application
  smokes passed. `comparison_open: true` remains protected.

### Checkpoint 2 — one TextBox and deliberate row participation

- Editable table cells now retain one `TextBox`, focus target, node identity,
  input policy, and commit binding across rest and participation. The active
  input target selects the ordinary single-line field viewport; inactivity
  selects the column-owned alignment, wrap, and overflow projection without a
  `TextArea`/`TextBox` node-species exchange.
- Pointer, Enter, and F2 enter the same ordinary draft target. Enter and Tab
  retain table navigation only after successful ordinary TextBox commit and
  deactivation. Numeric cells continue to use the signed-integer lexical
  policy; the lawful intermediate `-` reaches commit-time parse rejection.
- Each selectable-row pointer gesture snapshots pre-gesture focality. A
  non-focal or selected-but-not-focal row changes selection/focality only;
  Shift, Ctrl, and platform-primary selection gestures never participate.
  The same gate covers text, checkbox, button, and future cell members.
- Selection-only and rejected gestures cancel the general repeated-click
  chain. The pinned rapid journey is row selection, caret placement, then
  word/chunk selection—never an accidental select-all.
- Row departure runs ordinary text commit/deactivation before selection,
  including context-selection. A rejected draft blocks the focal-row change
  and menu opening. A successful departure retains draft storage but not the
  active input target; returning to that row is selection-only until a later
  participation gesture. Focus no longer promotes a retained draft into an
  active text task.
- `BeginTableEdit`, table editing ownership, begin/finish session helpers,
  `project_table_edit`, `TablePart::Editor`, forced table text modes, and the
  display/editor constructor switch are absent from production Rust sources.
  Table validation metadata remains temporarily for Checkpoints 4–5.
- `selectable_rows_gate_members_by_pre_gesture_focality_and_modifiers`,
  `table_text_selects_row_before_participation_and_keeps_one_text_box_identity`,
  `text_task_deactivates_when_focal_row_changes_and_reentry_is_selection_only`,
  and the expanded rejected-departure/cross-gate witness pin the correction.
- Formatting, diff hygiene, all-target compilation, 1,047 passing library
  tests with 10 intentional deep-tier ignores, all 4 doctests, and all three
  application smokes passed. `comparison_open: true` remains protected.

### Checkpoint 3 — one responder path

- Context ownership now projects table domain, focal row, cell identity, and
  exact member as distinct broad-to-exact layers. A service facet is explicit:
  the table layer contributes table selection commands, the exact text member
  contributes ordinary text commands, and the row/cell layers contribute only
  their own bindings or identity.
- `Inspection` walks that path table-first. First-claim consumption therefore
  leaves one table-owned `Select All`, keeps row commands between the table and
  text sections, and still exposes resting selectable-text operations.
  `Task` walks from the active member outward, so Select All, Copy, Cut, Delete,
  and Paste precede the row/table layers without table-target suppression.
- Context traversal compares the exact member's ordinary text target with the
  active input target. The editing-cell probe and `active_table_text_cell`
  helper are deleted. Context candidates no longer inspect session edit state,
  reorder table/text targets, or deduplicate a collapsed owner.
- Captured row and exact-member actions continue to invoke through their
  captured routes after projection. Live keyboard routing remains independent
  of the menu snapshot; Ctrl+A is claimed by the active text task and leaves
  table multiselection untouched.
- `table_context_cells_override_row_actions_and_virtual_removal_prunes_the_menu`
  now pins all four semantic layers, resting section order, conflict
  consumption, captured row/member invocation, and removal pruning.
  `active_table_editor_uses_task_order_and_owns_select_all` pins the full
  standard edit-command family before broader owners and live task ownership.
- Formatting, diff hygiene, warning-free all-target compilation, 1,047 passing
  library tests with 10 intentional deep-tier ignores, all 4 doctests, and all
  three application smokes passed. `comparison_open: true` remains protected.
  The independent checkpoint boundary is `bdddee71`.

### Checkpoint 4 — one current commit recipe

- `view::TextCommit` erases one closure from current draft text to either a
  current `AnyTrigger` or one formatted rejection. Infallible and fallible
  public TextBox constructors converge there through `on_commit`,
  `commit_with`, and `try_commit_with`; the retired submit vocabulary has no
  compatibility alias.
- Text commits no longer materialize a view binding from the displayed base or
  copy resolved command state across argument substitution. The recipe builds
  first, then its fresh trigger resolves and invokes through the current live
  responder chain. Disabled current arguments reject departure and keep the
  draft task active instead of inheriting the base arguments' state.
- `Column::text(...).validate(...)` accepts any displayable rejection. Its one
  erased recipe performs `FromStr`, optional domain validation, and typed
  command mapping exactly once. Syntax and validation errors are formatted
  once before crossing the erased boundary; no panic asserts agreement between
  duplicate paths.
- Public `table::TextEditor`, public `table::NumberEditor`, `table::Edit`,
  `table_edit_action`, `table_cell_is_editable`, the text-specific
  `AnyValueTrigger` binding, and `Binding::with_text_value` are absent. Typed
  columns and custom table fixtures now compose the ordinary TextBox directly,
  including its input and inactive-display policies.
- `text_commit_state_uses_current_draft_arguments` pins both inverse base/draft
  states. `typed_table_commit_parses_and_validates_once` counts each operation,
  and `fallible_commit_formats_one_rejection_snapshot` counts the one `Display`
  boundary. The old disabled-base cursor expectation now proves that command
  state cannot disable editing before draft arguments exist.
- The existing table rejection store is intentionally the temporary visual
  bridge at this independent boundary. Checkpoint 5 moves the rejection under
  the general draft entry and deletes that store; no new table retention was
  introduced here.
- Formatting, diff hygiene, warning-free all-target compilation, 1,050 passing
  library tests with 10 intentional deep-tier ignores, all 4 doctests, and all
  three application smokes passed. `comparison_open: true` remains protected.
  The independent checkpoint boundary is `4523be4f`.

### Checkpoint 5 — draft-owned validity

- The bounded draft store now retains an entry containing both `draft::State`
  and its ranked `feedback::Stack`. Rejections are reported only against an
  existing exact text target, so an attempt outcome cannot create a parallel
  or ownerless validity record.
- Failed recipe construction and disabled current command arguments retain one
  formatted Error with the current draft while leaving its text task active.
  Editing changed text clears that rejected-attempt Error; Warning remains
  independently ranked beneath Error. Sealing a successful commit and Escape
  cancellation clear the feedback with the draft lifecycle.
- Explicit clearing, bounded-store eviction, removed element identity,
  provider-row/cell pruning, and window destruction remove the whole entry and
  therefore retire its feedback structurally. The platform `Focused(false)`
  mapping remains absent, so system deactivation cannot become a trapped
  deliberate departure.
- `interaction::Tables.feedback`, its rejection methods, table-specific
  session accessors, and the former clearing checklist are deleted. The
  temporary Checkpoint 5 table presentation bridge reads the exact TextBox
  draft feedback directly; it owns no rejection copy and is removed by the
  inline projection in Checkpoint 6.
- `rejection_cannot_outlive_its_draft` covers edit, success, explicit clear,
  eviction, identity pruning, and provider-cell removal.
  `closing_window_destroys_its_text_draft_feedback` covers destruction, and
  the editable-table journey covers failed departure, correction, and cancel.
  `text_commit_state_uses_current_draft_arguments` now also proves that the
  inverse current-argument rejection retains one nonempty issue.
- Formatting, diff hygiene, warning-free all-target compilation, 1,050 passing
  library tests with 10 intentional deep-tier ignores, all 4 doctests, and all
  three application smokes passed. `comparison_open: true` remains protected.

## Mission

Audit the proposed invalid-input indicator and opt-in table status column at the
framework's lowest honest concepts. API compatibility is not a constraint.
Promotion, unification, and deletion are preferred when receipts prove that a
table-specific mechanism is a special case of an existing framework truth.

The motivating behavior is deliberately small:

- a failed text commit remains an editable draft;
- its input projects a trailing error indicator that reduces the text area;
- hovering the indicator reveals the retained reason through the one panel
  path;
- a table may opt into a fixed status column whose row-owned state projects an
  icon and the same hover-panel anatomy.

The audit asks whether those outcomes require new table machinery. They do
not.

## Executive verdict

**GO, but as an input-validity and text-participation campaign with a
status-column witness.** The current tree contains five concepts at the wrong
altitude:

1. Text commit is represented as a pre-resolved view binding even though its
   command arguments do not exist until the draft is committed.
2. Failed commit state is retained in `interaction::Tables`, despite ordinary
   text boxes and table editors already sharing the same draft store and
   `interaction::Target` identity.
3. Indicator semantics are encoded as panel-only `AuxiliaryChrome`, despite the
   proposed inline error/status glyph being a second projection of the same
   icon, tone, and explanatory text.
4. Table editing swaps a read-only `TextArea` for a `TextBox` and retains an
   independent table-edit identity, even though the ordinary text task already
   owns focus, draft, input policy, command routing, and commit.
5. Focus transfer and the interaction that depends on it are siblings in an
   unconditional `Action::Sequence`. A rejected focus departure therefore
   cannot prevent a later edit or control activation in the same gesture.

The clean route is therefore:

- promote a **fallible, type-erased commit recipe** shared by ordinary text
  boxes and table editors;
- keep one **TextBox species** across inactive display and active editing, with
  column display policy used while inactive and the ordinary single-line field
  viewport used while active;
- make accepted focus/task entry a **precondition** of dependent interaction,
  and derive table-member participation from the pre-gesture focal row rather
  than an OS double-click timer or a second edit flag;
- retain failed commit facts **with the draft entry** under the existing input
  target;
- promote one **resolved hint/indicator projection** consumed by inline chrome,
  hover panels, and later accessibility;
- add a general passive status indicator, then make `Column::status` thin
  table sugar over it.

This route deletes more table-specific code than the status column adds. It
also closes a pre-existing argument/state disagreement in ordinary text boxes,
a general focus-transition bypass, and an ad-hoc context-routing suppression.

## Proposed laws

1. **An outcome is not retained state.** `Result<T, E>` describes one attempt.
   The owner that needs persistence snapshots `E: Display` and retains the
   failure under its own identity and lifetime.
2. **Input validity is not command state.** A draft can fail before command
   arguments exist and before any `Target<C>` can be consulted. Command state
   continues to answer whether an already-formed command can act.
3. **A commit recipe is resolved at commit time.** Draft text produces exactly
   one current trigger or one formatted rejection. No command state resolved
   for the base value may authorize or suppress a different draft value.
4. **A rejection may not outlive its draft.** Clear, cancel, successful commit,
   bounded-store eviction, removed identity, and window destruction retire the
   failure no later than the draft they retire.
5. **Invalidity does not imply focus policy.** Retained failure, visible
   indicator, deliberate focus trapping, and platform deactivation remain
   separate facts. The current strict table policy may stay without becoming a
   universal semantic consequence of `Err`.
6. **One indicator geometry feeds every consumer.** Text layout, caret,
   selection, hit testing, icon paint, and accessibility bounds consume the
   same trailing-slot projection.
7. **Share semantic payload, not rendered panels.** Inline status and hover
   revelation may consume the same resolved icon/tone/text; only the hover
   policy constructs a floating panel.
8. **Severity, visual tone, and operational state are separate axes.** Error
   and warning are ranked runtime facts. Play/pause is application state.
   Neither becomes a single mixed enum merely because both can project an
   icon.
9. **A dependent interaction requires an accepted task transition.** Focus,
   pointer edit, control activation, and table edit entry are not independent
   siblings. If deliberate departure is rejected, no later action in that
   gesture may mutate another target.
10. **Selection precedes participation.** The first primary click on a table
    row makes it focal. Only a subsequent primary gesture whose row was already
    focal may invoke a member control. Membership in a multiselection is not an
    allowance, and Ctrl/Shift selection gestures never participate.
11. **One text surface owns rest and edit.** A table does not exchange a
    `TextArea` for a `TextBox`. One TextBox keeps one target and commit recipe;
    at rest it projects the column's alignment/wrap/overflow, and while active
    it projects the ordinary single-line editing viewport. The text input
    session's existing active target selects between those projections; cell
    focus alone does not imply editing.
12. **Routing follows semantic layers, not feature suppression.** Table, row,
    cell, and member facet are distinct layers in one responder path. Task
    traversal lets active text consume first; Inspection traversal lets the
    table consume first. No `editing_table_scope` exception may manufacture
    either result.

## Census

| Cell | Receipt | Finding | Verdict |
|---|---|---|---|
| A-01 — command boundary | `src/target/mod.rs:13-17`; `src/command/trigger.rs:20-55` | `Target<C>::state` requires `C::Args`. A parse failure occurs before `C::Args` exists. | Invalid draft state cannot honestly live in `command::State`. Non-merge. |
| A-02 — ordinary text commit | `src/widget/control/text_box.rs:39-54`; `src/widget/trigger.rs:69-85` | `TextBox` accepts only an infallible `String -> C::Args` mapper and lowers it into `AnyValueTrigger<String>`. | Add a fallible std-shaped boundary; do not add a validation trait. |
| A-03 — stale binding state | `src/view/binding.rs:155-165,214-227`; `src/view/node/traversal.rs:516-534` | Commit substitutes the new draft into the trigger but clones command state resolved for the old/base text. `text_action` gates on that old state. | Proven source-of-truth violation. Replace the text binding with a commit-time recipe. |
| A-04 — invocation truth | `src/runtime/services/target.rs:114-143`; `src/responder/chain.rs:232-279` | Invocation claims state again using the actual command arguments. | The runtime already owns current-argument authorization; the view-side stale gate is redundant and can disagree. |
| A-05 — table parsing | `src/table.rs:693-734` | Typed editing parses and validates once, then parses and validates again inside the commit mapper with `panic!`/`expect` as the agreement mechanism. | One fallible recipe can parse once and delete the duplicated proof-by-panic. |
| A-06 — manual table editors | `src/table.rs:1049-1198`; source-wide caller census | `TextEditor` and `NumberEditor` duplicate validation and commit mapping. Neither has a production caller outside tests/docs; `FromStr`, input policy, ordinary TextBox focus, and the shared commit recipe already name their behavior. | Delete both public table editor types. Typed columns and custom cells compose the general TextBox directly. |
| A-07 — draft identity | `src/draft/input/mod.rs:15-28`; `src/interaction/target.rs:10-35,90` | Ordinary fields and table cells already retain drafts under one `interaction::Target` vocabulary. | No universal validation anchor and no table-only identity are needed. |
| A-08 — draft lifetime | `src/draft/input/store.rs:8-21,53-124`; `src/draft/input/mod.rs:205-252` | The store already owns explicit clear, LRU eviction, removed-node/element/cell pruning, and active-target protection. | Store draft plus issue in one entry so the lifetime law holds structurally. |
| A-09 — table rejection store | `src/interaction/table.rs:3-9,56-104`; `src/session/table.rs:20-49` | A parallel `Cell -> feedback::Stack` store duplicates draft identity and clearing. | Delete after input-owned validity lands. Table interaction should retain widths/edit session, not text validity. |
| A-10 — table rejection plumbing | `table_edit_error` spans 12 source files; `first_table_rejection`, `blocked_by_feedback`, `AnchoredFeedback`, and `PanelAttachment::TableCell` exist only for this projection. | One table error currently travels session -> view node -> layout frame -> scene and a separate tree scan -> floating panel. | Large deletion target; exact input indicator replaces ancestor blocking and global first-error scanning. |
| A-11 — feedback truth | `src/feedback.rs:1-68`; `docs/master_design.md:1039-1052` | Severity plus eagerly formatted text and ranked stacks already exist. Severity deliberately does not own focus, lifetime, dismissal, or interaction. | Reuse the fact shape and `Display` boundary; do not create an error-message trait or wrapper. |
| A-12 — panel projection | `src/view/feedback.rs:53-92`; `src/view/node/mod.rs:75-127` | Hover panels currently resolve command text or overflow into panel-only `AuxiliaryChrome`. | Promote the resolved icon/tone/text payload; retain the single panel path and its policies. |
| A-13 — current priority coupling | `src/view/node/traversal.rs:18-60` | A table error blocks hints for the entire descendant subtree, and the first rejection is found by tree order. | Replace coarse blocking with exact indicator targeting: hover indicator -> issue; hover text -> overflow/description. |
| A-14 — text geometry | `src/layout/frame.rs:192-240,1073-1085`; `src/layout/control.rs:129-205` | One text rectangle already feeds shaping and pointer mapping. Table headers and choices already reserve trailing/leading parts from shared row recipes. | Add one input-parts projection beside the existing control-part recipes; no inline padding arithmetic in paint or hit code. |
| A-15 — paint fork | `src/scene/paint/mod.rs:305-325,651-686` | Table invalidity paints a whole-frame focus-colored outline, while auxiliary glyph selection is hard-coded inside panel paint. | General invalid input paint and general indicator paint replace both special assumptions. |
| A-16 — icon vocabulary | `src/icon.rs:1-65` | Stable Phosphor icon identity/style/glyph resolution is already public. | Reuse real icon IDs (`x-circle`, `warning`, `info`, later play/pause); never store Unicode lookalikes. |
| A-17 — table escape hatch | `src/table.rs:487-539` | Text, Boolean, and custom columns already share a typed-to-erased provider boundary. | `Column::status` should be sugar over a general indicator projection, not a fourth provider system. |
| A-18 — accessibility | `docs/master_design.md:1070-1074`; `docs/roadmap.md` AccessKit reservation | Description, DescribedBy, Invalid, ErrorMessage, and Live are already reserved as separate seams. | Input owns Invalid/ErrorMessage independent of whether a tooltip is visible; status owns its accessible description. |
| A-19 — pointer activation | `src/runtime/pointer.rs:84-155`; `src/view/action.rs:134-153` | Row selection happens first, then text focus/edit and `BeginTableEdit` are assembled as one unconditional sequence; checkbox/button activation is armed independently for pointer-up. | The current first click both selects and participates. Snapshot pre-gesture focal state and admit only one of those outcomes. |
| A-20 — unconditional sequencing | `src/runtime/routing.rs:65-75`; `src/runtime/input/text/focus.rs:45-77` | A rejected focus transfer returns a handled outcome, but `Action::Sequence` continues to later edits/activations. Sliders and ordinary focusable controls use the same shape. | Proven general bypass. Add a conditional focus/task transition; do not add another table rejection check. |
| A-21 — edit identity and species switch | `src/table.rs:1202-1258`; `src/view/node/builder.rs:300-314`; `src/view/node/traversal.rs:342-360`; `src/view/control/text_box.rs:138-150` | `table::Edit::node` alternates `TextArea` and `TextBox`; `interaction::Tables.editing` independently selects the species even though TextBox projection already computes whether its target is the active text input. | Replace with one TextBox carrying inactive display policy and active field policy. Promote the input target as active-task truth and delete the table edit-mode owner. |
| A-22 — duplicated activation | `src/runtime/pointer.rs:120-125`; `src/runtime/input/selection.rs:86-116`; `src/runtime/routing.rs:184-196` | Pointer double-click, Enter/F2, and direct `BeginTableEdit` each mutate edit state through separate routes. | One ordinary task-entry request must serve pointer and keyboard activation. |
| A-23 — forced text mode | `src/runtime/input/text/mod.rs:24-33`; `src/runtime/input/text/field.rs:60-68`; `src/runtime/services/text/focused/mod.rs:49-60` | Three text paths override the surface's own mode when `editing_table_cell` matches. | Delete; the TextBox session is the editability truth. |
| A-24 — collapsed context layer | `src/view/mod.rs:43-51`; `src/view/node/traversal.rs:99-166`; `src/runtime/services/mod.rs:69-96,254-296` | One `ContextOwner` can carry table/cell and text focus together. Services then try table before text and compensate by suppressing table targets throughout an editing table scope. | Project table/row/cell/member as separate path layers; traversal and first-claim consumption already express both desired orders. |
| A-25 — legitimate table residue | `src/selection.rs:7-14,52-68`; `src/runtime/input/selection.rs:15-60`; `docs/master_design.md:954-960` | Stable keys, membership, active/focal row, column navigation, and virtualization pins are real table/list truths. | Keep identity, selection, focal allowance, navigation-after-success, and pinning. Remove validation, draft, text mode, focus trapping, and text command ownership from table state. |

### Mechanical footprint of the table-only fork

The following census is the current deletion opportunity, not a promise to
delete by blind grep:

| Shape | Current source footprint | Expected disposition |
|---|---:|---|
| `table_edit_error` | 12 source files | Delete; input issue projection replaces it. |
| `table_edit_feedback` | 1 source file | Delete public table-only getter. |
| `reject_table_edit` | 2 source files | Replace with input-target issue retention. |
| `clear_table_edit_error` | 3 source files | Delete; draft-entry lifecycle owns clearing. |
| `first_table_rejection` | 2 source files | Delete; no first-error tree scan. |
| `blocked_by_feedback` | 1 source file | Delete; exact target resolution replaces ancestor suppression. |
| `with_table_panel_anchor` | 2 source files | Delete if no post-census caller remains. |
| `PanelPolicy::AnchoredFeedback` | 2 source files | Delete; validation revelation becomes ordinary hover-tip policy. |

## The hidden defect: old state, new arguments

This audit found a defect that is independent of the proposed status column.

`TextBoxBinding::bind` creates a trigger from the model/base text. During view
resolution, `Binding::resolve` obtains command state for those arguments.
Later, `Binding::with_text_value` replaces the trigger arguments with the draft
but copies the old `state`. `Binding::text_action` consults that copied state
before producing an action. Invocation then claims the target again with the
new arguments.

That produces two inverse failures:

- base arguments disabled, new draft enabled: the stale view state suppresses
  the commit before the runtime can see the valid arguments;
- base arguments enabled, new draft disabled: the view authorizes an action
  that current-argument invocation rejects.

The correct fix is not to refresh bindings on every keystroke. A text commit is
not a continuously presented button. Its arguments are born at commit time, so
its typed mapper and current responder resolution belong at commit time too.

Named reduction for the future campaign:

> A target whose state is enabled only for the string `"valid"` starts from an
> invalid base. Typing `valid` and committing invokes exactly once. Reversing
> the base and draft retains one issue and invokes zero times. Neither behavior
> depends on a rebuild between typing and commit.

## The second hidden defect: focus is sequenced, not gated

The pointer path currently treats focus and the interaction that needs focus as
independent actions. `Action::Sequence` deliberately aggregates every outcome
and never short-circuits. `focus_committing_text_box` can reject departure from
an invalid draft, but the next sibling still runs.

The named table reduction is:

1. cell A owns an invalid active draft;
2. the user double-clicks cell B;
3. focus transfer to B attempts commit and is rejected;
4. B nevertheless receives its pointer edit and `BeginTableEdit` action.

The same defect class exists for a slider or bound control clicked while an
invalid field is active. The correct repair is therefore a general conditional
transition, not `if table_edit_error` at every caller:

> An interaction that requires a new task may run only inside the success arm
> of that task's focus transition.

An ordinary independent `Sequence` remains useful. It is simply the wrong
vocabulary for dependent focus-then-act behavior.

## Table editing reduction: one surface behind one row gate

The proposed deletion is larger than moving validation. An editable typed cell
should materialize one TextBox carrying:

- stable `table::Cell` target identity;
- inactive display alignment, wrapping, and residual overflow from the column;
- active single-line field behavior from the ordinary text engine;
- lexical `text::Input` policy;
- one fallible commit recipe;
- one draft-owned issue.

The table supplies those policies when it constructs the cell but does not
retain or execute them. It retains only table truths. Existing
`draft::Input::target` is the edit-activation owner: the cell may retain
selection/navigation focus while the TextBox is inactive, and the permitted
member gesture activates that ordinary input target without minting a table
mode.

### Selection before participation

No extra armed bit is needed. `Selection::active()` is the existing focal row.
The pointer path must snapshot it before applying the gesture:

- row not previously focal: select/focus the row and suppress the member's
  press, focus, text-click classification, and activation;
- row already focal, no selection modifier: route the gesture to the ordinary
  member control;
- Ctrl/Shift gesture: update selection only;
- right-click: preserve the existing context-selection law rather than using
  the primary-participation gate.

If another input task is already active, its accepted departure precedes both
selection and participation. A rejected commit therefore leaves the old focal
row and the new row's member untouched; selection is not allowed to move first
and manufacture a split task.

This matters for multiselect: `contains(key)` is insufficient because any of
many selected rows could then toggle accidentally. The one focal key is the
allowance. It also means the first row-selection click does not enter the text
click chain. The next click activates and places the caret as a first text
click; repeated clicks after activation retain the global word/all selection
grammar.

Checkboxes, buttons, and future member controls consume the same gate. A fast
two-click gesture naturally selects then participates, but timing is not the
truth and a later second click works identically.

### Context routing falls out of the responder path

The current context owner collapses cell and text facet into one layer, then
`Services` tries the table service first. `editing_table_scope` compensates by
removing table targets whenever any cell in that table is editing. That is why
the desired behavior exists only through table-specific routing state.

Instead, one semantic path must contain the independently command-owning
layers: table, focal row, cell, and exact member facet. Then established
traversal already gives both results:

- resting cell: `Inspection` walks table to facet, so table `Select All`
  consumes and lower text `Select All` is omitted;
- active TextBox: its task frame selects `Task`, so text commands claim first
  and broader row/table commands follow where they do not conflict.

This deletes `context_traversal(...editing_cell)`, `editing_table_scope`, and
the table-wide target suppression. It also fixes the inactive text-command
gripes at their source: a text facet advertises text commands because it is a
text facet, not because table code remembered to proxy them.

### Expected deletion surface

The current special edit identity reaches nine source files through
`editing_table_cell`; `begin_table_edit` reaches seven; table rejection has 47
runtime/view/layout/scene references. The intended structural absences are:

- `Action::BeginTableEdit` and `view::Action::begin_table_edit`;
- `interaction::Tables.editing` and session begin/finish edit methods;
- `table::Edit::node(editing)` plus `project_table_edit`;
- table-specific `table_edit_action`, `table_cell_is_editable`, and forced text
  modes;
- `editing_table_scope` and editing-cell-driven context traversal;
- the separate table rejection store and its projection pipeline.

Do not delete table-specific Enter/Tab navigation semantics. They remain a
postcondition of successful ordinary TextBox commit: commit first, and only its
success arm moves to the next table cell.

## Recommended concepts

### 1. One type-erased commit recipe

The application boundary remains fully typed and std-shaped. Type erasure
occurs before `view::Node`, following the existing command and typed-column
pattern.

Illustrative internal shape, not a frozen public name:

```rust
struct Commit {
    build: Arc<dyn Fn(String) -> Result<command::AnyTrigger, String> + Send + Sync>,
}
```

Generic constructors accept `E: Display`, format `Err` exactly once, and erase
the successful command to `AnyTrigger`. No public validation trait is needed.

The commit sequence becomes:

1. read the current draft and its exact commit recipe;
2. build one trigger or one formatted issue;
3. on `Err`, retain the issue under the draft target and request the projection
   update;
4. on `Ok(trigger)`, resolve/invoke that trigger through the current responder
   chain with its current arguments;
5. only successful commit seals the draft, clears its issue, and completes the
   ordinary departure policy.

The existing infallible text-box API is sugar over `Ok(args)`. The typed table
column captures `FromStr`, domain validation, and application mapping in one
closure, so syntax and domain checks execute once.

### 2. Draft-owned input issue

Promote the draft store value from bare `draft::State` to an entry containing
the editor state and its retained input feedback. Keep buffer/history/cursor
inside `draft::State`; do not make the text buffer depend on presentation.

Illustrative ownership only:

```rust
struct Entry {
    draft: draft::State,
    feedback: feedback::Stack,
}
```

This is the smallest owner that can enforce rejection-at-most-draft without a
clearing checklist. It also generalizes beyond tables without promoting an
untyped global anchor.

The current strict local-departure rule may continue: a deliberate Tab, Enter,
cell change, or click-away that fails commit keeps the editing task active and
shows the reason. Window deactivation and system focus changes remain
non-trapping. Invalidity itself does not encode either policy.

### 3. Resolved hint and inline indicator

The panel presenter should consume one ephemeral resolved payload with these
independent meanings:

- explanatory text;
- optional `icon::Icon`;
- visual tone.

Command hint, command description, overflow text, retained validation, and row
status remain separate sources on separate clocks. They are converted into the
resolved payload only while projecting the exact hovered element. Do not copy
overflow into command state or row status into feedback storage.

The inline indicator consumes the same icon/tone/description but not the panel
node. Validation maps Error to `x-circle`; Warning maps to the warning glyph;
plain overflow has no glyph. The status widget may carry any real icon with a
neutral or severity-derived tone.

`AuxiliaryChrome` is therefore a promotion candidate: its four hard-coded
panel variants can become a resolved hint recipe rather than remaining the
owner of glyph selection. Theme remains the owner of extent, gap, and colors.

### 4. One indicator geometry

Add a control-parts helper analogous to table header and choice recipes:

```text
input bounds
  -> content bounds
     -> text bounds + trailing indicator bounds
```

The indicator bounds are authoritative for reservation, paint, hover targeting,
and accessibility. The text engine receives only the reduced text bounds, so
glyph layout, selection, caret, and hit mapping cannot disagree with the
indicator slot.

The indicator is a hover target without becoming an editing surface or command
button. Hovering it resolves its issue/status hint. Hovering the remaining text
continues to resolve confirmed overflow. This removes the current all-or-nothing
ancestor block.

### 5. General status atom, then table sugar

The framework should add a passive status indicator independently of tables.
An application owns the status and its lifetime; the widget projects one icon,
tone, and description. Interaction remains a separate optional future axis.

`Column::status` then becomes a small typed builder that:

- accepts a row accessor/projection returning `Option<Status>`;
- defaults to a fixed compact width and centered glyph;
- is passive and unsortable in v1;
- produces an empty cell for `None`;
- uses the same exact-target hover hint and accessible description as the
  general status widget.

The column is a final acceptance witness, not the campaign's architecture.
Errors and warnings prove severity mapping. A neutral icon proves operational
status is not being smuggled into `Severity`. Play/pause stays a watch-line
example until its model caller exists.

## Public API direction

API breakage is justified where current names encode the wrong event or retain
redundant types.

### Text boxes

`TextBox` commits on focus departure as well as Enter, so `submit` is narrower
than the behavior. Prefer the commit vocabulary consistently:

```rust
TextBox::on_commit::<C>()
TextBox::commit_with::<C>(|text| Args { text })
TextBox::try_commit_with::<C, E>(|text| -> Result<Args, E> { ... })
```

The infallible forms wrap the fallible recipe. If compatibility is not a goal,
delete `on_submit`/`submit_with` rather than maintaining aliases indefinitely.

`text::Input` owns lexical admission and normalization: signed integers,
decimal syntax, character classes, and lawful intermediate drafts. The
TextBox's commit recipe owns parse and domain validation because those are
questions about leaving the draft and forming application arguments. The text
engine does not learn what a record value means, and the table does not run a
second validator.

The TextBox also accepts inactive display policy (alignment, line formation,
residual overflow) independently from its active single-line field viewport.
This is general control behavior; table columns merely configure it.

### Typed table text

Keep the std capability boundary:

```rust
Column::text(...)
    .validate(|value: &V| -> Result<(), E> { ... }) // E: Display
    .editable::<C>(|cell, value| Args { cell, value })
```

`editable` continues to require `V: FromStr` and `V::Err: Display`. The erased
recipe performs `FromStr`, typed validation, and argument mapping once.

Delete both `TextEditor` and `NumberEditor`: integer editing is `FromStr` plus
the existing numeric `text::Input` policy, and text editing is the ordinary
TextBox plus a `table::Cell` focus identity. Custom columns can compose that
same public control directly; an ergonomic cell constructor may be thin sugar,
but it must not introduce a table edit state or runtime path.

### Status

Do not expose raw `Result<T, E>` as a widget. It lacks owner, lifetime, anchor,
severity policy, and a defined success presentation. A convenience constructor
may accept `E: Display`:

```rust
Status::error(error)
Status::warning(warning)
Status::new(icon, description)
```

The status value is a projection description, not application state. The row
model retains the real error/playback fact; the view maps it into `Status`.

## Required non-merges

- `command::State` and input validity: command args exist only after validation.
- `Result` and retained feedback: attempt versus lifetime-bearing fact.
- `feedback::Severity` and operational state: ranked urgency versus domain
  state.
- visual tone and severity: a neutral status icon is meaningful without being
  Info.
- inline indicator and floating panel: shared semantic payload, distinct
  projections.
- command hint, description, overflow, validation, and status: independent
  owners and clocks, resolved only at the element boundary.
- invalidity and focus trapping: state versus interaction policy.
- application row status and framework retention: the framework projects; the
  application owns source truth and recovery.

## Industry and accessibility evidence

The proposed behavior follows established desktop field-validation practice:

- Microsoft's Visual Studio guidance defines field validation as a control,
  icon, and tooltip; the icon may sit inside the control, the control gains an
  invalid border, and hovering the icon/control reveals the reason:
  <https://learn.microsoft.com/en-us/visualstudio/extensibility/ux-guidelines/notifications-and-progress-for-visual-studio?view=visualstudio>.
- WinForms `ErrorProvider` deliberately keeps validation visible beside the
  control and exposes its text on hover rather than using a dismissible modal:
  <https://learn.microsoft.com/en-us/dotnet/desktop/winforms/controls/errorprovider-component-overview-windows-forms>.
- WPF validation decorates the bound control through an `ErrorTemplate`, while
  validation occurs during target-to-source transfer before source update:
  <https://learn.microsoft.com/en-us/dotnet/desktop/wpf/data/how-to-implement-binding-validation>.
- WAI's ARIA guidance says invalidity must not be asserted before validation is
  attempted, and error text must be associated with the invalid object:
  <https://www.w3.org/WAI/WCAG22/Techniques/aria/ARIA21> and
  <https://www.w3.org/TR/wai-aria-1.2/#aria-errormessage>.

Consequences for this framework:

- lawful intermediate drafts such as `""`, `"-"`, or an incomplete exponent
  remain drafts, not errors, until a commit attempt;
- tooltip visibility is never the accessibility source of truth;
- the input owns Invalid/ErrorMessage semantics while the indicator and panel
  are visual projections;
- strict local focus retention is permitted, but it is a product policy rather
  than the meaning of invalidity.

## Campaign-ready sequence

### Checkpoint 0 — reductions and absence witnesses

Pin the stale-state/current-args reduction, focus-then-act bypass, row
selection-before-participation matrix, table failed departure family,
task-versus-inspection context ordering, draft-removal lifetime, exact
indicator-versus-text hover behavior, and accessibility independence. Record
structural absence requirements before code.

### Checkpoint 1 — one TextBox task path

Add conditional task entry so dependent actions run only after accepted focus.
Make the pre-gesture focal row the table-member participation gate. Materialize
one TextBox for editable cells with inactive column display policy and active
single-line field policy, selected by the existing active text-input target.
Pointer and Enter/F2 activation consume the same task entry. Delete
`BeginTableEdit`, the independent table edit identity, the
TextArea/TextBox node swap, and forced table edit modes.

Project table, row, cell, and exact member as distinct responder layers. Let
Task/Inspection traversal and first-claim consumption replace editing-scope
suppression. Preserve table navigation only after successful text departure.

### Checkpoint 2 — one current commit recipe

Introduce the generic-to-erased fallible commit recipe. Migrate ordinary
`TextBox`, typed `Column::text`, and custom table-cell witnesses. Parse and
validate once; resolve/invoke only current arguments. Delete the text-specific
state-substitution path, `TextEditor`, and `NumberEditor`.

### Checkpoint 3 — validity belongs to the draft entry

Move retained input feedback under the draft store's existing target/lifetime.
Preserve strict deliberate-departure behavior and the platform-focus exception.
Delete `interaction::Tables.feedback` and the session clearing checklist.

### Checkpoint 4 — one indicator, one exact hint target

Promote resolved hint/icon/tone, add one input-parts geometry, and project the
invalid indicator inline. Paint, reservation, hover, hit, and accessibility
consume the same bounds. Delete table error fields, first-rejection scan,
ancestor blocking, table-cell panel attachment, and anchored-feedback policy if
the final census confirms no other caller.

### Checkpoint 5 — status is the second inline caller

Add the passive general status indicator and thin `Column::status` sugar.
Prove Error, Warning, neutral operational icon, `None`, virtualization/removal,
and exact hover descriptions without adding a status store to the table.

### Checkpoint 6 — closeout by deletion

Update table/text/feedback doctrine, run the API review, record actual removed
symbols/lines, and preserve the one panel path. The final architecture witness
should fail if table validity returns to `interaction::Tables`, if commit state
is copied across argument substitution, or if status constructs a floating
panel directly.

## Acceptance matrix

| ID | Witness |
|---|---|
| IV-01 | Command state depending on draft arguments uses the committed value, never the base value. |
| IV-02 | Parse and domain validation execute once per commit attempt. |
| IV-03 | `Err(E: Display)` formats once and retains the snapshot for that attempt. |
| IV-04 | Failed Tab, Enter, click-another-cell, click-outside, and keyboard navigation retain the draft, focus/task, and exact issue. |
| IV-05 | Window deactivation and system focus changes are never trapped by input validity. |
| IV-06 | Draft mutation, cancel, success, eviction, provider removal, and window destruction clear the issue no later than the draft. |
| IV-07 | Indicator reservation moves text, caret, selection, hit mapping, and paint together at scales 1.0, 1.25, 1.5, and 2.0. |
| IV-08 | Hovering the error glyph shows the rejection; hovering overflowed text shows full text; neither shadows the other. |
| IV-09 | No invalid indicator exists before the first rejected commit attempt. |
| IV-10 | Error and warning status cells share the indicator/hint projection but not input retention. |
| IV-11 | Neutral operational status uses a real icon and accessible description without pretending to be Info. |
| IV-12 | Status `None` is blank, passive, and allocation/layout bounded under virtualization. |
| IV-13 | Invalid/ErrorMessage and status descriptions exist independently of hover-panel visibility. |
| IV-14 | Every revealed hint still uses the ordinary placement, host, receipt, fade, and exposure path. |
| IV-15 | A click on a non-focal row changes row selection/focus only; no checkbox, button, or text member is pressed or invoked. |
| IV-16 | A later unmodified click on the already-focal row invokes its ordinary member; Ctrl/Shift gestures remain selection-only. |
| IV-17 | First-row selection does not advance the text click chain; after activation, click/double/triple/repeated selection semantics remain global text behavior. |
| IV-18 | An invalid active TextBox blocks double-clicking another cell and clicking any other bound control: focus, draft, selection, target member state, and command count remain unchanged except issue presentation. |
| IV-19 | Resting cell context uses Inspection order (table, row, cell, member); active TextBox context uses Task order and text commands work without suppressing the table service. |
| IV-20 | Compact/Expanded and rest/edit transitions preserve one TextBox target and cell identity; inactive text obeys column overflow/wrap, active editing remains single-line. |
| IV-21 | Enter/F2 and pointer activation enter the same TextBox task path; Enter/Tab navigation occurs only after successful commit. |
| IV-22 | Structural searches find no `BeginTableEdit`, table edit-mode owner, table-forced text mode, or `editing_table_scope`. |

## Open decisions for implementation, not architecture

1. Whether a command disabled for the newly built arguments should retain its
   current `State::hint` as an input issue or return the ordinary disabled
   command outcome. In either case, state must be resolved from the new
   arguments and the draft must not disappear silently.
2. Whether the invalid field keeps a one-pixel error border in addition to the
   trailing glyph. Industry evidence supports it; the indicator alone satisfies
   the current product request. Theme owns the choice.
3. The exact public name of the neutral/severity visual axis. It is a real axis
   because neutral, warning, and error callers all exist, but it must not be
   named `Severity`.

None of these decisions changes the owner graph or blocks the campaign.

## Final recommendation

Proceed with the six implementation checkpoints above. The architectural
center is not “make `Result` displayable,” “add a table status column,” or
“patch table double-click.” It is:

> A fallible commit produces either a current command or a draft-owned issue;
> the issue and row-owned status facts share an indicator projection, while
> their ownership, lifetimes, and meanings remain distinct.

And its interaction companion is:

> A row selects before its member participates; once participation is allowed,
> the member follows the ordinary TextBox/control task path, and a rejected
> task transition admits no dependent action.

Those corrections remove the table-only edit-mode and validation stores, the
display/editor species switch, duplicated activation routes, context-service
suppression, the double parse, the stale command-state substitution, the
ancestor-wide tooltip suppression, and a panel-only glyph vocabulary. The
status column then arrives as a small proof that the promoted indicator concept
is genuinely general.
