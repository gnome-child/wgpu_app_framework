mod toml;

pub use self::toml::ThemeTomlError;

use crate::text as text_model;

use super::keymap;
use super::scene;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    variant: Variant,
    palette: Palette,
    surfaces: Surfaces,
    text: Text,
    typography: Typography,
    focus: Focus,
    control: Control,
    menu: Menu,
    choice: Choice,
    slider: Slider,
    text_input: TextInput,
    floating_panel: FloatingPanel,
    viewport: Viewport,
    scrollbar: Scrollbar,
    command_palette: CommandPalette,
    shortcuts: Shortcuts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub(in crate::scratch) accent: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surfaces {
    pub(in crate::scratch) canvas: scene::Color,
    pub(in crate::scratch) root: scene::Color,
    pub(in crate::scratch) panel: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    pub(in crate::scratch) primary: scene::Color,
    pub(in crate::scratch) inverse: scene::Color,
    pub(in crate::scratch) muted: scene::Color,
    pub(in crate::scratch) selection: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    pub(in crate::scratch) interface: TypeStyle,
    pub(in crate::scratch) body: TypeStyle,
    pub(in crate::scratch) caption: TypeStyle,
    pub(in crate::scratch) hint: TypeStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    pub(in crate::scratch) size: f32,
    pub(in crate::scratch) weight: text_model::document::Weight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus {
    pub(in crate::scratch) color: scene::Color,
    pub(in crate::scratch) outline: scene::Color,
    pub(in crate::scratch) width: i32,
    pub(in crate::scratch) offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Control {
    pub(in crate::scratch) background: scene::Color,
    pub(in crate::scratch) button_background: scene::Color,
    pub(in crate::scratch) disabled_background: scene::Color,
    pub(in crate::scratch) hover_tint: scene::Color,
    pub(in crate::scratch) pressed_tint: scene::Color,
    pub(in crate::scratch) rounding: scene::Rounding,
    pub(in crate::scratch) height: i32,
    pub(in crate::scratch) padding: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Menu {
    pub(in crate::scratch) bar_background: scene::Color,
    pub(in crate::scratch) title_background: scene::Color,
    pub(in crate::scratch) title_hover_tint: scene::Color,
    pub(in crate::scratch) title_pressed_tint: scene::Color,
    pub(in crate::scratch) title_active_tint: scene::Color,
    pub(in crate::scratch) row_background: scene::Color,
    pub(in crate::scratch) row_hover_tint: scene::Color,
    pub(in crate::scratch) row_pressed_tint: scene::Color,
    pub(in crate::scratch) separator: scene::Color,
    pub(in crate::scratch) bar_height: i32,
    pub(in crate::scratch) row_height: i32,
    pub(in crate::scratch) separator_line_height: i32,
    pub(in crate::scratch) panel_min_width: i32,
    pub(in crate::scratch) padding: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Choice {
    pub(in crate::scratch) background: scene::Color,
    pub(in crate::scratch) mark: scene::Color,
    pub(in crate::scratch) outline: scene::Color,
    pub(in crate::scratch) indicator: scene::Color,
    pub(in crate::scratch) mark_size: i32,
    pub(in crate::scratch) mark_inset: i32,
    pub(in crate::scratch) label_gap: i32,
    pub(in crate::scratch) icon_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slider {
    pub(in crate::scratch) background: scene::Color,
    pub(in crate::scratch) track: scene::Color,
    pub(in crate::scratch) value: scene::Color,
    pub(in crate::scratch) thumb: scene::Color,
    pub(in crate::scratch) thumb_outline: scene::Color,
    pub(in crate::scratch) label_width: i32,
    pub(in crate::scratch) inset: i32,
    pub(in crate::scratch) gap: i32,
    pub(in crate::scratch) track_height: i32,
    pub(in crate::scratch) thumb_width: i32,
    pub(in crate::scratch) thumb_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextInput {
    pub(in crate::scratch) area_background: scene::Color,
    pub(in crate::scratch) field_background: scene::Color,
    pub(in crate::scratch) foreground: scene::Color,
    pub(in crate::scratch) placeholder: scene::Color,
    pub(in crate::scratch) caret: scene::Color,
    pub(in crate::scratch) padding_x: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingPanel {
    pub(in crate::scratch) material: scene::Material,
    pub(in crate::scratch) rounding: scene::Rounding,
    pub(in crate::scratch) shadow: scene::Color,
    pub(in crate::scratch) shadow_blur: f32,
    pub(in crate::scratch) shadow_spread: f32,
    pub(in crate::scratch) shadow_offset_y: f32,
    pub(in crate::scratch) padding: i32,
    pub(in crate::scratch) content_gap: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub(in crate::scratch) min_viewport_extent: i32,
    pub(in crate::scratch) reveal_margin: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrollbar {
    pub(in crate::scratch) metrics: ScrollbarMetrics,
    pub(in crate::scratch) appearance: ScrollbarAppearance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarMetrics {
    pub(in crate::scratch) thickness: i32,
    pub(in crate::scratch) policy: ScrollbarPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    OverlayAuto,
    GutterAlways,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarAppearance {
    pub(in crate::scratch) overlay_thickness: i32,
    pub(in crate::scratch) hover_thickness: i32,
    pub(in crate::scratch) min_thumb_length: i32,
    pub(in crate::scratch) margin: i32,
    pub(in crate::scratch) fade_delay_ms: u64,
    pub(in crate::scratch) fade_duration_ms: u64,
    pub(in crate::scratch) track: scene::Color,
    pub(in crate::scratch) thumb: scene::Color,
    pub(in crate::scratch) thumb_hover_tint: scene::Color,
    pub(in crate::scratch) thumb_pressed_tint: scene::Color,
    pub(in crate::scratch) corner: scene::Color,
    pub(in crate::scratch) rounding: scene::Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPalette {
    pub(in crate::scratch) section_alignment: scene::TextAlign,
    pub(in crate::scratch) max_results_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcuts {
    pub(in crate::scratch) display: keymap::DisplayStyle,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            variant: Variant::Dark,
            palette: Palette {
                accent: scene::Color::rgb(10, 132, 255),
            },
            surfaces: Surfaces {
                canvas: scene::Color::rgb(17, 18, 20),
                root: scene::Color::rgb(17, 18, 20),
                panel: scene::Color::rgba(0, 0, 0, 0),
            },
            text: Text {
                primary: scene::Color::rgb(245, 245, 247),
                inverse: scene::Color::rgb(29, 29, 31),
                muted: scene::Color::rgb(161, 161, 166),
                selection: scene::Color::rgba(10, 132, 255, 96),
            },
            typography: Typography {
                interface: TypeStyle::new(12.0, text_model::document::Weight::Normal),
                body: TypeStyle::new(16.0, text_model::document::Weight::Normal),
                caption: TypeStyle::new(11.0, text_model::document::Weight::Medium),
                hint: TypeStyle::new(12.0, text_model::document::Weight::Normal),
            },
            focus: Focus {
                color: scene::Color::rgb(10, 132, 255),
                outline: scene::Color::rgba(0, 0, 0, 0),
                width: 1,
                offset: 2.0,
            },
            control: Control {
                background: scene::Color::rgba(0, 0, 0, 0),
                button_background: scene::Color::rgb(44, 44, 46),
                disabled_background: scene::Color::rgba(0, 0, 0, 0),
                hover_tint: scene::Color::rgba(255, 255, 255, 22),
                pressed_tint: scene::Color::rgba(255, 255, 255, 13),
                rounding: scene::Rounding::fixed(4.0),
                height: 22,
                padding: 4,
            },
            menu: Menu {
                bar_background: scene::Color::rgb(28, 28, 30),
                title_background: scene::Color::rgba(0, 0, 0, 0),
                title_hover_tint: scene::Color::rgba(255, 255, 255, 24),
                title_pressed_tint: scene::Color::rgba(255, 255, 255, 14),
                title_active_tint: scene::Color::rgba(10, 132, 255, 52),
                row_background: scene::Color::rgba(0, 0, 0, 0),
                row_hover_tint: scene::Color::rgba(255, 255, 255, 7),
                row_pressed_tint: scene::Color::rgba(255, 255, 255, 12),
                separator: scene::Color::rgb(58, 58, 60),
                bar_height: 22,
                row_height: 22,
                separator_line_height: 1,
                panel_min_width: 220,
                padding: 4,
            },
            choice: Choice {
                background: scene::Color::rgba(0, 0, 0, 0),
                mark: scene::Color::rgb(245, 245, 247),
                outline: scene::Color::rgba(0, 0, 0, 0),
                indicator: scene::Color::rgb(10, 132, 255),
                mark_size: 14,
                mark_inset: 8,
                label_gap: 8,
                icon_size: 13.0,
            },
            slider: Slider {
                background: scene::Color::rgba(0, 0, 0, 0),
                track: scene::Color::rgb(58, 58, 60),
                value: scene::Color::rgb(10, 132, 255),
                thumb: scene::Color::rgb(245, 245, 247),
                thumb_outline: scene::Color::rgba(0, 0, 0, 0),
                label_width: 112,
                inset: 8,
                gap: 10,
                track_height: 4,
                thumb_width: 8,
                thumb_height: 18,
            },
            text_input: TextInput {
                area_background: scene::Color::rgb(28, 28, 30),
                field_background: scene::Color::rgb(44, 44, 46),
                foreground: scene::Color::rgb(245, 245, 247),
                placeholder: scene::Color::rgb(142, 142, 147),
                caret: scene::Color::rgb(245, 245, 247),
                padding_x: 8,
            },
            floating_panel: FloatingPanel {
                material: scene::Material::glass(scene::Glass::panel_dark()),
                rounding: scene::Rounding::fixed(10.0),
                shadow: scene::Color::rgba(0, 0, 0, 96),
                shadow_blur: 24.0,
                shadow_spread: 0.5,
                shadow_offset_y: 10.0,
                padding: 6,
                content_gap: 6,
            },
            viewport: Viewport {
                min_viewport_extent: 96,
                reveal_margin: 0,
            },
            scrollbar: Scrollbar {
                metrics: ScrollbarMetrics {
                    thickness: 10,
                    policy: ScrollbarPolicy::OverlayAuto,
                },
                appearance: ScrollbarAppearance {
                    overlay_thickness: 10,
                    hover_thickness: 14,
                    min_thumb_length: 18,
                    margin: 2,
                    fade_delay_ms: 650,
                    fade_duration_ms: 180,
                    track: scene::Color::rgba(0, 0, 0, 107),
                    thumb: scene::Color::rgba(255, 255, 255, 71),
                    thumb_hover_tint: scene::Color::rgba(255, 255, 255, 31),
                    thumb_pressed_tint: scene::Color::rgba(0, 0, 0, 41),
                    corner: scene::Color::rgba(0, 0, 0, 107),
                    rounding: scene::Rounding::relative(1.0),
                },
            },
            command_palette: CommandPalette {
                section_alignment: scene::TextAlign::Center,
                max_results_height: 260,
            },
            shortcuts: Shortcuts {
                display: keymap::DisplayStyle::Default,
            },
        }
    }

    pub fn light() -> Self {
        Self {
            variant: Variant::Light,
            palette: Palette {
                accent: scene::Color::rgb(42, 104, 230),
            },
            surfaces: Surfaces {
                canvas: scene::Color::rgb(245, 247, 250),
                root: scene::Color::rgb(245, 247, 250),
                panel: scene::Color::rgba(0, 0, 0, 0),
            },
            text: Text {
                primary: scene::Color::rgb(28, 31, 36),
                inverse: scene::Color::rgb(28, 31, 36),
                muted: scene::Color::rgb(96, 105, 116),
                selection: scene::Color::rgba(42, 104, 230, 72),
            },
            typography: Typography {
                interface: TypeStyle::new(12.0, text_model::document::Weight::Normal),
                body: TypeStyle::new(16.0, text_model::document::Weight::Normal),
                caption: TypeStyle::new(11.0, text_model::document::Weight::Medium),
                hint: TypeStyle::new(12.0, text_model::document::Weight::Normal),
            },
            focus: Focus {
                color: scene::Color::rgb(42, 104, 230),
                outline: scene::Color::rgba(0, 0, 0, 0),
                width: 1,
                offset: 2.0,
            },
            control: Control {
                background: scene::Color::rgba(0, 0, 0, 0),
                button_background: scene::Color::rgb(232, 236, 242),
                disabled_background: scene::Color::rgba(0, 0, 0, 0),
                hover_tint: scene::Color::rgba(20, 22, 25, 14),
                pressed_tint: scene::Color::rgba(20, 22, 25, 28),
                rounding: scene::Rounding::fixed(4.0),
                height: 22,
                padding: 4,
            },
            menu: Menu {
                bar_background: scene::Color::rgb(232, 236, 242),
                title_background: scene::Color::rgba(0, 0, 0, 0),
                title_hover_tint: scene::Color::rgba(20, 22, 25, 18),
                title_pressed_tint: scene::Color::rgba(20, 22, 25, 30),
                title_active_tint: scene::Color::rgba(42, 104, 230, 36),
                row_background: scene::Color::rgba(0, 0, 0, 0),
                row_hover_tint: scene::Color::rgba(20, 22, 25, 8),
                row_pressed_tint: scene::Color::rgba(20, 22, 25, 18),
                separator: scene::Color::rgb(203, 211, 222),
                bar_height: 22,
                row_height: 22,
                separator_line_height: 1,
                panel_min_width: 220,
                padding: 4,
            },
            choice: Choice {
                background: scene::Color::rgba(0, 0, 0, 0),
                mark: scene::Color::rgb(255, 255, 255),
                outline: scene::Color::rgba(0, 0, 0, 0),
                indicator: scene::Color::rgb(42, 104, 230),
                mark_size: 14,
                mark_inset: 8,
                label_gap: 8,
                icon_size: 13.0,
            },
            slider: Slider {
                background: scene::Color::rgba(0, 0, 0, 0),
                track: scene::Color::rgb(185, 193, 204),
                value: scene::Color::rgb(42, 104, 230),
                thumb: scene::Color::rgb(255, 255, 255),
                thumb_outline: scene::Color::rgba(0, 0, 0, 0),
                label_width: 112,
                inset: 8,
                gap: 10,
                track_height: 4,
                thumb_width: 8,
                thumb_height: 18,
            },
            text_input: TextInput {
                area_background: scene::Color::rgb(255, 255, 255),
                field_background: scene::Color::rgb(255, 255, 255),
                foreground: scene::Color::rgb(29, 29, 31),
                placeholder: scene::Color::rgb(110, 110, 115),
                caret: scene::Color::rgb(29, 29, 31),
                padding_x: 8,
            },
            floating_panel: FloatingPanel {
                material: scene::Material::glass(scene::Glass::panel_light()),
                rounding: scene::Rounding::fixed(10.0),
                shadow: scene::Color::rgba(20, 22, 25, 48),
                shadow_blur: 24.0,
                shadow_spread: 0.5,
                shadow_offset_y: 10.0,
                padding: 6,
                content_gap: 6,
            },
            viewport: Viewport {
                min_viewport_extent: 96,
                reveal_margin: 0,
            },
            scrollbar: Scrollbar {
                metrics: ScrollbarMetrics {
                    thickness: 10,
                    policy: ScrollbarPolicy::OverlayAuto,
                },
                appearance: ScrollbarAppearance {
                    overlay_thickness: 10,
                    hover_thickness: 14,
                    min_thumb_length: 18,
                    margin: 2,
                    fade_delay_ms: 650,
                    fade_duration_ms: 180,
                    track: scene::Color::rgba(0, 0, 0, 36),
                    thumb: scene::Color::rgba(20, 22, 25, 72),
                    thumb_hover_tint: scene::Color::rgba(20, 22, 25, 30),
                    thumb_pressed_tint: scene::Color::rgba(20, 22, 25, 42),
                    corner: scene::Color::rgba(0, 0, 0, 36),
                    rounding: scene::Rounding::relative(1.0),
                },
            },
            command_palette: CommandPalette {
                section_alignment: scene::TextAlign::Center,
                max_results_height: 260,
            },
            shortcuts: Shortcuts {
                display: keymap::DisplayStyle::Default,
            },
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn from_toml_str(input: &str) -> Result<Self, ThemeTomlError> {
        toml::theme_from_str(input)
    }

    pub fn to_toml_string(&self) -> Result<String, ThemeTomlError> {
        toml::theme_to_string(self)
    }

    pub fn palette(&self) -> Palette {
        self.palette
    }

    pub(in crate::scratch) fn surfaces(&self) -> &Surfaces {
        &self.surfaces
    }

    pub(in crate::scratch) fn text(&self) -> &Text {
        &self.text
    }

    pub(in crate::scratch) fn typography(&self) -> Typography {
        self.typography
    }

    pub(in crate::scratch) fn focus(&self) -> &Focus {
        &self.focus
    }

    pub(in crate::scratch) fn control(&self) -> &Control {
        &self.control
    }

    pub(in crate::scratch) fn menu(&self) -> &Menu {
        &self.menu
    }

    pub(in crate::scratch) fn choice(&self) -> &Choice {
        &self.choice
    }

    pub(in crate::scratch) fn slider(&self) -> &Slider {
        &self.slider
    }

    pub(in crate::scratch) fn text_input(&self) -> &TextInput {
        &self.text_input
    }

    pub(in crate::scratch) fn floating_panel(&self) -> &FloatingPanel {
        &self.floating_panel
    }

    pub(in crate::scratch) fn floating_panel_mut(&mut self) -> &mut FloatingPanel {
        &mut self.floating_panel
    }

    pub(in crate::scratch) fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    #[cfg(test)]
    pub(in crate::scratch) fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    pub(in crate::scratch) fn scrollbar(&self) -> &Scrollbar {
        &self.scrollbar
    }

    pub(in crate::scratch) fn command_palette(&self) -> CommandPalette {
        self.command_palette
    }

    pub(in crate::scratch) fn shortcuts(&self) -> Shortcuts {
        self.shortcuts
    }

    #[cfg(test)]
    pub(in crate::scratch) fn scrollbar_mut(&mut self) -> &mut Scrollbar {
        &mut self.scrollbar
    }
}

impl Typography {
    pub(in crate::scratch) fn interface(self) -> TypeStyle {
        self.interface
    }

    pub(in crate::scratch) fn body(self) -> TypeStyle {
        self.body
    }

    pub(in crate::scratch) fn caption(self) -> TypeStyle {
        self.caption
    }

    #[allow(dead_code)]
    pub(in crate::scratch) fn hint(self) -> TypeStyle {
        self.hint
    }
}

impl TypeStyle {
    pub const fn new(size: f32, weight: text_model::document::Weight) -> Self {
        Self { size, weight }
    }

    pub(in crate::scratch) fn size(self) -> f32 {
        self.size
    }

    pub(in crate::scratch) fn weight(self) -> text_model::document::Weight {
        self.weight
    }

    pub(in crate::scratch) fn document_style(
        self,
        color: text_model::Color,
    ) -> text_model::document::Style {
        text_model::document::Style::default()
            .with_size(self.size.max(1.0))
            .with_weight(self.weight)
            .with_color(color)
    }
}

impl CommandPalette {
    pub(in crate::scratch) fn section_alignment(self) -> scene::TextAlign {
        self.section_alignment
    }

    pub(in crate::scratch) fn max_results_height(self) -> i32 {
        self.max_results_height
    }
}

impl Shortcuts {
    pub(in crate::scratch) fn display(self) -> keymap::DisplayStyle {
        self.display
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Palette {
    pub fn accent(self) -> scene::Color {
        self.accent
    }
}
