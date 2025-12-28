
use bevy::prelude::*;

use rand::SeedableRng;
use tilemap_shared::{AcGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec, RegionPos};

use crate::regioning::regioning_components::*;



#[allow(unused_parens)]
pub fn plan_structures_for_new_region(mut cmd: Commands, 
    settings: Single<&AcGlobalGenSettings>,
    query: Query<(&RegionPos,),(Added<ChunksInRegion>, )>,
) {
    for (region_pos, ) in query.iter() {
        let rng = region_pos.hash_value(&settings, 0);

        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(rng);
    }
}
