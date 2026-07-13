use super::Color;
use super::primitive::Brush;

#[derive(Debug, Clone, PartialEq)]
pub enum Material {
    Solid(Brush),
    Glass(Glass),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Glass {
    fallback: Brush,
    base: GlassBase,
    backdrop_layers: Vec<BackdropLayer>,
    surface_layers: Vec<SurfaceLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlassBase {
    FrameworkBackdrop,
    Transparent,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackdropLayer {
    Blur(BackdropBlur),
    Refraction(Refraction),
    Luminosity(Luminosity),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceLayer {
    Tint { brush: Brush, opacity: f32 },
    Noise(Noise),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackdropBlur {
    sigma: f32,
    edge_mode: BackdropEdgeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackdropEdgeMode {
    Mirror,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Refraction {
    displacement: f32,
    splay: f32,
    feather: f32,
    curve: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Luminosity {
    color: Color,
    opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    opacity: f32,
}

impl Material {
    pub const fn solid(brush: Brush) -> Self {
        Self::Solid(brush)
    }

    pub const fn glass(glass: Glass) -> Self {
        Self::Glass(glass)
    }

    pub(crate) fn without_backdrop_sampling(&self) -> Self {
        match self {
            Self::Solid(brush) => Self::Solid(*brush),
            Self::Glass(glass) => Self::Glass(glass.without_backdrop_layers()),
        }
    }

    pub(crate) fn without_realized_parts(&self, parts: super::RealizedMaterialParts) -> Self {
        match self {
            Self::Solid(brush) => Self::Solid(*brush),
            Self::Glass(glass) => Self::Glass(glass.without_realized_parts(parts)),
        }
    }
}

impl Glass {
    pub fn panel_dark() -> Self {
        Self {
            fallback: Brush::solid(Color::rgb(28, 28, 30)),
            base: GlassBase::FrameworkBackdrop,
            backdrop_layers: vec![
                BackdropLayer::Blur(BackdropBlur::new(44.55)),
                BackdropLayer::Luminosity(Luminosity::new(Color::rgb(28, 28, 30), 0.92)),
            ],
            surface_layers: vec![
                SurfaceLayer::Tint {
                    brush: Brush::solid(Color::rgb(28, 28, 30)),
                    opacity: 0.40,
                },
                SurfaceLayer::Noise(Noise::new(0.022)),
            ],
        }
    }

    pub fn panel_light() -> Self {
        Self {
            fallback: Brush::solid(Color::rgb(249, 249, 249)),
            base: GlassBase::FrameworkBackdrop,
            backdrop_layers: vec![
                BackdropLayer::Blur(BackdropBlur::new(30.0)),
                BackdropLayer::Luminosity(Luminosity::new(Color::rgb(252, 252, 252), 0.85)),
            ],
            surface_layers: vec![
                SurfaceLayer::Tint {
                    brush: Brush::solid(Color::rgb(252, 252, 252)),
                    opacity: 0.0,
                },
                SurfaceLayer::Noise(Noise::new(0.02)),
            ],
        }
    }

    pub fn fallback(&self) -> Brush {
        self.fallback
    }

    pub(crate) const fn base(&self) -> GlassBase {
        self.base
    }

    pub fn backdrop_layers(&self) -> &[BackdropLayer] {
        &self.backdrop_layers
    }

    pub fn surface_layers(&self) -> &[SurfaceLayer] {
        &self.surface_layers
    }

    pub(crate) fn without_backdrop_layers(&self) -> Self {
        Self {
            fallback: self.fallback,
            base: GlassBase::Fallback,
            backdrop_layers: Vec::new(),
            surface_layers: self.surface_layers.clone(),
        }
    }

    pub(crate) fn without_realized_parts(&self, parts: super::RealizedMaterialParts) -> Self {
        let mut residual = self.clone();
        if parts.backdrop_frost() {
            residual.backdrop_layers.clear();
            residual.base = GlassBase::Transparent;
        }
        if parts.surface_tint() {
            residual
                .surface_layers
                .retain(|layer| !matches!(layer, SurfaceLayer::Tint { .. }));
        }
        residual
    }

    pub fn blur(&self) -> Option<BackdropBlur> {
        self.backdrop_layers.iter().find_map(|layer| match layer {
            BackdropLayer::Blur(blur) => Some(*blur),
            _ => None,
        })
    }

    pub fn refraction(&self) -> Option<Refraction> {
        self.backdrop_layers.iter().find_map(|layer| match layer {
            BackdropLayer::Refraction(refraction) => Some(*refraction),
            _ => None,
        })
    }

    pub fn luminosity(&self) -> Option<Luminosity> {
        self.backdrop_layers.iter().find_map(|layer| match layer {
            BackdropLayer::Luminosity(luminosity) => Some(*luminosity),
            _ => None,
        })
    }

    pub fn tint(&self) -> Option<(Brush, f32)> {
        self.surface_layers.iter().find_map(|layer| match layer {
            SurfaceLayer::Tint { brush, opacity } => Some((*brush, *opacity)),
            _ => None,
        })
    }

    pub fn noise(&self) -> Option<Noise> {
        self.surface_layers.iter().find_map(|layer| match layer {
            SurfaceLayer::Noise(noise) => Some(*noise),
            _ => None,
        })
    }

    pub fn with_blur_sigma(mut self, sigma: f32) -> Self {
        let blur = BackdropBlur::new(sigma);
        if let Some(layer) = self
            .backdrop_layers
            .iter_mut()
            .find(|layer| matches!(layer, BackdropLayer::Blur(_)))
        {
            *layer = BackdropLayer::Blur(blur);
        } else {
            self.backdrop_layers.insert(0, BackdropLayer::Blur(blur));
        }
        self
    }

    pub fn with_refraction(mut self, refraction: Option<Refraction>) -> Self {
        self.backdrop_layers
            .retain(|layer| !matches!(layer, BackdropLayer::Refraction(_)));
        if let Some(refraction) = refraction {
            let insert_at = self
                .backdrop_layers
                .iter()
                .position(|layer| matches!(layer, BackdropLayer::Luminosity(_)))
                .unwrap_or(self.backdrop_layers.len());
            self.backdrop_layers
                .insert(insert_at, BackdropLayer::Refraction(refraction));
        }
        self
    }

    pub fn with_luminosity(mut self, luminosity: Luminosity) -> Self {
        if let Some(layer) = self
            .backdrop_layers
            .iter_mut()
            .find(|layer| matches!(layer, BackdropLayer::Luminosity(_)))
        {
            *layer = BackdropLayer::Luminosity(luminosity);
        } else {
            self.backdrop_layers
                .push(BackdropLayer::Luminosity(luminosity));
        }
        self
    }

    pub fn with_luminosity_opacity(mut self, opacity: f32) -> Self {
        let color = self
            .luminosity()
            .map(|luminosity| luminosity.color())
            .unwrap_or(Color::rgb(255, 255, 255));
        self = self.with_luminosity(Luminosity::new(color, opacity));
        self
    }

    pub fn with_tint(mut self, brush: Brush, opacity: f32) -> Self {
        let opacity = opacity.clamp(0.0, 1.0);
        if let Some(layer) = self
            .surface_layers
            .iter_mut()
            .find(|layer| matches!(layer, SurfaceLayer::Tint { .. }))
        {
            *layer = SurfaceLayer::Tint { brush, opacity };
        } else {
            self.surface_layers
                .insert(0, SurfaceLayer::Tint { brush, opacity });
        }
        self
    }

    pub fn with_noise_opacity(mut self, opacity: f32) -> Self {
        let noise = Noise::new(opacity);
        if let Some(layer) = self
            .surface_layers
            .iter_mut()
            .find(|layer| matches!(layer, SurfaceLayer::Noise(_)))
        {
            *layer = SurfaceLayer::Noise(noise);
        } else {
            self.surface_layers.push(SurfaceLayer::Noise(noise));
        }
        self
    }

    pub fn with_fallback(mut self, fallback: Brush) -> Self {
        self.fallback = fallback;
        self
    }
}

impl BackdropBlur {
    pub const fn new(sigma: f32) -> Self {
        Self {
            sigma,
            edge_mode: BackdropEdgeMode::Mirror,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            sigma: self.sigma.max(0.0),
            edge_mode: self.edge_mode,
        }
    }

    pub const fn sigma(self) -> f32 {
        self.sigma
    }

    pub const fn edge_mode(self) -> BackdropEdgeMode {
        self.edge_mode
    }
}

impl Refraction {
    const MAX_DISPLACEMENT: f32 = 4.0;

    pub const fn new(displacement: f32, splay: f32, feather: f32, curve: f32) -> Self {
        Self {
            displacement,
            splay,
            feather,
            curve,
        }
    }

    pub fn clamped(self) -> Self {
        Self {
            displacement: self.displacement.clamp(0.0, Self::MAX_DISPLACEMENT),
            splay: self.splay.max(0.0),
            feather: self.feather.max(0.0),
            curve: self.curve.max(0.1),
        }
    }

    pub const fn displacement(self) -> f32 {
        self.displacement
    }

    pub const fn splay(self) -> f32 {
        self.splay
    }

    pub const fn feather(self) -> f32 {
        self.feather
    }

    pub const fn curve(self) -> f32 {
        self.curve
    }
}

impl Luminosity {
    pub const fn new(color: Color, opacity: f32) -> Self {
        Self { color, opacity }
    }

    pub fn clamped(self) -> Self {
        Self {
            color: self.color,
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }

    pub const fn color(self) -> Color {
        self.color
    }

    pub const fn opacity(self) -> f32 {
        self.opacity
    }
}

impl Noise {
    pub const fn new(opacity: f32) -> Self {
        Self { opacity }
    }

    pub fn clamped(self) -> Self {
        Self {
            opacity: self.opacity.clamp(0.0, 1.0),
        }
    }

    pub const fn opacity(self) -> f32 {
        self.opacity
    }
}
