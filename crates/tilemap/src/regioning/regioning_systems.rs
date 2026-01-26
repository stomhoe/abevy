
use std::mem::take;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, futures_lite::future};
use common::{common_components::HashId, common_tag_components::TagSet};
use ::dimension_shared::*;
use game_common::{game_common_components_samplers::EntityWeightedSampler};
use rand::SeedableRng;
use ::tilemap_shared::*;

use crate::{
    chunking_components::ReadyForTerrgen,
    regioning::{
        regioning_components::*,
        regioning_messages::{ChunksClaim, OfferChunk, RecheckRegion, StructureBuildCompliance, StructurePrepareTilesOrder},
        regioning_resources::{LoadedRegions, RegioningSpawnAsyncTasks, RegionSpawnTaskResult, RegionTileSpawnRequest},
        regioning_sgc_components::*,
    },
    tile::tile_components::DeleteOtherTiles,
    tilemap_resources::MassCollectedTiles,
};

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
pub fn offer_chunks_of_new_regions_to_dungeoning_systems(
    mut cmd: Commands,
    settings: Single<&GlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<SgcsWeightedSampler>>,
    mut region_query: Query<(Entity, &RegionPos, &DimensionRef, &mut ClaimList),(Added<ChunksActiveInRegion>, )>,

    structured_gens: Query<(Option<&TagSet>, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    dimension_query: Query<(&HashId, Option<&WhitelistedStructureGenTags>, Option<&BlacklistedStructureGenTags>),()>,
    mut writer: MessageWriter<OfferChunk>,
) {
    let mut offers = Vec::new();
    
    for (region_ent, &region_pos, &dim_ref, mut claimlist) in region_query.iter_mut() {
        info!(target: "sgc_chunk_offer", "Offering chunks for new region at {:?}", region_pos);
        
        let Ok((&dim_hash, dim_wlist_tags, dim_blist_tags)) = dimension_query.get(dim_ref.0)
        else {
            error!(target: "sgc_chunk_offer", "Dimension entity {:?} not found when requesting chunk claims for region at  {:?}, skipping region", 
            dim_ref, region_pos);
            continue;
        };
        
        let rng = region_pos.hash_value(&settings, dim_hash, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
        
        let all_chunk_positions = region_pos.all_chunk_positions_shuffled(&mut rng);
        
        // solo por las dudas
        const MAX_REATTEMPTS: usize = 10000;
        
        'next_i: for (i, &chunk_pos) in all_chunk_positions.iter().enumerate() {
            if i >= MAX_CLAIMS as usize {
                break;
            }
            
            let mut weight_map = weight_map.clone();//ok to be here
            let mut reattempt_count = 0;
            'next_sgc_attempt: loop {
                if reattempt_count >= MAX_REATTEMPTS as u64 {
                    claimlist.skipped_is.insert(i);
                    continue 'next_i;
                }
                
                let Some(structured_gen_cfg_ent) = weight_map.sample_with_rng_and_remove(&mut rng)
                else { //ta bien
                    trace!(target: "sgc_chunk_offer", "No StructuredGenConfig left available to spawn structure in region at {}", region_pos);
                    claimlist.skipped_is.insert(i);
                    continue 'next_i;
                };
                let Ok((strgen_tags, poisson_disk, exclusive_for_dimensions)) = structured_gens.get(structured_gen_cfg_ent)
                else {
                    reattempt_count += 1;
                    continue 'next_sgc_attempt;
                };
                if let Some(exclusive_for_dimensions) = exclusive_for_dimensions {
                    if ! exclusive_for_dimensions.0.contains(&dim_ref.0) {
                        reattempt_count += 1;
                        trace!(target: "sgc_chunk_offer", "Dimension entity {:?} is not in exclusive dimension list for structure '{:?}', skipping", dim_ref, structured_gen_cfg_ent);
                        continue 'next_sgc_attempt;
                    }
                } else if !passes_dimension_tag_filters(strgen_tags, dim_wlist_tags, dim_blist_tags) {
                    reattempt_count += 1;
                    trace!(target: "sgc_chunk_offer", "Dimension entity {:?} fails tag filter checks for structure '{:?}', skipping", dim_ref, structured_gen_cfg_ent);
                    continue 'next_sgc_attempt;
                }
                
                if let Some(poisson_disk) = poisson_disk {
                    if ! poisson_disk.is_allowed_position(chunk_pos, &settings, dim_hash, false, OplistSize::default()) {
                        trace!(target: "sgc_chunk_offer", "Chunk position {:?} within {} rejected by PoissonDisk for structure '{:?}', reattempting", chunk_pos, region_pos, structured_gen_cfg_ent);
                        reattempt_count += 1;
                        continue 'next_sgc_attempt;
                    }
                }
                
                offers.push(OfferChunk {
                    i: i as u64,/*mal */
                    region_ent,
                    structured_gen_cfg_ent,
                    start_pos: chunk_pos,
                });
                debug!(target: "sgc_chunk_offer", "Emitting OfferChunk for structure '{:?}' in region at {} for {}", structured_gen_cfg_ent, region_pos, chunk_pos);
                continue 'next_i;
            }
        }
        
        if offers.is_empty() {
            error!(target: "sgc_chunk_offer", "No structures could be offered for region at {}, marking as BuildingStarted immediately", region_pos);
            cmd.entity(region_ent).try_insert((BuildingStarted, AllClaimsProcessed));
        } else {
            cmd.entity(region_ent).try_insert(PendingOfferTimeout { timeout_timer: Timer::from_seconds(0.2, TimerMode::Once) });
            writer.write_batch(take(&mut offers));
        }
        
    }
}


#[allow(unused_parens)]
pub fn advance_i_on_claimlist_timeout( 
    mut cmd: Commands,
    mut query: Query<(Entity, &mut ClaimList),(Without<AllClaimsProcessed>)>,
    time: Res<Time>,
    mut recheck_writer: MessageWriter<RecheckRegion>,
) {
    let mut recheck = Vec::new();

    query.iter_mut().for_each(|(region_ent, mut claimlist)| {
        claimlist.advance_timer.tick(time.delta());
        if claimlist.advance_timer.is_finished() {
            if claimlist.reached_end(){
                cmd.entity(region_ent).try_insert(AllClaimsProcessed);
                return;
            }
            claimlist.advance_processed_upto_i();
            recheck.push(RecheckRegion(region_ent));
        }
    });
    recheck_writer.write_batch(recheck);
}


#[allow(unused_parens)]
pub fn read_chunk_claims_for_region_and_emit_build_orders_to_dungeoning_systems(
    mut cmd: Commands,
    mut claims: MessageMutator<ChunksClaim>,
    mut region_query: Query<(&RegionPos, &DimensionRef, &mut ClaimList, &mut CountsOfSgcs, &mut GridOfSgcs, &mut RegionPlannedTiles),()>,
    structured_gens: Query<(&StructuredGenConfig,),()>,
    time: Res<Time>,
    mut recheck_reader: MessageReader<RecheckRegion>,
    mut writer: MessageWriter<StructurePrepareTilesOrder>,
){
    let mut regions_with_new_claims: Vec<Entity> = recheck_reader.read().map(|ent| ent.0).collect();
    let mut regions_which_started_building = Vec::new();
    let mut build_orders = Vec::new();
    for claim in claims.read() {
        if claim.region_ent == Entity::PLACEHOLDER {
            continue;
        }

        let Ok((_, _, mut claimlist, ..)) = region_query.get_mut(claim.region_ent)
        else {
            error!(target: "sgc_chunk_claim", "Region entity {:?} not found when receiving ClaimedChunks, skipping claim", claim.region_ent);
            continue;
        };
        
        if claim.i >= MAX_CLAIMS as u64 {
            error!(target: "sgc_chunk_claim", "Received claim with index {} >= MAX_CLAIMS {}, skipping", claim.i, MAX_CLAIMS);
            continue;
        }
        cmd.entity(claim.region_ent).try_remove::<PendingOfferTimeout>();
        
        let i = claim.i as usize;
        unsafe{
            *claimlist.claims.get_unchecked_mut(i) = Some(take(claim));
            let claim = claimlist.claims.get_unchecked(i).as_ref().unwrap_unchecked();
    
            if regions_with_new_claims.iter().all(|&e| e != claim.region_ent) {
                regions_with_new_claims.push(claim.region_ent);
            }
        }
    }
    let max_used_chunks_per_region = (REGION_SIZE_IN_CHUNKS.area_usize() as f32 * 0.07) as u64;
    
    for region_ent in regions_with_new_claims {
        let Ok((&region_pos, &dimension_ref, mut claimlist, mut counts_of_sgcs, mut grid_of_sgc, mut planned))
         = region_query.get_mut(region_ent)
        else {
            error!(target: "sgc_chunk_claim", "Region entity {:?} not found when processing chunk claims, skipping", region_ent);
            continue;
        };

        'nextregion: for i in claimlist.processed_up_to_i..MAX_CLAIMS {

            if claimlist.skipped_is.contains(&i) {
                claimlist.advance_processed_upto_i();
                continue 'nextregion;
            }//no tocar
            
            if grid_of_sgc.0.occupied_count() >= max_used_chunks_per_region {
                cmd.entity(region_ent).try_insert(AllClaimsProcessed);
                trace!(target: "sgc_chunk_claim", "Region at {:?} has reached max used chunks per region ({}), stopping further claim processing", 
                region_pos, max_used_chunks_per_region);
                break 'nextregion;
            }
            let Some(claim) = claimlist.claims.get_mut(i).unwrap() else {
                error!(target: "sgc_chunk_claim", "No claim found at index {} for region at {:?}, stopping further claim processing",
                i, region_pos);
                break 'nextregion;//ta bien, no hay que seguir hasta que aparezca la region structure en esta posicion
            };
            let mut claim = take(claim);
            claimlist.claims[i] = None;
            
            let Ok((structured_gen_cfg,)) = structured_gens.get(claim.sgc_ent)
            else {
                claimlist.advance_processed_upto_i();
                error!(target: "sgc_chunk_claim", "StructuredGenConfig entity {:?} not found when processing claims for region at {:?}, skipping claim", 
                claim.sgc_ent, region_pos);
                continue;
            };
            
            if counts_of_sgcs.0.get(&claim.sgc_ent).copied().unwrap_or(0) >= structured_gen_cfg.max_per_region  {
                claimlist.advance_processed_upto_i();
                debug!(target: "sgc_chunk_claim", "Max structures of type '{}' already spawned in region {:?}, skipping claim", 
                structured_gen_cfg.structure_id, region_pos);
                continue;
            }
            let mut undo_claims = false;
            let mut claimed_up_to: u64 = 0;
            let mut failed_claims_bitmask = BitVec::from_elem(REGION_SIZE_IN_CHUNKS.area_usize(), false);
            
            'nextpos: for (claim_i, &chunk_pos) in claim.chunks_gpos.iter().enumerate(){
                match (grid_of_sgc.0.occupy(
                    chunk_pos,
                    region_pos,
                    claim.sgc_ent,
                ), claim.partition_tolerant) {
                    (Ok(()), _) => {
                        debug!(target: "sgc_chunk_claim", "Successfully claimed chunk at {:?} in region {:?} for structure '{}'", chunk_pos, region_pos, structured_gen_cfg.structure_id);
                        claimed_up_to += 1;
                    }
                    (Err(ChunkOccupyError::OutOfRegionBounds(_)), _) => {
                        undo_claims = true;
                        error!(target: "sgc_chunk_claim", "Chunk at {:?} is outside region bounds, undoing all claims for this structure", 
                        chunk_pos);
                        break 'nextpos;
                    }
                    (Err(ChunkOccupyError::AlreadyOccupied(_)), true) => {
                        trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} already occupied, but claim is partition tolerant, continuing", 
                        chunk_pos, region_pos);
                        failed_claims_bitmask.set(claim_i, true);//OK
                        continue 'nextpos;
                    }
                    (Err(ChunkOccupyError::AlreadyOccupied(_)), false) => {
                        undo_claims = true;
                        trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} already occupied, undoing all claims for this structure", 
                        chunk_pos, region_pos);
                        break 'nextpos;
                    }
                }
            }
            if undo_claims {
                for i in 0..claimed_up_to {
                    let chunk_pos = claim.chunks_gpos[i as usize];
                    grid_of_sgc.0.free(chunk_pos, region_pos);
                }
            } else {
                counts_of_sgcs.0.entry(claim.sgc_ent)
                .and_modify(|count| *count += 1)
                .or_insert(1);
                
                for i in (0..claim.chunks_gpos.len()).rev() {
                    if failed_claims_bitmask.get(i).unwrap_or(false) {
                        claim.chunks_gpos.swap_remove(i);
                    }
                }
                planned.add_chunks_pending_build(&claim.chunks_gpos, time.elapsed().as_secs_f64());
                regions_which_started_building.push((region_ent, BuildingStarted));
                debug!(target: "sgc_chunk_claim", "Region at {:?} emitting {} build orders for structure '{}'", 
                    region_pos, claim.chunks_gpos.len(), structured_gen_cfg.structure_id);
                
                let order = StructurePrepareTilesOrder {
                    i: claim.i,
                    region_pos,
                    dimension_ref,
                    structured_gen_cfg_ent: claim.sgc_ent,
                    chunks_gpos: claim.chunks_gpos,
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
pub fn add_planned_tiles_to_region(mut cmd: Commands, 
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
    region_query: Query<(&ChunksActiveInRegion, &RegionPlannedTiles),(Or<(Changed<ChunksActiveInRegion>, Changed<RegionPlannedTiles>, )>, With<BuildingStarted>)>,
    chunk_query: Query<(Entity, &ChunkPos, &DimensionRef), (Without<ReadyForTerrgen>)>,
    mut collected: ResMut<MassCollectedTiles>,
    mut async_tasks: ResMut<RegioningSpawnAsyncTasks>,
) {
    let mut ready = Vec::new();
    let mut to_insert_delete_others = Vec::new();

    async_tasks.spawn_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            apply_spawn_task_result(&mut cmd, &mut collected, &mut ready, &mut to_insert_delete_others, result);
            false
        } else {
            true
        }
    });

    if !ready.is_empty() {
        cmd.try_insert_batch(ready);
    }
    if !to_insert_delete_others.is_empty() {
        cmd.try_insert_batch(to_insert_delete_others);
    }

    if !async_tasks.spawn_tasks.is_empty() {
        return;
    }

    let mut inputs = Vec::new();
    region_query.iter().for_each(|(chunks_active_in_region, reg_planned)| {
        chunk_query.iter_many(chunks_active_in_region.entities()).for_each(|(chunk_ent, &chunk_pos, &dimension_ref)| {
            if reg_planned.is_chunk_pending_build(chunk_pos) {
                return;
            }

            let tiles = reg_planned.get(&chunk_pos).cloned();
            inputs.push(RegionSpawnTaskInput {
                chunk_ent,
                dimension_ref,
                tiles,
            });
        });
    });

    if inputs.is_empty() {
        return;
    }

    let task_pool = AsyncComputeTaskPool::get();
    async_tasks.spawn_tasks.push(task_pool.spawn(async move {
        build_spawn_task_result(inputs)
    }));
}

#[derive(Clone)]
struct RegionSpawnTaskInput {
    chunk_ent: Entity,
    dimension_ref: DimensionRef,
    tiles: Option<TilesFromBuilder>,
}

fn build_spawn_task_result(inputs: Vec<RegionSpawnTaskInput>) -> RegionSpawnTaskResult {
    let mut result = RegionSpawnTaskResult::default();
    result.ready_chunks = Vec::with_capacity(inputs.len());

    for input in inputs {
        result.ready_chunks.push(input.chunk_ent);

        let Some(tiles) = input.tiles else {
            continue;
        };

        if !tiles.is_empty() {
            debug!(
                target: "structure_spawn",
                "Spawning {} structure tiles in chunk {:?}",
                tiles.len(),
                input.chunk_ent
            );
        }

        for (tile_gpos, ezero_ref, delete_others) in tiles {
            result.spawn_requests.push(RegionTileSpawnRequest {
                chunk_ent: input.chunk_ent,
                dimension_ref: input.dimension_ref,
                tile_gpos,
                ezero_ref,
                delete_others,
            });
        }
    }

    result
}

fn apply_spawn_task_result(
    cmd: &mut Commands,
    collected: &mut MassCollectedTiles,
    ready: &mut Vec<(Entity, ReadyForTerrgen)>,
    to_insert_delete_others: &mut Vec<(Entity, DeleteOtherTiles)>,
    result: RegionSpawnTaskResult,
) {
    ready.extend(result.ready_chunks.into_iter().map(|ent| (ent, ReadyForTerrgen)));

    for request in result.spawn_requests {
        let tile_ent = collected.clonespawn_and_push_tile(
            cmd,
            request.ezero_ref,
            request.tile_gpos,
            request.dimension_ref,
            OplistSize::default(),
        );
        if let Some(delete_others) = request.delete_others {
            to_insert_delete_others.push((tile_ent, delete_others));
        }
    }
}

#[allow(unused_parens, )]
pub fn despawn_empty_regions(mut cmd: Commands, 
    query: Query<(Entity, &DimensionRef, &RegionPos),(With<Region>, Without<ChunksActiveInRegion>, Without<EmptyRegionDespawnTimer>)>,
    mut despawn_query: Query<(Entity, &DimensionRef, &RegionPos, &mut EmptyRegionDespawnTimer), (Without<ChunksActiveInRegion>)>,
    saved_query: Query<(Entity, &DimensionRef, &RegionPos, &ChunksActiveInRegion), (Added<ChunksActiveInRegion>, With<EmptyRegionDespawnTimer>)>,
    mut loaded_regions: ResMut<LoadedRegions>,
    time: Res<Time>,
){
    // First pass: mark newly empty regions for despawn
    query.iter().for_each(|(region_ent, &dimension_ref, &region_pos)| {
        // Check if already marked, if not mark it
        if despawn_query.get(region_ent).is_err() {
            debug!(target: "region", "Region entity {:?} at position {:?} in dimension {:?} lost all active chunks, marking for despawn in 60s", 
                region_ent, region_pos, dimension_ref);
            cmd.entity(region_ent).try_insert(EmptyRegionDespawnTimer { despawn_timer: Timer::from_seconds(60.0, TimerMode::Once) });
        }
    });
    
    // Second pass: despawn regions that have timed out
    despawn_query.iter_mut().for_each(|(region_ent, &dimension_ref, &region_pos, mut despawn_timer)| {
        despawn_timer.despawn_timer.tick(time.delta());
        if despawn_timer.despawn_timer.is_finished() {
            debug!(target: "region", "Despawning empty region entity {:?} at position {:?} in dimension {:?} after 60s with no active chunks", 
                region_ent, region_pos, dimension_ref);
            loaded_regions.0.remove(&(dimension_ref, region_pos));
            cmd.entity(region_ent).try_despawn();
        }
    });
    saved_query.iter().for_each(|(region_ent, &dimension_ref, &region_pos, chunks_active_in_region, )| {
        if chunks_active_in_region.entities().is_empty() {
            return;
        }
        debug!(target: "region", "Region entity {:?} at position {:?} in dimension {:?} regained active chunks, cancelling despawn", 
            region_ent, region_pos, dimension_ref);
        cmd.entity(region_ent).try_remove::<EmptyRegionDespawnTimer>();
    });
}


pub fn failsafe_timeout_pending_chunks(
    mut cmd: Commands,
    time: Res<Time>,
    settings: Single<&GlobalGenSettings>,
    mut query: Query<(Entity, &RegionPos, &mut RegionPlannedTiles), Without<AllTilesPrepared>>,
) {
    let timeout = settings.structure_build_timeout_secs;
    let now = time.elapsed().as_secs_f64();
    query.iter_mut().for_each(|(region_ent, region_pos, mut planned)| {
        let mut timed_out = Vec::new();
        for (&chunk_pos, &since) in planned.pending_chunks_iter() {
            if now - since > timeout {
                timed_out.push(chunk_pos);
            }
        }
        if timed_out.is_empty() { return; }
        
        for chunk_pos in timed_out {
            planned.mark_chunk_timed_out(chunk_pos);
            warn!(target: "structure_spawn", "Timed out waiting for StructureBuildCompliance for chunk {:?} in region {:?}, marking as empty and continuing", chunk_pos, region_pos);
        }
        
        if planned.pending_chunks_iter().next().is_none() {
            info!(target: "structure_spawn", "Region entity {:?} at {} has timed out remaining pending chunks, marking as RegionPlanningFinished", region_ent, region_pos);
            cmd.entity(region_ent).try_insert(AllTilesPrepared);
        }
    });
}

#[allow(unused_parens)]
pub fn timeout_pending_offers(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &RegionPos, &mut PendingOfferTimeout), (Without<BuildingStarted>,)>,
) {
    query.iter_mut().for_each(|(region_ent, region_pos, mut pending_timeout)| {
        pending_timeout.timeout_timer.tick(time.delta());
        if pending_timeout.timeout_timer.is_finished() {
            warn!(target: "sgc_chunk_offer", "Offers for region at {} timed out after 0.2s with no claims, marking as BuildingStarted", region_pos);
            cmd.entity(region_ent).try_insert(BuildingStarted);
            cmd.entity(region_ent).remove::<PendingOfferTimeout>();
        }
    });
}

