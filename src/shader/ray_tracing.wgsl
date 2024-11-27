@compute @workgroup_size(16, 16)
fn compute_main(
    @builtin(global_invocation_id)
    gid : vec3<u32>
) {
    if gid.x >= context.width || gid.y >= context.height {
        return;
    }

    pixel_position.x = f32(gid.x);
    pixel_position.y = f32(gid.y);

    let pixel_index = gid.x + gid.y * context.width;
    init_random_generator(pixel_index);

    var ray = get_ray();
    let color = ray_color(&ray);

    pixel_color[pixel_index][0] = color.x;
    pixel_color[pixel_index][1] = color.y;
    pixel_color[pixel_index][2] = color.z;
    
    textureStore(surface, gid.xy, vec4(linear_to_srgb(color), 1.0));
}

var<private> pixel_position: vec2<f32>;

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
var<storage, read_write> pixel_color: array<array<f32, 3>>; // 这里如果内部使用 vec3 会浪费 4 个字节用于对齐

@group(0) @binding(2)
var<storage, read> quads: array<Quad>;

@group(0) @binding(3)
var surface: texture_storage_2d<rgba8unorm, write>;

fn ray_color(
    ray: ptr<function, Ray>,
) -> vec3<f32> {
    let n = arrayLength(&quads);
    var hit_record: HitRecord;
    var interval = Interval_init_2f(0.001, MAX);
    for (var i: u32 = 0; i < n; i++) {
        if Quad_hit(i, ray, &interval, &hit_record) {
            return vec3<f32>(1.0, 1.0, 1.0);
        }
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

/*----------------------------------------- Constants -------------------------------------------*/

const PI = 3.1415927f;
const ZERO_TOLERANCE = 1e-8;
// 这两个值是从 Rust 标准库抄来的，浏览器上无法编译
//const MAX = 3.40282347e+38f;
//const MIN = -3.40282347e+38f;
// 最低有效位 -1
const MAX = 3.40282346e+38f;
const MIN = -3.40282346e+38f;

/*------------------------------------------- Quad ----------------------------------------------*/

struct Quad {
    bottom_left: vec3<f32>,
    material_id: u32, // 放在这满足对齐，节省空间
    right: vec3<f32>,
    area: f32,
    up: vec3<f32>,
    d: f32,       // quad 所在平面的方程 ax + by + cz + d 中的 d
    normal: vec3<f32>,
    w: vec3<f32>  // w 是将 quad 所在平面上的点转换到 quad 定义的坐标系（bottom_left, right, up）上时需要用到的变量
                  // w = normal / dot(normal, normal) ，详见 Ray Tracing: The Next Week, p59
}

fn Quad_hit(
    id: u32, // storage 空间的指针不能作为函数参数，所以这里用索引
    ray: ptr<function, Ray>,
    interval: ptr<function, Interval>,
    hit_record: ptr<function, HitRecord>,
) -> bool {
    let quad: ptr<storage, Quad> = &quads[id];
    let nd = dot((*quad).normal, (*ray).direction);

    // No hit if the ray is parallel to the plane.
    if abs(nd) < ZERO_TOLERANCE {
        return false;
    }

    // Return false if the hit point parameter t is outside the ray interval.
    let t = ((*quad).d - dot((*quad).normal, (*ray).origin)) / nd;
    if !Interval_contains(interval, t) {
        return false;
    }

    // Determine the hit point lies within the planar shape using its plane coordinates.
    let intersection = Ray_at(ray, t);
    let planar_hit_vector = intersection - (*quad).bottom_left;
    let alpha = dot((*quad).w, cross(planar_hit_vector, (*quad).up));
    let beta = dot((*quad).w, cross((*quad).right, planar_hit_vector));

    if !Quad_is_interior(alpha, beta, hit_record) {
        return false;
    }

    (*hit_record).ray_t = t;
    (*hit_record).position = intersection;
    (*hit_record).material_id = (*quad).material_id;

    // 如果这里的第 3 个参数传入指针，就应该是 &quad.normal ，但这种写法要求支持 WGSL 扩展 unrestricted_pointer_parameters
    // https://www.w3.org/TR/WGSL/#language_extension-unrestricted_pointer_parameters
    // 浏览器是支持的，但 wgpu 没有在其他平台上实现。
    //
    // 如果在不支持的平台上在这里用指针，会报一个奇怪的错：
    // internal error: entered unreachable code: Expression [50] is not cached!
    HitRecord_set_face_normal(hit_record, ray, (*quad).normal);

    return true;
}

fn Quad_is_interior(
    alpha: f32,
    beta: f32,
    hit_record: ptr<function, HitRecord>
) -> bool {
    var unit_interval = Interval_init_2f(0.0, 1.0);

    // Given the hit point in plane coordinates, return false if it is outside the
    // primitive, otherwise set the hit record UV coordinates and return true.

    if !Interval_contains(&unit_interval, alpha) || !Interval_contains(&unit_interval, beta) {
        return false;
    }

    (*hit_record).u = alpha;
    (*hit_record).v = beta;

    return true;
}

/*---------------------------------------- Hit Record -------------------------------------------*/

struct HitRecord {
    position: vec3<f32>,
    ray_t: f32,
    normal: vec3<f32>,
    material_id: u32,
    u: f32,
    v: f32,
    is_front_face: bool,
}

fn HitRecord_set_face_normal(
    s: ptr<function, HitRecord>,
    ray: ptr<function, Ray>,
    outward_normal: vec3<f32>
) {
    (*s).is_front_face = (dot((*ray).direction, outward_normal) < 0);
}

/*----------------------------------------- Interval --------------------------------------------*/

struct Interval {
    min: f32,
    max: f32
}

fn Interval_init_empty() -> Interval {
    var interval: Interval;
    interval.min = MAX;
    interval.max = MIN;
    return interval;
}

fn Interval_init_universe() -> Interval {
    var interval: Interval;
    interval.min = MIN;
    interval.max = MAX;
    return interval;
}

fn Interval_init_2f(min: f32, max: f32) -> Interval {
    var interval: Interval;
    interval.min = min;
    interval.max = max;
    return interval;
}

fn Interval_init_2interval(a: ptr<function, Interval>, b: ptr<function, Interval>) -> Interval {
    var interval: Interval;

    if (*a).min <= (*b).min {
        interval.min = (*a).min;
    } else {
        interval.min = (*b).min;
    }

    if (*a).max >= (*b).max {
        interval.max = (*a).max;
    } else {
        interval.max = (*b).max;
    }

    return interval;
}

fn Interval_size(s: ptr<function, Interval>) -> f32 {
    return (*s).max - (*s).min;
}

fn Interval_contains(s: ptr<function, Interval>, x: f32) -> bool {
    return (*s).min <= x && x <= (*s).max;
}

fn Interval_surrounds(s: ptr<function, Interval>, x: f32) -> bool {
    return (*s).min < x && x < (*s).max;
}

fn Interval_clamp(s: ptr<function, Interval>, x: f32) -> f32 {
    if x < (*s).min { return (*s).min; }
    if x > (*s).max { return (*s).max; }
    return x;
}

fn Interval_expand(s: ptr<function, Interval>, delta: f32) {
    let padding = delta / 2.0;

    (*s).min = (*s).min - padding;
    (*s).max = (*s).max + padding;
}

fn Interval_displace(s: ptr<function, Interval>, displacement: f32) {
    (*s).min = (*s).min + displacement;
    (*s).max = (*s).max + displacement;
}

/*--------------------------------------------- Ray ---------------------------------------------*/

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn Ray_at(s: ptr<function, Ray>, t: f32) -> vec3<f32> {
    return (*s).origin + t * (*s).direction;
}

fn get_ray() -> Ray {
    let offset = sample_unit_square_stratified();
    let pixel_world_position = context.pixel_origin
                                + (pixel_position.x + offset.x) * context.pixel_delta_u
                                + (pixel_position.y + offset.y) * context.pixel_delta_v;

    var ray_origin: vec3<f32>;
    if context.defocus_angle <= 0 {
        ray_origin = context.camera_position;
    } else {
        ray_origin = defocus_disk_sample();
    }

    let ray_direction = normalize(pixel_world_position - ray_origin);

    return Ray(ray_origin, ray_direction);
}

fn defocus_disk_sample() -> vec3<f32> {
    let s = sample_unit_disk();
    return context.camera_position + s.x * context.defocus_disk_u + s.y * context.defocus_disk_v;
}

/*------------------------------------------- Sampling ------------------------------------------*/

fn sample_unit_disk() -> vec2<f32> {
    let phi = 2 * PI * random();
    let r = sqrt(random());

    return vec2<f32>(cos(phi), sin(phi)) * r;
}

fn sample_unit_square_stratified() -> vec2<f32> {
    if context.sample_id < context.sample_grid_num { // 分层采样
        return vec2<f32>(
            f32(context.sample_position.x) + random(),
            f32(context.sample_position.y) + random()
        ) * context.sample_grid_len - 0.5;
    } else { // 直接采样
        return sample_unit_square();
    }
}

fn sample_unit_square() -> vec2<f32> {
    return vec2<f32>(random(), random()) - 0.5;
}

/*---------------------------------- Random Number Generation -----------------------------------*/

struct RandomGeneratorState {
    z1: u32,
    z2: u32,
    z3: u32,
    z4: u32,
    r: u32,
}
var<private> random_state: RandomGeneratorState;

fn init_random_generator(id: u32) {
    random_state.r = seed(id);
}

fn random() -> f32 {
    // Efficient pseudo-random number generation for monte-carlo simulations using graphic processors
    // Siddhant Mohanty et al 2012 J. Phys.: Conf. Ser. 368 012024

    // Hybrid Tausworthe Generator:
    // Combined period is lcm(p1, p2, p3, p4) ~ 2^121
    random_state.z1 = taus_step(random_state.r, 13u, 19u, 12u, 4294967294u);  // p1 = 2^31-1
    random_state.z2 = taus_step(random_state.r, 2u, 25u, 4u, 4294967288u);    // p2 = 2^30-1
    random_state.z3 = taus_step(random_state.r, 3u, 11u, 17u, 4294967280u);   // p3 = 2^28-1
    random_state.z4 = lcg_step(random_state.r, 1664525u, 1013904223u);        // p4 = 2^32
    random_state.r = random_state.z1 ^ random_state.z2 ^ random_state.z3 ^ random_state.z4;

    // 论文里是这个数，但实际上 f32 只能表示 7 到 8 个十进制位
    return 2.3283064365387e-10f * f32(random_state.r); // [0, 1]
}

// S1, S2, S3, and M are all constants, and z is part of the
// private per-thread generator state.
fn taus_step(z: u32, s1: u32, s2: u32, s3: u32, m: u32) -> u32 {
    let b = (((z << s1) ^ z) >> s2);
    let new_z = (((z & m) << s3) ^ b);
    return new_z;
}

// A and C are constants
fn lcg_step(z: u32, a: u32, c: u32) -> u32 {
    let new_z = (a * z + c);
    return new_z;
}

// Function giving seed for each thread
fn seed(id: u32) -> u32  {
    return id * 1099087573u;
}

/*------------------------------------- sRGB Color Space ----------------------------------------*/


fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    // https://gamedev.stackexchange.com/questions/92015/optimized-linear-to-srgb-glsl
    let cutoff = color.rgb < vec3(0.0031308);
    let higher = vec3(1.055) * pow(color.rgb, vec3(1.0 / 2.4)) - vec3(0.055);
    let lower = color.rgb * vec3(12.92);
    return select(higher, lower, cutoff);
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    let cutoff = color.rgb < vec3(0.04045);
    let higher = pow((color.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
    let lower = color.rgb / vec3(12.92);
    return select(higher, lower, cutoff);
}