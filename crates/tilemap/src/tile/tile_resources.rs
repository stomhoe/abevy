use bevy::{math::f32, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;


use bevy_inspector_egui::{egui, inspector_egui_impls::{InspectorPrimitive}, reflect_inspector::InspectorUi};


use common::common_types::HashIdToEntityMap;
use game_common::game_common_components::Category;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Default, Reflect )]
#[reflect(Resource, Default)]
pub struct TileShaderEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
pub struct TileEntitiesMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Event, Reflect)]
#[reflect(Resource, Default)]
//NO SE USA
pub struct TileInstancesEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct TileCategories (pub HashMap<Category, Vec<Entity>>);



#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct TileSerisHandles {
    #[asset(path = "ron/tilemap/tiling/tile", collection(typed))] 
    pub handles: Vec<Handle<TileSeri>>,
}

#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct TileSeri {
    pub id: String,
    pub cats: HashSet<String>,
    pub name: String,
    pub img_paths: HashMap<String, String>,
    pub shader: String,
    pub sprite: bool,
    pub offset: [f32; 2],
    pub z: i32,
    pub color: Option<[u8; 4]>,
    pub color_map: String,
    pub spawns: Vec<String>,
    pub spawns_children: Vec<String>,//SPRITECONFIGS SON VÁLIDOS
    pub ysort: Option<f32>,
    pub randflipx: bool,
    pub tmapchild: bool,
    pub min_distances: Option<HashMap<String, u32>>,
    /// destination dimension, destination portal-tile, destination searched terrain
    pub portal: Option<(String, String, String)>,
}


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderRepeatTexSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/rep1" , collection(typed))] 
    pub handles: Vec<Handle<ShaderRepeatTexSeri>>,
}


#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct ShaderRepeatTexSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub mask_color: [f32; 4],
}

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)] 
pub struct ShaderVoronoiSerisHandles {
    #[asset(path ="ron/tilemap/tiling/shader/voro" , collection(typed))] 
    pub handles: Vec<Handle<ShaderVoronoiSeri>>,
}


#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct ShaderVoronoiSeri {
    pub id: String,
    pub img_path: String,
    pub scale: f32,
    pub voronoi_scale: f32,
    pub voronoi_scale_random: f32,
    pub voronoi_rotation: f32,
    pub mask_color: [f32; 4],
}