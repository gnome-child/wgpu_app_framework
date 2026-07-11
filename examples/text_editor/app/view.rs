use std::path::Path;

use super::{
    State,
    command::{LoadStressText, ToggleDebugPanel, ToggleWrapText},
};
use wgpu_l3::{
    View, document, geometry, interaction, scene, session,
    text::Overflow,
    timeline,
    view::{Context as ViewContext, Wrap},
    widget,
};

const FILE_MENU: interaction::Id = interaction::Id::new("menu.file");
const EDIT_MENU: interaction::Id = interaction::Id::new("menu.edit");
const VIEW_MENU: interaction::Id = interaction::Id::new("menu.view");
const DOCUMENT_FOCUS: interaction::Id = interaction::Id::new("document");

pub const WINDOW_TITLE: &str = "wgpu_l3 Notepad";
pub const CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 17, 19);

pub fn window_size() -> geometry::Size {
    geometry::Size::new(920, 680)
}

pub fn view(state: &State, cx: ViewContext) -> View {
    let wrap = if state.wrap_text {
        Wrap::Word
    } else {
        Wrap::None
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
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Untitled".to_owned());
        let wrap = if state.wrap_text { "on" } else { "off" };
        let diagnostics = cx.diagnostics();
        let text_diagnostics = &diagnostics.text;
        let scroll = &diagnostics.scroll;
        let frame = &diagnostics.frame;
        let render = &diagnostics.render;
        let status = format!(
            "Document: {} lines, {} bytes | Edits: {} | {dirty} | Wrap: {wrap}\nText layout: author overflows {}, paint {}, metrics {}, visible {}, shaped {}, segments {}+{}, overlays {}, highlight scans {}\nText caches: line {}/{}, render surfaces {}, render cache {}/{}, render source {} lines / {} bytes\nScroll: wheel {}, offsets {}, redraws {}, commits {}, text area viewports {}\nFrames: full {}, rebuilds {}, layout recomposes {}, layout reuses {}, text surfaces {}\nRender: frames {}, interval p95 {}us, acquire p95 {}us, draw p95 {}us, key->present p95 {}us, pending keys {}, groups {}, pools layer/scratch {}/{}",
            state.document.line_count(),
            state.document.len(),
            state.document.edit_count(),
            text_diagnostics.author_text_overflows,
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
            scroll.scroll_redraw_requests,
            scroll.frame_scroll_commits,
            scroll.text_area_viewports,
            frame.full_redraws,
            frame.view_rebuilds,
            frame.layout_recomposes,
            frame.layout_reuses,
            frame.text_area_render_surfaces,
            render.frames_presented,
            render.interval_p95_us(),
            render.acquire_wait_p95_us(),
            render.draw_p95_us(),
            render.key_to_present_p95_us(),
            render.pending_key_to_present_samples(),
            render.group_composites,
            render.filter_layer_pool_entries,
            render.filter_scratch_pool_entries,
        );
        Some(
            widget::Panel::new()
                .child(widget::Label::new("Debug"))
                .child(widget::Label::world(
                    format!("File: {path}"),
                    Overflow::EllipsisMiddle,
                ))
                .child(widget::Label::world(
                    format!("Status: {}", state.last_status),
                    Overflow::EllipsisEnd,
                ))
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
                widget::TextArea::from_document(&state.document)
                    .wrap(wrap)
                    .focus(session::Focus::text(DOCUMENT_FOCUS)),
            );
        });

        if let Some(debug_panel) = debug_panel {
            ui.add(debug_panel);
        }
    })
}

pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}
