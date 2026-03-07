#import bevy_ecs_tilemap::common::{sprite_texture, sprite_sampler, tilemap_data}
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

// Packed tile index texture (base + overlay indices encoded as two u16 pairs).
@group(3) @binding(1) var tile_indices_map: texture_2d<f32>;
// Packed tile flags texture (bitfield: has_params, blend_enabled, has_overlay).
@group(3) @binding(2) var tile_flags_map: texture_2d<f32>;
// Per-tile overlay animation params (scale, speed, wave strength, time offset).
@group(3) @binding(3) var tile_params_map: texture_2d<f32>;
// Map size in tiles, used for bounds checks when sampling neighbors.
@group(3) @binding(4) var<uniform> map_size_tiles: vec2<f32>;
// Global shader time used for animated overlay sampling.
@group(3) @binding(5) var<uniform> time: f32;
// Overlay texture atlases/slices addressed by overlay_tex_index.
@group(3) @binding(6) var overlay_tex_0: texture_2d<f32>;
@group(3) @binding(7) var overlay_tex_1: texture_2d<f32>;
@group(3) @binding(8) var overlay_tex_2: texture_2d<f32>;
@group(3) @binding(9) var overlay_tex_3: texture_2d<f32>;
@group(3) @binding(10) var overlay_tex_4: texture_2d<f32>;
@group(3) @binding(11) var overlay_tex_5: texture_2d<f32>;
@group(3) @binding(12) var overlay_tex_6: texture_2d<f32>;
@group(3) @binding(13) var overlay_tex_7: texture_2d<f32>;

// Base sprite color that marks pixels where overlay composition is allowed.
const MASK_COLOR = vec3<f32>(1.0, 0.0, 0.0);
// Tolerance around MASK_COLOR to tolerate texture filtering at tile borders.
const MASK_TOLERANCE = 0.12;

struct TileData {
    // Index into sprite array/layer for the tile base texture.
    base_tex_index: u32,
    // Index selecting one of overlay_tex_0..overlay_tex_7.
    overlay_tex_index: u32,
    // Whether tile_params_map contains valid custom params for this tile.
    has_params: bool,
    // Whether this tile participates in neighbor blending logic.
    blend_enabled: bool,
    // Whether this tile has an overlay at all.
    has_overlay: bool,
    // Overlay world-space tiling scale.
    scale: f32,
    // Overlay animation speed multiplier.
    speed: f32,
    // Overlay wave intensity.
    wavy_strength: f32,
    // Per-tile time phase offset.
    time_offset: f32,
};

// Decode helpers for packed tile metadata textures.

// Rebuild a u16 from two normalized [0..1] channels.
fn decode_u16(low: f32, high: f32) -> u32 {
    let lo = u32(round(low * 255.0));
    let hi = u32(round(high * 255.0));
    return lo + (hi << 8u);
}

// Decode a one-byte bitfield from a normalized channel.
fn decode_flags(v: f32) -> u32 {
    return u32(round(v * 255.0));
}

// Check if tile coordinates are inside the global map texture bounds.
fn in_bounds(tile: vec2<i32>) -> bool {
    return tile.x >= 0
        && tile.y >= 0
        && tile.x < i32(map_size_tiles.x)
        && tile.y < i32(map_size_tiles.y);
}

// Read and decode all per-tile data from packed metadata textures.
fn read_tile_data(tile: vec2<i32>) -> TileData {
    if !in_bounds(tile) {
        return TileData(0u, 0u, false, false, false, 0.0, 0.0, 0.0, 0.0);
    }

    let tex = textureLoad(tile_indices_map, tile, 0);
    let flags_raw = decode_flags(textureLoad(tile_flags_map, tile, 0).r);
    let params = textureLoad(tile_params_map, tile, 0);
    return TileData(
        decode_u16(tex.r, tex.g),
        decode_u16(tex.b, tex.a),
        (flags_raw & 1u) != 0u,
        (flags_raw & 2u) != 0u,
        (flags_raw & 4u) != 0u,
        params.r,
        params.g,
        params.b,
        params.a,
    );
}

// Procedural water animation offset for overlay UVs.
// Compute animated UV offset used for water-like overlay movement.
fn compute_water_offset(uv_world: vec2<f32>, t: f32, strength: f32) -> vec2<f32> {
    let wave1 = vec2<f32>(
        sin((uv_world.y * 6.0 + t * 1.6)),
        cos((uv_world.x * 5.0 - t * 1.2))
    ) * (0.008 * strength);
    let wave2 = vec2<f32>(
        sin((uv_world.x * 1.5 + t * 0.4)),
        sin((uv_world.y * 1.2 - t * 0.5))
    ) * (0.012 * strength);
    return wave1 + wave2;
}

// Select the overlay texture by index and sample at UV.
fn sample_overlay_texture(index: u32, uv: vec2<f32>) -> vec4<f32> {
    switch index {
        case 0u: { return textureSample(overlay_tex_0, sprite_sampler, uv); }
        case 1u: { return textureSample(overlay_tex_1, sprite_sampler, uv); }
        case 2u: { return textureSample(overlay_tex_2, sprite_sampler, uv); }
        case 3u: { return textureSample(overlay_tex_3, sprite_sampler, uv); }
        case 4u: { return textureSample(overlay_tex_4, sprite_sampler, uv); }
        case 5u: { return textureSample(overlay_tex_5, sprite_sampler, uv); }
        case 6u: { return textureSample(overlay_tex_6, sprite_sampler, uv); }
        case 7u: { return textureSample(overlay_tex_7, sprite_sampler, uv); }
        default: { return vec4<f32>(1.0, 0.0, 1.0, 1.0); }
    }
}

// Sample only a tile's overlay (without base), with animation and wrap.
fn sample_overlay_only(data: TileData, world_uv: vec2<f32>) -> vec4<f32> {
    // No overlay configured for this tile: contribute nothing.
    if !data.has_overlay {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let repeat_scale = max(data.scale, 1e-5);
    let uv_world = world_uv * repeat_scale / 10000.0;
    let t = (time + data.time_offset) * max(data.speed, 0.0);
    let strength = max(data.wavy_strength, 0.0);
    let offset = select(vec2<f32>(0.0, 0.0), compute_water_offset(uv_world, t, strength), strength > 0.0);
    var sample_uv = fract(fract(uv_world) + offset);

    // Apply tiny extra perturbation only when animated waves are enabled.
    if strength > 0.0 {
        sample_uv = sample_uv + 0.001 * vec2<f32>(
            sin(dot(sample_uv, vec2<f32>(12.9898, 78.233)) + t),
            cos(dot(sample_uv, vec2<f32>(93.9898, 67.345)) - t * 0.8)
        );
    }

    return sample_overlay_texture(data.overlay_tex_index, sample_uv);
}

// Compose base tile texture with this tile's own overlay where mask permits.
fn sample_tile_color(tile: vec2<i32>, uv: vec2<f32>, world_uv: vec2<f32>, tint: vec4<f32>) -> vec4<f32> {
    let data = read_tile_data(tile);
    let base = textureSample(sprite_texture, sprite_sampler, uv, i32(data.base_tex_index)) * tint;
    // Allow a small tolerance to survive filtering at tile borders.
    let is_mask = distance(base.rgb, MASK_COLOR) <= MASK_TOLERANCE;
    // If pixel is not the mask color, keep original base color unchanged.
    if !is_mask {
        return base;
    }
    // Mask matched but tile has no overlay: keep base color unchanged.
    if !data.has_overlay {
        return base;
    }
    let overlay = sample_overlay_only(data, world_uv);
    let composed_rgb = mix(base.rgb, overlay.rgb, clamp(overlay.a, 0.0, 1.0));
    return vec4<f32>(composed_rgb, base.a);
}

// Read contribution from one neighbor tile, respecting ownership and overlay rules.
fn sample_neighbor_contribution(
    tile: vec2<i32>,
    world_uv: vec2<f32>,
    self_data: TileData,
    self_color: vec4<f32>
) -> vec4<f32> {
    // Return neighbor-driven contribution only when this tile is the submissive one.
    let neighbor_data = read_tile_data(tile);
    // Neighbor opted out of blending globally.
    if !neighbor_data.blend_enabled {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // Both sides must have overlays to mix meaningful border color.
    if !self_data.has_overlay || !neighbor_data.has_overlay {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // Blend only across borders where overlay differs.
    // Same overlay type: no visible transition needed.
    if neighbor_data.overlay_tex_index == self_data.overlay_tex_index {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // One-sided ownership: only the higher overlay index (submissive) blends inward from lower (dominant).
    // Ownership rule rejects this direction (prevents two-sided painting).
    if self_data.overlay_tex_index <= neighbor_data.overlay_tex_index {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let neighbor_overlay = sample_overlay_only(neighbor_data, world_uv);
    let overlay_alpha = clamp(neighbor_overlay.a, 0.0, 1.0);
    // Fully transparent sampled neighbor overlay: no contribution.
    if overlay_alpha <= 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let neighbor_rgb = mix(self_color.rgb, neighbor_overlay.rgb, overlay_alpha);
    return vec4<f32>(neighbor_rgb, 1.0);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    // Raw tile coordinate from vertex output (storage-space origin).
    let raw_tile_pos = vec2<i32>(
        i32(in.storage_position.x),
        i32(in.storage_position.y),
    );
    // Absolute tile coordinate in map-space.
    let tile_pos_i = raw_tile_pos + vec2<i32>(
        i32(tilemap_data.chunk_pos.x),
        i32(tilemap_data.chunk_pos.y),
    );
    // Local UV inside current tile (0..1).
    let uv = in.uv.xy;
    // World-space position used for consistent cross-tile overlay sampling.
    let world_uv = in.world_position.xy;
    // Metadata for the current tile.
    let tile_data = read_tile_data(tile_pos_i);

    // Start from this tile's own composed color.
    var out = sample_tile_color(tile_pos_i, uv, world_uv, in.color);
    if out.a < 0.001 {
        discard;
    }

    if !tile_data.blend_enabled {
        return out;
    }

    // Neighbor color candidates for the 8 directions around this tile.
    // Each one is pre-filtered by ownership and compatibility rules.
    var north = sample_neighbor_contribution(tile_pos_i + vec2<i32>(0, 1), world_uv, tile_data, out);
    var north_east = sample_neighbor_contribution(tile_pos_i + vec2<i32>(1, 1), world_uv, tile_data, out);
    var east = sample_neighbor_contribution(tile_pos_i + vec2<i32>(1, 0), world_uv, tile_data, out);
    var south_east = sample_neighbor_contribution(tile_pos_i + vec2<i32>(1, -1), world_uv, tile_data, out);
    var south = sample_neighbor_contribution(tile_pos_i + vec2<i32>(0, -1), world_uv, tile_data, out);
    var south_west = sample_neighbor_contribution(tile_pos_i + vec2<i32>(-1, -1), world_uv, tile_data, out);
    var west = sample_neighbor_contribution(tile_pos_i + vec2<i32>(-1, 0), world_uv, tile_data, out);
    var north_west = sample_neighbor_contribution(tile_pos_i + vec2<i32>(-1, 1), world_uv, tile_data, out);

    let border_width = 0.24;
    // Directional masks in tile UV space (higher near each side).
    let n_axis = smoothstep(1.0 - border_width, 1.0, 1.0 - uv.y);
    let s_axis = smoothstep(1.0 - border_width, 1.0, uv.y);
    let e_axis = smoothstep(1.0 - border_width, 1.0, uv.x);
    let w_axis = smoothstep(1.0 - border_width, 1.0, 1.0 - uv.x);

    // Compact directional weights: cardinals on edges, diagonals in corners.
    let w_n = n_axis;
    let w_ne = min(n_axis, e_axis) * uv.x;
    let w_e = e_axis;
    let w_se = min(s_axis, e_axis);
    let w_s = s_axis;
    let w_sw = min(s_axis, w_axis);
    let w_w = w_axis;
    let w_nw = min(n_axis, w_axis) * (1.0 - uv.x);

    // Final weighted accumulation state.
    var accum_rgb = vec3<f32>(0.0);
    var accum_w = 0.0;

    // Per-direction contribution weights (directional mask * contribution availability).
    // Effective directional coverage = directional mask * contribution availability.
    let c_n = w_n * north.a;
    let c_ne = w_ne * north_east.a;
    let c_e = w_e * east.a;
    let c_se = w_se * south_east.a;
    let c_s = w_s * south.a;
    let c_sw = w_sw * south_west.a;
    let c_w = w_w * west.a;
    let c_nw = w_nw * north_west.a;

    // Accumulate weighted neighbor RGB and total weight.
    accum_rgb = accum_rgb + north.rgb * c_n;
    accum_w = accum_w + c_n;
    accum_rgb = accum_rgb + north_east.rgb * c_ne;
    accum_w = accum_w + c_ne;
    accum_rgb = accum_rgb + east.rgb * c_e;
    accum_w = accum_w + c_e;
    accum_rgb = accum_rgb + south_east.rgb * c_se;
    accum_w = accum_w + c_se;
    accum_rgb = accum_rgb + south.rgb * c_s;
    accum_w = accum_w + c_s;
    accum_rgb = accum_rgb + south_west.rgb * c_sw;
    accum_w = accum_w + c_sw;
    accum_rgb = accum_rgb + west.rgb * c_w;
    accum_w = accum_w + c_w;
    accum_rgb = accum_rgb + north_west.rgb * c_nw;
    accum_w = accum_w + c_nw;

    // Only mix neighbors when we have at least one non-zero contribution.
    if accum_w > 0.0 {
        // Normalize weighted sum into a neighbor average color.
        let neighbor_avg = accum_rgb / accum_w;
        // Union-style coverage keeps multiple semitransparent contributions from over-accumulating.
        let coverage = 1.0
            - (1.0 - c_n)
            * (1.0 - c_ne)
            * (1.0 - c_e)
            * (1.0 - c_se)
            * (1.0 - c_s)
            * (1.0 - c_sw)
            * (1.0 - c_w)
            * (1.0 - c_nw);
        out = vec4<f32>(mix(out.rgb, neighbor_avg, coverage), out.a);
    }
    return out;
}
