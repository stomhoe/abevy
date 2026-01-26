#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{prelude::*, tasks::Task};
use bevy_inspector_egui::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use common::common_types::HashIdToEntityMap;
use dimension_shared::DimensionRef;
use game_common::game_common_components::EntityZeroRef;
use ::tilemap_shared::*;
use crate::tile::tile_components::DeleteOtherTiles;
use crate::terrain_gen::terrgen_messages::OpFilterSerialization;
use serde::Deserialize;

#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Debug, Clone)]
pub struct RegionTileSpawnRequest {
    pub chunk_ent: Entity,
    pub dimension_ref: DimensionRef,
    pub tile_gpos: GlobalTilePos,
    pub ezero_ref: EntityZeroRef,
    pub delete_others: Option<DeleteOtherTiles>,
}

#[derive(Debug, Default)]
pub struct RegionSpawnTaskResult {
    pub ready_chunks: Vec<Entity>,
    pub spawn_requests: Vec<RegionTileSpawnRequest>,
}

#[derive(Resource, Debug, Default)]
pub struct RegioningSpawnAsyncTasks {
    pub spawn_tasks: Vec<Task<RegionSpawnTaskResult>>,
}

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct SgcEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct StructureSerisHandles {
    #[asset(path = "ron/tilemap/region/structures", collection(typed))]
    pub handles: Vec<Handle<StructuredGenConfigSeri>>,
}
#[derive(Deserialize, Asset, Reflect, )]
pub struct StructuredGenConfigSeri {
    pub id: String,
    /// village, cave, dungeon, fort, etc
    pub structure_id: String, 

    pub tags: Option<Vec<String>>,

    /// extra arguments given to structure generation
    pub args: Option<Vec<String>>,
    /// weight in weighted map of structured gens for region. (more weight= likely for this structure to be generated first within the map of valid generations for that region)
    pub weight: f32,
    
    //expected terr conditions for spawning
    pub whitelisted_filters: Option<Vec<OpFilterSerialization>>,
    
    pub pdisk_mindist_and_tag: Option<Vec<(Option<u8>, String)>>,
    
    //para evitar adyacencia con chunk fronterizo de otra region, comparar hash del chunkpos con adyacentes y que se quede si es el más grande?
    pub min_dists_from_other_structures: Option<HashMap<String, u8>>,//in chunks
    
    //if empty, active in all dimensions (but that dimension must have a matcfhing tag)
    pub exclusive_for_dimensions: Option<Vec<String>>,//TODO user tags en vez de ids?

    pub min_used_chunks: Option<u8>,//structure's own minimum chunk usage takes priority over this one
    pub max_used_chunks: Option<u16>,

    pub max_per_region: Option<u32>,
    
}


