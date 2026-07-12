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
| 0. Evidence harness and backend verdict | In progress | Phase timings, event/frame counts, release Vulkan/DX12-Visual/DX12-HWND baseline |
| 1. Presentation receipts and geometry epochs | Pending | Prepared candidate versus successfully presented geometry |
| 2. Presentation-rate coalescing | Pending | Event work immediate; scene/GPU work only at redraw boundary |
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
