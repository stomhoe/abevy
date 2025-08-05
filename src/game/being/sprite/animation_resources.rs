#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::being::sprite::{
    sprite_components::*,
//    sprite_constants::*,
//    sprite_events::*,
};

#[derive(AssetCollection, Resource)]
pub struct AnimSerisHandles {
    #[asset(path = "ron/sprite/animation", collection(typed))]
    pub handles: Vec<Handle<AnimationSeri>>,
}


// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo 

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct AnimationSeri {
    pub id: String,
    pub sheet_rows_cols: [u32; 2], //rows, cols
    pub target: u32,
    pub is_row: bool, //true: target is a row , false: target is a column
    pub partial: Option<[u32; 2]>, //start, end inclusive (0-indexed)
}