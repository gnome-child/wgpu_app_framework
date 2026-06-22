use std::time::Duration;

pub struct Frame {
    surface_texture: wgpu::SurfaceTexture,
}

pub enum Outcome {
    Acquired(Frame),
    Skipped(Reason),
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Presented,
    Skipped(Reason),
}

#[derive(Debug, Clone, Copy)]
pub enum Reason {
    Outdated,
    Timeout,
    Occluded,
    Validation,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SurfaceTimings {
    pub acquire: Duration,
    pub encode_submit: Duration,
    pub total: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceReport {
    pub status: Status,
    pub timings: SurfaceTimings,
}

impl Frame {
    pub fn new(surface_texture: wgpu::SurfaceTexture) -> Self {
        Self { surface_texture }
    }

    pub fn present(self) {
        self.surface_texture.present();
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
