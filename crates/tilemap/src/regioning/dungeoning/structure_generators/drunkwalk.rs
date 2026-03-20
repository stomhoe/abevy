#[allow(unused_imports)] use bevy::prelude::*;
use bevy::platform::collections::HashSet;

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use game_common::game_common_components::EntityZeroRef;
use game_common::game_common_samplers::EntityWeightedSampler;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_distr::{Distribution, Normal};
use ::tilemap_shared::*;

use crate::regioning::{    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::tile_resources::*;
use crate::tile::tile_sampler_components::TileWeightedSampler;
use crate::tile::tile_sampler_resources::TileWeightedSamplerEntityMap;
use super::super::dungeoning_carve_helpers::{
    carve_corridor_horizontal_floor, carve_corridor_vertical_floor,
};
use super::super::dungeoning_ids::DRUNKWALK;
use super::super::dungeoning_utils::{extend_occupied_gpos, resolve_sampled_tile_entity_from_sampler};

#[derive(Clone, Copy)]
pub struct Chamber {
    center_x: usize,
    center_y: usize,
}

#[allow(unused_parens)]
pub fn drunkwalk_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEntityMap>,
    sampler_map: Res<TileWeightedSamplerEntityMap>,
    sampler_query: Query<&EntityWeightedSampler, (With<TileWeightedSampler>, common::AnyDisabling)>,
    ezero_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::EntityZero>, common::AnyDisabling)>,
    dimension_hash: Query<&HashId>,
    settings: Query<&GlobalGenSettings>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
    mut chambers: Local<Vec<Chamber>>,
    mut candidates: Local<Vec<(usize, usize)>>,
    mut tiles4chunk: Local<TilesFromBuilder>,
) {
    let Ok(settings) = settings.single() else {
        return;
    };
    compliances_to_emit.clear();
    chambers.clear();
    candidates.clear();
    tiles4chunk.clear();
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent)
        else {
            error_once!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", build_order.structured_gen_cfg_ent);
            continue; };

        if structured_gen_cfg.structure_hash_id() != DRUNKWALK {
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
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("lava"));
        let delete_other_tiles_by_tile_id = super::super::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = super::super::dungeoning_utils::TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);
        let boulder_sampler_id = structured_gen_cfg.args
            .get("boulder_sampler_id")
            .and_then(|v| v.first())
            .map(|s| HashId::hash(s.as_str()))
            .unwrap_or_else(|| HashId::hash("boulder_sampler"));
        let boulder_frequency: f32 = structured_gen_cfg
            .args
            .parse_arg("boulder_frequency", 0.0_f32)
            .clamp(0.0, 1.0);
        let boulder_frequency_mult: f32 = structured_gen_cfg
            .args
            .parse_arg("boulder_frequency_mult", 0.1_f32)
            .clamp(0.0, 1.0);

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{:?}' not found in TileEntityMap when making DrunkwalkDungeon, skipping structure spawn", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{:?}' not found in TileEntityMap when making DrunkwalkDungeon, skipping structure spawn", wall_tile_id);
                continue;
            }
        };

        let lava_entity = match ezeros_map.0.get_cloned(lava_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{:?}' not found in TileEntityMap when making DrunkwalkDungeon, skipping structure spawn", lava_tile_id);
                continue;
            }
        };

        let chunk_positions = &build_order.chunks_gpos;
        if chunk_positions.is_empty() {
            continue;
        }

        let min_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).min().unwrap();
        let max_chunk_x = chunk_positions.iter().map(|chunk| chunk.x()).max().unwrap();
        let min_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).min().unwrap();
        let max_chunk_y = chunk_positions.iter().map(|chunk| chunk.y()).max().unwrap();

        let chunk_width = (max_chunk_x - min_chunk_x + 1) as usize;
        let chunk_height = (max_chunk_y - min_chunk_y + 1) as usize;
        let tile_width = chunk_width * ChunkPos::CHUNK_SIZE.x as usize;
        let tile_height = chunk_height * ChunkPos::CHUNK_SIZE.y as usize;
        if tile_width == 0 || tile_height == 0 {
            continue;
        }

        let origin_chunk = ChunkPos::new(min_chunk_x, min_chunk_y);
        let origin_tile = origin_chunk.to_tilepos();

        let tile_map_size = tile_width * tile_height;
        let mut floor_map = vec![false; tile_map_size];
        let mut hazard_map = vec![false; tile_map_size];
        let carve_margin: usize = 1;
        if tile_width <= carve_margin * 2 || tile_height <= carve_margin * 2 {
            continue;
        }

        let Ok(&dimension_hash) = dimension_hash.get(build_order.dimension_ref.0) else {
            error!(target: "dungeoning", "Dimension entity {:?} has no HashId component when making DrunkwalkDungeon, skipping structure spawn", build_order.dimension_ref);
            continue;
        };

        let seed = chunk_positions[0].hash_value(&settings, dimension_hash, 1);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let corridor_wiggle_chance: f32 = structured_gen_cfg
            .args
            .parse_arg("corridor_wiggle_chance", 0.35);
        let corridor_wiggle_chance = corridor_wiggle_chance.clamp(0.0, 1.0);
        let corridor_wiggle_step_max = structured_gen_cfg
            .args
            .parse_arg("corridor_wiggle_step_max", 1)
            .clamp(0, 4);
        let corridor_detour_chance: f32 = structured_gen_cfg
            .args
            .parse_arg("corridor_detour_chance", 0.0);
        let corridor_detour_chance = corridor_detour_chance.clamp(0.0, 1.0);
        let corridor_detour_max_offset = structured_gen_cfg
            .args
            .parse_arg("corridor_detour_max_offset", 0)
            .clamp(0, 32);

        let num_walkers = rng.random_range(5..=10);
        let target_floor_tiles = std::cmp::max(1, ((tile_map_size as f32) * 0.45).ceil() as usize);
        let mut carved = 0;

        for _ in 0..num_walkers {
            let mut walker_x = rng.random_range(carve_margin..tile_width - carve_margin);
            let mut walker_y = rng.random_range(carve_margin..tile_height - carve_margin);
            let walker_steps = rng.random_range(100..350);
            let bias_chance = rng.random_range(15..45);
            let path_width = rng.random_range(2..=4);

            for _ in 0..walker_steps {
                if carved >= target_floor_tiles { break; }

                // Carve a wider path
                for dy in -(path_width as i32)..=path_width as i32 {
                    for dx in -(path_width as i32)..=path_width as i32 {
                        let x = (walker_x as i32 + dx).max(0) as usize;
                        let y = (walker_y as i32 + dy).max(0) as usize;
                        if x >= carve_margin && y >= carve_margin && x < tile_width - carve_margin && y < tile_height - carve_margin {
                            let idx = y * tile_width + x;
                            if !floor_map[idx] {
                                floor_map[idx] = true;
                                carved += 1;
                            }
                        }
                    }
                }

                let direction = if rng.random_range(0..100) < bias_chance {
                    rng.random_range(0..4)
                } else {
                    rng.random_range(0..4)
                };

                match direction {
                    0 => if walker_x + 1 < tile_width - carve_margin { walker_x += 1; },
                    1 => if walker_x > carve_margin { walker_x -= 1; },
                    2 => if walker_y + 1 < tile_height - carve_margin { walker_y += 1; },
                    _ => if walker_y > carve_margin { walker_y -= 1; },
                }
            }
        }

        // Larger, more ambitious chambers
        let chamber_count = rng.random_range(6..=12);
        for _ in 0..chamber_count {
            let min_center_x = carve_margin + 8;
            let max_center_x = tile_width.saturating_sub(8 + carve_margin);
            let min_center_y = carve_margin + 8;
            let max_center_y = tile_height.saturating_sub(8 + carve_margin);
            if max_center_x <= min_center_x || max_center_y <= min_center_y {
                continue;
            }
            let center_x = rng.random_range(min_center_x..=max_center_x);
            let center_y = rng.random_range(min_center_y..=max_center_y);
            let radius = rng.random_range(4..=10) as i32;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > radius * radius { continue; }

                    let rx = center_x as i32 + dx;
                    let ry = center_y as i32 + dy;
                    if rx < 0 || ry < 0 { continue; }

                    let rx = rx as usize;
                    let ry = ry as usize;
                    if rx < carve_margin || ry < carve_margin || rx >= tile_width - carve_margin || ry >= tile_height - carve_margin { continue; }

                    floor_map[ry * tile_width + rx] = true;
                }
            }
            chambers.push(Chamber { center_x, center_y });
        }

        // Connect chambers with corridors with random widths
        if chambers.len() > 1 {
            let mut sorted_chambers = chambers.clone();
            sorted_chambers.sort_by_key(|c| (c.center_x, c.center_y));

            for i in 1..sorted_chambers.len() {
                let from = sorted_chambers[i - 1];
                let to = sorted_chambers[i];
                let corridor_width = {
                    let normal = Normal::new(1.5, 1.0).unwrap();
                    (normal.sample(&mut rng) as i32).clamp(1, 5) as usize
                };

                let from_x = from.center_x as i32;
                let from_y = from.center_y as i32;
                let to_x = to.center_x as i32;
                let to_y = to.center_y as i32;

                if rng.random_range(0.0..1.0) < corridor_detour_chance {
                    let min_x = (from_x.min(to_x) - corridor_detour_max_offset).clamp(0, (tile_width - 1) as i32);
                    let max_x = (from_x.max(to_x) + corridor_detour_max_offset).clamp(0, (tile_width - 1) as i32);
                    let min_y = (from_y.min(to_y) - corridor_detour_max_offset).clamp(0, (tile_height - 1) as i32);
                    let max_y = (from_y.max(to_y) + corridor_detour_max_offset).clamp(0, (tile_height - 1) as i32);

                    let detour_x = rng.random_range(min_x..=max_x);
                    let detour_y = rng.random_range(min_y..=max_y);

                    carve_corridor_horizontal_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        from_y,
                        from_x,
                        detour_x,
                    );
                    carve_corridor_vertical_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        detour_x,
                        from_y,
                        detour_y,
                    );
                    carve_corridor_horizontal_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        detour_y,
                        detour_x,
                        to_x,
                    );
                    carve_corridor_vertical_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        to_x,
                        detour_y,
                        to_y,
                    );
                } else {
                    carve_corridor_horizontal_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        from_y,
                        from_x,
                        to_x,
                    );
                    carve_corridor_vertical_floor(
                        &mut rng,
                        &mut floor_map,
                        tile_width,
                        tile_height,
                        Some(carve_margin),
                        Some(corridor_width),
                        Some(corridor_wiggle_chance),
                        Some(corridor_wiggle_step_max),
                        to_x,
                        from_y,
                        to_y,
                    );
                }
            }
        }

        // Add lava hazards in some chambers
        for _ in 0..chamber_count.max(2) {
            if rng.random_range(0..100) > 35 { continue; }
            let min_center_x = carve_margin + 8;
            let max_center_x = tile_width.saturating_sub(8 + carve_margin);
            let min_center_y = carve_margin + 8;
            let max_center_y = tile_height.saturating_sub(8 + carve_margin);
            if max_center_x <= min_center_x || max_center_y <= min_center_y {
                continue;
            }
            let center_x = rng.random_range(min_center_x..=max_center_x);
            let center_y = rng.random_range(min_center_y..=max_center_y);
            let hazard_radius = rng.random_range(3..=6) as i32;

            for dy in -hazard_radius..=hazard_radius {
                for dx in -hazard_radius..=hazard_radius {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > hazard_radius * hazard_radius { continue; }

                    let hx = center_x as i32 + dx;
                    let hy = center_y as i32 + dy;
                    if hx < 0 || hy < 0 { continue; }

                    let hx = hx as usize;
                    let hy = hy as usize;
                    if hx < carve_margin || hy < carve_margin || hx >= tile_width - carve_margin || hy >= tile_height - carve_margin { continue; }

                    if floor_map[hy * tile_width + hx] {
                        hazard_map[hy * tile_width + hx] = true;
                        floor_map[hy * tile_width + hx] = false;
                    }
                }
            }
        }

        // Smooth isolated tiles
        let mut smoothed = floor_map.clone();
        for y in 1..tile_height - 1 {
            for x in 1..tile_width - 1 {
                if !floor_map[y * tile_width + x] { continue; }
                let neighbors = [
                    floor_map[(y - 1) * tile_width + x],
                    floor_map[(y + 1) * tile_width + x],
                    floor_map[y * tile_width + (x - 1)],
                    floor_map[y * tile_width + (x + 1)],
                ]
                .iter().filter(|&&b| b).count();

                if neighbors == 0 {
                    smoothed[y * tile_width + x] = false;
                }
            }
        }
        floor_map = smoothed;

        // Create wall outlines only (around floor tiles)
        let mut wall_map = vec![false; tile_map_size];
        for y in 0..tile_height {
            for x in 0..tile_width {
                if !floor_map[y * tile_width + x] && !hazard_map[y * tile_width + x] { continue; }
                // Mark adjacent non-floor tiles as walls (cardinal + diagonal corners)
                let neighbors = [
                    (y.saturating_sub(1), x),
                    (y + 1, x),
                    (y, x.saturating_sub(1)),
                    (y, x + 1),
                    // Diagonal corners for connected walls
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
        let mut boulder_anchor_map: Vec<Option<EntityZeroRef>> = vec![None; tile_map_size];
        if boulder_frequency > 0.0 {
            let boulder_sampler: Option<&EntityWeightedSampler> = sampler_map
                .0
                .get_cloned(boulder_sampler_id)
                .ok()
                .and_then(|sampler_ent| sampler_query.get(sampler_ent).ok());
            if let Some(boulder_sampler) = boulder_sampler {
                for y in carve_margin..tile_height - carve_margin {
                    let row_idx = y * tile_width;
                    for x in carve_margin..tile_width - carve_margin {
                        let idx = row_idx + x;
                        if floor_map[idx] && !hazard_map[idx] && !wall_map[idx] {
                            candidates.push((x, y));
                        }
                    }
                }
                candidates.shuffle(&mut rng);
                let target_count = ((candidates.len() as f32) * boulder_frequency * boulder_frequency_mult).round() as usize;
                let mut placed = 0usize;
                let mut blocked_map = vec![false; tile_map_size];
                let padding = 1usize;

                for (x, y) in candidates.iter().copied() {
                    if placed >= target_count {
                        break;
                    }
                    let anchor_gpos = GlobalTilePos::new(
                        origin_tile.x() + x as i32,
                        origin_tile.y() + y as i32,
                    );
                    let Some(sampled_boulder_ent) = resolve_sampled_tile_entity_from_sampler(
                        boulder_sampler,
                        &sampler_query,
                        anchor_gpos,
                        settings,
                        dimension_hash,
                    ) else {
                        continue;
                    };
                    let size = ezero_size_query
                        .get(sampled_boulder_ent)
                        .copied()
                        .unwrap_or_default()
                        .inner();
                    let size_x = size.x as usize;
                    let size_y = size.y as usize;
                    if x + size_x > tile_width || y + size_y > tile_height {
                        continue;
                    }

                    let mut can_place = true;
                    'footprint: for yy in y..y + size_y {
                        let row_idx = yy * tile_width;
                        for xx in x..x + size_x {
                            let idx = row_idx + xx;
                            if !floor_map[idx] || hazard_map[idx] || wall_map[idx] || blocked_map[idx] {
                                can_place = false;
                                break 'footprint;
                            }
                        }
                    }
                    if !can_place {
                        continue;
                    }
                    boulder_anchor_map[y * tile_width + x] = Some(EntityZeroRef(sampled_boulder_ent));
                    placed += 1;

                    let start_x = x.saturating_sub(padding);
                    let start_y = y.saturating_sub(padding);
                    let end_x = (x + size_x + padding).min(tile_width);
                    let end_y = (y + size_y + padding).min(tile_height);
                    for yy in start_y..end_y {
                        let row_idx = yy * tile_width;
                        for xx in start_x..end_x {
                            blocked_map[row_idx + xx] = true;
                        }
                    }
                }
            } else {
                warn!(target: "dungeoning", "Boulder sampler '{:?}' missing/invalid for drunkwalk dungeon", boulder_sampler_id);
            }
        }

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let lava_delete_other_tiles = delete_other_tiles_by_tile_id.get("lava_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let disable_lava_terrgen = terrgen_disable_by_tile_id.should_disable_for("lava_tile_id");

        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        let mut terrgen_disabled_gpos_for_chunks = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            tiles4chunk.clear();
            let mut blocked_gpos = HashSet::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 {
                    continue;
                }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height {
                    continue;
                }
                let map_idx = idx_y * tile_width + idx_x;
                if let Some(boulder_entity) = boulder_anchor_map[map_idx] {
                    if floor_map[map_idx] {
                        tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                        if disable_floor_terrgen {
                            let size = ezero_size_query.get(floor_entity.0).copied().unwrap_or_default().inner();
                            extend_occupied_gpos(&mut blocked_gpos, tile_pos, size);
                        }
                    }
                    tiles4chunk.push((tile_pos, boulder_entity, None));
                } else if hazard_map[map_idx] {
                    tiles4chunk.push((tile_pos, lava_entity, lava_delete_other_tiles.clone()));
                    if disable_lava_terrgen {
                        let size = ezero_size_query.get(lava_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut blocked_gpos, tile_pos, size);
                    }
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        let size = ezero_size_query.get(floor_entity.0).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut blocked_gpos, tile_pos, size);
                    }
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, None));
                }
            }
            chunk_tiles.push((chunk_pos, std::mem::take(&mut *tiles4chunk)));
            terrgen_disabled_gpos_for_chunks.push((chunk_pos, blocked_gpos));
        }
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunks: chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            terrgen_disabled_for_chunks: Vec::new(),
        });
    }
    writer.write_batch(compliances_to_emit.drain(..));
}
