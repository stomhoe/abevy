
use std::mem::take;

use bevy::{ecs::entity::EntityHashMap, platform::collections::{HashMap, HashSet}, prelude::*, ui::debug};

use common::common_components::HashId;
use dimension_shared::{BlacklistedStructureGenTags, DimensionRef, MultipleDimensionRefs, WhitelistedStructureGenTags};
use game_common::{game_common_components::{EntityZeroRef, TagHashSet}, game_common_components_samplers::EntityWeightedSampler};
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{chunking_components::{Chunk, StructureSpawningChecked}, regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, OfferChunk, StructureBuildOrder}}, tile::{tile_components::DeleteOthersExceptZLevels, tile_resources::TileEzerosMap}, tilemap_resources::MassCollectedTiles};

use bit_vec::BitVec;

#[allow(unused_parens)]
pub fn offer_chunks_of_new_region(mut cmd: Commands, 
    settings: Single<&AcGlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<StructuredGenConfigWeightedMap>>,
    region_query: Query<(Entity, &RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    structured_gens: Query<(Option<&TagHashSet>, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    dimension_query: Query<(Option<&WhitelistedStructureGenTags>, Option<&BlacklistedStructureGenTags>),()>,
    mut writer: MessageWriter<OfferChunk>,
) {
    
    let mut to_write = Vec::new();
    
    for (region_ent, &region_pos, dim_ref) in region_query.iter() {
        
        let Ok((dim_wlist_tags, dim_blist_tags)) = dimension_query.get(dim_ref.parent())
        else {
            error!(target: "structure_spawn", "Dimension entity {:?} not found when requesting chunk claims for region at position ({}, {}), skipping region", 
            dim_ref.parent(), region_pos.0.x, region_pos.0.y);
            continue;
        };
        
        let rng = region_pos.hash_value(&settings, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
        

        let mut offered_chunks_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area(), false);
        
        let mut claim_i = 0;
        let mut reattempt_count = 0;
        const MAX_REATTEMPTS: usize = 50;
        'nextattempt: while claim_i < MAX_CLAIMS as u64 {
            if reattempt_count >= MAX_REATTEMPTS as u64 {
                //safety break to avoid infinite loops
                break 'nextattempt;
            }
            
            let Some(structured_gen_cfg_ent) = weight_map.sample_with_rng(&mut rng)
            else { 
                error!(target: "structure_spawn", "No StructuredGenConfig available to spawn structure in region at position ({}, {})", region_pos.0.x, region_pos.0.y);
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
            } else{
                if let Some(dim_blist_tags) = dim_blist_tags {
                    if let Some(strgen_tags) = strgen_tags {
                        if strgen_tags.intersects(&dim_blist_tags.0) {
                            reattempt_count += 1;
                            continue 'nextattempt;
                        }
                    }
                }
                if let Some(dim_wlist_tags) = dim_wlist_tags {
                    if let Some(strgen_tags) = strgen_tags {
                        if ! strgen_tags.intersects(&dim_wlist_tags.0) {
                            reattempt_count += 1;
                            trace!(target: "structure_spawn", "Dimension entity {:?} does not whitelist any tags of structure '{:?}', skipping", dim_ref.parent(), structured_gen_cfg_ent);
                            continue 'nextattempt;
                        }
                    } else {
                        reattempt_count += 1;
                        trace!(target: "structure_spawn", "Dimension entity {:?} does not whitelist any tags of structure '{:?}', skipping", dim_ref.parent(), structured_gen_cfg_ent);
                        continue 'nextattempt;
                    }
                }
            }
            let mut available_indices = Vec::new();
            for i in 0..REGION_SIZE_IN_CHUNKS.area() {
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
                    debug!(target: "structure_spawn", "Random chunk position {:?} within region at {} rejected by PoissonDisk for structure '{:?}', reattempting", rand_chunk_pos_within_region, region_pos, structured_gen_cfg_ent);
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
            debug!(target: "structure_spawn", "Emitting RequestChunkClaim for structure '{:?}' in region at {} for chunk position {:?}", structured_gen_cfg_ent, region_pos, rand_chunk_pos_within_region);
            
            claim_i += 1;
            reattempt_count = 0;
        }
        
    }
    writer.write_batch(to_write);
}

#[allow(unused_parens, )]
pub fn clonespawn_structure_tile_on_chunk_spawn(mut cmd: Commands, 
    mut query: Query<(Entity, &Chunk, &ChunkPos),(Without<StructureSpawningChecked>)>,
    region_query: Query<(&TilesToSpawnPerChunk),()>,
    mut collected: ResMut<MassCollectedTiles>,

) {
    let mut to_insert_checked = Vec::new();
    let mut to_insert_delete_others = Vec::new();
    for (chunk_ent, chunk, chunk_pos) in query.iter_mut() {
        let Ok((tiles_to_spawn_per_chunk)) = region_query.get(chunk.region_ent)
        else {
            error!(target: "structure_spawn", "Region entity {:?} not found when spawning structure tiles for chunk at position {:?}, skipping structure tile spawn",
            chunk.region_ent, chunk_pos);
            to_insert_checked.push((chunk_ent, StructureSpawningChecked, ));

            continue;
        };

        if let Some(tiles_to_spawn) = tiles_to_spawn_per_chunk.get(&chunk_pos) {
            debug!(target: "structure_spawn", "Spawning {} structure tiles in chunk at {:?}", tiles_to_spawn.len(), chunk_pos);
            for (tile_gpos, ezero_ref, delete_others) in tiles_to_spawn {
                let tile_ent = collected.clonespawn_and_push_tile(&mut cmd, *ezero_ref, *tile_gpos, DimensionRef(chunk.region_ent), OplistSize::default(),);
                if let Some(delete_others) = delete_others {
                    to_insert_delete_others.push((tile_ent, delete_others.clone(), ));
                }
            }
        }
        else {
            debug!(target: "structure_spawn", "No structure tiles to spawn in chunk at {:?}", chunk_pos);
        }
        to_insert_checked.push((chunk_ent, StructureSpawningChecked, ));
    }
    cmd.try_insert_batch(to_insert_checked);
    cmd.try_insert_batch(to_insert_delete_others);
}


#[allow(unused_parens)]
pub fn read_chunk_claims_for_region_and_emit_build_orders(
    mut reader: MessageReader<ClaimedChunks>,
    mut region_query: Query<(&RegionPos, &mut RegionStructures, ),()>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    
    mut writer: MessageWriter<StructureBuildOrder>,
) {
    unsafe{
        let mut regions_with_new_claims: Vec<Entity> = Vec::new();
        let mut build_orders = Vec::new();
        
        for claim in reader.read() {
            let Ok((_, mut region_structures, )) = region_query.get_mut(claim.region_ent)
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
            let Ok((&region_pos, mut region_structures, )) = region_query.get_mut(region_ent)
            else {
                continue;
            };
            'nextregion: for i in region_structures.processed_up_to_i..MAX_CLAIMS {
                
                if region_structures.occupation_grid.occupied_count() >= max_used_chunks_per_region as u32 {
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
                let mut non_claimed_chunks_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area(), false);
                
                'nextpos: for (i, &chunk_pos) in claimed.chunks_gpos.iter().enumerate(){
                    match (region_structures.occupation_grid.occupy(
                        chunk_pos,
                        region_pos,
                        claimed.structured_gen_cfg_ent,
                    ), claimed.partition_tolerant) {
                        (Ok(()), _) => {
                            debug!(target: "structure_spawn", "Successfully claimed chunk at {:?} in region {:?} for structure '{}'", chunk_pos, region_pos, structured_gen_cfg.structure_id);
                            claimed_up_to += 1;
                        }
                        (Err(ChunkOccupyError::OutOfRegionBounds(_)), false) => {
                            undo_claims = true;
                            error!(target: "structure_spawn", "Chunk at {:?} is outside region bounds, undoing all claims for this structure", 
                            chunk_pos);
                            break 'nextpos;
                        }
                        (Err(ChunkOccupyError::OutOfRegionBounds(_)), true) => {
                            undo_claims = true;
                            error!(target: "structure_spawn", "Chunk at {:?} is outside region bounds, undoing all claims for this structure", 
                            chunk_pos);
                            break 'nextpos;
                        }
                        (Err(ChunkOccupyError::AlreadyOccupied(_)), true) => {
                            trace!(target: "structure_spawn", "Chunk at {:?} in region {:?} already occupied, but claim is partition tolerant, continuing", 
                            chunk_pos, region_pos);
                            non_claimed_chunks_bitmask.set(i, true);
                            claimed_up_to += 1;
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
                        region_structures.occupation_grid.free(chunk_pos, region_pos);
                    }
                } else{
                    region_structures.struct_gen_counts.entry(claimed.structured_gen_cfg_ent)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                    
                    for i in (0..claimed.chunks_gpos.len()).rev() {
                        if (non_claimed_chunks_bitmask.get(i).unwrap_or(false)) {
                            claimed.chunks_gpos.swap_remove(i);
                        }
                    }
                    
                    let order = StructureBuildOrder {
                        i: claimed.i,
                        region_ent: claimed.region_ent,
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
pub fn example_emit_claims_system(mut cmd: Commands, 
    mut reader: MessageReader<OfferChunk>,
    region_query: Query<(&RegionPos, ),()>,
    mut writer: MessageWriter<ClaimedChunks>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    
) {
    let mut claims_to_emit = Vec::new();
    for claim_request in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(claim_request.structured_gen_cfg_ent)
        else { continue; };
        
        if structured_gen_cfg.structure_id != "ExampleStructure" {
            trace!(target: "structure_spawn", "StructuredGenConfig entity {:?} is not for ExampleStructure, skipping", claim_request.structured_gen_cfg_ent);
            continue;
        }
        
        let Ok((region_pos, )) = region_query.get(claim_request.region_ent)
        else {
            trace!(target: "structure_spawn", "Region entity {:?} not found when processing ExampleStructure claim, skipping", claim_request.region_ent);
            continue;
        };
        let region_bounds = region_pos.chunk_bounds();
        let start_pos = claim_request.start_gpos;
        
        let mut chunk_gpos = Vec::new();
        chunk_gpos.push(start_pos);
        
        
        let claimed = ClaimedChunks {
            i: claim_request.i,
            region_ent: claim_request.region_ent,
            structured_gen_cfg_ent: claim_request.structured_gen_cfg_ent,
            chunks_gpos: chunk_gpos,
            partition_tolerant: false,
        };
        claims_to_emit.push(claimed);
        trace!(target: "structure_spawn", "Emitting ClaimedChunks for ExampleStructure in region at {} for chunk position {:?}", region_pos, start_pos);
    }
    writer.write_batch(claims_to_emit);
}


#[allow(unused_parens)]
pub fn example_building_system(mut cmd: Commands, 
    mut reader: MessageReader<StructureBuildOrder>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    mut region_query: Query<(&ChildOf, &mut TilesToSpawnPerChunk),()>,
    
    ezeros_map: Res<TileEzerosMap>,
) {
    for build_order in reader.read() {
        let Ok((structured_gen_cfg,)) = structured_gens.get(build_order.structured_gen_cfg_ent) 
        else { continue; };
        
        if structured_gen_cfg.structure_id != "ExampleStructure" {
            continue;
        }
        let Ok((child_of, mut tiles_to_spawn_per_chunk)) = region_query.get_mut(build_order.region_ent)
        else {
            continue;
        };
        
        let dim_ref = DimensionRef(child_of.parent());
        
        let tile_ezero_id = "cyan";
        let Ok(ezero_ref) = ezeros_map.0.get(&tile_ezero_id)
        else {
            error!(target: "structure_spawn", "TileEzero with id '{}' not found in TileEzerosMap when spawning ExampleStructure, skipping structure spawn", tile_ezero_id);
            continue;
        };
        
        let ezero_ref = EntityZeroRef(ezero_ref);
        
        for &chunk_pos in &build_order.chunks_gpos {
            //todo hacer que se haga después de q se spawneen las tiles de terrgen
            for tpos in chunk_pos.get_tilepositions_within_chunk(OplistSize::default()){


                tiles_to_spawn_per_chunk.push_one_unchecked(chunk_pos, tpos, ezero_ref, Some(DeleteOthersExceptZLevels::default()));
            }
        }
        debug!(target: "structure_spawn", "Spawned ExampleStructure in region at {:?} occupying chunks: {:?}",
        build_order.region_ent, build_order.chunks_gpos);
        
    }
}


