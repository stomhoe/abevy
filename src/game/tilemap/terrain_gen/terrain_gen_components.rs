
use bevy::{math::{U8Vec2, U8Vec4}, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;

use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::game::{game_utils::WeightedMap, tilemap::tile_imgs::*};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct TileId(u64);

impl TileId {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        let mut hasher = DefaultHasher::new();
        id.as_ref().hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Component, Default, )]
pub struct FnlComp(pub FastNoiseLite);

#[derive(Component, Debug, Default, )]
pub struct Thresholds(pub Vec<(f32, Entity)>); //usar menor igual valor -> entidad. Entidad-> tiledist?


#[derive(Component, Debug, Default, )]
pub struct Tree();

//ES COMPONENT PORQ PUEDE HABER UNO PARA ARBUSTOS, OTRO PARA ARBOLES, ETC
//VA EN UNA ENTIDAD PROPIA ASI ES QUERYABLE. AGREGAR MARKER COMPONENTS PARA DISTINTOS TIPOS DE VEGETACIÓN
#[derive(Component, Debug, )]
pub struct TileWeightedMap(
    pub WeightedMap<Entity>, 
);

#[derive(Component, Debug, Default, Hash, PartialEq, Eq, Clone, )]
pub enum AppliedShader{
    #[default]
    None,
    MonoRepeating(RepeatingTexture),
    BiRepeating(RepeatingTexture, RepeatingTexture),
    //se pueden poner nuevos shaders con otros parámetros (por ej para configurar luminosidad o nose)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, )]
pub struct RepeatingTexture{
    img: Handle<Image>,
    scale: u32, //scale to be divided by 1M
    mask_color: U8Vec4,
}

impl RepeatingTexture {
    pub fn new<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32, mask_color: U8Vec4) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color }
    }
    pub fn new_w_red_mask<S: Into<String>>(asset_server: &AssetServer, path: S, scale: u32) -> Self {
        Self { img: asset_server.load(path.into()), scale, mask_color: U8Vec4::new(255, 0, 0, 255) }
    }
    pub fn cloned_handle(&self) -> Handle<Image> {
        self.img.clone()
    }

    #[allow(non_snake_case)]
    pub fn scale_div_1M(&self) -> f32 {//PARA GRASS DEBE SER MIL EN NEW
        self.scale as f32 / 1_000_000.0
    }
    pub fn mask_color(&self) -> Vec4 {
        self.mask_color.as_vec4()/255.0 
    }
}

//"texture/world/terrain/temperate_grass/grass.png"