
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_asset_loader::prelude::*;
use common::{common_types::HashIdToEntityMap, define_entity_map_systems};
use crate::race::Race;
//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct RaceSerisHandles {
    #[asset(path = "ron/being/race", collection(typed))]
    pub handles: Vec<Handle<RaceSerialization>>,
}



#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct RaceSerialization {
    pub id: String,
    pub name: String,
    pub name_generator: Option<String>,
    pub icon_path: Option<String>,
    //pub body_id: StrId,//NO SÉ SI ASOCIAR A CADA SPRITE UNA BODYPART (OPCIONALMENTE)
    pub description: Option<String>,
    pub demonym: Option<String>,
    pub singular: Option<String>,
    pub plural: Option<String>,
    pub sexes: HashMap<String, u32>,//id, weight
    pub sentient: Option<bool>,
    pub sprite_pool: Vec<String>,
    pub selectable_sprites: Option<Vec<String>>,

    pub hunger_rate: Option<f32>,
    pub can_walk_on: Option<HashSet<String>>, 
    pub walk_speeds: Option<HashMap<String, f32>>,

    pub whitelisted_tiles_for_spawning: Option<HashSet<String>>,
    pub blacklisted_tiles_for_spawning: Option<HashSet<String>>,

}

common::define_entity_map_systems!(
    RaceEntityMap,
    common::common_components::StrId,
    Race
);