
use std::{mem::take};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use common::{common_components::{HashId, Prefix, StrId}, common_tag_components::TagSet, log_targets::SGC_CHUNK_CLAIM};
use debug_unwraps::DebugUnwrapExt;
use game_common::game_common_timers::*;
use tilemap_shared::tilemap_shared_samplers::EntityWeightedSampler;
use rand::SeedableRng;
use ::tilemap_shared::*;

use crate::{regioning::{regioning_components::*, regioning_messages::{ChunksClaim, OfferChunk, RecheckRegion, StructureBuildCompliance, SgcPrepareTilesOrder}, regioning_resources::{LoadedRegions, Prioritized, PrioritizedPerRegion, StructureGenerationSettings}, regioning_sgc_components::*}, tilemap_resources::MassCollectedTiles};
use crate::regioning::natural::RiverDebugData;
use crate::terrain::terrgen_resources::TerrGenDisabledGposByChunk;
use crate::regioning::regioning_sgc_seris::load_structure_generation_settings_seri_defs;

use bitvec::prelude::*;

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
    settings: Query<&GlobalGenSettings>,
    weight_map: Query<&EntityWeightedSampler, With<SgcsWeightedSampler>>,
    mut region_query: Query<(Entity, &mut ClaimList, &DimensionRef, &RegionPos, ),(Added<RegionPos>)>,

    structured_gens: Query<(&StructuredGenConfig, Option<&TagSet>, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    strid_query: Query<&StrId>,
    dimension_query: Query<(&HashId, Option<&WhitelistedStructureGenTags>, Option<&BlacklistedStructureGenTags>),()>,
    dimension_map: Res<DimensionEntityMap>,
    mut writer: MessageWriter<OfferChunk>,
    mut loaded_regions: ResMut<LoadedRegions>,
    prioritized: Res<Prioritized>,
    mut prioritized_per_region: ResMut<PrioritizedPerRegion>,
) {

    let Ok(settings) = settings.single() else {
        error!("Failed to terrgen settings");
        return;
    };
    let Ok(weight_map) = weight_map.single() else {
        error!("Failed to get sgc weight map");
        return;
    };
    let mut offers = Vec::new();

    for (region_ent, mut claimlist, &dim_ref, &region_pos) in region_query.iter_mut() {
        let Ok(dim_ent) = dimension_map.0.get_cloned(dim_ref.0) else {
            continue;
        };
        trace!(target: "sgc_chunk_offer", "Offering chunks for new region at {:?}", region_pos);
        let region_key = (dim_ref, region_pos);
        loaded_regions.0.insert(region_key, region_ent);
        let prioritized_queue = prioritized_per_region
            .0
            .entry(region_key)
            .or_insert_with(|| prioritized.0.clone());


        let Ok((&dim_hash, dim_wlist_tags, dim_blist_tags)) = dimension_query.get(dim_ent)
        else {
            error!(target: "sgc_chunk_offer", "Dimension {:?} not found when requesting chunk claims for region at {:?}, skipping region",
            dim_ref, region_pos);
            continue;
        };

        let rng = region_pos.hash_value(&settings, dim_hash, 0);
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);

        let all_chunk_positions = region_pos.all_chunk_positions_shuffled(&mut rng);

        /// not necessary since weight_map is finite, but just in case
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

                let structured_gen_cfg_ent = if let Some(&ent) = prioritized_queue.first() {
                    prioritized_queue.remove(0);
                    weight_map.remove(&ent);
                    ent
                } else {
                    let Some(sampled_ent) = weight_map.sample_with_rng_and_remove(&mut rng)
                    else {
                        trace!(target: "sgc_chunk_offer", "No StructuredGenConfig left available to spawn structure in region at {}", region_pos);
                        claimlist.skipped_is.insert(i);
                        continue 'next_i;
                    };
                    sampled_ent
                };
                let Ok((structured_gen_cfg, strgen_tags, poisson_disk, exclusive_for_dimensions)) = structured_gens.get(structured_gen_cfg_ent)
                else {
                    reattempt_count += 1;
                    continue 'next_sgc_attempt;
                };
                let structured_gen_label = strid_query
                    .get(structured_gen_cfg_ent)
                    .ok()
                    .map(|strid| strid.to_string())
                    .unwrap_or_else(|| structured_gen_cfg.structure_id().to_string());
                if let Some(exclusive_for_dimensions) = exclusive_for_dimensions {
                    if !exclusive_for_dimensions.0.contains(&dim_hash) {
                        reattempt_count += 1;
                        trace!(target: "sgc_chunk_offer", "Dimension {:?} is not in exclusive dimension list for structure '{}', skipping", dim_ref, structured_gen_label);
                        continue 'next_sgc_attempt;
                    }
                } else if !passes_dimension_tag_filters(strgen_tags, dim_wlist_tags, dim_blist_tags) {
                    reattempt_count += 1;
                    trace!(target: "sgc_chunk_offer", "Dimension {:?} fails tag filter checks for structure '{}', skipping", dim_ref, structured_gen_label);
                    continue 'next_sgc_attempt;
                }

                if let Some(poisson_disk) = poisson_disk {
                    if ! poisson_disk.is_allowed_position(chunk_pos, &settings, dim_hash, false, OplistSize::default()) {
                        trace!(target: "sgc_chunk_offer", "Chunk position {:?} within {} rejected by PoissonDisk for structure '{}', reattempting", chunk_pos, region_pos, structured_gen_label);
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
                trace!(target: "sgc_chunk_offer", "Emitting OfferChunk for structure '{}' in region at {} for {}", structured_gen_label, region_pos, chunk_pos);
                continue 'next_i;
            }
        }

        if offers.is_empty() {
            warn!(target: "sgc_chunk_offer", "No structures could be offered for region at {}, marking as BuildingStarted immediately", region_pos);
            cmd.entity(region_ent).try_insert(RegionState::BuildingStarted);
        } else {
            cmd.entity(region_ent).try_insert(TimeoutTimer::secs(0.2));
            writer.write_batch(take(&mut offers));
        }
    }
}

#[allow(unused_parens)]
pub fn advance_i_on_claimlist_timeout(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut ClaimList, &RegionState), ()>,
    time: Res<Time>,
    mut recheck_writer: MessageWriter<RecheckRegion>,
) {
    let mut recheck = Vec::new();

    query.iter_mut().for_each(|(region_ent, mut claimlist, state)| {
        if *state != RegionState::OfferingChunks {
            return;
        }
        claimlist.advance_timer.tick(time.delta());
        if claimlist.advance_timer.is_finished() {
            if claimlist.reached_end(){
                cmd.entity(region_ent).try_insert(RegionState::ClaimsProcessed);
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
    structured_gens: Query<(&StructuredGenConfig, Option<&TagSet>),()>,
    strid_query: Query<&StrId>,
    structure_settings: Query<&StructureGenerationSettings>,
    mut recheck_reader: MessageReader<RecheckRegion>,
    mut writer: MessageWriter<SgcPrepareTilesOrder>,
){
    let Ok(structure_settings) = structure_settings.single() else {
        error!("Failed to get structure generation settings");
        return;
    };
    let mut regions_with_new_claims: Vec<Entity> = recheck_reader.read().map(|ent| ent.0).collect();
    let mut regions_which_started_building = Vec::new();
    let mut build_orders = Vec::new();
    let mut claims_received = 0_u32;
    let mut claims_stored = 0_u32;
    let mut claims_placeholder = 0_u32;
    let mut claims_invalid_index = 0_u32;
    for claim in claims.read() {
        claims_received = claims_received.saturating_add(1);
        if claim.region_ent == Entity::PLACEHOLDER {
            claims_placeholder = claims_placeholder.saturating_add(1);
            continue;
        }

        let Ok((_, _, mut claimlist, ..)) = region_query.get_mut(claim.region_ent)
        else {
            trace!(target: "sgc_chunk_claim", "Region entity {:?} is despawned when receiving ClaimedChunks, skipping claim", claim.region_ent);
            continue;
        };

        if claim.i >= MAX_CLAIMS as u64 {
            error!(target: "sgc_chunk_claim", "Received claim with index {} >= MAX_CLAIMS {}, skipping", claim.i, MAX_CLAIMS);
            claims_invalid_index = claims_invalid_index.saturating_add(1);
            continue;
        }
        cmd.entity(claim.region_ent).try_remove::<(MessageOnTimeout, TimerComp)>();

        let i = claim.i as usize;
        unsafe{
            *claimlist.claims.get_unchecked_mut(i) = Some(take(claim));
            let claim = claimlist.claims.get_unchecked(i).as_ref().unwrap_unchecked();
            claims_stored = claims_stored.saturating_add(1);
            trace!(
                target: SGC_CHUNK_CLAIM,
                "Stored claim i={} region={:?} sgc={} chunk_count={} partition_tolerant={}",
                claim.i,
                claim.region_ent,
                strid_query.get(claim.sgc_ent).ok().map(|strid| strid.to_string()).unwrap_or_else(|| format!("{:?}", claim.sgc_ent)),
                claim.chunks_pos.len(),
                claim.partition_tolerant
            );

            if regions_with_new_claims.iter().all(|&e| e != claim.region_ent) {
                regions_with_new_claims.push(claim.region_ent);
            }
        }
    }
    let max_used_chunks_per_region = (REGION_SIZE_IN_CHUNKS.area_usize() as f32 * 0.14) as u64;

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
                cmd.entity(region_ent).try_insert(RegionState::ClaimsProcessed);
                trace!(target: "sgc_chunk_claim", "Region at {:?} has reached max used chunks per region ({}), stopping further claim processing",
                region_pos, max_used_chunks_per_region);
                break 'nextregion;
            }
            unsafe{

                let Some(claim) = claimlist.claims.get_mut(i).debug_unwrap_unchecked() else {
                    error!(target: "sgc_chunk_claim", "No claim found at index {} for region at {:?}, stopping further claim processing",
                    i, region_pos);
                    break 'nextregion;//ta bien, no hay que seguir hasta que aparezca la region structure en esta posicion
                };
                let mut claim = take(claim);

                *claimlist.claims.get_mut(i).debug_unwrap_unchecked() = None;

                let Ok((structured_gen_cfg, claim_sgc_tags_opt)) = structured_gens.get(claim.sgc_ent)
                else {
                    claimlist.advance_processed_upto_i();
                    error!(target: "sgc_chunk_claim", "StructuredGenConfig for claim at region {:?} in dimension {:?} was not found, skipping claim", region_pos, dimension_ref);
                    continue;
                };

                if counts_of_sgcs.0.get(&claim.sgc_ent).copied().unwrap_or(0) >= structured_gen_cfg.max_per_region  {
                    claimlist.advance_processed_upto_i();
                    debug!(target: "sgc_chunk_claim", "Max structures of type '{}' already spawned in region {:?}, skipping claim", structured_gen_cfg.structure_id(), region_pos);
                    continue;
                }
                let mut undo_claims = false;
                let mut claimed_up_to: u64 = 0;
                let mut failed_claims_bitmask: BitArr!(for MAX_CLAIMS) = BitArray::ZERO;

                'nextpos: for (claim_i, &chunk_pos) in claim.chunks_pos.iter().enumerate(){
                    let Some(occupiers) = grid_of_sgc.0.get_values(chunk_pos, region_pos) else {
                        undo_claims = true;
                        error!(target: "sgc_chunk_claim", "Chunk at {:?} is outside region bounds, undoing all claims for this structure", chunk_pos);
                        break 'nextpos;
                    };
                    if !occupiers.is_empty() {
                        if claim_i == 0 {
                            undo_claims = true;
                            trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} already occupied at initial position, undoing all claims for this structure", chunk_pos, region_pos);
                            break 'nextpos;
                        }
                        let claim_sgc_tags = claim_sgc_tags_opt.cloned().unwrap_or_default();
                        let mut is_mutually_tolerated = true;
                        for &occupier_ent in occupiers {
                            let Ok((occupier_cfg, occupier_tags_opt)) = structured_gens.get(occupier_ent) else {
                                is_mutually_tolerated = false;
                                break;
                            };
                            let occupier_tags = occupier_tags_opt.cloned().unwrap_or_default();
                            if !occupier_cfg.tolerates_tags(&claim_sgc_tags)
                                || !structured_gen_cfg.tolerates_tags(&occupier_tags)
                            {
                                is_mutually_tolerated = false;
                                break;
                            }
                        }
                        if !is_mutually_tolerated {
                            if claim.partition_tolerant {
                                failed_claims_bitmask.set(claim_i, true);
                                continue 'nextpos;
                            }
                            undo_claims = true;
                            trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} is not mutually tolerated, undoing all claims for this structure", chunk_pos, region_pos);
                            break 'nextpos;
                        }
                    }
                    match (grid_of_sgc.0.occupy(
                        chunk_pos,
                        region_pos,
                        claim.sgc_ent,
                    ), claim.partition_tolerant) {
                        (Ok(()), _) => {
                            debug!(target: "sgc_chunk_claim", "Successfully claimed chunk at {:?} in region {:?} for structure '{}'", chunk_pos, region_pos, structured_gen_cfg.structure_id());
                            claimed_up_to += 1;
                        }
                        (Err(ChunkOccupyError::OutOfRegionBounds(_)), _) => {
                            undo_claims = true;
                            error!(target: "sgc_chunk_claim", "Chunk at {:?} is outside region bounds, undoing all claims for this structure",
                            chunk_pos);
                            break 'nextpos;
                        }
                        (Err(ChunkOccupyError::AlreadyOccupied), true) => {
                            trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} already occupied, but claim is partition tolerant, continuing",
                            chunk_pos, region_pos);
                            failed_claims_bitmask.set(claim_i, true);//OK
                            continue 'nextpos;
                        }
                        (Err(ChunkOccupyError::AlreadyOccupied), false) => {
                            undo_claims = true;
                            trace!(target: "sgc_chunk_claim", "Chunk at {:?} in region {:?} already occupied, undoing all claims for this structure",
                            chunk_pos, region_pos);
                            break 'nextpos;
                        }
                    }
                }
                if undo_claims {
                    for i in 0..claimed_up_to {
                        let chunk_pos = claim.chunks_pos[i as usize];
                        grid_of_sgc.0.free(chunk_pos, region_pos, claim.sgc_ent);
                    }
                } else {
                    counts_of_sgcs.0.entry(claim.sgc_ent)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);

                    for i in (0..claim.chunks_pos.len()).rev() {
                        if failed_claims_bitmask.as_bitslice().get(i).map_or(false, |bit| *bit) {
                            claim.chunks_pos.swap_remove(i);
                        }
                    }
                    planned.add_build_order_pending(claim.i, &claim.chunks_pos, structure_settings.structure_build_timeout_secs as f32);
                    regions_which_started_building.push((region_ent, RegionState::BuildingStarted));
                    debug!(target: "sgc_chunk_claim", "Region at {:?} emitting {} build orders for structure '{}'", region_pos, claim.chunks_pos.len(), structured_gen_cfg.structure_id());

                    let order = SgcPrepareTilesOrder {
                        i: claim.i,
                        region_pos,
                        dimension_ref,
                        structured_gen_cfg_ent: claim.sgc_ent,
                        chunks_pos: claim.chunks_pos,
                    };
                    build_orders.push(order);
                }
                claimlist.processed_up_to_i += 1;
            }
        }
    }
    let build_orders_count = build_orders.len();
    writer.write_batch(build_orders);
    if claims_received > 0 || build_orders_count > 0 {
        info!(
            target: SGC_CHUNK_CLAIM,
            "claims->orders summary: claims_received={}, claims_stored={}, claims_placeholder={}, claims_invalid_index={}, build_orders_emitted={}",
            claims_received,
            claims_stored,
            claims_placeholder,
            claims_invalid_index,
            build_orders_count
        );
    }
    cmd.try_insert_batch(regions_which_started_building);
}

#[allow(unused_parens, )]
pub fn add_planned_tiles_to_region(mut cmd: Commands,
    mut reader: MessageMutator<StructureBuildCompliance>,
    loaded_regions: Res<LoadedRegions>,
    mut region_query: Query<(&mut RegionPlannedTiles, &RegionState, ), ()>,
    macro_chunk_key_query: Query<(Entity, &DimensionRef, &MacrochunkPos, ), (With<MacroChunk>, )>,
    mut macro_chunk_biome_distributions: Query<(&mut BiomeDistribution, ), ()>,
) {
    let mut macrochunks_by_key = HashMap::new();
    for (macro_chunk_ent, &dimension_ref, &macro_chunk_pos, ) in macro_chunk_key_query.iter() {
        macrochunks_by_key.insert((dimension_ref, macro_chunk_pos), macro_chunk_ent);
    }
    for build in reader.read() {
        let order_i = build.i;
        let region_pos = build
            .chunks
            .first()
            .map(|(chunk_pos, _)| chunk_pos.to_region_pos())
            .or_else(|| build.terrgen_disabled_gpos_for_chunks.first_chunk_pos().map(|chunk_pos| chunk_pos.to_region_pos()))
            .or_else(|| build.forced_chunk_biomes.first().map(|forced| forced.chunk_pos.to_region_pos()));
        let chunks = take(&mut build.chunks);
        let terrgen_disabled_gpos_for_chunks = take(&mut build.terrgen_disabled_gpos_for_chunks);
        let forced_chunk_biomes = take(&mut build.forced_chunk_biomes);
        let Some(region_pos) = region_pos else {
            continue;
        };
        let Some(&region_ent) = loaded_regions.0.get(&(build.dimension_ref, region_pos))
        else {
            error!(target: "structure_spawn", "Region at position {:?} in dimension {:?} not found when processing structure build compliance, skipping",
            region_pos, build.dimension_ref);
            continue;
        };

        let Ok((mut planned_tiles, state)) = region_query.get_mut(region_ent)
        else {
            continue;
        };
        if *state == RegionState::AllTilesPrepared {
            error!(target: "structure_spawn", "Region at position {:?} in dimension {:?} already marked as RegionPlanningFinished when processing structure build compliance, skipping",
            region_pos, build.dimension_ref);
            continue;
        }

        let Ok(finished) = planned_tiles.add_planned_tiles_and_remove_from_pending(order_i, chunks, terrgen_disabled_gpos_for_chunks)
        else {
            error!(target: "structure_spawn", "Failed to add planned tiles for structure build compliance in region at {:?} in dimension {:?} for build order {}, skipping", region_pos, build.dimension_ref, order_i);
            continue;
        };
        for forced_chunk_biome in forced_chunk_biomes {
            let macro_chunk_pos = forced_chunk_biome.chunk_pos.to_macrochunk_pos();
            let Some(&macro_chunk_ent) = macrochunks_by_key.get(&(build.dimension_ref, macro_chunk_pos)) else {
                continue;
            };
            let Ok((mut biome_distribution, )) = macro_chunk_biome_distributions.get_mut(macro_chunk_ent) else {
                continue;
            };
            biome_distribution.add_tag_weights_in_chunk(forced_chunk_biome.chunk_pos, forced_chunk_biome.biome_tags);
        }

        if finished {
            debug!(target: "structure_spawn", "Region at {:?} in dimension {:?} has finished planning all structure tiles, marking as RegionPlanningFinished", region_pos, build.dimension_ref);
            cmd.entity(region_ent).try_insert(RegionState::AllTilesPrepared);
        }
    }
}
#[allow(unused_parens, )]
pub fn clonespawn_tiles_on_chunk_spawn(mut cmd: Commands,
    mut region_query: Query<(&ActiveChunksInRegion, &mut RegionPlannedTiles, &RegionState),(Or<(Changed<ActiveChunksInRegion>, Changed<RegionPlannedTiles>, Changed<RegionState>)>, )>,
    chunk_query: Query<(Entity, &ChunkPos, &DimensionRef, &TerrGenState), With<Chunk>>,
    mut collected: ResMut<MassCollectedTiles>,
    mut blocked_terrgen_gpos: ResMut<TerrGenDisabledGposByChunk>,
) {
    let mut ready = Vec::new();
    let mut to_insert_delete_others = Vec::new();
    region_query.iter_mut().for_each(|(chunks_active_in_region, mut reg_planned, state)| {
        if *state != RegionState::BuildingStarted
            && *state != RegionState::ClaimsProcessed
            && *state != RegionState::AllTilesPrepared
        {
            return;
        }
        chunk_query.iter_many(chunks_active_in_region.iter()).for_each(|(chunk_ent, &chunk_pos, &dimension_ref, chunk_terrgen_state)| {
            if *chunk_terrgen_state != TerrGenState::Pending {
                return;
            }
            if reg_planned.is_chunk_pending_build(chunk_pos) {
                return;
            }

            if let Some(tiles_to_spawn) = reg_planned.get(&chunk_pos) {
                debug!(target: "structure_spawn", "Spawning {} structure tiles in chunk at {:?}", tiles_to_spawn.len(), chunk_pos);
                for (tile_gpos, templ_ref, delete_others) in tiles_to_spawn {
                    let tile_ent = collected.clonespawn_and_push_tile(&mut cmd, *templ_ref, *tile_gpos, dimension_ref, );
                    if let Some(delete_others) = delete_others {
                        to_insert_delete_others.push((tile_ent, (delete_others.clone())));
                    }
                }
            } else {
                trace!(target: "structure_spawn", "No structure tiles to spawn in chunk at {:?}", chunk_pos);
            }
            let blocked_gpos = reg_planned.take_terrgen_disabled_gpos(chunk_pos);
            blocked_terrgen_gpos.insert_for_chunk(dimension_ref, chunk_pos, blocked_gpos);
            ready.push((chunk_ent, TerrGenState::Ready));
        });
    });
    cmd.try_insert_batch(ready);
    cmd.try_insert_batch(to_insert_delete_others);
}

#[allow(unused_parens, )]
pub fn despawn_empty_regions(mut cmd: Commands,
    to_add_despawn_timer_query: Query<(Entity, ),
    (With<Region>, Without<ActiveChunksInRegion>, Without<DespawnOnTimeout>)>,
    regions_which_regained_chunks_query: Query<(Entity, &DimensionRef, &RegionPos, &ActiveChunksInRegion), (Added<ActiveChunksInRegion>, With<DespawnOnTimeout>)>,
){
    for (region_ent, ) in to_add_despawn_timer_query.iter() {
        cmd.entity(region_ent).try_insert_if_new(DespawnTimer::secs(0.5));
    }
    for (region_ent, &dimension_ref, &region_pos, chunks_active_in_region, ) in regions_which_regained_chunks_query.iter() {
        if chunks_active_in_region.is_empty() {
            continue;
        }
        debug!(target: "region", "Region at position {:?} in dimension {:?} regained active chunks, cancelling despawn",
            region_pos, dimension_ref);
        cmd.entity(region_ent).try_remove::<TimeoutTimer>();
    }
}
#[allow(unused_parens, )]
pub fn on_region_despawn_remove_from_loaded_regions(
    trig: On<Despawn, Region>,
    region_query: Query<(&DimensionRef, &RegionPos),(common::AnyDisabling)>,
    mut loaded_regions: ResMut<LoadedRegions>,
    mut prioritized_per_region: ResMut<PrioritizedPerRegion>,
    mut river_debug: ResMut<RiverDebugData>,
)
{
    let Ok((&dimension_ref, &region_pos)) = region_query.get(trig.entity) else {
        return;
    };
    river_debug.remove_region(dimension_ref, region_pos);
    let Some(region_ent) = loaded_regions.0.get(&(dimension_ref, region_pos))
    else {
        return;
    };
    if *region_ent == trig.entity {
        loaded_regions.0.remove(&(dimension_ref, region_pos));
        prioritized_per_region.0.remove(&(dimension_ref, region_pos));
    }
}


pub fn failsafe_timeout_pending_chunks(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &RegionPos, &mut RegionPlannedTiles, &RegionState), ()>,
) {
    query.iter_mut().for_each(|(region_ent, region_pos, mut planned, state)| {
        if *state == RegionState::AllTilesPrepared {
            return;
        }
        let mut timed_out_orders: Vec<u64> = Vec::new();
        for (&order_i, order) in planned.pending_build_orders_iter_mut() {
            order.timer.tick(time.delta());
            if order.timer.is_finished() {
                timed_out_orders.push(order_i);
            }
        }
        if timed_out_orders.is_empty() { return; }

        for order_i in timed_out_orders {
            if let Some(order) = planned.take_pending_build_order(order_i) {
                for chunk_pos in order.chunks {
                    planned.mark_chunk_timed_out(chunk_pos);
                    error!(target: "structure_spawn", "Timed out waiting for StructureBuildCompliance for chunk {:?} in region {:?}, marking as empty and continuing", chunk_pos, region_pos);
                }
            }
        }

        if planned.pending_build_orders_iter().next().is_none() {
            error!(target: "structure_spawn", "Region at {} has timed out remaining pending build orders, marking as RegionPlanningFinished", region_pos);
            cmd.entity(region_ent).try_insert(RegionState::AllTilesPrepared);
        }
    });
}

#[allow(unused_parens)]
pub fn mark_as_building_started_timed_out(
    mut cmd: Commands,
    query: Query<(&RegionPos, &RegionState), (With<Region>, With<MessageOnTimeout>)>,
    mut reader: MessageReader<TimedOut>,
) {
    for TimedOut(region_ent) in reader.read() {
        let Ok((region_pos, state)) = query.get(*region_ent) else { continue; };
        if *state != RegionState::OfferingChunks {
            continue;
        }
        warn!(target: "sgc_chunk_offer", "Offers for region at {} timed out after 0.2s with no claims, marking as BuildingStarted", region_pos);
        cmd.entity(*region_ent).try_insert(RegionState::BuildingStarted);
        cmd.entity(*region_ent).try_remove::<TimeoutTimer>();
    }
}

#[allow(unused_parens)]
pub fn init_structure_generation_settings(
    mut cmd: Commands,
    mut settings: Query<&mut StructureGenerationSettings>,
) {
    let settings_from_defs = load_structure_generation_settings_seri_defs()
        .into_iter()
        .next()
        .map(|seri| seri.to_structure_generation_settings());
    if settings.is_empty() {
        let settings_to_spawn = settings_from_defs.clone().unwrap_or_default();
        cmd.spawn((settings_to_spawn, Prefix::trunc("STRUCTURE_GENERATION_SETTINGS")));
    } else if let Some(settings_from_defs) = settings_from_defs {
        for mut existing_settings in &mut settings {
            *existing_settings = settings_from_defs.clone();
        }
    }
    info!(target: common::log_targets::TERRGEN_INIT, "Loaded Structure Generation Settings");
}
