# One Resolved Press campaign

Status: active. `comparison_open: true`. No push during the campaign.

Ignition is `7f9f734e`. The worktree was clean at that boundary. Roadmap item
33 was already occupied by the independent direct-participation taste question,
so this campaign takes item 34 without renumbering or absorbing that decision.

## Mission

Make the platform cursor a projection of the same resolved ordinary primary
press that runtime would execute at the visible pointer position.

A cursor must never advertise text participation, resizing, or another
specialized interaction when the current press would instead be inert, select
only a containing row, or operate on different chrome. Resolution consumes the
last successfully presented geometry, retained pointer position and physical
surface, current modifiers, capture, target and frame meaning, selection and
pre-gesture focality, and deterministic participation policy. Cursor selection
and pointer execution consume that one result.

## Naming ruling

- Private `ResolvedPress` is the read-only runtime answer to "what would an
  ordinary primary press at this visible point currently attempt?"
- Private `PressAdmission` names deterministic runtime permission. It must not
  reuse public declarative `view::Participation`.
- Existing private `PressIntent` remains the activation-versus-manipulation
  classification.
- The actual indication remains `pointer::Cursor`.
- No `Plan`, `Operation`, `PointerCapability`, `PointerAffordance`, `CursorCue`,
  `PointerBehavior`, generic `Axis`, or new generic `State`, `Snapshot`,
  `Surface`, `Frame`, or `Presentation` is introduced.
- Existing overloaded-name cleanup remains independent. `ResizeHorizontal`
  remains exact until another demonstrated resize direction earns vocabulary.

## Constitution

- The cursor projects the ordinary primary press under current modifiers;
  secondary-click grammar remains separate.
- Presented geometry is the spatial truth. Candidate composition cannot affect
  cursor or pointer execution.
- Cursor projection and pointer-down call the same resolver over the same
  categories of truth. No cursor-specific semantic reconstruction survives.
- Targets retain identity, not contextual permission. Admission belongs to the
  resolved press, never `interaction::Target`.
- A non-focal selectable row, a selected-but-not-focal row, or a selection
  modifier admits row selection/focality only and projects no member cursor.
- Text means the admitted press can place or drag a caret or text selection;
  read-only selectable text qualifies, painted labels do not.
- Position, physical surface, and modifiers are retained pointer facts.
  Modifier changes re-resolve without requiring motion, redraw, or rebuild.
- Capture preserves cursor meaning resolved at press time; underlying hover no
  longer determines the cursor during manipulation.
- Hover resolution never executes parsing, validation, command-state
  resolution, commit recipes, or application work. Fallible task departure is
  attempted exactly once by the real gesture.
- Applications cannot assign arbitrary cursors to nodes or primitives. A new
  cursor variant requires a real resolved-press species, precise semantic
  criterion, execution and capture witnesses, and platform adaptation.

## Checkpoints

| Checkpoint | State | Boundary |
|---|---|---|
| 0. Reductions, naming census, and baseline pins | Complete | Ledger, roadmap, current failures, behavior matrix, and structural absences before production edits |
| 1. Name and migrate one resolved press | Complete | Private `ResolvedPress` / `PressAdmission`; current behavior preserved; scattered cursor helpers retired |
| 2. Pointer execution consumes the resolved press | Complete | Target, task focus, row gesture, admission, intent, overlay relationship, and capture share the one answer |
| 3. Modifiers become pointer truth | Complete | Retained modifiers and stationary parent/popup re-resolution without presentation |
| 4. Cursor consumes press admission | Pending | Selection-only rows project Default; focal admitted selectable text projects Text |
| 5. Future seam, doctrine, and closure | Pending | Architecture witnesses, master doctrine, full ritual, roadmap close-out |

## Checkpoint 0 reductions

| ID | Opening reduction | Required postcondition |
|---|---|---|
| ORP-R01 | A non-focal editable table cell shows Text although its click selects/focalizes the row only. | Default until the row is focal; Text only after a successful presented focal-row state. |
| ORP-R02 | A selected-but-not-focal cell shows Text although its click changes focality only. | Default until the row was focal before the gesture. |
| ORP-R03 | Shift, Ctrl, and platform-primary selection gestures show Text while suppressing member participation. | Default while the selection modifier is held, including without pointer motion. |
| ORP-R04 | Platform `ModifiersChanged` is stored below runtime and cannot refresh a stationary cursor. | Runtime retains modifiers and re-resolves parent and native-popup hosts without redraw. |
| ORP-R05 | Presentation refresh repeats the role-only cursor helper. | Successful presentation re-resolves the same prospective press. |
| ORP-R06 | Captured text and divider drags reconstruct Text/ResizeHorizontal from `Target::kind`. | Capture retains the cursor resolved at press time. |
| ORP-R07 | Invalid-indicator hover is correctly Default while its owning text surface is independently targetable. | Indicator/chrome remain Default and owning admitted text retains Text. |
| ORP-R08 | Visible overlay clipping and native-popup hosting already resolve the correct physical hit/host. | Shared press resolution preserves both boundaries without popup-specific semantics. |

### Checkpoint 0 receipts

- `git status --short` was empty at ignition; HEAD was `7f9f734e`.
- The opening source census found the two cursor-only predicates
  `pointer_cursor_for_hit` and `hit_promises_text_edit`, presentation-time reuse
  of the first, and captured text/resize reconstruction from target kind.
- No rejected campaign name existed in production. No cursor assignment exists
  on `view::Node`, `layout::Frame`, or `interaction::Target`; the similarly
  named `TextBox::cursor` is text-buffer caret projection, not a pointer cursor.
- `git diff --check` and `cargo fmt --all -- --check` passed.
- `cargo check --all-targets` passed without warnings.
- `cargo test --lib` passed: 1,052 passed, 10 intentional deep-tier ignores,
  0 failed.
- `cargo test --doc` passed: 1 ordinary and 3 compile-fail doctests.
- `text_editor`, `control_gallery`, and `glass_tuner --smoke` exited 0.
- `examples/glass_tuner/app/state.rs` still declares
  `comparison_open: true`.

## Required cursor matrix

| Surface/state | Required cursor |
|---|---|
| ordinary editable TextBox | Text |
| editable TextArea | Text |
| read-only selectable TextArea | Text |
| disabled text surface | Default |
| painted label | Default |
| non-focal table text | Default |
| selected-but-not-focal table text | Default |
| focal-row selectable text | Text |
| focal-row text under Shift/Ctrl/Super | Default |
| text scrollbar chrome | Default |
| invalid indicator | Default |
| table divider | ResizeHorizontal |
| captured text drag outside hit area | Text |
| captured divider drag outside hit area | ResizeHorizontal |

### Checkpoint 1 receipts

- Private runtime-owned `ResolvedPress` now owns the hit, deterministic initial
  `PressAdmission`, and logical cursor projection. Pointer move, pointer-down
  preliminary interpretation, pointer up, uncaptured drag, and successful
  presentation refresh all consume it.
- The platform cursor mapping and physical cursor-host path are unchanged.
- `pointer_cursor_for_hit` and `hit_promises_text_edit` are deleted. No
  replacement cursor field was added to view, layout, or target data.
- This migration deliberately preserves the opening table I-beam error;
  selectable-row admission becomes cursor data only after execution and
  capture consume the resolved press.
- `cargo check --all-targets` passed. Five focused cursor tests and the captured
  text-drag cursor witness passed with identical update behavior.

### Checkpoint 2 receipts

- `ResolvedPress` now carries exact target, task-focus destination,
  `VirtualRowGesture`, `PressAdmission`, `PressIntent`, overlay relationship,
  hit truth, captured cursor, and the cursor to project after release.
- Pointer-down consumes those fields in the pinned order: resolve, attempt task
  departure, reject as a whole, apply row selection, stop on `SelectionOnly`,
  classify the click, construct the ordinary target action, then establish
  press/capture state.
- The one resolved row gesture supplies both selection mutation and admission;
  no second participation computation was introduced.
- Crate-private pointer actions carry the resolved cursor into the session.
  `Capture` retains that value beside target identity. Drag and pointer-leave no
  longer infer Text or ResizeHorizontal from `Target::kind`; pointer-up projects
  the same resolver's post-release cursor.
- Existing public raw `input::Input` pointer helpers receive Default and cannot
  assign an application-selected cursor. The logical value remains private to
  runtime action routing.
- All-target compilation passed without warnings. Cursor, text-capture,
  divider-capture, rejected-departure, selection-before-participation, and the
  full 1,052-pass library suite remained green.

### Checkpoint 3 receipts

- Existing `interaction::Pointer` now retains `input::Modifiers` beside point
  and physical surface; no `PointerState` or `PointerSnapshot` was introduced.
- Native `ModifiersChanged` now produces a host event instead of ending below
  runtime. Parent and native-popup adaptation keep distinct routing witnesses,
  while both update the one parent-session pointer fact.
- Shell routing re-resolves the stationary point against the retained physical
  surface and last-presented layout. It emits only a deduplicated cursor update;
  it performs no view action, redraw, layout, rebuild, or synthetic movement.
- Pointer-down installs its event modifiers before resolution. Move, drag, up,
  leave, and successful-presentation refresh consume the retained value.
- All-target compilation passed without warnings. The cursor difference under
  selection modifiers becomes observable at Checkpoint 4 when cursor projection
  starts consuming the already-resolved admission.

## Structural-absence contract

- no cursor field on `view::Node`, `layout::Frame`, or `interaction::Target`;
- no public application cursor setter;
- no rejected naming vocabulary from the naming ruling;
- no logical cursor decision from raw `Role` or `Target::kind` outside the one
  resolver;
- no hover-time commit, validation, parsing, or command resolution;
- no table-local cursor workaround or second row-participation computation;
- no candidate-geometry cursor resolution, popup-specific cursor semantics,
  redraw-only cursor update, or speculative cursor variant.

## Execution order

Pointer-down must preserve the established transaction order:

1. resolve from presented geometry;
2. attempt required task departure;
3. reject the entire gesture if departure fails;
4. apply row selection/focality;
5. stop if admission is selection-only;
6. classify the click;
7. enter the target's ordinary action path;
8. establish press/capture state.

## Acceptance matrix

| ID | Required witness |
|---|---|
| ORP-01 | Cursor and pointer-down resolve through one `ResolvedPress` owner. |
| ORP-02 | Resolution consumes last-presented geometry, never a candidate. |
| ORP-03 | Non-focal table text uses Default and performs row selection only. |
| ORP-04 | Selected-but-not-focal text uses Default and focalizes only. |
| ORP-05 | Focal-row selectable text uses Text. |
| ORP-06 | Read-only selectable focal text retains Text. |
| ORP-07 | Disabled text, labels, indicators, and text chrome use Default. |
| ORP-08 | Shift, Ctrl, and Super change focal-row text to Default while held. |
| ORP-09 | Stationary modifier press/release updates cursor without redraw. |
| ORP-10 | First-click focalization changes Default to Text after presentation without motion. |
| ORP-11 | Selection-only presses contribute nothing to the text click chain. |
| ORP-12 | Captured text drag retains Text outside the original hit. |
| ORP-13 | Captured divider drag retains ResizeHorizontal outside the divider. |
| ORP-14 | Capture does not reconstruct cursor from `Target::kind`. |
| ORP-15 | Overlay clipping prevents hidden text leaking an I-beam. |
| ORP-16 | Parent and native-popup hosts receive the same logical truth. |
| ORP-17 | Cursor-only changes deduplicate and produce no presentation. |
| ORP-18 | Hover executes no parsing, validation, command resolution, or commit. |
| ORP-19 | Rejected task departure admits no dependent action or click-chain contribution. |
| ORP-20 | Application-authored nodes, targets, frames, and primitives have no cursor assignment. |
| ORP-21 | Every cursor variant has a demonstrated resolved-press caller. |
| ORP-22 | Searches find no rejected names or retired cursor predicates. |

## Required non-merges

- runtime `PressAdmission` versus declarative `view::Participation`;
- `PressIntent` versus `Cursor`;
- target identity versus current admission;
- text selectability versus editability;
- pointer modifiers versus keyboard command routing;
- hover versus capture;
- deterministic admission versus fallible task departure;
- logical cursor value versus physical platform cursor host;
- framework interaction meaning versus raw winit cursor variants;
- this campaign versus stacking contexts and overloaded-name cleanup.

## Completion theorem

The campaign is complete when, at every visible pointer position, runtime
resolves one prospective primary press from presented geometry and retained
interaction facts. Its admission determines whether the exact target may
participate; its cursor projects that same answer; pointer-down consumes it;
capture preserves it. Selection-only rows cannot advertise text, modifier
changes cannot leave stale indication, and future cursor species enter through
one semantic press resolver rather than scattered role checks.
