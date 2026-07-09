use crate::paint::{self, Rect};
use crate::render::silhouette::PreparedSilhouette;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PreparedFilter {
    pub(super) raster_rect: Rect,
    pub(super) shape_rect: Rect,
    pub(super) rounding: [f32; 4],
    pub(super) blur_amount: f32,
    pub(super) blur_sigma_px: f32,
    pub(super) blur_radius_px: f32,
}

impl PreparedFilter {
    pub(super) fn with_blur(mut self, blur_amount: f32, scale_factor: f32) -> Self {
        let blur_amount = blur_amount.clamp(0.0, 1.0);

        self.blur_amount = blur_amount;
        self.blur_radius_px = blur_radius_px(blur_amount, scale_factor);
        self.blur_sigma_px = self.blur_radius_px * 0.42;
        self
    }

    pub(super) fn with_blur_sigma(mut self, blur_sigma: f32, scale_factor: f32) -> Self {
        let blur_sigma = blur_sigma.max(0.0);

        self.blur_amount = 0.0;
        self.blur_sigma_px = blur_sigma_px(blur_sigma, scale_factor);
        self.blur_radius_px = blur_kernel_radius_px(blur_sigma, scale_factor);
        self
    }
}

pub(super) fn prepare_filter(rect: Rect, scale_factor: f32) -> Option<PreparedFilter> {
    prepare_clip(rect, scale_factor)
}

pub(super) fn prepare_clip(rect: Rect, scale_factor: f32) -> Option<PreparedFilter> {
    let silhouette = PreparedSilhouette::for_filter_rect(rect, scale_factor)?;

    Some(PreparedFilter {
        raster_rect: silhouette.raster_rect,
        shape_rect: silhouette.shape_rect,
        rounding: silhouette.rounding,
        blur_amount: 0.0,
        blur_sigma_px: 0.0,
        blur_radius_px: 0.0,
    })
}

pub(super) fn source_rect_for_prepared_destination(
    destination: Rect,
    prepared: PreparedFilter,
    source: Rect,
) -> Rect {
    let origin_delta = paint::point::logical(
        prepared.shape_rect.origin.x() - destination.origin.x(),
        prepared.shape_rect.origin.y() - destination.origin.y(),
    );

    Rect::new(
        paint::point::logical(
            source.origin.x() + origin_delta.x(),
            source.origin.y() + origin_delta.y(),
        ),
        prepared.shape_rect.area,
    )
}

pub(super) fn blur_radius_px(amount: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_radius_px(amount, scale_factor)
}

pub(super) fn blur_sigma_px(sigma: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_sigma_px(sigma, scale_factor)
}

pub(super) fn blur_kernel_radius_px(sigma: f32, scale_factor: f32) -> f32 {
    paint::filter_blur_kernel_radius_px(sigma, scale_factor)
}

#[cfg(test)]
pub(crate) fn prepared_filter_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> Option<PreparedSilhouette> {
    let prepared = prepare_filter(rect, scale_factor)?;

    Some(
        PreparedSilhouette::from_parts(prepared.shape_rect, prepared.raster_rect)
            .with_rounding(prepared.rounding),
    )
}

#[cfg(test)]
pub(crate) fn prepared_clip_silhouette_for_test(
    rect: Rect,
    scale_factor: f32,
) -> Option<PreparedSilhouette> {
    let prepared = prepare_clip(rect, scale_factor)?;

    Some(
        PreparedSilhouette::from_parts(prepared.shape_rect, prepared.raster_rect)
            .with_rounding(prepared.rounding),
    )
}
