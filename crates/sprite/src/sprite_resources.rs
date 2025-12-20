use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;


use common::common_types::HashIdToEntityMap;
use::serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Reflect, )]
#[reflect(Resource, Default)]
pub struct SpriteCfgEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SpriteSerisHandles {
    #[asset(path = "ron/sprite/config", collection(typed))]
    pub handles: Vec<Handle<SpriteConfigSeri>>,
}




#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct SpriteConfigSeri {
    pub id: String,
    pub name: String,
    pub mapped_anims: HashMap<(String, String, String, String), String>, 
    pub parent_cat: Option<String>, //adds ChildOf referencing other brother entity sprite possessing this category
    pub categories: Option<HashSet<String>>,
    pub shares_category: Option<Vec<bool>>,//asignar un componente
    pub children_sprites: Option<Vec<String>>,// these will get spawned as children of the entity that has this sprite data
    pub directionable: Option<bool>,
    pub movement_based: Option<bool>,
    pub grounding_based: Option<bool>,
    
    //use fly animation when standing still
    pub visibility: Option<u8>, //0: inherited, 1: visible, 2: invisible
    pub offset4children: Option<HashMap<String, (f32, f32, String)>>,//k:category, v:(offset, direction(s)) in which it is applied
    pub exclude_from_sys: Option<bool>,

    pub offset: Option<(f32, f32)>,
    pub scale: Option<(f32, f32)>,
    pub scale_up_down: Option<(f32, f32)>,
    pub scale_sideways: Option<(f32, f32)>,
    pub flip_horiz_if_dir: Option<u8>, //Left, Right, Any
    pub offset_up_down: Option<(f32, f32)>, 
    pub offset_down: Option<(f32, f32)>,
    pub offset_up: Option<(f32, f32)>,
    pub offset_sideways: Option<(f32, f32)>,


}
// PARA LAS BODY PARTS INTANGIBLES LASTIMABLES/CON HP, HACER Q EN LA DEFINICIÓN DE ESTOS SEAN ASOCIABLES A SPRITES CONCRETOS MEDIANTE SU ID O CATEGORY (AL DESTRUIR LA BODY PART SE INVISIBILIZA (NO BORRAR POR SI SE CURA DESP)). NO ASOCIAR BODY PARTS A SPRITE MEDIANTE EL PROPIO SPRITE PORQ AFECTA EL REUSO DE ESTE (P EJ EL CUERPO DE UN HUMANO PUEDE SER USADO EN OTRAS ESPECIES Q LE ASIGNAN OTRA HP U ÓRGANOS)
