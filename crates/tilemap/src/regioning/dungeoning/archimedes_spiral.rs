#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{regioning_components::*, regioning_messages::{SgcPrepareTilesOrder, StructureBuildCompliance}, regioning_sgc_components::StructuredGenConfig
};
use crate::tile::{tile_components::DeleteOtherTiles, TileEzerosMap};
use super::dungeoning_ids::ARCHI;

#[allow(unused_parens, )]
pub fn archimedes_spiral_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
) {
    let mut compliances_to_emit = Vec::new();
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != ARCHI {
            continue;
        }

        let floor_tile_id = structured_gen_cfg.args
            .get("floor_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("dunewbie"));
        
        let wall_tile_id = structured_gen_cfg.args
            .get("wall_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("gray"));

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for archimedes spiral dungeon", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for archimedes spiral dungeon", wall_tile_id);
                continue;
            }
        };


        let chunk_positions = &build_order.chunks_gpos;
        if chunk_positions.is_empty() { continue; }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let tile_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let tile_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if tile_width == 0 || tile_height == 0 { continue; }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let tile_map_size = tile_width * tile_height;
        let mut floor_map = vec![false; tile_map_size];
        let carve_margin: usize = 1;
        if tile_width <= carve_margin * 2 || tile_height <= carve_margin * 2 {
            continue;
        }

        let Ok(&dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component for archimedes spiral dungeon", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, dimension_hash, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let corridor_width_min = structured_gen_cfg.args
            .parse_arg("arch_spiral_corridor_width_min", 3)
            .max(1);
        let corridor_width_max = structured_gen_cfg.args
            .parse_arg("arch_spiral_corridor_width_max", 5)
            .max(1);
        let (corridor_width_min, corridor_width_max) = if corridor_width_min <= corridor_width_max {
            (corridor_width_min, corridor_width_max)
        } else {
            (corridor_width_max, corridor_width_min)
        };
        let corridor_width = rng.random_range(corridor_width_min..=corridor_width_max) as usize;
        let wall_thickness = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_wall_thickness", 2)
            .clamp(1, 6) as i32;
        let corridor_separation_mult: f32 = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_corridor_separation_mult", 5.0);
        let corridor_separation_mult = corridor_separation_mult.clamp(0.5, 32.0);
        let turn_spacing = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_turn_spacing", corridor_width as f32)
            .clamp(1.0, 64.0)
            * corridor_separation_mult;
        let angle_step: f32 = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_angle_step", 0.08);
        let angle_step = angle_step.clamp(0.02, 0.5);

        let min_x = carve_margin as i32;
        let min_y = carve_margin as i32;
        let max_x = tile_width as i32 - 1 - carve_margin as i32;
        let max_y = tile_height as i32 - 1 - carve_margin as i32;
        let center_x = (min_x + max_x) / 2;
        let center_y = (min_y + max_y) / 2;
        let max_radius = (center_x - min_x)
            .min(max_x - center_x)
            .min(center_y - min_y)
            .min(max_y - center_y);
        if max_radius <= wall_thickness + 1 {
            continue;
        }

        let max_radius_sq = max_radius * max_radius;
        for y in min_y..=max_y {
            let dy = y - center_y;
            let row_idx = y as usize * tile_width;
            for x in min_x..=max_x {
                let dx = x - center_x;
                if dx * dx + dy * dy <= max_radius_sq {
                    floor_map[row_idx + x as usize] = true;
                }
            }
        }

        let b = turn_spacing / std::f32::consts::TAU;
        let start_radius = wall_thickness as f32;
        let max_theta = ((max_radius as f32 - start_radius) / b).max(std::f32::consts::TAU);

        let wall_half = (wall_thickness / 2).max(0) as i32;
        let wall_end = if wall_thickness % 2 == 0 {
            (wall_half - 1).max(0)
        } else {
            wall_half
        };
        let mut wall_map = vec![false; tile_map_size];

        let mark_wall = |wall_map: &mut Vec<bool>, wx: i32, wy: i32| {
            if wx < min_x || wy < min_y || wx > max_x || wy > max_y {
                return;
            }
            for dy in -wall_half..=wall_end {
                for dx in -wall_half..=wall_end {
                    let tx = wx + dx;
                    let ty = wy + dy;
                    if tx >= min_x && ty >= min_y && tx <= max_x && ty <= max_y {
                        let idx = ty as usize * tile_width + tx as usize;
                        wall_map[idx] = true;
                    }
                }
            }
        };

        let draw_line = |wall_map: &mut Vec<bool>, x0: i32, y0: i32, x1: i32, y1: i32| {
            let mut x = x0;
            let mut y = y0;
            let dx = (x1 - x0).abs();
            let sx = if x0 < x1 { 1 } else { -1 };
            let dy = -(y1 - y0).abs();
            let sy = if y0 < y1 { 1 } else { -1 };
            let mut err = dx + dy;
            loop {
                mark_wall(wall_map, x, y);
                if x == x1 && y == y1 { break; }
                let e2 = 2 * err;
                if e2 >= dy {
                    err += dy;
                    x += sx;
                }
                if e2 <= dx {
                    err += dx;
                    y += sy;
                }
            }
        };

        let mut theta = 0.0f32;
        let mut prev_x: Option<i32> = None;
        let mut prev_y: Option<i32> = None;
        while theta <= max_theta {
            let radius = start_radius + b * theta;
            if radius > max_radius as f32 {
                break;
            }
            let x = (center_x as f32 + radius * theta.cos()).round() as i32;
            let y = (center_y as f32 + radius * theta.sin()).round() as i32;
            if let (Some(px), Some(py)) = (prev_x, prev_y) {
                draw_line(&mut wall_map, px, py, x, y);
            } else {
                mark_wall(&mut wall_map, x, y);
            }
            prev_x = Some(x);
            prev_y = Some(y);
            theta += angle_step;
        }

        let delete_template = DeleteOtherTiles::default();
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()) {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;
                if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, Some(delete_template.clone())));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, Some(delete_template.clone())));
                }
            }
            chunk_tiles.push((chunk_pos, tiles4chunk));
        }
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_for_chunks: Vec::new(),
        });
        let region_pos = chunk_positions[0].to_region_pos();
        debug!(target: "dungeoning", "Spawned archimedes spiral dungeon across {} chunks at {:?}", chunk_positions.len(), region_pos);
    }

    writer.write_batch(compliances_to_emit);
}
