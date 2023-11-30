/// Clear the texture to a solid color.
pub struct ClearPipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub clear_color: wgpu::Color,
    pub bind_group: wgpu::BindGroup,
}

impl ClearPipeline {}
