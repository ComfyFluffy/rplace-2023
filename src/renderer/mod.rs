use log::info;
use wgpu::TextureUsages;
use winit::window::Window;

use self::{
    data::Std140GpuPixelData, presentation::PresentationPipeline,
    update_texture::UpdateTexturePipeline,
};

pub mod data;
mod presentation;
pub mod update_texture;

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    window: Window,

    update_texture_pipeline: UpdateTexturePipeline,
    // offscreen_pipeline: OffscreenPipeline,
    presentation_pipeline: PresentationPipeline,
    // last_frame_time: Option<std::time::Instant>,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface_from_raw(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    ..Default::default()
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        println!("surface_caps: {:?}", surface_caps);

        // let surface_format = surface_caps
        //     .formats
        //     .iter()
        //     .copied()
        //     .find(|f| !f.is_srgb())
        //     .expect("No suitable surface format.");
        let surface_format = wgpu::TextureFormat::Rgba16Float;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_size = wgpu::Extent3d {
            width: 3000,
            height: 2000,
            depth_or_array_layers: 1,
        };

        // Create a 3000x2000 texture for offscreen rendering
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rplace Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        clear_texture_to_color(&device, &queue, &texture_view, wgpu::Color::WHITE);

        let update_texture_pipeline =
            UpdateTexturePipeline::new(&device, &texture_view, (3000, 2000));

        let presentation_pipeline = PresentationPipeline::new(
            &device,
            &config,
            size.width as f32 / size.height as f32,
            texture_size.width as f32 / texture_size.height as f32,
            &texture_view,
        );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            update_texture_pipeline,
            presentation_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            info!("Resizing to {:?}", new_size);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.presentation_pipeline.update_window_size(
                &self.device,
                self.size.width as f32 / self.size.height as f32,
                3000.0 / 2000.0,
            )
        }
    }

    pub fn render(&mut self, data: &[Std140GpuPixelData]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.update_texture_pipeline
            .begin_compute_pass(&self.queue, &mut encoder, data);
        self.presentation_pipeline
            .begin_render_pass(&mut encoder, &view);

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn clear_texture_to_color(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_view: &wgpu::TextureView,
    color: wgpu::Color,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Clear Texture Encoder"),
    });
    let render_pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(color),
                store: wgpu::StoreOp::Store,
            },
            view: texture_view,
        })],
        depth_stencil_attachment: None,
        label: Some("Clear Pass"),
        occlusion_query_set: None,
        timestamp_writes: None,
    };

    encoder.begin_render_pass(&render_pass_desc);
    queue.submit(Some(encoder.finish()));
}
