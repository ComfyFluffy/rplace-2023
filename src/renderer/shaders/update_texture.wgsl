struct GpuPixelData {
    miliseconds_since_first_pixel: u32,
    coordinate_tag: u32,
    coordinate_data: vec4<u32>,
    color: vec3<u32>,
}

@group(0) @binding(0) var<storage, read> pixel_updates: array<GpuPixelData>;
@group(0) @binding(1) var texture_out: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(256)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
) {
    let pixel_data = pixel_updates[id.x];
    switch (pixel_data.coordinate_tag) {
        case 1u: {
            // Single pixel. Fill pixel_data.coordinate.data.xy.
            textureStore(
                texture_out,
                vec2<u32>(pixel_data.coordinate_data.x, pixel_data.coordinate_data.y),
                vec4<f32>(vec3<f32>(
                    pixel_data.color.xyz
                ) / 255.0, 1.0)
            );
        }
        // case 1u: {
        //     // Quad. Fill from pixel_data.coordinate.data.xy to pixel_data.coordinate.data.zw.
        //     for (var x: u32 = pixel_data.coordinate.data.x; x < pixel_data.coordinate.data.z; x = x + 1u) {
        //         for (var y: u32 = pixel_data.coordinate.data.y; y < pixel_data.coordinate.data.w; y = y + 1u) {
        //             textureStore(texture_out, vec2<u32>(x, y),
        //                         vec4<f32>(vec3<f32>(pixel_data.pixel_color.xyz) / 255.0, 1.0)
        //             );
        //         }
        //     } 
        // }
        // case 2u: {
        //     // Circle. Fill circle with center pixel_data.coordinate.data.xy and radius pixel_data.coordinate.data.z.
        //     for (var x: u32 = pixel_data.coordinate.data.x - pixel_data.coordinate.data.z; x < pixel_data.coordinate.data.x + pixel_data.coordinate.data.z; x = x + 1u) {
        //         for (var y: u32 = pixel_data.coordinate.data.y - pixel_data.coordinate.data.z; y < pixel_data.coordinate.data.y + pixel_data.coordinate.data.z; y = y + 1u) {
        //             if (pow(f32(x) - f32(pixel_data.coordinate.data.x), 2.0) + pow(f32(y) - f32(pixel_data.coordinate.data.y), 2.0) < pow(f32(pixel_data.coordinate.data.z), 2.0)) {
        //                 textureStore(texture_out, vec2<u32>(x, y),
        //                             vec4<f32>(vec3<f32>(pixel_data.pixel_color.xyz) / 255.0, 1.0)
        //                 );
        //             }
        //         }
        //     }
        // }
        default: {}
    }
}
