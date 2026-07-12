# Show-cycle presentation contract

Status: complete at `d4a6072b`. Prerequisite to the material-regions campaign
is satisfied on Windows.

## Contract

The first visible frame of every native popup show cycle is a freshly
presented frame for the current scene. Window existence, OS visibility, GPU
presentation, compositor pickup, and user visibility are separate facts.

The current path shows the popup before acquiring and presenting its first
frame. A successful later present therefore cannot prove that stale or empty
swapchain content was never visible. The correction is a readiness gate:

1. establish shell geometry and immediately-due material while the popup is
   not user-visible;
2. make the native surface presentable without exposing it;
3. acquire and present the current scene;
4. cross the platform presentation barrier;
5. only then expose the popup to the user.

On Windows the hidden-but-presentable state is an application DWM cloak. The
HWND is shown with `SW_SHOWNOACTIVATE` while cloaked, the current frame is
presented and synchronized, and successful readiness removes the cloak. A
skipped acquire or failed synchronization retains the cloak and requests the
bounded follow-up already owned by `PopupFirstPresentTrace`.

This checkpoint proves the Windows contract required by the material-regions
campaign. Other native platforms retain their established show behavior until
they gain and witness their own concealment primitive; no portability claim is
minted from Windows evidence.

## Census receipts

- `src/platform/native/popup.rs`: `set_popup_visibility(true)` currently
  precedes `renderer.draw`; `PopupFirstPresentTrace` owns the finite
  first/confirmation lifecycle and DWM synchronization.
- `src/platform/native/window.rs`: native popup windows are created hidden;
  popup visibility already routes through the platform `sys` seam.
- `src/platform/native/sys/windows.rs`: Windows show uses
  `SW_SHOWNOACTIVATE`; `DwmFlush` is the existing compositor-pickup barrier.
- `src/render/surface.rs`: skipped acquire outcomes produce no present timing,
  so they cannot authorize exposure.

## Required evidence

- Ordering witness: configure/material → cloak/show → current present →
  synchronization → uncloak.
- No-present outcomes remain unexposed and request a retry.
- A synchronization failure remains unexposed through one bounded
  confirmation attempt; no unbounded retry budget appears.
- Reopened/reused identity begins a new readiness cycle rather than inheriting
  a prior visible state.
- Popup shell remains nonactivating and existing cursor/IME/overlay lifetime
  behavior remains unchanged.
- Full suite, doctests, all-target compilation, three application smokes, and
  comparison protection pass before closure.

## Close-out evidence

- `d4a6072b` replaces show-before-draw with the finite readiness action:
  configure and immediately-due material, `DWMWA_CLOAK`, no-activate show,
  current draw/present, `DwmFlush`, then uncloak.
- Skipped acquisition remains concealed. First synchronization failure earns
  one concealed confirmation present; no retry budget was introduced.
- The architecture witness pins draw → present → expose and requires both
  cloak transitions and every diagnostic stage.
- Full library gate: 906 passed, 8 ignored. Doctests: 4 passed. `cargo check
  --all-targets`, `cargo fmt --all --check`, `git diff --check`, and all three
  application smokes passed.
- A deliberately broader `cargo test --all-targets` exposed an existing
  example-test cfg mismatch (`control_gallery::window_size` is hidden under
  `cfg(test)`); all-target compilation and the required full library gate are
  green, so that unrelated example-test issue was not absorbed here.
- Live Windows acceptance used the Vulkan redirected path and reopened the
  Controls native popup eight times. Every observed popup contained its
  current Click/Reset content; no empty or stale first frame, activation
  change, or crash appeared.
- `comparison_open: true`: the unrelated local gallery-height edit remains the
  sole uncommitted file and was not staged.
