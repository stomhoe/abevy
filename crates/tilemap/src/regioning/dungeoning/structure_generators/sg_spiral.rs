use std::collections::VecDeque;
#[allow(unused_imports)] use bevy::{platform::collections::*, prelude::*};

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use game_common::game_common_components::TemplEntiRef;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::regioning::{
    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder, TerrGenDisabledGposForChunks},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::terrain::biome::biome_resources::BiomeEntityMap;
use crate::tile::tile_resources::*;
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;
use super::super::dungeoning_carve_helpers::carve_room_circle;
use super::super::dungeoning_ids::SPIRAL;
use super::super::dungeoning_utils::{carve_external_wall_doorways, extend_occupied_gpos, ExternalDoorwayConfig, seal_structure_border_band};

#[allow(unused_parens, )]
pub fn spiral_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    templs_map: Res<TileEntityMap>,
    biome_map: Res<BiomeEntityMap>,
    mut room_pack_spawn: super::super::dungeoning_utils::DungeonRoomPackSpawnSystemParams,
    templ_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::Templ>, common::AnyDisabling)>,
    settings: Query<&GlobalGenSettings>,
    dimension_hash: Query<&HashId>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
) {
    compliances_to_emit.clear();
    room_pack_spawn.begin_pass();

    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };

    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != SPIRAL {
            continue;
        }
        let room_spawn_config = super::super::dungeoning_utils::DungeonRoomPackSpawnConfig::from_typed_args(
            &structured_gen_cfg.typed_args,
            &room_pack_spawn.command_registry,
            structured_gen_cfg.structure_id().as_str(),
        );
        let mut beings_remaining = structured_gen_cfg.max_being_count.unwrap_or(u32::MAX);

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
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("lava"));
        let delete_other_tiles_by_tile_id = super::super::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = super::super::dungeoning_utils::TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);

        let floor_entity = match templs_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => TemplEntiRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileTempl '{:?}' not found for spiral dungeon", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match templs_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => TemplEntiRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileTempl '{:?}' not found for spiral dungeon", wall_tile_id);
                continue;
            }
        };

        let lava_entity = match templs_map.0.get_cloned(lava_tile_id) {
            Ok(entity) => TemplEntiRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileTempl '{:?}' not found for spiral dungeon", lava_tile_id);
                continue;
            }
        };

        let chunk_positions = &build_order.chunks_pos;
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
        let mut hazard_map = vec![false; tile_map_size];
        let carve_margin: usize = 1;
        if tile_width <= carve_margin * 2 || tile_height <= carve_margin * 2 {
            continue;
        }
        let border_seal_margin: usize = structured_gen_cfg
            .args
            .parse_arg("border_seal_margin", carve_margin);
        let external_doorway_cfg = ExternalDoorwayConfig::from_args(&structured_gen_cfg.args);

        let dimension_hash = build_order.dimension_ref.0;

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
        let center_anchor_gpos = GlobalTilePos::new(
            origin_tile.x() + center_x,
            origin_tile.y() + center_y,
        );
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
        let estimated_path_points = ((max_theta / angle_step).ceil() as usize).saturating_add(1);
        let mut carved_path_points: Vec<IVec2> = Vec::with_capacity(estimated_path_points);

        let mut out_of_bounds_steps = 0u32;
        while theta <= max_theta && radius <= max_radius as f32 {
            let x = center_x as f32 + radius * theta.cos();
            let y = center_y as f32 + radius * theta.sin();
            let xi = x.round() as i32;
            let yi = y.round() as i32;

            if can_step(xi, yi) {
                out_of_bounds_steps = 0;
                let point = IVec2::new(xi, yi);
                if carved_path_points.last().copied() != Some(point) {
                    carved_path_points.push(point);
                }
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

        seal_structure_border_band(
            &mut floor_map,
            Some(&mut hazard_map),
            tile_width,
            tile_height,
            border_seal_margin,
        );

        let mut spiral_arm_spawn_anchors: Vec<(GlobalTilePos, &'static str)> = Vec::with_capacity(2);
        let entryway_margin = border_seal_margin.max(carve_margin);
        let can_carve_entryway_at = |x: i32, y: i32| {
            x >= entryway_margin as i32
                && y >= entryway_margin as i32
                && x < (tile_width - entryway_margin) as i32
                && y < (tile_height - entryway_margin) as i32
        };
        let axis_direction = |from: IVec2, to: IVec2| {
            let delta = to - from;
            if delta.x.abs() >= delta.y.abs() {
                IVec2::new(delta.x.signum(), 0)
            } else {
                IVec2::new(0, delta.y.signum())
            }
        };
        let mut carve_tip_entryway = |tip: IVec2, outward: IVec2, depth: usize| {
            if outward == IVec2::ZERO || depth == 0 {
                return;
            }
            for step in 1..=depth as i32 {
                let carve_pos = tip + outward * step;
                if !can_carve_entryway_at(carve_pos.x, carve_pos.y) {
                    break;
                }
                let idx = carve_pos.y as usize * tile_width + carve_pos.x as usize;
                floor_map[idx] = true;
                hazard_map[idx] = false;
            }
        };
        if carved_path_points.len() >= 3 {
            let first_tip = carved_path_points[0];
            let first_next = carved_path_points[1];
            let last_tip = carved_path_points[carved_path_points.len() - 1];
            let last_prev = carved_path_points[carved_path_points.len() - 2];
            let first_outward = -axis_direction(first_tip, first_next);
            let last_outward = axis_direction(last_prev, last_tip);
            carve_tip_entryway(first_tip, first_outward, 1);
            carve_tip_entryway(last_tip, last_outward, 2);
            let first_tip_idx = first_tip.y as usize * tile_width + first_tip.x as usize;
            if floor_map[first_tip_idx] {
                spiral_arm_spawn_anchors.push((
                    GlobalTilePos::new(origin_tile.x() + first_tip.x, origin_tile.y() + first_tip.y),
                    "arm_inner",
                ));
            }
            let last_tip_idx = last_tip.y as usize * tile_width + last_tip.x as usize;
            if floor_map[last_tip_idx] && first_tip != last_tip {
                spiral_arm_spawn_anchors.push((
                    GlobalTilePos::new(origin_tile.x() + last_tip.x, origin_tile.y() + last_tip.y),
                    "arm_outer",
                ));
            }
            trace!(
                target: DUNGEONING_SYSTEM,
                "structure={} carved_spiral_tip_entryways first_tip={} first_depth=1 last_tip={} last_depth=2",
                structured_gen_cfg.structure_id(),
                GlobalTilePos::new(origin_tile.x() + first_tip.x, origin_tile.y() + first_tip.y),
                GlobalTilePos::new(origin_tile.x() + last_tip.x, origin_tile.y() + last_tip.y),
            );
        }
        let queued = super::super::dungeoning_utils::queue_room_spawn_instance_message(
            "center_circle",
            center_anchor_gpos,
            build_order.dimension_ref,
            None,
            &mut beings_remaining,
            &room_spawn_config,
            &room_pack_spawn.source_lookup,
            &mut room_pack_spawn.pending_messages,
            &mut rng,
        );
        if queued {
            trace!(
                target: DUNGEONING_SYSTEM,
                "Queued room_spawn InstancePack for structure={} shape=center_circle at {}",
                structured_gen_cfg.structure_id(),
                center_anchor_gpos,
            );
        }
        for (anchor_gpos, room_spawn_key) in spiral_arm_spawn_anchors.iter().copied() {
            let queued = super::super::dungeoning_utils::queue_room_spawn_instance_message(
                room_spawn_key,
                anchor_gpos,
                build_order.dimension_ref,
                None,
                &mut beings_remaining,
                &room_spawn_config,
                &room_pack_spawn.source_lookup,
                &mut room_pack_spawn.pending_messages,
                &mut rng,
            );
            if queued {
                trace!(
                    target: DUNGEONING_SYSTEM,
                    "Queued room_spawn InstancePack for structure={} shape={} at {}",
                    structured_gen_cfg.structure_id(),
                    room_spawn_key,
                    anchor_gpos,
                );
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
        let opened_doorways = carve_external_wall_doorways(
            &mut floor_map,
            Some(&mut hazard_map),
            &mut wall_map,
            tile_width,
            tile_height,
            external_doorway_cfg,
            &mut rng,
        );
        trace!(target: DUNGEONING_SYSTEM, "structure={} opened_external_doorways={}", structured_gen_cfg.structure_id(), opened_doorways);

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let lava_delete_other_tiles = delete_other_tiles_by_tile_id.get("lava_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let disable_lava_terrgen = terrgen_disable_by_tile_id.should_disable_for("lava_tile_id");
        let forced_chunk_biomes = super::super::dungeoning_utils::forced_chunk_biomes_from_args(&structured_gen_cfg.typed_args, &biome_map);
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            let mut terrgen_disabled_gpos = TerrGenBlockedGposMask::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;
                if hazard_map[map_idx] {
                    tiles4chunk.push((tile_pos, lava_entity, lava_delete_other_tiles.clone()));
                    if disable_lava_terrgen {
                        let size = templ_size_query.get(lava_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut terrgen_disabled_gpos, chunk_pos, tile_pos, size);
                    }
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        let size = templ_size_query.get(floor_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut terrgen_disabled_gpos, chunk_pos, tile_pos, size);
                    }
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, None));
                }
            }

            chunk_tiles.push((chunk_pos, tiles4chunk));
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(chunk_pos, terrgen_disabled_gpos);
        }
        info!(target: DUNGEONING_SYSTEM, "structure={} pushing compliance blocked_terrgen_gpos={}", structured_gen_cfg.structure_id(), terrgen_disabled_gpos_for_chunks.count_blocked());
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            terrgen_disabled_for_chunks: Vec::new(),
            forced_chunk_biomes,
        });
    }
    writer.write_batch(compliances_to_emit.drain(..));
    room_pack_spawn.finish_pass();
}
