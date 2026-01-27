#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(1) var<uniform> scale: f32;
@group(3) @binding(2) var<uniform> time: f32;
@group(3) @binding(3) var<uniform> speed: f32;
@group(3) @binding(4) var<uniform> amplitude: f32;
@group(3) @binding(5) var<uniform> wave_color: vec4<f32>;

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

// Simple hashing and smooth noise (value noise + FBM)
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0, 3.0) - 2.0 * f);
    let a = hash(i + vec2<f32>(0.0, 0.0));
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var v: f32 = 0.0;
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var pp: vec2<f32> = p;
    for (var i: i32 = 0; i < 5; i = i + 1) {
        v = v + amp * noise(pp * freq);
        freq = freq * 2.0;
        amp = amp * 0.5;
    }
    return v;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base = process_fragment(in);

    let pos = in.world_position.xy * scale;
    let n = fbm(pos + vec2<f32>(time * speed, time * speed * 0.4));
    let alpha = clamp(n * amplitude, 0.0, 1.0);
    let overlay = vec4<f32>(wave_color.rgb, wave_color.a * alpha);

    // Blend RGB by overlay alpha but preserve visibility by keeping max alpha
    let blended_rgb = mix(base.rgb, overlay.rgb, overlay.a);
    let out_alpha = min(base.a, overlay.a);
    return vec4<f32>(blended_rgb, out_alpha);
}
