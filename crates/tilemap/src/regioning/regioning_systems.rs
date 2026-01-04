
use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};

use game_common::game_common_components_samplers::EntityWeightedSampler;
use rand::{Rng, SeedableRng};
use ::tilemap_shared::*;

use crate::{chunking_components::Chunk, regioning::regioning_components::*};



#[allow(unused_parens)]
pub fn plan_structures_for_new_region(mut cmd: Commands, 
    settings: Single<&AcGlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<StructuredGenConfigWeightedMap>>,
    mut region_query: Query<(&RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    structured_gens: Query<(Option<&PoissonDisk>),(With<StructuredGenConfig>, )>,
) {
    for (region_pos, child_of) in region_query.iter_mut() {
        let rng = region_pos.hash_value(&settings, 0);
        let max_occupied_chunks = 35;

        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);

        let mut struct_gen_counts: EntityHashMap<u32> = EntityHashMap::default();

        let mut occupied_chunks_map: HashMap<ChunkPos, Entity> = HashMap::new();
        while occupied_chunks_map.len() < max_occupied_chunks {

            let Some(structured_gen_ent) = weight_map.sample_with_rng(&mut rng)
            else { 
                error!(target: "structure_spawn", "No StructuredGenConfig available to spawn structure in region at position ({}, {})", region_pos.0.x, region_pos.0.y);
                break; 
            };

            let rand_chunk_pos_within_region = IVec2 {
                x: rng.random_range(0..REGION_SIZE_IN_CHUNKS.x()),
                y: rng.random_range(0..REGION_SIZE_IN_CHUNKS.y()),
            }; 

            let Ok((poisson_disk)) = structured_gens.get(structured_gen_ent)
            else {
                continue;
            };
            if let Some(poisson_disk) = poisson_disk {
                let global_chunk_pos = ChunkPos::from(region_pos.0 * REGION_SIZE_IN_CHUNKS.0 + rand_chunk_pos_within_region);

                if ! poisson_disk.is_allowed_position(&settings, global_chunk_pos, false, OplistSize::default()) {
                    continue;
                }
            }


        }

    }
}

