@compute @workgroup_size(16, 16)
fn compute_main(
    @builtin(global_invocation_id)
    gid : vec3<u32>
) {
    if gid.x >= context.width || gid.y >= context.height {
        return;
    }

    let pixel_index = gid.x + gid.y * context.width;
    init_random_generator(pixel_index);

    pixel_color[pixel_index][0] = random();
    pixel_color[pixel_index][1] = random();
    pixel_color[pixel_index][2] = random();
}

struct RayTracingContext {
    width: u32,
    height: u32
}
@group(0) @binding(0)
var<uniform> context: RayTracingContext;

@group(0) @binding(1)
var<storage, read_write> pixel_color: array<array<f32, 3>>;

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
    random_state.z1 = taus_step(random_state.r, 13u, 19u, 12u, 4294967294u);
    random_state.z2 = taus_step(random_state.r, 2u, 25u, 4u, 4294967288u);
    random_state.z3 = taus_step(random_state.r, 3u, 11u, 17u, 4294967280u);
    random_state.z4 = lcg_step(random_state.r, 1664525u, 1013904223u);
    random_state.r = random_state.z1 ^             // p1 = 2^31-1
                     random_state.z2 ^             // p2 = 2^30-1
                     random_state.z3 ^             // p3 = 2^28-1
                     random_state.z4;              // p4 = 2^32
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