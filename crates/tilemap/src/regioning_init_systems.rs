
use bevy::prelude::*;
use camera::camera_components::CameraTarget;
use common::common_components::{StrId, StrId20B};
use dimension_shared::DimensionRef
;
use rand::SeedableRng;
use tilemap_shared::{AcGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec, RegionPos};

use crate::{chunking_components::*, chunking_resources::*, regioning_components::ChunksInRegion, regioning_resources::*, tile::tile_events::SavedTileHadChunkDespawn};




#[allow(unused_parens)]
pub fn init_structures (
    mut cmd: Commands, 
    map: Option<Res<StructureEntityMap>>,
    mut seris_handles: ResMut<StructureSerisHandles>,
    mut assets: ResMut<Assets<StructureGenConfig>>,
) {
    if map.is_some(){ return;}
    let mut map = StructureEntityMap::default();
    for handle in std::mem::take(&mut seris_handles.handles) {
        let Some(seri) = assets.get(&handle) else {
            warn!(target: "structure_loading", "Failed to load StructureSeri from handle: {:?}", handle);
            continue;
        };
        info!(target: "structure_loading", "Loading StructureSeri from handle: {:?}", handle);
    }
    cmd.insert_resource(map);
}