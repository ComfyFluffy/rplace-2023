
struct GpuCoordinate {
    tag: i32,
    data: vec4<i32>,
}

struct GpuPixelData {
    miliseconds_since_first_pixel: u32,
    coordinate: GpuCoordinate,
    pixel_color: vec3<u32>,
}

@group(0) @binding(0) var<storage, read> pixel_updates: array<GpuPixelData>;
@group(0) @binding(1) var texture_out: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pixel_data = pixel_updates[id.x];
    textureStore(texture_out, pixel_data.coordinate.data.xy,
                    vec4<f32>(vec3<f32>(pixel_data.pixel_color.xyz) / 255.0, 1.0)
    );
    textureStore(texture_out, vec2(id.x, id.x),
                    vec4<f32>(1.0)
    );
}
