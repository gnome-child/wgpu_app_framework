# The Composed Context World

Status: in flight.

## Mission

Correct contextual discovery so the exact member under the pointer contributes
its intrinsic commands while the nearest explicitly contextual object supplies
the containing command boundary. Reuse the retained composition ancestry,
`Scope::contextual`, `Candidates<Local>`, the one command resolver, the shared
menu session, ordinary menu rows, separators, placement, and popup realization.

No second scope hierarchy, stored ancestry path, public erasure, or authored
context-menu recipe is admitted.

## Protected baseline

Campaign ignition follows `a110cee5` (`Complete derived context menu campaign`).
The pre-existing modified files reported by `git status --short` at ignition
remain protected. In particular, the gallery's 500-pixel table height and the
completed renderer-roadmap transition are not campaign changes. No push.

## Laws

1. Context is bounded composition: intrinsic members contribute inward; the
   nearest explicit boundary stops outward discovery.
2. Exact hit and owning object are separate truths.
3. Selectable text participates automatically; painted labels and control
   captions do not.
4. Opening is observational: no focus, selection, edit-mode, or focus-chrome
   mutation.
5. Membership follows capability; availability remains live.
6. Discovery and invocation retain the same exact route.
7. Composition ancestry remains the source; no `context::Path` is stored.

## Census

| Species/callsite | Evidence | Verdict |
|---|---|---|
| `TextArea` / `TextBox` with an exact focus and non-disabled mode | Both already project through the text service and `Scope` already separates `responder` from `focus` | Intrinsic text member; no marker required |
| Bound `Checkbox`, including typed Boolean table cells | One concrete toggle trigger already exists on the leaf | Intrinsic toggle member; no marker required |
| `Button` and generic `Binding` | The ordinary binding is a primary action, not evidence of a secondary contextual action | Explicit boundary only |
| `Radio` | A choice belongs to its group; repeating the primary selection action is not a contextual meaning | Not intrinsic |
| `Slider` | Its binding requires a pointer-derived value and has no stable context action | Not intrinsic |
| `Label`, captions, decorative text | No selectable source mapping or text focus exists | Not intrinsic |
| Text commit binding | Submission/commit plumbing is not a menu command | Excluded even when the text surface participates |
| `TypedColumn::context_menu` | Only current caller marks a bound Boolean cell | Redundant; delete after intrinsic checkbox participation lands |
| Gallery checkbox and search wrappers | Both wrap intrinsic species | Redundant; delete |
| `Table::context_rows<C>` | Captures a typed row-object command from stable identity | Keep as explicit row boundary |
| Generic `widget::context_menu` roots/panels/bindings | Declares the bounded object whose targets may be projected | Keep and document as boundary |

## Checkpoints

| # | Boundary | Status |
|---|---|---|
| 1 | Census and constitution | Complete |
| 2 | One retained context world | Pending |
| 3 | Grouped local candidate composition | Pending |
| 4 | Exact text targeting and focus law | Pending |
| 5 | Existing grouped presentation | Pending |
| 6 | API deletion, gallery migration, witnesses, doctrine, closeout | Pending |

## Structural-absence conditions

Closeout fails if the tree contains a second resolver or menu session, stored
context ancestry, layout-to-registry access, global contextual candidates,
public erased command types, text-specific menu painting, or
`TypedColumn::context_menu`.

## Final doctrine

The nearest boundary owns the command world, not every command inside it.
Intrinsic members contribute capabilities within that boundary; they do not
replace it.
