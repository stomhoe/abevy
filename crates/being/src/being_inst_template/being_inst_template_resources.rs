use being_shared::BeingInstTemplate;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use common::{common_components::StrId, define_entity_map_systems};

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct BitSerisHandles {
    #[asset(path = "ron/being/being_template", collection(typed))]
    pub handles: Vec<Handle<BitSerialization>>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct BitSerialization {
    pub id: String,
    pub fallback_faction: Option<String>,

    pub consecutive_name_weighted_distributions: Option<Vec<Vec<(String, f32)>>>,//to be appended to each other. 
    pub points: u32,
    pub race: String,
    pub scs_samplers: Option<Vec<String>>, // sprite weighted sampler ids
    pub scs_ids: Option<Vec<String>>, // sprite config ids to directly use
    pub sprites_scale_ranges: Option<HashMap<String, (f32, f32)>>,
    pub health_multiplier: Option<f32>,
}

define_entity_map_systems!(
    BitEntityMap,
    StrId,
    BeingInstTemplate
);
