use std::mem::size_of;

use wgpu::{util::DeviceExt, VertexAttribute};

pub struct PresentationPipeline {
    pub bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2], // x, y
    uv: [f32; 2],       // u, v
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[VertexAttribute] =
            &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

impl PresentationPipeline {
    fn fit_quad(window_aspect_ratio: f32, texture_aspect_ratio: f32) -> [Vertex; 4] {
        let scale_x;
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
            Vertex {
                position: [-scale_x, scale_y],
                uv: [0.0, 0.0],
            },
            // Top right
            Vertex {
                position: [scale_x, scale_y],
                uv: [1.0, 0.0],
            },
            // Bottom left
            Vertex {
                position: [-scale_x, -scale_y],
                uv: [0.0, 1.0],
            },
            // Bottom right
            Vertex {
                position: [scale_x, -scale_y],
                uv: [1.0, 1.0],
            },
        ]
    }

    fn vertex_buffer_from_vertices(device: &wgpu::Device, vertices: &[Vertex]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Presentation Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        window_aspect_ratio: f32,
        texture_aspect_ratio: f32,
        texture_view: &wgpu::TextureView,
    ) -> Self {
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Presentation Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Presentation Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&nearest_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/presentation.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Presentation Render Pipeline Layout"),
                bind_group_layouts: &[
                    &bind_group_layout, // 0
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Presentation Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(), // 0
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertices = Self::fit_quad(window_aspect_ratio, texture_aspect_ratio);
        let vertex_buffer = Self::vertex_buffer_from_vertices(device, &vertices);

        Self {
            bind_group,
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn update_window_size(
        &mut self,
        device: &wgpu::Device,
        window_aspect_ratio: f32,
        texture_aspect_ratio: f32,
    ) {
        let vertices = Self::fit_quad(window_aspect_ratio, texture_aspect_ratio);
        self.vertex_buffer = Self::vertex_buffer_from_vertices(device, &vertices);
    }

    pub fn begin_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Presentation Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 2.0,
                        g: 2.0,
                        b: 2.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fit_quad() {
        let texture_aspect_ratio = 3000.0 / 2000.0;
        let window_aspect_ratio = 1920.0 / 1080.0;
        let vertices =
            super::PresentationPipeline::fit_quad(window_aspect_ratio, texture_aspect_ratio);
        assert_eq!(vertices[0].position, [-0.84375, 1.0]);
    }
}
