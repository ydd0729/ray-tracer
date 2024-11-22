@vertex
fn vertex_main(
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = position;
    out.color = color;
    return out;
}

@fragment
fn fragment_main(
    @location(0) color: vec4<f32>,
) -> @location(0) vec4<f32> {
    return linear_to_srgb(color);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

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