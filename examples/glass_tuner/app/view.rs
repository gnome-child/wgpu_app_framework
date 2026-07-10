use super::{
    State,
    command::{CycleForegroundMode, SetToken, ToggleComparison, ToggleForcePromoted, TogglePanel},
    state::{AcrylicToken, ForegroundMode},
};
use wgpu_l3::{
    View, geometry, interaction, scene,
    view::{Align, Context as ViewContext, Dimension, Padding},
    widget,
};

const PANEL_ID: interaction::Id = interaction::Id::new("glass_tuner.panel");
const COMPARISON_ID: interaction::Id = interaction::Id::new("glass_tuner.promotion_comparison");
const FOREGROUND_ID: interaction::Id = interaction::Id::new("glass_tuner.foreground_clarity");
const PANEL_SURFACE_COLOR: scene::Color = scene::Color::rgb(28, 28, 30);

pub const WINDOW_TITLE: &str = "wgpu_l3 Acrylic Tuner";
pub const CANVAS_COLOR: scene::Color = scene::Color::rgb(17, 18, 20);

pub fn window_size() -> geometry::Size {
    geometry::Size::new(980, 760)
}

pub fn view(state: &State, _: ViewContext) -> View {
    widget::view(|ui| {
        ui.add(
            widget::Element::new()
                .column()
                .width(Dimension::grow())
                .height(Dimension::grow())
                .children(|ui| {
                    ui.add(toolbar(state));
                    ui.add(stage(state));
                }),
        );
    })
}

fn toolbar(state: &State) -> widget::Element {
    let label = if state.panel_open {
        "Hide panel"
    } else {
        "Show panel"
    };

    widget::Element::new()
        .row()
        .height(Dimension::fixed(44))
        .layout(|layout| {
            layout
                .gap(8)
                .padding(Padding::all(8))
                .align_items(Align::Center)
        })
        .children(|ui| {
            ui.button(
                widget::Button::new(label)
                    .reserve_labels(["Hide panel", "Show panel"])
                    .trigger::<TogglePanel>(()),
            );
            ui.button(
                widget::Button::new("Compare fade")
                    .reserve_labels(["Compare fade"])
                    .trigger::<ToggleComparison>(()),
            );
            ui.button(
                widget::Button::new("Force promote")
                    .reserve_labels(["Force promote"])
                    .trigger::<ToggleForcePromoted>(()),
            );
            ui.button(
                widget::Button::new(format!("Foreground: {}", state.foreground_mode.label()))
                    .reserve_labels([
                        "Foreground: Acrylic",
                        "Foreground: Opaque fallback",
                        "Foreground: No accent",
                    ])
                    .trigger::<CycleForegroundMode>(()),
            );
            ui.label(format!("Status: {}", state.last_status));
        })
}

fn stage(state: &State) -> widget::Element {
    widget::Element::new()
        .overlay()
        .height(Dimension::grow())
        .layout(|layout| {
            layout
                .padding(Padding::symmetric(24, 18))
                .align_items(Align::Center)
                .justify_content(Align::Start)
        })
        .children(|ui| {
            ui.add(color_bars());
            if state.panel_open {
                ui.add(floating_panel(state));
                ui.add(foreground_panel(state));
            }
            if state.comparison_open {
                ui.add(comparison_panel(state));
            }
        })
}

fn color_bars() -> widget::Element {
    const COLORS: [scene::Color; 10] = [
        scene::Color::rgb(235, 73, 83),
        scene::Color::rgb(243, 153, 54),
        scene::Color::rgb(248, 224, 92),
        scene::Color::rgb(92, 198, 117),
        scene::Color::rgb(68, 194, 191),
        scene::Color::rgb(78, 130, 238),
        scene::Color::rgb(137, 91, 225),
        scene::Color::rgb(242, 99, 180),
        scene::Color::rgb(245, 245, 247),
        scene::Color::rgb(35, 38, 44),
    ];

    widget::Element::new()
        .row()
        .width(Dimension::grow())
        .height(Dimension::grow())
        .children(|ui| {
            for color in COLORS {
                ui.add(
                    widget::Element::new()
                        .background(scene::Brush::solid(color))
                        .width(Dimension::grow())
                        .height(Dimension::grow()),
                );
            }
        })
}

fn floating_panel(state: &State) -> widget::panel::Floating {
    widget::panel::Floating::new(PANEL_ID)
        .offset(32, 18)
        .column()
        .width(Dimension::fixed(520))
        .height(Dimension::fixed(520))
        .children(|ui| {
            add_slider(ui, AcrylicToken::BlurSigma, state.blur_sigma, 0.0..=60.0);
            add_slider(ui, AcrylicToken::TintOpacity, state.tint_opacity, 0.0..=1.0);
            add_slider(
                ui,
                AcrylicToken::LuminosityOpacity,
                state.luminosity_opacity,
                0.0..=1.0,
            );
            add_slider(
                ui,
                AcrylicToken::NoiseOpacity,
                state.noise_opacity,
                0.0..=0.08,
            );
            add_slider(ui, AcrylicToken::TintR, state.tint.r as f64, 0.0..=255.0);
            add_slider(ui, AcrylicToken::TintG, state.tint.g as f64, 0.0..=255.0);
            add_slider(ui, AcrylicToken::TintB, state.tint.b as f64, 0.0..=255.0);

            for line in state.toml_patch().lines() {
                ui.label(line);
            }
        })
}

fn comparison_panel(state: &State) -> widget::panel::Floating {
    widget::panel::Floating::new(COMPARISON_ID)
        .offset(588, 18)
        .diagnostic_force_promoted_at_full_opacity(state.force_promoted)
        .column()
        .width(Dimension::fixed(340))
        .height(Dimension::fixed(240))
        .children(|ui| {
            ui.label("Promotion comparison");
            ui.label(format!("Blur sigma: {:.2}", state.blur_sigma));
            ui.label(format!("Tint opacity: {:.2}", state.tint_opacity));
            ui.label(format!(
                "Luminosity opacity: {:.2}",
                state.luminosity_opacity
            ));
            ui.label(format!("Noise opacity: {:.3}", state.noise_opacity));
            ui.label(format!("Forced group: {}", state.force_promoted));
            ui.label("Alpha witness");
            ui.add(
                widget::Element::new()
                    .width(Dimension::fixed(150))
                    .height(Dimension::fixed(28))
                    .background(scene::Brush::solid(scene::Color::rgba(255, 0, 255, 128))),
            );
            ui.label(state.tint.hex());
        })
}

fn foreground_panel(state: &State) -> widget::panel::Floating {
    widget::panel::Floating::new(FOREGROUND_ID)
        .offset(588, 282)
        .diagnostic_native_popup_material(state.foreground_mode.popup_preference())
        .column()
        .width(Dimension::fixed(340))
        .height(Dimension::fixed(430))
        .layout(|layout| layout.gap(8))
        .children(|ui| {
            ui.label("Foreground clarity");
            ui.label(format!("Mode: {}", state.foreground_mode.label()));
            ui.label("Backed: in-frame surface reference");
            ui.add(foreground_sample(Some(PANEL_SURFACE_COLOR)));
            ui.label("Unbacked: native material boundary");
            ui.add(foreground_sample(None));
            ui.label(foreground_hint(state.foreground_mode));
        })
}

fn foreground_hint(mode: ForegroundMode) -> &'static str {
    match mode {
        ForegroundMode::Acrylic => "Compare against OS acrylic.",
        ForegroundMode::OpaqueFallback => "If crust remains here, suspect scale.",
        ForegroundMode::NoAccent => "Transparent surface with no accent.",
    }
}

fn foreground_sample(backing: Option<scene::Color>) -> widget::Element {
    let mut sample = widget::Element::new()
        .column()
        .width(Dimension::fixed(300))
        .height(Dimension::fixed(132))
        .layout(|layout| {
            layout
                .gap(6)
                .padding(Padding::symmetric(10, 8))
                .align_items(Align::Start)
        })
        .children(foreground_sample_content);

    if let Some(color) = backing {
        sample = sample.background(scene::Brush::solid(color));
    }

    sample
}

fn foreground_sample_content(ui: &mut widget::Ui) {
    ui.add(line(scene::Color::rgba(245, 245, 247, 120), 280, 1));
    ui.row(|ui| {
        ui.add(swatch(scene::Color::rgba(255, 64, 64, 128)));
        ui.add(swatch(scene::Color::rgba(64, 180, 255, 128)));
        ui.add(swatch(scene::Color::rgba(245, 245, 247, 180)));
        ui.label("Half-alpha quads");
    });
    ui.add(grid_strip());
    ui.label("Glyph coverage: Agjpqy 0123456789");
    ui.label("Shortcut glyphs: Ctrl Shift Alt Enter");
    ui.add(slider_specimen());
}

fn grid_strip() -> widget::Element {
    widget::Element::new()
        .row()
        .width(Dimension::fixed(280))
        .height(Dimension::fixed(34))
        .children(|ui| {
            for index in 0..28 {
                let color = if index % 2 == 0 {
                    scene::Color::rgba(245, 245, 247, 220)
                } else {
                    scene::Color::rgba(20, 22, 26, 220)
                };
                ui.add(line(color, 1, 34));
                ui.add(
                    widget::Element::new()
                        .width(Dimension::fixed(9))
                        .height(Dimension::fixed(34)),
                );
            }
        })
}

fn line(color: scene::Color, width: i32, height: i32) -> widget::Element {
    widget::Element::new()
        .width(Dimension::fixed(width))
        .height(Dimension::fixed(height))
        .background(scene::Brush::solid(color))
}

fn swatch(color: scene::Color) -> widget::Element {
    widget::Element::new()
        .width(Dimension::fixed(34))
        .height(Dimension::fixed(22))
        .background(scene::Brush::solid(color))
}

fn slider_specimen() -> widget::Element {
    widget::Element::new()
        .row()
        .width(Dimension::fixed(280))
        .height(Dimension::fixed(18))
        .layout(|layout| layout.gap(8).align_items(Align::Center))
        .children(|ui| {
            ui.add(
                widget::Element::new()
                    .width(Dimension::fixed(196))
                    .height(Dimension::fixed(4))
                    .background(scene::Brush::solid(scene::Color::rgba(245, 245, 247, 72))),
            );
            ui.add(
                widget::Element::new()
                    .width(Dimension::fixed(18))
                    .height(Dimension::fixed(18))
                    .background(scene::Brush::solid(scene::Color::rgba(245, 245, 247, 196))),
            );
            ui.label("Slider AA");
        })
}

fn add_slider(
    ui: &mut widget::Ui,
    token: AcrylicToken,
    value: f64,
    range: std::ops::RangeInclusive<f64>,
) {
    ui.slider(
        widget::Slider::new(token.label(), value, range)
            .trigger_with::<SetToken, _>(move |value| (token, value)),
    );
}
