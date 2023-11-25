use crate::{data::Coordinate, parse::GzippedBinPixelDataReader};

mod data;
mod parse;

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
