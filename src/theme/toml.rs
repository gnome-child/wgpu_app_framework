use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

use crate::text as text_model;

use super::{
    AuxiliaryPanel, Choice, CommandPalette, Control, FloatingPanel, Focus, Menu, Overlay, Palette,
    Scrollbar, ScrollbarAppearance, ScrollbarMetrics, ScrollbarPolicy, Shortcuts, Slider, Surfaces,
    Text, TextInput, Theme, TypeStyle, Typography, Variant, Viewport, keymap, scene,
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
    #[error("unknown material recipe `{name}` for `{field}`")]
    UnknownMaterialRecipe { field: String, name: String },
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
    typography: Option<TypographyPatch>,
    focus: Option<FocusPatch>,
    control: Option<ControlPatch>,
    menu: Option<MenuPatch>,
    choice: Option<ChoicePatch>,
    slider: Option<SliderPatch>,
    #[serde(rename = "text-input")]
    text_input: Option<TextInputPatch>,
    #[serde(rename = "floating-panel")]
    floating_panel: Option<FloatingPanelPatch>,
    #[serde(rename = "auxiliary-panel")]
    auxiliary_panel: Option<AuxiliaryPanelPatch>,
    overlay: Option<OverlayPatch>,
    viewport: Option<ViewportPatch>,
    scrollbar: Option<ScrollbarPatch>,
    #[serde(rename = "command-palette")]
    command_palette: Option<CommandPalettePatch>,
    shortcuts: Option<ShortcutsPatch>,
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
struct TypographyPatch {
    interface_size: Option<f32>,
    interface_weight: Option<WeightToml>,
    body_size: Option<f32>,
    body_weight: Option<WeightToml>,
    caption_size: Option<f32>,
    caption_weight: Option<WeightToml>,
    hint_size: Option<f32>,
    hint_weight: Option<WeightToml>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum WeightToml {
    Normal,
    Medium,
    Bold,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct FocusPatch {
    color: Option<String>,
    outline: Option<String>,
    width: Option<i32>,
    offset: Option<f32>,
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
    height: Option<i32>,
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
    panel_min_width: Option<i32>,
    padding: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ChoicePatch {
    background: Option<String>,
    mark: Option<String>,
    outline: Option<String>,
    hover_tint: Option<String>,
    pressed_tint: Option<String>,
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
    foreground: Option<String>,
    placeholder: Option<String>,
    caret: Option<String>,
    padding_x: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct FloatingPanelPatch {
    material: Option<MaterialToml>,
    rounding: Option<RoundingToml>,
    border: Option<String>,
    shadow: Option<String>,
    shadow_blur: Option<f32>,
    shadow_spread: Option<f32>,
    shadow_offset_y: Option<f32>,
    padding: Option<i32>,
    content_gap: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct AuxiliaryPanelPatch {
    hover_delay_ms: Option<u64>,
    max_width: Option<i32>,
    max_height: Option<i32>,
    icon_extent: Option<i32>,
    icon_gap: Option<i32>,
    info: Option<String>,
    warning: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct OverlayPatch {
    enter_fade_ms: Option<u64>,
    exit_fade_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ViewportPatch {
    min_viewport_extent: Option<i32>,
    reveal_margin: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ScrollbarPatch {
    policy: Option<ScrollbarPolicyToml>,
    thickness: Option<i32>,
    overlay_thickness: Option<i32>,
    hover_thickness: Option<i32>,
    min_thumb_length: Option<i32>,
    margin: Option<i32>,
    fade_delay_ms: Option<u64>,
    fade_duration_ms: Option<u64>,
    track: Option<String>,
    thumb: Option<String>,
    thumb_hover_tint: Option<String>,
    thumb_pressed_tint: Option<String>,
    corner: Option<String>,
    rounding: Option<RoundingToml>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ScrollbarPolicyToml {
    OverlayAuto,
    GutterAlways,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TextAlignToml {
    Start,
    Center,
    End,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct CommandPalettePatch {
    section_alignment: Option<TextAlignToml>,
    max_results_height: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct ShortcutsPatch {
    display: Option<ShortcutDisplayToml>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ShortcutDisplayToml {
    Default,
    Symbols,
    Text,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
enum RoundingToml {
    Name(String),
    Fixed { fixed: f32 },
    Relative { relative: f32 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, rename_all = "kebab-case")]
enum BrushToml {
    Color(String),
    LinearGradient { from: String, to: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(
    deny_unknown_fields,
    rename_all = "kebab-case",
    rename_all_fields = "kebab-case",
    tag = "kind"
)]
enum MaterialToml {
    Solid {
        color: BrushToml,
    },
    Glass {
        recipe: Option<String>,
        blur_sigma: Option<f32>,
        tint: Option<BrushToml>,
        tint_opacity: Option<f32>,
        luminosity_opacity: Option<f32>,
        noise_opacity: Option<f32>,
        fallback: Option<BrushToml>,
        refraction_displacement: Option<f32>,
        refraction_splay: Option<f32>,
        refraction_feather: Option<f32>,
        refraction_curve: Option<f32>,
    },
}

struct ThemeExport {
    variant: &'static str,
    palette: PaletteExport,
    surfaces: SurfacesExport,
    text: TextExport,
    typography: TypographyExport,
    focus: FocusExport,
    control: ControlExport,
    menu: MenuExport,
    choice: ChoiceExport,
    slider: SliderExport,
    text_input: TextInputExport,
    floating_panel: FloatingPanelExport,
    auxiliary_panel: AuxiliaryPanelExport,
    overlay: OverlayExport,
    viewport: ViewportExport,
    scrollbar: ScrollbarExport,
    command_palette: CommandPaletteExport,
    shortcuts: ShortcutsExport,
}

struct PaletteExport {
    accent: String,
}

struct SurfacesExport {
    canvas: String,
    root: String,
    panel: String,
}

struct TextExport {
    primary: String,
    inverse: String,
    muted: String,
    selection: String,
}

struct TypographyExport {
    interface_size: f32,
    interface_weight: &'static str,
    body_size: f32,
    body_weight: &'static str,
    caption_size: f32,
    caption_weight: &'static str,
    hint_size: f32,
    hint_weight: &'static str,
}

struct FocusExport {
    color: String,
    outline: String,
    width: i32,
    offset: f32,
}

struct ControlExport {
    background: String,
    button_background: String,
    disabled_background: String,
    hover_tint: String,
    pressed_tint: String,
    rounding: RoundingToml,
    height: i32,
    padding: i32,
}

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
    panel_min_width: i32,
    padding: i32,
}

struct ChoiceExport {
    background: String,
    mark: String,
    outline: String,
    hover_tint: String,
    pressed_tint: String,
    indicator: String,
    mark_size: i32,
    mark_inset: i32,
    label_gap: i32,
    icon_size: f32,
}

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

struct TextInputExport {
    area_background: String,
    field_background: String,
    foreground: String,
    placeholder: String,
    caret: String,
    padding_x: i32,
}

struct FloatingPanelExport {
    material: MaterialToml,
    rounding: RoundingToml,
    border: String,
    shadow: String,
    shadow_blur: f32,
    shadow_spread: f32,
    shadow_offset_y: f32,
    padding: i32,
    content_gap: i32,
}

struct AuxiliaryPanelExport {
    hover_delay_ms: u64,
    max_width: i32,
    max_height: i32,
    icon_extent: i32,
    icon_gap: i32,
    info: String,
    warning: String,
    error: String,
}

struct OverlayExport {
    enter_fade_ms: u64,
    exit_fade_ms: u64,
}

struct ViewportExport {
    min_viewport_extent: i32,
    reveal_margin: i32,
}

struct ScrollbarExport {
    policy: &'static str,
    thickness: i32,
    overlay_thickness: i32,
    hover_thickness: i32,
    min_thumb_length: i32,
    margin: i32,
    fade_delay_ms: u64,
    fade_duration_ms: u64,
    track: String,
    thumb: String,
    thumb_hover_tint: String,
    thumb_pressed_tint: String,
    corner: String,
    rounding: RoundingToml,
}

struct CommandPaletteExport {
    section_alignment: &'static str,
    max_results_height: i32,
}

struct ShortcutsExport {
    display: &'static str,
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
    if let Some(typography) = patch.typography {
        apply_typography(&mut theme.typography, typography);
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
    if let Some(floating_panel) = patch.floating_panel {
        apply_floating_panel(&mut theme.floating_panel, floating_panel, &palette)?;
    }
    if let Some(auxiliary_panel) = patch.auxiliary_panel {
        apply_auxiliary_panel(&mut theme.auxiliary_panel, auxiliary_panel, &palette)?;
    }
    if let Some(overlay) = patch.overlay {
        apply_overlay(&mut theme.overlay, overlay);
    }
    if let Some(viewport) = patch.viewport {
        apply_viewport(&mut theme.viewport, viewport);
    }
    if let Some(scrollbar) = patch.scrollbar {
        apply_scrollbar(&mut theme.scrollbar, scrollbar, &palette)?;
    }
    if let Some(command_palette) = patch.command_palette {
        apply_command_palette(&mut theme.command_palette, command_palette);
    }
    if let Some(shortcuts) = patch.shortcuts {
        apply_shortcuts(&mut theme.shortcuts, shortcuts);
    }
    Ok(theme)
}

pub(super) fn theme_to_string(theme: &Theme) -> Result<String, ThemeTomlError> {
    let export = ThemeExport::from_theme(theme)?;
    Ok(export.to_toml_string())
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

fn apply_typography(typography: &mut Typography, patch: TypographyPatch) {
    let interface_size = patch.interface_size;
    let caption_size = patch
        .caption_size
        .or_else(|| interface_size.map(|size| (size - 1.0).max(1.0)));
    let hint_size = patch.hint_size.or(interface_size);

    apply_type_style(
        &mut typography.interface,
        interface_size,
        patch.interface_weight,
    );
    apply_type_style(&mut typography.body, patch.body_size, patch.body_weight);
    apply_type_style(&mut typography.caption, caption_size, patch.caption_weight);
    apply_type_style(&mut typography.hint, hint_size, patch.hint_weight);
}

fn apply_shortcuts(shortcuts: &mut Shortcuts, patch: ShortcutsPatch) {
    if let Some(display) = patch.display {
        shortcuts.display = shortcut_display_from_toml(display);
    }
}

fn apply_type_style(style: &mut TypeStyle, size: Option<f32>, weight: Option<WeightToml>) {
    apply_f32(&mut style.size, size);
    if let Some(weight) = weight {
        style.weight = weight_from_toml(weight);
    }
}

fn apply_focus(
    focus: &mut Focus,
    patch: FocusPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_color(&mut focus.color, patch.color, palette, "focus.color")?;
    apply_color(&mut focus.outline, patch.outline, palette, "focus.outline")?;
    apply_i32(&mut focus.width, patch.width);
    apply_f32(&mut focus.offset, patch.offset);
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
    apply_i32(&mut control.height, patch.height);
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
    apply_i32(&mut menu.panel_min_width, patch.panel_min_width);
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
        &mut choice.hover_tint,
        patch.hover_tint,
        palette,
        "choice.hover-tint",
    )?;
    apply_color(
        &mut choice.pressed_tint,
        patch.pressed_tint,
        palette,
        "choice.pressed-tint",
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
    apply_color(
        &mut text_input.foreground,
        patch.foreground,
        palette,
        "text-input.foreground",
    )?;
    apply_color(
        &mut text_input.placeholder,
        patch.placeholder,
        palette,
        "text-input.placeholder",
    )?;
    apply_color(
        &mut text_input.caret,
        patch.caret,
        palette,
        "text-input.caret",
    )?;
    apply_i32(&mut text_input.padding_x, patch.padding_x);
    Ok(())
}

fn apply_floating_panel(
    floating_panel: &mut FloatingPanel,
    patch: FloatingPanelPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    if let Some(material) = patch.material {
        floating_panel.material = material_from_toml(
            material,
            floating_panel.material.clone(),
            palette,
            "floating-panel.material",
        )?;
    }
    apply_rounding(
        &mut floating_panel.rounding,
        patch.rounding,
        "floating-panel.rounding",
    )?;
    apply_color(
        &mut floating_panel.border,
        patch.border,
        palette,
        "floating-panel.border",
    )?;
    apply_color(
        &mut floating_panel.shadow,
        patch.shadow,
        palette,
        "floating-panel.shadow",
    )?;
    apply_f32(&mut floating_panel.shadow_blur, patch.shadow_blur);
    apply_f32(&mut floating_panel.shadow_spread, patch.shadow_spread);
    apply_f32(&mut floating_panel.shadow_offset_y, patch.shadow_offset_y);
    apply_i32(&mut floating_panel.padding, patch.padding);
    apply_i32(&mut floating_panel.content_gap, patch.content_gap);
    Ok(())
}

fn apply_auxiliary_panel(
    auxiliary: &mut AuxiliaryPanel,
    patch: AuxiliaryPanelPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_u64(&mut auxiliary.hover_delay_ms, patch.hover_delay_ms);
    apply_i32(&mut auxiliary.max_width, patch.max_width);
    apply_i32(&mut auxiliary.max_height, patch.max_height);
    apply_i32(&mut auxiliary.icon_extent, patch.icon_extent);
    apply_i32(&mut auxiliary.icon_gap, patch.icon_gap);
    apply_color(
        &mut auxiliary.info,
        patch.info,
        palette,
        "auxiliary-panel.info",
    )?;
    apply_color(
        &mut auxiliary.warning,
        patch.warning,
        palette,
        "auxiliary-panel.warning",
    )?;
    apply_color(
        &mut auxiliary.error,
        patch.error,
        palette,
        "auxiliary-panel.error",
    )?;
    Ok(())
}

fn apply_overlay(overlay: &mut Overlay, patch: OverlayPatch) {
    apply_u64(&mut overlay.enter_fade_ms, patch.enter_fade_ms);
    apply_u64(&mut overlay.exit_fade_ms, patch.exit_fade_ms);
}

fn apply_viewport(viewport: &mut Viewport, patch: ViewportPatch) {
    apply_i32(&mut viewport.min_viewport_extent, patch.min_viewport_extent);
    apply_i32(&mut viewport.reveal_margin, patch.reveal_margin);
}

fn apply_scrollbar(
    scrollbar: &mut Scrollbar,
    patch: ScrollbarPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_scrollbar_metrics(&mut scrollbar.metrics, &patch);
    apply_scrollbar_appearance(&mut scrollbar.appearance, patch, palette)
}

fn apply_scrollbar_metrics(metrics: &mut ScrollbarMetrics, patch: &ScrollbarPatch) {
    if let Some(policy) = patch.policy {
        metrics.policy = scrollbar_policy_from_toml(policy);
    }
    apply_i32(&mut metrics.thickness, patch.thickness);
}

fn apply_scrollbar_appearance(
    appearance: &mut ScrollbarAppearance,
    patch: ScrollbarPatch,
    palette: &HashMap<String, scene::Color>,
) -> Result<(), ThemeTomlError> {
    apply_i32(&mut appearance.overlay_thickness, patch.overlay_thickness);
    apply_i32(&mut appearance.hover_thickness, patch.hover_thickness);
    apply_i32(&mut appearance.min_thumb_length, patch.min_thumb_length);
    apply_i32(&mut appearance.margin, patch.margin);
    apply_u64(&mut appearance.fade_delay_ms, patch.fade_delay_ms);
    apply_u64(&mut appearance.fade_duration_ms, patch.fade_duration_ms);
    apply_color(
        &mut appearance.track,
        patch.track,
        palette,
        "scrollbar.track",
    )?;
    apply_color(
        &mut appearance.thumb,
        patch.thumb,
        palette,
        "scrollbar.thumb",
    )?;
    apply_color(
        &mut appearance.thumb_hover_tint,
        patch.thumb_hover_tint,
        palette,
        "scrollbar.thumb-hover-tint",
    )?;
    apply_color(
        &mut appearance.thumb_pressed_tint,
        patch.thumb_pressed_tint,
        palette,
        "scrollbar.thumb-pressed-tint",
    )?;
    apply_color(
        &mut appearance.corner,
        patch.corner,
        palette,
        "scrollbar.corner",
    )?;
    apply_rounding(
        &mut appearance.rounding,
        patch.rounding,
        "scrollbar.rounding",
    )?;
    Ok(())
}

fn apply_command_palette(command_palette: &mut CommandPalette, patch: CommandPalettePatch) {
    if let Some(alignment) = patch.section_alignment {
        command_palette.section_alignment = text_align_from_toml(alignment);
    }
    apply_i32(
        &mut command_palette.max_results_height,
        patch.max_results_height,
    );
}

fn weight_from_toml(value: WeightToml) -> text_model::document::Weight {
    match value {
        WeightToml::Normal => text_model::document::Weight::Normal,
        WeightToml::Medium => text_model::document::Weight::Medium,
        WeightToml::Bold => text_model::document::Weight::Bold,
    }
}

fn text_align_from_toml(value: TextAlignToml) -> scene::TextAlign {
    match value {
        TextAlignToml::Start => scene::TextAlign::Start,
        TextAlignToml::Center => scene::TextAlign::Center,
        TextAlignToml::End => scene::TextAlign::End,
    }
}

fn shortcut_display_from_toml(value: ShortcutDisplayToml) -> keymap::DisplayStyle {
    match value {
        ShortcutDisplayToml::Default => keymap::DisplayStyle::Default,
        ShortcutDisplayToml::Symbols => keymap::DisplayStyle::Symbols,
        ShortcutDisplayToml::Text => keymap::DisplayStyle::Text,
    }
}

fn weight_to_toml(value: text_model::document::Weight) -> &'static str {
    match value {
        text_model::document::Weight::Normal => "normal",
        text_model::document::Weight::Medium => "medium",
        text_model::document::Weight::Bold => "bold",
    }
}

fn text_align_to_toml(value: scene::TextAlign) -> &'static str {
    match value {
        scene::TextAlign::Start => "start",
        scene::TextAlign::Center => "center",
        scene::TextAlign::End => "end",
    }
}

fn shortcut_display_to_toml(value: keymap::DisplayStyle) -> &'static str {
    match value {
        keymap::DisplayStyle::Default => "default",
        keymap::DisplayStyle::Symbols => "symbols",
        keymap::DisplayStyle::Text => "text",
    }
}

fn scrollbar_policy_from_toml(value: ScrollbarPolicyToml) -> ScrollbarPolicy {
    match value {
        ScrollbarPolicyToml::OverlayAuto => ScrollbarPolicy::OverlayAuto,
        ScrollbarPolicyToml::GutterAlways => ScrollbarPolicy::GutterAlways,
    }
}

fn material_from_toml(
    value: MaterialToml,
    current: scene::Material,
    palette: &HashMap<String, scene::Color>,
    field: &'static str,
) -> Result<scene::Material, ThemeTomlError> {
    match value {
        MaterialToml::Solid { color } => Ok(scene::Material::solid(brush_from_toml(
            field, color, palette,
        )?)),
        MaterialToml::Glass {
            recipe,
            blur_sigma,
            tint,
            tint_opacity,
            luminosity_opacity,
            noise_opacity,
            fallback,
            refraction_displacement,
            refraction_splay,
            refraction_feather,
            refraction_curve,
        } => {
            let mut glass = match recipe {
                Some(recipe) => glass_recipe_from_toml(field, &recipe)?,
                None => match current {
                    scene::Material::Glass(glass) => glass,
                    scene::Material::Solid(_) => scene::Glass::panel_dark(),
                },
            };

            if let Some(blur_sigma) = blur_sigma {
                glass = glass.with_blur_sigma(blur_sigma);
            }

            if tint.is_some() || tint_opacity.is_some() {
                let (current_tint, current_opacity) = glass
                    .tint()
                    .unwrap_or((scene::Brush::solid(scene::Color::rgba(0, 0, 0, 0)), 0.0));
                let tint = match tint {
                    Some(tint) => brush_from_toml("floating-panel.material.tint", tint, palette)?,
                    None => current_tint,
                };
                glass = glass.with_tint(tint, tint_opacity.unwrap_or(current_opacity));
            }

            if let Some(opacity) = luminosity_opacity {
                glass = glass.with_luminosity_opacity(opacity);
            }

            if let Some(opacity) = noise_opacity {
                glass = glass.with_noise_opacity(opacity);
            }

            if let Some(fallback) = fallback {
                glass = glass.with_fallback(brush_from_toml(
                    "floating-panel.material.fallback",
                    fallback,
                    palette,
                )?);
            }

            if refraction_displacement.is_some()
                || refraction_splay.is_some()
                || refraction_feather.is_some()
                || refraction_curve.is_some()
            {
                let current = glass.refraction();
                let displacement = refraction_displacement
                    .or_else(|| current.map(|refraction| refraction.displacement()))
                    .unwrap_or(3.0);
                let splay = refraction_splay
                    .or_else(|| current.map(|refraction| refraction.splay()))
                    .unwrap_or(0.0);
                let feather = refraction_feather
                    .or_else(|| current.map(|refraction| refraction.feather()))
                    .unwrap_or(18.0);
                let curve = refraction_curve
                    .or_else(|| current.map(|refraction| refraction.curve()))
                    .unwrap_or(2.0);
                let refraction = (displacement > 0.0)
                    .then(|| scene::Refraction::new(displacement, splay, feather, curve));
                glass = glass.with_refraction(refraction);
            }

            Ok(scene::Material::glass(glass))
        }
    }
}

fn glass_recipe_from_toml(
    field: &'static str,
    recipe: &str,
) -> Result<scene::Glass, ThemeTomlError> {
    match recipe {
        "panel-dark" => Ok(scene::Glass::panel_dark()),
        "panel-light" => Ok(scene::Glass::panel_light()),
        _ => Err(ThemeTomlError::UnknownMaterialRecipe {
            field: field.to_owned(),
            name: recipe.to_owned(),
        }),
    }
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

fn apply_u64(target: &mut u64, value: Option<u64>) {
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

fn brush_from_toml(
    field: &'static str,
    value: BrushToml,
    palette: &HashMap<String, scene::Color>,
) -> Result<scene::Brush, ThemeTomlError> {
    match value {
        BrushToml::Color(value) => Ok(scene::Brush::solid(parse_color(field, &value, palette)?)),
        BrushToml::LinearGradient { from, to } => Ok(scene::Brush::linear_gradient(
            parse_color(field, &from, palette)?,
            parse_color(field, &to, palette)?,
        )),
    }
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
            typography: TypographyExport {
                interface_size: theme.typography.interface.size,
                interface_weight: weight_to_toml(theme.typography.interface.weight),
                body_size: theme.typography.body.size,
                body_weight: weight_to_toml(theme.typography.body.weight),
                caption_size: theme.typography.caption.size,
                caption_weight: weight_to_toml(theme.typography.caption.weight),
                hint_size: theme.typography.hint.size,
                hint_weight: weight_to_toml(theme.typography.hint.weight),
            },
            focus: FocusExport {
                color: color_string(theme.focus.color, theme.palette),
                outline: color_string(theme.focus.outline, theme.palette),
                width: theme.focus.width,
                offset: theme.focus.offset,
            },
            control: ControlExport {
                background: color_string(theme.control.background, theme.palette),
                button_background: color_string(theme.control.button_background, theme.palette),
                disabled_background: color_string(theme.control.disabled_background, theme.palette),
                hover_tint: color_string(theme.control.hover_tint, theme.palette),
                pressed_tint: color_string(theme.control.pressed_tint, theme.palette),
                rounding: rounding_to_toml("control.rounding", theme.control.rounding)?,
                height: theme.control.height,
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
                panel_min_width: theme.menu.panel_min_width,
                padding: theme.menu.padding,
            },
            choice: ChoiceExport {
                background: color_string(theme.choice.background, theme.palette),
                mark: color_string(theme.choice.mark, theme.palette),
                outline: color_string(theme.choice.outline, theme.palette),
                hover_tint: color_string(theme.choice.hover_tint, theme.palette),
                pressed_tint: color_string(theme.choice.pressed_tint, theme.palette),
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
                foreground: color_string(theme.text_input.foreground, theme.palette),
                placeholder: color_string(theme.text_input.placeholder, theme.palette),
                caret: color_string(theme.text_input.caret, theme.palette),
                padding_x: theme.text_input.padding_x,
            },
            floating_panel: FloatingPanelExport {
                material: material_to_toml(
                    &theme.floating_panel.material,
                    theme.palette,
                    theme.variant,
                ),
                rounding: rounding_to_toml(
                    "floating-panel.rounding",
                    theme.floating_panel.rounding,
                )?,
                border: color_string(theme.floating_panel.border, theme.palette),
                shadow: color_string(theme.floating_panel.shadow, theme.palette),
                shadow_blur: theme.floating_panel.shadow_blur,
                shadow_spread: theme.floating_panel.shadow_spread,
                shadow_offset_y: theme.floating_panel.shadow_offset_y,
                padding: theme.floating_panel.padding,
                content_gap: theme.floating_panel.content_gap,
            },
            auxiliary_panel: AuxiliaryPanelExport {
                hover_delay_ms: theme.auxiliary_panel.hover_delay_ms,
                max_width: theme.auxiliary_panel.max_width,
                max_height: theme.auxiliary_panel.max_height,
                icon_extent: theme.auxiliary_panel.icon_extent,
                icon_gap: theme.auxiliary_panel.icon_gap,
                info: color_string(theme.auxiliary_panel.info, theme.palette),
                warning: color_string(theme.auxiliary_panel.warning, theme.palette),
                error: color_string(theme.auxiliary_panel.error, theme.palette),
            },
            overlay: OverlayExport {
                enter_fade_ms: theme.overlay.enter_fade_ms,
                exit_fade_ms: theme.overlay.exit_fade_ms,
            },
            viewport: ViewportExport {
                min_viewport_extent: theme.viewport.min_viewport_extent,
                reveal_margin: theme.viewport.reveal_margin,
            },
            scrollbar: ScrollbarExport {
                policy: match theme.scrollbar.metrics.policy {
                    ScrollbarPolicy::OverlayAuto => "overlay-auto",
                    ScrollbarPolicy::GutterAlways => "gutter-always",
                },
                thickness: theme.scrollbar.metrics.thickness,
                overlay_thickness: theme.scrollbar.appearance.overlay_thickness,
                hover_thickness: theme.scrollbar.appearance.hover_thickness,
                min_thumb_length: theme.scrollbar.appearance.min_thumb_length,
                margin: theme.scrollbar.appearance.margin,
                fade_delay_ms: theme.scrollbar.appearance.fade_delay_ms,
                fade_duration_ms: theme.scrollbar.appearance.fade_duration_ms,
                track: color_string(theme.scrollbar.appearance.track, theme.palette),
                thumb: color_string(theme.scrollbar.appearance.thumb, theme.palette),
                thumb_hover_tint: color_string(
                    theme.scrollbar.appearance.thumb_hover_tint,
                    theme.palette,
                ),
                thumb_pressed_tint: color_string(
                    theme.scrollbar.appearance.thumb_pressed_tint,
                    theme.palette,
                ),
                corner: color_string(theme.scrollbar.appearance.corner, theme.palette),
                rounding: rounding_to_toml(
                    "scrollbar.rounding",
                    theme.scrollbar.appearance.rounding,
                )?,
            },
            command_palette: CommandPaletteExport {
                section_alignment: text_align_to_toml(theme.command_palette.section_alignment),
                max_results_height: theme.command_palette.max_results_height,
            },
            shortcuts: ShortcutsExport {
                display: shortcut_display_to_toml(theme.shortcuts.display),
            },
        })
    }

    fn to_toml_string(&self) -> String {
        let mut out = String::new();

        push_string_field(&mut out, "variant", self.variant);
        push_blank_line(&mut out);

        push_header(&mut out, "palette");
        push_string_field(&mut out, "accent", &self.palette.accent);
        push_blank_line(&mut out);

        push_header(&mut out, "surfaces");
        push_string_field(&mut out, "canvas", &self.surfaces.canvas);
        push_string_field(&mut out, "root", &self.surfaces.root);
        push_string_field(&mut out, "panel", &self.surfaces.panel);
        push_blank_line(&mut out);

        push_header(&mut out, "text");
        push_string_field(&mut out, "primary", &self.text.primary);
        push_string_field(&mut out, "inverse", &self.text.inverse);
        push_string_field(&mut out, "muted", &self.text.muted);
        push_string_field(&mut out, "selection", &self.text.selection);
        push_blank_line(&mut out);

        push_header(&mut out, "typography");
        push_f32_field(&mut out, "interface-size", self.typography.interface_size);
        push_string_field(
            &mut out,
            "interface-weight",
            self.typography.interface_weight,
        );
        push_f32_field(&mut out, "body-size", self.typography.body_size);
        push_string_field(&mut out, "body-weight", self.typography.body_weight);
        push_f32_field(&mut out, "caption-size", self.typography.caption_size);
        push_string_field(&mut out, "caption-weight", self.typography.caption_weight);
        push_f32_field(&mut out, "hint-size", self.typography.hint_size);
        push_string_field(&mut out, "hint-weight", self.typography.hint_weight);
        push_blank_line(&mut out);

        push_header(&mut out, "focus");
        push_string_field(&mut out, "color", &self.focus.color);
        push_string_field(&mut out, "outline", &self.focus.outline);
        push_i32_field(&mut out, "width", self.focus.width);
        push_f32_field(&mut out, "offset", self.focus.offset);
        push_blank_line(&mut out);

        push_header(&mut out, "control");
        push_string_field(&mut out, "background", &self.control.background);
        push_string_field(
            &mut out,
            "button-background",
            &self.control.button_background,
        );
        push_string_field(
            &mut out,
            "disabled-background",
            &self.control.disabled_background,
        );
        push_string_field(&mut out, "hover-tint", &self.control.hover_tint);
        push_string_field(&mut out, "pressed-tint", &self.control.pressed_tint);
        push_rounding_field(&mut out, "rounding", &self.control.rounding);
        push_i32_field(&mut out, "height", self.control.height);
        push_i32_field(&mut out, "padding", self.control.padding);
        push_blank_line(&mut out);

        push_header(&mut out, "menu");
        push_string_field(&mut out, "bar-background", &self.menu.bar_background);
        push_string_field(&mut out, "title-background", &self.menu.title_background);
        push_string_field(&mut out, "title-hover-tint", &self.menu.title_hover_tint);
        push_string_field(
            &mut out,
            "title-pressed-tint",
            &self.menu.title_pressed_tint,
        );
        push_string_field(&mut out, "title-active-tint", &self.menu.title_active_tint);
        push_string_field(&mut out, "row-background", &self.menu.row_background);
        push_string_field(&mut out, "row-hover-tint", &self.menu.row_hover_tint);
        push_string_field(&mut out, "row-pressed-tint", &self.menu.row_pressed_tint);
        push_string_field(&mut out, "separator", &self.menu.separator);
        push_i32_field(&mut out, "bar-height", self.menu.bar_height);
        push_i32_field(&mut out, "row-height", self.menu.row_height);
        push_i32_field(
            &mut out,
            "separator-line-height",
            self.menu.separator_line_height,
        );
        push_i32_field(&mut out, "panel-min-width", self.menu.panel_min_width);
        push_i32_field(&mut out, "padding", self.menu.padding);
        push_blank_line(&mut out);

        push_header(&mut out, "choice");
        push_string_field(&mut out, "background", &self.choice.background);
        push_string_field(&mut out, "mark", &self.choice.mark);
        push_string_field(&mut out, "outline", &self.choice.outline);
        push_string_field(&mut out, "hover-tint", &self.choice.hover_tint);
        push_string_field(&mut out, "pressed-tint", &self.choice.pressed_tint);
        push_string_field(&mut out, "indicator", &self.choice.indicator);
        push_i32_field(&mut out, "mark-size", self.choice.mark_size);
        push_i32_field(&mut out, "mark-inset", self.choice.mark_inset);
        push_i32_field(&mut out, "label-gap", self.choice.label_gap);
        push_f32_field(&mut out, "icon-size", self.choice.icon_size);
        push_blank_line(&mut out);

        push_header(&mut out, "slider");
        push_string_field(&mut out, "background", &self.slider.background);
        push_string_field(&mut out, "track", &self.slider.track);
        push_string_field(&mut out, "value", &self.slider.value);
        push_string_field(&mut out, "thumb", &self.slider.thumb);
        push_string_field(&mut out, "thumb-outline", &self.slider.thumb_outline);
        push_i32_field(&mut out, "label-width", self.slider.label_width);
        push_i32_field(&mut out, "inset", self.slider.inset);
        push_i32_field(&mut out, "gap", self.slider.gap);
        push_i32_field(&mut out, "track-height", self.slider.track_height);
        push_i32_field(&mut out, "thumb-width", self.slider.thumb_width);
        push_i32_field(&mut out, "thumb-height", self.slider.thumb_height);
        push_blank_line(&mut out);

        push_header(&mut out, "text-input");
        push_string_field(
            &mut out,
            "area-background",
            &self.text_input.area_background,
        );
        push_string_field(
            &mut out,
            "field-background",
            &self.text_input.field_background,
        );
        push_string_field(&mut out, "foreground", &self.text_input.foreground);
        push_string_field(&mut out, "placeholder", &self.text_input.placeholder);
        push_string_field(&mut out, "caret", &self.text_input.caret);
        push_i32_field(&mut out, "padding-x", self.text_input.padding_x);
        push_blank_line(&mut out);

        push_header(&mut out, "floating-panel");
        push_material_field(&mut out, "material", &self.floating_panel.material);
        push_rounding_field(&mut out, "rounding", &self.floating_panel.rounding);
        push_string_field(&mut out, "border", &self.floating_panel.border);
        push_string_field(&mut out, "shadow", &self.floating_panel.shadow);
        push_f32_field(&mut out, "shadow-blur", self.floating_panel.shadow_blur);
        push_f32_field(&mut out, "shadow-spread", self.floating_panel.shadow_spread);
        push_f32_field(
            &mut out,
            "shadow-offset-y",
            self.floating_panel.shadow_offset_y,
        );
        push_i32_field(&mut out, "padding", self.floating_panel.padding);
        push_i32_field(&mut out, "content-gap", self.floating_panel.content_gap);
        push_blank_line(&mut out);

        push_header(&mut out, "auxiliary-panel");
        push_u64_field(
            &mut out,
            "hover-delay-ms",
            self.auxiliary_panel.hover_delay_ms,
        );
        push_i32_field(&mut out, "max-width", self.auxiliary_panel.max_width);
        push_i32_field(&mut out, "max-height", self.auxiliary_panel.max_height);
        push_i32_field(&mut out, "icon-extent", self.auxiliary_panel.icon_extent);
        push_i32_field(&mut out, "icon-gap", self.auxiliary_panel.icon_gap);
        push_string_field(&mut out, "info", &self.auxiliary_panel.info);
        push_string_field(&mut out, "warning", &self.auxiliary_panel.warning);
        push_string_field(&mut out, "error", &self.auxiliary_panel.error);
        push_blank_line(&mut out);

        push_header(&mut out, "overlay");
        push_u64_field(&mut out, "enter-fade-ms", self.overlay.enter_fade_ms);
        push_u64_field(&mut out, "exit-fade-ms", self.overlay.exit_fade_ms);
        push_blank_line(&mut out);

        push_header(&mut out, "viewport");
        push_i32_field(
            &mut out,
            "min-viewport-extent",
            self.viewport.min_viewport_extent,
        );
        push_i32_field(&mut out, "reveal-margin", self.viewport.reveal_margin);
        push_blank_line(&mut out);

        push_header(&mut out, "scrollbar");
        push_string_field(&mut out, "policy", self.scrollbar.policy);
        push_i32_field(&mut out, "thickness", self.scrollbar.thickness);
        push_i32_field(
            &mut out,
            "overlay-thickness",
            self.scrollbar.overlay_thickness,
        );
        push_i32_field(&mut out, "hover-thickness", self.scrollbar.hover_thickness);
        push_i32_field(
            &mut out,
            "min-thumb-length",
            self.scrollbar.min_thumb_length,
        );
        push_i32_field(&mut out, "margin", self.scrollbar.margin);
        push_u64_field(&mut out, "fade-delay-ms", self.scrollbar.fade_delay_ms);
        push_u64_field(
            &mut out,
            "fade-duration-ms",
            self.scrollbar.fade_duration_ms,
        );
        push_string_field(&mut out, "track", &self.scrollbar.track);
        push_string_field(&mut out, "thumb", &self.scrollbar.thumb);
        push_string_field(
            &mut out,
            "thumb-hover-tint",
            &self.scrollbar.thumb_hover_tint,
        );
        push_string_field(
            &mut out,
            "thumb-pressed-tint",
            &self.scrollbar.thumb_pressed_tint,
        );
        push_string_field(&mut out, "corner", &self.scrollbar.corner);
        push_rounding_field(&mut out, "rounding", &self.scrollbar.rounding);
        push_blank_line(&mut out);

        push_header(&mut out, "command-palette");
        push_string_field(
            &mut out,
            "section-alignment",
            self.command_palette.section_alignment,
        );
        push_i32_field(
            &mut out,
            "max-results-height",
            self.command_palette.max_results_height,
        );
        push_blank_line(&mut out);

        push_header(&mut out, "shortcuts");
        push_string_field(&mut out, "display", self.shortcuts.display);

        out
    }
}

fn brush_to_toml(brush: scene::Brush, palette: Palette) -> BrushToml {
    match brush {
        scene::Brush::Solid(color) => BrushToml::Color(color_string(color, palette)),
        scene::Brush::LinearGradient { from, to } => BrushToml::LinearGradient {
            from: color_string(from, palette),
            to: color_string(to, palette),
        },
    }
}

fn material_to_toml(
    material: &scene::Material,
    palette: Palette,
    variant: Variant,
) -> MaterialToml {
    match material {
        scene::Material::Solid(brush) => MaterialToml::Solid {
            color: brush_to_toml(*brush, palette),
        },
        scene::Material::Glass(glass) => {
            let blur = glass.blur().unwrap_or(scene::BackdropBlur::new(0.0));
            let luminosity = glass.luminosity();
            let (tint, tint_opacity) = glass
                .tint()
                .unwrap_or((scene::Brush::solid(scene::Color::rgba(0, 0, 0, 0)), 0.0));
            let noise = glass.noise();
            let refraction = glass.refraction();

            MaterialToml::Glass {
                recipe: Some(match variant {
                    Variant::Dark => "panel-dark".to_owned(),
                    Variant::Light => "panel-light".to_owned(),
                }),
                blur_sigma: Some(blur.sigma()),
                tint: Some(brush_to_toml(tint, palette)),
                tint_opacity: Some(tint_opacity),
                luminosity_opacity: luminosity.map(|luminosity| luminosity.opacity()),
                noise_opacity: noise.map(|noise| noise.opacity()),
                fallback: Some(brush_to_toml(glass.fallback(), palette)),
                refraction_displacement: refraction.map(|refraction| refraction.displacement()),
                refraction_splay: refraction.map(|refraction| refraction.splay()),
                refraction_feather: refraction.map(|refraction| refraction.feather()),
                refraction_curve: refraction.map(|refraction| refraction.curve()),
            }
        }
    }
}

fn push_header(out: &mut String, name: &str) {
    out.push('[');
    out.push_str(name);
    out.push_str("]\n");
}

fn push_blank_line(out: &mut String) {
    out.push('\n');
}

fn push_string_field(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&toml_string(value));
    out.push('\n');
}

fn push_i32_field(out: &mut String, key: &str, value: i32) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&value.to_string());
    out.push('\n');
}

fn push_f32_field(out: &mut String, key: &str, value: f32) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&f32_literal(value));
    out.push('\n');
}

fn push_u64_field(out: &mut String, key: &str, value: u64) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&value.to_string());
    out.push('\n');
}

fn push_rounding_field(out: &mut String, key: &str, value: &RoundingToml) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&rounding_literal(value));
    out.push('\n');
}

fn push_material_field(out: &mut String, key: &str, value: &MaterialToml) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&material_literal(value));
    out.push('\n');
}

fn rounding_literal(value: &RoundingToml) -> String {
    match value {
        RoundingToml::Name(name) => toml_string(name),
        RoundingToml::Fixed { fixed } => format!("{{ fixed = {} }}", f32_literal(*fixed)),
        RoundingToml::Relative { relative } => {
            format!("{{ relative = {} }}", f32_literal(*relative))
        }
    }
}

fn brush_literal(value: &BrushToml) -> String {
    match value {
        BrushToml::Color(color) => toml_string(color),
        BrushToml::LinearGradient { from, to } => {
            format!(
                "{{ from = {}, to = {} }}",
                toml_string(from),
                toml_string(to)
            )
        }
    }
}

fn material_literal(value: &MaterialToml) -> String {
    match value {
        MaterialToml::Solid { color } => {
            format!(
                "{{ kind = {}, color = {} }}",
                toml_string("solid"),
                brush_literal(color)
            )
        }
        MaterialToml::Glass {
            recipe,
            blur_sigma,
            tint,
            tint_opacity,
            luminosity_opacity,
            noise_opacity,
            fallback,
            refraction_displacement,
            refraction_splay,
            refraction_feather,
            refraction_curve,
        } => {
            let mut fields = vec![format!("kind = {}", toml_string("glass"))];
            if let Some(recipe) = recipe {
                fields.push(format!("recipe = {}", toml_string(recipe)));
            }
            if let Some(blur_sigma) = blur_sigma {
                fields.push(format!("blur-sigma = {}", f32_literal(*blur_sigma)));
            }
            if let Some(tint) = tint {
                fields.push(format!("tint = {}", brush_literal(tint)));
            }
            if let Some(tint_opacity) = tint_opacity {
                fields.push(format!("tint-opacity = {}", f32_literal(*tint_opacity)));
            }
            if let Some(luminosity_opacity) = luminosity_opacity {
                fields.push(format!(
                    "luminosity-opacity = {}",
                    f32_literal(*luminosity_opacity)
                ));
            }
            if let Some(noise_opacity) = noise_opacity {
                fields.push(format!("noise-opacity = {}", f32_literal(*noise_opacity)));
            }
            if let Some(fallback) = fallback {
                fields.push(format!("fallback = {}", brush_literal(fallback)));
            }
            if let Some(displacement) = refraction_displacement {
                fields.push(format!(
                    "refraction-displacement = {}",
                    f32_literal(*displacement)
                ));
            }
            if let Some(splay) = refraction_splay {
                fields.push(format!("refraction-splay = {}", f32_literal(*splay)));
            }
            if let Some(feather) = refraction_feather {
                fields.push(format!("refraction-feather = {}", f32_literal(*feather)));
            }
            if let Some(curve) = refraction_curve {
                fields.push(format!("refraction-curve = {}", f32_literal(*curve)));
            }

            format!("{{ {} }}", fields.join(", "))
        }
    }
}

fn toml_string(value: &str) -> String {
    let mut quoted = String::from("\"");

    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            character if character.is_control() => {
                quoted.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => quoted.push(character),
        }
    }

    quoted.push('"');
    quoted
}

fn f32_literal(value: f32) -> String {
    let mut literal = if value.is_nan() {
        "nan".to_owned()
    } else if value == f32::INFINITY {
        "inf".to_owned()
    } else if value == f32::NEG_INFINITY {
        "-inf".to_owned()
    } else {
        value.to_string()
    };

    if !literal.contains('.') && !literal.contains('e') && !literal.contains('E') {
        literal.push_str(".0");
    }

    literal
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
        scene::Radius::Fixed(0.0) => Ok(RoundingToml::Name("none".to_owned())),
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

            [focus]
            offset = 3.5

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
        assert_eq!(theme.focus().offset, 3.5);
    }

    #[test]
    fn floating_panel_defaults_match_production_glass_tokens() {
        let theme = Theme::dark();
        let floating = theme.floating_panel();

        assert_eq!(
            floating.material,
            scene::Material::glass(scene::Glass::panel_dark())
        );
        assert_eq!(floating.rounding, scene::Rounding::fixed(10.0));
        assert_eq!(floating.padding, 6);
        assert_eq!(floating.content_gap, 6);
        assert_eq!(floating.shadow_blur, 24.0);
        assert_eq!(floating.shadow_spread, 0.5);
        assert_eq!(floating.shadow_offset_y, 10.0);
    }

    #[test]
    fn floating_panel_toml_accepts_gradient_solid_and_round_trips() {
        let theme = Theme::from_toml_str(
            r##"
            [palette]
            accent = "#3366cc"
            glass-end = "#44556677"

            [floating-panel]
            material = { kind = "glass", recipe = "panel-dark", blur-sigma = 24.0, tint = { from = "#11223344", to = "glass-end" }, tint-opacity = 1.0, luminosity-opacity = 0.72, noise-opacity = 0.03, fallback = "transparent", refraction-displacement = 3.0, refraction-splay = 3.0, refraction-feather = 14.0, refraction-curve = 3.0 }
            rounding = { fixed = 12.0 }
            shadow = "accent"
            shadow-blur = 11.5
            shadow-spread = 0.25
            shadow-offset-y = 5.0
            padding = 9
            content-gap = 11
            "##,
        )
        .expect("theme should parse");
        let floating = theme.floating_panel();
        let expected_material = scene::Material::glass(
            scene::Glass::panel_dark()
                .with_blur_sigma(24.0)
                .with_tint(
                    scene::Brush::linear_gradient(
                        scene::Color::rgba(17, 34, 51, 68),
                        scene::Color::rgba(68, 85, 102, 119),
                    ),
                    1.0,
                )
                .with_luminosity_opacity(0.72)
                .with_noise_opacity(0.03)
                .with_fallback(scene::Brush::solid(scene::Color::rgba(0, 0, 0, 0)))
                .with_refraction(Some(scene::Refraction::new(3.0, 3.0, 14.0, 3.0))),
        );

        assert_eq!(floating.material, expected_material);
        assert_eq!(floating.rounding, scene::Rounding::fixed(12.0));
        assert_eq!(floating.shadow, scene::Color::rgb(51, 102, 204));
        assert_eq!(floating.shadow_blur, 11.5);
        assert_eq!(floating.shadow_spread, 0.25);
        assert_eq!(floating.shadow_offset_y, 5.0);
        assert_eq!(floating.padding, 9);
        assert_eq!(floating.content_gap, 11);

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn overlay_fade_tokens_parse_and_round_trip() {
        let theme = Theme::from_toml_str(
            r##"
            [overlay]
            enter-fade-ms = 45
            exit-fade-ms = 0
            "##,
        )
        .expect("overlay theme should parse");

        assert_eq!(theme.overlay().enter_fade_ms, 45);
        assert_eq!(theme.overlay().exit_fade_ms, 0);

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        assert!(serialized.contains("[overlay]\n"));
        assert!(serialized.contains("enter-fade-ms = 45\n"));
        assert!(serialized.contains("exit-fade-ms = 0\n"));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn text_input_color_tokens_parse_and_round_trip() {
        let theme = Theme::from_toml_str(
            r##"
            [text-input]
            area-background = "#101112"
            field-background = "#202124"
            foreground = "#f5f5f7"
            placeholder = "#8e8e93"
            caret = "accent"
            padding-x = 11
            "##,
        )
        .expect("text input theme should parse");

        assert_eq!(
            theme.text_input().area_background,
            scene::Color::rgb(16, 17, 18)
        );
        assert_eq!(
            theme.text_input().field_background,
            scene::Color::rgb(32, 33, 36)
        );
        assert_eq!(
            theme.text_input().foreground,
            scene::Color::rgb(245, 245, 247)
        );
        assert_eq!(
            theme.text_input().placeholder,
            scene::Color::rgb(142, 142, 147)
        );
        assert_eq!(theme.text_input().caret, theme.palette().accent());
        assert_eq!(theme.text_input().padding_x, 11);

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        assert!(serialized.contains("foreground = \"#f5f5f7\"\n"));
        assert!(serialized.contains("placeholder = \"#8e8e93\"\n"));
        assert!(serialized.contains("caret = \"accent\"\n"));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn choice_state_tints_parse_and_round_trip() {
        let theme = Theme::from_toml_str(
            r##"
            [palette]
            choice-hover = "#00000022"
            choice-pressed = "#00000044"

            [choice]
            hover-tint = "choice-hover"
            pressed-tint = "choice-pressed"
            "##,
        )
        .expect("choice state tints should parse");

        assert_eq!(theme.choice().hover_tint, scene::Color::rgba(0, 0, 0, 34));
        assert_eq!(theme.choice().pressed_tint, scene::Color::rgba(0, 0, 0, 68));

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        assert!(serialized.contains("hover-tint = \"#00000022\"\n"));
        assert!(serialized.contains("pressed-tint = \"#00000044\"\n"));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn scrollbar_policy_and_viewport_tokens_parse_and_round_trip() {
        let theme = Theme::from_toml_str(
            r##"
            [viewport]
            min-viewport-extent = 72

            [scrollbar]
            policy = "gutter-always"
            thickness = 12
            overlay-thickness = 8
            hover-thickness = 16
            min-thumb-length = 24
            margin = 3
            fade-delay-ms = 700
            fade-duration-ms = 220
            track = "#11223344"
            thumb = "accent"
            thumb-hover-tint = "#ffffff22"
            thumb-pressed-tint = "#00000033"
            corner = "transparent"
            rounding = { relative = 1.0 }
            "##,
        )
        .expect("scrollbar theme should parse");

        assert_eq!(theme.viewport().min_viewport_extent, 72);
        assert_eq!(
            theme.scrollbar().metrics.policy,
            ScrollbarPolicy::GutterAlways
        );
        assert_eq!(theme.scrollbar().metrics.thickness, 12);
        assert_eq!(theme.scrollbar().appearance.overlay_thickness, 8);
        assert_eq!(theme.scrollbar().appearance.hover_thickness, 16);
        assert_eq!(theme.scrollbar().appearance.min_thumb_length, 24);
        assert_eq!(theme.scrollbar().appearance.margin, 3);
        assert_eq!(theme.scrollbar().appearance.fade_delay_ms, 700);
        assert_eq!(theme.scrollbar().appearance.fade_duration_ms, 220);
        assert_eq!(
            theme.scrollbar().appearance.track,
            scene::Color::rgba(17, 34, 51, 68)
        );
        assert_eq!(theme.scrollbar().appearance.thumb, theme.palette().accent());

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        assert!(serialized.contains("[viewport]\n"));
        assert!(serialized.contains("[scrollbar]\n"));
        assert!(serialized.contains("policy = \"gutter-always\"\n"));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn shortcut_display_token_parses_and_round_trips() {
        let theme = Theme::from_toml_str(
            r##"
            [shortcuts]
            display = "default"
            "##,
        )
        .expect("shortcut display theme should parse");

        assert_eq!(theme.shortcuts().display(), keymap::DisplayStyle::Default);

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize shortcuts");
        assert!(serialized.contains("[shortcuts]\n"));
        assert!(serialized.contains("display = \"default\""));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed.shortcuts().display(), keymap::DisplayStyle::Default);

        let symbols = Theme::from_toml_str(
            r##"
            [shortcuts]
            display = "symbols"
            "##,
        )
        .expect("symbols shortcut display theme should parse");
        assert_eq!(symbols.shortcuts().display(), keymap::DisplayStyle::Symbols);
    }

    #[test]
    fn typography_and_command_palette_tokens_parse_and_round_trip() {
        let theme = Theme::from_toml_str(
            r##"
            [typography]
            interface-size = 13.0
            interface-weight = "medium"
            body-size = 21.0
            body-weight = "bold"
            caption-size = 9.5
            caption-weight = "medium"
            hint-size = 11.0
            hint-weight = "normal"

            [control]
            height = 31

            [command-palette]
            section-alignment = "end"
            max-results-height = 312
            "##,
        )
        .expect("typography theme should parse");

        assert_eq!(theme.typography().interface().size(), 13.0);
        assert_eq!(
            theme.typography().interface().weight(),
            text_model::document::Weight::Medium
        );
        assert_eq!(theme.typography().body().size(), 21.0);
        assert_eq!(
            theme.typography().body().weight(),
            text_model::document::Weight::Bold
        );
        assert_eq!(theme.typography().caption().size(), 9.5);
        assert_eq!(
            theme.typography().caption().weight(),
            text_model::document::Weight::Medium
        );
        assert_eq!(theme.typography().hint().size(), 11.0);
        assert_eq!(
            theme.typography().hint().weight(),
            text_model::document::Weight::Normal
        );
        assert_eq!(theme.control().height, 31);
        assert_eq!(
            theme.command_palette().section_alignment(),
            scene::TextAlign::End
        );
        assert_eq!(theme.command_palette().max_results_height(), 312);

        let serialized = theme
            .to_toml_string()
            .expect("theme should serialize to TOML");
        assert!(serialized.contains("[typography]\n"));
        assert!(serialized.contains("interface-size = 13.0\n"));
        assert!(serialized.contains("interface-weight = \"medium\"\n"));
        assert!(serialized.contains("body-size = 21.0\n"));
        assert!(serialized.contains("body-weight = \"bold\"\n"));
        assert!(serialized.contains("[control]\n"));
        assert!(serialized.contains("height = 31\n"));
        assert!(serialized.contains("[command-palette]\n"));
        assert!(serialized.contains("section-alignment = \"end\"\n"));
        assert!(serialized.contains("max-results-height = 312\n"));
        let parsed = Theme::from_toml_str(&serialized).expect("theme should parse again");

        assert_eq!(parsed, theme);
    }

    #[test]
    fn interface_typography_derives_caption_and_hint_sizes_when_unset() {
        let derived = Theme::from_toml_str(
            r##"
            [typography]
            interface-size = 14.0
            "##,
        )
        .expect("interface-only typography theme should parse");

        assert_eq!(derived.typography().interface().size(), 14.0);
        assert_eq!(derived.typography().caption().size(), 13.0);
        assert_eq!(derived.typography().hint().size(), 14.0);

        let explicit = Theme::from_toml_str(
            r##"
            [typography]
            interface-size = 14.0
            caption-size = 9.0
            hint-size = 10.0
            "##,
        )
        .expect("explicit typography theme should parse");

        assert_eq!(explicit.typography().interface().size(), 14.0);
        assert_eq!(explicit.typography().caption().size(), 9.0);
        assert_eq!(explicit.typography().hint().size(), 10.0);
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

        for stale_field in ["title-size", "title-weight"] {
            let stale_theme = format!("[menu]\n{stale_field} = 12\n");
            let error = Theme::from_toml_str(&stale_theme)
                .expect_err("old menu title typography fields should fail");
            assert!(error.to_string().contains("unknown field"));
        }

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

        let unknown_gradient_stop = Theme::from_toml_str(
            r##"
            [floating-panel]
            material = { kind = "glass", tint = { from = "missing", to = "#ffffff" } }
            "##,
        )
        .expect_err("unknown palette names in brushes should fail");
        assert!(matches!(
            unknown_gradient_stop,
            ThemeTomlError::UnknownColor { field, name }
                if field == "floating-panel.material.tint" && name == "missing"
        ));

        let unknown_material_recipe = Theme::from_toml_str(
            r##"
            [floating-panel]
            material = { kind = "glass", recipe = "missing-glass" }
            "##,
        )
        .expect_err("unknown material recipes should fail");
        assert!(matches!(
            unknown_material_recipe,
            ThemeTomlError::UnknownMaterialRecipe { field, name }
                if field == "floating-panel.material" && name == "missing-glass"
        ));

        let misspelled_material_field = Theme::from_toml_str(
            r##"
            [floating-panel]
            material = { kind = "glass", blur-sgima = 24.0 }
            "##,
        )
        .expect_err("misspelled material fields should fail");
        assert!(
            misspelled_material_field
                .to_string()
                .contains("unknown field")
        );

        for stale_field in ["backdrop-tint", "filters", "background"] {
            let stale_theme = format!("[floating-panel]\n{stale_field} = \"transparent\"\n");
            let error = Theme::from_toml_str(&stale_theme)
                .expect_err("old floating panel material fields should fail");
            assert!(error.to_string().contains("unknown field"));
        }

        let old_filter_array = Theme::from_toml_str(
            r##"
            [floating-panel]
            filters = [{ kind = "legacy", amount = 0.2 }]
            "##,
        )
        .expect_err("old filter arrays should fail");
        assert!(old_filter_array.to_string().contains("unknown field"));

        let unknown_scrollbar_policy = Theme::from_toml_str(
            r##"
            [scrollbar]
            policy = "overlay"
            "##,
        )
        .expect_err("unknown scrollbar policies should fail");
        assert!(
            unknown_scrollbar_policy
                .to_string()
                .contains("unknown variant")
        );

        let unknown_palette_alignment = Theme::from_toml_str(
            r##"
            [command-palette]
            section-alignment = "middle"
            "##,
        )
        .expect_err("unknown command palette alignments should fail");
        assert!(
            unknown_palette_alignment
                .to_string()
                .contains("unknown variant")
        );

        let unknown_shortcut_display = Theme::from_toml_str(
            r##"
            [shortcuts]
            display = "emoji"
            "##,
        )
        .expect_err("unknown shortcut display styles should fail");
        assert!(
            unknown_shortcut_display
                .to_string()
                .contains("unknown variant")
        );

        let old_platform_shortcut_display = Theme::from_toml_str(
            r##"
            [shortcuts]
            display = "platform"
            "##,
        )
        .expect_err("old platform shortcut display style should fail");
        assert!(
            old_platform_shortcut_display
                .to_string()
                .contains("unknown variant")
        );
    }

    #[test]
    fn serializes_rounding_and_gradient_brushes_as_inline_values() {
        let serialized = Theme::dark()
            .to_toml_string()
            .expect("theme should serialize");

        assert!(serialized.contains("rounding = { fixed = 4.0 }\n"));
        assert!(serialized.contains("material = { kind = \"glass\", recipe = \"panel-dark\", blur-sigma = 44.55, tint = \"#1c1c1e\", tint-opacity = 0.4, luminosity-opacity = 0.92, noise-opacity = 0.022, fallback = \"#1c1c1e\" }\n"));
        assert!(serialized.contains("content-gap = 6\n"));
        assert!(serialized.contains("[viewport]\n"));
        assert!(serialized.contains("[scrollbar]\n"));
        assert!(serialized.contains("policy = \"overlay-auto\"\n"));
        assert!(serialized.contains("[command-palette]\n"));
        assert!(serialized.contains("max-results-height = 260\n"));
        assert!(!serialized.contains("backdrop-blur"));
        assert!(!serialized.contains("backdrop-tint"));
        assert!(!serialized.contains("filters = ["));
        assert!(!serialized.contains("[control.rounding]"));
        assert!(!serialized.contains("[floating-panel.backdrop-tint]"));
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
