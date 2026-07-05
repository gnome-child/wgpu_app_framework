use crate::geometry::{area, point};
use crate::text;

use super::Path;

pub(crate) use crate::widget::scroll::content_size_from_children;
pub use crate::widget::scroll::{
    ActiveAxes, Adjustment, Axes, Bars, Metrics, Part, Policy, Style, metrics,
    paint_metrics_chrome, viewport_rect, viewport_rect_for_axes,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroll {
    offset: point::Logical,
    axes: Axes,
    bars: Bars,
    style: Style,
}

pub trait Projections {
    fn metrics(&self, path: &Path) -> Option<Metrics>;

    fn visual_offset(&self, path: &Path) -> Option<point::Logical>;

    fn text_area(&self, path: &Path) -> Option<&TextAreaProjection>;
}

#[derive(Debug, Clone)]
pub struct TextAreaProjection {
    metrics: Metrics,
    layout: text::layout::TextFieldLayout,
    interaction_surfaces: Vec<text::layout::TextAreaSurface>,
    render_surfaces: Vec<text::layout::TextAreaSurface>,
}

impl TextAreaProjection {
    pub fn from_layout(metrics: Metrics, layout: text::layout::TextAreaPaintLayout) -> Self {
        let (layout, interaction_surfaces, render_surfaces) = layout.into_projection_parts();
        Self {
            metrics,
            layout,
            interaction_surfaces,
            render_surfaces,
        }
    }

    pub fn metrics(&self) -> Metrics {
        self.metrics
    }

    pub fn set_metrics(&mut self, metrics: Metrics) {
        self.metrics = metrics;
    }

    pub fn content_area(&self) -> area::Logical {
        self.layout.content_area()
    }

    pub fn interaction_surfaces(&self) -> &[text::layout::TextAreaSurface] {
        &self.interaction_surfaces
    }

    pub fn render_surfaces(&self) -> impl Iterator<Item = &text::layout::TextAreaSurface> {
        let viewport = self.metrics.viewport().area;
        self.render_surfaces
            .iter()
            .filter(move |surface| surface_intersects_viewport(surface, viewport))
    }

    pub fn observed_area(&self) -> text::edit::ObservedArea<'_> {
        text::edit::ObservedArea::new(
            self.metrics.viewport(),
            self.metrics.offset(),
            self.layout.content_area(),
            &self.interaction_surfaces,
        )
    }

    pub fn scroll_anchor(&self, area_model: &text::edit::Area) -> Option<text::edit::ScrollAnchor> {
        text::edit::View::scroll_anchor_for_text_area(
            area_model,
            self.observed_area(),
            &self.render_surfaces,
        )
    }

    #[cfg(test)]
    pub fn buffer(&self) -> std::rc::Rc<std::cell::RefCell<glyphon::Buffer>> {
        self.interaction_surfaces
            .first()
            .or_else(|| self.render_surfaces.first())
            .expect("text area projection should have at least one surface")
            .buffer()
    }

    pub(crate) fn translate_for_metrics(&mut self, old_metrics: Metrics, metrics: Metrics) -> bool {
        if !same_area(old_metrics.viewport().area, metrics.viewport().area)
            || !same_area(old_metrics.content_size(), metrics.content_size())
            || old_metrics.max_offset() != metrics.max_offset()
        {
            return false;
        }

        if !self.covers_metrics(old_metrics, metrics) {
            return false;
        }

        let old_offset = old_metrics.offset();
        let new_offset = metrics.offset();
        let viewport = metrics.viewport().area;
        let interaction_surfaces = self
            .interaction_surfaces
            .iter()
            .map(|surface| surface.translated_for_scroll(old_offset, new_offset, viewport))
            .collect::<Vec<_>>();
        if !interaction_surfaces.is_empty()
            && !surfaces_cover_viewport(&interaction_surfaces, viewport)
        {
            return false;
        }
        let render_surfaces = self
            .render_surfaces
            .iter()
            .map(|surface| surface.translated_for_scroll(old_offset, new_offset, viewport))
            .collect::<Vec<_>>();
        if !surfaces_cover_viewport(&render_surfaces, viewport) {
            return false;
        }
        self.layout = self
            .layout
            .translated_for_scroll(new_offset.x(), new_offset.y());
        self.interaction_surfaces = interaction_surfaces;
        self.render_surfaces = render_surfaces;
        self.metrics = metrics;
        true
    }

    fn covers_metrics(&self, old_metrics: Metrics, metrics: Metrics) -> bool {
        if !same_area(old_metrics.viewport().area, metrics.viewport().area)
            || !same_area(old_metrics.content_size(), metrics.content_size())
            || old_metrics.max_offset() != metrics.max_offset()
        {
            return false;
        }

        if self.interaction_surfaces.is_empty() {
            return surfaces_cover_viewport_after_scroll(
                &self.render_surfaces,
                old_metrics.offset(),
                metrics.offset(),
                metrics.viewport().area,
            );
        }

        surfaces_cover_viewport_after_scroll(
            &self.interaction_surfaces,
            old_metrics.offset(),
            metrics.offset(),
            metrics.viewport().area,
        )
    }
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset: point::logical(0.0, 0.0),
            axes: Axes::vertical(),
            bars: Bars::vertical(),
            style: Style::default(),
        }
    }

    pub fn offset(self) -> point::Logical {
        self.offset
    }

    pub fn axes(self) -> Axes {
        self.axes
    }

    pub fn bars(self) -> Bars {
        self.bars
    }

    pub fn style(self) -> Style {
        self.style
    }

    pub fn with_offset(mut self, offset: point::Logical) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_axes(mut self, axes: Axes) -> Self {
        self.axes = axes;
        self
    }

    pub fn with_bars(mut self, bars: Bars) -> Self {
        self.bars = bars;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

fn same_area(left: area::Logical, right: area::Logical) -> bool {
    left.width().to_bits() == right.width().to_bits()
        && left.height().to_bits() == right.height().to_bits()
}

fn surfaces_cover_viewport(
    surfaces: &[text::layout::TextAreaSurface],
    viewport: area::Logical,
) -> bool {
    if surfaces.is_empty() {
        return false;
    }

    let top = surfaces
        .iter()
        .map(text::layout::TextAreaSurface::y)
        .fold(f32::INFINITY, f32::min);
    let bottom = surfaces
        .iter()
        .map(|surface| surface.y() + surface.height().max(1.0))
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal = surfaces
        .iter()
        .all(|surface| surface.x() <= 0.0 && surface.x() + surface.width() >= viewport.width());

    top <= 0.0 && bottom >= viewport.height() && horizontal
}

fn surfaces_cover_viewport_after_scroll(
    surfaces: &[text::layout::TextAreaSurface],
    old_scroll: point::Logical,
    new_scroll: point::Logical,
    viewport: area::Logical,
) -> bool {
    if surfaces.is_empty() {
        return false;
    }

    let dx = old_scroll.x() - new_scroll.x();
    let dy = old_scroll.y() - new_scroll.y();
    let top = surfaces
        .iter()
        .map(|surface| surface.y() + dy)
        .fold(f32::INFINITY, f32::min);
    let bottom = surfaces
        .iter()
        .map(|surface| surface.y() + dy + surface.height().max(1.0))
        .fold(f32::NEG_INFINITY, f32::max);
    let horizontal = surfaces.iter().all(|surface| {
        let x = surface.x() + dx;
        x <= 0.0 && x + surface.width() >= viewport.width()
    });

    top <= 0.0 && bottom >= viewport.height() && horizontal
}

fn surface_intersects_viewport(
    surface: &text::layout::TextAreaSurface,
    viewport: area::Logical,
) -> bool {
    let left = surface.x();
    let top = surface.y();
    let right = left + surface.width();
    let bottom = top + surface.height().max(1.0);

    right > 0.0 && bottom > 0.0 && left < viewport.width() && top < viewport.height()
}
