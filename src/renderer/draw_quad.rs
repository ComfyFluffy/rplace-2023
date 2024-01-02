use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferBeginInfo, CommandBufferLevel,
        CommandBufferUsage, RecordingCommandBuffer, RenderingAttachmentInfo, RenderingInfo,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    device::Queue,
    image::{
        sampler::{Filter, Sampler, SamplerCreateInfo},
        view::ImageView,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::PipelineRenderingCreateInfo,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{AttachmentLoadOp, AttachmentStoreOp},
    sync::GpuFuture,
};

use super::App;

pub struct DrawQuadPipeline {
    gfx_queue: Arc<Queue>,
    gfx_pipeline: Arc<GraphicsPipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    // descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    vertex_buffer: Subbuffer<[TexturedVertex]>,
    src_image: Arc<ImageView>,

    descriptor_set: Arc<DescriptorSet>,
}

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
struct TexturedVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2], // x, y
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2], // u, v
}

impl DrawQuadPipeline {
    fn fit_quad(window_aspect_ratio: f32, texture_aspect_ratio: f32) -> [TexturedVertex; 4] {
        let scale_x: f32;
        let scale_y;

        if texture_aspect_ratio > window_aspect_ratio {
            // Window is taller than the texture, scale based on width
            scale_x = 1.0;
            scale_y = window_aspect_ratio / texture_aspect_ratio;
        } else {
            // Window is wider than the texture, scale based on height
            scale_x = texture_aspect_ratio / window_aspect_ratio;
            scale_y = 1.0;
        }

        [
            // Top left
            TexturedVertex {
                position: [-scale_x, scale_y],
                uv: [0.0, 0.0],
            },
            // Top right
            TexturedVertex {
                position: [scale_x, scale_y],
                uv: [1.0, 0.0],
            },
            // Bottom left
            TexturedVertex {
                position: [-scale_x, -scale_y],
                uv: [0.0, 1.0],
            },
            // Bottom right
            TexturedVertex {
                position: [scale_x, -scale_y],
                uv: [1.0, 1.0],
            },
        ]
    }

    pub fn new(
        app: &App,
        gfx_queue: Arc<Queue>,
        window_aspect_ratio: f32,
        src_image: Arc<ImageView>,
        rendering_info: PipelineRenderingCreateInfo,
    ) -> Self {
        let context = &app.context;
        let memory_allocator = context.memory_allocator().clone();

        let src_image_extent = src_image.image().extent();
        let src_image_aspect_ratio = src_image_extent[0] as f32 / src_image_extent[1] as f32;
        let vertices = Self::fit_quad(window_aspect_ratio, src_image_aspect_ratio);

        let vertex_buffer = {
            Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vertices,
            )
            .unwrap()
        };

        let gfx_pipeline = {
            let device = gfx_queue.device();
            let vs = vs::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .expect("shader entry point not found");
            let fs = fs::load(device.clone())
                .expect("failed to create shader module")
                .entry_point("main")
                .expect("shader entry point not found");
            let vertex_input_state = TexturedVertex::per_vertex()
                .definition(&vs.info().input_interface)
                .unwrap();
            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();

            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState {
                        topology: PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    }),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        rendering_info.color_attachment_formats.len() as u32,
                        ColorBlendAttachmentState::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(rendering_info.into()),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };

        let sampler = Sampler::new(
            context.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                ..Default::default()
            },
        )
        .unwrap();

        let descriptor_set = {
            let desc_layout = gfx_pipeline.layout().set_layouts()[0].clone();
            let descriptor_set = DescriptorSet::new(
                app.descriptor_set_allocator.clone(),
                desc_layout,
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    src_image.clone(),
                    sampler,
                )],
                [],
            )
            .unwrap();
            descriptor_set
        };

        Self {
            src_image,
            gfx_pipeline,
            gfx_queue,
            vertex_buffer,
            command_buffer_allocator: app.command_buffer_allocator.clone(),
            descriptor_set,
        }
    }

    pub fn update_window_aspect_ratio(&mut self, window_aspect_ratio: f32) {
        let src_image_extent = self.src_image.image().extent();
        let src_image_aspect_ratio = src_image_extent[0] as f32 / src_image_extent[1] as f32;
        let vertices = Self::fit_quad(window_aspect_ratio, src_image_aspect_ratio);

        self.vertex_buffer
            .write()
            .unwrap()
            .copy_from_slice(&vertices);
    }

    pub fn draw(
        &self,
        before: Box<dyn GpuFuture>,
        dst_image: Arc<ImageView>,
    ) -> Box<dyn GpuFuture> {
        let mut builder = RecordingCommandBuffer::new(
            self.command_buffer_allocator.clone(),
            self.gfx_queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        let viewport = {
            let extent = dst_image.image().extent();
            Viewport {
                extent: [extent[0] as f32, extent[1] as f32],
                ..Default::default()
            }
        };

        builder
            .begin_rendering(RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo {
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                    clear_value: Some([0.1, 0.1, 0.1, 1.0].into()),
                    ..RenderingAttachmentInfo::image_view(dst_image)
                })],
                ..Default::default()
            })
            .unwrap()
            .set_viewport(0, [viewport].into_iter().collect())
            .unwrap()
            .bind_pipeline_graphics(self.gfx_pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap()
            .bind_descriptor_sets(
                self.gfx_pipeline.bind_point(),
                self.gfx_pipeline.layout().clone(),
                0,
                self.descriptor_set.clone(),
            )
            .unwrap();

        unsafe {
            builder.draw(4, 1, 0, 0).unwrap();
        }

        builder.end_rendering().unwrap();

        let command_buffer = builder.end().unwrap();

        before
            .then_execute(self.gfx_queue.clone(), command_buffer)
            .unwrap()
            .boxed()
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/renderer/shaders/draw_quad.vert"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/renderer/shaders/draw_quad.frag"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fit_quad() {
        let texture_aspect_ratio = 3000.0 / 2000.0;
        let window_aspect_ratio = 1920.0 / 1080.0;
        let vertices = super::DrawQuadPipeline::fit_quad(window_aspect_ratio, texture_aspect_ratio);
        assert_eq!(vertices[0].position, [-0.84375, 1.0]);
    }
}
