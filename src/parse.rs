use std::{fs::File, io::Write};

use chrono::{DateTime, Utc};
use csv::ReaderBuilder;
use flate2::read::GzDecoder;
use rayon::prelude::*;
use serde::Deserialize;
use snafu::{prelude::*, Whatever};

use crate::data::PixelData;

#[derive(Debug, Deserialize)]
pub struct CsvRecord {
    pub timestamp: String,
    pub user: String, // We'll ignore this in PixelData
    pub coordinate: String,
    pub pixel_color: String,
}

impl CsvRecord {
    pub fn to_pixel_data(
        self,
        first_pixel_time: DateTime<Utc>,
    ) -> Result<crate::data::PixelData, Whatever> {
        let timestamp = DateTime::parse_from_rfc3339(&self.timestamp.replace(" UTC", "Z"))
            .whatever_context("Invalid timestamp")?;
        let miliseconds_since_first_pixel: u32 = (timestamp.timestamp_millis()
            - first_pixel_time.timestamp_millis())
        .try_into()
        .whatever_context("Timestamp is before first pixel")?;
        let coordinate = self
            .coordinate
            .parse()
            .whatever_context("Invalid coordinate")?;
        let pixel_color = self
            .pixel_color
            .parse()
            .whatever_context("Invalid pixel color")?;
        Ok(crate::data::PixelData {
            miliseconds_since_first_pixel,
            coordinate,
            pixel_color,
        })
    }
}

struct GzippedCsvPixelDataReader {
    deserializer: csv::DeserializeRecordsIntoIter<GzDecoder<File>, CsvRecord>,
    first_pixel_time: DateTime<Utc>,
}

impl GzippedCsvPixelDataReader {
    fn new(first_pixel_time: DateTime<Utc>, path: &str) -> Result<Self, Whatever> {
        let file = File::open(path).whatever_context("Failed to open file")?;
        let decoder = GzDecoder::new(file);
        let reader = ReaderBuilder::new().has_headers(true).from_reader(decoder);
        let deserializer = reader.into_deserialize();
        Ok(Self {
            deserializer,
            first_pixel_time,
        })
    }
}

impl Iterator for GzippedCsvPixelDataReader {
    type Item = Result<PixelData, Whatever>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.deserializer.next() {
            Some(Ok(record)) => Some(record.to_pixel_data(self.first_pixel_time)),
            Some(Err(e)) => Some(Err(e).whatever_context("Failed to parse record")),
            None => None,
        }
    }
}

pub fn parse_and_write_to_bin(parent_dir: &str) {
    let first_pixel_time = DateTime::parse_from_rfc3339("2023-07-20 13:00:26.088Z")
        .unwrap()
        .with_timezone(&Utc);
    let gz_writer = File::create("pixels.bin").unwrap();
    let mut gz_writer = flate2::write::GzEncoder::new(gz_writer, flate2::Compression::default());
    let bincode_config = bincode::config::standard();

    // for index in 0..=52 {
    //     let path = format!("{parent_dir}/2023_place_canvas_history-{index:012}.csv.gzip");
    //     println!("Reading {}", path);
    //     let reader = GzippedCsvPixelDataReader::new(first_pixel_time, &path).unwrap();
    //     for pixel_data in reader {
    //         let pixel_data = pixel_data.unwrap();

    //         bincode::encode_into_std_write(&pixel_data, &mut gz_writer, bincode_config).unwrap();
    //     }
    // }

    // Parrallel version
    let data: Vec<u8> = (0..=52)
        .into_par_iter()
        .map(|index| {
            let path = format!("{parent_dir}/2023_place_canvas_history-{index:012}.csv.gzip");
            println!("Reading {}", path);
            let reader = GzippedCsvPixelDataReader::new(first_pixel_time, &path).unwrap();

            let mut data = vec![];
            for pixel_data in reader {
                let pixel_data = pixel_data.unwrap();

                let current = bincode::encode_to_vec(&pixel_data, bincode_config).unwrap();
                data.extend(current);
            }
            data
        })
        .flatten()
        .collect();

    gz_writer.write_all(&data).unwrap();
}

pub fn parse_bin() -> Result<Vec<PixelData>, Whatever> {
    let results = vec![];
    let gz_reader = File::open("pixels.bin").unwrap();
    let mut gz_reader = flate2::read::GzDecoder::new(gz_reader);
    let bincode_config = bincode::config::standard();
    loop {
        match bincode::decode_from_std_read::<PixelData, _, _>(&mut gz_reader, bincode_config) {
            Ok(pixel_data) => {
                // println!("{:?}", pixel_data);
            }
            Err(e) => {
                match &e {
                    bincode::error::DecodeError::Io { inner, .. } => {
                        if inner.kind() == std::io::ErrorKind::UnexpectedEof {
                            break;
                        }
                    }
                    _ => {}
                };
                whatever!("Failed to decode: {}", e)
            }
        }
    }
    Ok(results)
}
