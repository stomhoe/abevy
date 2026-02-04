
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use common::common_types::HashIdToEntityMap;


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
    pub can_equip_tools: bool,
    pub sprite_pool: Vec<String>,
    pub selectable_sprites: Vec<String>,

}