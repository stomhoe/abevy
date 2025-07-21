
use bevy::{math::{U8Vec2, U8Vec4}, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;
use rand::Rng;
use rand_pcg::Pcg64;
use superstate::{SuperstateInfo};

use crate::game::{game_utils::WeightedMap, tilemap::tile_imgs::*};

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
    path: String,
    scale: u32, //scale to be divided by 1M
    mask_color: U8Vec4,
}

impl RepeatingTexture {
    pub fn new<S: Into<String>>(path: S, scale: u32, mask_color: U8Vec4) -> Self {
        Self { path: path.into(), scale, mask_color}
    }
    pub fn new_w_red_mask<S: Into<String>>(path: S, scale: u32) -> Self {
        Self { path: path.into(), scale, mask_color: U8Vec4::new(255, 0, 0, 255)}
    }
    pub fn path(&self) -> &str {
        &self.path
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