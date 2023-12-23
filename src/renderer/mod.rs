use std::sync::Arc;

use log::info;
use vulkano::{
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::AllocationCreateInfo,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};
use wgpu::TextureUsages;
use winit::{event_loop::EventLoop, window::Window};

use self::{
    data::Std140GpuPixelData, presentation::PresentationPipeline,
    update_texture::UpdateTexturePipeline,
};

pub mod data;
mod presentation;
pub mod update_texture;

pub struct State {
    surface: wgpu::Surface,
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

// Transform State to Vulkano App.
pub struct App {
    context: VulkanoContext,
    windows: VulkanoWindows,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanoContext::new(VulkanoConfig::default());
        let windows = VulkanoWindows::default();

        let memory_allocator = context.memory_allocator();

        let canvas_image = ImageView::new_default(
            Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    extent: [3000, 2000, 1],
                    format: Format::R8G8B8A8_UNORM,
                    usage: ImageUsage::STORAGE,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        Self { context, windows }
    }

    pub fn run(&self, event_loop: EventLoop<()>) {
        let id = self.windows.create_window(
            &event_loop,
            &self.context,
            &WindowDescriptor {
                title: "r/place 2023".to_string(),
                ..Default::default()
            },
            |_| {},
        );
    }
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

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
