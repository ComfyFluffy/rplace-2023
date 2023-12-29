use std::{sync::Arc, time::Instant};

use log::{debug, warn};
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    pipeline::graphics::subpass::PipelineRenderingCreateInfo,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
};

use crate::parse::GzippedBinPixelDataReader;

use self::{draw_quad::DrawQuadPipeline, update_texture::UpdateTexturePipeline};

mod draw_quad;
pub mod update_texture;

// Transform State to Vulkano App.
pub struct App {
    context: VulkanoContext,
    windows: VulkanoWindows,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl App {
    pub fn new() -> Self {
        let mut config = VulkanoConfig::default();
        config.device_features.dynamic_rendering = true;

        // let api_version = device.physical_device().api_version();
        // info!("API Version: {:?}", api_version);
        // if api_version < Version::V1_3 {
        config.device_extensions.khr_dynamic_rendering = true;
        // }
        let context = VulkanoContext::new(config);
        let windows = VulkanoWindows::default();

        let device = context.device();

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            context,
            windows,
            command_buffer_allocator,
            descriptor_set_allocator,
            // data_reader,
            // playback_speed,
            // update_texture_pipeline,
            // draw_quad_pipeline,
        }
    }

    pub fn run(&mut self, mut data_reader: GzippedBinPixelDataReader, playback_speed: u32) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let window_id = self.windows.create_window(
            &event_loop,
            &self.context,
            &WindowDescriptor {
                width: 1280.0,
                height: 720.0,
                title: "r/place 2023 Player".to_string(),
                ..Default::default()
            },
            |_| {},
        );

        let queue = self.context.graphics_queue();

        let mut update_texture_pipeline =
            UpdateTexturePipeline::new(self, queue.clone(), (3000, 2000));

        let mut draw_quad_pipeline = DrawQuadPipeline::new(
            self,
            queue.clone(),
            1280.0 / 720.0,
            update_texture_pipeline.canvas_image().clone(),
            PipelineRenderingCreateInfo {
                color_attachment_formats: vec![Some(
                    self.windows
                        .get_renderer(window_id)
                        .unwrap()
                        .swapchain_format(),
                )],
                ..Default::default()
            },
        );

        let render_start = Instant::now();

        let mut buffer = Vec::new();

        let mut redraw = |renderer: &mut VulkanoWindowRenderer,
                          draw_quad_pipeline: &DrawQuadPipeline| {
            let elapsed_ms = render_start.elapsed().as_millis() as u32 * playback_speed;
            debug!("Render started at {}ms", elapsed_ms);

            for pixel_data in &mut data_reader {
                let pixel_data = pixel_data.unwrap();
                buffer.push(pixel_data.clone());
                if pixel_data.miliseconds_since_first_pixel > elapsed_ms {
                    break;
                }
            }

            let data = buffer
                .drain(
                    ..(buffer.len() / UpdateTexturePipeline::WORKGROUP_SIZE as usize
                        * UpdateTexturePipeline::WORKGROUP_SIZE as usize)
                        .min(UpdateTexturePipeline::MAX_PIXEL_UPDATES as usize),
                )
                .collect::<Vec<_>>();

            debug!("Updating {} pixels", data.len());

            let before_pipeline = match renderer.acquire() {
                Ok(before) => before,
                Err(err) => {
                    warn!("Error while drawing: {:?}", err);
                    return;
                }
            };

            let after_compute = update_texture_pipeline.compute(before_pipeline, data.into_iter());

            let after_draw =
                draw_quad_pipeline.draw(after_compute, renderer.swapchain_image_view());

            renderer.present(after_draw, true);
        };

        event_loop
            .run(move |event, elwt| {
                let renderer = self.windows.get_renderer_mut(window_id).unwrap();
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        WindowEvent::Resized(size) => {
                            draw_quad_pipeline
                                .update_window_aspect_ratio(size.width as f32 / size.height as f32);
                            renderer.resize();
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            renderer.resize();
                        }
                        WindowEvent::RedrawRequested => {
                            redraw(renderer, &draw_quad_pipeline);
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
                        self.windows.get_window(window_id).unwrap().request_redraw();
                    }
                    _ => {}
                }
            })
            .unwrap();
    }
}
