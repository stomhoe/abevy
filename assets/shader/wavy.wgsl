#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(0) var texture_overlay: texture_2d<f32>;
@group(3) @binding(1) var overlay_sampler: sampler;
@group(3) @binding(2) var<uniform> mask_color: vec4<f32>;
@group(3) @binding(3) var<uniform> scale: f32;
@group(3) @binding(4) var<uniform> time: f32;
@group(3) @binding(5) var<uniform> speed: f32;
@group(3) @binding(6) var<uniform> debug_mode: f32;

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}





// --- helper noise + fbm for organic-looking distortion ---
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
    var shift = vec2<f32>(100.0, 100.0);
    var q: vec2<f32> = p;
    for (var i: i32 = 0; i < 5; i = i + 1) {
        v = v + a * noise(q);
        q = q * 2.0 + shift;
        a = a * 0.5;
    }
    return v;
}

// compute a small, layered offset based on world position and time
fn compute_water_offset(uv_world: vec2<f32>, t: f32) -> vec2<f32> {
    // multi-frequency sine waves (fast fine ripples + slow broad swells)
    let wave1 = vec2<f32>(
        sin((uv_world.y * 6.0 + t * 1.6)),
        cos((uv_world.x * 5.0 - t * 1.2))
    ) * 0.008;
    let wave2 = vec2<f32>(
        sin((uv_world.x * 1.5 + t * 0.4)),
        sin((uv_world.y * 1.2 - t * 0.5))
    ) * 0.012;

    // organic turbulence from fbm
    let n = fbm(uv_world * 0.8 + t * 0.15) * 0.02;
    let n2 = fbm(uv_world * 2.5 - t * 0.12) * 0.01;

    return wave1 + wave2 + vec2<f32>(n + n2, n - n2);
}

// enhanced fragment: distort overlay sampling and add subtle sheen
@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base = process_fragment(in);

    // normalized tile-space UV
    let uv_world = in.world_position.xy * scale;
    let uv = fract(uv_world);

    // time-driven offset
    let t = time * speed;
    let offset = compute_water_offset(uv_world, t);

    // sample distorted overlay
    var overlay_uv = fract(uv + offset);

    // sub-pixel jitter helps avoid tiling seams
    overlay_uv = overlay_uv + 0.001 * vec2<f32>(
        sin(dot(overlay_uv, vec2<f32>(12.9898, 78.233)) + t),
        cos(dot(overlay_uv, vec2<f32>(93.9898, 67.345)) - t * 0.8)
    );

    var tex = textureSample(texture_overlay, overlay_sampler, overlay_uv);

    // subtle animated highlight (makes water look alive)
    let shimmer = 0.03 * fbm(uv_world * 1.8 + t * 0.6);
    tex = vec4<f32>(tex.rgb + vec3<f32>(shimmer), tex.a);

    // slight chromatic separation on faster ripples
    let sep = 0.003 * sin(t * 2.0 + uv_world.x * 3.0);
    let r = textureSample(texture_overlay, overlay_sampler, fract(overlay_uv + vec2<f32>(sep, 0.0))).r;
    let g = textureSample(texture_overlay, overlay_sampler, fract(overlay_uv)).g;
    let b = textureSample(texture_overlay, overlay_sampler, fract(overlay_uv - vec2<f32>(sep, 0.0))).b;
    tex = vec4<f32>(vec3<f32>(r, g, b), tex.a);

    // debug visualization: show flow vector as color
    if (debug_mode > 0.5) {
        let flow_vis = (normalize(vec3<f32>(offset, 0.0)).xyz * 0.5) + vec3<f32>(0.5);
        return vec4<f32>(flow_vis, 1.0);
    }

    // mask handling: return distorted overlay where mask matches
    if (almost_equal(base, mask_color / vec4<f32>(255.0), 0.00001)) {
        return tex;
    }

    return base;
}