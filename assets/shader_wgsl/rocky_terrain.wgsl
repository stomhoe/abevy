#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_shader_utils::fbm

@group(3) @binding(1) var<uniform> roughness: f32;
@group(3) @binding(2) var<uniform> scale: f32;
@group(3) @binding(3) var<uniform> height_scale: f32;
@group(3) @binding(4) var<uniform> color_base: vec4<f32>;
@group(3) @binding(5) var<uniform> color_shadow: vec4<f32>;

// --- Fragment ---
@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.world_position.xy * scale;
    
    // Use FBM for natural rocky terrain variation
    let fbm_value = fbm(uv, 5u, 2.0, 0.5);
    
    // Add roughness modulation using higher frequency noise
    let roughness_uv = uv * (1.0 + roughness * 5.0);
    let rough_detail = fbm(roughness_uv, 3u, 1.0, 0.6);
    
    // Combine base height with roughness detail
    let height = fbm_value * 0.7 + rough_detail * 0.3;
    
    // Create rocky appearance with height-based shading
    let shadow_amount = pow(height, 1.5) * height_scale;
    let color = mix(color_shadow, color_base, shadow_amount);
    
    // Add subtle cracks/ridges using derivative-like pattern
    let edge_detail = abs(fract(fbm_value * 4.0) - 0.5) * 2.0;
    let crack_sharpness = smoothstep(0.3, 0.7, edge_detail);
    let final_color = color * (0.85 + crack_sharpness * 0.15);
    
    return vec4<f32>(final_color.rgb, 1.0);
}
