#import bevy_ecs_tilemap::common::{sprite_texture, sprite_sampler, tilemap_data}
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput

const SQUARE_DISTANCE = 0.9;
const DIAGONAL_DISTANCE = 0.88;
const STRENGTH = vec2<f32>(0.0, 0.93);
const CENTER = vec2<f32>(0.5, 0.5);
const EAST = CENTER + vec2<f32>(SQUARE_DISTANCE, 0.0);
const WEST = CENTER + vec2<f32>(-SQUARE_DISTANCE, 0.0);
const NORTH = CENTER + vec2<f32>(0.0, -SQUARE_DISTANCE);
const SOUTH = CENTER + vec2<f32>(0.0, SQUARE_DISTANCE);
const SOUTH_EAST = CENTER + vec2<f32>(DIAGONAL_DISTANCE, DIAGONAL_DISTANCE);
const SOUTH_WEST = CENTER + vec2<f32>(-DIAGONAL_DISTANCE, DIAGONAL_DISTANCE);
const NORTH_EAST = CENTER + vec2<f32>(DIAGONAL_DISTANCE, -DIAGONAL_DISTANCE);
const NORTH_WEST = CENTER + vec2<f32>(-DIAGONAL_DISTANCE, -DIAGONAL_DISTANCE);
const EPS = 0.00001;

@group(3) @binding(1) var tile_indices_map: texture_2d<f32>;
@group(3) @binding(2) var tile_flags_map: texture_2d<f32>;
@group(3) @binding(3) var tile_params_map: texture_2d<f32>;
@group(3) @binding(4) var tile_mask_map: texture_2d<f32>;
@group(3) @binding(5) var<uniform> map_size_tiles: vec2<f32>;
@group(3) @binding(6) var<uniform> time: f32;

struct TileData {
    base_tex_index: u32,
    overlay_tex_index: u32,
    has_params: bool,
    blend_enabled: bool,
    has_overlay: bool,
    scale: f32,
    speed: f32,
    wavy_strength: f32,
    time_offset: f32,
    mask_color: vec4<f32>,
};

fn decode_u16(low: f32, high: f32) -> u32 {
    let lo = u32(round(low * 255.0));
    let hi = u32(round(high * 255.0));
    return lo + (hi << 8u);
}

fn decode_flags(v: f32) -> u32 {
    return u32(round(v * 255.0));
}

fn almost_equal(a: vec4<f32>, b: vec4<f32>, epsilon: f32) -> bool {
    return all(abs(a - b) < vec4<f32>(epsilon));
}

fn in_bounds(tile: vec2<i32>) -> bool {
    return tile.x >= 0
        && tile.y >= 0
        && tile.x < i32(map_size_tiles.x)
        && tile.y < i32(map_size_tiles.y);
}

fn read_tile_data(tile: vec2<i32>) -> TileData {
    if !in_bounds(tile) {
        return TileData(0u, 0u, false, false, false, 0.0, 0.0, 0.0, 0.0, vec4<f32>(0.0));
    }

    let tex = textureLoad(tile_indices_map, tile, 0);
    let flags_raw = decode_flags(textureLoad(tile_flags_map, tile, 0).r);
    let params = textureLoad(tile_params_map, tile, 0);
    let mask = textureLoad(tile_mask_map, tile, 0);
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
        mask,
    );
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
    var v = 0.0;
    var a = 0.5;
    var q = p;
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

fn sample_tile_color(tile: vec2<i32>, uv: vec2<f32>) -> vec4<f32> {
    let data = read_tile_data(tile);
    let base = textureSample(sprite_texture, sprite_sampler, uv, i32(data.base_tex_index));
    if !(data.has_params && data.has_overlay) {
        return base;
    }
    if !almost_equal(base, data.mask_color, EPS) {
        return base;
    }

    let repeat_scale = max(data.scale, 1e-5);
    let uv_world = (vec2<f32>(f32(tile.x), f32(tile.y)) + uv) * repeat_scale / 10000.0;
    let t = (time + data.time_offset) * max(data.speed, 0.0);
    let strength = max(data.wavy_strength, 0.0);
    let offset = select(vec2<f32>(0.0, 0.0), compute_water_offset(uv_world, t, strength), strength > 0.0);
    var sample_uv = fract(fract(uv_world) + offset);

    if strength > 0.0 {
        sample_uv = sample_uv + 0.001 * vec2<f32>(
            sin(dot(sample_uv, vec2<f32>(12.9898, 78.233)) + t),
            cos(dot(sample_uv, vec2<f32>(93.9898, 67.345)) - t * 0.8)
        );
    }

    var overlay = textureSample(sprite_texture, sprite_sampler, sample_uv, i32(data.overlay_tex_index));
    if strength > 0.0 {
        let shimmer = 0.03 * strength * fbm(uv_world * 1.8 + t * 0.6);
        overlay = vec4<f32>(overlay.rgb + vec3<f32>(shimmer), overlay.a);
    }
    return vec4<f32>(overlay.rgb, overlay.a * base.a);
}

fn sample_neighbor_or_self(tile: vec2<i32>, uv: vec2<f32>, self_color: vec4<f32>) -> vec4<f32> {
    let data = read_tile_data(tile);
    if !data.blend_enabled {
        return self_color;
    }
    return sample_tile_color(tile, uv);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let tile_pos = vec2<i32>(
        i32(in.storage_position.x) + i32(tilemap_data.chunk_pos.x),
        i32(in.storage_position.y) + i32(tilemap_data.chunk_pos.y),
    );
    let uv = in.uv.xy;
    var out = sample_tile_color(tile_pos, uv) * in.color;
    if out.a < 0.001 {
        discard;
    }

    let tile_data = read_tile_data(tile_pos);
    if !tile_data.blend_enabled {
        return out;
    }

    let y_uv = vec2<f32>(uv.x, 1.0 - uv.y);
    let xy_uv = vec2<f32>(1.0 - uv.x, 1.0 - uv.y);
    let x_uv = vec2<f32>(1.0 - uv.x, uv.y);

    let north = sample_neighbor_or_self(tile_pos + vec2<i32>(0, 1), y_uv, out);
    let north_east = sample_neighbor_or_self(tile_pos + vec2<i32>(1, 1), xy_uv, out);
    let east = sample_neighbor_or_self(tile_pos + vec2<i32>(1, 0), x_uv, out);
    let south_east = sample_neighbor_or_self(tile_pos + vec2<i32>(1, -1), xy_uv, out);
    let south = sample_neighbor_or_self(tile_pos + vec2<i32>(0, -1), y_uv, out);
    let south_west = sample_neighbor_or_self(tile_pos + vec2<i32>(-1, -1), xy_uv, out);
    let west = sample_neighbor_or_self(tile_pos + vec2<i32>(-1, 0), x_uv, out);
    let north_west = sample_neighbor_or_self(tile_pos + vec2<i32>(-1, 1), xy_uv, out);

    out = mix(out, north, smoothstep(STRENGTH.y, STRENGTH.x, distance(y_uv, SOUTH)));
    out = mix(out, north_east, smoothstep(STRENGTH.y, STRENGTH.x, distance(xy_uv, SOUTH_WEST)));
    out = mix(out, east, smoothstep(STRENGTH.y, STRENGTH.x, distance(x_uv, WEST)));
    out = mix(out, south_east, smoothstep(STRENGTH.y, STRENGTH.x, distance(xy_uv, NORTH_WEST)));
    out = mix(out, south, smoothstep(STRENGTH.y, STRENGTH.x, distance(y_uv, NORTH)));
    out = mix(out, south_west, smoothstep(STRENGTH.y, STRENGTH.x, distance(xy_uv, NORTH_EAST)));
    out = mix(out, west, smoothstep(STRENGTH.y, STRENGTH.x, distance(x_uv, EAST)));
    out = mix(out, north_west, smoothstep(STRENGTH.y, STRENGTH.x, distance(xy_uv, SOUTH_EAST)));
    return out;
}
