
use bevy::{platform::collections::HashMap, prelude::*, sprite};
use bevy_asset_loader::prelude::*;

use crate::{common::{common_components::DisplayName, common_utils::HashIdToEntityMap}, game::{being::{race::race_components::*, sprite::sprite_resources::SpriteCfgEntityMap}, game_components::Description}};

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default )]
pub struct RaceEntityMap (pub HashIdToEntityMap);



#[derive(AssetCollection, Resource)]
pub struct RaceSerisHandles {
    #[asset(path = "ron/race", collection(typed))]
    pub handles: Vec<Handle<RaceSeri>>,
}


