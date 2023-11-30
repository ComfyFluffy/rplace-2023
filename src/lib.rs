use std::{mem::size_of, time::Instant};

use crevice::std140::AsStd140;
use log::{error, warn};
use renderer::{data::Std140GpuPixelData, update_texture::WORKGROUP_SIZE, State};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

use crate::{data::Coordinate, parse::GzippedBinPixelDataReader, renderer::data::GpuPixelData};

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

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;
    let mut window_occluded = false;

    let mut reader = GzippedBinPixelDataReader::new("pixels.bin").unwrap();

    let render_start = Instant::now();
    let playback_speed = 10000;

    let mut buffer = Vec::new();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let elapsed_ms = render_start.elapsed().as_millis() as u32 * playback_speed;

                    for pixel_data in &mut reader {
                        let pixel_data: GpuPixelData = pixel_data.unwrap().into();
                        buffer.push(pixel_data.as_std140());
                        if pixel_data.miliseconds_since_first_pixel > elapsed_ms {
                            break;
                        }
                    }

                    let data = buffer
                        .drain(
                            ..(buffer.len() / WORKGROUP_SIZE as usize * WORKGROUP_SIZE as usize)
                                // 128 MiB (max buffer size)
                                .min(128 * 1024 * 1024 / size_of::<Std140GpuPixelData>())
                                .min(65535 * WORKGROUP_SIZE as usize),
                        )
                        .collect::<Vec<_>>();

                    match state.render(data.as_slice()) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            state.resize(state.size);
                            warn!("Lost surface");
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                        Err(e) => error!("render error: {:?}", e),
                    }
                }
                WindowEvent::Occluded(occluded) => {
                    window_occluded = occluded;
                }
                _ => {}
            },
            Event::AboutToWait => {
                if !window_occluded {
                    state.window().request_redraw();
                }
            }
            _ => {}
        })
        .unwrap();
}
