use vulkano::buffer::BufferContents;

use crate::data::Coordinate;

#[derive(Debug, BufferContents)]
#[repr(C)]
pub struct GpuCoordinate {
    pub tag: u32,
    pub data: [u32; 4],
}

impl From<Coordinate> for GpuCoordinate {
    fn from(value: Coordinate) -> Self {
        fn convert((x, y): (i16, i16)) -> (u32, u32) {
            // In Coordinate the origin is in the center of the image.
            // We need to convert it to the top left corner.
            // Coordinate: min: (-1500, -1000), max: (1499, 999)
            // GpuCoordinate: min: (0, 0), max: (2999, 1999)
            ((x + 1500) as u32, (y + 1000) as u32)
        }
        let (tag, data) = match value {
            Coordinate::Simple { x, y } => {
                let (x, y) = convert((x, y));
                (0, [x, y, 0, 0])
            }
            Coordinate::Rectangle { x1, y1, x2, y2 } => {
                let (x1, y1) = convert((x1, y1));
                let (x2, y2) = convert((x2, y2));
                (1, [x1, y1, x2, y2])
            }
            Coordinate::Circle { x, y, radius } => {
                let (x, y) = convert((x, y));
                (2, [x, y, radius as u32, 0])
            }
        };
        GpuCoordinate { tag, data }
    }
}

#[derive(Debug, BufferContents)]
#[repr(C)]
pub struct GpuPixelData {
    pub miliseconds_since_first_pixel: u32,
    pub coordinate: GpuCoordinate,
    pub color: [u32; 3],
}

impl From<crate::data::PixelData> for GpuPixelData {
    fn from(pixel_data: crate::data::PixelData) -> Self {
        GpuPixelData {
            miliseconds_since_first_pixel: pixel_data.miliseconds_since_first_pixel,
            coordinate: pixel_data.coordinate.into(),
            color: pixel_data.pixel_color.into(),
        }
    }
}
