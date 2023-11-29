use bytemuck::{Pod, Zeroable};

use crate::data::Coordinate;

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct GpuCoordinate {
    pub tag: u32,
    pub data: [u32; 4],
}

impl From<Coordinate> for GpuCoordinate {
    fn from(coordinate: Coordinate) -> Self {
        fn convert((x, y): (i16, i16)) -> (u32, u32) {
            // In Coordinate the origin is in the center of the image.
            // We need to convert it to the top left corner.
            // Coordinate: min: (-1500, -1000), max: (1499, 999)
            // GpuCoordinate: min: (0, 0), max: (2999, 1999)
            ((x + 1500) as u32, (y + 1000) as u32)
        }
        match coordinate {
            Coordinate::Simple { x, y } => {
                let (x, y) = convert((x, y));
                GpuCoordinate {
                    tag: 0,
                    data: [x, y, 0, 0],
                }
            }
            Coordinate::Rectangle { x1, y1, x2, y2 } => {
                let (x1, y1) = convert((x1, y1));
                let (x2, y2) = convert((x2, y2));
                GpuCoordinate {
                    tag: 1,
                    data: [x1, y1, x2, y2],
                }
            }
            Coordinate::Circle { x, y, radius } => {
                let (x, y) = convert((x, y));
                GpuCoordinate {
                    tag: 2,
                    data: [x, y, radius as u32, 0],
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Pod, Zeroable, Clone, Copy)]
pub struct GpuPixelData {
    pub miliseconds_since_first_pixel: u32,
    pub coordinate: GpuCoordinate,
    pub pixel_color: [u32; 3],
}

impl From<crate::data::PixelData> for GpuPixelData {
    fn from(pixel_data: crate::data::PixelData) -> Self {
        GpuPixelData {
            miliseconds_since_first_pixel: pixel_data.miliseconds_since_first_pixel,
            coordinate: pixel_data.coordinate.into(),
            pixel_color: [
                pixel_data.pixel_color.r as u32,
                pixel_data.pixel_color.g as u32,
                pixel_data.pixel_color.b as u32,
            ],
        }
    }
}
