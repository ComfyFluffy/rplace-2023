use crevice::std140::{AsStd140, UVec3, UVec4};

use crate::data::Coordinate;

#[derive(Debug, AsStd140)]
pub struct GpuPixelData {
    pub miliseconds_since_first_pixel: u32,
    pub coordinate_tag: u32,
    pub coordinate_data: crevice::std140::UVec4,
    pub color: crevice::std140::UVec3,
}

impl From<crate::data::PixelData> for GpuPixelData {
    fn from(pixel_data: crate::data::PixelData) -> Self {
        fn convert((x, y): (i16, i16)) -> (u32, u32) {
            // In Coordinate the origin is in the center of the image.
            // We need to convert it to the top left corner.
            // Coordinate: min: (-1500, -1000), max: (1499, 999)
            // GpuCoordinate: min: (0, 0), max: (2999, 1999)
            ((x + 1500) as u32, (y + 1000) as u32)
        }
        let (coordinate_tag, coordinate_data) = match pixel_data.coordinate {
            Coordinate::Simple { x, y } => {
                let (x, y) = convert((x, y));
                (1, UVec4 { x, y, z: 0, w: 0 })
            }
            Coordinate::Rectangle { x1, y1, x2, y2 } => {
                let (x1, y1) = convert((x1, y1));
                let (x2, y2) = convert((x2, y2));
                (
                    2,
                    UVec4 {
                        x: x1,
                        y: y1,
                        z: x2,
                        w: y2,
                    },
                )
            }
            Coordinate::Circle { x, y, radius } => {
                let (x, y) = convert((x, y));
                (
                    3,
                    UVec4 {
                        x,
                        y,
                        z: radius as u32,
                        w: 0,
                    },
                )
            }
        };

        GpuPixelData {
            miliseconds_since_first_pixel: pixel_data.miliseconds_since_first_pixel,
            coordinate_tag,
            coordinate_data,
            color: UVec3 {
                x: pixel_data.pixel_color.r as u32,
                y: pixel_data.pixel_color.g as u32,
                z: pixel_data.pixel_color.b as u32,
            },
        }
    }
}
