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
    backend: Backend,
    force_fallback_adapter: bool,
    presentation_system: String,
    windows_popup_composition_supported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Backends(wgpu::Backends);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Backend(wgpu::Backend);

#[derive(Debug, Clone)]
pub struct Options {
    pub(in crate::render) device_label: &'static str,
    pub(in crate::render) backends: wgpu::Backends,
    pub(in crate::render) power_preference: wgpu::PowerPreference,
    pub(in crate::render) force_fallback_adapter: bool,
    pub(in crate::render) required_features: wgpu::Features,
    pub(in crate::render) required_limits: wgpu::Limits,
}

impl Backends {
    pub(crate) fn from_env() -> Option<Self> {
        wgpu::Backends::from_env().map(Self)
    }

    pub(crate) const fn all() -> Self {
        Self(wgpu::Backends::all())
    }

    #[cfg(target_os = "windows")]
    pub(crate) const fn dx12() -> Self {
        Self(wgpu::Backends::DX12)
    }

    #[cfg(test)]
    pub(crate) const fn vulkan() -> Self {
        Self(wgpu::Backends::VULKAN)
    }

    #[cfg(test)]
    pub(crate) const fn contains(self, other: Self) -> bool {
        self.0.contains(other.0)
    }
}

impl Backend {
    pub(crate) fn is_dx12(self) -> bool {
        self.0 == wgpu::Backend::Dx12
    }
}

impl Options {
    pub(crate) fn native(backends: Backends) -> Self {
        Self {
            device_label: "wgpu_l3 device",
            backends: backends.0,
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: diagnostic_force_fallback_adapter(
                std::env::var("WGPU_L3_FORCE_FALLBACK_ADAPTER")
                    .ok()
                    .as_deref(),
            ),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }
    }
}

impl Context {
    pub async fn new(options: Options) -> render::Result<Self> {
        let backends = options.backends.with_env();
        log::debug!(
            "creating wgpu context: backends={:?}, power_preference={:?}, fallback={}",
            backends,
            options.power_preference,
            options.force_fallback_adapter
        );
        let backend_options = default_backend_options().with_env();
        #[cfg(target_os = "windows")]
        let dx12_presentation_system = backend_options.dx12.presentation_system;
        #[cfg(target_os = "windows")]
        let presentation_system = format!("{dx12_presentation_system:?}");
        #[cfg(not(target_os = "windows"))]
        let presentation_system = "platform-default".to_owned();
        #[cfg(target_os = "windows")]
        log::info!(
            target: "wgpu_l3::native_popup",
            "using Windows DX12 presentation system {:?}",
            dx12_presentation_system
        );
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options,
            display: None,
        });

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: None,
                force_fallback_adapter: options.force_fallback_adapter,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(error) => {
                log::error!("failed to request wgpu adapter: {error}");
                return Err(Error::from(error).into());
            }
        };
        let adapter_info = adapter.get_info();
        let adapter_backend = adapter_info.backend;
        if options.force_fallback_adapter && adapter_info.device_type != wgpu::DeviceType::Cpu {
            log::error!(
                target: "wgpu_l3::renderer_receipt",
                "fallback adapter was requested but selected adapter is not CPU-class: {adapter_info:?}"
            );
        }
        #[cfg(target_os = "windows")]
        let windows_popup_composition_supported = adapter_backend == wgpu::Backend::Dx12
            && dx12_presentation_system == wgpu::Dx12SwapchainKind::DxgiFromVisual;
        #[cfg(not(target_os = "windows"))]
        let windows_popup_composition_supported = false;
        log::debug!("selected wgpu adapter: {:?}", adapter_info);
        #[cfg(target_os = "windows")]
        log::info!(
            target: "wgpu_l3::native_popup",
            "selected Windows graphics backend {:?} with DX12 presentation system {:?}",
            adapter_info.backend,
            dx12_presentation_system
        );

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some(options.device_label),
                required_features: options.required_features,
                experimental_features: Default::default(),
                required_limits: options.required_limits,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
        {
            Ok(device) => device,
            Err(error) => {
                log::error!("failed to request wgpu device: {error}");
                return Err(Error::from(error).into());
            }
        };
        log::debug!("created wgpu device: {}", options.device_label);

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            backend: Backend(adapter_backend),
            force_fallback_adapter: options.force_fallback_adapter,
            presentation_system,
            windows_popup_composition_supported,
        })
    }

    pub(in crate::render) fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub(in crate::render) fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub(in crate::render) fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub(crate) fn backend(&self) -> Backend {
        self.backend
    }

    pub(crate) fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    pub(crate) fn force_fallback_adapter(&self) -> bool {
        self.force_fallback_adapter
    }

    pub(crate) fn presentation_system(&self) -> &str {
        &self.presentation_system
    }

    pub(crate) fn windows_popup_composition_supported(&self) -> bool {
        self.windows_popup_composition_supported
    }

    pub(in crate::render) fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

fn default_backend_options() -> wgpu::BackendOptions {
    let mut options = wgpu::BackendOptions::default();
    configure_default_backend_options(&mut options);
    options
}

fn diagnostic_force_fallback_adapter(value: Option<&str>) -> bool {
    value == Some("1")
}

#[cfg(target_os = "windows")]
fn configure_default_backend_options(options: &mut wgpu::BackendOptions) {
    options.dx12.presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;
}

#[cfg(not(target_os = "windows"))]
fn configure_default_backend_options(_options: &mut wgpu::BackendOptions) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_defaults_to_dx12_visual_presentation() {
        assert_eq!(
            default_backend_options().dx12.presentation_system,
            wgpu::Dx12SwapchainKind::DxgiFromVisual
        );
    }

    #[test]
    fn fallback_adapter_diagnostic_requires_explicit_one() {
        assert!(diagnostic_force_fallback_adapter(Some("1")));
        for value in [None, Some(""), Some("0"), Some("true"), Some("yes")] {
            assert!(!diagnostic_force_fallback_adapter(value));
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    #[ignore = "requires the Windows DX12 fallback adapter"]
    fn dx12_fallback_adapter_reports_cpu_class() {
        let mut options = Options::native(Backends::dx12());
        options.force_fallback_adapter = true;

        let context = pollster::block_on(Context::new(options))
            .expect("Windows should provide a DX12 fallback adapter");
        let info = context.adapter_info();

        assert_eq!(info.backend, wgpu::Backend::Dx12);
        assert_eq!(info.device_type, wgpu::DeviceType::Cpu);
        assert!(context.force_fallback_adapter());
    }
}
