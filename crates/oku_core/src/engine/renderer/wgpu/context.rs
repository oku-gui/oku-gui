use std::sync::Arc;
use tokio::sync::RwLock;
use wgpu::{CompositeAlphaMode, PresentMode};
use glyphon::{Cache, TextAtlas, TextRenderer, Viewport};
use crate::engine::renderer::color::Color;
use crate::engine::renderer::wgpu::texture::Texture;
use crate::platform::resource_manager::ResourceManager;

pub struct Context<'a> {
    pub(crate) device: wgpu::Device,
    pub(crate) resource_manager: Arc<RwLock<ResourceManager>>,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) surface_clear_color: Color,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) default_texture: Texture,
    pub(crate) is_srgba_format: bool,
    pub glyphon_cache: Cache,
    pub glyphon_viewport: Viewport,
    pub glyphon_atlas: TextAtlas,
    pub glyphon_text_renderer: TextRenderer,
}

pub async fn request_adapter(instance: wgpu::Instance, surface: &wgpu::Surface<'_>) -> wgpu::Adapter {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request an adapter, cannot request GPU access without an adapter.");
    adapter
}

pub async fn request_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: wgpu::Label::from("oku_wgpu_renderer"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None, // Trace path
        )
        .await
        .expect("Failed to request a GPU!");
    (device, queue)
}

pub fn create_surface_config(
    surface: &wgpu::Surface<'_>,
    width: u32,
    height: u32,
    _device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);

    // Require that we use a surface with a srgb format.
    surface_caps.formats.iter().copied().find(|f| f.is_srgb()).expect("Failed to find a SRGB surface!");

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8Unorm,
        width,
        height,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 0,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    }
}