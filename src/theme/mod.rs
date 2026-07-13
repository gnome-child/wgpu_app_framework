mod toml;

pub use self::toml::ThemeTomlError;

use crate::text as text_model;

use super::keymap;
use super::scene;

pub(crate) const DEFAULT_CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 18, 20);

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
    overlay: Overlay,
    viewport: Viewport,
    scrollbar: Scrollbar,
    command_palette: CommandPalette,
    shortcuts: Shortcuts,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Table {
    pub(crate) header_background: scene::Color,
    pub(crate) header_hover_tint: scene::Color,
    pub(crate) header_pressed_tint: scene::Color,
    pub(crate) cell_background: scene::Color,
    pub(crate) alternate_row_tint: scene::Color,
    pub(crate) passive_indicator: scene::Color,
    pub(crate) cell_padding: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub(crate) accent: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surfaces {
    pub(crate) canvas: scene::Color,
    pub(crate) root: scene::Color,
    pub(crate) panel: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    pub(crate) primary: scene::Color,
    pub(crate) inverse: scene::Color,
    pub(crate) muted: scene::Color,
    pub(crate) selection: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    pub(crate) interface: TypeStyle,
    pub(crate) body: TypeStyle,
    pub(crate) caption: TypeStyle,
    pub(crate) hint: TypeStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    pub(crate) size: f32,
    pub(crate) weight: text_model::document::Weight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus {
    pub(crate) color: scene::Color,
    pub(crate) outline: scene::Color,
    pub(crate) width: i32,
    pub(crate) offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Control {
    pub(crate) background: scene::Color,
    pub(crate) button_background: scene::Color,
    pub(crate) disabled_background: scene::Color,
    pub(crate) hover_tint: scene::Color,
    pub(crate) pressed_tint: scene::Color,
    pub(crate) rounding: scene::Rounding,
    pub(crate) height: i32,
    pub(crate) padding: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Menu {
    pub(crate) bar_background: scene::Color,
    pub(crate) title_background: scene::Color,
    pub(crate) title_hover_tint: scene::Color,
    pub(crate) title_pressed_tint: scene::Color,
    pub(crate) title_active_tint: scene::Color,
    pub(crate) row_background: scene::Color,
    pub(crate) row_hover_tint: scene::Color,
    pub(crate) row_pressed_tint: scene::Color,
    pub(crate) separator: scene::Color,
    pub(crate) bar_height: i32,
    pub(crate) row_height: i32,
    pub(crate) separator_line_height: i32,
    pub(crate) panel_min_width: i32,
    pub(crate) padding: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Choice {
    pub(crate) background: scene::Color,
    pub(crate) mark: scene::Color,
    pub(crate) outline: scene::Color,
    pub(crate) hover_tint: scene::Color,
    pub(crate) pressed_tint: scene::Color,
    pub(crate) indicator: scene::Color,
    pub(crate) mark_size: i32,
    pub(crate) mark_inset: i32,
    pub(crate) label_gap: i32,
    pub(crate) icon_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slider {
    pub(crate) background: scene::Color,
    pub(crate) track: scene::Color,
    pub(crate) value: scene::Color,
    pub(crate) thumb: scene::Color,
    pub(crate) thumb_outline: scene::Color,
    pub(crate) label_width: i32,
    pub(crate) inset: i32,
    pub(crate) gap: i32,
    pub(crate) track_height: i32,
    pub(crate) thumb_width: i32,
    pub(crate) thumb_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextInput {
    pub(crate) area_background: scene::Color,
    pub(crate) field_background: scene::Color,
    pub(crate) foreground: scene::Color,
    pub(crate) placeholder: scene::Color,
    pub(crate) caret: scene::Color,
    pub(crate) padding_x: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingPanel {
    pub(crate) material: scene::Material,
    pub(crate) rounding: scene::Rounding,
    pub(crate) border: scene::Color,
    pub(crate) shadow: scene::Color,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_spread: f32,
    pub(crate) shadow_offset_y: f32,
    pub(crate) padding: i32,
    pub(crate) content_gap: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overlay {
    pub(crate) enter_fade_ms: u64,
    pub(crate) exit_fade_ms: u64,
}

impl FloatingPanel {
    pub fn material(&self) -> &scene::Material {
        &self.material
    }

    pub fn set_material(&mut self, material: scene::Material) {
        self.material = material;
    }

    pub fn border(&self) -> scene::Color {
        self.border
    }

    pub fn set_border(&mut self, border: scene::Color) {
        self.border = border;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub(crate) min_viewport_extent: i32,
    pub(crate) reveal_margin: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrollbar {
    pub(crate) metrics: ScrollbarMetrics,
    pub(crate) appearance: ScrollbarAppearance,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarMetrics {
    pub(crate) thickness: i32,
    pub(crate) policy: ScrollbarPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPolicy {
    OverlayAuto,
    GutterAlways,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarAppearance {
    pub(crate) overlay_thickness: i32,
    pub(crate) hover_thickness: i32,
    pub(crate) min_thumb_length: i32,
    pub(crate) margin: i32,
    pub(crate) fade_delay_ms: u64,
    pub(crate) fade_duration_ms: u64,
    pub(crate) track: scene::Color,
    pub(crate) thumb: scene::Color,
    pub(crate) thumb_hover_tint: scene::Color,
    pub(crate) thumb_pressed_tint: scene::Color,
    pub(crate) corner: scene::Color,
    pub(crate) rounding: scene::Rounding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandPalette {
    pub(crate) section_alignment: scene::TextAlign,
    pub(crate) max_results_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcuts {
    pub(crate) display: keymap::DisplayStyle,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            variant: Variant::Dark,
            palette: Palette {
                accent: scene::Color::rgb(10, 132, 255),
            },
            surfaces: Surfaces {
                canvas: DEFAULT_CANVAS_COLOR,
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
                padding: 12,
            },
            choice: Choice {
                background: scene::Color::rgba(0, 0, 0, 0),
                mark: scene::Color::rgb(245, 245, 247),
                outline: scene::Color::rgba(0, 0, 0, 0),
                hover_tint: scene::Color::rgba(0, 0, 0, 24),
                pressed_tint: scene::Color::rgba(0, 0, 0, 46),
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
                border: scene::Color::rgb(58, 58, 60),
                shadow: scene::Color::rgba(0, 0, 0, 96),
                shadow_blur: 24.0,
                shadow_spread: 0.5,
                shadow_offset_y: 10.0,
                padding: 6,
                content_gap: 6,
            },
            overlay: Overlay {
                enter_fade_ms: 100,
                exit_fade_ms: 100,
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
                hover_tint: scene::Color::rgba(20, 22, 25, 18),
                pressed_tint: scene::Color::rgba(20, 22, 25, 42),
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
                border: scene::Color::rgb(185, 193, 204),
                shadow: scene::Color::rgba(20, 22, 25, 48),
                shadow_blur: 24.0,
                shadow_spread: 0.5,
                shadow_offset_y: 10.0,
                padding: 6,
                content_gap: 6,
            },
            overlay: Overlay {
                enter_fade_ms: 90,
                exit_fade_ms: 120,
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

    pub(crate) fn surfaces(&self) -> &Surfaces {
        &self.surfaces
    }

    pub(crate) fn text(&self) -> &Text {
        &self.text
    }

    pub(crate) fn typography(&self) -> Typography {
        self.typography
    }

    pub(crate) fn focus(&self) -> &Focus {
        &self.focus
    }

    pub(crate) fn control(&self) -> &Control {
        &self.control
    }

    pub(crate) fn menu(&self) -> &Menu {
        &self.menu
    }

    pub(crate) fn choice(&self) -> &Choice {
        &self.choice
    }

    pub(crate) fn slider(&self) -> &Slider {
        &self.slider
    }

    pub(crate) fn text_input(&self) -> &TextInput {
        &self.text_input
    }

    pub(crate) fn table(&self) -> Table {
        Table {
            header_background: self.menu.bar_background,
            header_hover_tint: self.control.hover_tint,
            header_pressed_tint: self.control.pressed_tint,
            cell_background: scene::Color::rgba(0, 0, 0, 0),
            alternate_row_tint: match self.variant {
                Variant::Dark => scene::Color::rgba(255, 255, 255, 4),
                Variant::Light => scene::Color::rgba(20, 22, 25, 4),
            },
            passive_indicator: self.text.muted,
            cell_padding: self.control.padding,
        }
    }

    pub fn floating_panel(&self) -> &FloatingPanel {
        &self.floating_panel
    }

    pub fn floating_panel_mut(&mut self) -> &mut FloatingPanel {
        &mut self.floating_panel
    }

    pub(crate) fn overlay(&self) -> Overlay {
        self.overlay
    }

    pub(crate) fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    #[cfg(test)]
    pub(crate) fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    pub(crate) fn scrollbar(&self) -> &Scrollbar {
        &self.scrollbar
    }

    pub(crate) fn command_palette(&self) -> CommandPalette {
        self.command_palette
    }

    pub(crate) fn shortcuts(&self) -> Shortcuts {
        self.shortcuts
    }

    #[cfg(test)]
    pub(crate) fn scrollbar_mut(&mut self) -> &mut Scrollbar {
        &mut self.scrollbar
    }
}

impl Typography {
    pub(crate) fn interface(self) -> TypeStyle {
        self.interface
    }

    pub(crate) fn body(self) -> TypeStyle {
        self.body
    }

    pub(crate) fn caption(self) -> TypeStyle {
        self.caption
    }

    #[cfg(test)]
    pub(crate) fn hint(self) -> TypeStyle {
        self.hint
    }
}

impl TypeStyle {
    pub const fn new(size: f32, weight: text_model::document::Weight) -> Self {
        Self { size, weight }
    }

    pub(crate) fn size(self) -> f32 {
        self.size
    }

    pub(crate) fn weight(self) -> text_model::document::Weight {
        self.weight
    }

    pub(crate) fn document_style(self, color: text_model::Color) -> text_model::document::Style {
        text_model::document::Style::default()
            .with_size(self.size.max(1.0))
            .with_weight(self.weight)
            .with_color(color)
    }
}

impl CommandPalette {
    pub(crate) fn section_alignment(self) -> scene::TextAlign {
        self.section_alignment
    }

    pub(crate) fn max_results_height(self) -> i32 {
        self.max_results_height
    }
}

impl Shortcuts {
    pub(crate) fn display(self) -> keymap::DisplayStyle {
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
