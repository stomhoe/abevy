use bevy::{math::f32, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use common::{common_components::EntityPrefix, common_types::HashIdToEntityMap};
use serde::{Deserialize, Serialize};

use crate::terrain_gen::terrgen_components::Terrgen;


#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Message, Reflect)]
#[reflect(Resource, Default)]
pub struct TileWeightedSamplersMap(pub HashIdToEntityMap);



#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileWeightedSamplerHandles {
    #[asset(path = "ron/tilemap/tiling/weighted_sampler", collection(typed))] 
    pub handles: Vec<Handle<TileWeightedSamplerSeri>>,
}
//SE PUEDE USAR TMB PARA SAMPLEAR COLORES PARA TILES
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
}




