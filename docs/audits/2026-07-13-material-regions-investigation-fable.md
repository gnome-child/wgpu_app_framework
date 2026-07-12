# Material Regions Investigation — Fable execution — 2026-07-13

Independent execution of the material-regions investigation ledger, for
comparison against Codex's run. Protocol deviation, recorded: the repo tree was
hot with the One Selectable Truth campaign during execution, so probes were
built as a standalone crate in the session scratchpad (`material_probe`,
`windows` 0.61.3 + `windows-numerics` 0.2) rather than under `examples/`. The
repository was not touched until this ledger landed on a clean tree. Probe
sources and captured screenshots remain in the scratchpad.

## Phase A — repo census

| Cell | Verdict | Receipts |
| --- | --- | --- |
| A-01 | In-frame material vocabulary is already region-shaped | `scene/material.rs` exports `BackdropBlur`, `BackdropEdgeMode`, `BackdropLayer`, `Glass`, `Luminosity`, `Material`, `Noise`, `Refraction`; `Glass` at `material.rs:11`, `BackdropBlur` at `:31`, tint accessors at `:153/:219`. Pane field census left partial (hot tree); recipe shape confirmed sufficient for region requests. |
| A-02 | The whole-window seam exists and is the promotion baseline | `PopupMaterialRealization { WindowsAccentAcrylic, TransparentNoAccent, OpaqueFallback }` (`platform/native/mod.rs:80`) — the three-tier report already exists at window granularity. `realization_for` at `:268`; `native_popup_scenes` at `scene/mod.rs:146`; `overlay::PopupPresentation { parent, id: interaction::Id, bounds, scene, opaque_fallback_scene }` (`overlay.rs`) carries per-popup identity and the residual-scene pair. |
| A-03 | Settle machinery generalizes by type parameter; region keys exist | `SysApplicator<T>` instances: `PopupGeometryState`, `PopupAccentState`, `PopupBorderState` (`native/mod.rs:96-98`); `POPUP_SYS_SETTLE_DELAY = 150ms` (`:181`); presence/parameter split via `accent_presence` (`:206`). Region identity precedent: `interaction::Id` per popup entry; per-pane keys would extend the same species. |
| A-04 | Composition presentation is wgpu-internal, classic DComp | Crate side: `wgpu::Dx12SwapchainKind::DxgiFromVisual` set as Windows default (`render/context.rs:75,144,159`, pinned by `tests/architecture.rs:2221`). wgpu-hal 29.0.3 side: `DCompositionCreateDevice2` + `IDCompositionDevice::CreateTargetForHwnd(hwnd, false)` + internally-owned `IDCompositionVisual` (`wgpu-hal/src/dx12/dcomp.rs:42-101`); `CreateSwapChainForComposition` (`dx12/mod.rs:1388`). No DComp objects exist in this repo — the embassy's embassy is wgpu. |
| A-05 | Capability vocabulary is one bit today | `Capabilities { native_popups: bool }` (`overlay.rs:116`); `resolve_backend` (`overlay.rs:400`). Tier vocabulary (Full/Frost/None) must be grown here; forecast-vs-outcome separation is not yet representable. |
| A-06 | The all-`None` unification is shape-compatible | `PopupPresentation` already carries `scene` + `opaque_fallback_scene` — the residual-assembly pair. Desk-level verdict only; the decisive check (in-frame path expressed as all-`None` realizations) awaits the campaign census. |
| A-07 | Retirement inventory | `CornerPreference` / `DWMWA_BORDER_COLOR` / `undecorated_shadow` / `ACCENT_ENABLE_*`: six call/decl sites in `platform/native/sys/windows.rs`, three in `platform/native/window.rs`. All become deletable under an owned visual tree except the accent fallback tier. |

## Phase B — Windows probes (hardware: Win11, this machine, 2026-07-13)

Probe: one process, pattern owner window + NOACTIVATE|TOOLWINDOW|TOPMOST
subject popup + timed foreground thief. Subject paints nothing (validated
empty WM_PAINT, WM_ERASEBKGND=1). Modes: `host`, `accent`, `accent5`, `clip`,
`fade`. Artifacts: `host_active.png`, `host_stolen.png`, `clip.png`,
`fade_a/b/c.png` (scratchpad).

| Probe | Verdict | Evidence |
| --- | --- | --- |
| B-01 focus independence | **CONFIRMED, strongest form** | The probe process never held foreground at all (launched from a background shell; its windows spawned behind the active app). `HostBackdropBrush` + `DWMWA_USE_HOSTBACKDROPBRUSH` produced live frost on the never-activated subject in both captures — before and after the in-process foreground steal. Frost is fully decoupled from activation on this machine. The composition route survives its gating question. |
| B-02 pre-blurred or raw | **CONFIRMED: pre-blurred** | The probe attached the bare brush with zero effect chain; captured frost shows strong gaussian-class blur of the app behind (text illegible through the region). Acrylic-lite therefore needs no `IGraphicsEffect` COM: backdrop brush + our tint sprite suffices. Full-material fidelity (saturation/luminosity/noise) remains the only consumer of the effect chain. |
| B-03 tree ownership | **CONFIRMED CONSTRAINT + fork identified (source census; live probe pending)** | wgpu owns a classic-DComp target with `topmost=false`; a second same-flag target on one HWND fails by DComp rule, and a WinRT `DesktopWindowTarget` is such a target. Fork: (i) **works today** — hand wgpu an externally-owned visual via `SurfaceTarget::Visual` (`wgpu-hal/dx12/mod.rs:614`, classic-typed) and build the whole tree classic — but backdrop brushes are WinRT-only, so frost dies; (ii) **upstream ask** — wgpu accepts a WinRT visual or exposes the swapchain for `ICompositorInterop::CreateCompositionSurfaceForSwapChain`; (iii) **two-window pair** — frost HWND beneath content HWND, geometry synced by the existing settle applicator (works today, no upstream); (iv) bonus: a `topmost=true` WinRT target CAN coexist above wgpu's — insufficient for frost (wrong side) but a candidate OS-side host for above-content late chrome. Campaign must choose (iii) now vs (ii) upstream. |
| B-04 clip fidelity | **CONFIRMED at current scale** | `CompositionRoundedRectangleGeometry(24px)` clip on the frost visual renders smooth antialiased corners (`clip.png`). Four-scale matrix not exercised — pending. |
| B-05 compositor-side fade | **CONFIRMED at capture granularity** | Looping `ScalarKeyFrameAnimation` on root `Opacity`: captures ~1.4s apart show frost at partial opacity then absent — the material itself fades, not merely content, with the probe's thread asleep in `GetMessage` (zero app-side frames). Frame-cadence smoothness unmeasured — pending instrumented probe. |
| B-06 tint ownership | Derived, unprobed | With pre-blurred backdrop, tint = our sprite/paint above frost; double-tint risk exists only on the accent fallback tier, handled by the realization report. Low risk; probe optional. |
| B-07 shadow | Pending | Not probed. Note recorded: self-drawn shadows require surface margin outside the hit region; retaining DWM `undecorated_shadow` per tier remains an option. |
| B-08 effect chain spike | Mooted for acrylic-lite by B-02 | Required only for the full-material fidelity slice; defer to that slice's census. |
| B-09 cost | Pending | Setup/teardown latency and show-cycle interaction unmeasured; must ride the show-cycle contract (roadmap item 2) work. |

## Phase C — cross-platform

- **C-01 macOS (question bank; zero verified claims — no hardware):**
  `NSVisualEffectView` is region-shaped material with `maskImage` rounding and
  a semantic material palette; `state = .active` pre-answers focus coupling
  (roadmap 16). Hardware questions banked: does `NSWindow.alphaValue` fade the
  material; nonactivating-panel + material interaction; per-corner mask
  fidelity at retina scales.
- **C-02 Linux (desk research, cited):** `ext-background-effect-v1` is merged
  into wayland-protocols — based on `org_kde_kwin_blur`, in discussion since
  January 2024. KDE 6.7 removed its private blur protocol in favor of it
  (with reported transition regressions); Niri 26.04 adopted it. GNOME/Mutter
  support: no evidence found — assume absent. X11 KDE retains the
  `_KDE_NET_WM_BLUR_BEHIND_REGION` property. Consequence: the Linux `Frost`
  tier standardizes on one protocol rather than per-compositor shims; the
  corner-contraction note for pixel-region blur behind rounded glass stands.
  Sources: wayland.app kde-blur page; Phoronix "Wayland Background Effect";
  kitty issue #9534; ghostty discussion #13068; Niri 26.04 release coverage.
- **C-03 tier table:** Windows composition / macOS → `Full` (constructed vs
  semantic); KWin + Niri (+ Hyprland expected) via `ext-background-effect`,
  KDE-X11 via property → `Frost` (+ our tint); GNOME, bare X11, in-frame →
  `None`. The three-tier enum from A-02 covers every observed platform tier.

## Phase D — synthesis

**Draft campaign constitution:** scene keeps declaring `Pane`/`Material`
regions; presentation submits a retained, keyed region set (identity: pane /
overlay-entry `interaction::Id` species); the platform returns per-region
realizations (promote `PopupMaterialRealization` from window to region
granularity and into the seam vocabulary: `Full` / `Frost` / `None`); the
compositor assembles the residual scene (generalizing `native_popup_scenes`);
capabilities forecast (grow `Capabilities` past one bit), realizations
testify; the embassy keeps all OS objects; settle-rate discipline per region
via the existing `SysApplicator` species.

**The one structural decision the campaign cannot dodge (from B-03):** frost
and wgpu content on one Windows HWND cannot share the tree today. Choose:
two-window pair (works now; geometry sync burden on the proven applicator) or
upstream wgpu extension (one window; cleaner; external dependency and
timeline). Recommendation: prototype the pair behind the seam — the seam hides
the choice, so upstreaming later is a realization swap, not a redesign.

**Sequencing:** show-cycle contract (item 2) precedes any composition-tree
work (B-09 rides it); context menu (item 4) needs none of this; the material
campaign follows as the popup-polish arc (item 18 subsumed).

**Taste questions for Shea:** shadow ownership per tier (self-drawn vs DWM vs
`hasShadow`); acrylic-lite as the shipping default with full-material as a
later fidelity slice; Win10 floor (host backdrop needs 2004+; recommend
declaring Win11 the floor and recording it).

**Honest coverage gaps vs the ledger spec:** A-01 pane-field census partial;
A-06 desk-level; B-04 single-scale; B-05 cadence unmeasured; B-06/07/09
unprobed; B-01 owner-focused-state variant subsumed by the stronger
never-foreground result rather than exercised separately; `accent5` mode built
but not exercised (mooted by B-01/B-02 succeeding on the documented path).
