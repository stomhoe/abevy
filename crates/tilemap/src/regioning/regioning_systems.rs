
use std::{mem::take};
#[allow(unused_imports)] use bevy::prelude::*;

use common::common_tag_components::TagSet;
use dimension_shared::{BlacklistedStructureGenTags, DimensionRef, MultipleDimensionRefs, WhitelistedStructureGenTags};
use game_common::{game_common_components_samplers::EntityWeightedSampler};
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{chunking_components::{Chunk, ReadyForTerrgen}, regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, OfferChunk, StructureBuildCompliance, StructurePrepareTilesOrder}, regioning_resources::LoadedRegions, regioning_structured_gen_cfg_components::*}, tilemap_resources::MassCollectedTiles};

use bit_vec::BitVec;

#[inline]
fn passes_dimension_tag_filters(
    strgen_tags: Option<&TagSet>,
    dim_wlist_tags: Option<&WhitelistedStructureGenTags>,
    dim_blist_tags: Option<&BlacklistedStructureGenTags>,
) -> bool {
    if let Some(dim_blist_tags) = dim_blist_tags {
        if let Some(strgen_tags) = strgen_tags {
            if strgen_tags.intersects(&dim_blist_tags.0) {
                return false;
            }
        }
    }
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
pub fn offer_chunks_of_new_regions(
    mut cmd: Commands,
    settings: Single<&GlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<SgcsEntityWeightedMap>>,
    region_query: Query<(Entity, &RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    structured_gens: Query<(Option<&TagSet>, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    dimension_query: Query<(Option<&WhitelistedStructureGenTags>, Option<&BlacklistedStructureGenTags>),()>,
    mut writer: MessageWriter<OfferChunk>,
) {
    let mut offers = Vec::new();
    
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
                if offers.is_empty() {
                    error!(target: "structure_spawn", "No structures could be offered for region at {}, stopping attempts", region_pos);
                    cmd.entity(region_ent).try_insert(AllTilesPrepared);
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
            
            offers.push(OfferChunk {
                i: claim_i,
                region_ent,
                structured_gen_cfg_ent,
                start_gpos: rand_chunk_pos_within_region,
            });
            debug!(target: "structure_spawn", "Emitting OfferChunk for structure '{:?}' in region at {} for {}", structured_gen_cfg_ent, region_pos, rand_chunk_pos_within_region);
            
            claim_i += 1;
            reattempt_count = 0;
        }
        
    }
    writer.write_batch(offers);
}


#[allow(unused_parens)]
pub fn read_chunk_claims_for_region_and_emit_build_orders(
    mut cmd: Commands,
    mut claims: MessageReader<ClaimedChunks>,
    mut region_query: Query<(&RegionPos, &DimensionRef, &mut ClaimList, &mut CountsOfSgcs, &mut GridOfSgcs, &mut RegionPlannedTiles),()>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    time: Res<Time>,
    mut writer: MessageWriter<StructurePrepareTilesOrder>,
) {
    let mut regions_with_new_claims: Vec<Entity> = Vec::new();
    let mut regions_which_started_building = Vec::new();
    let mut build_orders = Vec::new();
    
    for claim in claims.read() {
        let Ok((_, _, mut claimlist, ..)) = region_query.get_mut(claim.region_ent)
        else {
            continue;
        };
        claimlist.claims[claim.i as usize] = Some(claim.clone());
        
        if !regions_with_new_claims.contains(&claim.region_ent) {
            regions_with_new_claims.push(claim.region_ent);
        }
    }
    let max_used_chunks_per_region = (REGION_SIZE_IN_CHUNKS.0.element_product() as f32 * 0.1) as u16;
    
    
    for region_ent in regions_with_new_claims {
        let Ok((&region_pos, &dimension_ref, mut claimlist, mut counts_of_sgcs, mut grid_of_sgc, mut planned)) = region_query.get_mut(region_ent)
        else {
            continue;
        };
        'nextregion: for i in claimlist.processed_up_to_i..MAX_CLAIMS {
            
            if grid_of_sgc.occupied_count() >= max_used_chunks_per_region as u32 {
                break 'nextregion;
            }
            let Some(claimed) = claimlist.claims.get_mut(i).unwrap() else {
                break 'nextregion;//ta bien, no hay que seguir hasta que aparezca la region structure en esta posicion
            };
            let mut claimed = take(claimed);
            claimlist.claims[i] = None;
            
            let Ok((structured_gen_cfg,)) = structured_gens.get(claimed.sgc_ent)
            else {
                claimlist.processed_up_to_i += 1; 
                error!(target: "structure_spawn", "StructuredGenConfig entity {:?} not found when processing claims for region at {:?}, skipping claim", 
                claimed.sgc_ent, region_pos);
                continue;
            };
            
            if counts_of_sgcs.0.get(&claimed.sgc_ent).copied().unwrap_or(0) >= structured_gen_cfg.max_per_region  {
                claimlist.processed_up_to_i += 1; 
                debug!(target: "structure_spawn", "Max structures of type '{}' already spawned in region {:?}, skipping claim", 
                structured_gen_cfg.structure_id, region_pos);
                continue;
            }
            let mut undo_claims = false;
            let mut claimed_up_to: u64 = 0;
            let mut failed_claims_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area() as usize, false);
            
            'nextpos: for (claim_i, &chunk_pos) in claimed.chunks_gpos.iter().enumerate(){
                match (grid_of_sgc.occupy(
                    chunk_pos,
                    region_pos,
                    claimed.sgc_ent,
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
                    grid_of_sgc.free(chunk_pos, region_pos);
                }
            } else {
                counts_of_sgcs.0.entry(claimed.sgc_ent)
                .and_modify(|count| *count += 1)
                .or_insert(1);
                
                for i in (0..claimed.chunks_gpos.len()).rev() {
                    if failed_claims_bitmask.get(i).unwrap_or(false) {
                        claimed.chunks_gpos.swap_remove(i);
                    }
                }
                
                planned.add_chunks_pending_build(&claimed.chunks_gpos, time.elapsed().as_secs_f64());
                regions_which_started_building.push((region_ent, BuildingStarted));
                
                let order = StructurePrepareTilesOrder {
                    i: claimed.i,
                    region_pos,
                    dimension_ref,
                    structured_gen_cfg_ent: claimed.sgc_ent,
                    chunks_gpos: claimed.chunks_gpos,
                };
                build_orders.push(order);
            }
            claimlist.processed_up_to_i += 1; 
        }
        
    }
    writer.write_batch(build_orders);
    cmd.try_insert_batch(regions_which_started_building);
}

#[allow(unused_parens)]
pub fn add_planed_tiles_to_region(mut cmd: Commands, 
    mut reader: MessageMutator<StructureBuildCompliance>,
    loaded_regions: Res<LoadedRegions>,
    mut region_query: Query<(&mut RegionPlannedTiles, Has<AllTilesPrepared>),()>,
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
        
        let Ok(finished) = planned_tiles.add_planned_tiles_and_remove_from_pending(build.chunk_pos, take(&mut build.tiles))
        else {
            error!(target: "structure_spawn", "Failed to add planned tiles for structure build compliance in region entity {:?} at chunk position {:?}, skipping", region_ent, build.chunk_pos);
            continue;
        };
        
        if finished {
            info!(target: "structure_spawn", "Region entity {:?} has finished planning all structure tiles, marking as RegionPlanningFinished", region_ent);
            cmd.entity(region_ent).try_insert(AllTilesPrepared);
        }
        
    }
}




#[allow(unused_parens, )]
pub fn clonespawn_tiles_on_chunk_spawn(mut cmd: Commands, 
    region_query: Query<(&ChunksActiveInRegion, &RegionPlannedTiles),(Or<(Changed<ChunksActiveInRegion>, Changed<RegionPlannedTiles>)>, With<BuildingStarted>, )>,
    chunk_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<ReadyForTerrgen>)>,
    mut collected: ResMut<MassCollectedTiles>,
) {
    let mut ready = Vec::new();
    let mut to_insert_delete_others = Vec::new();
    
    for (chunks_active_in_region, reg_planned) in region_query.iter() {
        
        chunk_query.iter_many(chunks_active_in_region.entities()).for_each(|(chunk_ent, &chunk_pos, &dimension_ref)| {
            if reg_planned.is_chunk_pending_build(chunk_pos) {
                return;
            }
            if let Some(tiles_to_spawn) = reg_planned.get(&chunk_pos) {
                debug!(target: "structure_spawn", "Spawning {} structure tiles in chunk at {:?}", tiles_to_spawn.len(), chunk_pos);
                for (tile_gpos, ezero_ref, delete_others) in tiles_to_spawn {
                    let tile_ent = collected.clonespawn_and_push_tile(&mut cmd, *ezero_ref, *tile_gpos, dimension_ref, OplistSize::default());
                    if let Some(delete_others) = delete_others {
                        to_insert_delete_others.push((tile_ent, (delete_others.clone())));
                    }
                }
            } else {
                debug!(target: "structure_spawn", "No structure tiles to spawn in chunk at {:?}", chunk_pos);
            }
            ready.push((chunk_ent, ReadyForTerrgen));
        });
    }
    cmd.try_insert_batch(ready);
    cmd.try_insert_batch(to_insert_delete_others);
}


pub fn despawn_empty_regions(mut cmd: Commands, 
    query: Query<(Entity, &DimensionRef, &RegionPos),(With<Region>, Without<ChunksActiveInRegion>,)>,
    mut loaded_regions: ResMut<LoadedRegions>,

){
    query.iter().for_each(|(region_ent, &dimension_ref, &region_pos)| {
        info!(target: "structure_spawn", "Despawning empty region entity {:?} at position {:?} in dimension {:?} which has no active chunks", 
        region_ent, region_pos, dimension_ref);
        loaded_regions.0.remove(&(dimension_ref, region_pos));
        cmd.entity(region_ent).despawn();
    });
}


pub fn failsafe_timeout_pending_chunks(
    mut cmd: Commands,
    time: Res<Time>,
    settings: Single<&GlobalGenSettings>,
    mut query: Query<(Entity, &RegionPos, &mut RegionPlannedTiles), (Without<AllTilesPrepared>)>,
) {
    let timeout = settings.structure_build_timeout_secs;
    let now = time.elapsed().as_secs_f64();
    
    for (region_ent, region_pos, mut planned) in query.iter_mut() {
        let mut timed_out = Vec::new();
        for (&chunk_pos, &since) in planned.pending_chunks_iter() {
            if now - since > timeout {
                timed_out.push(chunk_pos);
            }
        }
        if timed_out.is_empty() { continue; }
        
        for chunk_pos in timed_out {
            planned.mark_chunk_timed_out(chunk_pos);
            warn!(target: "structure_spawn", "Timed out waiting for StructureBuildCompliance for chunk {:?} in region {:?}, marking as empty and continuing", chunk_pos, region_pos);
        }
        
        if planned.pending_chunks_iter().next().is_none() {
            info!(target: "structure_spawn", "Region entity {:?} at {} has timed out remaining pending chunks, marking as RegionPlanningFinished", region_ent, region_pos);
            cmd.entity(region_ent).try_insert(AllTilesPrepared);
        }
    }
}