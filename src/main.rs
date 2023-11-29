use rplace_2023::get_max_min_coord;

// mod renderer;
fn main() {
    // let instant = std::time::Instant::now();
    // get_max_min_coord();
    // println!("Time: {:?}", instant.elapsed());
    pollster::block_on(rplace_2023::run());
}
