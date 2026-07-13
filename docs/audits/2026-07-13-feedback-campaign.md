# Feedback campaign

Status: in flight (2026-07-13).

## Constitution

- `command::Spec::description` is stable meaning; `command::State::hint` is a
  contextual claim-time explanation. Neither populates the other.
- Feedback is a retained runtime fact: severity plus eagerly formatted text.
  Its typed store owns anchor identity and lifetime.
- The element-level auxiliary-content resolver consults independent owners in
  this order: Error, Warning, Info, hint, description, confirmed overflow.
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
| F-01 | Rejected table edit traps focus without explaining why. | Open |
| F-02 | Runtime command explanations carry truth but produce no pixels. | Open |
| F-03 | Ellipsized table text cannot reveal its complete source. | Open |
| F-04 | Fully visible text must never produce an overflow tip. | Open |
| F-05 | Competing feedback, hint, description, and overflow produce one winner. | Open |
| F-06 | Auxiliary panels never focus, capture, or outlive their anchors. | Open |
| F-07 | Panel content cannot resize after placement is selected. | Open |

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
