mod toml;

pub use self::toml::ThemeTomlError;

use super::scene;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    variant: Variant,
    palette: Palette,
    surfaces: Surfaces,
    text: Text,
    focus: Focus,
    control: Control,
    menu: Menu,
    choice: Choice,
    slider: Slider,
    text_input: TextInput,
    popup: Popup,
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
pub struct Focus {
    pub(in crate::scratch) color: scene::Color,
    pub(in crate::scratch) outline: scene::Color,
    pub(in crate::scratch) width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Control {
    pub(in crate::scratch) background: scene::Color,
    pub(in crate::scratch) button_background: scene::Color,
    pub(in crate::scratch) disabled_background: scene::Color,
    pub(in crate::scratch) hover_tint: scene::Color,
    pub(in crate::scratch) pressed_tint: scene::Color,
    pub(in crate::scratch) rounding: scene::Rounding,
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
    pub(in crate::scratch) popup_min_width: i32,
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
    pub(in crate::scratch) padding_x: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Popup {
    pub(in crate::scratch) background: scene::Color,
    pub(in crate::scratch) rounding: scene::Rounding,
    pub(in crate::scratch) shadow: scene::Color,
    pub(in crate::scratch) shadow_blur: f32,
    pub(in crate::scratch) shadow_spread: f32,
    pub(in crate::scratch) shadow_offset_y: f32,
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            variant: Variant::Dark,
            palette: Palette {
                accent: scene::Color::rgb(76, 132, 255),
            },
            surfaces: Surfaces {
                canvas: scene::Color::rgb(20, 22, 25),
                root: scene::Color::rgb(20, 22, 25),
                panel: scene::Color::rgb(28, 31, 36),
            },
            text: Text {
                primary: scene::Color::rgb(238, 241, 245),
                inverse: scene::Color::rgb(26, 29, 33),
                muted: scene::Color::rgb(132, 139, 148),
                selection: scene::Color::rgba(76, 132, 255, 96),
            },
            focus: Focus {
                color: scene::Color::rgb(76, 132, 255),
                outline: scene::Color::rgb(75, 80, 88),
                width: 1,
            },
            control: Control {
                background: scene::Color::rgb(38, 42, 48),
                button_background: scene::Color::rgb(44, 49, 56),
                disabled_background: scene::Color::rgb(32, 35, 40),
                hover_tint: scene::Color::rgba(255, 255, 255, 18),
                pressed_tint: scene::Color::rgba(0, 0, 0, 36),
                rounding: scene::Rounding::fixed(4.0),
                padding: 4,
            },
            menu: Menu {
                bar_background: scene::Color::rgb(34, 37, 42),
                title_background: scene::Color::rgba(0, 0, 0, 0),
                title_hover_tint: scene::Color::rgba(255, 255, 255, 20),
                title_pressed_tint: scene::Color::rgba(0, 0, 0, 36),
                title_active_tint: scene::Color::rgba(76, 132, 255, 46),
                row_background: scene::Color::rgba(0, 0, 0, 0),
                row_hover_tint: scene::Color::rgba(255, 255, 255, 20),
                row_pressed_tint: scene::Color::rgba(0, 0, 0, 36),
                separator: scene::Color::rgb(78, 84, 94),
                bar_height: 28,
                row_height: 28,
                separator_line_height: 1,
                popup_min_width: 220,
                padding: 4,
            },
            choice: Choice {
                background: scene::Color::rgb(31, 35, 40),
                mark: scene::Color::rgb(245, 247, 250),
                outline: scene::Color::rgb(119, 128, 139),
                indicator: scene::Color::rgb(76, 132, 255),
                mark_size: 14,
                mark_inset: 8,
                label_gap: 8,
                icon_size: 13.0,
            },
            slider: Slider {
                background: scene::Color::rgb(31, 35, 40),
                track: scene::Color::rgb(75, 80, 88),
                value: scene::Color::rgb(76, 132, 255),
                thumb: scene::Color::rgb(238, 241, 245),
                thumb_outline: scene::Color::rgb(31, 35, 40),
                label_width: 96,
                inset: 8,
                gap: 10,
                track_height: 4,
                thumb_width: 8,
                thumb_height: 18,
            },
            text_input: TextInput {
                area_background: scene::Color::rgb(245, 247, 250),
                field_background: scene::Color::rgb(245, 247, 250),
                padding_x: 8,
            },
            popup: Popup {
                background: scene::Color::rgb(32, 35, 40),
                rounding: scene::Rounding::fixed(6.0),
                shadow: scene::Color::rgba(0, 0, 0, 96),
                shadow_blur: 18.0,
                shadow_spread: 2.0,
                shadow_offset_y: 8.0,
            },
        }
    }

    pub const fn light() -> Self {
        Self {
            variant: Variant::Light,
            palette: Palette {
                accent: scene::Color::rgb(42, 104, 230),
            },
            surfaces: Surfaces {
                canvas: scene::Color::rgb(245, 247, 250),
                root: scene::Color::rgb(245, 247, 250),
                panel: scene::Color::rgb(239, 243, 248),
            },
            text: Text {
                primary: scene::Color::rgb(28, 31, 36),
                inverse: scene::Color::rgb(28, 31, 36),
                muted: scene::Color::rgb(96, 105, 116),
                selection: scene::Color::rgba(42, 104, 230, 72),
            },
            focus: Focus {
                color: scene::Color::rgb(42, 104, 230),
                outline: scene::Color::rgb(185, 193, 204),
                width: 1,
            },
            control: Control {
                background: scene::Color::rgb(248, 250, 253),
                button_background: scene::Color::rgb(232, 236, 242),
                disabled_background: scene::Color::rgb(242, 245, 249),
                hover_tint: scene::Color::rgba(20, 22, 25, 14),
                pressed_tint: scene::Color::rgba(20, 22, 25, 28),
                rounding: scene::Rounding::fixed(4.0),
                padding: 4,
            },
            menu: Menu {
                bar_background: scene::Color::rgb(232, 236, 242),
                title_background: scene::Color::rgba(0, 0, 0, 0),
                title_hover_tint: scene::Color::rgba(20, 22, 25, 18),
                title_pressed_tint: scene::Color::rgba(20, 22, 25, 30),
                title_active_tint: scene::Color::rgba(42, 104, 230, 36),
                row_background: scene::Color::rgba(0, 0, 0, 0),
                row_hover_tint: scene::Color::rgba(20, 22, 25, 18),
                row_pressed_tint: scene::Color::rgba(20, 22, 25, 30),
                separator: scene::Color::rgb(203, 211, 222),
                bar_height: 28,
                row_height: 28,
                separator_line_height: 1,
                popup_min_width: 220,
                padding: 4,
            },
            choice: Choice {
                background: scene::Color::rgb(232, 236, 242),
                mark: scene::Color::rgb(255, 255, 255),
                outline: scene::Color::rgb(146, 156, 170),
                indicator: scene::Color::rgb(42, 104, 230),
                mark_size: 14,
                mark_inset: 8,
                label_gap: 8,
                icon_size: 13.0,
            },
            slider: Slider {
                background: scene::Color::rgb(232, 236, 242),
                track: scene::Color::rgb(185, 193, 204),
                value: scene::Color::rgb(42, 104, 230),
                thumb: scene::Color::rgb(255, 255, 255),
                thumb_outline: scene::Color::rgb(146, 156, 170),
                label_width: 96,
                inset: 8,
                gap: 10,
                track_height: 4,
                thumb_width: 8,
                thumb_height: 18,
            },
            text_input: TextInput {
                area_background: scene::Color::rgb(255, 255, 255),
                field_background: scene::Color::rgb(255, 255, 255),
                padding_x: 8,
            },
            popup: Popup {
                background: scene::Color::rgb(255, 255, 255),
                rounding: scene::Rounding::fixed(6.0),
                shadow: scene::Color::rgba(20, 22, 25, 48),
                shadow_blur: 18.0,
                shadow_spread: 2.0,
                shadow_offset_y: 8.0,
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

    pub(in crate::scratch) fn popup(&self) -> &Popup {
        &self.popup
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
