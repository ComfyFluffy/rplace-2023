use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, ClearColorImageInfo, CommandBufferBeginInfo,
        CommandBufferLevel, CommandBufferUsage, RecordingCommandBuffer,
    },
    descriptor_set::{DescriptorSet, WriteDescriptorSet},
    device::Queue,
    format::{ClearColorValue, Format},
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    padded::Padded,
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    sync::GpuFuture,
};

use super::App;

pub struct UpdateTexturePipeline {
    compute_queue: Arc<Queue>,
    compute_pipeline: Arc<ComputePipeline>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    // descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    pixel_updates_buffer: Subbuffer<cs::PixelUpdates>,
    canvas_image: Arc<ImageView>,
    atomic_buffer: Subbuffer<[i32]>,

    descriptor_set: Arc<DescriptorSet>,

    should_clear_canvas: bool,
}

impl UpdateTexturePipeline {
    pub const MAX_PIXEL_UPDATES: u64 = 1024 * 1024 * 2;
    pub const WORKGROUP_SIZE: u64 = 256;

    pub fn new(app: &App, compute_queue: Arc<Queue>, canvas_size: (u32, u32)) -> Self {
        let context = &app.context;
        let allocator = context.memory_allocator();
        let device = context.device();
        let pixel_updates_buffer = Buffer::new_unsized(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            Self::MAX_PIXEL_UPDATES,
        )
        .unwrap();

        let atomic_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            (0..canvas_size.0 * canvas_size.1).map(|_| 0),
        )
        .unwrap();

        let canvas_size_buffer = Buffer::from_data(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            cs::CanvasSize {
                canvas_size: canvas_size.into(),
            },
        )
        .unwrap();

        let compute_pipeline = {
            let cs_main = cs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let stage = PipelineShaderStageCreateInfo::new(cs_main);
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();
            let pipeline = ComputePipeline::new(
                device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout),
            )
            .unwrap();
            pipeline
        };

        let canvas_image = ImageView::new_default(
            Image::new(
                allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::R8G8B8A8_UNORM,
                    extent: [canvas_size.0, canvas_size.1, 1],
                    usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED | ImageUsage::STORAGE,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let descriptor_set = {
            let desc_layout = compute_pipeline
                .layout()
                .set_layouts()
                .get(0)
                .unwrap()
                .clone();
            let descriptor_set = DescriptorSet::new(
                app.descriptor_set_allocator.clone(),
                desc_layout,
                [
                    WriteDescriptorSet::buffer(0, pixel_updates_buffer.clone()),
                    WriteDescriptorSet::image_view(1, canvas_image.clone()),
                    WriteDescriptorSet::buffer(2, atomic_buffer.clone()),
                    WriteDescriptorSet::buffer(3, canvas_size_buffer),
                ],
                [],
            )
            .unwrap();
            descriptor_set
        };

        Self {
            compute_queue,
            compute_pipeline,
            command_buffer_allocator: app.command_buffer_allocator.clone(),
            // descriptor_set_allocator,
            pixel_updates_buffer,
            atomic_buffer,
            canvas_image,

            descriptor_set,

            should_clear_canvas: true,
        }
    }

    pub fn canvas_image(&self) -> &Arc<ImageView> {
        &self.canvas_image
    }

    pub fn compute(
        &mut self,
        before: Box<dyn GpuFuture>,
        data: impl Iterator<Item = crate::data::PixelData> + ExactSizeIterator,
    ) -> Box<dyn GpuFuture> {
        let len = data.len() as u64;
        assert!(
            len % Self::WORKGROUP_SIZE == 0,
            "length of data must be a multiple of workgroup size ({}), but is {}",
            Self::WORKGROUP_SIZE,
            len
        );

        {
            let mut pixel_updates_buffer = self.pixel_updates_buffer.write().unwrap();
            for (i, pixel_data) in data.enumerate() {
                pixel_updates_buffer.pixel_updates[i] = Padded(pixel_data.into());
            }
        }

        self.atomic_buffer.write().unwrap().fill(0);

        let mut command_buffer_builder = RecordingCommandBuffer::new(
            self.command_buffer_allocator.clone(),
            self.compute_queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        if self.should_clear_canvas {
            command_buffer_builder
                .clear_color_image(ClearColorImageInfo {
                    clear_value: ClearColorValue::Float([1.0; 4]),
                    ..ClearColorImageInfo::image(self.canvas_image.image().clone())
                })
                .unwrap();
            self.should_clear_canvas = false;
        }

        unsafe {
            command_buffer_builder
                .bind_pipeline_compute(self.compute_pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Compute,
                    self.compute_pipeline.layout().clone(),
                    0,
                    self.descriptor_set.clone(),
                )
                .unwrap()
                .dispatch([len as u32 / 256, 1, 1])
                .unwrap();
        }

        let command_buffer = command_buffer_builder.end().unwrap();

        before
            .then_execute(self.compute_queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed()
    }
}

mod cs {
    use vulkano::padded::Padded;

    use crate::data;

    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/renderer/shaders/update_texture.comp"
    }

    impl From<data::Coordinate> for Coordinate {
        fn from(value: data::Coordinate) -> Self {
            fn convert((x, y): (i16, i16)) -> (u32, u32) {
                // In Coordinate the origin is in the center of the image.
                // We need to convert it to the top left corner.
                // Coordinate: min: (-1500, -1000), max: (1499, 999)
                // GpuCoordinate: min: (0, 0), max: (2999, 1999)
                ((x + 1500) as u32, (-y - 1 + 1000) as u32)
            }
            let (tag, data) = match value {
                data::Coordinate::Simple { x, y } => {
                    let (x, y) = convert((x, y));
                    (0, [x, y, 0, 0])
                }
                data::Coordinate::Rectangle { x1, y1, x2, y2 } => {
                    let (x1, y1) = convert((x1, y1));
                    let (x2, y2) = convert((x2, y2));
                    (1, [x1, y1, x2, y2])
                }
                data::Coordinate::Circle { x, y, radius } => {
                    let (x, y) = convert((x, y));
                    (2, [x, y, radius as u32, 0])
                }
            };
            Coordinate {
                tag: Padded(tag),
                data,
            }
        }
    }

    impl From<data::PixelData> for PixelData {
        fn from(pixel_data: crate::data::PixelData) -> Self {
            PixelData {
                miliseconds_since_first_pixel: Padded(pixel_data.miliseconds_since_first_pixel),
                coordinate: pixel_data.coordinate.into(),
                color: pixel_data.pixel_color.into(),
            }
        }
    }
}
