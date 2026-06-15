use crate::{geometry, layout, paint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    palette: Palette,
    surfaces: Surfaces,
    text: Text,
    density: Density,
    roundings: Roundings,
    control: Control,
    menu: Menu,
    floating_panel: FloatingPanel,
    scroll: Scroll,
    effects: Effects,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    accent: paint::Color,
    accent_subtle: paint::Color,
    warning: paint::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surfaces {
    canvas: paint::Color,
    app: paint::Brush,
    panel: paint::Brush,
    panel_stroke: paint::Brush,
    separator: paint::Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Text {
    primary: paint::Color,
    secondary: paint::Color,
    disabled: paint::Color,
    busy: paint::Color,
    body_size: f32,
    control_size: f32,
    menu_size: f32,
    label_size: f32,
    icon_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Density {
    app_padding: f32,
    panel_padding: f32,
    control_height: f32,
    icon_button_height: f32,
    label_height: f32,
    menu_bar_height: f32,
    menu_row_height: f32,
    menu_popup_min_width: f32,
    menu_title_min_width: f32,
    menu_title_horizontal_padding: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roundings {
    panel: geometry::rect::Rounding,
    control: geometry::rect::Rounding,
    menu_title: geometry::rect::Rounding,
    popup: geometry::rect::Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Control {
    background: paint::Brush,
    stroke: paint::Stroke,
    hover_tint: paint::Brush,
    pressed_tint: paint::Brush,
    active_tint: paint::Brush,
    busy_tint: paint::Brush,
    disabled_tint: paint::Brush,
    focus_outline: Outline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Menu {
    bar_background: paint::Brush,
    bar_stroke: paint::Stroke,
    title_background: paint::Brush,
    title_hover_tint: paint::Brush,
    title_pressed_tint: paint::Brush,
    title_active_tint: paint::Brush,
    title_focus_outline: Outline,
    row_background: paint::Brush,
    row_hover_tint: paint::Brush,
    row_pressed_tint: paint::Brush,
    row_disabled_tint: paint::Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingPanel {
    backdrop_fill: paint::Brush,
    backdrop_blur: f32,
    stroke: paint::Stroke,
    shadow: Shadow,
    rounding: geometry::rect::Rounding,
    padding: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroll {
    thickness: f32,
    min_thumb_length: f32,
    track: paint::Brush,
    thumb: paint::Brush,
    thumb_hover_tint: paint::Brush,
    thumb_pressed_tint: paint::Brush,
    corner: paint::Brush,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Effects {
    popup_shadow: Shadow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    brush: paint::Brush,
    width: f32,
    offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    brush: paint::Brush,
    blur: f32,
    spread: f32,
    offset: geometry::point::Logical,
}

impl Theme {
    pub fn default_dark() -> Self {
        let accent = srgb8(10, 132, 255);
        let accent_subtle = srgba8(10, 132, 255, 0.24);
        let warning = srgb8(255, 190, 80);
        let scroll_track = paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.18));
        let popup_shadow = Shadow {
            brush: paint::Brush::linear_gradient(
                paint::Color::rgba(0.0, 0.0, 0.0, 0.24),
                paint::Color::rgba(0.0, 0.0, 0.0, 0.58),
            ),
            blur: 18.0,
            spread: 1.0,
            offset: geometry::point::logical(0.0, 7.0),
        };

        Self {
            palette: Palette {
                accent,
                accent_subtle,
                warning,
            },
            surfaces: Surfaces {
                canvas: srgb8(14, 15, 18),
                app: paint::Brush::linear_gradient(srgb8(14, 15, 18), srgb8(20, 22, 26)),
                panel: paint::Brush::linear_gradient(
                    srgba8(31, 33, 39, 0.98),
                    srgba8(24, 26, 31, 0.98),
                ),
                panel_stroke: paint::Brush::linear_gradient(
                    paint::Color::rgba(1.0, 1.0, 1.0, 0.07),
                    paint::Color::rgba(1.0, 1.0, 1.0, 0.13),
                ),
                separator: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.11)),
            },
            text: Text {
                primary: srgb8(232, 233, 236),
                secondary: srgb8(166, 168, 174),
                disabled: srgb8(94, 97, 105),
                busy: srgb8(255, 218, 138),
                body_size: 13.0,
                control_size: 13.0,
                menu_size: 13.0,
                label_size: 12.5,
                icon_size: 18.0,
            },
            density: Density {
                app_padding: 10.0,
                panel_padding: 8.0,
                control_height: 32.0,
                icon_button_height: 32.0,
                label_height: 22.0,
                menu_bar_height: 28.0,
                menu_row_height: 22.0,
                menu_popup_min_width: 192.0,
                menu_title_min_width: 48.0,
                menu_title_horizontal_padding: 24.0,
            },
            roundings: Roundings {
                panel: geometry::rect::Rounding::fixed(8.0),
                control: geometry::rect::Rounding::fixed(7.0),
                menu_title: geometry::rect::Rounding::none(),
                popup: geometry::rect::Rounding::fixed(10.0),
            },
            control: Control {
                background: paint::Brush::linear_gradient(
                    srgba8(38, 40, 47, 0.96),
                    srgba8(29, 31, 37, 0.96),
                ),
                stroke: paint::Stroke {
                    brush: paint::Brush::linear_gradient(
                        paint::Color::rgba(1.0, 1.0, 1.0, 0.08),
                        paint::Color::rgba(1.0, 1.0, 1.0, 0.16),
                    ),
                    width: 1.0,
                },
                hover_tint: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.07)),
                pressed_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.18)),
                active_tint: paint::Brush::solid(accent_subtle),
                busy_tint: paint::Brush::solid(srgba8(255, 184, 46, 0.30)),
                disabled_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.38)),
                focus_outline: Outline {
                    brush: paint::Brush::solid(paint::Color::rgba(0.18, 0.52, 1.0, 0.78)),
                    width: 1.0,
                    offset: 2.0,
                },
            },
            menu: Menu {
                bar_background: paint::Brush::solid(srgba8(19, 20, 24, 0.96)),
                bar_stroke: paint::Stroke {
                    brush: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.09)),
                    width: 1.0,
                },
                title_background: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.0)),
                title_hover_tint: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.08)),
                title_pressed_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.18)),
                title_active_tint: paint::Brush::solid(srgba8(10, 132, 255, 0.22)),
                title_focus_outline: Outline {
                    brush: paint::Brush::solid(srgba8(10, 132, 255, 0.72)),
                    width: 1.0,
                    offset: 1.0,
                },
                row_background: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.0)),
                row_hover_tint: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.09)),
                row_pressed_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.17)),
                row_disabled_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.30)),
            },
            floating_panel: FloatingPanel {
                backdrop_fill: paint::Brush::linear_gradient(
                    srgba8(24, 26, 32, 0.56),
                    srgba8(36, 40, 50, 0.66),
                ),
                backdrop_blur: 0.92,
                stroke: paint::Stroke {
                    brush: paint::Brush::linear_gradient(
                        paint::Color::rgba(1.0, 1.0, 1.0, 0.10),
                        paint::Color::rgba(1.0, 1.0, 1.0, 0.22),
                    ),
                    width: 1.0,
                },
                shadow: popup_shadow,
                rounding: geometry::rect::Rounding::fixed(10.0),
                padding: 6.0,
            },
            scroll: Scroll {
                thickness: 10.0,
                min_thumb_length: 18.0,
                track: scroll_track,
                thumb: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.24)),
                thumb_hover_tint: paint::Brush::solid(paint::Color::rgba(1.0, 1.0, 1.0, 0.10)),
                thumb_pressed_tint: paint::Brush::solid(paint::Color::rgba(0.0, 0.0, 0.0, 0.18)),
                corner: scroll_track,
            },
            effects: Effects { popup_shadow },
        }
    }

    pub fn palette(&self) -> Palette {
        self.palette
    }

    pub fn surfaces(&self) -> Surfaces {
        self.surfaces
    }

    pub fn text(&self) -> Text {
        self.text
    }

    pub fn density(&self) -> Density {
        self.density
    }

    pub fn roundings(&self) -> Roundings {
        self.roundings
    }

    pub fn control(&self) -> Control {
        self.control
    }

    pub fn menu(&self) -> Menu {
        self.menu
    }

    pub fn floating_panel(&self) -> FloatingPanel {
        self.floating_panel
    }

    pub fn scroll(&self) -> Scroll {
        self.scroll
    }

    pub fn effects(&self) -> Effects {
        self.effects
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}

impl Palette {
    pub fn accent(self) -> paint::Color {
        self.accent
    }

    pub fn accent_subtle(self) -> paint::Color {
        self.accent_subtle
    }

    pub fn warning(self) -> paint::Color {
        self.warning
    }
}

impl Surfaces {
    pub fn canvas(self) -> paint::Color {
        self.canvas
    }

    pub fn app(self) -> paint::Brush {
        self.app
    }

    pub fn panel(self) -> paint::Brush {
        self.panel
    }

    pub fn panel_stroke(self) -> paint::Brush {
        self.panel_stroke
    }

    pub fn separator(self) -> paint::Brush {
        self.separator
    }
}

impl Text {
    pub fn primary(self) -> paint::Color {
        self.primary
    }

    pub fn secondary(self) -> paint::Color {
        self.secondary
    }

    pub fn disabled(self) -> paint::Color {
        self.disabled
    }

    pub fn busy(self) -> paint::Color {
        self.busy
    }

    pub fn body_size(self) -> f32 {
        self.body_size
    }

    pub fn control_size(self) -> f32 {
        self.control_size
    }

    pub fn menu_size(self) -> f32 {
        self.menu_size
    }

    pub fn label_size(self) -> f32 {
        self.label_size
    }

    pub fn icon_size(self) -> f32 {
        self.icon_size
    }
}

impl Density {
    pub fn app_padding(self) -> f32 {
        self.app_padding
    }

    pub fn panel_padding(self) -> f32 {
        self.panel_padding
    }

    pub fn control_height(self) -> f32 {
        self.control_height
    }

    pub fn icon_button_height(self) -> f32 {
        self.icon_button_height
    }

    pub fn label_height(self) -> f32 {
        self.label_height
    }

    pub fn menu_bar_height(self) -> f32 {
        self.menu_bar_height
    }

    pub fn menu_row_height(self) -> f32 {
        self.menu_row_height
    }

    pub fn menu_popup_min_width(self) -> f32 {
        self.menu_popup_min_width
    }

    pub fn menu_title_min_width(self) -> f32 {
        self.menu_title_min_width
    }

    pub fn menu_title_horizontal_padding(self) -> f32 {
        self.menu_title_horizontal_padding
    }
}

impl Roundings {
    pub fn panel(self) -> geometry::rect::Rounding {
        self.panel
    }

    pub fn control(self) -> geometry::rect::Rounding {
        self.control
    }

    pub fn menu_title(self) -> geometry::rect::Rounding {
        self.menu_title
    }

    pub fn popup(self) -> geometry::rect::Rounding {
        self.popup
    }
}

impl Control {
    pub fn background(self) -> paint::Brush {
        self.background
    }

    pub fn stroke(self) -> paint::Stroke {
        self.stroke
    }

    pub fn hover_tint(self) -> paint::Brush {
        self.hover_tint
    }

    pub fn pressed_tint(self) -> paint::Brush {
        self.pressed_tint
    }

    pub fn active_tint(self) -> paint::Brush {
        self.active_tint
    }

    pub fn busy_tint(self) -> paint::Brush {
        self.busy_tint
    }

    pub fn disabled_tint(self) -> paint::Brush {
        self.disabled_tint
    }

    pub fn focus_outline(self) -> Outline {
        self.focus_outline
    }
}

impl Menu {
    pub fn bar_background(self) -> paint::Brush {
        self.bar_background
    }

    pub fn bar_stroke(self) -> paint::Stroke {
        self.bar_stroke
    }

    pub fn title_background(self) -> paint::Brush {
        self.title_background
    }

    pub fn title_hover_tint(self) -> paint::Brush {
        self.title_hover_tint
    }

    pub fn title_pressed_tint(self) -> paint::Brush {
        self.title_pressed_tint
    }

    pub fn title_active_tint(self) -> paint::Brush {
        self.title_active_tint
    }

    pub fn title_focus_outline(self) -> Outline {
        self.title_focus_outline
    }

    pub fn row_background(self) -> paint::Brush {
        self.row_background
    }

    pub fn row_hover_tint(self) -> paint::Brush {
        self.row_hover_tint
    }

    pub fn row_pressed_tint(self) -> paint::Brush {
        self.row_pressed_tint
    }

    pub fn row_disabled_tint(self) -> paint::Brush {
        self.row_disabled_tint
    }
}

impl FloatingPanel {
    pub fn backdrop_fill(self) -> paint::Brush {
        self.backdrop_fill
    }

    pub fn backdrop_blur(self) -> f32 {
        self.backdrop_blur
    }

    pub fn stroke(self) -> paint::Stroke {
        self.stroke
    }

    pub fn shadow(self) -> Shadow {
        self.shadow
    }

    pub fn rounding(self) -> geometry::rect::Rounding {
        self.rounding
    }

    pub fn padding(self) -> f32 {
        self.padding
    }
}

impl Scroll {
    pub fn thickness(self) -> f32 {
        self.thickness
    }

    pub fn min_thumb_length(self) -> f32 {
        self.min_thumb_length
    }

    pub fn track(self) -> paint::Brush {
        self.track
    }

    pub fn thumb(self) -> paint::Brush {
        self.thumb
    }

    pub fn thumb_hover_tint(self) -> paint::Brush {
        self.thumb_hover_tint
    }

    pub fn thumb_pressed_tint(self) -> paint::Brush {
        self.thumb_pressed_tint
    }

    pub fn corner(self) -> paint::Brush {
        self.corner
    }
}

impl Effects {
    pub fn popup_shadow(self) -> Shadow {
        self.popup_shadow
    }
}

impl Outline {
    pub fn brush(self) -> paint::Brush {
        self.brush
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn offset(self) -> f32 {
        self.offset
    }
}

impl Shadow {
    pub fn brush(self) -> paint::Brush {
        self.brush
    }

    pub fn blur(self) -> f32 {
        self.blur
    }

    pub fn spread(self) -> f32 {
        self.spread
    }

    pub fn offset(self) -> geometry::point::Logical {
        self.offset
    }
}

pub fn stroke(brush: paint::Brush, width: f32) -> paint::Stroke {
    paint::Stroke { brush, width }
}

pub fn insets(value: f32) -> layout::Insets {
    layout::Insets::splat(value)
}

fn srgb8(r: u8, g: u8, b: u8) -> paint::Color {
    paint::Color::rgb(linear_channel(r), linear_channel(g), linear_channel(b))
}

fn srgba8(r: u8, g: u8, b: u8, a: f32) -> paint::Color {
    paint::Color::rgba(
        linear_channel(r),
        linear_channel(g),
        linear_channel(b),
        a.clamp(0.0, 1.0),
    )
}

fn linear_channel(channel: u8) -> f32 {
    let value = channel as f32 / 255.0;

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_dark_exposes_all_token_groups() {
        let theme = Theme::default_dark();

        assert!(theme.palette().accent().a > 0.0);
        assert!(theme.surfaces().panel().is_visible());
        assert!(theme.text().primary().a > 0.0);
        assert!(theme.density().control_height() > 0.0);
        assert_ne!(
            theme.roundings().control(),
            geometry::rect::Rounding::none()
        );
        assert!(theme.control().background().is_visible());
        assert!(theme.floating_panel().backdrop_fill().is_visible());
        assert!(theme.floating_panel().shadow().blur() > 0.0);
        assert!(theme.scroll().thickness() > 0.0);
        assert!(theme.scroll().thumb().is_visible());
        assert!(theme.effects().popup_shadow().blur() > 0.0);
    }

    #[test]
    fn default_density_is_compact_desktop_scale() {
        let density = Theme::default_dark().density();

        assert!(density.control_height() <= 34.0);
        assert!(density.menu_row_height() <= 24.0);
        assert!(density.app_padding() <= 12.0);
    }

    #[test]
    fn default_menu_rows_are_compact_without_shrinking_text() {
        let theme = Theme::default_dark();

        assert_eq!(theme.density().menu_row_height(), 22.0);
        assert_eq!(theme.text().menu_size(), 13.0);
    }

    #[test]
    fn default_canvas_is_authored_as_dark_linear_color() {
        let canvas = Theme::default_dark().surfaces().canvas();

        assert!(canvas.r < 0.01);
        assert!(canvas.g < 0.01);
        assert!(canvas.b < 0.01);
    }
}
