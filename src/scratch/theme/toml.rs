use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    Choice, Control, Focus, Menu, Palette, Popup, Slider, Surfaces, Text, TextInput, Theme,
    Variant, scene,
};

#[derive(Debug, Error)]
pub enum ThemeTomlError {
    #[error("failed to parse theme TOML: {0}")]
    Parse(#[from] ::toml::de::Error),
    #[error("failed to serialize theme TOML: {0}")]
    Serialize(#[from] ::toml::ser::Error),
    #[error("invalid color `{value}` for `{field}`")]
    InvalidColor { field: String, value: String },
    #[error("unknown color reference `{name}` for `{field}`")]
    UnknownColor { field: String, name: String },
    #[error("invalid rounding `{value}` for `{field}`")]
    InvalidRounding { field: String, value: String },
    #[error("non-uniform rounding cannot be serialized for `{field}`")]
    NonUniformRounding { field: String },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ThemePatch {
    variant: Option<VariantToml>,
    palette: Option<HashMap<String, String>>,
    surfaces: Option<SurfacesPatch>,
    text: Option<TextPatch>,
    focus: Option<FocusPatch>,
    control: Option<ControlPatch>,
    menu: Option<MenuPatch>,
    choice: Option<ChoicePatch>,
    slider: Option<SliderPatch>,
    #[serde(rename = "text-input")]
    text_input: Option<TextInputPatch>,
    popup: Option<PopupPatch>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VariantToml {
    Dark,
    Light,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SurfacesPatch {
    canvas: Option<String>,
    root: Option<String>,
    panel: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TextPatch {
    primary: Option<String>,
    inverse: Option<String>,
    muted: Option<String>,
    selection: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct FocusPatch {
    color: Option<String>,
    outline: Option<String>,
    width: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ControlPatch {
    background: Option<String>,
    button_background: Option<String>,
    disabled_background: Option<String>,
    hover_tint: Option<String>,
    pressed_tint: Option<String>,
    rounding: Option<RoundingToml>,
    padding: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct MenuPatch {
    bar_background: Option<String>,
    title_background: Option<String>,
    title_hover_tint: Option<String>,
    title_pressed_tint: Option<String>,
    title_active_tint: Option<String>,
    row_background: Option<String>,
    row_hover_tint: Option<String>,
    row_pressed_tint: Option<String>,
    separator: Option<String>,
    bar_height: Option<i32>,
    row_height: Option<i32>,
    separator_line_height: Option<i32>,
    popup_min_width: Option<i32>,
    padding: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ChoicePatch {
    background: Option<String>,
    mark: Option<String>,
    outline: Option<String>,
    indicator: Option<String>,
    mark_size: Option<i32>,
    mark_inset: Option<i32>,
    label_gap: Option<i32>,
    icon_size: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct SliderPatch {
    background: Option<String>,
    track: Option<String>,
    value: Option<String>,
    thumb: Option<String>,
    thumb_outline: Option<String>,
    label_width: Option<i32>,
    inset: Option<i32>,
    gap: Option<i32>,
    track_height: Option<i32>,
    thumb_width: Option<i32>,
    thumb_height: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct TextInputPatch {
    area_background: Option<String>,
    field_background: Option<String>,
    padding_x: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct PopupPatch {
    background: Option<String>,
    rounding: Option<RoundingToml>,
    shadow: Option<String>,
    shadow_blur: Option<f32>,
    shadow_spread: Option<f32>,
    shadow_offset_y: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
enum RoundingToml {
    Name(String),
    Fixed { fixed: f32 },
    Relative { relative: f32 },
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ThemeExport {
    variant: &'static str,
    palette: PaletteExport,
    surfaces: SurfacesExport,
    text: TextExport,
    focus: FocusExport,
    control: ControlExport,
    menu: MenuExport,
    choice: ChoiceExport,
    slider: SliderExport,
    #[serde(rename = "text-input")]
    text_input: TextInputExport,
    popup: PopupExport,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PaletteExport {
    accent: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SurfacesExport {
    canvas: String,
    root: String,
    panel: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct TextExport {
    primary: String,
    inverse: String,
    muted: String,
    selection: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct FocusExport {
    color: String,
    outline: String,
    width: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ControlExport {
    background: String,
    button_background: String,
    disabled_background: String,
    hover_tint: String,
    pressed_tint: String,
    rounding: RoundingToml,
    padding: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct MenuExport {
    bar_background: String,
    title_background: String,
    title_hover_tint: String,
    title_pressed_tint: String,
    title_active_tint: String,
    row_background: String,
    row_hover_tint: String,
    row_pressed_tint: String,
    separator: String,
    bar_height: i32,
    row_height: i32,
    separator_line_height: i32,
    popup_min_width: i32,
    padding: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ChoiceExport {
    background: String,
    mark: String,
    outline: String,
    indicator: String,
    mark_size: i32,
    mark_inset: i32,
    label_gap: i32,
    icon_size: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct SliderExport {
    background: String,
    track: String,
    value: String,
    thumb: String,
    thumb_outline: String,
    label_width: i32,
    inset: i32,
    gap: i32,
    track_height: i32,
    thumb_width: i32,
    thumb_height: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct TextInputExport {
    area_background: String,
    field_background: String,
    padding_x: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct PopupExport {
    background: String,
    rounding: RoundingToml,
    shadow: String,
    shadow_blur: f32,
    shadow_spread: f32,
    shadow_offset_y: f32,
}

pub(super) fn theme_from_str(input: &str) -> Result<Theme, ThemeTomlError> {
    let patch: ThemePatch = ::toml::from_str(input)?;
    let mut theme = match patch.variant {
        Some(VariantToml::Light) => Theme::light(),
        Some(VariantToml::Dark) | None => Theme::dark(),
    };
    let mut palette = palette_map(theme.palette);

    if let Some(palette_patch) = patch.palette {
        for (name, value) in palette_patch {
            let color = parse_palette_color("palette", &value)?;
            if name == "accent" {
                theme.palette.accent = color;
            }
            palette.insert(name, color);
        }
    }

    if let Some(surfaces) = patch.surfaces {
        apply_surfaces(&mut theme.surfaces, surfaces, &palette)?;
    }
    if let Some(text) = patch.text {
        apply_text(&mut theme.text, text, &palette)?;
    }
    if let Some(focus) = patch.focus {
        apply_focus(&mut theme.focus, focus, &palette)?;
    }
    if let Some(control) = patch.control {
        apply_control(&mut theme.control, control, &palette)?;
    }
    if let Some(menu) = patch.menu {
        apply_menu(&mut theme.menu, menu, &palette)?;
    }
    if let Some(choice) = patch.choice {
        apply_choice(&mut theme.choice, choice, &palette)?;
    }
    if let Some(slider) = patch.slider {
        apply_slider(&mut theme.slider, slider, &palette)?;
    }
    if let Some(text_input) = patch.text_input {
        apply_text_input(&mut theme.text_input, text_input, &palette)?;
    }
    if let Some(popup) = patch.popup {
        apply_popup(&mut theme.popup, popup, &palette)?;
    }

    Ok(theme)
}

pub(super) fn theme_to_string(theme: &Theme) -> Result<String, ThemeTomlError> {
    let export = ThemeExport::from_theme(theme)?;
    Ok(::toml::to_string_pretty(&export)?)
}

fn apply_surfaces(
    surfaces: &mut Surfaces,
    patch: SurfacesPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut surfaces.canvas,
        patch.canvas,
        palette,
        "surfaces.canvas",
    )?;
    apply_color(&mut surfaces.root, patch.root, palette, "surfaces.root")?;
    apply_color(&mut surfaces.panel, patch.panel, palette, "surfaces.panel")?;
    Ok(())
}

fn apply_text(
    text: &mut Text,
    patch: TextPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(&mut text.primary, patch.primary, palette, "text.primary")?;
    apply_color(&mut text.inverse, patch.inverse, palette, "text.inverse")?;
    apply_color(&mut text.muted, patch.muted, palette, "text.muted")?;
    apply_color(
        &mut text.selection,
        patch.selection,
        palette,
        "text.selection",
    )?;
    Ok(())
}

fn apply_focus(
    focus: &mut Focus,
    patch: FocusPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(&mut focus.color, patch.color, palette, "focus.color")?;
    apply_color(&mut focus.outline, patch.outline, palette, "focus.outline")?;
    apply_i32(&mut focus.width, patch.width);
    Ok(())
}

fn apply_control(
    control: &mut Control,
    patch: ControlPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut control.background,
        patch.background,
        palette,
        "control.background",
    )?;
    apply_color(
        &mut control.button_background,
        patch.button_background,
        palette,
        "control.button-background",
    )?;
    apply_color(
        &mut control.disabled_background,
        patch.disabled_background,
        palette,
        "control.disabled-background",
    )?;
    apply_color(
        &mut control.hover_tint,
        patch.hover_tint,
        palette,
        "control.hover-tint",
    )?;
    apply_color(
        &mut control.pressed_tint,
        patch.pressed_tint,
        palette,
        "control.pressed-tint",
    )?;
    apply_rounding(&mut control.rounding, patch.rounding, "control.rounding")?;
    apply_i32(&mut control.padding, patch.padding);
    Ok(())
}

fn apply_menu(
    menu: &mut Menu,
    patch: MenuPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut menu.bar_background,
        patch.bar_background,
        palette,
        "menu.bar-background",
    )?;
    apply_color(
        &mut menu.title_background,
        patch.title_background,
        palette,
        "menu.title-background",
    )?;
    apply_color(
        &mut menu.title_hover_tint,
        patch.title_hover_tint,
        palette,
        "menu.title-hover-tint",
    )?;
    apply_color(
        &mut menu.title_pressed_tint,
        patch.title_pressed_tint,
        palette,
        "menu.title-pressed-tint",
    )?;
    apply_color(
        &mut menu.title_active_tint,
        patch.title_active_tint,
        palette,
        "menu.title-active-tint",
    )?;
    apply_color(
        &mut menu.row_background,
        patch.row_background,
        palette,
        "menu.row-background",
    )?;
    apply_color(
        &mut menu.row_hover_tint,
        patch.row_hover_tint,
        palette,
        "menu.row-hover-tint",
    )?;
    apply_color(
        &mut menu.row_pressed_tint,
        patch.row_pressed_tint,
        palette,
        "menu.row-pressed-tint",
    )?;
    apply_color(
        &mut menu.separator,
        patch.separator,
        palette,
        "menu.separator",
    )?;
    apply_i32(&mut menu.bar_height, patch.bar_height);
    apply_i32(&mut menu.row_height, patch.row_height);
    apply_i32(&mut menu.separator_line_height, patch.separator_line_height);
    apply_i32(&mut menu.popup_min_width, patch.popup_min_width);
    apply_i32(&mut menu.padding, patch.padding);
    Ok(())
}

fn apply_choice(
    choice: &mut Choice,
    patch: ChoicePatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut choice.background,
        patch.background,
        palette,
        "choice.background",
    )?;
    apply_color(&mut choice.mark, patch.mark, palette, "choice.mark")?;
    apply_color(
        &mut choice.outline,
        patch.outline,
        palette,
        "choice.outline",
    )?;
    apply_color(
        &mut choice.indicator,
        patch.indicator,
        palette,
        "choice.indicator",
    )?;
    apply_i32(&mut choice.mark_size, patch.mark_size);
    apply_i32(&mut choice.mark_inset, patch.mark_inset);
    apply_i32(&mut choice.label_gap, patch.label_gap);
    apply_f32(&mut choice.icon_size, patch.icon_size);
    Ok(())
}

fn apply_slider(
    slider: &mut Slider,
    patch: SliderPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut slider.background,
        patch.background,
        palette,
        "slider.background",
    )?;
    apply_color(&mut slider.track, patch.track, palette, "slider.track")?;
    apply_color(&mut slider.value, patch.value, palette, "slider.value")?;
    apply_color(&mut slider.thumb, patch.thumb, palette, "slider.thumb")?;
    apply_color(
        &mut slider.thumb_outline,
        patch.thumb_outline,
        palette,
        "slider.thumb-outline",
    )?;
    apply_i32(&mut slider.label_width, patch.label_width);
    apply_i32(&mut slider.inset, patch.inset);
    apply_i32(&mut slider.gap, patch.gap);
    apply_i32(&mut slider.track_height, patch.track_height);
    apply_i32(&mut slider.thumb_width, patch.thumb_width);
    apply_i32(&mut slider.thumb_height, patch.thumb_height);
    Ok(())
}

fn apply_text_input(
    text_input: &mut TextInput,
    patch: TextInputPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut text_input.area_background,
        patch.area_background,
        palette,
        "text-input.area-background",
    )?;
    apply_color(
        &mut text_input.field_background,
        patch.field_background,
        palette,
        "text-input.field-background",
    )?;
    apply_i32(&mut text_input.padding_x, patch.padding_x);
    Ok(())
}

fn apply_popup(
    popup: &mut Popup,
    patch: PopupPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(
        &mut popup.background,
        patch.background,
        palette,
        "popup.background",
    )?;
    apply_rounding(&mut popup.rounding, patch.rounding, "popup.rounding")?;
    apply_color(&mut popup.shadow, patch.shadow, palette, "popup.shadow")?;
    apply_f32(&mut popup.shadow_blur, patch.shadow_blur);
    apply_f32(&mut popup.shadow_spread, patch.shadow_spread);
    apply_f32(&mut popup.shadow_offset_y, patch.shadow_offset_y);
    Ok(())
}

fn apply_color(
    target: &mut scene::Color,
    value: Option<String>,
    palette: &HashMap<String, scene::Color>,
    field: &'static str,
) -> Result<(), ThemeTomlError> {
    if let Some(value) = value {
        *target = parse_color(field, &value, palette)?;
    }
    Ok(())
}

fn apply_rounding(
    target: &mut scene::Rounding,
    value: Option<RoundingToml>,
    field: &'static str,
) -> Result<(), ThemeTomlError> {
    if let Some(value) = value {
        *target = rounding_from_toml(field, value)?;
    }
    Ok(())
}

fn apply_i32(target: &mut i32, value: Option<i32>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn apply_f32(target: &mut f32, value: Option<f32>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn palette_map(palette: Palette) -> HashMap<String, scene::Color> {
    HashMap::from([("accent".to_owned(), palette.accent)])
}

fn parse_palette_color(field: &'static str, value: &str) -> Result<scene::Color, ThemeTomlError> {
    parse_color(field, value, &HashMap::new())
}

fn parse_color(
    field: &'static str,
    value: &str,
    palette: &HashMap<String, scene::Color>,
) -> Result<scene::Color, ThemeTomlError> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("transparent") {
        return Ok(scene::Color::rgba(0, 0, 0, 0));
    }

    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_color(field, value, hex);
    }

    palette
        .get(value)
        .copied()
        .ok_or_else(|| ThemeTomlError::UnknownColor {
            field: field.to_owned(),
            name: value.to_owned(),
        })
}

fn parse_hex_color(
    field: &'static str,
    value: &str,
    hex: &str,
) -> Result<scene::Color, ThemeTomlError> {
    let channels = match hex.len() {
        6 => [
            parse_hex_channel(field, value, &hex[0..2])?,
            parse_hex_channel(field, value, &hex[2..4])?,
            parse_hex_channel(field, value, &hex[4..6])?,
            255,
        ],
        8 => [
            parse_hex_channel(field, value, &hex[0..2])?,
            parse_hex_channel(field, value, &hex[2..4])?,
            parse_hex_channel(field, value, &hex[4..6])?,
            parse_hex_channel(field, value, &hex[6..8])?,
        ],
        _ => {
            return Err(ThemeTomlError::InvalidColor {
                field: field.to_owned(),
                value: value.to_owned(),
            });
        }
    };

    Ok(scene::Color::rgba(
        channels[0],
        channels[1],
        channels[2],
        channels[3],
    ))
}

fn parse_hex_channel(
    field: &'static str,
    value: &str,
    channel: &str,
) -> Result<u8, ThemeTomlError> {
    u8::from_str_radix(channel, 16).map_err(|_| ThemeTomlError::InvalidColor {
        field: field.to_owned(),
        value: value.to_owned(),
    })
}

fn rounding_from_toml(
    field: &'static str,
    value: RoundingToml,
) -> Result<scene::Rounding, ThemeTomlError> {
    match value {
        RoundingToml::Name(name) if name == "none" => Ok(scene::Rounding::none()),
        RoundingToml::Name(name) => Err(ThemeTomlError::InvalidRounding {
            field: field.to_owned(),
            value: name,
        }),
        RoundingToml::Fixed { fixed } => Ok(scene::Rounding::fixed(fixed)),
        RoundingToml::Relative { relative } => Ok(scene::Rounding::relative(relative)),
    }
}

impl ThemeExport {
    fn from_theme(theme: &Theme) -> Result<Self, ThemeTomlError> {
        Ok(Self {
            variant: match theme.variant {
                Variant::Dark => "dark",
                Variant::Light => "light",
            },
            palette: PaletteExport {
                accent: color_literal(theme.palette.accent),
            },
            surfaces: SurfacesExport {
                canvas: color_string(theme.surfaces.canvas, theme.palette),
                root: color_string(theme.surfaces.root, theme.palette),
                panel: color_string(theme.surfaces.panel, theme.palette),
            },
            text: TextExport {
                primary: color_string(theme.text.primary, theme.palette),
                inverse: color_string(theme.text.inverse, theme.palette),
                muted: color_string(theme.text.muted, theme.palette),
                selection: color_string(theme.text.selection, theme.palette),
            },
            focus: FocusExport {
                color: color_string(theme.focus.color, theme.palette),
                outline: color_string(theme.focus.outline, theme.palette),
                width: theme.focus.width,
            },
            control: ControlExport {
                background: color_string(theme.control.background, theme.palette),
                button_background: color_string(theme.control.button_background, theme.palette),
                disabled_background: color_string(theme.control.disabled_background, theme.palette),
                hover_tint: color_string(theme.control.hover_tint, theme.palette),
                pressed_tint: color_string(theme.control.pressed_tint, theme.palette),
                rounding: rounding_to_toml("control.rounding", theme.control.rounding)?,
                padding: theme.control.padding,
            },
            menu: MenuExport {
                bar_background: color_string(theme.menu.bar_background, theme.palette),
                title_background: color_string(theme.menu.title_background, theme.palette),
                title_hover_tint: color_string(theme.menu.title_hover_tint, theme.palette),
                title_pressed_tint: color_string(theme.menu.title_pressed_tint, theme.palette),
                title_active_tint: color_string(theme.menu.title_active_tint, theme.palette),
                row_background: color_string(theme.menu.row_background, theme.palette),
                row_hover_tint: color_string(theme.menu.row_hover_tint, theme.palette),
                row_pressed_tint: color_string(theme.menu.row_pressed_tint, theme.palette),
                separator: color_string(theme.menu.separator, theme.palette),
                bar_height: theme.menu.bar_height,
                row_height: theme.menu.row_height,
                separator_line_height: theme.menu.separator_line_height,
                popup_min_width: theme.menu.popup_min_width,
                padding: theme.menu.padding,
            },
            choice: ChoiceExport {
                background: color_string(theme.choice.background, theme.palette),
                mark: color_string(theme.choice.mark, theme.palette),
                outline: color_string(theme.choice.outline, theme.palette),
                indicator: color_string(theme.choice.indicator, theme.palette),
                mark_size: theme.choice.mark_size,
                mark_inset: theme.choice.mark_inset,
                label_gap: theme.choice.label_gap,
                icon_size: theme.choice.icon_size,
            },
            slider: SliderExport {
                background: color_string(theme.slider.background, theme.palette),
                track: color_string(theme.slider.track, theme.palette),
                value: color_string(theme.slider.value, theme.palette),
                thumb: color_string(theme.slider.thumb, theme.palette),
                thumb_outline: color_string(theme.slider.thumb_outline, theme.palette),
                label_width: theme.slider.label_width,
                inset: theme.slider.inset,
                gap: theme.slider.gap,
                track_height: theme.slider.track_height,
                thumb_width: theme.slider.thumb_width,
                thumb_height: theme.slider.thumb_height,
            },
            text_input: TextInputExport {
                area_background: color_string(theme.text_input.area_background, theme.palette),
                field_background: color_string(theme.text_input.field_background, theme.palette),
                padding_x: theme.text_input.padding_x,
            },
            popup: PopupExport {
                background: color_string(theme.popup.background, theme.palette),
                rounding: rounding_to_toml("popup.rounding", theme.popup.rounding)?,
                shadow: color_string(theme.popup.shadow, theme.palette),
                shadow_blur: theme.popup.shadow_blur,
                shadow_spread: theme.popup.shadow_spread,
                shadow_offset_y: theme.popup.shadow_offset_y,
            },
        })
    }
}

fn color_string(color: scene::Color, palette: Palette) -> String {
    if color == palette.accent {
        "accent".to_owned()
    } else {
        color_literal(color)
    }
}

fn color_literal(color: scene::Color) -> String {
    let (r, g, b, a) = color.channels();
    if (r, g, b, a) == (0, 0, 0, 0) {
        return "transparent".to_owned();
    }

    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

fn rounding_to_toml(
    field: &'static str,
    rounding: scene::Rounding,
) -> Result<RoundingToml, ThemeTomlError> {
    let top_left = rounding.top_left();
    if rounding.top_right() != top_left
        || rounding.bottom_right() != top_left
        || rounding.bottom_left() != top_left
    {
        return Err(ThemeTomlError::NonUniformRounding {
            field: field.to_owned(),
        });
    }

    match top_left {
        scene::Radius::Fixed(value) if value == 0.0 => Ok(RoundingToml::Name("none".to_owned())),
        scene::Radius::Fixed(value) => Ok(RoundingToml::Fixed { fixed: value }),
        scene::Radius::Relative(value) => Ok(RoundingToml::Relative { relative: value }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_variant_and_palette_patch() {
        let theme = Theme::from_toml_str(
            r##"
            variant = "light"

            [palette]
            accent = "#ff00cc"
            "##,
        )
        .expect("theme should parse");

        assert_eq!(theme.variant(), Variant::Light);
        assert_eq!(theme.palette().accent(), scene::Color::rgb(255, 0, 204));
        assert_eq!(theme.surfaces().canvas, Theme::light().surfaces().canvas);
    }

    #[test]
    fn resolves_palette_transparent_and_alpha_colors() {
        let theme = Theme::from_toml_str(
            r##"
            [palette]
            base = "#112233"
            overlay = "#44556677"

            [surfaces]
            canvas = "base"

            [menu]
            title-background = "transparent"
            row-hover-tint = "overlay"
            "##,
        )
        .expect("theme should parse");

        assert_eq!(theme.surfaces().canvas, scene::Color::rgb(17, 34, 51));
        assert_eq!(
            theme.menu().title_background,
            scene::Color::rgba(0, 0, 0, 0)
        );
        assert_eq!(
            theme.menu().row_hover_tint,
            scene::Color::rgba(68, 85, 102, 119)
        );
    }

    #[test]
    fn rejects_unknown_fields_and_color_references() {
        let unknown_field = Theme::from_toml_str(
            r##"
            [menu]
            missing-token = "#ffffff"
            "##,
        )
        .expect_err("unknown fields should fail");
        assert!(unknown_field.to_string().contains("unknown field"));

        let unknown_color = Theme::from_toml_str(
            r##"
            [text]
            primary = "missing"
            "##,
        )
        .expect_err("unknown palette names should fail");
        assert!(matches!(
            unknown_color,
            ThemeTomlError::UnknownColor { field, name }
                if field == "text.primary" && name == "missing"
        ));
    }

    #[test]
    fn round_trips_builtin_themes() {
        for theme in [Theme::dark(), Theme::light()] {
            let serialized = theme
                .to_toml_string()
                .expect("theme should serialize to TOML");
            let parsed = Theme::from_toml_str(&serialized).expect("theme should parse");

            assert_eq!(parsed, theme);
        }
    }
}
