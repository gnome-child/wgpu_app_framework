use crate::paint::{self, Rect};

use super::geometry::PreparedFilter;
use super::{Target, TextureSource};

#[derive(Clone, Copy)]
pub(crate) enum FilterSource<'a> {
    Backdrop {
        texture: TextureSource<'a>,
        global_rect: Rect,
    },
    Local {
        texture: TextureSource<'a>,
        local_rect: Rect,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FilterSourceSpace {
    Backdrop,
    Local,
}

#[derive(Clone, Copy)]
pub(super) struct FilterSample<'a> {
    pub(super) texture: TextureSource<'a>,
    pub(super) rect: Rect,
    pub(super) space: FilterSourceSpace,
}

pub(super) struct FilterChainContext<'a> {
    target: Target,
    output: &'a wgpu::TextureView,
    prepared: PreparedFilter,
    current_source: TextureSource<'a>,
    current_rect: Rect,
    current_space: FilterSourceSpace,
}

impl<'a> FilterSource<'a> {
    fn initial_sample(self) -> FilterSample<'a> {
        match self {
            Self::Backdrop {
                texture,
                global_rect,
            } => FilterSample {
                texture,
                rect: global_rect,
                space: FilterSourceSpace::Backdrop,
            },
            Self::Local {
                texture,
                local_rect,
            } => FilterSample {
                texture,
                rect: local_rect,
                space: FilterSourceSpace::Local,
            },
        }
    }
}

impl<'a> FilterChainContext<'a> {
    pub(super) fn new(
        target: Target,
        output: &'a wgpu::TextureView,
        prepared: PreparedFilter,
        source: FilterSource<'a>,
    ) -> Self {
        let sample = source.initial_sample();

        Self {
            target,
            output,
            prepared,
            current_source: sample.texture,
            current_rect: sample.rect,
            current_space: sample.space,
        }
    }

    pub(super) fn target(&self) -> Target {
        self.target
    }

    pub(super) fn output(&self) -> &'a wgpu::TextureView {
        self.output
    }

    pub(super) fn base_prepared(&self) -> PreparedFilter {
        self.prepared
    }

    pub(super) fn current_sample(&self) -> FilterSample<'a> {
        FilterSample {
            texture: self.current_source,
            rect: self.current_rect,
            space: self.current_space,
        }
    }

    fn local_rect(&self) -> Rect {
        self.prepared.shape_rect
    }

    pub(super) fn local_intermediate<'b>(&self, texture: TextureSource<'b>) -> FilterSample<'b> {
        FilterSample {
            texture,
            rect: self.local_rect(),
            space: FilterSourceSpace::Local,
        }
    }

    pub(super) fn mark_output_as_current(&mut self) {
        self.current_source = TextureSource::for_target_view(
            self.output,
            self.target,
            paint::LayerSampling::PixelAligned,
        );
        self.current_rect = self.local_rect();
        self.current_space = FilterSourceSpace::Local;
    }
}
