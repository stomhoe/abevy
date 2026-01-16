
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
pub fn example_emit_claims_system(
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
        
        let claimed = ClaimedChunks {
            i: claim_request.i,
            region_ent: claim_request.region_ent,
            structured_gen_cfg_ent: claim_request.structured_gen_cfg_ent,
            chunks_gpos: vec![claim_request.start_gpos],
            partition_tolerant: false,
        };
        claims_to_emit.push(claimed);
        trace!(target: "structure_spawn", "Emitting ClaimedChunks for ExampleStructure in region for chunk position {:?}", claim_request.start_gpos);
    }
    writer.write_batch(claims_to_emit);
}


#[allow(unused_parens)]
pub fn example_building_system(
    mut reader: MessageReader<StructureBuildOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut writer: MessageWriter<StructureBuildCompliance>,
    ezeros_map: Res<TileEzerosMap>,
) {
    
    let mut compliances_to_emit = Vec::new();    
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { continue; };
        
        if structured_gen_cfg.structure_id != "ExampleStructure" {
            continue;
        }
        
        let tile_ezero_id = "cyan";
        let Ok(ezero_ref) = ezeros_map.0.get(&tile_ezero_id)
        else {
            error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when spawning ExampleStructure, skipping structure spawn", tile_ezero_id);
            continue;
        };
        
        let ezero_ref = EntityZeroRef(ezero_ref);
        
        
        for &chunk_pos in &build_order.chunks_gpos {
            
            let mut tiles4chunk: TilesFromBuilder = Vec::new();
            //todo hacer que se haga después de q se spawneen las tiles de terrgen
            for tpos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()){
                tiles4chunk.push((tpos, ezero_ref, Some(DeleteOthersExceptZLevels::default())));
                
            }
            compliances_to_emit.push(StructureBuildCompliance {
                structure_gen_cfg_ent: build_order.structured_gen_cfg_ent,
                dimension_ref: build_order.dimension_ref,
                chunk_pos,
                tiles: tiles4chunk,
            });
            debug!(target: "structure_spawn", "Spawned ExampleStructure in region at {:?} occupying chunks: {:?}",
            build_order.region_pos, build_order.chunks_gpos);
        }
        
    }
    writer.write_batch(compliances_to_emit);
}

#[allow(unused_parens)]
pub fn despawn_empty_regions(mut cmd: Commands, 
    query: Query<(Entity),(With<Region>, Without<ChunksActiveInRegion>)>
){
    query.iter().for_each(|region_ent| cmd.entity(region_ent).try_despawn());
}
