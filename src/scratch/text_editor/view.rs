use std::path::Path;

use super::super::{
    View, document, geometry, interaction, scene, session, timeline, view as framework_view, widget,
};
use super::{LoadStressText, State, ToggleDebugPanel, ToggleWrapText};

const FILE_MENU: interaction::Id = interaction::Id::new("menu.file");
const EDIT_MENU: interaction::Id = interaction::Id::new("menu.edit");
const VIEW_MENU: interaction::Id = interaction::Id::new("menu.view");
const DOCUMENT_FOCUS: interaction::Id = interaction::Id::new("document");

pub const WINDOW_TITLE: &str = "wgpu_l3 Notepad";
pub const CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 17, 19);

pub fn window_size() -> geometry::Size {
    geometry::Size::new(920, 680)
}

pub fn view(state: &State, cx: framework_view::Context) -> View {
    let wrap = if state.wrap_text {
        framework_view::control::Wrap::Word
    } else {
        framework_view::control::Wrap::None
    };
    let debug_panel = if state.show_debug_panel {
        let dirty = if state.document.is_dirty() {
            "modified"
        } else {
            "saved"
        };
        let path = state
            .document
            .path()
            .map(compact_path)
            .unwrap_or_else(|| "Untitled".to_owned());
        let wrap = if state.wrap_text { "on" } else { "off" };
        let diagnostics = cx.diagnostics();
        let text_diagnostics = &diagnostics.text;
        let scroll = &diagnostics.scroll;
        let frame = &diagnostics.frame;
        let status = format!(
            "File: {path} ({dirty}) | Wrap: {wrap}\nDocument: {} lines, {} bytes | Edits: {} | Status: {}\nText layout: paint {}, metrics {}, visible {}, shaped {}, segments {}+{}, overlays {}, highlight scans {}\nText caches: line {}/{}, render surfaces {}, render cache {}/{}, render source {} lines / {} bytes\nScroll: wheel {}, offsets {}, queued {}, redraws {}, commits {}, pending {}/{}\nText scroll: projections {}, resolve/reuse/shift {}/{}/{}, shift misses {}, cold jumps {}, async reconcile/skips {}/{}\nRetained scroll: layer hits {}, text skips {}, target fallbacks {}, layer rebuilds {}\nFrames: full {}, scroll-only {}, scroll fallbacks {}, render skips {}\nFrame us latest/avg: paint {}/{}, render {}/{}, text {}/{}, total {}/{}\nLast scroll frame: text {}us, render {}us, total {}us, surfaces {}, glyph batches {}",
            state.document.line_count(),
            state.document.len(),
            state.document.edit_count(),
            state.last_status,
            text_diagnostics.text_area_paint_layout_calls,
            text_diagnostics.text_area_metrics_layout_calls,
            text_diagnostics.text_area_visible_logical_lines,
            text_diagnostics.text_area_shaped_logical_lines,
            text_diagnostics.text_area_layout_segments,
            text_diagnostics.text_area_overscan_segments,
            text_diagnostics.text_area_interaction_surfaces,
            text_diagnostics.highlight_run_scans,
            text_diagnostics.text_area_line_cache_hits,
            text_diagnostics.text_area_line_cache_misses,
            text_diagnostics.text_area_render_surface_calls,
            text_diagnostics.text_area_render_surface_cache_hits,
            text_diagnostics.text_area_render_surface_cache_misses,
            text_diagnostics.text_area_render_surface_source_lines,
            text_diagnostics.text_area_render_surface_source_bytes,
            scroll.wheel_events,
            scroll.scroll_offset_changes,
            scroll.queued_scroll_updates,
            scroll.scroll_redraw_requests,
            scroll.frame_scroll_commits,
            scroll.pending_scroll_applications,
            scroll.pending_scroll_updates,
            scroll.projection_count,
            scroll.text_area_resolves,
            scroll.text_area_projection_reuses,
            scroll.text_area_projection_shifts,
            scroll.text_area_projection_shift_misses,
            scroll.text_area_projection_cold_jumps,
            scroll.async_scroll_reconciles,
            scroll.async_scroll_projection_sync_skips,
            scroll.retained_scroll_layer_hits,
            scroll.retained_scroll_layer_text_prepare_skips,
            scroll.retained_scroll_target_repaint_fallbacks,
            scroll.retained_scroll_layer_rebuilds,
            frame.full_redraws,
            frame.scroll_only_redraws,
            frame.scroll_only_fallbacks_to_full,
            frame.render_skips,
            frame.paint.latest_us,
            frame.paint.average_us,
            frame.render.latest_us,
            frame.render.average_us,
            frame.render_text_prepare.latest_us,
            frame.render_text_prepare.average_us,
            frame.total.latest_us,
            frame.total.average_us,
            frame.last_scroll_frame.render_text_prepare_us,
            frame.last_scroll_frame.render_total_us,
            frame.last_scroll_frame.total_us,
            frame.last_scroll_frame.text_surfaces,
            frame.last_scroll_frame.glyph_batches,
        );
        Some(
            widget::Panel::new()
                .child(widget::Label::new("Debug"))
                .child(widget::Label::new(status)),
        )
    } else {
        None
    };

    widget::view(|ui| {
        ui.column(|ui| {
            ui.menu_bar(|ui| {
                ui.menu(FILE_MENU, "File", |ui| {
                    ui.add(widget::Binding::<document::NewFile>::menu());
                    ui.add(widget::Binding::<document::OpenFile>::menu());
                    ui.add(widget::Binding::<document::SaveFile>::menu());
                    ui.add(widget::Binding::<document::SaveAsFile>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<LoadStressText>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<session::CloseWindow>::menu());
                });
                ui.menu(EDIT_MENU, "Edit", |ui| {
                    ui.add(widget::Binding::<timeline::Undo>::menu());
                    ui.add(widget::Binding::<timeline::Redo>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<document::Cut>::menu());
                    ui.add(widget::Binding::<document::Copy>::menu());
                    ui.add(widget::Binding::<document::Paste>::menu());
                    ui.add(widget::Binding::<document::Delete>::menu());
                    ui.separator();
                    ui.add(widget::Binding::<document::SelectAll>::menu());
                });
                ui.menu(VIEW_MENU, "View", |ui| {
                    ui.add(widget::Binding::<ToggleWrapText>::menu());
                    ui.add(widget::Binding::<ToggleDebugPanel>::menu());
                });
            });
            ui.text_area(
                widget::TextArea::from_buffer(
                    state.document.buffer().clone(),
                    state.document.text_state(),
                )
                .wrap(wrap)
                .focus(session::Focus::text(DOCUMENT_FOCUS)),
            );
        });

        if let Some(debug_panel) = debug_panel {
            ui.add(debug_panel);
        }
    })
}

pub fn compact_path(path: &Path) -> String {
    let path = path.display().to_string();
    let max_chars = 120;
    if path.chars().count() <= max_chars {
        return path;
    }

    let suffix = path
        .chars()
        .rev()
        .take(max_chars - 3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("...{suffix}")
}
