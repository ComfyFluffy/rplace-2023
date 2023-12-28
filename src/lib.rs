use renderer::App;

use crate::{data::Coordinate, parse::GzippedBinPixelDataReader};

pub mod data;
pub mod parse;
mod renderer;

pub fn get_max_min_coord() {
    let iter = GzippedBinPixelDataReader::new("pixels.bin").unwrap();

    let mut min = (std::i16::MAX, std::i16::MAX);
    let mut max = (std::i16::MIN, std::i16::MIN);

    for pixel_data in iter {
        let pixel_data = pixel_data.unwrap();
        if let Coordinate::Simple { x, y } = pixel_data.coordinate {
            if x < min.0 {
                min.0 = x;
            }
            if y < min.1 {
                min.1 = y;
            }
            if x > max.0 {
                max.0 = x;
            }
            if y > max.1 {
                max.1 = y;
            }
        }
    }
    println!("min: {:?}, max: {:?}", min, max);
}

pub fn print_quad_circle() {
    let reader = GzippedBinPixelDataReader::new("pixels.bin").unwrap();
    for pixel_data in reader {
        let pixel_data = pixel_data.unwrap();
        match pixel_data.coordinate {
            Coordinate::Circle { x, y, radius } => {
                if radius > 10 {
                    println!("Circle: {:?}, {:?}", x, y);
                }
            }
            Coordinate::Rectangle { x1, y1, x2, y2 } => {
                if x2 - x1 > 10 && y2 - y1 > 10 {
                    println!("Rectangle: {:?}, {:?}, {:?}, {:?}", x1, y1, x2, y2);
                }
            }
            _ => {}
        }
    }
}

pub fn run() {
    let mut app = App::new();

    app.run(GzippedBinPixelDataReader::new("pixels.bin").unwrap(), 10000);
}
