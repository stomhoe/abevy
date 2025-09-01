use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::*, common_states::*};
use game_common::game_common_components::{DimensionRef, MyZ};

use std::hash::{DefaultHasher, Hash, Hasher};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ::tilemap_shared::*;

use crate::{tile::{tile_materials::*}, };


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
#[require(MyZ, EntityPrefix::new("Tile"), AssetScoped,)]
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 3;
    // for non-sprite tiles
    pub const MAX_Z: MyZ = MyZ(1_000);
}
//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Reflect)]
pub struct ChunkOrTilemapChild;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct TileRef(#[entities] pub Entity);

#[derive(Bundle)]
pub struct BundleToDenyOnTileClone(
    MinDistancesMap, KeepDistanceFrom, Disabled/*DisplayName, StrId*/
);


pub fn tile_pos_hash_rand(initial_pos: InitialPos, settings: &AaGlobalGenSettings) -> f32 {
    let mut hasher = DefaultHasher::new();
    initial_pos.hash(&mut hasher);
    settings.seed.hash(&mut hasher);
    (hasher.finish() as f64 / u64::MAX as f64).abs() as f32
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct FlipAlongX;

#[derive(Component, Debug,  Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct TileShaderRef(pub Entity);
impl Default for TileShaderRef { fn default() -> Self { Self(Entity::PLACEHOLDER) } }

#[derive(Component, Debug, PartialEq, Eq, Clone, Reflect, )]
#[require(EntityPrefix::new("TileShader"), AssetScoped, TgenScoped)]
pub enum TileShader{
    TexRepeat(MonoRepeatTextureOverlayMat),
    TwoTexRepeat(TwoOverlaysExample),
    Voronoi(VoronoiTextureOverlayMat),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}


#[derive(Component, Clone, Deserialize, Serialize, Default, Hash, PartialEq, Eq, Copy, Reflect, )]
pub struct InitialPos(pub GlobalTilePos);




#[derive(Component, Debug, Clone, Default)]
pub struct TileHidsHandles { ids: Vec<HashId>, handles: Vec<Handle<Image>>,}

impl TileHidsHandles {
    pub fn from_paths(asset_server: &AssetServer, img_paths: HashMap<String, String>, ) -> Result<Self, BevyError> {

        if img_paths.is_empty() {
            return Err(BevyError::from("TileImgsMap cannot be created with an empty image paths map"));
        }
        let mut ids = Vec::with_capacity(img_paths.len());
        let mut handles = Vec::with_capacity(img_paths.len());
        for (key, path) in img_paths {
            let image_holder = ImageHolder::new(asset_server, &path)?;
            ids.push(HashId::from(key));
            handles.push(image_holder.0);
        }
        Ok(Self { ids, handles, })
    }

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
pub struct MinDistancesMap(pub EntityHashMap<u32>);

impl MinDistancesMap {
    #[allow(unused_parens, )]
    pub fn check_min_distances(&self, 
        my_pos: (DimensionRef, GlobalTilePos), new: (TileRef, DimensionRef, GlobalTilePos)
    ) -> bool {
        self.0.get(&new.0.0).map_or(true, |&min_dist| {
            my_pos.0 != new.1 || my_pos.1.distance_squared(&new.2) > min_dist * min_dist
        })
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct KeepDistanceFrom(#[entities] pub Vec<Entity>);