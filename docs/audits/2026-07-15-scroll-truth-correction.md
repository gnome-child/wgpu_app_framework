# Scroll Truth correction — One Position, Complete Pixels

Status: **active**. This correction follows the completed retained-renderer
campaign and starts from production HEAD `66c84945`. It owns the scroll,
virtual-residency, scene-property, and pending/active presentation seams until
the exit theorem below is proved.

This is an architectural correction, not a compatibility patch. Public APIs
and internal call sites may move when the practiced ownership model requires
it. Existing popup/menu corrections in the starting worktree remain protected
and are verified and published independently from the scroll mechanism.

## Indictment

The framework owns one logical session offset but projects it into resident
layout, scene properties, and pending/active presentation without one atomic
admission contract. Three invalid states are consequently representable:

1. an integral logical offset is narrowed to `f32`, then rounded back to a
   different integer at large content extents;
2. a layout/commit may be drawable without proving that every requested
   virtual row and every viewport pixel is resident;
3. an older prepared semantic state may activate after a newer scroll state
   exists, visibly restoring the older offset.

Wheel input and scrollbar-thumb input mutate the same session owner but pass
through duplicated transition logic, allowing their scheduling behavior and
event receipts to diverge.

## Constitution

> Scroll has one authoritative state. Every materialization, property sample,
> scrollbar, hit projection, activation, and receipt is derived from it.

> A scrollable presentation is drawable only when its immutable structure
> proves complete pixel coverage for the integral property value it presents.

> Activation may change resident structure; it may never regress the newest
> accepted scroll property.

These specialize the retained-renderer laws rather than replacing them:
structure remains in the commit, values remain in property state, and
presentation owns activation. The correction changes the contracts that let
those clocks disagree without an admission proof.

## Ownership and representation

### Authoritative state

`interaction` / `session` owns scroll mutation. Relative wheel deltas,
absolute thumb positions, programmatic scrolling, and layout-derived extent
corrections enter one typed transition and produce one resulting integral
`ScrollOffset`. Runtime scheduling consumes that result; it does not duplicate
mutation policy.

For stable extents, the authoritative position is the integral offset. For
provisional variable-height geometry, a stable row anchor and its within-row
displacement may be used to resolve the same logical position after a geometry
revision. That anchor is a resolution rule, not a second offset authority:
virtual layout proposes the correction and the session owner alone applies it.

### Integral property law

Logical scroll coordinates remain `ScrollOffset` / `i32` through interaction,
layout, scene properties, projection, validation, activation, and receipts.
The renderer may convert only the bounded difference between a commit baseline
and the active offset to GPU `f32`. Total content extent never crosses that
floating-point boundary.

### Exact residency law

Virtual residency is a proved snapshot, not a bounding-box inference. The
layout snapshot binds:

- the existing composition identity of the scroll owner;
- the requested contiguous index range;
- the actual realized row identities and indices;
- ordered row rectangles and their contiguous pixel coverage;
- the viewport, integral baseline offset, and admitted property interval.

No independent materialization identity is minted. The request and realized
rows are checked in the same immutable layout snapshot, and the resulting scene
commit revision is the existing structural revision currency.

A virtual snapshot is complete only when every requested index occurs exactly
once, row geometry is ordered and gap-free across the visible axis, and the
baseline viewport is wholly covered. Distant focus/menu pins are retained
content but cannot contribute to contiguous residency.

An incomplete snapshot is `NotReady`. It cannot silently omit a scroll
declaration, create a drawable scene commit, update presented geometry, or
activate. Reaching the refinement limit with incomplete coverage is a rejected
candidate, not a warning followed by presentation.

### Monotonic activation law

When a prepared state `B` completes while a newer semantic state `C` exists,
presentation must first project the newest property values onto `B`.

- If every newest scroll value is declared and accepted by `B`'s proved
  residency, `B` may activate using those newest values while `C` continues
  preparing.
- If any newest scroll value is not accepted, `B` does not activate. The
  complete active state remains drawable and `C` becomes the preparation
  target.

Generic best-effort property projection is insufficient for this decision: a
missing or rejected scroll value is an activation veto. Successful presentation
receipts bind the exact layout and rebased property snapshot that drew.

### Unified input law

Wheel, touch/trackpad deltas, scrollbar tracks/thumbs, and programmatic absolute
scroll requests share one transition result:

```text
Unchanged | PropertyTick(offset) | NeedsResidency(offset)
```

Horizontal and vertical scrollbars consume the same axis-generic geometry,
capture, mutation, and scheduling path. Axis-specific arithmetic may select a
component; it may not select different ownership or presentation physics.

## Checkpoints

### 0. Protect starting work and ratify the contract

- verify and publish the already-confirmed retained popup/menu-origin fix;
- retain or deliberately replace each uncommitted scroll correction;
- plant this formulation and focused structural witnesses.

### 1. Preserve integral scroll truth

- replace floating scene scroll values with `ScrollOffset`;
- convert only bounded renderer deltas to `f32`;
- cover offsets on both sides of `2^24`, the 24,000,000-pixel gallery extent,
  and exact lower/upper resident boundaries.

### 2. Make complete residency constructible and incomplete residency rejected

- introduce exact virtual coverage validation at layout ownership;
- require a valid coverage snapshot for every virtual scroll declaration;
- reject non-converged/incomplete candidates before scene painting;
- derive guard rows from viewport/cost bounds rather than a two-row magic
  default while retaining bounded nodes for one million rows.

### 3. Make activation scroll-monotonic

- strictly rebase newest scroll properties onto an older prepared structure;
- veto stale activation when its residency cannot accept them;
- retain the complete active state while the newest compatible state prepares;
- receipt only the property/layout pair actually drawn.

### 4. Unify scroll transitions and close the old paths

- route relative and absolute input through one session transition;
- delete duplicated runtime scheduling logic;
- prove horizontal/vertical scrollbar capture and mutation parity;
- restore variable-height anchoring as an explicit geometry-resolution rule,
  not an implicit competing writer.

### 5. Runtime and burn-down

- run focused semantic, architecture, and pending/active tests;
- run workspace all-target, doctest, renderer-debug, and deep GPU tiers in
  proportion to changed code;
- repeatedly launch the release Control Gallery and exercise slow/fast wheel
  scrolling, large thumb jumps, both scrollbars, column resizing, typing, and
  selection while checking process survival;
- remove temporary diagnostics and displaced rectangle-only/float/duplicated
  mechanisms;
- commit and push each coherent checkpoint.

## Required witnesses

The correction cannot close without deterministic witnesses for:

1. exact integral round trips at `16_777_215`, `16_777_216`, `16_777_217`,
   representative odd values near 24,000,000, and the gallery maximum;
2. missing, duplicate, out-of-order, and gap-producing virtual rows yielding
   `NotReady` and no scene commit;
3. a complete requested range covering every viewport pixel;
4. a distant pin being excluded from contiguous residency;
5. `active A -> preparing B -> latest C` activating `B` only with `C`'s scroll
   value when accepted, and otherwise preserving `A` while preparing `C`;
6. no successful receipt whose scroll property serial/value regresses;
7. relative and absolute requests for the same resulting offset producing the
   same scheduling transition;
8. vertical and horizontal scrollbar hit/capture parity;
9. one-million-row resident node/resource bounds and a crash-free large-thumb
   release-gallery run.

Tests that inspect only a union rectangle, one non-clear pixel, settled output,
or an offscreen renderer without the native pending queue are insufficient.

## Exit theorem

The goal is complete only when:

1. scroll coordinates remain integral until a bounded renderer-local delta;
2. every drawable virtual scroll commit carries exact, complete residency;
3. incomplete or non-converged residency cannot enter scene painting or
   activation;
4. pending/active activation cannot regress the newest accepted scroll value;
5. all scroll inputs and both axes share one mutation/scheduling contract;
6. variable-height correction has one explicit anchor-resolution path;
7. presented geometry, scrollbar geometry, hit testing, and renderer output
   consume the same receipted property state;
8. the required deterministic, workspace, renderer, and real release-gallery
   witnesses pass; and
9. the old lossy, rectangle-only, stale-activation, and duplicated transition
   paths are absent.

