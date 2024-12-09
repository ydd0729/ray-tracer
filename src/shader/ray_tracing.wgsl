/*--------------------------------------- Constants ---------------------------------------------*/

const PI = 3.14159265358979323846264338327950288f;
const ZERO_TOLERANCE = 1e-8;

const MAX = 0x1.fffffep+127f;
const MIN = -MAX;

const MAX_RAY_BOUNCES = 64;

const VEC3F_UNIT_X: vec3f = vec3f(1.0, 0.0, 0.0);
const VEC3F_UNIT_Y: vec3f = vec3f(0.0, 1.0, 0.0);
const VEC3F_UNIT_Z: vec3f = vec3f(0.0, 0.0, 1.0);
const VEC3F_ZEROS: vec3f = vec3f(0.0, 0.0, 0.0);
const MAT3X3F_IDENTITY: mat3x3f = mat3x3f(VEC3F_UNIT_X, VEC3F_UNIT_Y, VEC3F_UNIT_Z);
const F32_POSITIVE_MIN: f32 = 0x1p-126f;

/*---------------------------------------- Bindings ---------------------------------------------*/

@group(0) @binding(0)
var<uniform> context: RenderContext;

@group(0) @binding(1)
var<storage, read_write> pixel_color: array<array<f32, 3>>; // 这里如果使用 vec3 会浪费 4 个字节用于对齐

@group(0) @binding(2)
var<storage, read> bvh_tree: array<BvhNode>;

@group(0) @binding(3)
var<storage, read> importance: array<PrimitiveIndex>;

@group(0) @binding(4)
var<storage, read> quads: array<Quad>;

@group(0) @binding(5)
var<storage, read> spheres: array<Sphere>;

@group(0) @binding(6)
var<storage, read> lambertian_materials: array<Lambertian>;

@group(0) @binding(7)
var<storage, read> diffuse_light_materials: array<DiffuseLight>;

@group(0) @binding(8)
var<storage, read> dielectric_materials: array<Dielectric>;

@group(0) @binding(9)
var surface: texture_storage_2d<rgba8unorm, write>;

/*----------------------------------------- Ray Tracing -----------------------------------------*/

@compute @workgroup_size(16, 16)
fn compute_main(
    @builtin(global_invocation_id)
    gid : vec3<u32>
) {
    if gid.x >= context.width || gid.y >= context.height {
        return;
    }

    let pixel_index = gid.x + gid.y * context.width;
    init_random_generator(pixel_index + context.width * context.height * context.sample_id);

    var sample_color = vec3(0.0, 0.0, 0.0);
    var sample_id = context.sample_id;

    // 这个 for 循环对性能影响较大，即使 samples_per_pixel = 1 ，也会使帧率 - 8
    for (var i: u32 = 0; i < context.samples_per_frame && sample_id < context.samples_per_pixel; i++) {
        var ray = get_ray(vec2(f32(gid.x), f32(gid.y)), sample_id);
        sample_color += ray_color(&ray);
        sample_id += 1u;
    }

    if context.sample_id == 0 {
        pixel_color[pixel_index][0] = sample_color.x;
        pixel_color[pixel_index][1] = sample_color.y;
        pixel_color[pixel_index][2] = sample_color.z;
    } else if context.sample_id != sample_id {
        pixel_color[pixel_index][0] += sample_color.x;
        pixel_color[pixel_index][1] += sample_color.y;
        pixel_color[pixel_index][2] += sample_color.z;
    }

    let color = vec3(
        pixel_color[pixel_index][0] / f32(sample_id),
        pixel_color[pixel_index][1] / f32(sample_id),
        pixel_color[pixel_index][2] / f32(sample_id)
    );

    textureStore(surface, gid.xy, vec4(linear_to_srgb(color), 1.0));
}

fn get_ray(pixel_position: vec2f, sample_id: u32) -> Ray {
    let offset = sample_unit_square_stratified(sample_id);
    let pixel_world_position = context.pixel_origin
                                + (pixel_position.x + offset.x) * context.pixel_delta_u
                                + (pixel_position.y + offset.y) * context.pixel_delta_v;

    var ray_origin: vec3f;
    if context.defocus_angle == 0 {
        ray_origin = context.camera_position;
    } else {
        ray_origin = defocus_disk_sample();
    }

    let ray_direction = normalize(pixel_world_position - ray_origin);

    return Ray(ray_origin, ray_direction);
}

fn defocus_disk_sample() -> vec3f {
    let s = sample_unit_disk();
    return context.camera_position + s.x * context.defocus_disk_u + s.y * context.defocus_disk_v;
}

struct RayColorCalculationEntry {
    color_from_emission: vec3f,
    pdf_val: f32,
    attenuation: vec3f,
    scattering_pdf: f32,
    skip_pdf: bool,
}
var<private> ray_color_stack: array<RayColorCalculationEntry, MAX_RAY_BOUNCES>;

fn ray_color(
    ray: ptr<function, Ray>,
) -> vec3f {
    var stack_id = -1;

    for (var bounce = 0; bounce <= i32(context.ray_bounces); bounce++) {
        var hit_record: HitRecord;
        var interval = Interval_init_2f(0.001, MAX);
        
        var debug_color: vec3f;
        Bvh_hit(0u, ray, &interval, &hit_record, &debug_color);
        // return debug_color;

        if !hit_record.hit {
            let background = vec3f(0.0, 0.0, 0.0);
            return resolve_ray_color(stack_id, background);
        }

        let emitted_color = Material_emit(ray, &hit_record);

        var scatter_record: ScatterRecord;
        if !Material_scatter(ray, &hit_record, &scatter_record) 
        {
            return resolve_ray_color(stack_id, emitted_color);
        }

        if scatter_record.skip_pdf {
            stack_id += 1;
            ray_color_stack[stack_id].skip_pdf = true;
            ray_color_stack[stack_id].attenuation = scatter_record.attenuation;
            *ray = scatter_record.skip_pdf_ray;
            continue;
        }

        var scattered_ray: Ray;
        var scattered_origin = hit_record.position;
        scattered_ray.origin = scattered_origin;

        // importance only
        // scattered_ray.direction = importance_random(&scattered_origin);
        // let pdf_value = importance_pdf_value(&scattered_ray);
        // if pdf_value == 0 {
        //     return VEC3F_ZEROS;
        // }

        // material only
        // scattered_ray.direction = Material_random(&scattered_origin, &hit_record);
        // let pdf_value = Material_pdf_value(&scattered_ray, &hit_record);

        // Mixed
        let material_weight = 0.6; // this is an empirical value
        if randomf() > material_weight {
            scattered_ray.direction = importance_random(&scattered_origin);
            let pdf_value = importance_pdf_value(&scattered_ray);
            if pdf_value == 0 {
                return VEC3F_ZEROS;
            }
        } else {
            scattered_ray.direction = Material_random(&scattered_origin, &hit_record);
        }
        let pdf_value = (1.0 - material_weight) * importance_pdf_value(&scattered_ray) 
                        + material_weight * Material_pdf_value(&scattered_ray, &hit_record);

        let scattering_pdf_value = Material_scattering_pdf_value(&scattered_ray, &hit_record);
        
        stack_id += 1;
        *ray = scattered_ray;
        ray_color_stack[stack_id].skip_pdf = false;
        ray_color_stack[stack_id].color_from_emission = emitted_color;
        ray_color_stack[stack_id].attenuation = scatter_record.attenuation;
        ray_color_stack[stack_id].scattering_pdf = scattering_pdf_value;
        ray_color_stack[stack_id].pdf_val = pdf_value;
    }
    
    // return vec3(0.0,0.0,0.0);
    return resolve_ray_color(stack_id, VEC3F_ZEROS);
}

fn resolve_ray_color(
    stack_last_index: i32,
    last_color: vec3f
) -> vec3f {
    var color = last_color;
    for (var i: i32 = stack_last_index; i >= 0; i--) {
        let entry = &ray_color_stack[i];
        if (*entry).skip_pdf {
            color *= (*entry).attenuation;
        } else {
            color = (*entry).color_from_emission + ((*entry).attenuation * (*entry).scattering_pdf * color) / (*entry).pdf_val;
        }
    }
    return color;
}

fn importance_pdf_value(ray: ptr<function, Ray>) -> f32 {
    // arrayLength 得到的长度不正确，可能是因为对齐
    // let len = arrayLength(&importance);
    let len = context.important_index_len;
    var pdf = 0.0;
    for (var i: u32 = 0; i < len; i++) {
        let primitive_type = importance[i].primitive_type;
        let primitive_id = importance[i].primitive_id;
        pdf += Primitive_pdf_value(primitive_type, primitive_id, ray);
    }
    return pdf / f32(len);
}

fn importance_random(origin: ptr<function, vec3f>) -> vec3f {
    // let len = arrayLength(&importance);
    let len = context.important_index_len;
    var i = randomi_range(0, i32(len - 1));
    let primitive_type = importance[i].primitive_type;
    let primitive_id = importance[i].primitive_id;
    return Primitive_random(primitive_type, primitive_id, origin);
}

/*-------------------------------------- Render Context -----------------------------------------*/

struct RenderContext {
    width: u32,
    height: u32,
    sample_position: vec2<u32>,
    pixel_origin: vec3f,
    samples_per_pixel: u32,
    pixel_delta_u: vec3f,
    sample_grid_num: u32,
    pixel_delta_v: vec3f,
    defocus_angle: f32,
    defocus_disk_u: vec3f,
    sample_grid_len: f32,
    defocus_disk_v: vec3f,
    sample_id: u32,
    camera_position: vec3f,
    ray_bounces: u32,
    important_index_len: u32,
    samples_per_frame: u32
}
/*------------------------------------------ BVH ------------------------------------------------*/

struct BoundingBox {
    xyz: array<Interval, 3>
}

fn BoundingBox_hit(
    box: BoundingBox,
    ray: ptr<function, Ray>,
    interval: Interval
) -> bool {
    var temp_interval = interval;

    var ad_inv = 1.0 / (*ray).direction[0];
    var t0 = (box.xyz[0].min - (*ray).origin[0]) * ad_inv;
    var t1 = (box.xyz[0].max - (*ray).origin[0]) * ad_inv;

    if !BoundingBox_hit_update_interval(t0, t1, &temp_interval) {
        return false;
    }

    ad_inv = 1.0 / (*ray).direction[1];
    t0 = (box.xyz[1].min - (*ray).origin[1]) * ad_inv;
    t1 = (box.xyz[1].max - (*ray).origin[1]) * ad_inv;

    if !BoundingBox_hit_update_interval(t0, t1, &temp_interval) {
        return false;
    }

    ad_inv = 1.0 / (*ray).direction[2];
    t0 = (box.xyz[2].min - (*ray).origin[2]) * ad_inv;
    t1 = (box.xyz[2].max - (*ray).origin[2]) * ad_inv;

    if !BoundingBox_hit_update_interval(t0, t1, &temp_interval) {
        return false;
    }

    return true;
}

fn BoundingBox_hit_update_interval(
    t0: f32,
    t1: f32,
    interval: ptr<function, Interval>
) -> bool {
    if t0 < t1 {
        if t0 > (*interval).min { (*interval).min = t0; }
        if t1 < (*interval).max { (*interval).max = t1; }
    } else {
        if t1 > (*interval).min { (*interval).min = t1; }
        if t0 < (*interval).max { (*interval).max = t0; }
    }
    if (*interval).max <= (*interval).min {
        return false;
    }
    return true;
}

// https://www.sci.utah.edu/~wald/Publications/2011/StackFree/sccg2011.pdf

struct BvhNode {
    left_or_primitive_type: u32,
    right_or_primitive_id: u32,
    parent: u32,
    is_leaf: u32,
    box: BoundingBox
}

const FROM_CHILD: u32 = 0;
const FROM_SIBLING: u32 = 1;
const FROM_PARENT: u32 = 2;

fn Bvh_hit(
    root: u32,
    ray: ptr<function, Ray>,
    interval: ptr<function, Interval>,
    hit_record: ptr<function, HitRecord>,
    debug_color: ptr<function, vec3f>
) -> bool {
    if !BoundingBox_hit(bvh_tree[root].box, ray, *interval) {
        return false;
    }

    var current = Bvh_left(root);
    var state = FROM_PARENT;
    var depth = 2.0;
    var max_depth = 2.0;
    var primitive_missed = false;
    let bounding_color = vec3f(0.02, 0.02, 0.02);
    let hit_color = vec3f(0.5, 0.0, 0.0);
    let miss_color = vec3f(0.0, 0.5, 0.0);
    loop {
        switch (state) {
            case FROM_CHILD: {
                if current == root {
                    *debug_color = max_depth * bounding_color;
                    if primitive_missed {
                         *debug_color += miss_color;
                    }
                    else if (*hit_record).hit {
                        *debug_color += hit_color;
                    }
                    return (*hit_record).hit; 
                }
                if current == Bvh_left(Bvh_parent(current)) {
                    current = Bvh_sibling(current);
                    state = FROM_SIBLING;
                } else {
                    depth -= 1.0;
                    current = Bvh_parent(current);
                    state = FROM_CHILD;
                }
            }
            case FROM_SIBLING: {
                if !BoundingBox_hit(bvh_tree[current].box, ray, *interval) {
                    depth -= 1.0;
                    current = Bvh_parent(current);
                    state = FROM_CHILD;
                } else if Bvh_is_leaf(current) {
                    let primitive_type = bvh_tree[current].left_or_primitive_type;
                    let primitive_id = bvh_tree[current].right_or_primitive_id;
                    primitive_missed = !Primitive_hit(primitive_type, primitive_id, ray, interval, hit_record);
                    depth -= 1.0;
                    current = Bvh_parent(current);
                    state = FROM_CHILD;
                } else {
                    depth += 1.0;
                    max_depth = max(max_depth, depth);
                    current = Bvh_left(current);
                    state = FROM_PARENT;
                }
            }
            case FROM_PARENT: {
                if !BoundingBox_hit(bvh_tree[current].box, ray, *interval) {
                    current = Bvh_sibling(current);
                    state = FROM_SIBLING;
                } else if (Bvh_is_leaf(current)) {
                    let primitive_type = bvh_tree[current].left_or_primitive_type;
                    let primitive_id = bvh_tree[current].right_or_primitive_id;
                    primitive_missed = !Primitive_hit(primitive_type, primitive_id, ray, interval, hit_record);
                    current = Bvh_sibling(current);
                    state = FROM_SIBLING;
                } else {
                    depth += 1.0;
                    max_depth = max(max_depth, depth);
                    current = Bvh_left(current);
                    state = FROM_PARENT;
                }
            }
            default: {
                return false;
            }
        }
    }
    return (*hit_record).hit;
}

fn Bvh_parent(id: u32) -> u32 {
    return bvh_tree[id].parent;
}

fn Bvh_is_leaf(id: u32) -> bool {
    return bvh_tree[id].is_leaf == 1;
}

fn Bvh_left(id: u32) -> u32 {
    return bvh_tree[id].left_or_primitive_type;
}

fn Bvh_sibling(id: u32) -> u32 {
    return bvh_tree[Bvh_parent(id)].right_or_primitive_id;
}

/*------------------------------------- Scatter Record ------------------------------------------*/

struct ScatterRecord {
    attenuation: vec3f,
    skip_pdf: bool,
    skip_pdf_ray: Ray,
}

/*---------------------------------------- Materials --------------------------------------------*/

fn Material_scatter(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
    scatter_record: ptr<function, ScatterRecord>
) -> bool {
    switch ((*hit_record).material_type) {
        case 1u: { // Lambertian
            return Lambertian_scatter(ray_in, hit_record, scatter_record);
        }
        case 3u: { // Dielectric
            return Dielectric_scatter(ray_in, hit_record, scatter_record);
        }
        default: {
            return false;
        }
    }
}

fn Material_scattering_pdf_value(
    ray: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> f32 {
    switch ((*hit_record).material_type) {
        case 1u: { // Lambertian
            return Lambertian_scattering_pdf_value(ray, hit_record);
        }
        default: {
            return 0.0;
        }
    }
}

fn Material_emit(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> vec3f {
    switch ((*hit_record).material_type) {
        case 0u: { // DebugNormal
            return DebugNormal_emit(ray_in, hit_record);
        }
        case 2u: { // Diffuse Light
            return DiffuseLight_emit(ray_in, hit_record);
        }
        default: {
            return vec3f(0.0, 0.0, 0.0);
        }
    }
}

fn Material_pdf_value(
    ray: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> f32 {
    switch ((*hit_record).material_type) {
        case 1u: { // Lambertian
            return Lambertian_pdf_value(ray, hit_record);
        }
        default: {
            return 0.0;
        }
    }
}

fn Material_random(
    origin: ptr<function, vec3f>,
    hit_record: ptr<function, HitRecord>,
) -> vec3f {
    switch ((*hit_record).material_type) {
        case 1u: { // Lambertian
            return Lambertian_random(origin, hit_record);
        }
        default: {
            return vec3f(0.0, 0.0, 0.0);
        }
    }
}

/*------------------------------------- DebugNormal Material ------------------------------------*/

fn DebugNormal_emit(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> vec3f {
    return (*hit_record).normal * 0.5 + 0.5;
}

/*------------------------------------- Lambertian Material -------------------------------------*/

struct Lambertian {
    albedo: vec3f
}

fn Lambertian_scatter(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
    scatter_record: ptr<function, ScatterRecord>
) -> bool {
    let lambertian = &lambertian_materials[(*hit_record).material_id];

    (*scatter_record).attenuation = (*lambertian).albedo;
    (*scatter_record).skip_pdf = false;

    return true;
}

fn Lambertian_pdf_value(
    ray: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> f32 {
    let cosine_theta = dot(normalize((*ray).direction), (*hit_record).normal);
    return max(0.0, cosine_theta / PI);
}

fn Lambertian_random(
    origin: ptr<function, vec3f>,
    hit_record: ptr<function, HitRecord>,
) -> vec3f {
    return rotation_matrix(VEC3F_UNIT_Y, (*hit_record).normal) * random_cosine_direction();
}

fn Lambertian_scattering_pdf_value(
    ray: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
) -> f32 {
    let cos_theta = dot((*hit_record).normal, normalize((*ray).direction));
    if cos_theta < 0 {
        return 0f;
    } else {
        return cos_theta / PI;
    }
}

fn random_cosine_direction() -> vec3f {
    let xi1 = randomf();
    let xi2 = randomf();

    let phi = 2 * PI * xi1;
    let z = cos(phi) * sqrt(xi2);
    let x = sin(phi) * sqrt(xi2);
    let y = sqrt(1 - xi2);

    return vec3f(x, y, z);
}

/*------------------------------------- Dielectric Material -------------------------------------*/

struct Dielectric {
    refraction_index: f32
}

fn Dielectric_scatter(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>,
    scatter_record: ptr<function, ScatterRecord>
) -> bool {
    let dielectric = &dielectric_materials[(*hit_record).material_id];

    (*scatter_record).attenuation = vec3f(1.0, 1.0, 1.0);
    (*scatter_record).skip_pdf = true;

    var refraction_index = (*dielectric).refraction_index;
    if (*hit_record).is_front_face {
        refraction_index = 1.0 / (*dielectric).refraction_index;
    }

    let in_direction = normalize((*ray_in).direction);
    let cos_theta = min(dot(-in_direction, (*hit_record).normal), 1.0);
    let sin_theta = sqrt(1 - pow(cos_theta, 2.0));

    let cannot_refract = refraction_index * sin_theta > 1.0;

    var out_direction: vec3f;
    if cannot_refract || Dielectric_reflectance((*hit_record).material_id, cos_theta, refraction_index) > randomf() {
        out_direction = reflect(in_direction, (*hit_record).normal);
    } else {
        out_direction = refract(in_direction, (*hit_record).normal, refraction_index);
    }

    (*scatter_record).skip_pdf_ray = Ray_init((*hit_record).position, out_direction);
    return true;
}

fn Dielectric_reflectance(
    material_id: u32,
    cosine: f32,
    refraction_index: f32
) -> f32 {
    let dielectric = &dielectric_materials[material_id];

    var r0 = (1 - (*dielectric).refraction_index) / (1 + (*dielectric).refraction_index);
    r0 = pow(r0, 2.0);
    return r0 + (1 - r0) * pow((1 - cosine), 5.0);
}

/*----------------------------------- Diffuse Light Material ------------------------------------*/

struct DiffuseLight {
    emit: vec3f
}

fn DiffuseLight_emit(
    ray_in: ptr<function, Ray>,
    hit_record: ptr<function, HitRecord>
) -> vec3f {
    if !(*hit_record).is_front_face {
        return VEC3F_ZEROS;
    }
    return diffuse_light_materials[(*hit_record).material_id].emit;
}

/*----------------------------------------- Primitive --------------------------------------------*/

struct PrimitiveIndex {
    primitive_type: u32,
    primitive_id: u32,
}

fn Primitive_hit(
    primitive_type: u32,
    primitive_id: u32,
    ray: ptr<function, Ray>,
    interval: ptr<function, Interval>,
    hit_record: ptr<function, HitRecord>,
) -> bool {
    var hit: bool;
    switch (primitive_type) {
        case 0u: { // Quad
            hit = Quad_hit(primitive_id, ray, interval, hit_record);
        }
        case 1u: { // Sphere
            hit = Sphere_hit(primitive_id, ray, interval, hit_record);
        }
        default: {
            return false;
        }
    }
    if hit {
        (*interval).max = (*hit_record).ray_t;
    }
    return hit;
}

fn Primitive_pdf_value(
    primitive_type: u32,
    primitive_id: u32,
    ray: ptr<function, Ray>,
) -> f32 {
    switch (primitive_type) {
        case 0u: { // Quad
            return Quad_pdf_value(primitive_id, ray);
        }
        case 1u: { // Sphere
            return Sphere_pdf_value(primitive_id, ray);
        }
        default: {
            return 0.0;
        }
    }
}

fn Primitive_random(
    primitive_type: u32,
    primitive_id: u32,
    origin: ptr<function, vec3f>
) -> vec3f {
    switch (primitive_type) {
        case 0u: { // Quad
            return Quad_random(primitive_id, origin);
        }
        case 1u: { // Sphere
            return Sphere_random(primitive_id, origin);
        }
        default: {
            return VEC3F_ZEROS;
        }
    }
}

/*------------------------------------------ Sphere ---------------------------------------------*/

struct Sphere {
    center: vec3f,
    radius: f32,
    material_type: u32,
    material_id: u32,
}

fn Sphere_hit(
    id: u32,
    ray: ptr<function, Ray>,
    interval: ptr<function, Interval>,
    hit_record: ptr<function, HitRecord>,
) -> bool {
    let sphere: ptr<storage, Sphere, read> = &spheres[id];

    let oc = (*sphere).center - (*ray).origin;
    let a = length_squared((*ray).direction);
    let h = dot((*ray).direction, oc);
    let c = length_squared(oc) - (*sphere).radius * (*sphere).radius;

    let discriminant = h * h - a * c;
    if discriminant < 0 {
        return false;
    }

    let sqrt_discriminant = sqrt(discriminant);

    // Find the nearest root that lies in the acceptable range.
    var root = (h - sqrt_discriminant) / a;
    if !Interval_surrounds(interval, root) {
        root = (h + sqrt_discriminant) / a;
        if !Interval_surrounds(interval, root) {
            return false;
        }
    }

    (*hit_record).hit = true;    
    (*hit_record).ray_t = root;
    (*hit_record).position = Ray_at(ray, root);

    let outward_normal = ((*hit_record).position - (*sphere).center) / (*sphere).radius;
    // let outward_normal = normalize((*hit_record).position - (*sphere).center);
    HitRecord_set_face_normal(hit_record, ray, outward_normal);
    (*hit_record).uv = Sphere_uv(outward_normal);
    (*hit_record).material_id = (*sphere).material_id;
    (*hit_record).material_type = (*sphere).material_type;

    return true;
}

fn Sphere_uv(position: vec3f) -> vec2f {
    // p: a given point on the sphere of radius one, centered at the origin.
    // u: returned value [0,1] of angle around the Y axis from X=-1.
    // v: returned value [0,1] of angle from Y=-1 to Y=+1.
    //     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
    //     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
    //     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>
    let theta = acos(-position.y);
    let phi = atan2(-position.z, position.x) + PI;
    return vec2f(phi / (2 * PI), theta / PI);
}

fn Sphere_pdf_value(
    id: u32,
    ray: ptr<function, Ray>,
) -> f32 {
    let sphere = &spheres[id];

    // This method only works for stationary spheres.
    var hit_record: HitRecord;
    var interval = Interval_init_2f(0.001, MAX);

    if !Sphere_hit(id, ray, &interval, &hit_record) {
        return 0.0;
    }

    let cp = (*sphere).center - (*ray).origin;
    let cos_theta_max = sqrt(1 - pow((*sphere).radius, 2.0)  / length_squared(cp));

    let solid_angle = 2 * PI * (1 - cos_theta_max);
    return 1.0 / solid_angle;
}

fn Sphere_random(
    id: u32,
    origin: ptr<function, vec3f>
) -> vec3f {
    let sphere = &spheres[id];
    var direction = (*sphere).center - *origin;
    let distance_squared = length_squared(direction);
    return rotation_matrix(VEC3F_UNIT_Y, normalize(direction)) * (random_to_sphere((*sphere).radius, distance_squared));
}

// Ray Tracing: The Rest of Your Life, p80
fn random_to_sphere(
    radius: f32,
    distance_squared: f32
) -> vec3f {
    let xi1 = randomf();
    let xi2 = randomf();

    let cos_theta_max = sqrt(1 - radius * radius / distance_squared);
    let y = 1 + xi2 * (cos_theta_max - 1);
    let phi = 2 * PI * xi1;
    let z = cos(phi) * sqrt(1 - y * y);
    let x = sin(phi) * sqrt(1 - y * y);

    return vec3f(x, y, z);
}

/*------------------------------------------- Quad ----------------------------------------------*/

struct Quad {
    bottom_left: vec3f,
    material_id: u32, // 放在这满足对齐，节省空间
    right: vec3f,
    area: f32,
    up: vec3f,
    d: f32,       // quad 所在平面的方程 ax + by + cz + d 中的 d
    normal: vec3f,
    material_type: u32,
    w: vec3f  // w 是将 quad 所在平面上的点转换到 quad 定义的坐标系（bottom_left, right, up）上时需要用到的变量
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

    (*hit_record).hit = true;
    (*hit_record).ray_t = t;
    (*hit_record).position = intersection;
    (*hit_record).material_id = (*quad).material_id;
    (*hit_record).material_type = (*quad).material_type;

    // 如果这里的第 3 个参数传入指针，就应该是 &quad.normal ，但这种写法要求支持 WGSL 扩展 unrestricted_pointer_parameters
    // https://www.w3.org/TR/WGSL/#language_extension-unrestricted_pointer_parameters
    // 浏览器是支持的，但 wgpu 没有在其他平台上实现。
    //
    // 在不支持的平台上在这里用指针会报一个奇怪的错：
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

    (*hit_record).uv = vec2<f32>(alpha, beta);

    return true;
}

fn Quad_pdf_value(
    id: u32,
    ray: ptr<function, Ray>,
) -> f32 {
    var hit_record: HitRecord;
    var interval = Interval_init_2f(0.001, MAX);

    if !Quad_hit(id, ray, &interval, &hit_record) {
        return 0.0;
    }

    let distance_squared = pow(hit_record.ray_t, 2.0) * length_squared((*ray).direction);
    let cosine = abs(dot((*ray).direction, hit_record.normal) / length((*ray).direction));

    return distance_squared / (cosine * quads[id].area);
}

fn Quad_random(
    id: u32,
    origin: ptr<function, vec3f>,
) -> vec3f {
    let quad = &quads[id];
    let p = (*quad).bottom_left + randomf() * (*quad).up + randomf() * (*quad).right;
    return normalize(p - *origin);
}

/*---------------------------------------- Hit Record -------------------------------------------*/

struct HitRecord {
    position: vec3f,
    ray_t: f32,
    normal: vec3f,
    material_id: u32,
    uv: vec2<f32>,
    material_type: u32,
    hit: bool,
    is_front_face: bool,
}

fn HitRecord_set_face_normal(
    s: ptr<function, HitRecord>,
    ray: ptr<function, Ray>,
    outward_normal: vec3f
) {
    (*s).is_front_face = (dot((*ray).direction, outward_normal) < 0);
    (*s).normal = outward_normal;
    if !(*s).is_front_face {
        (*s).normal = -outward_normal;
    }
}

/*----------------------------------------- Interval --------------------------------------------*/

struct Interval {
    min: f32,
    max: f32,
    _padding: array<u32, 2>
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

fn Interval_expand(s: ptr<function, Interval>, DELTA: f32) {
    let padding = DELTA / 2.0;

    (*s).min = (*s).min - padding;
    (*s).max = (*s).max + padding;
}

fn Interval_displace(s: ptr<function, Interval>, displacement: f32) {
    (*s).min = (*s).min + displacement;
    (*s).max = (*s).max + displacement;
}

/*--------------------------------------------- Ray ---------------------------------------------*/

struct Ray {
    origin: vec3f,
    direction: vec3f,
}

fn Ray_init(
    origin: vec3f,
    direction: vec3f,
) -> Ray {
    var ray: Ray;
    ray.origin = origin;
    ray.direction = direction;
    return ray;
}

fn Ray_at(s: ptr<function, Ray>, t: f32) -> vec3f {
    return (*s).origin + t * (*s).direction;
}

/*------------------------------------------- Sampling ------------------------------------------*/

fn sample_unit_disk() -> vec2<f32> {
    let phi = 2 * PI * randomf();
    let r = sqrt(randomf());
    return vec2<f32>(cos(phi), sin(phi)) * r;
}

fn sample_unit_square_stratified(sample_id: u32) -> vec2<f32> {
    if sample_id < context.sample_grid_num { // 分层采样
        return vec2<f32>(
            f32(context.sample_position.x) + randomf(),
            f32(context.sample_position.y) + randomf()
        ) * context.sample_grid_len - 0.5;
    } else { // 直接采样
        return sample_unit_square();
    }
}

fn sample_unit_square() -> vec2<f32> {
    return vec2<f32>(randomf(), randomf()) - 0.5;
}

/*---------------------------------- Random Number Generation -----------------------------------*/

// https://indico.cern.ch/event/93877/papers/2118070/files/4416-acat3.pdf

fn randomi_range(min: i32, max: i32) -> i32 {
    return i32(floor(randomf_range(f32(min), f32(max + 1))));
}

fn randomf_range(min: f32, max: f32) -> f32 {
    return min + (max - min) * randomf();
}

fn randomf() -> f32 {
    // Hybrid Tausworthe Generator:
    // Combined period is lcm(p1, p2, p3, p4) ~ 2^121
    random_state.z1 = taus_step(random_state.r, 13u, 19u, 12u, 4294967294u);  // p1 = 2^31-1
    random_state.z2 = taus_step(random_state.r, 2u, 25u, 4u, 4294967288u);    // p2 = 2^30-1
    random_state.z3 = taus_step(random_state.r, 3u, 11u, 17u, 4294967280u);   // p3 = 2^28-1
    random_state.z4 = lcg_step(random_state.r, 1664525u, 1013904223u);        // p4 = 2^32
    random_state.r = random_state.z1 ^ random_state.z2 ^ random_state.z3 ^ random_state.z4;

    var value = 2.3283064365387e-10f * f32(random_state.r); // [0, 1]
    return max(0.0, min(value - F32_POSITIVE_MIN, 1.0)); // [0, 1)
    // return value;
}

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

// https://gamedev.stackexchange.com/questions/92015/optimized-linear-to-srgb-glsl

fn linear_to_srgb(color: vec3f) -> vec3f {
    let cutoff = color.rgb < vec3(0.0031308);
    let higher = vec3(1.055) * pow(color.rgb, vec3(1.0 / 2.4)) - vec3(0.055);
    let lower = color.rgb * vec3(12.92);
    return select(higher, lower, cutoff);
}

fn srgb_to_linear(color: vec3f) -> vec3f {
    let cutoff = color.rgb < vec3(0.04045);
    let higher = pow((color.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
    let lower = color.rgb / vec3(12.92);
    return select(higher, lower, cutoff);
}

/*---------------------------- a Matrix to Rotate One Vector to Another -------------------------*/

// https://cs.brown.edu/people/jhughes/papers/Moller-EBA-1999/paper.pdf

fn rotation_matrix(unit_from: vec3f, unit_to: vec3f) -> mat3x3f { // require inputs are unit vectors
    let c = dot(unit_from, unit_to);
    if abs(c) <= 0.99 {
        let v = cross(unit_from, unit_to);
        // let h = (1 - c) / dot(v, v);
        let h = 1 / (1 + c);

        let hxx = h * v.x * v.x;
        let hxy = h * v.x * v.y;
        let hxz = h * v.x * v.z;
        let hyy = h * v.y * v.y;
        let hyz = h * v.y * v.z;
        let hzz = h * v.z * v.z;

        let c1 = vec3(c + hxx, hxy + v.z, hxz - v.y);
        let c2 = vec3(hxy - v.z, c + hyy, hyz + v.x);
        let c3 = vec3(hxz + v.y, hyz - v.x, c + hzz);

        return mat3x3(c1, c2, c3);
    } else {
        let xmin = unit_from.x < unit_from.y && unit_from.x < unit_from.z;
        let ymin = unit_from.y < unit_from.x && unit_from.y < unit_from.z;
        let p = select(select(VEC3F_UNIT_Z, VEC3F_UNIT_Y, ymin), VEC3F_UNIT_X, xmin);
        return reflection_matrix(p - unit_to) * reflection_matrix(p - unit_from);
    }
}

fn reflection_matrix(u: vec3f) -> mat3x3f {
    return MAT3X3F_IDENTITY - 2 / dot(u, u) * outer_product(u, u);
}

/*------------------------------------------- Vector --------------------------------------------*/

fn length_squared(x: vec3f) -> f32 {
    return x.x * x.x + x.y * x.y + x.z * x.z;
}

fn outer_product(a: vec3f, b: vec3f) -> mat3x3f {
    let c1 = b.x * a;
    let c2 = b.y * a;
    let c3 = b.z * a;
    return mat3x3f(c1, c2, c3);
}

fn reflect(v: vec3f, n: vec3f) -> vec3f {
    return v - 2 * dot(v, n) * n;
}

fn refract(uv: vec3f, n: vec3f, etai_over_etat: f32) -> vec3f {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -sqrt(abs(1.0 - length_squared(r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}