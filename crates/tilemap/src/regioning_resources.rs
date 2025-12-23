#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{ecs::entity::MapEntities, prelude::*};
use bevy_inspector_egui::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;

use common::common_types::HashIdToEntityMap;
use dimension_shared::DimensionRef;
use ::tilemap_shared::*;
use std::mem::take;
use crate::{
    regioning_components::*, terrain_gen::terrgen_messages::OpFilterSerialization,
//    regioning_constants::*,
//    regioning_events::*,
};
use serde::Deserialize;

#[derive(Resource, Reflect, InspectorOptions, Default)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct LoadedRegions(pub HashMap<(DimensionRef, RegionPos), Entity>);

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct StructureEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct StructureSerisHandles {
    #[asset(path = "ron/tilemap/region/structures", collection(typed))]
    pub handles: Vec<Handle<StructureGenConfig>>,
}
#[derive(Deserialize, Asset, Reflect, )]
pub struct StructureGenConfig {
    pub id: String,
    pub gen_type: String, pub args: Vec<String>,
    pub whitelisted_filters: Option<Vec<OpFilterSerialization>>,
    pub whitelisted_biomes: Option<Vec<String>>,
    //para evitar adyacencia con chunk fronterizo de otra region, comparar hashpos y que se quede el más grande?
    pub min_dists_from_other_structures: Option<HashMap<String, u8>>,//in chunks

    pub priority: Option<i32>,//those with higher priority get placed first. poner en otra cosa envolvente
    pub used_chunks: Option<(u8, u8)>,//randomly generated, min-max
    pub occupy_chunk_strat: Option<String>,//drunk walk or rectangle
    pub generate_per_region: Option<(u8, u8)>,//min, max//no se si poner esto o decidir a mano

}


