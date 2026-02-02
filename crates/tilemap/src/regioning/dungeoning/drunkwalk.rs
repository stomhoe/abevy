#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use game_common::game_common_components::EntityZeroRef;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use ::tilemap_shared::*;

use crate::regioning::{
    regioning_components::*,
    regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder},
    regioning_sgc_components::StructuredGenConfig,
};
use crate::tile::{tile_components::DeleteOtherTiles, tile_resources::TileEzerosMap};
use super::dungeoning_ids::DRUNKWALK;

#[allow(unused_parens)]
pub fn drunkwalk_dungeon_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    dimension_hash: Query<&HashId>,
    settings: Single<&GlobalGenSettings>,
) {
    let mut compliances_to_emit = Vec::new();    
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { 
            error!(target: "dungeoning", "StructuredGenConfig entity {:?} not found when making DrunkwalkDungeon, skipping structure spawn", build_order.structured_gen_cfg_ent);
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
            .map(|s| HashId::hash(s.as_str()));

        let floor_entity = match ezeros_map.0.get_cloned(floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{:?}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match ezeros_map.0.get_cloned(wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "dungeoning", "TileEzero with id '{:?}' not found in TileEzerosMap when making DrunkwalkDungeon, skipping structure spawn", wall_tile_id);
                continue;
            }
        };

        let lava_entity = lava_tile_id.and_then(|id| ezeros_map.0.get_cloned(id).ok()).map(EntityZeroRef);

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

        // Multiple drunkwalks with wider paths and more aggressive carving
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
        #[derive(Clone, Copy)]
        struct Chamber { center_x: usize, center_y: usize }
        let mut chambers: Vec<Chamber> = Vec::new();
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
        if lava_entity.is_some() {
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

        let delete_template = DeleteOtherTiles::default();
        let mut chunk_tiles: Vec<(ChunkPos, TilesFromBuilder)> = Vec::with_capacity(chunk_positions.len());
        for &chunk_pos in chunk_positions {
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()) {
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
                if hazard_map[map_idx] {
                    let ezero_ref = if let Some(lava) = lava_entity { lava } else { wall_entity };
                    tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, Some(delete_template.clone())));
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, Some(delete_template.clone())));
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
        debug!(target: "dungeoning", "Spawned organic drunkwalk dungeon across {} chunks at {:?}", chunk_positions.len(), region_pos);
    }
    writer.write_batch(compliances_to_emit);
}

use crate::regioning::dungeoning_utils::{
    carve_corridor_horizontal_floor, carve_corridor_vertical_floor,
};
