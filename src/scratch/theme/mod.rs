use super::scene;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    variant: Variant,
    palette: Palette,
    metrics: Metrics,
    roles: Roles,
    controls: Controls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub(in crate::scratch) canvas: scene::Color,
    pub(in crate::scratch) text: scene::Color,
    pub(in crate::scratch) text_inverse: scene::Color,
    pub(in crate::scratch) text_muted: scene::Color,
    pub(in crate::scratch) accent: scene::Color,
    pub(in crate::scratch) selection: scene::Color,
    pub(in crate::scratch) focus: scene::Color,
    pub(in crate::scratch) outline: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub(in crate::scratch) control_rounding: scene::Rounding,
    pub(in crate::scratch) popup_rounding: scene::Rounding,
    pub(in crate::scratch) focus_width: i32,
    pub(in crate::scratch) choice_mark_size: i32,
    pub(in crate::scratch) choice_mark_inset: i32,
    pub(in crate::scratch) choice_label_gap: i32,
    pub(in crate::scratch) choice_icon_size: f32,
    pub(in crate::scratch) slider_label_width: i32,
    pub(in crate::scratch) slider_inset: i32,
    pub(in crate::scratch) slider_gap: i32,
    pub(in crate::scratch) slider_track_height: i32,
    pub(in crate::scratch) slider_thumb_width: i32,
    pub(in crate::scratch) slider_thumb_height: i32,
    pub(in crate::scratch) text_box_padding_x: i32,
    pub(in crate::scratch) popup_shadow_blur: f32,
    pub(in crate::scratch) popup_shadow_spread: f32,
    pub(in crate::scratch) popup_shadow_offset_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roles {
    pub(in crate::scratch) root: scene::Color,
    pub(in crate::scratch) menu_bar: scene::Color,
    pub(in crate::scratch) menu: scene::Color,
    pub(in crate::scratch) popup: scene::Color,
    pub(in crate::scratch) binding: scene::Color,
    pub(in crate::scratch) binding_disabled: scene::Color,
    pub(in crate::scratch) separator: scene::Color,
    pub(in crate::scratch) text_area: scene::Color,
    pub(in crate::scratch) button: scene::Color,
    pub(in crate::scratch) choice: scene::Color,
    pub(in crate::scratch) slider: scene::Color,
    pub(in crate::scratch) text_box: scene::Color,
    pub(in crate::scratch) panel: scene::Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controls {
    pub(in crate::scratch) choice_mark: scene::Color,
    pub(in crate::scratch) choice_outline: scene::Color,
    pub(in crate::scratch) choice_indicator: scene::Color,
    pub(in crate::scratch) slider_track: scene::Color,
    pub(in crate::scratch) slider_value: scene::Color,
    pub(in crate::scratch) slider_thumb: scene::Color,
    pub(in crate::scratch) slider_thumb_outline: scene::Color,
    pub(in crate::scratch) popup_shadow: scene::Color,
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            variant: Variant::Dark,
            palette: Palette {
                canvas: scene::Color::rgb(20, 22, 25),
                text: scene::Color::rgb(238, 241, 245),
                text_inverse: scene::Color::rgb(26, 29, 33),
                text_muted: scene::Color::rgb(132, 139, 148),
                accent: scene::Color::rgb(76, 132, 255),
                selection: scene::Color::rgba(76, 132, 255, 96),
                focus: scene::Color::rgb(76, 132, 255),
                outline: scene::Color::rgb(75, 80, 88),
            },
            metrics: Metrics::default(),
            roles: Roles {
                root: scene::Color::rgb(20, 22, 25),
                menu_bar: scene::Color::rgb(34, 37, 42),
                menu: scene::Color::rgb(40, 44, 50),
                popup: scene::Color::rgb(32, 35, 40),
                binding: scene::Color::rgb(38, 42, 48),
                binding_disabled: scene::Color::rgb(32, 35, 40),
                separator: scene::Color::rgb(78, 84, 94),
                text_area: scene::Color::rgb(245, 247, 250),
                button: scene::Color::rgb(44, 49, 56),
                choice: scene::Color::rgb(31, 35, 40),
                slider: scene::Color::rgb(31, 35, 40),
                text_box: scene::Color::rgb(245, 247, 250),
                panel: scene::Color::rgb(28, 31, 36),
            },
            controls: Controls {
                choice_mark: scene::Color::rgb(245, 247, 250),
                choice_outline: scene::Color::rgb(119, 128, 139),
                choice_indicator: scene::Color::rgb(76, 132, 255),
                slider_track: scene::Color::rgb(75, 80, 88),
                slider_value: scene::Color::rgb(76, 132, 255),
                slider_thumb: scene::Color::rgb(238, 241, 245),
                slider_thumb_outline: scene::Color::rgb(31, 35, 40),
                popup_shadow: scene::Color::rgba(0, 0, 0, 96),
            },
        }
    }

    pub const fn light() -> Self {
        Self {
            variant: Variant::Light,
            palette: Palette {
                canvas: scene::Color::rgb(245, 247, 250),
                text: scene::Color::rgb(28, 31, 36),
                text_inverse: scene::Color::rgb(28, 31, 36),
                text_muted: scene::Color::rgb(96, 105, 116),
                accent: scene::Color::rgb(42, 104, 230),
                selection: scene::Color::rgba(42, 104, 230, 72),
                focus: scene::Color::rgb(42, 104, 230),
                outline: scene::Color::rgb(185, 193, 204),
            },
            metrics: Metrics::default(),
            roles: Roles {
                root: scene::Color::rgb(245, 247, 250),
                menu_bar: scene::Color::rgb(232, 236, 242),
                menu: scene::Color::rgb(225, 230, 238),
                popup: scene::Color::rgb(255, 255, 255),
                binding: scene::Color::rgb(248, 250, 253),
                binding_disabled: scene::Color::rgb(242, 245, 249),
                separator: scene::Color::rgb(203, 211, 222),
                text_area: scene::Color::rgb(255, 255, 255),
                button: scene::Color::rgb(232, 236, 242),
                choice: scene::Color::rgb(232, 236, 242),
                slider: scene::Color::rgb(232, 236, 242),
                text_box: scene::Color::rgb(255, 255, 255),
                panel: scene::Color::rgb(239, 243, 248),
            },
            controls: Controls {
                choice_mark: scene::Color::rgb(255, 255, 255),
                choice_outline: scene::Color::rgb(146, 156, 170),
                choice_indicator: scene::Color::rgb(42, 104, 230),
                slider_track: scene::Color::rgb(185, 193, 204),
                slider_value: scene::Color::rgb(42, 104, 230),
                slider_thumb: scene::Color::rgb(255, 255, 255),
                slider_thumb_outline: scene::Color::rgb(146, 156, 170),
                popup_shadow: scene::Color::rgba(20, 22, 25, 48),
            },
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub(in crate::scratch) fn palette(&self) -> &Palette {
        &self.palette
    }

    pub(in crate::scratch) fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    pub(in crate::scratch) fn roles(&self) -> &Roles {
        &self.roles
    }

    pub(in crate::scratch) fn controls(&self) -> &Controls {
        &self.controls
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Metrics {
    pub const fn default() -> Self {
        Self {
            control_rounding: scene::Rounding::fixed(4.0),
            popup_rounding: scene::Rounding::fixed(6.0),
            focus_width: 1,
            choice_mark_size: 14,
            choice_mark_inset: 8,
            choice_label_gap: 8,
            choice_icon_size: 13.0,
            slider_label_width: 96,
            slider_inset: 8,
            slider_gap: 10,
            slider_track_height: 4,
            slider_thumb_width: 8,
            slider_thumb_height: 18,
            text_box_padding_x: 8,
            popup_shadow_blur: 18.0,
            popup_shadow_spread: 2.0,
            popup_shadow_offset_y: 8.0,
        }
    }
}
