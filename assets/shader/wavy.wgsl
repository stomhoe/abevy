#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

// --- Uniforms (kept compatible) ---
@group(3) @binding(1) var<uniform> scale: f32;
@group(3) @binding(2) var<uniform> time: f32;
@group(3) @binding(3) var<uniform> speed: vec2<f32>;
@group(3) @binding(4) var<uniform> amplitude: f32; // global wave amplitude
@group(3) @binding(5) var<uniform> wave_color: vec4<f32>; // rgb = shallow color, a = strength
@group(3) @binding(6) var<uniform> cell_scale: f32; // larger -> bigger Worley cells (cross tiles)
@group(3) @binding(7) var<uniform> seam_strength: f32; // world-space seam suppression
@group(3) @binding(8) var<uniform> highlight_strength: f32; // bright-line intensity
@group(3) @binding(9) var<uniform> warp_strength: f32; // local distortion strength
@group(3) @binding(10) var<uniform> flow_speed: f32; // advection speed multiplier

// --- Small math helpers ---
fn lerp(a: f32, b: f32, t: f32) -> f32 { return a + (b - a) * t; }
fn lerp3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> { return a + (b - a) * t; }
fn clamp01(x: f32) -> f32 { return clamp(x, 0.0, 1.0); }

// normalize colors if author used 0..255 values (heuristic)
fn normalize_color(c: vec4<f32>) -> vec4<f32> {
    if (max(max(c.x, c.y), max(c.z, c.w)) > 2.0) {
        return c / 255.0;
    }
    return c;
}

fn luma(col: vec3<f32>) -> f32 { return dot(col, vec3<f32>(0.2126, 0.7152, 0.0722)); }

// --- Value / gradient noise (kept simple & cheap) ---
fn hash01(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn hash2(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(hash01(p), hash01(p + vec2<f32>(1.2345, 7.8901)));
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash01(i + vec2<f32>(0.0, 0.0));
    let b = hash01(i + vec2<f32>(1.0, 0.0));
    let c = hash01(i + vec2<f32>(0.0, 1.0));
    let d = hash01(i + vec2<f32>(1.0, 1.0));
    return lerp(lerp(a, b, u.x), lerp(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var sum: f32 = 0.0;
    var amp: f32 = 0.6;
    var freq: f32 = 1.0;
    var i: u32 = 0u;
    loop {
        if (i >= 5u) { break; }
        sum = sum + amp * noise(p * freq);
        freq = freq * 2.0;
        amp = amp * 0.5;
        i = i + 1u;
    }
    return sum;
}

// --- Cellular (Worley) noise: returns (F1, F2) distances ---
fn worley(p: vec2<f32>) -> vec2<f32> {
    let i = floor(p);
    let f = fract(p);
    var minDist: f32 = 1e9;
    var secondMin: f32 = 1e9;

    // examine 3x3 neighborhood
    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let cell = i + vec2<f32>(f32(x), f32(y));
            let r = hash2(cell);
            let feature = vec2<f32>(f.x + r.x - f32(x), f.y + r.y - f32(y));
            let d = length(feature);
            if (d < minDist) {
                secondMin = minDist;
                minDist = d;
            } else if (d < secondMin) {
                secondMin = d;
            }
        }
    }
    return vec2<f32>(minDist, secondMin);
}

// --- Wave / water height (combined sin layers, fbm & cellular bumps) ---
fn waveHeight(p: vec2<f32>) -> f32 {
    // base directional swells
    let swell = sin(dot(p, vec2<f32>(3.0, 1.5)) + time * 0.8) * 0.6
              + sin(p.x * 2.2 + time * 1.1) * 0.4
              + sin(p.y * 1.6 + time * 0.9) * 0.3;

    // frilly detail
    let detail = (fbm(p * 1.5 + time * 0.05) - 0.5) * 0.7;

    // cellular (Worley) contribution makes patchy whitecaps and local turbulence
    // use a smaller multiplier so Worley cells are larger and span multiple tiles
    let w = worley(p * 0.12);
    let cellular_ridge = pow(clamp01(1.0 - w.x * 1.6), 2.0) * 0.9; // strong near feature points

    return (swell + detail + cellular_ridge * 0.7) * amplitude;
}

// --- compute normal via finite difference ---
fn computeNormal(p: vec2<f32>) -> vec3<f32> {
    let h = waveHeight(p);
    let eps = 0.0008;
    let hdx = waveHeight(p + vec2<f32>(eps, 0.0)) - h;
    let hdy = waveHeight(p + vec2<f32>(0.0, eps)) - h;
    return normalize(vec3<f32>(-hdx, -hdy, 1.0));
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    // base texture and coordinates
    let base = process_fragment(in);
    let world_pos = in.world_position.xy * scale;

    // advection / flow: combine global speed + noise-derived local flow
    let base_flow = speed * flow_speed * 0.6;

    // noise-derived local flow (two-band FBM used as a vector field)
    let n_x = fbm(world_pos * 0.4 + vec2<f32>(time * 0.08, -time * 0.04));
    let n_y = fbm(world_pos * 0.4 + vec2<f32>(-time * 0.06, time * 0.09));
    let local_flow = (vec2<f32>(n_x, n_y) * 2.0 - vec2<f32>(1.0, 1.0)) * 0.8;

    // curl-ish component to generate eddies
    let curl = vec2<f32>(local_flow.y, -local_flow.x) * 0.5;

    // composite displacement for domain warping (animated, local)
    let warp = (local_flow + curl) * warp_strength;

    // animated sample position (flow + warp) - advect over time for motion
    let p = world_pos + base_flow * time * 0.04 + warp;

    // add per-region jitter + rotation to decorrelate tile-aligned artifacts
    let region_seed = hash01(floor(world_pos * 0.32) + vec2<f32>(12.345, 45.678));
    let region_angle = (region_seed - 0.5) * 6.283185 * 0.8;
    let rot_angle = sin(dot(p, vec2<f32>(12.9898, 78.233)) + time * 0.07) * 0.9 + region_angle * 0.35;
    let c = cos(rot_angle); let s = sin(rot_angle);
    fn rot(v: vec2<f32>, c: f32, s: f32) -> vec2<f32> { return vec2<f32>(v.x * c - v.y * s, v.x * s + v.y * c); }
    let jitter_seed = hash01(floor(p * 0.25));
    let jitter = (jitter_seed - 0.5) * 0.7 * (1.0 / max(cell_scale, 0.001));
    let anim_base = rot(p, c, s) + vec2<f32>(jitter, jitter) + vec2<f32>(sin(time * 0.18), cos(time * 0.13)) * 0.14;

    // palette and base colors
    let wc = normalize_color(wave_color);
    let deep = vec3<f32>(0.00, 0.04, 0.16);
    let shallow = (wc.rgb * 0.9) + vec3<f32>(0.02, 0.05, 0.06);

    // multi-scale Worley computed on warped coords to break tile alignment (use warped positions)
    let base1 = anim_base * cell_scale;
    let base2 = (anim_base + warp * 0.28) * (cell_scale * 2.2);

    // randomized small-kernel blur: offsets vary per-region so lines don't line up across tiles
    let off = 0.6 / max(cell_scale, 0.001);
    let a_ang = hash01(floor(anim_base * 0.2)) * 6.283185;
    let b_ang = hash01(floor(anim_base * 0.37) + vec2<f32>(3.21, 6.54)) * 6.283185;
    let off1 = vec2<f32>(cos(a_ang), sin(a_ang)) * off;
    let off2 = vec2<f32>(cos(b_ang), sin(b_ang)) * off * 1.2;

    let r00 = worley(base1);
    let r01 = worley(base1 + off1);
    let r02 = worley(base1 + off2);
    let ridge1 = clamp01(((r00.y - r00.x) + (r01.y - r01.x) + (r02.y - r02.x)) / 3.0 * 7.8);

    let s00 = worley(base2);
    let s01 = worley(base2 + off2 * 1.1);
    let s02 = worley(base2 + off1 * 1.3);
    let ridge2 = clamp01(((s00.y - s00.x) + (s01.y - s01.x) + (s02.y - s02.x)) / 3.0 * 8.8) * 0.52;

    let ridge = clamp01(ridge1 + ridge2);
    // soften contrast so highlights remain delicate
    let caustic = pow(ridge, 1.08);

    // micro-detail (higher frequency FBM) for crackling
    let micro = fbm((anim_base + warp * 0.12) * (cell_scale * 7.8) + time * 0.14);

    // final highlights (thinner, denser lines) modulated by micro detail and highlight strength
    // reduce peakiness and vary by region to prevent uniform straight streaks
    let bright = clamp01(caustic * (0.38 + micro * 0.55) * highlight_strength * (1.0 - jitter_seed * 0.35));

    // depth color modulated by micro detail
    let depth_mix = clamp01(0.25 + micro * 0.25);
    let water_col = lerp3(deep, shallow, depth_mix);

    // subtle micro-specular using fbm derivatives
    let dh = 0.001;
    let base_h = fbm(anim_p * (cell_scale * 2.2));
    let dhdx = fbm((anim_p + vec2<f32>(dh, 0.0)) * (cell_scale * 2.2)) - base_h;
    let dhdy = fbm((anim_p + vec2<f32>(0.0, dh)) * (cell_scale * 2.2)) - base_h;
    let normal = normalize(vec3<f32>(-dhdx, -dhdy, 1.0));
    let light_dir = normalize(vec3<f32>(-0.2, 0.5, 0.85));
    let spec = pow(clamp(dot(reflect(-light_dir, normal), vec3<f32>(0.0, 0.0, 1.0)), 0.0, 1.0), 40.0) * 0.12;
    let shaded = water_col * (0.26 + spec);

    // seam suppression using world-space low-frequency noise
    let base_gray = vec3<f32>(luma(base.rgb));
    let base_desat = lerp3(base_gray, base.rgb, 0.12);
    let seam = clamp01(seam_strength * fbm(world_pos * 0.12 + time * 0.02));
    let base_dark = base_desat * (0.55 + 0.45 * seam);

    // compose final color: prefer shaded water, then add narrow caustic highlights (tinted & muted)
    let tint_strength = clamp01(wc.a);
    var color_mix = lerp3(base_dark * 0.10, shaded, tint_strength);
    // tint highlights toward water color to avoid pure white streaks
    let highlight_color = normalize(lerp(vec3<f32>(0.9,0.97,1.0), water_col + vec3<f32>(0.12,0.14,0.16), 0.9));
    // subtle depth attenuation so highlights fade in deeper areas
    let depth_att = 1.0 - clamp01(depth_mix * 0.6);
    // blend highlights gently and tint by water tone; lower peak intensity
    color_mix = mix(color_mix, color_mix + highlight_color * 0.42 * depth_att, bright * 0.48);
    // soften midtones and add micro detail for natural look
    color_mix = color_mix * (0.96 + 0.04 * micro);

    // final clamp
    let final_color = clamp(color_mix, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(final_color, base.a);
} 
