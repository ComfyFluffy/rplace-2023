// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

// Fragment shader
@group(0) @binding(0) var mySampler: sampler;
@group(0) @binding(1) var myTexture: texture_2d<f32>;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // let color = vec4<f32>(vec3<f32>(in.uv.x, in.uv.x, in.uv.x) * 3.0, 1.0);
    let color = textureSample(myTexture, mySampler, in.uv);
    return color;
}
