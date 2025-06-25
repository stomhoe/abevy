#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(1) var overlay_texture: texture_2d<f32>;
@group(3) @binding(2) var overlay_sampler: sampler;
@group(3) @binding(3) var<uniform> mask_color: vec4<f32>;
@group(3) @binding(4) var<uniform> scale: f32;

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);
    let tex_color = textureSample(overlay_texture, overlay_sampler, fract(in.world_position.xy * scale));
    if almost_equal(color, mask_color, 0.01) {
        return tex_color;
    }
    return color;
}
