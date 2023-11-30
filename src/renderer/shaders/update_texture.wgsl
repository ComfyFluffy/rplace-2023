struct GpuCoordinate {
    tag: u32,
    data: vec4<u32>,
}

struct GpuPixelData {
    miliseconds_since_first_pixel: u32,
    coordinate: GpuCoordinate,
    color: vec3<u32>,
}

@group(0) @binding(0) var<storage, read> pixel_updates: array<GpuPixelData>;
@group(0) @binding(1) var texture_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<storage, read_write> last_index_for_coordinate: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> canvas_size: vec2<u32>;

fn store_pixel_to_texture(
    index: u32,
    coordinate: vec2<u32>,
    color: vec3<u32>,
) {
    let last_updated_by_atomic = &last_index_for_coordinate[
        coordinate.x + coordinate.y * canvas_size.x
    ];
    let last_updated_by = atomicMax(last_updated_by_atomic, index);
    if (last_updated_by > index) {
        // This pixel has already been updated by a newer pixel.
        return;
    }
    textureStore(
        texture_out,
        coordinate,
        vec4<f32>(vec3<f32>(color.xyz) / 255.0, 1.0)
    );
}

@compute
@workgroup_size(256)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
) {
    let pixel_data = pixel_updates[id.x];

    switch (pixel_data.coordinate.tag) {
        case 0u: {
            // Single pixel. Fill pixel_data.coordinate.data.xy.
            store_pixel_to_texture(
                id.x,
                pixel_data.coordinate.data.xy,
                pixel_data.color,
            );
        }
        case 1u: {
            // Quad. Fill from pixel_data.coordinate.data.xy to pixel_data.coordinate.data.zw.
            for (var x: u32 = pixel_data.coordinate.data.x; x < pixel_data.coordinate.data.z; x = x + 1u) {
                for (var y: u32 = pixel_data.coordinate.data.y; y < pixel_data.coordinate.data.w; y = y + 1u) {
                    store_pixel_to_texture(
                        id.x,
                        vec2<u32>(x, y),
                        pixel_data.color,
                    );
                }
            }
        }
        case 2u: {
            // Circle. Fill circle with center pixel_data.coordinate.data.xy and radius pixel_data.coordinate.data.z.
            for (var x: u32 = pixel_data.coordinate.data.x - pixel_data.coordinate.data.z; x < pixel_data.coordinate.data.x + pixel_data.coordinate.data.z; x = x + 1u) {
                for (var y: u32 = pixel_data.coordinate.data.y - pixel_data.coordinate.data.z; y < pixel_data.coordinate.data.y + pixel_data.coordinate.data.z; y = y + 1u) {
                    if (pow(f32(x) - f32(pixel_data.coordinate.data.x), 2.0) + pow(f32(y) - f32(pixel_data.coordinate.data.y), 2.0) < pow(f32(pixel_data.coordinate.data.z), 2.0)) {
                        store_pixel_to_texture(
                            id.x,
                            vec2<u32>(x, y),
                            pixel_data.color,
                        );
                    }
                }
            }
        }
        default: {}
    }
}
