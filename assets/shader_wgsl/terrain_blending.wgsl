#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

@group(3) @binding(1) var texture_a: texture_2d<f32>;
@group(3) @binding(2) var sampler_a: sampler;
@group(3) @binding(3) var texture_b: texture_2d<f32>;
@group(3) @binding(4) var sampler_b: sampler;
@group(3) @binding(5) var<uniform> mask_color: vec4<f32>;
@group(3) @binding(6) var<uniform> scale_a: f32;
@group(3) @binding(7) var<uniform> scale_b: f32;
@group(3) @binding(8) var<uniform> blend_sharpness: f32;

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let base = process_fragment(in);
    if !almost_equal(base, mask_color, 0.00001) {
        return base;
    }

    let uv_a = fract(in.world_position.xy * scale_a / 10000.0);
    let uv_b = fract(in.world_position.xy * scale_b / 10000.0);
    let col_a = textureSample(texture_a, sampler_a, uv_a);
    let col_b = textureSample(texture_b, sampler_b, uv_b);

    var blend = clamp(base.r, 0.0, 1.0);
    if blend_sharpness > 0.0 {
        let edge = mix(0.5, 0.15, clamp(blend_sharpness, 0.0, 1.0));
        blend = smoothstep(0.5 - edge, 0.5 + edge, blend);
    }

    let out_rgb = mix(col_a.rgb, col_b.rgb, blend);
    let out_a = mix(col_a.a, col_b.a, blend) * base.a;
    return vec4<f32>(out_rgb, out_a);
}
