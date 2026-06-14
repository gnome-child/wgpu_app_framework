use thiserror::Error;

use crate::render;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RequestAdapter(#[from] wgpu::RequestAdapterError),

    #[error(transparent)]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct Context {
    device: wgpu::Device,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    queue: wgpu::Queue,
}

pub struct Options {
    pub device_label: &'static str,
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub force_fallback_adapter: bool,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
}

impl Context {
    pub async fn new(options: Options) -> render::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: options.backends,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: None,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
            .map_err(Error::from)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some(options.device_label),
                required_features: options.required_features,
                experimental_features: Default::default(),
                required_limits: options.required_limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(Error::from)?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}
