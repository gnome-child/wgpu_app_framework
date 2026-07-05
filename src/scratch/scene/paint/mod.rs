mod choice;
mod slider;
mod text_area;
mod text_box;

use super::super::{context, geometry, layout, theme::Theme, view};
use super::{Offset, Outline, Quad, Scene, Shadow, Text, TextWrap};

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
    }

    if let Some(fill) = role_fill(frame, theme) {
        scene.push_quad(Quad::new(frame.rect(), fill).with_rounding(role_rounding(frame, theme)));
    }

    match frame.role() {
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

    if frame.role() == view::node::Role::TextBox && text_box::paint_text(frame, scene) {
        // TextBox contents use the shaped field viewport so glyphs, selection,
        // caret, and horizontal scroll share one coordinate system.
    } else if let Some(value) = text_for(frame) {
        scene.push_text(Text::new(
            text_rect_for(frame, theme),
            value,
            text_color_for(frame, theme),
            text_wrap_for(frame),
        ));
    }

    if frame.role() == view::node::Role::TextBox {
        text_box::paint_caret(frame, scene, theme);
    }

    if let Some(color) = focus_outline_color_for(frame, theme) {
        overlays.push(
            Outline::new(frame.rect(), color).with_width(theme.metrics().focus_width as f32),
        );
    } else if let Some(color) = outline_color_for(frame, theme) {
        scene.push_outline(
            Outline::new(frame.rect(), color).with_width(theme.metrics().focus_width as f32),
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
    let roles = theme.roles();
    match frame.role() {
        view::node::Role::Root => Some(roles.root),
        view::node::Role::MenuBar => Some(roles.menu_bar),
        view::node::Role::Menu => Some(roles.menu),
        view::node::Role::Popup => Some(roles.popup),
        view::node::Role::Binding => Some(if frame.is_enabled() {
            roles.binding
        } else {
            roles.binding_disabled
        }),
        view::node::Role::Separator => Some(roles.separator),
        view::node::Role::TextArea => Some(roles.text_area),
        view::node::Role::Button => Some(roles.button),
        view::node::Role::Checkbox | view::node::Role::Radio => Some(roles.choice),
        view::node::Role::Slider => Some(roles.slider),
        view::node::Role::TextBox => Some(roles.text_box),
        view::node::Role::Panel => Some(roles.panel),
        view::node::Role::Label | view::node::Role::Stack => None,
    }
}

fn role_rounding(frame: &layout::frame::Frame, theme: &Theme) -> super::Rounding {
    match frame.role() {
        view::node::Role::Popup => theme.metrics().popup_rounding,
        view::node::Role::Button
        | view::node::Role::Checkbox
        | view::node::Role::Radio
        | view::node::Role::Slider
        | view::node::Role::TextBox
        | view::node::Role::Panel => theme.metrics().control_rounding,
        _ => super::Rounding::none(),
    }
}

fn paint_popup_shadow(frame: &layout::frame::Frame, scene: &mut Scene, theme: &Theme) {
    let metrics = theme.metrics();
    scene.push_shadow(Shadow::new(
        frame.rect(),
        theme.controls().popup_shadow,
        metrics.popup_shadow_blur,
        metrics.popup_shadow_spread,
        Offset::new(0.0, metrics.popup_shadow_offset_y),
    ));
}

fn text_for(frame: &layout::frame::Frame) -> Option<&str> {
    frame.label_text().or_else(|| frame.text())
}

fn text_rect_for(frame: &layout::frame::Frame, theme: &Theme) -> geometry::Rect {
    match frame.role() {
        view::node::Role::Checkbox | view::node::Role::Radio => {
            let metrics = theme.metrics();
            let mark_right = frame
                .rect()
                .x()
                .saturating_add(metrics.choice_mark_inset)
                .saturating_add(metrics.choice_mark_size);
            let x = mark_right.saturating_add(metrics.choice_label_gap);
            geometry::Rect::new(
                x,
                frame.rect().y(),
                frame.rect().right().saturating_sub(x),
                frame.rect().height(),
            )
        }
        view::node::Role::Slider => {
            let metrics = theme.metrics();
            let rect = frame.rect();
            geometry::Rect::new(
                rect.x().saturating_add(metrics.slider_inset),
                rect.y(),
                rect.width().min(metrics.slider_label_width),
                rect.height(),
            )
        }
        view::node::Role::TextBox => frame.text_box_text_rect(),
        _ => frame.rect(),
    }
}

fn text_color_for(frame: &layout::frame::Frame, theme: &Theme) -> super::Color {
    if !frame.is_enabled() {
        return theme.palette().text_muted;
    }

    match frame.role() {
        view::node::Role::TextBox
            if frame
                .text_box()
                .is_some_and(|text_box| text_box.text().is_empty()) =>
        {
            theme.palette().text_muted
        }
        view::node::Role::TextArea | view::node::Role::TextBox => theme.palette().text_inverse,
        view::node::Role::Separator => theme.roles().separator,
        _ => theme.palette().text,
    }
}

fn text_wrap_for(frame: &layout::frame::Frame) -> TextWrap {
    match frame.text_wrap() {
        Some(view::control::Wrap::None) => TextWrap::None,
        Some(view::control::Wrap::Word) | None => TextWrap::WordOrGlyph,
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
    .then_some(theme.palette().outline)
}

fn focus_outline_color_for(frame: &layout::frame::Frame, theme: &Theme) -> Option<super::Color> {
    (frame.is_focused() && is_focus_outline_role(frame.role())).then_some(theme.palette().focus)
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
