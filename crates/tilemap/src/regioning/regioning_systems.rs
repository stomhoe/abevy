
use std::mem::take;

use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};

use common::common_components::HashId;
use dimension_shared::MultipleDimensionRefs;
use game_common::game_common_components_samplers::EntityWeightedSampler;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{chunking_components::Chunk, regioning::{regioning_components::*, regioning_messages::{ClaimedChunks, RequestChunkClaim, StructureBuildOrder}}};


#[allow(unused_parens)]
pub fn request_chunk_claims_for_new_region(mut cmd: Commands, 
    settings: Single<&AcGlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<StructuredGenConfigWeightedMap>>,
    mut region_query: Query<(Entity, &RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    structured_gens: Query<(&StructuredGenConfig, Option<&PoissonDisk>, Option<&MultipleDimensionRefs>),()>,
    mut writer: MessageWriter<RequestChunkClaim>,
) {
    
    let mut to_write = Vec::new();
    
    for (region_ent, region_pos, dim_ref) in region_query.iter_mut() {
        let rng = region_pos.hash_value(&settings, 0);
        
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
        
        
        let mut i = 0;
        'nextattempt: while i < MAX_CLAIMS as u64 {
            
            let Some(structured_gen_cfg_ent) = weight_map.sample_with_rng(&mut rng)
            else { 
                error!(target: "structure_spawn", "No StructuredGenConfig available to spawn structure in region at position ({}, {})", region_pos.0.x, region_pos.0.y);
                break; 
            };
            let Ok((config, poisson_disk, exclusive_for_dimensions)) = structured_gens.get(structured_gen_cfg_ent)
            else {
                continue 'nextattempt;
            };
            if let Some(exclusive_for_dimensions) = exclusive_for_dimensions {
                if ! exclusive_for_dimensions.0.contains(&dim_ref.parent()) {
                    continue 'nextattempt;
                }
            }
            
            
            let rand_chunk_pos_within_region = IVec2 {
                x: rng.random_range(0..REGION_SIZE_IN_CHUNKS.x()),
                y: rng.random_range(0..REGION_SIZE_IN_CHUNKS.y()),
            }; 
            let rand_chunk_pos_within_region = ChunkPos::from(rand_chunk_pos_within_region);
            
            if let Some(poisson_disk) = poisson_disk {
                
                if ! poisson_disk.is_allowed_position(&settings, rand_chunk_pos_within_region, false, OplistSize::default()) {
                    continue 'nextattempt;
                }
            }
            
            to_write.push(RequestChunkClaim {
                i,
                region_ent,
                structured_gen_cfg_ent,
                start: rand_chunk_pos_within_region,
                args: vec![],
                max_used_chunks: config.max_per_region as u16,
            });
            
            //occupied_chunks_map.insert(global_chunk_pos, structured_gen_ent);
            
            
            i += 1;
        }
        
    }
    writer.write_batch(to_write);
}


#[allow(unused_parens)]
pub fn get_chunk_claims_for_new_region(
    mut reader: MessageReader<ClaimedChunks>,
    mut region_query: Query<(&RegionPos, &mut RegionStructures, ),(Added<ChunksActiveInRegion>, )>,
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
            region_structures.vec[claim.i as usize] = Some(claim.clone());
            
            if !regions_with_new_claims.contains(&claim.region_ent) {
                regions_with_new_claims.push(claim.region_ent);
            }
        }
        let max_used_chunks = (REGION_SIZE_IN_CHUNKS.0.element_product() as f32 * 0.1) as u16;
        
        
        for region_ent in regions_with_new_claims {
            let Ok((region_pos, mut region_structures, )) = region_query.get_mut(region_ent)
            else {
                continue;
            };
            for i in region_structures.processed_up_to_i..MAX_CLAIMS {
                
                let Some(claimed) = region_structures.vec.get_unchecked_mut(i) else {
                    break;//ta bien
                };
                let claimed = take(claimed);

                let Ok((structured_gen_cfg,)) = structured_gens.get(claimed.structured_gen_cfg_ent)
                else {
                    region_structures.processed_up_to_i += 1; 
                    continue;
                };

                if  region_structures.struct_gen_counts.get(&claimed.structured_gen_cfg_ent).copied().unwrap_or(0) >= structured_gen_cfg.max_per_region as u32 {
                    region_structures.processed_up_to_i += 1; 
                    continue;
                }
                
                let mut undo_claims = false;
                let mut successfully_claimed_chunks = Vec::new();
                'nextpos: for chunk_pos in claimed.claimed_chunks.iter(){
                    if region_structures.occupied_chunks_grid[chunk_pos.0.y as usize][chunk_pos.0.x as usize].is_none() {
                        region_structures.occupied_chunks_grid[chunk_pos.0.y as usize][chunk_pos.0.x as usize] = Some(claimed.structured_gen_cfg_ent);
                        successfully_claimed_chunks.push(*chunk_pos);
                    } else if claimed.partition_tolerant {
                        region_structures.processed_up_to_i += 1; 
                        continue 'nextpos;
                    } else {//undo all previous claims
                        undo_claims = true;
                        break;
                    }
                }
                if undo_claims {
                    for chunk_pos in successfully_claimed_chunks.iter(){
                        region_structures.occupied_chunks_grid[chunk_pos.0.y as usize][chunk_pos.0.x as usize] = None;
                    }
                } else{
                    region_structures.struct_gen_counts.entry(claimed.structured_gen_cfg_ent)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                    let chunks_global_positions: Vec<ChunkPos> = successfully_claimed_chunks.iter().map(|local_chunk_pos| {
                        let global_chunk_x = region_pos.0.x * REGION_SIZE_IN_CHUNKS.x() + local_chunk_pos.0.x;
                        let global_chunk_y = region_pos.0.y * REGION_SIZE_IN_CHUNKS.y() + local_chunk_pos.0.y;
                        ChunkPos::new(global_chunk_x, global_chunk_y)
                    }).collect();

                    let order = StructureBuildOrder {
                        i: claimed.i,
                        structured_gen_cfg_ent: claimed.structured_gen_cfg_ent,
                        chunks: chunks_global_positions,
                    };
                    build_orders.push(order);
                }
                region_structures.processed_up_to_i += 1; 
            }
            
        }
        writer.write_batch(build_orders);
    }
}