use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use crate::common::common_utils::HashIdToEntityMap;

use::serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event)]
pub struct SpriteCfgEntityMap(pub HashIdToEntityMap);



#[derive(AssetCollection, Resource)]
pub struct SpriteSerisHandles {
    #[asset(path = "ron/sprite/config", collection(typed))]
    pub handles: Vec<Handle<SpriteConfigSeri>>,
}


#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct SpriteConfigSeri {
    pub id: String,
    pub name: String,
    pub img_path: String,
    pub parent_cat: String, //adds ChildOf referencing other brother entity sprite possessing this category
    pub categories: Vec<String>,
    pub children_sprites: Vec<String>,// these will get spawned as children of the entity that has this sprite data
    pub shares_category: Vec<bool>,//asignar un componente
    pub rows_cols: [u32; 2], 
    pub frame_size: [u32; 2],
    pub offset: [f32; 2],
    pub z: i32,
    pub directionable: bool,
    pub walk_anim: bool,
    pub swim_anim: bool,
    pub swim_anim_still: bool,
    pub fly_anim: bool,
    pub fly_anim_still: bool,
    pub flip_horiz: u8, //0: none, 1: any, 2: if looking left, 3: if looking right
    pub anim_prefix: String,
    pub visibility: u8, //0: inherited, 1: visible, 2: invisible
    pub offset4children: HashMap<String, [f32; 2]>,//category, offset
    pub offset_down: Option<[f32; 2]>,
    pub offset_up: Option<[f32; 2]>,
    pub offset_sideways: Option<[f32; 2]>,
    pub offset_up_down: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub scale_up_down: Option<[f32; 2]>,
    pub scale_sideways: Option<[f32; 2]>,
    pub color: Option<[u8; 4]>, 
    pub exclude_from_sys: Option<bool>,
}
// PARA LAS BODY PARTS INTANGIBLES LASTIMABLES/CON HP, HACER Q EN LA DEFINICIÓN DE ESTOS SEAN ASOCIABLES A SPRITES CONCRETOS MEDIANTE SU ID O CATEGORY (AL DESTRUIR LA BODY PART SE INVISIBILIZA (NO BORRAR POR SI SE CURA DESP)). NO ASOCIAR BODY PARTS A SPRITE MEDIANTE EL PROPIO SPRITE PORQ AFECTA EL REUSO DE ESTE (P EJ EL CUERPO DE UN HUMANO PUEDE SER USADO EN OTRAS ESPECIES Q LE ASIGNAN OTRA HP U ÓRGANOS)
