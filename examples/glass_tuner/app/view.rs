use super::{
    State,
    command::{SetToken, ToggleComparison, TogglePanel},
    state::AcrylicToken,
};
use wgpu_l3::{
    View, geometry, interaction, scene,
    view::{Align, Context as ViewContext, Dimension, Padding},
    widget,
};

const PANEL_ID: interaction::Id = interaction::Id::new("glass_tuner.panel");
const COMPARISON_ID: interaction::Id = interaction::Id::new("glass_tuner.promotion_comparison");

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
            ui.label(state.tint.hex());
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
