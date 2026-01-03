
use bevy::prelude::*;

use game_common::game_common_components_samplers::EntityWeightedSampler;
use rand::SeedableRng;
use tilemap_shared::{AcGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec, RegionPos};

use crate::regioning::regioning_components::*;



#[allow(unused_parens)]
pub fn plan_structures_for_new_region(mut cmd: Commands, 
    settings: Single<&AcGlobalGenSettings>,
    weight_map: Single<&EntityWeightedSampler, With<StructuredGenConfigWeightedMap>>,
    region_query: Query<(&RegionPos, &ChildOf),(Added<ChunksActiveInRegion>, )>,
    //dimension_query: Query<(),()>,
) {
    for (region_pos, child_of) in region_query.iter() {
        let rng = region_pos.hash_value(&settings, 0);

        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
    }
}
