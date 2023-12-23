use std::sync::Arc;

use vulkano::{buffer::Subbuffer, pipeline::ComputePipeline};
use wgpu::util::DeviceExt;

use super::data::GpuPixelData;

pub const WORKGROUP_SIZE: u32 = 256;

pub struct UpdateTexturePipeline {
    pub compute_pipeline: Arc<ComputePipeline>,
    pub bind_group: wgpu::BindGroup,
    pub pixel_updates_buffer: Subbuffer<[GpuPixelData]>,
    pub atomic_buffer: Subbuffer<[u32]>,
    pub canvas_size: (u32, u32),
    atomic_zeros: Vec<u32>,
}

impl UpdateTexturePipeline {
    pub fn new(
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        canvas_size: (u32, u32),
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Update Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                    },
                    visibility: wgpu::ShaderStages::COMPUTE,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        });

        let pixel_updates_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Update Texture Buffer"),
            mapped_at_creation: false,
            size: 128 * 1024 * 1024,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let atomic_zeros = vec![0u32; canvas_size.0 as usize * canvas_size.1 as usize];

        let atomic_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Update Texture Atomic Buffer"),
            contents: bytemuck::cast_slice(atomic_zeros.as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let canvas_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Canvas Uniform Buffer"),
            contents: bytemuck::cast_slice(&[canvas_size.0, canvas_size.1]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Update Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pixel_updates_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: atomic_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: canvas_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Update Texture Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Update Texture Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &device
                .create_shader_module(wgpu::include_wgsl!("shaders/update_texture.wgsl")),
            entry_point: "main",
        });

        Self {
            compute_pipeline,
            bind_group,
            pixel_updates_buffer,
            atomic_buffer,
            canvas_size,
            atomic_zeros,
        }
    }

    pub fn begin_compute_pass(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        data: &[Std140GpuPixelData],
    ) {
        if data.is_empty() {
            return;
        }
        queue.write_buffer(&self.pixel_updates_buffer, 0, bytemuck::cast_slice(data));

        // Clear atomic buffer
        queue.write_buffer(
            &self.atomic_buffer,
            0,
            bytemuck::cast_slice(self.atomic_zeros.as_slice()),
        );
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Update Texture Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(data.len() as u32 / WORKGROUP_SIZE, 1, 1);
    }
}

mod cs {
    vulkano::shader! {
        ty: "compute",
        path: "shaders/update_texture.comp"
    }
}
