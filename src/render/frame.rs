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

    pub fn texture(&self) -> &wgpu::Texture {
        &self.surface_texture.texture
    }
}
