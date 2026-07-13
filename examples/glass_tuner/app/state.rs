use wgpu_l3::{scene, theme::Theme};

#[derive(Debug, Clone)]
pub struct State {
    pub panel_open: bool,
    pub comparison_open: bool,
    pub force_promoted: bool,
    pub foreground_mode: ForegroundMode,
    pub blur_sigma: f64,
    pub tint: Rgb,
    pub tint_opacity: f64,
    pub luminosity_opacity: f64,
    pub noise_opacity: f64,
    pub last_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcrylicToken {
    BlurSigma,
    TintOpacity,
    LuminosityOpacity,
    NoiseOpacity,
    TintR,
    TintG,
    TintB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForegroundMode {
    Acrylic,
    OpaqueFallback,
    NoAccent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl State {
    pub fn theme(&self) -> Theme {
        let mut theme = Theme::dark();
        let tint = self.tint.color();
        theme
            .floating_panel_mut()
            .set_material(scene::Material::glass(
                scene::Glass::panel_dark()
                    .with_blur_sigma(self.blur_sigma as f32)
                    .with_refraction(None)
                    .with_luminosity(scene::Luminosity::new(tint, self.luminosity_opacity as f32))
                    .with_tint(scene::Brush::solid(tint), self.tint_opacity as f32)
                    .with_noise_opacity(self.noise_opacity as f32),
            ));
        theme
    }

    pub fn set_token(&mut self, token: AcrylicToken, value: f64) {
        match token {
            AcrylicToken::BlurSigma => self.blur_sigma = value.clamp(0.0, 60.0),
            AcrylicToken::TintOpacity => self.tint_opacity = value.clamp(0.0, 1.0),
            AcrylicToken::LuminosityOpacity => self.luminosity_opacity = value.clamp(0.0, 1.0),
            AcrylicToken::NoiseOpacity => self.noise_opacity = value.clamp(0.0, 0.08),
            AcrylicToken::TintR => self.tint.r = clamp_channel(value),
            AcrylicToken::TintG => self.tint.g = clamp_channel(value),
            AcrylicToken::TintB => self.tint.b = clamp_channel(value),
        }
        self.last_status = format!("{} = {}", token.label(), self.token_value(token));
    }

    pub fn token_value(&self, token: AcrylicToken) -> String {
        match token {
            AcrylicToken::BlurSigma => format!("{:.2}", self.blur_sigma),
            AcrylicToken::TintOpacity => format!("{:.2}", self.tint_opacity),
            AcrylicToken::LuminosityOpacity => format!("{:.2}", self.luminosity_opacity),
            AcrylicToken::NoiseOpacity => format!("{:.3}", self.noise_opacity),
            AcrylicToken::TintR => self.tint.r.to_string(),
            AcrylicToken::TintG => self.tint.g.to_string(),
            AcrylicToken::TintB => self.tint.b.to_string(),
        }
    }

    pub fn toml_patch(&self) -> String {
        format!(
            "[floating-panel]\nmaterial = {{ kind = \"glass\", recipe = \"panel-dark\", blur-sigma = {:.2}, tint = \"{}\", tint-opacity = {:.2}, luminosity-opacity = {:.2}, noise-opacity = {:.3}, fallback = \"#1c1c1e\" }}",
            self.blur_sigma,
            self.tint.hex(),
            self.tint_opacity,
            self.luminosity_opacity,
            self.noise_opacity,
        )
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            panel_open: true,
            comparison_open: true,
            force_promoted: false,
            foreground_mode: ForegroundMode::Acrylic,
            blur_sigma: 44.55,
            tint: Rgb::new(28, 28, 30),
            tint_opacity: 0.40,
            luminosity_opacity: 0.92,
            noise_opacity: 0.022,
            last_status: "ready".to_owned(),
        }
    }
}

impl wgpu_l3::state::State for State {}

impl ForegroundMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Acrylic => "Acrylic",
            Self::OpaqueFallback => "Opaque fallback",
            Self::NoAccent => "No accent",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Acrylic => Self::OpaqueFallback,
            Self::OpaqueFallback => Self::NoAccent,
            Self::NoAccent => Self::Acrylic,
        }
    }

    pub const fn popup_preference(self) -> wgpu_l3::view::NativePopupMaterialPreference {
        match self {
            Self::Acrylic => wgpu_l3::view::NativePopupMaterialPreference::System,
            Self::OpaqueFallback => wgpu_l3::view::NativePopupMaterialPreference::OpaqueFallback,
            Self::NoAccent => wgpu_l3::view::NativePopupMaterialPreference::NoAccent,
        }
    }
}

impl AcrylicToken {
    pub fn label(self) -> &'static str {
        match self {
            Self::BlurSigma => "Blur sigma",
            Self::TintOpacity => "Tint opacity",
            Self::LuminosityOpacity => "Luminosity opacity",
            Self::NoiseOpacity => "Noise opacity",
            Self::TintR => "Tint R",
            Self::TintG => "Tint G",
            Self::TintB => "Tint B",
        }
    }
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn color(self) -> scene::Color {
        scene::Color::rgb(self.r, self.g, self.b)
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn clamp_channel(value: f64) -> u8 {
    value.clamp(0.0, 255.0).round() as u8
}
