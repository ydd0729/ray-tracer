@vertex
fn vertex_main(
    @location(0) position: vec4<f32>
) -> @builtin(position) vec4<f32> {
    return position;
}

@fragment
fn fragment_main(
    @builtin(position) position: vec4<f32>,
) -> @location(0) vec4<f32> {
    let pixel_index = u32(position.x) + context.width * u32(position.y);

    let color = vec4<f32>(
        pixel_color[pixel_index][0],
        pixel_color[pixel_index][1],
        pixel_color[pixel_index][2],
        1.0
    );
    return linear_to_srgb(color);
}

struct RenderContext {
    width: u32,
    height: u32,
    sample_position: vec2<u32>,
    pixel_origin: vec3<f32>,
    samples_per_pixel: u32,
    pixel_delta_u: vec3<f32>,
    sample_grid_num: u32,
    pixel_delta_v: vec3<f32>,
    defocus_angle: f32,
    defocus_disk_u: vec3<f32>,
    sample_grid_len: f32,
    defocus_disk_v: vec3<f32>,
    sample_id: u32,
    camera_position: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> context: RenderContext;

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