
use bevy::{math::U8Vec2, platform::collections::HashMap};
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


#[derive(Component, Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum UsedShader{
    #[default]
    None,
    Grass
}