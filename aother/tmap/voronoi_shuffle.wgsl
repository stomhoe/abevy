#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(1) var overlay_texture: texture_2d<f32>;
@group(3) @binding(2) var overlay_sampler: sampler;
@group(3) @binding(3) var<uniform> mask_color: vec4<f32>;
@group(3) @binding(4) var<uniform> scale: f32;
@group(3) @binding(5) var<uniform> voronoi_scale: f32;
@group(3) @binding(6) var<uniform> voronoi_scale_random: f32;   // strength of random scaling
@group(3) @binding(7) var<uniform> voronoi_rotation: f32;       // max rotation (in radians)

// --- Hash helpers ---
fn hash(x: u32) -> u32 {
    var v = x;
    v ^= v >> 16u;
    v *= 0x21f0aaadu;
    v ^= v >> 15u;
    v *= 0x735a2d97u;
    v *= v >> 15u;
    return v;
}

fn scale_uint(input: u32) -> f32 {
    return f32(input) / f32(0xffffffffu);
}

fn hashvec2(input: vec2<f32>) -> vec2<f32> {
    let hx = bitcast<u32>(input.x);
    let hy = bitcast<u32>(input.y);
    let h1 = hash(hash(hy) ^ hx);
    let h2 = hash(h1);
    return vec2<f32>(scale_uint(h1), scale_uint(h2));
}

fn hashvec2f(input: vec2<f32>) -> f32 {
    let hx = bitcast<u32>(input.x);
    let hy = bitcast<u32>(input.y);
    return scale_uint(hash(hash(hx) * hy));
}

// --- Voronoi ---
fn voronoi(point: vec2<f32>) -> vec2<f32> {
    let base_cell = floor(point);
    var min_dist = 1e9;
    var min_pos = vec2<f32>(0.0);

    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let cell = base_cell + vec2<f32>(f32(i), f32(j));
            let cell_pos = cell + hashvec2(cell);
            let to_cell = cell_pos - point;
            let dist = length(to_cell);

            let less = step(dist, min_dist);
            min_pos = cell_pos * less + min_pos * (1.0 - less);
            min_dist = min(dist, min_dist);
        }
    }
    return min_pos;
}

fn create_rotation_matrix(a: f32) -> mat2x2<f32> {
    let csin = sin(a);
    let ccos = cos(a);
    return mat2x2<f32>(
        vec2<f32>(ccos, csin),
        vec2<f32>(-csin, ccos)
    );
}

fn voronoi_uv(uv: vec2<f32>) -> vec2<f32> {
    let vpos = voronoi(uv * scale);
    var uvpos = (uv * scale) - vpos;

    // scale
    uvpos *= voronoi_scale;

    // random scale
    if (voronoi_scale_random > 0.0) {
        let random_scale = (hashvec2f(vpos + 1.618) - 0.5) * voronoi_scale_random + 1.0;
        uvpos *= random_scale;
    }

    // random rotation
    if (voronoi_rotation > 0.0) {
        let rotation_amt = hashvec2f(vpos) * voronoi_rotation;
        let rotation_matrix = create_rotation_matrix(rotation_amt);
        uvpos = rotation_matrix * uvpos;
    }

    return uvpos;
}

// --- Almost equal helper ---
fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

// --- Fragment ---
@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);

    let uv = in.world_position.xy;
    let warped_uv = voronoi_uv(uv);

    let tex_color = textureSample(overlay_texture, overlay_sampler, warped_uv);

    if almost_equal(color, mask_color, 0.00001) {
        return tex_color;
    }
    return color;
}
