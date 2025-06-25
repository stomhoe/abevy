
use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use fastnoise_lite::FastNoiseLite;
use superstate::{SuperstateInfo};

use crate::game::tilemap::terrain_gen::terrain_gen_utils::{TileInstantiationData, UniqueTileDto};

#[derive(Component, Default, )]
pub struct FnlComp(pub FastNoiseLite);

#[derive(Component, Debug, Default, )]
pub struct Thresholds(pub Vec<(f32, Entity)>); //usar menor igual valor -> entidad. Entidad-> tiledist?



#[derive(Component, Default)]
#[require(SuperstateInfo<TileDeterminism>)]
pub struct TileDeterminism;

#[derive(Component, Debug, Default, )]
#[require(TileDeterminism)]
pub struct TileDistribution(
    pub HashMap<Entity, f32>, //Entity: una instancia de Fill
);


#[derive(Component, Debug, Default, )]
#[require(SuperstateInfo<Fill>, TileDeterminism)]
pub struct Fill;//no sé si ponerle id o q se referencie la entity instanciada 



#[derive(Component, Debug, Default, )]
pub struct Tree();


