use rand::Rng;
use rand_pcg::Pcg64Mcg;
use bevy::{platform::collections::HashMap, prelude::*};
use common::{common_components::{HashId, Tag}, common_tag_components::TagSet};
use game_common::{game_common_components::ArgsDict, game_common_samplers::EntityWeightedSampler};
use sprite_shared::prelude::AcZ;
use ::tilemap_shared::{GlobalGenSettings, GlobalTilePos};
use crate::tile::{tile_components::DeleteOtherTilesInSamePos, tile_sampler_components::TileWeightedSampler};

#[derive(Default)]
pub struct DeleteOtherTilesConfigMap {
    global: Option<DeleteOtherTilesInSamePos>,
    by_used_tile_tag: HashMap<Tag, DeleteOtherTilesInSamePos>,
    by_tile_id: HashMap<HashId, DeleteOtherTilesInSamePos>,
}
impl DeleteOtherTilesConfigMap {
    pub fn get(&self, tile_id: &HashId, used_tile_tags: Option<&TagSet>) -> Option<DeleteOtherTilesInSamePos> {
        let mut merged = self.global.clone().unwrap_or_default();
        let mut has_any = self.global.is_some();
        if let Some(used_tile_tags) = used_tile_tags {
            for (tag, spec) in &self.by_used_tile_tag {
                if used_tile_tags.contains(tag.clone()) {
                    merged.merge_from(spec);
                    has_any = true;
                }
            }
        }
        if let Some(specific) = self.by_tile_id.get(tile_id) {
            merged.merge_from(specific);
            has_any = true;
        }
        has_any.then_some(merged)
    }
}

pub fn build_delete_other_tiles_by_tile_id(args: &ArgsDict) -> DeleteOtherTilesConfigMap {
    let mut out = DeleteOtherTilesConfigMap::default();
    for (key, values) in args.iter() {
        let Some(key) = key.as_str().strip_prefix("delete_other_tiles.") else {
            let Some(key) = key.as_str().strip_prefix("delete_other_tiles_tag.") else {
                continue;
            };
            let Some((tag, field)) = key.rsplit_once('.') else {
                continue;
            };
            if tag.trim().is_empty() {
                continue;
            }
            let spec = out.by_used_tile_tag.entry(Tag::trunc(tag)).or_default();
            spec.apply_delete_other_tiles_field(field, values);
            continue;
        };
        let Some((tile_id, field)) = key.rsplit_once('.') else {
            continue;
        };
        if tile_id.trim().is_empty() {
            continue;
        }
        if tile_id == "*" {
            let spec = out.global.get_or_insert_with(DeleteOtherTilesInSamePos::default);
            spec.apply_delete_other_tiles_field(field, values);
        } else {
            let spec = out.by_tile_id.entry(HashId::hash(tile_id)).or_default();
            spec.apply_delete_other_tiles_field(field, values);
        }
    }
    out
}

pub fn carve_room_rectangle(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    for yy in 0..h {
        for xx in 0..w {
            let tx = (x + xx) as usize;
            let ty = (y + yy) as usize;
            if tx < tile_width && ty < tile_height {
                floor_map[ty * tile_width + tx] = true;
            }
        }
    }
}

pub fn carve_room_circle(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) {
    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h) / 2).max(1);
    let radius_sq = radius * radius;
    for yy in 0..h {
        for xx in 0..w {
            let dx = (x + xx) - cx;
            let dy = (y + yy) - cy;
            if dx * dx + dy * dy <= radius_sq {
                let tx = (x + xx) as usize;
                let ty = (y + yy) as usize;
                if tx < tile_width && ty < tile_height {
                    floor_map[ty * tile_width + tx] = true;
                }
            }
        }
    }
}

/// Carve a triangle with arbitrary vertices and rotation angle.
/// v0, v1, v2 are the three vertices of the triangle as (x, y) coordinates.
/// This supports any triangle type and orientation.
pub fn carve_room_triangle_vertices(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
) {
    let (x0, y0) = v0;
    let (x1, y1) = v1;
    let (x2, y2) = v2;

    // Find bounding box
    let min_x = x0.min(x1).min(x2).max(0) as usize;
    let max_x = x0.max(x1).max(x2).min((tile_width - 1) as i32) as usize;
    let min_y = y0.min(y1).min(y2).max(0) as usize;
    let max_y = y0.max(y1).max(y2).min((tile_height - 1) as i32) as usize;

    if min_x > max_x || min_y > max_y {
        return;
    }

    // Helper function to compute sign of point relative to triangle edge
    fn sign(px: i32, py: i32, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
    }

    // Fill triangle using barycentric coordinates
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as i32;
            let py = y as i32;

            let d1 = sign(px, py, x0, y0, x1, y1);
            let d2 = sign(px, py, x1, y1, x2, y2);
            let d3 = sign(px, py, x2, y2, x0, y0);

            let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
            let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);

            if !(has_neg && has_pos) {
                floor_map[y * tile_width + x] = true;
            }
        }
    }
}

pub fn carve_room_regular_polygon(
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    sides: i32,
    rotation_deg: f32,
) {
    let w = w.max(0);
    let h = h.max(0);
    if w == 0 || h == 0 {
        return;
    }

    let sides = sides.max(3);
    let cx = x + w / 2;
    let cy = y + h / 2;
    let radius = (w.min(h) / 2).max(1) as f32;
    let rotation_rad = rotation_deg.to_radians();
    let step = std::f32::consts::TAU / sides as f32;

    let mut vertices: Vec<(i32, i32)> = Vec::with_capacity(sides as usize);
    for i in 0..sides {
        let angle = rotation_rad + step * i as f32;
        let vx = cx as f32 + radius * angle.cos();
        let vy = cy as f32 + radius * angle.sin();
        vertices.push((vx.round() as i32, vy.round() as i32));
    }

    let center = (cx, cy);
    for i in 0..sides as usize {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];
        carve_room_triangle_vertices(floor_map, tile_width, tile_height, center, v1, v2);
    }
}

pub fn carve_corridor_horizontal_floor(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    carve_margin: Option<usize>,
    corridor_width: Option<usize>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    y_base: i32,
    x0: i32,
    x1: i32,
) {
    let carve_margin = carve_margin.unwrap_or(1).clamp(0, 32);
    let corridor_width = corridor_width.unwrap_or(1).clamp(1, 16);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sx, ex) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    for x in sx..=ex {
        if x < carve_margin as i32 || x >= (tile_width - carve_margin) as i32 { continue; }
        for dy in -(corridor_width as i32)..=corridor_width as i32 {
            let mut y = y_base + dy;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                y += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if y < carve_margin as i32 || y >= (tile_height - carve_margin) as i32 { continue; }
            let x = x as usize;
            let y = y as usize;
            floor_map[y * tile_width + x] = true;
        }
    }
}

pub fn carve_corridor_vertical_floor(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    carve_margin: Option<usize>,
    corridor_width: Option<usize>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    x_base: i32,
    y0: i32,
    y1: i32,
) {
    let carve_margin = carve_margin.unwrap_or(1).clamp(0, 32);
    let corridor_width = corridor_width.unwrap_or(1).clamp(1, 16);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sy, ey) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    for y in sy..=ey {
        if y < carve_margin as i32 || y >= (tile_height - carve_margin) as i32 { continue; }
        for dx in -(corridor_width as i32)..=corridor_width as i32 {
            let mut x = x_base + dx;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                x += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if x < carve_margin as i32 || x >= (tile_width - carve_margin) as i32 { continue; }
            let x = x as usize;
            let y = y as usize;
            floor_map[y * tile_width + x] = true;
        }
    }
}

pub fn carve_corridor_horizontal(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: Option<i32>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    y_base: i32,
    x_start: i32,
    x_end: i32,
) {
    let corridor_radius = corridor_radius.unwrap_or(1).clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sx, ex) = if x_start <= x_end {(x_start, x_end)} else {(x_end, x_start)};
    for x in sx..=ex {
        for dy in -corridor_radius..=corridor_radius {
            let mut yy = y_base + dy;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                yy += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if x >= 0 && (x as usize) < tile_width && yy >= 0 && (yy as usize) < tile_height {
                let idx = (yy as usize) * tile_width + (x as usize);
                floor_map[idx] = true;
                corridor_map[idx] = true;
            }
        }
    }
}

pub fn carve_corridor_vertical(
    rng: &mut Pcg64Mcg,
    floor_map: &mut [bool],
    corridor_map: &mut [bool],
    tile_width: usize,
    tile_height: usize,
    corridor_radius: Option<i32>,
    corridor_wiggle_chance: Option<f32>,
    corridor_wiggle_step_max: Option<i32>,
    x_base: i32,
    y_start: i32,
    y_end: i32,
) {
    let corridor_radius = corridor_radius.unwrap_or(1).clamp(1, 8);
    let corridor_wiggle_step_max = corridor_wiggle_step_max.unwrap_or(1).clamp(1, 4);
    let corridor_wiggle_chance = corridor_wiggle_chance.unwrap_or(0.).clamp(0.0, 1.0);

    let (sy, ey) = if y_start <= y_end {(y_start, y_end)} else {(y_end, y_start)};
    for y in sy..=ey {
        for dx in -corridor_radius..=corridor_radius {
            let mut xx = x_base + dx;
            if rng.random_range(0.0..1.0) < corridor_wiggle_chance {
                xx += rng.random_range(-corridor_wiggle_step_max..=corridor_wiggle_step_max);
            }
            if xx >= 0 && (xx as usize) < tile_width && y >= 0 && (y as usize) < tile_height {
                let idx = (y as usize) * tile_width + (xx as usize);
                floor_map[idx] = true;
                corridor_map[idx] = true;
            }
        }
    }
}

pub fn resolve_sampled_tile_entity_from_sampler(
    root_sampler: &EntityWeightedSampler,
    sampler_query: &Query<&EntityWeightedSampler, (With<TileWeightedSampler>, common::AnyDisabling)>,
    anchor_gpos: GlobalTilePos,
    settings: &GlobalGenSettings,
    dimension_hash: HashId,
) -> Option<Entity> {
    let mut current_sampler = root_sampler;
    let mut depth = 0u8;
    while depth < 8 {
        let sampled_ent = current_sampler.sample_with_pos(anchor_gpos, settings, dimension_hash)?;
        if let Ok(next_sampler) = sampler_query.get(sampled_ent) {
            current_sampler = next_sampler;
            depth += 1;
            continue;
        }
        return Some(sampled_ent);
    }
    None
}
