mod choice;
mod slider;
mod text_area;
mod text_box;
mod text_surface;

use crate::icon as icons;

use super::super::{context, geometry, keymap, layout, theme::Theme, view};
use super::{
    BackdropLayer, Brush, Clip, Filter, FilterOp, Icon, Material, Offset, Outline, Quad, Scene,
    Shadow, Style, SurfaceLayer, Text, TextAlign, TextStyle, TextWrap, Visuals,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Chrome,
    Floating,
}

pub(super) fn paint_layout_with_theme(
    layout: &layout::Layout,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
) {
    for layer in [Layer::Base, Layer::Chrome, Layer::Floating] {
        let mut focus_overlays = Vec::new();

        for frame in layout
            .frames()
            .iter()
            .filter(|frame| layer_for(frame) == layer)
        {
            paint_frame_with_clip(frame, scene, theme, visuals, &mut focus_overlays);
        }

        for chrome in layout
            .chrome()
            .iter()
            .filter(|chrome| chrome_layer_for(layout, chrome) == layer)
        {
            paint_chrome(chrome, scene, theme, visuals);
        }

        for outline in focus_overlays {
            scene.push_outline(outline);
        }
    }
}

fn paint_frame_with_clip(
    frame: &layout::Frame,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
    overlays: &mut Vec<Outline>,
) {
    let clip = frame.clip();
    if let Some(clip) = clip {
        scene.push_clip(Clip::new(clip.rect()).with_rounding(clip.rounding()));
    }

    paint_frame(frame, scene, theme, visuals, overlays);

    if clip.is_some() {
        scene.pop_clip();
    }
}

fn paint_frame(
    frame: &layout::Frame,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
    overlays: &mut Vec<Outline>,
) {
    if is_floating_panel_role(frame.role()) {
        paint_floating_panel_shadow(frame, scene, theme);
        paint_floating_panel_filter(frame, scene, theme);
    }

    let rounding = role_rounding(frame, theme);
    if is_floating_panel_role(frame.role()) {
        paint_floating_panel_material(frame, scene, theme);
    } else if let Some(fill) = frame.background() {
        paint_brush_quad(scene, frame.rect(), fill, rounding);
    } else if let Some(fill) = role_fill(frame, theme) {
        scene.push_quad(Quad::new(frame.rect(), fill).with_rounding(rounding));
    }

    if let Some(tint) = visual_tint_for(frame, theme, visuals) {
        scene.push_quad(
            Quad::new(visual_tint_rect_for(frame), tint)
                .with_rounding(visual_tint_rounding_for(frame, rounding, theme)),
        );
    }

    match frame.role() {
        view::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            paint_menu_row(frame, scene, theme);
        }
        _ if frame.binding_source() == Some(context::Source::Palette) => {
            paint_palette_row(frame, scene, theme);
        }
        view::Role::Separator => {
            paint_menu_separator(frame, scene, theme);
        }
        view::Role::SectionHeader => {
            paint_section_header(frame, scene, theme);
        }
        view::Role::Checkbox | view::Role::Radio => {
            choice::paint(frame, scene, theme, visuals);
        }
        view::Role::Slider => {
            slider::paint(frame, scene, theme, visuals);
        }
        view::Role::TextArea => {
            if let Some(text_area) = frame.text_area_layout() {
                text_area::paint(frame, text_area, scene, theme, visuals);
            }
        }
        view::Role::TextBox => {
            text_box::paint_selection(frame, scene, theme);
        }
        _ => {}
    }

    if frame.binding_source() == Some(context::Source::Menu)
        || frame.binding_source() == Some(context::Source::Palette)
        || frame.role() == view::Role::Separator
        || frame.role() == view::Role::SectionHeader
        || is_floating_panel_role(frame.role())
    {
        // Floating panes and menu rows paint their own content.
    } else if frame.role() == view::Role::TextBox && text_box::paint_text(frame, scene) {
        // TextBox contents use the shaped field viewport so glyphs, selection,
        // caret, and horizontal scroll share one coordinate system.
    } else if let Some(value) = text_for(frame) {
        scene.push_text(
            Text::new(
                text_rect_for(frame, theme),
                value,
                text_color_for(frame, theme),
                text_wrap_for(frame),
            )
            .with_style(text_style_for(frame, theme))
            .with_align(text_align_for(frame)),
        );
    }

    if frame.role() == view::Role::TextBox {
        text_box::paint_caret(frame, scene, theme, visuals);
    }

    if let Some(color) = focus_outline_color_for(frame, theme) {
        overlays.push(
            Outline::new(focus_outline_rect_for(frame), color)
                .with_width(theme.focus().width as f32)
                .with_offset(theme.focus().offset)
                .with_rounding(focus_outline_rounding_for(frame, rounding, theme)),
        );
    } else if let Some(color) = outline_color_for(frame, theme) {
        scene.push_outline(
            Outline::new(focus_outline_rect_for(frame), color)
                .with_width(theme.focus().width as f32)
                .with_rounding(focus_outline_rounding_for(frame, rounding, theme)),
        );
    }
}

fn layer_for(frame: &layout::Frame) -> Layer {
    if frame.is_floating_layer() {
        return Layer::Floating;
    }

    match frame.role() {
        view::Role::MenuBar | view::Role::Menu => Layer::Chrome,
        view::Role::FloatingPanel | view::Role::Separator => Layer::Floating,
        view::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            Layer::Floating
        }
        _ if frame.binding_source() == Some(context::Source::Palette) => Layer::Floating,
        _ => Layer::Base,
    }
}

fn role_fill(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    match frame.role() {
        view::Role::Root => Some(theme.surfaces().root),
        view::Role::MenuBar => Some(theme.menu().bar_background),
        view::Role::Menu => visible_fill(theme.menu().title_background),
        view::Role::FloatingPanel => None,
        view::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            visible_fill(theme.menu().row_background)
        }
        view::Role::Label if frame.binding_source() == Some(context::Source::Palette) => {
            visible_fill(theme.menu().row_background)
        }
        view::Role::Binding => visible_fill(if frame.is_enabled() {
            theme.control().background
        } else {
            theme.control().disabled_background
        }),
        view::Role::Separator => None,
        view::Role::TextArea => Some(theme.text_input().area_background),
        view::Role::Button => visible_fill(theme.control().button_background),
        view::Role::Checkbox | view::Role::Radio => visible_fill(theme.choice().background),
        view::Role::Slider => visible_fill(theme.slider().background),
        view::Role::TextBox => Some(theme.text_input().field_background),
        view::Role::Scroll => None,
        view::Role::Panel => visible_fill(theme.surfaces().panel),
        view::Role::SectionHeader | view::Role::Label | view::Role::Stack => None,
    }
}

fn visible_fill(color: super::Color) -> Option<super::Color> {
    (color.channels().3 > 0).then_some(color)
}

fn paint_chrome(chrome: &layout::Chrome, scene: &mut Scene, theme: &Theme, visuals: &Visuals) {
    match chrome.kind() {
        layout::ChromeKind::Scrollbar(scrollbar) => {
            paint_scrollbar(chrome, *scrollbar, scene, theme, visuals);
        }
    }
}

fn paint_scrollbar(
    chrome: &layout::Chrome,
    scrollbar: layout::Scrollbar,
    scene: &mut Scene,
    theme: &Theme,
    visuals: &Visuals,
) {
    let theme_scrollbar = theme.scrollbar();
    let visual = visuals.scrollbar(chrome.target());
    let visible = match theme_scrollbar.metrics.policy {
        crate::theme::ScrollbarPolicy::GutterAlways => 1.0,
        crate::theme::ScrollbarPolicy::OverlayAuto => visual.opacity(),
    };
    if visible <= 0.0 {
        return;
    }

    let base_thickness = match theme_scrollbar.metrics.policy {
        crate::theme::ScrollbarPolicy::GutterAlways => theme_scrollbar.metrics.thickness.max(1),
        crate::theme::ScrollbarPolicy::OverlayAuto => {
            theme_scrollbar.appearance.overlay_thickness.max(1)
        }
    };
    let thickness = visual.thickness().max(base_thickness);
    let track = scrollbar.track_with_thickness(thickness);
    let thumb = scrollbar.thumb_with_thickness(thickness);
    let appearance = theme_scrollbar.appearance;

    if appearance.track.channels().3 > 0 {
        scene.push_quad(
            Quad::new(track, color_with_opacity(appearance.track, visible))
                .with_rounding(appearance.rounding),
        );
    }
    if appearance.thumb.channels().3 > 0 {
        scene.push_quad(
            Quad::new(thumb, color_with_opacity(appearance.thumb, visible))
                .with_rounding(appearance.rounding),
        );
    }

    let tint = if visual.pressed() {
        appearance.thumb_pressed_tint
    } else if visual.hovered() {
        appearance.thumb_hover_tint
    } else {
        super::Color::rgba(0, 0, 0, 0)
    };
    if tint.channels().3 > 0 {
        scene.push_quad(
            Quad::new(thumb, color_with_opacity(tint, visible)).with_rounding(appearance.rounding),
        );
    }
}

fn chrome_layer_for(layout: &layout::Layout, chrome: &layout::Chrome) -> Layer {
    layout
        .frames()
        .iter()
        .rev()
        .find(|frame| frame.target() == Some(chrome.scroll_target()))
        .map(layer_for)
        .unwrap_or(Layer::Base)
}

fn color_with_opacity(color: super::Color, opacity: f32) -> super::Color {
    let (r, g, b, a) = color.channels();
    super::Color::rgba(
        r,
        g,
        b,
        ((a as f32) * opacity.clamp(0.0, 1.0)).round() as u8,
    )
}

fn role_rounding(frame: &layout::Frame, theme: &Theme) -> super::Rounding {
    match frame.role() {
        view::Role::FloatingPanel => theme.floating_panel().rounding,
        view::Role::Menu => theme.control().rounding,
        view::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            theme.control().rounding
        }
        _ if frame.binding_source() == Some(context::Source::Palette) => theme.control().rounding,
        view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox
        | view::Role::Panel => theme.control().rounding,
        _ => super::Rounding::none(),
    }
}

fn is_floating_panel_role(role: view::Role) -> bool {
    role == view::Role::FloatingPanel
}

fn paint_brush_quad(
    scene: &mut Scene,
    rect: geometry::Rect,
    fill: Brush,
    rounding: super::Rounding,
) {
    if fill.is_visible() {
        scene.push_quad(Quad::styled(rect, Style::filled_with(fill)).with_rounding(rounding));
    }
}

fn paint_floating_panel_shadow(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let panel = theme.floating_panel();
    scene.push_shadow(
        Shadow::new(
            frame.rect(),
            panel.shadow,
            panel.shadow_blur,
            panel.shadow_spread,
            Offset::new(0.0, panel.shadow_offset_y),
        )
        .with_rounding(role_rounding(frame, theme)),
    );
}

fn paint_floating_panel_filter(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let panel = theme.floating_panel();
    let Material::Glass(glass) = &panel.material else {
        return;
    };

    for layer in glass.backdrop_layers() {
        let op = match *layer {
            BackdropLayer::Blur(blur) => FilterOp::backdrop_blur(blur),
            BackdropLayer::Refraction(refraction) => FilterOp::refraction(refraction),
            BackdropLayer::Luminosity(luminosity) => FilterOp::luminosity(luminosity),
        };
        scene.push_filter(
            Filter::stack(frame.rect(), [op]).with_rounding(role_rounding(frame, theme)),
        );
    }
}

fn paint_floating_panel_material(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let panel = theme.floating_panel();
    match &panel.material {
        Material::Solid(brush) => {
            paint_brush_quad(scene, frame.rect(), *brush, role_rounding(frame, theme))
        }
        Material::Glass(glass) => {
            for layer in glass.surface_layers() {
                match *layer {
                    SurfaceLayer::Tint { brush, opacity } => {
                        paint_brush_quad(
                            scene,
                            frame.rect(),
                            brush_with_opacity(brush, opacity),
                            role_rounding(frame, theme),
                        );
                    }
                    SurfaceLayer::Noise(noise) => {
                        if noise.opacity() <= 0.0 {
                            continue;
                        }
                        scene.push_filter(
                            Filter::stack(frame.rect(), [FilterOp::noise(noise)])
                                .with_rounding(role_rounding(frame, theme)),
                        );
                    }
                }
            }
        }
    }
}

fn brush_with_opacity(brush: Brush, opacity: f32) -> Brush {
    let opacity = opacity.clamp(0.0, 1.0);
    match brush {
        Brush::Solid(color) => Brush::solid(color_with_opacity(color, opacity)),
        Brush::LinearGradient { from, to } => Brush::linear_gradient(
            color_with_opacity(from, opacity),
            color_with_opacity(to, opacity),
        ),
    }
}

fn paint_menu_row(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let parts = layout::menu_row_parts(frame.rect(), frame.shortcut_width(), theme);
    let color = text_color_for(frame, theme);

    if frame.checked() == Some(true) {
        scene.push_icon(Icon::new(
            parts.glyph,
            icons::Icon::phosphor(icons::Id::new("check")).with_style(icons::Style::Bold),
            color,
            layout::control_content_extent(theme.menu().row_height, theme) as f32,
        ));
    }

    if let Some(label) = frame.label_text() {
        scene.push_text(
            Text::new(parts.label, label, color, TextWrap::None)
                .with_style(scene_text_style(layout::interface_text_style(theme)))
                .with_align(TextAlign::Start),
        );
    }

    paint_shortcut(frame, parts.shortcut, scene, theme);
}

fn paint_menu_separator(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let parts = layout::menu_row_parts(frame.rect(), frame.shortcut_width(), theme);

    scene.push_quad(Quad::new(parts.separator, theme.menu().separator));
}

fn paint_palette_row(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let rect = frame.rect();
    let parts = layout::palette_row_parts(rect, frame.shortcut_width(), theme);
    let color = theme.text().primary;

    if let Some(label) = frame.label_text() {
        scene.push_text(
            Text::new(parts.label, label, color, TextWrap::None)
                .with_style(scene_text_style(layout::interface_text_style(theme)))
                .with_align(TextAlign::Start),
        );
    }

    paint_shortcut(frame, parts.shortcut, scene, theme);
}

fn shortcut_text_color(theme: &Theme) -> super::Color {
    theme.text().muted
}

fn paint_shortcut(frame: &layout::Frame, slot: geometry::Rect, scene: &mut Scene, theme: &Theme) {
    let Some(parts) = frame.shortcut_display() else {
        return;
    };
    if parts.is_empty() {
        return;
    }

    let mut x = slot.right().saturating_sub(frame.shortcut_content_width());
    let color = shortcut_text_color(theme);
    let style = scene_text_style(layout::shortcut_text_style(theme));
    let gap = layout::shortcut_run_gap(theme);

    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            x = x.saturating_add(gap);
        }

        let width = part.width();
        let rect = geometry::Rect::new(x, slot.y(), width, slot.height());
        match part.run() {
            keymap::ShortcutRun::Text(value) => {
                scene.push_text(
                    Text::new(rect, value, color, TextWrap::None)
                        .with_style(style)
                        .with_align(TextAlign::Start),
                );
            }
            keymap::ShortcutRun::Icon(icon) => {
                let extent = width.min(slot.height()).max(1);
                let icon_rect = geometry::Rect::new(
                    x,
                    slot.y()
                        .saturating_add((slot.height().saturating_sub(extent)) / 2),
                    extent,
                    extent,
                );
                scene.push_icon(Icon::new(
                    icon_rect,
                    shortcut_icon(*icon),
                    color,
                    extent as f32,
                ));
            }
        }

        x = x.saturating_add(width);
    }
}

fn shortcut_icon(icon: keymap::ShortcutIcon) -> icons::Icon {
    let id = match icon {
        keymap::ShortcutIcon::Control => "caret-up",
        keymap::ShortcutIcon::Shift => "arrow-fat-up",
        keymap::ShortcutIcon::Alt | keymap::ShortcutIcon::Option => "option",
        keymap::ShortcutIcon::Command => "command",
    };

    icons::Icon::phosphor(icons::Id::new(id))
}

fn paint_section_header(frame: &layout::Frame, scene: &mut Scene, theme: &Theme) {
    let Some(label) = frame.label_text() else {
        return;
    };

    scene.push_text(
        Text::new(
            frame.rect(),
            layout::section_header_text(label),
            theme.text().muted,
            TextWrap::None,
        )
        .with_style(scene_text_style(layout::section_header_style(theme)))
        .with_align(theme.command_palette().section_alignment()),
    );
}

fn visual_tint_for(
    frame: &layout::Frame,
    theme: &Theme,
    visuals: &Visuals,
) -> Option<super::Color> {
    if !frame.is_enabled() {
        return None;
    }
    let target_visual = frame
        .target()
        .map(|target| visuals.target(target))
        .unwrap_or_default();

    if frame.role() == view::Role::Menu {
        if target_visual.pressed() {
            return Some(theme.menu().title_pressed_tint);
        }

        if target_visual.active() {
            return Some(theme.menu().title_active_tint);
        }

        if target_visual.hovered() || frame.focus_visible() {
            return Some(theme.menu().title_hover_tint);
        }

        return None;
    }

    if matches!(
        frame.binding_source(),
        Some(context::Source::Menu | context::Source::Palette)
    ) {
        return row_highlight_tint_for(frame, theme, visuals);
    }

    if frame.role() == view::Role::TextBox {
        return target_visual
            .active()
            .then_some(theme.control().pressed_tint);
    }

    if !is_control_visual_role(frame.role()) {
        return None;
    }

    if target_visual.pressed() {
        Some(theme.control().pressed_tint)
    } else if target_visual.hovered() {
        Some(theme.control().hover_tint)
    } else {
        None
    }
}

fn row_highlight_tint_for(
    frame: &layout::Frame,
    theme: &Theme,
    visuals: &Visuals,
) -> Option<super::Color> {
    let source = frame.binding_source()?;
    if !matches!(source, context::Source::Menu | context::Source::Palette) {
        return None;
    }
    let target_visual = frame
        .target()
        .map(|target| visuals.target(target))
        .unwrap_or_default();

    if target_visual.pressed() {
        return Some(theme.menu().row_pressed_tint);
    }

    let highlighted = target_visual.hovered()
        || frame.focus_visible()
        || (source == context::Source::Palette && target_visual.selected());

    highlighted.then_some(theme.menu().row_hover_tint)
}

fn visual_tint_rect_for(frame: &layout::Frame) -> geometry::Rect {
    frame.rect()
}

fn visual_tint_rounding_for(
    _frame: &layout::Frame,
    default: super::Rounding,
    _theme: &Theme,
) -> super::Rounding {
    default
}

fn is_control_visual_role(role: view::Role) -> bool {
    matches!(role, view::Role::Binding | view::Role::Button)
}

fn text_for(frame: &layout::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn text_rect_for(frame: &layout::Frame, theme: &Theme) -> geometry::Rect {
    match frame.role() {
        view::Role::Checkbox | view::Role::Radio => layout::choice_label_rect(frame.rect(), theme),
        view::Role::Slider => layout::slider_label_rect(frame.rect(), frame.label_width(), theme),
        view::Role::TextBox => frame.text_box_text_rect(),
        _ => frame.rect(),
    }
}

fn text_color_for(frame: &layout::Frame, theme: &Theme) -> super::Color {
    if !frame.is_enabled() {
        return theme.text().muted;
    }

    match frame.role() {
        view::Role::TextBox
            if frame
                .text_box()
                .is_some_and(|text_box| text_box.text().is_empty()) =>
        {
            theme.text_input().placeholder
        }
        view::Role::SectionHeader => theme.text().muted,
        view::Role::TextArea | view::Role::TextBox => theme.text_input().foreground,
        view::Role::Separator => theme.menu().separator,
        _ => theme.text().primary,
    }
}

fn text_style_for(frame: &layout::Frame, theme: &Theme) -> TextStyle {
    match frame.role() {
        view::Role::SectionHeader => scene_text_style(layout::section_header_style(theme)),
        view::Role::Menu
        | view::Role::Binding
        | view::Role::Button
        | view::Role::Checkbox
        | view::Role::Radio
        | view::Role::Slider
        | view::Role::TextBox => scene_text_style(layout::interface_text_style(theme)),
        view::Role::Label if frame.binding_source() == Some(context::Source::Palette) => {
            scene_text_style(layout::interface_text_style(theme))
        }
        _ => scene_text_style(theme.typography().body()),
    }
}

fn scene_text_style(style: crate::theme::TypeStyle) -> TextStyle {
    TextStyle::new(style.size(), style.weight())
}

fn text_wrap_for(frame: &layout::Frame) -> TextWrap {
    if matches!(
        frame.role(),
        view::Role::Checkbox | view::Role::Radio | view::Role::Slider
    ) {
        return TextWrap::None;
    }

    match frame.text_wrap() {
        Some(view::Wrap::None) => TextWrap::None,
        Some(view::Wrap::Word) | None => TextWrap::WordOrGlyph,
    }
}

fn text_align_for(frame: &layout::Frame) -> TextAlign {
    match frame.role() {
        view::Role::Button | view::Role::Menu => TextAlign::Center,
        _ => TextAlign::Start,
    }
}

fn outline_color_for(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    matches!(
        frame.role(),
        view::Role::TextArea
            | view::Role::Button
            | view::Role::Slider
            | view::Role::TextBox
            | view::Role::Panel
    )
    .then_some(theme.focus().outline)
    .and_then(visible_fill)
}

fn focus_outline_color_for(frame: &layout::Frame, theme: &Theme) -> Option<super::Color> {
    (frame.focus_visible() && is_focus_outline_role(frame.role())).then_some(theme.focus().color)
}

fn focus_outline_rect_for(frame: &layout::Frame) -> geometry::Rect {
    match frame.role() {
        view::Role::Checkbox | view::Role::Radio | view::Role::Slider => frame.active_rect(),
        _ => frame.rect(),
    }
}

fn focus_outline_rounding_for(
    frame: &layout::Frame,
    default: super::Rounding,
    theme: &Theme,
) -> super::Rounding {
    match frame.role() {
        view::Role::Radio | view::Role::Slider => super::Rounding::relative(1.0),
        view::Role::Checkbox => theme.control().rounding,
        _ => default,
    }
}

fn is_focus_outline_role(role: view::Role) -> bool {
    matches!(
        role,
        view::Role::Menu
            | view::Role::Binding
            | view::Role::TextArea
            | view::Role::Button
            | view::Role::Checkbox
            | view::Role::Radio
            | view::Role::Slider
            | view::Role::TextBox
    )
}
