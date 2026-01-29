use bevy::ecs::entity::MapEntities;
use bevy::{ecs::entity::EntityHashMap, };
use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_tag_components::{HashedTagsVec, TagSet};
use common::common_components::*;
use dimension_shared::DimensionRef;
use game_common::game_common_components::*;
use ::sprite_shared::*;

use std::hash::{DefaultHasher, Hash, Hasher};
use serde::{Serialize, Deserialize};
use ::tilemap_shared::*;

use crate::tile::tile_resources::{PortalSeri, TileImagePaths};
use crate::tile::tile_shader::tile_shader_components::*;
use crate::{terrain_gen::{terrgen_components::Terrgen, terrgen_messages::{OpFilter},}, };

#[derive(Bundle)]
pub struct ToDenyOnTileClone(
    MinDistancesMap, KeepDistanceFrom, TileHidsHandles, Replicated,
    TileShaderRef, AcZ, YSortOrigin, ChildOf, Description, TileColor, ImagePathHolder,
    DeleteOtherTiles, PortalRecipe, PortalSeri, 
    TagSet, HashedTagsVec,
    //children entities don't get cloned
    Children, EntityZero, 
    DisplayName, Prefix, TileStrId,
    
);//Disabled no porque se elimina posteriormente

#[derive(Bundle)]
struct ToDenyOnReleaseBuild( 
    Name, 
);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct KeepDisabled;


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
//NO PONER REQUIRE ENTITYPREFIX ACA PORQ SE LO FUERZA A LOS CLONES
#[require(AssetScoped, )]//no poner Replicated acá, sino el deny de Replicated quita el Tile
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 3;
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct TileChildSprite;

#[derive(Component, Debug, Copy, Clone, Hash, Reflect)]
pub struct LocalChunkRef(#[entities] pub Entity);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, AssetScoped, Prefix::trunc("Tiling"), Name, Transform, Visibility)]
pub struct TilesEguiHolder;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Prefix::trunc("Tile instances"), Name, Transform, Replicated, )]
pub struct TileInstancesHolder;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, AssetScoped, Prefix::trunc("PortalsZero"), Name, Transform, Visibility, )]
pub struct PortalsZeroEguiHolder;


pub type TileStrId = StrId20B;

//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct SeekingPortalOtherEnd;


#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, MapEntities)]
pub struct PortalRecipe { 
    #[entities]pub dest_dimension: Entity,
    #[entities]pub oe_portal_tile: Entity, 
    pub tags: TagSet,
    pub op_i: i16,
    pub min_val: f32,
    pub max_val: f32,
    pub one_way: bool,
}
impl PortalRecipe {
    pub fn to_op_filter(&self, start_pos: GlobalTilePos, oe_rootoplist: Entity) -> OpFilter {
        OpFilter {
            tags: HashedTagsVec::from(&self.tags),
            start_oplist: oe_rootoplist,
            op_i: self.op_i,
            min_val: self.min_val,
            max_val: self.max_val,
            search_start_pos: start_pos,
        }
    }
}
impl Default for PortalRecipe {
    fn default() -> Self {
        Self { dest_dimension: Entity::PLACEHOLDER, oe_portal_tile: Entity::PLACEHOLDER, tags: TagSet::default(), op_i: -1, min_val: 0.0, max_val: 0.0, one_way: false}
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect, MapEntities)]
pub struct PortalTo { #[entities]pub dest_portal: Entity, }
impl PortalTo {
    pub fn new(dest_portal: Entity) -> Self {
        Self { dest_portal }
    }
}

pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &GlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct FlipHorizontallyBasedOnHash;


#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, Debug)]
pub struct InitialPos(pub GlobalTilePos);


#[derive(Component, Debug, Clone, Default)]
pub struct TileHidsHandles { ids: Vec<HashId>, handles: Vec<Handle<Image>>,}

impl TileHidsHandles {
    pub fn from_paths(asset_server: &AssetServer, img_paths: TileImagePaths, ) -> Result<Self, BevyError> {
        
        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::with_capacity(img_paths.len());
        let mut handles = Vec::with_capacity(img_paths.len());
        for (key, path) in img_paths {
            let Ok(image_holder) = ImageHolder::new(asset_server, path.clone())
            else {
                error!("Failed to find image file for key {} at path: {}", key, path);
                continue;
            };
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }
        if ids.is_empty() {
            return Err(BevyError::from("No valid entries"));
        }
        
        Ok(Self { ids, handles, })
    }
    pub fn len(&self) -> usize { self.handles.len() }
    
    pub fn first_handle(&self) -> Handle<Image> {
        self.handles.first().cloned().unwrap_or_else(|| Handle::default())
    }
    
    // NO HACER take() porque lo necesitan multiples isntancias de tiles
    pub fn handles(&self) -> &Vec<Handle<Image>> { &self.handles }
    
    pub fn iter(&self) -> impl Iterator<Item = (HashId, &Handle<Image>)> {
        self.ids.iter().cloned().zip(self.handles.iter())
    }
}



#[derive(Component, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Reflect, Default)]
pub struct MinDistancesMap(pub EntityHashMap<u64>);

impl MinDistancesMap {
    #[allow(unused_parens, )]
    pub fn check_min_distances(&self, 
        my_pos: (DimensionRef, GlobalTilePos), new: (EntityZeroRef, DimensionRef, GlobalTilePos)
    ) -> bool {
        self.0.get(&new.0.0).map_or(true, |&min_dist| {
            my_pos.0 != new.1 || my_pos.1.distance_squared(&new.2) > (min_dist * min_dist) 
        })
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Terrgen, Prefix::trunc("TileSamplers"), )]
pub struct TileSamplerHolder;



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct DeleteOtherTiles { pub spared_z: HashSet<AcZ>, pub spared_tags: TagSet, pub extra_radius: u32, }
