# Roadmap

Tracked future work, pruned in place as items land. Sibling to
`master_design.md`: that document records what is true; this one records what
is next.

## In flight

(nothing)

## Specced and ready

2. **Show-cycle presentation contract** — the first visible frame of every
   popup show cycle must be a freshly presented current frame. Fixes the
   stale-swapchain silhouette on reused popup windows and is expected to close
   the unresolved first-frame skip recorded in `master_design.md`.
4. **Context menu** — right-click → hit-scoped claims → derived command menu
   at the pointer, on the native popup backend. The popup arc's original
   payoff feature.

## Pending manual verification

5. Popup fade consistency (after item 2), popup-hosted cursor icon and IME
   candidate placement, fractional-alpha edge quality after the premultiplied
   audit. Outstanding: post-fix `key -> present` panel reading from the Mac
   latency incident.
6. Local visual-test edits in `examples/glass_tuner/app/state.rs` — revert
   when tuning sessions end.

## Decisions awaiting product taste

7. **Dirty-document confirmation flow** — one destructive-intent coordinator
   (Save / Don't Save / Cancel) around New, Open, OS close, and Exit. Design
   proposed; awaiting product decision.
8. **Enter-curve on long fades** — the ease-out tail reads as a stall at
   cinematic durations; curves are designed for duration regimes. Only
   relevant if long fades ever ship.

## Named arcs

9. **Text overflow** — `Overflow::{Clip, EllipsisEnd, EllipsisMiddle}`, the
   three-kinds-of-text doctrine (author text must fit; world text declares
   overflow; user text scrolls), a required-overflow world-text node, and the
   inline cache `TextKey` amendment. Prerequisite for tables.
10. **Tables** — track layout, lazy view regions/virtualization (uniform row
    height v1), record-table v1 (sticky sortable resizable headers, row
    multi-selection, striping, rules, truncation), cell editing via drafts,
    sheet species when a real caller arrives. Row providers carry total count
    and stable row identity as the accessibility seam.
11. **Accessibility (AccessKit)** — after tables; seams reserved
    (`composition::Changes::removed_elements`, subject labels, roles,
    active-item concept).
12. **Music player** (flagship one) — remaining framework blockers: image /
    texture primitive, virtualized table (item 10), file drag-and-drop, media
    keys / SMTC. Cleared: task executor, native menus, async atomic saves.
13. **Trading terminal** (flagship two) — charts primitive domain, real-time
    invalidation stress, tabular figures, kiosk-scoped BSD session.
14. **Targeted redraw** — v0.5: overlay/fade frames skip base re-render
    (cheap, ready when profiling asks). Full damage-tracking arc after
    tables.

## Deferred until a caller or hardware appears

15. **Local blur** — unblocked by the premultiplied audit; build when
    something wants it.
16. **macOS popup realization** — nonactivating panel shim, semantic AppKit
    materials, `NSVisualEffectView.state = .active` (the focus-coupling
    lesson, pre-applied). Needs macOS hardware.
17. **Linux popup realization** — KWin/Hyprland blur hints, best effort;
    Wayland remains in-frame by design.
18. **DirectComposition mode** — owned composition visual per popup;
    compositor-side opacity animation (the true whole-window fade, frost
    included).
19. **Fade overflow style and marquee** — need gradient/mask infrastructure
    and the text-in-motion policy decision respectively.
20. Micro-parked: density presets, menu mnemonics, user rebinding,
    reveal-margin theme datum, Mac Home/End viewport scroll, Ctrl+A/E field
    bindings.

## Watch items

21. Unresolved first-frame skip note in `master_design.md` — expected to die
    with item 2; the first-present trace names it otherwise.
22. Documented non-merges, revisit only on evidence: scene-transform
    sanitization duplicate, the two `AnyTarget` shapes, equal cache
    capacities and transition durations.
23. Process gauges: suite harness runtime (~1s target, distinct from Cargo
    wall time), push cadence, periodic cold external audits.
