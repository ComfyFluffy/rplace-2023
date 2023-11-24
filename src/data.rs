use std::str::FromStr;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum Coordinate {
    Simple { x: i16, y: i16 },
    Rectangle { x1: i16, y1: i16, x2: i16, y2: i16 },
    Circle { x: i16, y: i16, radius: i16 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct PixelData {
    pub miliseconds_since_first_pixel: u32,
    pub coordinate: Coordinate,
    pub pixel_color: PixelColor,
}

impl FromStr for Coordinate {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_coordinate(
            coordinate_str: &str,
        ) -> Result<Coordinate, Box<dyn std::error::Error>> {
            if coordinate_str.starts_with('{') {
                // Parse as Circle
                // Example: "{X: 424, Y: 336, R: 3}"
                let parts: Vec<_> = coordinate_str
                    .strip_prefix('{')
                    .ok_or("Invalid format")?
                    .strip_suffix('}')
                    .ok_or("Invalid format")?
                    .split(',')
                    .collect();
                let x = parts[0]
                    .split_once(": ")
                    .ok_or("Invalid format")?
                    .1
                    .parse()?;
                let y = parts[1]
                    .split_once(": ")
                    .ok_or("Invalid format")?
                    .1
                    .parse()?;

                let radius = parts[2]
                    .split(": ")
                    .nth(1)
                    .ok_or("Invalid format")?
                    .parse()?;
                Ok(Coordinate::Circle { x, y, radius })
            } else {
                let parts: Vec<_> = coordinate_str.split(',').collect();
                match parts.len() {
                    2 => {
                        // Parse as Simple
                        let x = parts[0].parse()?;
                        let y = parts[1].parse()?;
                        Ok(Coordinate::Simple { x, y })
                    }
                    4 => {
                        // Parse as Rectangle
                        let x1 = parts[0].parse()?;
                        let y1 = parts[1].parse()?;
                        let x2 = parts[2].parse()?;
                        let y2 = parts[3].parse()?;
                        Ok(Coordinate::Rectangle { x1, y1, x2, y2 })
                    }
                    _ => Err("Unknown coordinate format".into()),
                }
            }
        }

        parse_coordinate(s)
    }
}

impl FromStr for PixelColor {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_pixel_color(color_str: &str) -> Result<PixelColor, Box<dyn std::error::Error>> {
            // Remove the '#' character and parse the remaining hex string
            let color = u32::from_str_radix(&color_str[1..], 16)?;

            // Extract RGB components
            let r = ((color >> 16) & 255) as u8;
            let g = ((color >> 8) & 255) as u8;
            let b = (color & 255) as u8;

            Ok(PixelColor { r, g, b })
        }

        parse_pixel_color(s)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_coordinate() {
        use super::Coordinate;
        use std::str::FromStr;

        let coordinate = Coordinate::from_str("{X: 424, Y: 336, R: 3}").unwrap();
        assert_eq!(
            coordinate,
            Coordinate::Circle {
                x: 424,
                y: 336,
                radius: 3
            }
        );

        let coordinate = Coordinate::from_str("424,336").unwrap();
        assert_eq!(coordinate, Coordinate::Simple { x: 424, y: 336 });

        let coordinate = Coordinate::from_str("424,336,425,337").unwrap();
        assert_eq!(
            coordinate,
            Coordinate::Rectangle {
                x1: 424,
                y1: 336,
                x2: 425,
                y2: 337
            }
        );

        let coordinate = Coordinate::from_str("424,336,425,337,3").unwrap_err();
        assert_eq!(coordinate.to_string(), "Unknown coordinate format");
    }
}
