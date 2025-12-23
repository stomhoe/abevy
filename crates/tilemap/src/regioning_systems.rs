
use bevy::prelude::*;
use camera::camera_components::CameraTarget;
use common::common_components::{StrId, StrId20B};
use dimension_shared::DimensionRef
;
use rand::SeedableRng;
use tilemap_shared::{AcGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec, RegionPos};

use crate::{chunking_components::*, chunking_resources::*, regioning_components::ChunksInRegion,  regioning_resources::LoadedRegions, tile::tile_events::SavedTileHadChunkDespawn};


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
