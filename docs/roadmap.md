# Roadmap

Tracked future work, pruned in place as items land. Sibling to
`master_design.md`: that document records what is true; this one records what
is next.

## In flight

## Specced and ready

## Pending manual verification

5. Popup-hosted cursor icon and IME candidate placement, fractional-alpha edge
   quality after the premultiplied audit, palette query editing feel (selection
   tint, caret, I-beam) after the scope slice. Tables campaign eyes: ellipsis
   glyph/cut spacing in both
   themes, striping and rule weight, sticky-header feel, divider resize feel
   and cursor, selection tint and keyboard-extend feel, editor placement,
   rejection outline presentation, commit/cancel feel. Outstanding: post-fix
   `key -> present` panel reading from the Mac latency incident.

## Decisions awaiting product taste

7. **Dirty-document confirmation flow** — one destructive-intent coordinator
   (Save / Don't Save / Cancel) around New, Open, OS close, and Exit. Design
   proposed; awaiting product decision.
8. **Enter-curve on long fades** — the ease-out tail reads as a stall at
   cinematic durations; curves are designed for duration regimes. Only
   relevant if long fades ever ship.
24. **Escape with a non-empty palette query** — immediate dismissal (current)
    vs clear-query-first, dismiss-second (common palette UX).
25. **Panel layout contract** — `Panel::row/column/overlay` silently converts
    the node to Stack, dropping Panel's surface presentation (widget grammar
    audit R-11). Decide whether Panel is a persistent surface or Element's
    default column form; the contract has no whole witness today.

## Named arcs

10. **Tables arc — v1 COMPLETE** (`d5082175`, 2026-07-13; 906 tests). Six
    campaigns end to end: tables, grammar, polish, five truths, one text
    truth, one selectable truth — ledgers in `docs/audits/`. Final capability
    boundary is std: `Display` is citizenship, `FromStr` editability, `Ord`
    sortability, `bool` conversions the toggle medium; zero framework
    capability traits. Original checkpoints:
    1. Text overflow (complete at `dfa728f2`; item 9 pruned).
    2. **`FrameContent` decomposition** (complete at `35736441`) — one `Frame` keeps common
       geometry/identity/clip/presentation; role payloads become a typed
       content enum (the `view::Node`/`Control` idiom), over the existing
       17 roles. Success condition: incompatible payload combinations are
       unrepresentable while all roles keep identical behavior. Cashes the
       Examen R-02 flag.
    3. **Virtual region/list** (complete at `f223d454`) — the provided-container species, v1 flat:
       uniform row height, stable provider keys, overscan, jump scrolling,
       provider shrink/reorder, bounded materialization. Doctrine:
       dematerialization is not removal. Pinning rule: focused, captured,
       or actively edited rows stay materialized (may be clipped); selected
       rows do NOT pin. Drafts survive ordinary scrolling and die on actual
       row deletion. Complexity witnesses: one million logical rows produce
       bounded nodes, frames, paint items, and work per scroll.
    4. **Keyed selection + active item** (complete at `cd91ad95`) — separate state machine from
       virtualization: anchor/extend, reorder persistence, departure,
       keyboard navigation that can target an unmaterialized row and
       materialize it before focus moves.
    5. **Read-only record table** (complete at `d7d8cd98`) — track layout with explicit/weighted
       widths; resizing owns presentation state independently of provider
       data; headers; sorting emits intent (the table never reorders
       application data); striping, rules, truncation; cells host public
       widgets — Table must not become a giant specialized leaf.
    6. **Editable cells** (complete at `2daf1ab7`) — typed edit policies derived from at least two
       real column types (numeric, textual/enumerated); display formatting,
       text parsing, domain validation, and commit/rejection policy remain
       separable meanings until the evidence converges them.
    Identity doctrine: row identity = provider key; column identity =
    column id; cell identity = (table id, row key, column id) — drafts,
    focus restoration, selection, retained layout, and accessibility all
    key on that tuple. Only visible rows participate in measurement; v1
    never scans the provider for intrinsic column widths. Header/cell
    relationships and logical row/column indices are reserved now for
    AccessKit. Sheet species still waits for a real caller. Items 25/26 do
    not block: a table is a provided container, not a custom leaf.
11. **Accessibility (AccessKit)** — after tables; seams reserved
    (`composition::Changes::removed_elements`, subject labels, roles,
    active-item concept). The widget grammar audit added the missing field
    concept: semantic label/description/error association (`label-for`),
    demonstrated by the compound labeled-field experiment. Feedback reserves
    direct Description, DescribedBy, Invalid, ErrorMessage, and future Live
    projections without making accessibility depend on panel visibility.
12. **Music player** (flagship one) — remaining framework blockers: image /
    texture primitive, file drag-and-drop, media keys / SMTC. Cleared: task
    executor, native menus, async atomic saves, virtualized editable tables
    (campaign closed at `bc4df416`), meaning-derived menu bars (`33ab1cc4`).
13. **Trading terminal** (flagship two) — charts primitive domain, real-time
    invalidation stress, tabular figures, kiosk-scoped BSD session.
28. **Presentation Clock — COMPLETE** (`20c31cae`, 2026-07-13; 939 tests at
    the last behavioral boundary). Events update truth immediately; redraw is
    the frame boundary; successful presentation receipts alone promote visible
    geometry; input targets that geometry; pointer position derives hover; and
    table widths are layout-transient. One hundred divider positions now cause
    zero view rebuilds and one latest-state frame. The Vulkan / DX12-Visual /
    DX12-HWND matrix acquits `DxgiFromVisual`; DX12 amplifies the remaining
    renderer cost but does not own it. Ledger:
    `docs/audits/2026-07-13-presentation-clock-campaign.md`.
29. **Current Context — COMPLETE** (`c3bb7673`, 2026-07-13; 1,002 ordinary
    tests and 10 deep-tier witnesses). One popup realization owns presentation
    and interaction geometry; popup-local generations expose only current
    content; directional responder paths derive grouped table context over the
    existing keyed multiselection, focal row, and exact facet. Authored and
    contextual menus share one retirement/z-order lifecycle. Ledger:
    `docs/audits/2026-07-13-current-context-campaign.md`.
30. **Menus From Meaning — COMPLETE** (`33ab1cc4`, 2026-07-13; 1,025 library
    tests + 10 deep-tier witnesses; per-checkpoint commits). Registration is
    the source of menu vocabulary: `Spec::standard` roles derive labels,
    platform chords, chord-display policy, and cultural topology;
    `command::menu::{Category, Placement}` place static deviations with
    virtual-slot anchors; one `command::Population` owner discovers,
    resolves, and composes candidates for bar, context, and palette under
    distinct surface policies (bar resolves live at activation; context
    keeps captured routes). `ui.standard_menu_bar()` replaced 19
    conventional rows, 6 separators, and 4 literal chords across the
    examples; authored menus remain, derivation is opt-in. Ledger:
    `docs/audits/2026-07-13-menus-from-meaning-campaign.md`.
31. **Feedback — COMPLETE** (`fc4927a0`, 2026-07-13; 1,041 library tests +
    10 deep-tier witnesses). Stable command descriptions, contextual hints,
    severity-ranked runtime facts, table rejection reasons, and confirmed
    text overflow now resolve independently into one auxiliary-panel policy.
    Every resulting panel consumes the existing placement, host, receipt,
    fade, and exposure path; noninteractive policies are focus-free and
    hit-transparent. `OverflowProjection` owns eligibility without
    remeasurement, and rejected edits rebuild their explanatory panel on the
    rejecting input itself. Ledger:
    `docs/audits/2026-07-13-feedback-campaign.md`.
32. **One Text Task — COMPLETE** (`d4a81a7c`, 2026-07-13; 1,052 library tests +
    10 intentional deep-tier ignores; per-checkpoint commits). Editable table
    cells are ordinary `TextBox` tasks behind one
    selection-before-participation gate. Fallible commits resolve from the
    current draft exactly once, retain rejection with that draft, and project
    invalidity inline; rejected departure admits no dependent action. The
    table edit identity, rejection store/panel route, forced text modes,
    duplicate parsing, and editing-scope command suppression are deleted. The
    optional status-projection tail was explicitly trimmed after the green core
    boundary and has no partial implementation. Ledger:
    `docs/audits/2026-07-13-input-validity-status-projection-audit.md`.

## Deferred until a caller or hardware appears

15. **Local blur** — unblocked by the premultiplied audit; build when
    something wants it.
16. **macOS popup realization** — nonactivating panel shim, semantic AppKit
    materials, `NSVisualEffectView.state = .active` (the focus-coupling
    lesson, pre-applied). Needs macOS hardware.
17. **Linux popup realization** — KWin/Hyprland blur hints, best effort;
    Wayland remains in-frame by design.
19. **Fade overflow style and marquee** — need gradient/mask infrastructure
    and the text-in-motion policy decision respectively.
20. Micro-parked: density presets, menu mnemonics, user rebinding,
    reveal-margin theme datum, Mac Home/End viewport scroll, Ctrl+A/E field
    bindings, SPDX header sweep + architecture witness via
    `tools/license_headers` micro-crate (one commit at the next quiet tree,
    alongside the push).
26. **Application-authored semantic leaf controls** — the widget catalog is a
    good structural grammar but a closed semantic leaf catalog: apps cannot
    author a new keyboard-focusable, themed, accessibility-ready control
    without an internal role (~14 coordinated touch points). Decide openness
    when the music player demands its first custom control (seek bar with
    buffering, rating); until then closure is coherent and safer than a
    property bag.

## Watch items

22. Documented non-merges, revisit only on evidence: scene-transform
    sanitization duplicate, the two `AnyTarget` shapes, equal cache
    capacities and transition durations.
23. Process gauges: suite harness runtime (~1s target, distinct from Cargo
    wall time), push cadence, periodic cold external audits.
