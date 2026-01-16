
use std::{mem::take};
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_components::HashId;
use dimension_shared::{BlacklistedStructureGenTags, DimensionRef, MultipleDimensionRefs, WhitelistedStructureGenTags};
use game_common::{game_common_components::{EntityZeroRef, TagHashSet}, game_common_components_samplers::EntityWeightedSampler};
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{chunking_components::{Chunk, StructureSpawnDone}, regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, OfferChunk, StructureBuildCompliance, StructureBuildOrder}, regioning_resources::LoadedRegions}, tile::{tile_components::DeleteOthersExceptZLevels, tile_resources::TileEzerosMap}, tilemap_resources::MassCollectedTiles};

use bit_vec::BitVec;

/// Check if a structure's tags pass dimension whitelist/blacklist filters
#[inline]
fn passes_dimension_tag_filters(
    strgen_tags: Option<&TagHashSet>,
    dim_wlist_tags: Option<&WhitelistedStructureGenTags>,
    dim_blist_tags: Option<&BlacklistedStructureGenTags>,
) -> bool {
    // Check blacklist first
    if let Some(dim_blist_tags) = dim_blist_tags {
        if let Some(strgen_tags) = strgen_tags {
            if strgen_tags.intersects(&dim_blist_tags.0) {
                return false;
            }
        }
    }
    
    // Check whitelist
    if let Some(dim_wlist_tags) = dim_wlist_tags {
        if let Some(strgen_tags) = strgen_tags {
            return strgen_tags.intersects(&dim_wlist_tags.0);
        } else {
            return false;
        }
    }
    
    true
}


#[allow(unused_parens)]
pub fn offer_chunks_of_new_region(
    mut cmd: Commands,
    settings: Single<&GlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<StructuredGenCfgsWeightedMap>>,
    region_query: Query<(Entity, &RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    structured_gens: Query<(Option<&TagHashSet>, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    dimension_query: Query<(Option<&WhitelistedStructureGenTags>, Option<&BlacklistedStructureGenTags>),()>,
    mut writer: MessageWriter<OfferChunk>,
) {
    let mut to_write = Vec::new();
    
    for (region_ent, &region_pos, dim_ref) in region_query.iter() {
        info!(target: "structure_spawn", "Offering chunks for new region at position ({}, {})", region_pos.0.x, region_pos.0.y);
        
        let Ok((dim_wlist_tags, dim_blist_tags)) = dimension_query.get(dim_ref.parent())
        else {
            error!(target: "structure_spawn", "Dimension entity {:?} not found when requesting chunk claims for region at position ({}, {}), skipping region", 
            dim_ref.parent(), region_pos.0.x, region_pos.0.y);
            continue;
        };
        
        let rng = region_pos.hash_value(&settings, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
        
        
        let mut offered_chunks_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area() as usize, false);
        
        let mut claim_i = 0;
        let mut reattempt_count = 0;
        const MAX_REATTEMPTS: usize = 50;
        'nextattempt: while claim_i < MAX_CLAIMS as u64 {
            if reattempt_count >= MAX_REATTEMPTS as u64 {
                //safety break to avoid infinite loops
                if to_write.is_empty() {
                    error!(target: "structure_spawn", "No structures could be offered for region at {}, stopping attempts", region_pos);
                    cmd.entity(region_ent).try_insert(RegionPlanningFinished);
                } 
                break 'nextattempt;
            }
            
            let Some(structured_gen_cfg_ent) = weight_map.sample_with_rng(&mut rng)
            else { 
                error!(target: "structure_spawn", "No StructuredGenConfig available to spawn structure in region at {}", region_pos);
                break; 
            };
            let Ok((strgen_tags, poisson_disk, exclusive_for_dimensions)) = structured_gens.get(structured_gen_cfg_ent)
            else {
                reattempt_count += 1;
                continue 'nextattempt;
            };
            if let Some(exclusive_for_dimensions) = exclusive_for_dimensions {
                if ! exclusive_for_dimensions.0.contains(&dim_ref.parent()) {
                    reattempt_count += 1;
                    trace!(target: "structure_spawn", "Dimension entity {:?} is not in exclusive dimension list for structure '{:?}', skipping", dim_ref.parent(), structured_gen_cfg_ent);
                    continue 'nextattempt;
                }
            } else if !passes_dimension_tag_filters(strgen_tags, dim_wlist_tags, dim_blist_tags) {
                reattempt_count += 1;
                trace!(target: "structure_spawn", "Dimension entity {:?} fails tag filter checks for structure '{:?}', skipping", dim_ref.parent(), structured_gen_cfg_ent);
                continue 'nextattempt;
            }
            let mut available_indices = Vec::new();
            for i in 0..REGION_SIZE_IN_CHUNKS.area() as usize {
                if ! offered_chunks_bitmask.get(i).unwrap_or(false) {
                    available_indices.push(i);
                }
            }
            
            if available_indices.is_empty() {
                break 'nextattempt;
            }
            
            let chunk_index = available_indices[rng.random_range(0..available_indices.len())];
            
            let rand_chunk_pos_within_region = ChunkPos::chunk_pos_from_flat_index_within_region(chunk_index, region_pos);
            
            
            if let Some(poisson_disk) = poisson_disk {
                
                if ! poisson_disk.is_allowed_position(&settings, rand_chunk_pos_within_region, false, OplistSize::default()) {
                    trace!(target: "structure_spawn", "Random chunk position {:?} within region at {} rejected by PoissonDisk for structure '{:?}', reattempting", rand_chunk_pos_within_region, region_pos, structured_gen_cfg_ent);
                    reattempt_count += 1;
                    continue 'nextattempt;
                }
            }
            offered_chunks_bitmask.set(chunk_index, true);
            
            to_write.push(OfferChunk {
                i: claim_i,
                region_ent,
                structured_gen_cfg_ent,
                start_gpos: rand_chunk_pos_within_region,
            });
            debug!(target: "structure_spawn", "Emitting RequestChunkClaim for structure '{:?}' in region at {} for {}", structured_gen_cfg_ent, region_pos, rand_chunk_pos_within_region);
            
            claim_i += 1;
            reattempt_count = 0;
        }
        
    }
    writer.write_batch(to_write);
}


#[allow(unused_parens)]
pub fn track_when_region_is_ready_for_spawning(mut cmd: Commands, 
    loaded_regions: Res<LoadedRegions>,
    mut region_query: Query<(&mut RegionPlannedTiles, Has<RegionPlanningFinished>),()>,
    mut reader: MessageMutator<StructureBuildCompliance>,
) {
    for build in reader.read() {
        
        let region_pos = build.chunk_pos.to_region_pos();
        
        let Some(&region_ent) = loaded_regions.0.get(&(build.dimension_ref, region_pos))
        else {
            error!(target: "structure_spawn", "Region at position {:?} in dimension {:?} not found when processing structure build compliance, skipping", 
            region_pos, build.dimension_ref);
            continue;
        };
        
        let Ok((mut planned_tiles, region_planning_already_done)) = region_query.get_mut(region_ent)
        else {
            continue;
        };
        if region_planning_already_done {
            error!(target: "structure_spawn", "Region entity {:?} at position {:?} in dimension {:?} already marked as RegionPlanningFinished when processing structure build compliance, skipping",
            region_ent, region_pos, build.dimension_ref);
            continue;
        }
        
        let Ok(finished) = planned_tiles.add_planned_tiles(build.chunk_pos, take(&mut build.tiles))
        else {
            error!(target: "structure_spawn", "Failed to add planned tiles for structure build compliance in region entity {:?} at chunk position {:?}, skipping", region_ent, build.chunk_pos);
            continue;
        };
        
        if finished {
            info!(target: "structure_spawn", "Region entity {:?} has finished planning all structure tiles, marking as RegionPlanningFinished", region_ent);
            cmd.entity(region_ent).try_insert(RegionPlanningFinished);
        }
        
    }
}


#[allow(unused_parens, )]
pub fn clonespawn_structure_tile_on_chunk_spawn(mut cmd: Commands, 
    region_query: Query<(&ChunksActiveInRegion, &RegionPlannedTiles),(With<RegionPlanningFinished>)>,
    chunk_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<StructureSpawnDone>)>,
    mut collected: ResMut<MassCollectedTiles>,
) {
    let mut to_insert_structure_spawning_done = Vec::new();
    let mut to_insert_delete_others = Vec::new();
    
    for (chunks_active_in_region, tiles_to_spawn_per_chunk) in region_query.iter() {
        chunk_query.iter_many(chunks_active_in_region.entities()).for_each(|(chunk_ent, chunk_pos, dimension_ref)| {
            
            if let Some(tiles_to_spawn) = tiles_to_spawn_per_chunk.get(&chunk_pos) {
                debug!(target: "structure_spawn", "Spawning {} structure tiles in chunk at {:?}", tiles_to_spawn.len(), chunk_pos);
                for (tile_gpos, ezero_ref, delete_others) in tiles_to_spawn {
                    let tile_ent = collected.clonespawn_and_push_tile(&mut cmd, *ezero_ref, *tile_gpos, *dimension_ref, OplistSize::default());
                    if let Some(delete_others) = delete_others {
                        to_insert_delete_others.push((tile_ent, (delete_others.clone(), StructureSpawnDone)));
                    }
                }
            } else {
                debug!(target: "structure_spawn", "No structure tiles to spawn in chunk at {:?}", chunk_pos);
            }
            to_insert_structure_spawning_done.push((chunk_ent, StructureSpawnDone));//hacerlo en otro sistema una vez se detecte el spawneo de la tile con structurespawningdone
        }
    );
}
cmd.try_insert_batch(to_insert_structure_spawning_done);
cmd.try_insert_batch(to_insert_delete_others);
}


#[allow(unused_parens)]
pub fn read_chunk_claims_for_region_and_emit_build_orders(
    mut reader: MessageReader<ClaimedChunks>,
    mut region_query: Query<(&RegionPos, &DimensionRef, &mut RegionStructures, &mut RegionPlannedTiles),()>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    
    mut writer: MessageWriter<StructureBuildOrder>,
) {
    unsafe{
        let mut regions_with_new_claims: Vec<Entity> = Vec::new();
        let mut build_orders = Vec::new();
        
        for claim in reader.read() {
            let Ok((_, _, mut region_structures, _)) = region_query.get_mut(claim.region_ent)
            else {
                continue;
            };
            region_structures.claims[claim.i as usize] = Some(claim.clone());
            
            if !regions_with_new_claims.contains(&claim.region_ent) {
                regions_with_new_claims.push(claim.region_ent);
            }
        }
        let max_used_chunks_per_region = (REGION_SIZE_IN_CHUNKS.0.element_product() as f32 * 0.1) as u16;
        
        
        for region_ent in regions_with_new_claims {
            let Ok((&region_pos, &dimension_ref, mut region_structures, mut planned)) = region_query.get_mut(region_ent)
            else {
                continue;
            };
            'nextregion: for i in region_structures.processed_up_to_i..MAX_CLAIMS {
                
                if region_structures.strgen_grid.occupied_count() >= max_used_chunks_per_region as u32 {
                    break 'nextregion;
                }
                let Some(claimed) = region_structures.claims.get_unchecked_mut(i) else {
                    break 'nextregion;//ta bien, no hay que seguir hasta que aparezca la region structure en esta posicion
                };
                let mut claimed = take(claimed);
                
                let Ok((structured_gen_cfg,)) = structured_gens.get(claimed.structured_gen_cfg_ent)
                else {
                    region_structures.processed_up_to_i += 1; 
                    error!(target: "structure_spawn", "StructuredGenConfig entity {:?} not found when processing claims for region at {:?}, skipping claim", 
                    claimed.structured_gen_cfg_ent, region_pos);
                    continue;
                };
                
                if  region_structures.struct_gen_counts.get(&claimed.structured_gen_cfg_ent).copied().unwrap_or(0) >= structured_gen_cfg.max_per_region  {
                    region_structures.processed_up_to_i += 1; 
                    debug!(target: "structure_spawn", "Max structures of type '{}' already spawned in region {:?}, skipping claim", 
                    structured_gen_cfg.structure_id, region_pos);
                    continue;
                }
                let mut undo_claims = false;
                let mut claimed_up_to: u64 = 0;
                let mut failed_claims_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area() as usize, false);
                
                'nextpos: for (claim_i, &chunk_pos) in claimed.chunks_gpos.iter().enumerate(){
                    match (region_structures.strgen_grid.occupy(
                        chunk_pos,
                        region_pos,
                        claimed.structured_gen_cfg_ent,
                    ), claimed.partition_tolerant) {
                        (Ok(()), _) => {
                            debug!(target: "structure_spawn", "Successfully claimed chunk at {:?} in region {:?} for structure '{}'", chunk_pos, region_pos, structured_gen_cfg.structure_id);
                            claimed_up_to += 1;
                        }
                        (Err(ChunkOccupyError::OutOfRegionBounds(_)), _) => {
                            undo_claims = true;
                            error!(target: "structure_spawn", "Chunk at {:?} is outside region bounds, undoing all claims for this structure", 
                            chunk_pos);
                            break 'nextpos;
                        }
                        (Err(ChunkOccupyError::AlreadyOccupied(_)), true) => {
                            trace!(target: "structure_spawn", "Chunk at {:?} in region {:?} already occupied, but claim is partition tolerant, continuing", 
                            chunk_pos, region_pos);
                            failed_claims_bitmask.set(claim_i, true);//OK
                            continue 'nextpos;
                        }
                        (Err(ChunkOccupyError::AlreadyOccupied(_)), false) => {
                            undo_claims = true;
                            trace!(target: "structure_spawn", "Chunk at {:?} in region {:?} already occupied, undoing all claims for this structure", 
                            chunk_pos, region_pos);
                            break 'nextpos;
                        }
                    }
                    
                }
                if undo_claims {
                    for i in 0..claimed_up_to {
                        let chunk_pos = claimed.chunks_gpos[i as usize];
                        region_structures.strgen_grid.free(chunk_pos, region_pos);
                    }
                } else {
                    region_structures.struct_gen_counts.entry(claimed.structured_gen_cfg_ent)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                    
                    for i in (0..claimed.chunks_gpos.len()).rev() {
                        if failed_claims_bitmask.get(i).unwrap_or(false) {
                            claimed.chunks_gpos.swap_remove(i);
                        }
                    }
                    
                    planned.extend_pending_chunks(&claimed.chunks_gpos);
                    
                    let order = StructureBuildOrder {
                        i: claimed.i,
                        region_pos,
                        dimension_ref,
                        structured_gen_cfg_ent: claimed.structured_gen_cfg_ent,
                        chunks_gpos: claimed.chunks_gpos,
                    };
                    build_orders.push(order);
                }
                region_structures.processed_up_to_i += 1; 
            }
            
        }
        writer.write_batch(build_orders);
    }
}


#[allow(unused_parens)]
pub fn example_emit_chunk_claims_system(
    mut reader: MessageReader<OfferChunk>,
    mut writer: MessageWriter<ClaimedChunks>,
    structured_gens: Query<(&StructuredGenConfig,)>,
) {
    let mut claims_to_emit = Vec::new();
    for claim_request in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(claim_request.structured_gen_cfg_ent)
        else { continue; };
        
        if structured_gen_cfg.structure_id != "ExampleStructure" {
            trace!(target: "structure_spawn", "StructuredGenConfig entity {:?} is not for ExampleStructure, skipping", claim_request.structured_gen_cfg_ent);
            continue;
        }

        let center_chunk = claim_request.start_gpos;
        let parity_seed = (center_chunk.x() as i64).abs() + (center_chunk.y() as i64).abs();
        let side_length = 3 + (parity_seed % 2) as i32;
        let half_spread = side_length / 2;
        let start_offset = -half_spread;
        let end_offset = start_offset + side_length - 1;
        let region_pos = center_chunk.to_region_pos();

        let mut chunk_positions = Vec::new();
        for dy in start_offset..=end_offset {
            for dx in start_offset..=end_offset {
                let candidate = center_chunk + IVec2::new(dx, dy);
                if region_pos.contains_chunkpos(candidate) {
                    chunk_positions.push(candidate);
                }
            }
        }

        if chunk_positions.is_empty() {
            warn!(target: "structure_spawn", "No eligible chunks around {:?} for ExampleStructure, skipping", center_chunk);
            continue;
        }
        chunk_positions.sort_unstable_by_key(|chunk| (chunk.y(), chunk.x()));
        let chunk_count = chunk_positions.len();
        claims_to_emit.push(ClaimedChunks {
            i: claim_request.i,
            region_ent: claim_request.region_ent,
            structured_gen_cfg_ent: claim_request.structured_gen_cfg_ent,
            chunks_gpos: chunk_positions,
            partition_tolerant: false,
        });
        trace!(target: "structure_spawn", "Emitting ClaimedChunks for ExampleStructure covering {} chunks around {:?}", chunk_count, center_chunk);
    }
    writer.write_batch(claims_to_emit);
}


#[allow(unused_parens)]
pub fn drunkwalk_building_system(
    mut reader: MessageReader<StructureBuildOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
    settings: Single<&GlobalGenSettings>,
) {
    
    let mut compliances_to_emit = Vec::new();    
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { continue; };
        
        if structured_gen_cfg.structure_id != "ExampleStructure" {
            continue;
        }

        let floor_tile_id = "cyan";
        let floor_entity = match ezeros_map.0.get(&floor_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when spawning ExampleStructure, skipping structure spawn", floor_tile_id);
                continue;
            }
        };

        let wall_tile_id = "purple";
        let wall_entity = match ezeros_map.0.get(&wall_tile_id) {
            Ok(entity) => EntityZeroRef(entity),
            Err(_) => {
                error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when spawning ExampleStructure, skipping structure spawn", wall_tile_id);
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
        let mut seed = build_order.region_pos.hash_value(&settings, 0);
        for chunk_pos in chunk_positions {
            seed = chunk_pos.hash_value(&settings, seed);
        }
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);

        let mut walker_x = tile_width / 2;
        let mut walker_y = tile_height / 2;
        let target_floor_tiles = std::cmp::max(1, ((tile_map_size as f32) * 0.35).ceil() as usize);
        let tile_width_minus_one = tile_width - 1;
        let tile_height_minus_one = tile_height - 1;
        let mut carved = 0;
        while carved < target_floor_tiles {
            let idx = walker_y * tile_width + walker_x;
            if !floor_map[idx] {
                floor_map[idx] = true;
                carved += 1;
            }
            match rng.random_range(0..4) {
                0 => {
                    if walker_x < tile_width_minus_one {
                        walker_x += 1;
                    }
                }
                1 => {
                    if walker_x > 0 {
                        walker_x -= 1;
                    }
                }
                2 => {
                    if walker_y < tile_height_minus_one {
                        walker_y += 1;
                    }
                }
                _ => {
                    if walker_y > 0 {
                        walker_y -= 1;
                    }
                }
            }
        }

        let room_attempts = std::cmp::max(1, tile_map_size / 250);
        for _ in 0..room_attempts {
            let center_x = rng.random_range(0..tile_width);
            let center_y = rng.random_range(0..tile_height);
            let radius = rng.random_range(1..=3) as i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let rx = center_x as i32 + dx;
                    let ry = center_y as i32 + dy;
                    if rx < 0 || ry < 0 {
                        continue;
                    }
                    let rx = rx as usize;
                    let ry = ry as usize;
                    if rx >= tile_width || ry >= tile_height {
                        continue;
                    }
                    floor_map[ry * tile_width + rx] = true;
                }
            }
        }

        let delete_template = DeleteOthersExceptZLevels::default();
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
                let ezero_ref = if floor_map[map_idx] {
                    floor_entity
                } else {
                    wall_entity
                };
                tiles4chunk.push((tile_pos, ezero_ref, Some(delete_template.clone())));
            }
            compliances_to_emit.push(StructureBuildCompliance {
                structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
                dimension_ref: build_order.dimension_ref,
                chunk_pos,
                tiles: tiles4chunk,
            });
        }
        debug!(target: "structure_spawn", "Spawned dungeon for ExampleStructure across {} chunks at {:?}", chunk_positions.len(), build_order.region_pos);
    }
    writer.write_batch(compliances_to_emit);
}

pub fn despawn_empty_regions(mut cmd: Commands, 
    query: Query<(Entity),(With<Region>, Without<ChunksActiveInRegion>)>
){
    query.iter().for_each(|region_ent| cmd.entity(region_ent).try_despawn());
}
