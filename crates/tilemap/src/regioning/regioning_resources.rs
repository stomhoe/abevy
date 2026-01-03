#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_inspector_egui::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use common::common_types::HashIdToEntityMap;
use dimension_shared::DimensionRef;
use ::tilemap_shared::*;
use std::mem::take;
use crate::terrain_gen::terrgen_messages::OpFilterSerialization;
use serde::Deserialize;

#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct StructuredGenConfigEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct StructureSerisHandles {
    #[asset(path = "ron/tilemap/region/structures", collection(typed))]
    pub handles: Vec<Handle<StructuredGenConfig>>,
}
#[derive(Deserialize, Asset, Reflect, )]
pub struct StructuredGenConfig {
    pub id: String,
    /// village, cave, dungeon, fort, etc
    pub structure_id: String, 
    /// extra arguments given to structure generation
    pub args: Vec<String>,
    /// weight in weighted map of structured gens for region. (more weight= likely for this structure to be generated first within the map of valid generations for that region)
    pub weight: f32,
    
    //expected terrain conditions for spawning
    pub whitelisted_filters: Option<Vec<OpFilterSerialization>>,
    pub whitelisted_biomes: Option<Vec<String>>,
    //para evitar adyacencia con chunk fronterizo de otra region, comparar hash del chunkpos con adyacentes y que se quede si es el más grande?
    pub min_dists_from_other_structures: Option<HashMap<String, u8>>,//in chunks
    
    //if empty, active in all dimensions (but that dimension must have a matcfhing tag)
    pub active_in_dimensions: Option<Vec<String>>,

    pub min_used_chunks: Option<u8>,//structure's own minimum chunk usage takes priority over this one
    pub max_used_chunks: Option<u8>,
    pub occupy_chunk_strat: Option<String>,//drunk walk or rectangle. StructuredGen's own strategy takes priority over this one
    pub min_per_region: Option<u8>,
    pub max_per_region: Option<u8>,

}


