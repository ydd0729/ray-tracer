@vertex
fn vertex_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = in.position;
    out.tex = in.tex;
    return out;
}

@fragment
fn fragment_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let u = in.tex.x;
    let v = in.tex.y;

    let x = u32(u * f32(context.width));
    let y = u32(v * f32(context.height));
    let pixel_index = x + context.width * y;

    let color = vec4<f32>(
        pixel_color[pixel_index][0],
        pixel_color[pixel_index][1],
        pixel_color[pixel_index][2],
        1.0
    );
    return linear_to_srgb(color);
}

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec4<f32>,
    @location(3) tex: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex: vec2<f32>,
};

struct RayTracingContext {
    width: u32,
    height: u32
}
@group(0) @binding(0)
var<uniform> context: RayTracingContext;

@group(0) @binding(1)
var<storage, read_write> pixel_color: array<array <f32, 3>>;

fn linear_to_srgb(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        linear_to_srgb_f32(color.x),
        linear_to_srgb_f32(color.y),
        linear_to_srgb_f32(color.z),
        linear_to_srgb_f32(color.w),
    );
}

fn linear_to_srgb_f32(color: f32) -> f32 {
    if color <= 0.0031308 {
        return color * 12.92;
    } else {
        return 1.055 * pow(color, 1.0 / 2.4) - 0.055;
    }
}

fn srgb_to_linear(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        srgb_to_linear_f32(color.x),
        srgb_to_linear_f32(color.y),
        srgb_to_linear_f32(color.z),
        srgb_to_linear_f32(color.w),
    );
}

fn srgb_to_linear_f32(color: f32) -> f32 {
    if color <= 0.04045 {
        return color / 12.92;
    } else {
        return pow((color + 0.055) / 1.055, 2.4);
    }
}