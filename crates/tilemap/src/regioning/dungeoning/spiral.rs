use std::collections::VecDeque;
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use common::common_tag_components::TagSet;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{
    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::tile_resources::*;
use super::dungeoning_carve_helpers::carve_room_circle;
use super::dungeoning_ids::SPIRAL;

#[allow(unused_parens, )]
pub fn spiral_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEntityMap>,
    ezero_tag_query: Query<Option<&'static TagSet>, (With<game_common::game_common_components::EntityZero>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
) {
    let mut compliances_to_emit = Vec::new();

    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };

    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != SPIRAL {
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

        let lava_tile_id = structured_gen_cfg.args
            .get("lava_tile_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()));
        let delete_other_tiles_by_tile_id = crate::regioning::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for spiral dungeon", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero '{:?}' not found for spiral dungeon", wall_tile_id);
                continue;
            }
        };

        let lava_entity = lava_tile_id.and_then(|id| ezeros_map.0.get_cloned(id).ok()).map(EntityZeroRef);

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
        let hazard_map = vec![false; tile_map_size];
        let carve_margin: usize = 1;
        if tile_width <= carve_margin * 2 || tile_height <= carve_margin * 2 {
            continue;
        }

        let Ok(&dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component for spiral dungeon", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, dimension_hash, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let corridor_width_min = structured_gen_cfg
            .args
            .parse_arg("spiral_corridor_width_min", 3)
            .max(1);
        let corridor_width_max = structured_gen_cfg
            .args
            .parse_arg("spiral_corridor_width_max", 5)
            .max(1);
        let (corridor_width_min, corridor_width_max) = if corridor_width_min <= corridor_width_max {
            (corridor_width_min, corridor_width_max)
        } else {
            (corridor_width_max, corridor_width_min)
        };
        let corridor_width = rng.random_range(corridor_width_min..=corridor_width_max) as usize;
        let corridor_radius = (corridor_width as i32) / 2;

        let max_room_radius = (tile_width.min(tile_height) as i32 / 4).max(2);
        let room_radius = structured_gen_cfg
            .args
            .parse_arg("spiral_room_radius", 5)
            .clamp(2, max_room_radius);
        let wall_thickness = structured_gen_cfg
            .args
            .parse_arg("spiral_center_wall_thickness", 1)
            .clamp(1, 6);

        let center_x = (tile_width as i32) / 2;
        let center_y = (tile_height as i32) / 2;
        let inner_radius = room_radius;
        let outer_radius = room_radius + wall_thickness;

        carve_room_circle(
            &mut floor_map,
            tile_width,
            tile_height,
            center_x - inner_radius,
            center_y - inner_radius,
            inner_radius * 2 + 1,
            inner_radius * 2 + 1,
        );

        let mut wall_ring_map = vec![false; tile_map_size];
        for y in 0..tile_height {
            for x in 0..tile_width {
                let dx = x as i32 - center_x;
                let dy = y as i32 - center_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= outer_radius * outer_radius && dist_sq >= inner_radius * inner_radius {
                    wall_ring_map[y * tile_width + x] = true;
                }
            }
        }

        let center_hole_radius = (room_radius - 2).max(1);
        let center_hole_radius_sq = center_hole_radius * center_hole_radius;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let dx = x as i32 - center_x;
                let dy = y as i32 - center_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= center_hole_radius_sq {
                    floor_map[y * tile_width + x] = false;
                }
            }
        }

        let mut center_wall_ring_map = vec![false; tile_map_size];
        let center_ring_radius = center_hole_radius + 1;
        let center_ring_outer = center_ring_radius + 1;
        let center_ring_radius_sq = center_ring_radius * center_ring_radius;
        let center_ring_outer_sq = center_ring_outer * center_ring_outer;
        for y in 0..tile_height {
            for x in 0..tile_width {
                let dx = x as i32 - center_x;
                let dy = y as i32 - center_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq >= center_ring_radius_sq && dist_sq <= center_ring_outer_sq {
                    center_wall_ring_map[y * tile_width + x] = true;
                }
            }
        }

        let ring_samples = ((std::f32::consts::TAU * center_ring_radius as f32) * 2.0)
            .ceil()
            .max(24.0) as usize;
        for i in 0..ring_samples {
            let theta = (i as f32 / ring_samples as f32) * std::f32::consts::TAU;
            let x = center_x as f32 + center_ring_radius as f32 * theta.cos();
            let y = center_y as f32 + center_ring_radius as f32 * theta.sin();
            let xi = x.round() as i32;
            let yi = y.round() as i32;
            if xi >= 0 && yi >= 0 && (xi as usize) < tile_width && (yi as usize) < tile_height {
                center_wall_ring_map[yi as usize * tile_width + xi as usize] = true;
            }
        }

        let can_step = |nx: i32, ny: i32| {
            nx >= carve_margin as i32
            && ny >= carve_margin as i32
            && nx < (tile_width - carve_margin) as i32
            && ny < (tile_height - carve_margin) as i32
        };
        let max_radius = (tile_width.min(tile_height) as i32 / 2) - carve_margin as i32 - corridor_radius - 1;
        if max_radius <= room_radius + 1 {
            continue;
        }
        let angle_step: f32 = structured_gen_cfg
            .args
            .parse_arg("spiral_angle_step", 0.15);
        let angle_step = angle_step.clamp(0.02, 1.0);
        let turn_spacing: f32 = structured_gen_cfg
            .args
            .parse_arg("spiral_turn_spacing", 1.0);
        let turn_spacing = turn_spacing.clamp(0.5, 32.0);
        let base_radius_per_rad = turn_spacing / std::f32::consts::TAU;
        let radius_step_scale: f32 = structured_gen_cfg
            .args
            .parse_arg("spiral_radius_step", 1.0);
        let radius_step_scale = radius_step_scale.clamp(0.2, 5.0);
        let radius_per_rad = (base_radius_per_rad * radius_step_scale).clamp(0.05, 10.0);
        let start_radius = outer_radius as f32 + 1.0;
        let turns: f32 = structured_gen_cfg
            .args
            .parse_arg("spiral_turns", 0.0);
        let mut max_theta = ((max_radius as f32 - start_radius) / radius_per_rad)
        .max(std::f32::consts::TAU);
        if turns > 0.0 {
            max_theta = max_theta.min(std::f32::consts::TAU * turns.max(1.0));
        }

        let mut theta = 0.0f32;
        let mut radius = start_radius;

        let mut out_of_bounds_steps = 0u32;
        while theta <= max_theta && radius <= max_radius as f32 {
            let x = center_x as f32 + radius * theta.cos();
            let y = center_y as f32 + radius * theta.sin();
            let xi = x.round() as i32;
            let yi = y.round() as i32;

            if can_step(xi, yi) {
                out_of_bounds_steps = 0;
                for dy in -corridor_radius..=corridor_radius {
                    for dx in -corridor_radius..=corridor_radius {
                        let tx = xi + dx;
                        let ty = yi + dy;
                        if can_step(tx, ty) {
                            let idx = (ty as usize) * tile_width + (tx as usize);
                            floor_map[idx] = true;
                        }
                    }
                }
            } else {
                out_of_bounds_steps += 1;
                if out_of_bounds_steps > 20 {
                    break;
                }
            }

            theta += angle_step;
            radius += radius_per_rad * angle_step;
        }

        // Bridge thin walls between adjacent spiral arms
        let mut bridged = floor_map.clone();
        for y in 1..tile_height - 1 {
            for x in 1..tile_width - 1 {
                let idx = y * tile_width + x;
                if floor_map[idx] { continue; }
                let left = floor_map[y * tile_width + (x - 1)];
                let right = floor_map[y * tile_width + (x + 1)];
                let up = floor_map[(y - 1) * tile_width + x];
                let down = floor_map[(y + 1) * tile_width + x];
                if (left && right) || (up && down) {
                    bridged[idx] = true;
                }
            }
        }
        floor_map = bridged;

        // Remove disconnected floor islands
        let mut visited = vec![false; tile_map_size];
        let mut queue = VecDeque::new();
        let center_idx = center_y as usize * tile_width + center_x as usize;
        let mut seed_idx = None;
        if floor_map[center_idx] {
            seed_idx = Some(center_idx);
        } else {
            let max_search_radius = (tile_width.max(tile_height) as i32).saturating_sub(1);
            'find_seed: for radius in 1..=max_search_radius {
                let min_x = (center_x - radius).max(0);
                let max_x = (center_x + radius).min(tile_width as i32 - 1);
                let min_y = (center_y - radius).max(0);
                let max_y = (center_y + radius).min(tile_height as i32 - 1);
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        let idx = y as usize * tile_width + x as usize;
                        if floor_map[idx] {
                            seed_idx = Some(idx);
                            break 'find_seed;
                        }
                    }
                }
            }
            if seed_idx.is_none() {
                seed_idx = floor_map.iter().position(|&v| v);
            }
        }
        if let Some(seed_idx) = seed_idx {
            visited[seed_idx] = true;
            queue.push_back(seed_idx);
        }
        while let Some(idx) = queue.pop_front() {
            let x = idx % tile_width;
            let y = idx / tile_width;
            let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
            ];
            for (nx, ny) in neighbors {
                if nx >= tile_width || ny >= tile_height { continue; }
                let nidx = ny * tile_width + nx;
                if floor_map[nidx] && !visited[nidx] {
                    visited[nidx] = true;
                    queue.push_back(nidx);
                }
            }
        }
        if seed_idx.is_some() {
            for idx in 0..tile_map_size {
                if floor_map[idx] && !visited[idx] {
                    floor_map[idx] = false;
                }
            }
        }

        // Create wall outlines only (around floor tiles)
        let mut wall_map = vec![false; tile_map_size];
        for y in 0..tile_height {
            for x in 0..tile_width {
                if !floor_map[y * tile_width + x] && !hazard_map[y * tile_width + x] { continue; }
                let neighbors = [
                (y.saturating_sub(1), x),
                (y + 1, x),
                (y, x.saturating_sub(1)),
                (y, x + 1),
                (y.saturating_sub(1), x.saturating_sub(1)),
                (y.saturating_sub(1), x + 1),
                (y + 1, x.saturating_sub(1)),
                (y + 1, x + 1),
                ];
                for (ny, nx) in neighbors {
                    if ny < tile_height && nx < tile_width {
                        let idx = ny * tile_width + nx;
                        if !floor_map[idx] && !hazard_map[idx] {
                            wall_map[idx] = true;
                        }
                    }
                }
            }
        }
        for idx in 0..tile_map_size {
            if wall_ring_map[idx] {
                wall_map[idx] = true;
            }
        }

        for idx in 0..tile_map_size {
            if center_wall_ring_map[idx] {
                wall_map[idx] = true;
            }
        }

        let mut closed_wall_map = wall_map.clone();
        for y in 1..tile_height - 1 {
            for x in 1..tile_width - 1 {
                let dx = x as i32 - center_x;
                let dy = y as i32 - center_y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= center_hole_radius_sq {
                    continue;
                }
                let idx = y * tile_width + x;
                if wall_map[idx] || floor_map[idx] || hazard_map[idx] {
                    continue;
                }
                let left = wall_map[y * tile_width + (x - 1)];
                let right = wall_map[y * tile_width + (x + 1)];
                let up = wall_map[(y - 1) * tile_width + x];
                let down = wall_map[(y + 1) * tile_width + x];
                let up_left = wall_map[(y - 1) * tile_width + (x - 1)];
                let up_right = wall_map[(y - 1) * tile_width + (x + 1)];
                let down_left = wall_map[(y + 1) * tile_width + (x - 1)];
                let down_right = wall_map[(y + 1) * tile_width + (x + 1)];
                if (left && right)
                    || (up && down)
                    || (up_left && down_right)
                    || (up_right && down_left)
                {
                    closed_wall_map[idx] = true;
                }
            }
        }
        wall_map = closed_wall_map;

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get(&floor_tile_id, ezero_tag_query.get(floor_entity.0).ok().flatten());
        let wall_delete_other_tiles = delete_other_tiles_by_tile_id.get(&wall_tile_id, ezero_tag_query.get(wall_entity.0).ok().flatten());
        let lava_delete_other_tiles = lava_tile_id.and_then(|tile_id| {
            lava_entity.and_then(|lava_entity| delete_other_tiles_by_tile_id.get(&tile_id, ezero_tag_query.get(lava_entity.0).ok().flatten()))
        });
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;
                if hazard_map[map_idx] {
                    let ezero_ref = if let Some(lava) = lava_entity { lava } else { wall_entity };
                    let delete_other_tiles = if lava_entity.is_some() {
                        lava_delete_other_tiles.clone()
                    } else {
                        wall_delete_other_tiles.clone()
                    };
                    tiles4chunk.push((tile_pos, ezero_ref, delete_other_tiles));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, wall_delete_other_tiles.clone()));
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
        debug!(target: "dungeoning", "Spawned spiral dungeon across {} chunks at {:?}", chunk_positions.len(), region_pos);
    }
    writer.write_batch(compliances_to_emit);
}
