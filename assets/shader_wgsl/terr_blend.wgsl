#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(1) var overlay_texture: texture_2d<f32>;
@group(3) @binding(2) var overlay_sampler: sampler;
@group(3) @binding(3) var<uniform> mask_color: vec4<f32>;
@group(3) @binding(4) var<uniform> scale: f32;
@group(3) @binding(5) var<uniform> time: f32;
@group(3) @binding(6) var<uniform> speed: f32;
@group(3) @binding(7) var<uniform> wavy_strength: f32;

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0) - 2.0 * f);
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    let ab = a + (b - a) * u.x;
    let cd = c + (d - c) * u.x;
    return ab + (cd - ab) * u.y;
}

fn fbm(p: vec2<f32>) -> f32 {
    var v: f32 = 0.0;
    var a: f32 = 0.5;
    var q: vec2<f32> = p;
    let shift = vec2<f32>(100.0, 100.0);
    for (var i: i32 = 0; i < 5; i = i + 1) {
        v = v + a * noise(q);
        q = q * 2.0 + shift;
        a = a * 0.5;
    }
    return v;
}

fn compute_water_offset(uv_world: vec2<f32>, t: f32, strength: f32) -> vec2<f32> {
    let wave1 = vec2<f32>(
        sin((uv_world.y * 6.0 + t * 1.6)),
        cos((uv_world.x * 5.0 - t * 1.2))
    ) * (0.008 * strength);
    let wave2 = vec2<f32>(
        sin((uv_world.x * 1.5 + t * 0.4)),
        sin((uv_world.y * 1.2 - t * 0.5))
    ) * (0.012 * strength);
    let n = fbm(uv_world * 0.8 + t * 0.15) * (0.02 * strength);
    let n2 = fbm(uv_world * 2.5 - t * 0.12) * (0.01 * strength);
    return wave1 + wave2 + vec2<f32>(n + n2, n - n2);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base = process_fragment(in);

    if !almost_equal(base, mask_color, 0.00001) {
        return base;
    }

    let repeat_scale = max(scale, 1e-5);
    let uv_world = in.world_position.xy * repeat_scale / 10000.0;
    let uv = fract(uv_world);
    let t = time * max(speed, 0.0);
    let strength = max(wavy_strength, 0.0);
    let offset = select(vec2<f32>(0.0, 0.0), compute_water_offset(uv_world, t, strength), strength > 0.0);
    var sample_uv = fract(uv + offset);

    if strength > 0.0 {
        sample_uv = sample_uv + 0.001 * vec2<f32>(
            sin(dot(sample_uv, vec2<f32>(12.9898, 78.233)) + t),
            cos(dot(sample_uv, vec2<f32>(93.9898, 67.345)) - t * 0.8)
        );
    }

    var tex_color = textureSample(overlay_texture, overlay_sampler, sample_uv);

    if strength > 0.0 {
        let shimmer = 0.03 * strength * fbm(uv_world * 1.8 + t * 0.6);
        tex_color = vec4<f32>(tex_color.rgb + vec3<f32>(shimmer), tex_color.a);
    }

    return vec4<f32>(tex_color.rgb, tex_color.a * base.a);
}
