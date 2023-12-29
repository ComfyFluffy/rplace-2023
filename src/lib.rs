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

pub fn find_never_updated_pixels() {
    let iter = GzippedBinPixelDataReader::new("pixels.bin").unwrap();
    let mut pixels = vec![false; 3000 * 2000];

    for (index, pixel_data) in iter.enumerate() {
        if index % 100000 == 0 {
            println!("{}", index);
        }
        let pixel_data = pixel_data.unwrap();
        fn convert((x, y): (i16, i16)) -> Option<(usize, usize)> {
            let x = x as i32 + 1500;
            let y = -y as i32 - 1 + 1000;
            if x >= 0 && x < 3000 && y >= 0 && y < 2000 {
                Some((x as usize, y as usize))
            } else {
                None
            }
        }
        match pixel_data.coordinate {
            Coordinate::Simple { x, y } => {
                if let Some((x, y)) = convert((x, y)) {
                    pixels[y * 3000 + x] = true;
                }
            }
            Coordinate::Circle { x, y, radius } => {
                if let Some((x, y)) = convert((x, y)) {
                    let (min_i, max_i) = (
                        x.saturating_sub(radius as usize),
                        (x + radius as usize).min(2999),
                    );
                    let (min_j, max_j) = (
                        y.saturating_sub(radius as usize),
                        (y + radius as usize).min(1999),
                    );

                    for i in min_i..=max_i {
                        for j in min_j..=max_j {
                            if (i as i32 - x as i32).pow(2) + (j as i32 - y as i32).pow(2)
                                < radius.pow(2) as i32
                            {
                                pixels[j * 3000 + i] = true;
                            }
                        }
                    }
                }
            }
            Coordinate::Rectangle { x1, y1, x2, y2 } => {
                if let (Some((x1, y1)), Some((x2, y2))) = (convert((x1, y1)), convert((x2, y2))) {
                    for i in x1..x2 {
                        for j in y1..y2 {
                            pixels[j * 3000 + i] = true;
                        }
                    }
                }
            }
        }
    }

    let never_updated = pixels.iter().filter(|&&x| !x).count();

    println!("Never updated: {}", never_updated);
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
