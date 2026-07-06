mod choice;
mod slider;
mod text_area;
mod text_box;

use crate::icon as framework_icon;

use super::super::{context, geometry, layout, theme::Theme, view};
use super::{
    Backdrop, Brush, Icon, Offset, Outline, Quad, Scene, Shadow, Style, Text, TextAlign, TextWrap,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layer {
    Base,
    Chrome,
    Popup,
}

pub(super) fn paint_layout_with_theme(layout: &layout::Layout, scene: &mut Scene, theme: &Theme) {
    let mut overlays = Vec::new();

    for layer in [Layer::Base, Layer::Chrome, Layer::Popup] {
        for frame in layout
            .frames()
            .iter()
            .filter(|frame| layer_for(frame) == layer)
        {
            paint_frame(frame, scene, theme, &mut overlays);
        }
    }

    for outline in overlays {
        scene.push_outline(outline);
    }
}

fn paint_frame(
    frame: &layout::frame::Frame,
    scene: &mut Scene,
    theme: &Theme,
    overlays: &mut Vec<Outline>,
) {
    if frame.role() == view::node::Role::Popup {
        paint_popup_shadow(frame, scene, theme);
        paint_popup_backdrop(frame, scene, theme);
    }

    let rounding = role_rounding(frame, theme);
    if frame.role() == view::node::Role::Popup {
        paint_popup_material(frame, scene, theme);
    } else if let Some(fill) = role_fill(frame, theme) {
        scene.push_quad(Quad::new(frame.rect(), fill).with_rounding(rounding));
    }

    if let Some(tint) = visual_tint_for(frame, theme) {
        scene.push_quad(Quad::new(frame.rect(), tint).with_rounding(rounding));
    }

    match frame.role() {
        view::node::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            paint_menu_row(frame, scene, theme);
        }
        view::node::Role::Separator => {
            paint_menu_separator(frame, scene, theme);
        }
        view::node::Role::Checkbox | view::node::Role::Radio => {
            choice::paint(frame, scene, theme);
        }
        view::node::Role::Slider => {
            slider::paint(frame, scene, theme);
        }
        view::node::Role::TextArea => {
            if let Some(text_area) = frame.text_area_layout() {
                text_area::paint(frame.rect(), text_area, scene, theme);
            }
        }
        view::node::Role::TextBox => {
            text_box::paint_selection(frame, scene, theme);
        }
        _ => {}
    }

    if frame.binding_source() == Some(context::Source::Menu)
        || frame.role() == view::node::Role::Separator
        || frame.role() == view::node::Role::Popup
    {
        // Popup panes and rows paint their own slot content.
    } else if frame.role() == view::node::Role::TextBox && text_box::paint_text(frame, scene) {
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
            .with_align(text_align_for(frame)),
        );
    }

    if frame.role() == view::node::Role::TextBox {
        text_box::paint_caret(frame, scene, theme);
    }

    if let Some(color) = focus_outline_color_for(frame, theme) {
        overlays.push(
            Outline::new(frame.rect(), color)
                .with_width(theme.focus().width as f32)
                .with_offset(theme.focus().offset)
                .with_rounding(rounding),
        );
    } else if let Some(color) = outline_color_for(frame, theme) {
        scene.push_outline(
            Outline::new(frame.rect(), color)
                .with_width(theme.focus().width as f32)
                .with_rounding(rounding),
        );
    }
}

fn layer_for(frame: &layout::frame::Frame) -> Layer {
    match frame.role() {
        view::node::Role::MenuBar | view::node::Role::Menu => Layer::Chrome,
        view::node::Role::Popup | view::node::Role::Separator => Layer::Popup,
        view::node::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            Layer::Popup
        }
        _ => Layer::Base,
    }
}

fn role_fill(frame: &layout::frame::Frame, theme: &Theme) -> Option<super::Color> {
    match frame.role() {
        view::node::Role::Root => Some(theme.surfaces().root),
        view::node::Role::MenuBar => Some(theme.menu().bar_background),
        view::node::Role::Menu => visible_fill(theme.menu().title_background),
        view::node::Role::Popup => None,
        view::node::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            visible_fill(theme.menu().row_background)
        }
        view::node::Role::Binding => Some(if frame.is_enabled() {
            theme.control().background
        } else {
            theme.control().disabled_background
        }),
        view::node::Role::Separator => None,
        view::node::Role::TextArea => Some(theme.text_input().area_background),
        view::node::Role::Button => Some(theme.control().button_background),
        view::node::Role::Checkbox | view::node::Role::Radio => Some(theme.choice().background),
        view::node::Role::Slider => Some(theme.slider().background),
        view::node::Role::TextBox => Some(theme.text_input().field_background),
        view::node::Role::Panel => Some(theme.surfaces().panel),
        view::node::Role::Label | view::node::Role::Stack => None,
    }
}

fn visible_fill(color: super::Color) -> Option<super::Color> {
    (color.channels().3 > 0).then_some(color)
}

fn role_rounding(frame: &layout::frame::Frame, theme: &Theme) -> super::Rounding {
    match frame.role() {
        view::node::Role::Popup => theme.floating_panel().rounding,
        view::node::Role::Menu => theme.control().rounding,
        view::node::Role::Binding if frame.binding_source() == Some(context::Source::Menu) => {
            theme.control().rounding
        }
        view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox
        | view::node::Role::Panel => theme.control().rounding,
        _ => super::Rounding::none(),
    }
}

fn paint_popup_shadow(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let popup = theme.floating_panel();
    scene.push_shadow(
        Shadow::new(
            frame.rect(),
            popup.shadow,
            popup.shadow_blur,
            popup.shadow_spread,
            Offset::new(0.0, popup.shadow_offset_y),
        )
        .with_rounding(role_rounding(frame, theme)),
    );
}

fn paint_popup_backdrop(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let popup = theme.floating_panel();
    scene.push_backdrop(
        Backdrop::new(frame.rect(), popup.backdrop_blur).with_rounding(role_rounding(frame, theme)),
    );
}

fn paint_popup_material(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let popup = theme.floating_panel();
    let fill = popup_material_fill(popup.backdrop_tint, popup.background);

    if fill.is_visible() {
        scene.push_quad(
            Quad::styled(frame.rect(), Style::filled_with(fill))
                .with_rounding(role_rounding(frame, theme)),
        );
    }
}

fn popup_material_fill(tint: Brush, background: Brush) -> Brush {
    if tint.is_visible() { tint } else { background }
}

fn paint_menu_row(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let slots = layout::control::menu_row_slots(frame.rect(), frame.menu_shortcut_width(), theme);
    let color = text_color_for(frame, theme);

    if frame.checked() == Some(true) {
        scene.push_icon(Icon::new(
            slots.glyph,
            framework_icon::Icon::phosphor(framework_icon::Id::new("check"))
                .with_style(framework_icon::Style::Bold),
            color,
            layout::control::control_content_extent(theme.menu().row_height, theme) as f32,
        ));
    }

    if let Some(label) = frame.label_text() {
        scene.push_text(
            Text::new(slots.label, label, color, TextWrap::None).with_align(TextAlign::Start),
        );
    }

    if let Some(shortcut) = frame.shortcut() {
        scene.push_text(
            Text::new(slots.shortcut, shortcut.as_str(), color, TextWrap::None)
                .with_align(TextAlign::End),
        );
    }
}

fn paint_menu_separator(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let slots = layout::control::menu_row_slots(frame.rect(), frame.menu_shortcut_width(), theme);

    scene.push_quad(Quad::new(slots.separator, theme.menu().separator));
}

fn visual_tint_for(frame: &layout::frame::Frame, theme: &Theme) -> Option<super::Color> {
    if !frame.is_enabled() {
        return None;
    }

    if frame.role() == view::node::Role::Menu {
        if frame.is_pressed() {
            return Some(theme.menu().title_pressed_tint);
        }

        if frame.is_active() {
            return Some(theme.menu().title_active_tint);
        }

        if frame.is_hovered() || frame.focus_visible() {
            return Some(theme.menu().title_hover_tint);
        }

        return None;
    }

    if frame.binding_source() == Some(context::Source::Menu) {
        if frame.is_pressed() {
            return Some(theme.menu().row_pressed_tint);
        }

        if frame.is_hovered() || frame.focus_visible() {
            return Some(theme.menu().row_hover_tint);
        }

        return None;
    }

    if !is_control_visual_role(frame.role()) {
        return None;
    }

    if frame.is_pressed() {
        Some(theme.control().pressed_tint)
    } else if frame.is_hovered() {
        Some(theme.control().hover_tint)
    } else {
        None
    }
}

fn is_control_visual_role(role: view::node::Role) -> bool {
    matches!(
        role,
        view::node::Role::Binding
            | view::node::Role::Button
            | view::node::Role::Checkbox
            | view::node::Role::Radio
            | view::node::Role::Slider
            | view::node::Role::TextBox
    )
}

fn text_for(frame: &layout::frame::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn text_rect_for(frame: &layout::frame::Frame, theme: &Theme) -> geometry::Rect {
    match frame.role() {
        view::node::Role::Checkbox | view::node::Role::Radio => {
            let choice = theme.choice();
            let mark_right = frame
                .rect()
                .x()
                .saturating_add(choice.mark_inset)
                .saturating_add(choice.mark_size);
            let x = mark_right.saturating_add(choice.label_gap);
            geometry::Rect::new(
                x,
                frame.rect().y(),
                frame.rect().right().saturating_sub(x),
                frame.rect().height(),
            )
        }
        view::node::Role::Slider => {
            let slider = theme.slider();
            let rect = frame.rect();
            geometry::Rect::new(
                rect.x().saturating_add(slider.inset),
                rect.y(),
                rect.width().min(slider.label_width),
                rect.height(),
            )
        }
        view::node::Role::TextBox => frame.text_box_text_rect(),
        _ => frame.rect(),
    }
}

fn text_color_for(frame: &layout::frame::Frame, theme: &Theme) -> super::Color {
    if !frame.is_enabled() {
        return theme.text().muted;
    }

    match frame.role() {
        view::node::Role::TextBox
            if frame
                .text_box()
                .is_some_and(|text_box| text_box.text().is_empty()) =>
        {
            theme.text().muted
        }
        view::node::Role::TextArea | view::node::Role::TextBox => theme.text().inverse,
        view::node::Role::Separator => theme.menu().separator,
        _ => theme.text().primary,
    }
}

fn text_wrap_for(frame: &layout::frame::Frame) -> TextWrap {
    match frame.text_wrap() {
        Some(view::control::Wrap::None) => TextWrap::None,
        Some(view::control::Wrap::Word) | None => TextWrap::WordOrGlyph,
    }
}

fn text_align_for(frame: &layout::frame::Frame) -> TextAlign {
    match frame.role() {
        view::node::Role::Menu => TextAlign::Center,
        _ => TextAlign::Start,
    }
}

fn outline_color_for(frame: &layout::frame::Frame, theme: &Theme) -> Option<super::Color> {
    matches!(
        frame.role(),
        view::node::Role::Popup
            | view::node::Role::TextArea
            | view::node::Role::Button
            | view::node::Role::Slider
            | view::node::Role::TextBox
            | view::node::Role::Panel
    )
    .then_some(theme.focus().outline)
}

fn focus_outline_color_for(frame: &layout::frame::Frame, theme: &Theme) -> Option<super::Color> {
    (frame.focus_visible() && is_focus_outline_role(frame.role())).then_some(theme.focus().color)
}

fn is_focus_outline_role(role: view::node::Role) -> bool {
    matches!(
        role,
        view::node::Role::Menu
            | view::node::Role::Binding
            | view::node::Role::TextArea
            | view::node::Role::Button
            | view::node::Role::Checkbox
            | view::node::Role::Radio
            | view::node::Role::Slider
            | view::node::Role::TextBox
    )
}
