#[allow(unused_imports)] use bevy::{platform::collections::*, prelude::*};

use common::common_components::HashId;
#[allow(unused_imports)] use common::log_targets::DUNGEONING_SYSTEM;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use tilemap_shared::tilemap_shared_samplers::HashIdWeightedSampler;
use ::tilemap_shared::*;

use crate::regioning::{regioning_components::*, regioning_messages::{StructureBuildCompliance, SgcPrepareTilesOrder, TerrGenDisabledGposForChunks}, regioning_sgc_components::StructuredGenConfig
};
use crate::terrain::biome::biome_resources::BiomeEntityMap;
use crate::tile::tile_resources::*;
use crate::tile::tile_sampler_components::TileWeightedSampler;
use crate::tile::tile_sampler_resources::TileWeightedSamplerEntityMap;
use super::super::dungeoning_ids::ARCHI;
use super::super::dungeoning_utils::{extend_occupied_gpos, resolve_sampled_tile_entity_from_sampler, seal_structure_border_band};
use crate::terrain::terrgen_async_resources::TerrGenBlockedGposMask;

inventory::submit! {
    crate::regioning::dungeoning::dungeoning_ids::StructureGeneratorDescriptor {
        structure_hash_id: ARCHI,
    }
}

#[allow(unused_parens, )]
pub fn archimedes_spiral_building_system(
    mut reader: MessageReader<SgcPrepareTilesOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    templs_map: Res<TileEntityMap>,
    biome_map: Res<BiomeEntityMap>,
    mut room_pack_spawn: super::super::dungeoning_utils::DungeonRoomPackSpawnSystemParams,
    sampler_map: Res<TileWeightedSamplerEntityMap>,
    sampler_query: Query<&HashIdWeightedSampler, (With<TileWeightedSampler>, common::AnyDisabling)>,
    templ_size_query: Query<&SizeInTiles, (With<game_common::game_common_components::Templ>, common::AnyDisabling)>,
    templ_hash_query: Query<&HashId>,
    settings: Query<&GlobalGenSettings>,
    mut compliances_to_emit: Local<Vec<StructureBuildCompliance>>,
    mut candidates: Local<Vec<(usize, usize)>>,
    mut ring_spawn_anchors: Local<Vec<(GlobalTilePos, String, i32)>>,
    mut tiles4chunk: Local<TilesFromBuilder>,
) {
    let Ok(settings) = settings.single() else {
        return;
    };
    compliances_to_emit.clear();
    room_pack_spawn.begin_pass();
    candidates.clear();
    ring_spawn_anchors.clear();
    tiles4chunk.clear();
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) else { continue; };

        if structured_gen_cfg.structure_hash_id() != ARCHI {
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
        let delete_other_tiles_by_tile_id = super::super::dungeoning_utils::DeleteOtherTilesConfigMap::from_args(&structured_gen_cfg.args);
        let terrgen_disable_by_tile_id = super::super::dungeoning_utils::TerrGenDisableConfigMap::from_args(&structured_gen_cfg.args);

        let floor_entity = match templs_map.0.get_cloned(floor_tile_id) {
            Ok(_) => TileRef(floor_tile_id),
            Err(_) => {
                error!(target: "dungeoning", "TileTempl '{:?}' not found for archimedes spiral dungeon", floor_tile_id);
                continue;
            }
        };

        let wall_entity = match templs_map.0.get_cloned(wall_tile_id) {
            Ok(_) => TileRef(wall_tile_id),
            Err(_) => {
                error!(target: "dungeoning", "TileTempl '{:?}' not found for archimedes spiral dungeon", wall_tile_id);
                continue;
            }
        };
        let Ok(floor_entity_ent) = templs_map.0.get_cloned(floor_entity.0) else {
            continue;
        };
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
        let carve_margin: usize = 1;
        if tile_width <= carve_margin * 2 || tile_height <= carve_margin * 2 {
            continue;
        }
        let border_seal_margin: usize = structured_gen_cfg
            .args
            .parse_arg("border_seal_margin", carve_margin);

        let dimension_hash = build_order.dimension_ref.0;

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
        let ring_pack_count_min = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_ring_pack_count_min", 1_usize)
            .max(1);
        let ring_pack_count_max = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_ring_pack_count_max", 4_usize)
            .max(ring_pack_count_min);
        let ring_pack_count_outer_start_pct = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_ring_pack_count_outer_start_pct", 35_i32)
            .clamp(0, 100);
        let ring_pack_count_outer_step_pct = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_ring_pack_count_outer_step_pct", 20_i32)
            .max(1);
        let ring_pack_angle_jitter = structured_gen_cfg
            .args
            .parse_arg("arch_spiral_ring_pack_angle_jitter", 0.35_f32)
            .clamp(0.0, std::f32::consts::PI);

        let min_x = carve_margin as i32;
        let min_y = carve_margin as i32;
        let max_x = tile_width as i32 - 1 - carve_margin as i32;
        let max_y = tile_height as i32 - 1 - carve_margin as i32;
        let center_x = (min_x + max_x) / 2;
        let center_y = (min_y + max_y) / 2;
        let center_anchor_gpos = GlobalTilePos::new(
            origin_tile.x() + center_x,
            origin_tile.y() + center_y,
        );
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
        let ring_spacing = (b * std::f32::consts::TAU).max(1.0);
        let ring_count = (((max_radius as f32 - start_radius) / ring_spacing).floor().max(0.0) as usize)
            .saturating_add(1);
        ring_spawn_anchors.reserve(ring_count * ring_pack_count_max);
        let ring_radius_span = (max_radius as f32 - start_radius).max(1.0);
        let theta_offsets = [
            0.0_f32,
            0.35,
            -0.35,
            std::f32::consts::PI * 0.5,
            -std::f32::consts::PI * 0.5,
            std::f32::consts::PI,
        ];
        for ring_idx in 0..ring_count {
            let ring_radius = (max_radius as f32 - ring_idx as f32 * ring_spacing)
                .clamp(start_radius, max_radius as f32);
            let ring_radius_pct = (((ring_radius - start_radius) / ring_radius_span) * 100.0)
                .round()
                .clamp(0.0, 100.0) as i32;
            let ring_extra_pack_count = if ring_radius_pct <= ring_pack_count_outer_start_pct {
                0usize
            } else {
                ((ring_radius_pct - ring_pack_count_outer_start_pct) / ring_pack_count_outer_step_pct) as usize
            };
            let ring_pack_count = (ring_pack_count_min + ring_extra_pack_count).min(ring_pack_count_max);
            let ring_phase = rng.random_range(0.0..std::f32::consts::TAU);
            for pack_idx in 0..ring_pack_count {
                let anchor_theta = ring_phase
                    + (pack_idx as f32 * std::f32::consts::TAU / ring_pack_count as f32)
                    + rng.random_range(-ring_pack_angle_jitter..=ring_pack_angle_jitter);
                let mut found_anchor = None;
                for offset in theta_offsets {
                    let search_theta = (anchor_theta + offset).rem_euclid(std::f32::consts::TAU);
                    let search_x = (center_x as f32 + ring_radius * search_theta.cos()).round() as i32;
                    let search_y = (center_y as f32 + ring_radius * search_theta.sin()).round() as i32;
                    for search_radius in 0..=3 {
                        let min_y = (search_y - search_radius).max(0);
                        let max_y = (search_y + search_radius).min(tile_height as i32 - 1);
                        let min_x = (search_x - search_radius).max(0);
                        let max_x = (search_x + search_radius).min(tile_width as i32 - 1);
                        let mut local_found = None;
                        for ay in min_y..=max_y {
                            for ax in min_x..=max_x {
                                let idx = ay as usize * tile_width + ax as usize;
                                if floor_map[idx] {
                                    local_found = Some((ax, ay));
                                    break;
                                }
                            }
                            if local_found.is_some() {
                                break;
                            }
                        }
                        if let Some(found) = local_found {
                            found_anchor = Some(found);
                            break;
                        }
                    }
                    if found_anchor.is_some() {
                        break;
                    }
                }
                let Some((anchor_x, anchor_y)) = found_anchor else {
                    continue;
                };
                let dx = anchor_x - center_x;
                let dy = anchor_y - center_y;
                let radius_pct = (((dx * dx + dy * dy) as f32).sqrt() / max_radius as f32 * 100.0)
                .round()
                .clamp(0.0, 100.0) as i32;
                let anchor_gpos = GlobalTilePos::new(
                    origin_tile.x() + anchor_x,
                    origin_tile.y() + anchor_y,
                );
                ring_spawn_anchors.push((anchor_gpos, format!("ring_{ring_idx}_pack_{pack_idx}"), radius_pct));
            }
        }
        for (anchor_gpos, ring_key, radius_pct) in ring_spawn_anchors.iter() {
            let queued = super::super::dungeoning_utils::queue_room_spawn_instance_message(
                "center_spiral",
                *anchor_gpos,
                build_order.dimension_ref,
                Some(*radius_pct),
                &mut beings_remaining,
                &room_spawn_config,
                &room_pack_spawn.source_lookup,
                &mut room_pack_spawn.pending_messages,
                &mut rng,
            );
            if queued {
                trace!(
                    target: DUNGEONING_SYSTEM,
                    "Queued room_spawn InstancePack for structure={} ring={} radius_pct={:.1} at {}",
                    structured_gen_cfg.structure_id(),
                    ring_key,
                    radius_pct,
                    *anchor_gpos,
                );
            }
        }
        let center_radius_pct = 0_i32;
        let queued = super::super::dungeoning_utils::queue_room_spawn_instance_message(
            "center_spiral",
            center_anchor_gpos,
            build_order.dimension_ref,
            Some(center_radius_pct),
            &mut beings_remaining,
            &room_spawn_config,
            &room_pack_spawn.source_lookup,
            &mut room_pack_spawn.pending_messages,
            &mut rng,
        );
        if queued {
            trace!(
                target: DUNGEONING_SYSTEM,
                "Queued room_spawn InstancePack for structure={} shape=center_spiral radius_pct={:.1} at {}",
                structured_gen_cfg.structure_id(),
                center_radius_pct,
                center_anchor_gpos,
            );
        }

        seal_structure_border_band(
            &mut floor_map,
            None,
            tile_width,
            tile_height,
            border_seal_margin,
        );

        let mut boulder_anchor_map: Vec<Option<TileRef>> = vec![None; tile_map_size];
        if boulder_frequency > 0.0 {
            let boulder_sampler: Option<&HashIdWeightedSampler> = sampler_map
                .0
                .get_cloned(boulder_sampler_id)
                .ok()
                .and_then(|sampler_ent| sampler_query.get(sampler_ent).ok());
            if let Some(boulder_sampler) = boulder_sampler {
                for y in min_y as usize..=max_y as usize {
                    let row_idx = y * tile_width;
                    for x in min_x as usize..=max_x as usize {
                        let idx = row_idx + x;
                        if floor_map[idx] && !wall_map[idx] {
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
                        &sampler_map,
                        &templs_map,
                        anchor_gpos,
                        settings,
                        dimension_hash,
                    ) else {
                        continue;
                    };
                    let Ok(&sampled_boulder_hash) = templ_hash_query.get(sampled_boulder_ent) else {
                        continue;
                    };
                    let size = templ_size_query
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
                            if !floor_map[idx] || wall_map[idx] || blocked_map[idx] {
                                can_place = false;
                                break 'footprint;
                            }
                        }
                    }
                    if !can_place {
                        continue;
                    }
                    boulder_anchor_map[y * tile_width + x] = Some(TileRef(sampled_boulder_hash));
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
                warn!(target: "dungeoning", "Boulder sampler '{:?}' missing/invalid for archimedes spiral dungeon", boulder_sampler_id);
            }
        }

        let floor_delete_other_tiles = delete_other_tiles_by_tile_id.get("floor_tile_id");
        let disable_floor_terrgen = terrgen_disable_by_tile_id.should_disable_for("floor_tile_id");
        let forced_chunk_biomes = super::super::dungeoning_utils::forced_chunk_biomes_from_args(&structured_gen_cfg.typed_args, &biome_map);
        let mut chunk_tiles: Vec<(GlobalTilePos, TileRef, Option<DeleteOtherTilesInSamePos>)> = Vec::with_capacity(chunk_positions.len());
        let mut terrgen_disabled_gpos_for_chunks = TerrGenDisabledGposForChunks::default();
        for &chunk_pos in chunk_positions {
            tiles4chunk.clear();
            let mut blocked_gpos = TerrGenBlockedGposMask::default();
            for tile_pos in chunk_pos.get_tilepositions_within_chunk() {
                let local_tile = tile_pos.0 - origin_tile.0;
                if local_tile.x < 0 || local_tile.y < 0 { continue; }
                let idx_x = local_tile.x as usize;
                let idx_y = local_tile.y as usize;
                if idx_x >= tile_width || idx_y >= tile_height { continue; }
                let map_idx = idx_y * tile_width + idx_x;
                if let Some(boulder_entity) = boulder_anchor_map[map_idx] {
                    if floor_map[map_idx] {
                        tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                        if disable_floor_terrgen {
                            let size = templ_size_query.get(floor_entity_ent).copied().unwrap_or_default().inner();
                            extend_occupied_gpos(&mut blocked_gpos, chunk_pos, tile_pos, size);
                        }
                    }
                    tiles4chunk.push((tile_pos, boulder_entity, None));
                } else if wall_map[map_idx] {
                    tiles4chunk.push((tile_pos, wall_entity, None));
                } else if floor_map[map_idx] {
                    tiles4chunk.push((tile_pos, floor_entity, floor_delete_other_tiles.clone()));
                    if disable_floor_terrgen {
                        let size = templ_size_query.get(floor_entity_ent).copied().unwrap_or_default().inner();
                        extend_occupied_gpos(&mut blocked_gpos, chunk_pos, tile_pos, size);
                    }
                }
            }
            chunk_tiles.extend(tiles4chunk.drain(..));
            terrgen_disabled_gpos_for_chunks.insert_for_chunk(chunk_pos, blocked_gpos);
        }
        info!(target: DUNGEONING_SYSTEM, "structure={} pushing compliance blocked_terrgen_gpos={}", structured_gen_cfg.structure_id(), terrgen_disabled_gpos_for_chunks.count_blocked());
        compliances_to_emit.push(StructureBuildCompliance {
            i: build_order.i,
            structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
            dimension_ref: build_order.dimension_ref,
            chunk_tiles,
            terrgen_disabled_gpos_for_chunks,
            forced_chunk_biomes,
        });
    }

    writer.write_batch(compliances_to_emit.drain(..));
    room_pack_spawn.finish_pass();
}
