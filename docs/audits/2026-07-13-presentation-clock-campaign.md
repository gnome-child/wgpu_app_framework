# Presentation Clock campaign

Status: in flight. `comparison_open: true`. No push.

Mission: input updates model and session truth at event rate; presentation
samples the latest truth at the platform frame boundary. Many events may
coalesce into one frame, but no semantic input is discarded.

## Rails

- Census and baseline precede behavioral changes.
- Each checkpoint lands independently green.
- Formatting, all-target compilation, the library suite, doctests, the three
  application smokes, release witnesses, and diff hygiene gate each boundary.
- No targeted redraw, cache enlargement, second graphics context, or input
  throttling without a profile receipt.
- No public timing/configuration API without an external caller.
- `examples/control_gallery/app/view.rs` contains the user's protected
  136-to-500 table-height witness. It remains unstaged unless explicitly
  adopted; campaign work must not absorb it silently.

## Constitution

- Events and frames are different clocks. One event does not imply one frame.
- Coalescing removes obsolete frames, never semantic mutations.
- `state::Revision` remains application-model truth; per-window presentation
  freshness receives its own epoch.
- Prepared is not presented. Only a successful platform receipt promotes
  candidate geometry to last-presented geometry.
- Input is interpreted against the frame the user saw, not a private layout
  recomposed during the event.
- Hover is derived from logical pointer position and last-presented geometry.
- Direct manipulation mutates session sources; it does not rebuild view
  structure merely to transport geometry.
- Backend capability is surface-scoped. Popup composition needs do not silently
  dictate unrelated main-window presentation.

## Ownership map

| Fact | Owner |
| --- | --- |
| Application data | Model revision |
| Scroll, resize, focus, selection, drafts, capture | Per-window session |
| Logical pointer position | Per-window interaction state |
| Hovered target | Pointer position x last-presented geometry |
| Pending frame strength | One coalesced per-window invalidation |
| Frame freshness | Per-window presentation epoch |
| Candidate layout and scene | Prepared frame |
| Input geometry | Last successfully presented layout |
| GPU/presentation outcome | Platform report |
| Native redraw scheduling | Platform/window backend |
| Table width override | Table session projection consumed by layout |
| Text shaping | Text engine, once per width selected for a frame |

## Checkpoints

| Checkpoint | State | Boundary |
| --- | --- | --- |
| 0. Evidence harness and backend verdict | Complete | `4abc2472`; phase timings and event/frame counts; release 136/500/800-pixel Vulkan/DX12-Visual/DX12-HWND matrix |
| 1. Presentation receipts and geometry epochs | Complete | `713ff311`; candidate geometry travels with epoch/invalidation; only a successful backend receipt promotes it |
| 2. Presentation-rate coalescing | Complete | `3b95c174`; ordinary work is immediate/non-rendering; `RedrawRequested` is the sole platform frame boundary |
| 3. Last-presented input geometry | Pending | No event-local speculative layout |
| 4. Pointer truth and hover projection | Pending | Pointer position retained; hover re-hits before changed frame paint |
| 5. Transient table widths | Pending | Divider drag is layout-only, never a view rebuild |
| 6. Backend scope verdict | Pending | Apply only the measured DX12/presentation conclusion |
| 7. Reprofile and doctrine | Pending | Admit only remaining evidenced mechanism; close roadmap and laws |

## Checkpoint 0 initial receipts

The current native runner handles each translated `WindowEvent`, immediately
drains shell work, and synchronously applies every produced presentation before
returning to the event loop. High-frequency input therefore pays surface
acquisition and presentation at event rate.

Scrolling currently composes a routing layout in `Runtime::scroll_at`, applies
the session delta, requests `Invalidation::Layout`, then composes layout again
while preparing the presentation. Divider dragging composes layout to derive
the drag action, stores the width in session state, and requests a full
`Rebuild`; table widths project only during application view reconstruction.

The runtime retains hovered target but not logical pointer position. Hit testing
accepts cached layout without distinguishing candidate from successfully
presented geometry. `session::Window::presented_revision` is marked while the
view projection is installed, before the platform attempts surface acquisition;
model revision is therefore currently carrying a stronger name than its
evidence warrants.

Renderer `DrawStats` already owns scene-item, render/glyph batch, inline text
cache, icon cache, shaping-call, vertex, clip, group, and filter-pool counts.
Only group and filter-pool facts currently cross into framework diagnostics.
The evidence harness will carry the existing facts rather than mint duplicates.

The three live startup configurations already share `Mailbox`,
`desired_frame_latency=1`, `Bgra8UnormSrgb`, and the same initial surface size:
Vulkan, DX12 `DxgiFromVisual`, and DX12 `DxgiFromHwnd`. Their interaction
difference therefore lies below shared surface configuration. The manual
interaction verdict remains open.

The campaign ledger sentence is:

> Many events may become one frame; every semantic mutation survives, and the
> frame presents the latest truth.

## Checkpoint 0 instrumentation and first baseline

The behavior-neutral harness now separates native translation, semantic event
handling, total native event pass, view reconstruction, composition
reconciliation, routing layout, presentation layout (including virtual
refinement), scene assembly, renderer batch preparation, surface acquisition,
and encode/submit/present. Existing draw facts cross the same report boundary:
scene items, render and glyph batches, text surfaces, vertices, clips, groups,
filter pools, and inline text/icon cache activity. Received events, prepared
frames, attempted frames, and successful frames are separate counters.

Input-latency samples now carry a per-window presentation epoch. Model revision
is no longer the diagnostic proxy for scroll, hover, focus, or resize freshness.
Skipped surface attempts do not acknowledge those samples. This is diagnostic
currency only at this boundary; candidate-layout promotion remains checkpoint
1 work.

The protected 500-pixel gallery edit was exercised directly but remains
unstaged and unadopted. The window was 813 by 1106 logical pixels and the scene
held approximately 618 items, 605 render batches, 118 glyph batches, and 48
inline text runs per settled frame. Fixed, repeated wheel and Count/Enabled
divider gestures were injected at stable window-relative coordinates. The
Windows capture API produced incomplete damage-like snapshots after some
visual-surface runs, so screenshots are not used as rendering evidence here;
the counters and the user's direct display remain the witnesses.

### Sustained 500-pixel table baseline

| Backend/surface | Native event p95 | Encode/present p95 | Total draw p95 | Frame interval p95 | Acquire p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Vulkan | 27.8 ms | 17.3 ms | 18.9 ms | 27.9 ms | 0.08 ms |
| DX12 + `DxgiFromVisual` | 62.6 ms | 50.2 ms | 54.2 ms | 63.0 ms | 0.03 ms |
| DX12 + `DxgiFromHwnd` | 61.5-64.6 ms | 49.4-50.7 ms | 53.1-54.9 ms | 62.0-63.7 ms | 0.03 ms |

The dominant measured residual is below surface acquisition: CPU routing and
presentation layouts together cost roughly 7-8 ms, while DX12 spends about
50 ms in encode/submit/present and Vulkan about 17 ms. Surface acquisition is
not the blocker.

The structural counts are more decisive than the timings. During sustained
wheel input, `events`, `prepared`, `attempted`, and `presented` converge
one-for-one; routing-layout count rises once per wheel event, followed by one
presentation layout. During 100 divider drags, view-rebuild count rises by
approximately 98, while the text cache records only two new width shapes after
the first pair and then hits. The lag is therefore not cache-capacity thrash:
event-rate frame production and rebuild transport dominate.

### Backend decision gate

Verdict: **both DX12 modes lag similarly**. `DxgiFromHwnd` does not recover the
Vulkan behavior, so DirectComposition is not convicted and ordinary-window
surface selection does not change at checkpoint 0. Complete coalescing and
remeasure before considering a second context or a Vulkan-main/DX12-popup
split. All three paths continued to report `Mailbox`, desired latency 1,
`Bgra8UnormSrgb`, and the same surface size.

### Visible-size scaling matrix

The harness commit was checked out into a detached comparison worktree. Its
committed 136-pixel gallery fixture and a disposable 800-pixel build supplied
the missing sizes without touching the user's 500-pixel edit. All runs used the
same release profile, gesture coordinates, backend environment, and diagnostic
window. Scene population rose from approximately 258 items / 245 batches at
136 pixels, through 618 / 605 at 500 pixels, to 690 / 677 at 800 pixels (the
outer window clip bounded the final materialization).

| Table height | Vulkan native / encode | DX12 visual native / encode | DX12 HWND native / encode |
| --- | ---: | ---: | ---: |
| 136 px | 10.6 / 6.4 ms | 17.4 / 12.2 ms | 17.3 / 12.1 ms |
| 500 px | 27.8 / 17.3 ms | 62.6 / 50.2 ms | 61.5-64.6 / 49.4-50.7 ms |
| 800 px | 28.9 / 17.9 ms | 68.6 / 59.4 ms | 67.8 / 56.2 ms |

The field report is reproduced: frame cost follows visible scene population,
not logical record count, and DX12 amplifies the renderer/present portion. The
plateau from 500 to 800 corresponds to the outer clip bounding visible
materialization; it is evidence that virtualization is working. The remaining
per-visible-cell cost is paid once per raw input event, which is the clock
violation checkpoints 2-5 remove.

Checkpoint 0 boundary: formatting, all-target compilation, 926 library tests
passed with 8 deliberate ignores, all 4 doctests passed, the three application
smokes passed, diff hygiene held, and the protected gallery edit remained the
only unrelated working-tree change.

## Checkpoint 1 — prepared is not presented

`PresentedGeometry` is now a retained per-window store distinct from the
layout cache. A prepared presentation carries its model revision, presentation
epoch, invalidation strength, and candidate layout across the shell/platform
boundary. The backend render report explicitly distinguishes an acquired and
presented surface frame from a skipped attempt.

On success, and only on success, the runtime acknowledges the candidate epoch
and promotes its layout to input-visible geometry. Monotonic acknowledgement
prevents an older receipt from replacing newer geometry. A skip promotes
nothing and restores the candidate invalidation without advancing the desired
epoch: retrying delivery is not a new presentation truth. Per-window departure
and snapshot restoration remove the presented-geometry store.

The old `presented_revision` field was renamed `projected_revision`, matching
what it actually proves: the application view projection has been installed.
Model revision, desired presentation epoch, projected revision, and
acknowledged presentation epoch no longer borrow each other's names.

Named witnesses cover skipped-then-successful delivery, stable retry epoch,
stale successful receipt after a newer success, independent model/desired/
acknowledged advancement, and teardown cleanup. Boundary: 930 library tests
passed with 8 deliberate ignores; all 4 doctests, three application smokes,
formatting, all-target compilation, and diff hygiene passed. The protected
gallery-height edit remains untouched.

## Checkpoint 2 — events and frames are different clocks

Shell/runtime work now has two explicit products. `ImmediateWork` carries
window lifecycle, cursor, dialog, task, request, poll, animation schedule, and
the windows whose coalesced invalidation requires a native redraw. It cannot
carry a presentation. `RenderWork` is produced for one window only when the
platform delivers `RedrawRequested`.

Startup opens the backend window and requests its first redraw; it no longer
builds a frame in the resumed callback. Resize, input, task completion, and
animation due-work mutate truth and request redraw without presenting. An OS
redraw with no pending mutation still prepares an honest paint frame. A skipped
surface receipt restores the same invalidation and requests another native
redraw. Native popup presentations remain synchronized with the parent frame
that realizes them.

The platform backend gained one surface-scoped `request_redraw` operation;
the Windows implementation delegates to the retained winit window. No frame
queue, input throttle, targeted damage mechanism, or second graphics context
was added.

Deterministic witnesses prove that 1,000 pointer events plus ten ordered click
commands execute immediately with zero interim backend presentations and one
final frame; 1,000 wheel deltas retain an exact cumulative offset and one final
frame; and a skipped frame requests redraw until one receipt succeeds. Existing
host/platform tests were rewritten to state the new law explicitly: lifecycle
or input work is followed by a redraw opportunity when visual output is
required.

Checkpoint boundary: 933 library tests passed with 8 deliberate ignores; all
4 doctests, three application smokes, formatting, all-target compilation, and
diff hygiene passed. The remaining per-event routing layout is deliberately
visible in diagnostics and is removed by checkpoint 3; no input event performs
view reconstruction, scene assembly, renderer drawing, or backend presentation.
