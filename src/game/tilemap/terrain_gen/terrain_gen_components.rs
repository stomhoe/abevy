
use bevy::{math::{U8Vec2, U8Vec4}, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TileFlip, TileTextureIndex, TileVisible};
use fastnoise_lite::FastNoiseLite;

use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::game::{game_utils::WeightedMap, };

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
pub struct FnlComp { pub noise: FastNoiseLite, pub offset: IVec2, }


#[derive(Component, Default, )]
pub struct HashPosComp;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub enum NextAction {
    Continue,
    Break,
    OverwriteAcc(f32),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, )]
pub struct OnCompareConfig {
    pub tiles_on_success: ProducedTiles,
    pub tiles_on_failure: ProducedTiles,
    pub on_success: NextAction,
    pub on_failure: NextAction,
}


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq,)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Log,
    Min,
    Max,
    Pow,
    Assign,
    GreaterThan(OnCompareConfig),
    LessThan(OnCompareConfig),
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, )]
pub struct Operand(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, PartialEq, )]
pub struct Finished;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ProducedTiles(pub Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct FirstOperand(pub f32);



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct OperationList {
    pub trunk: Vec<(Entity, Operation)>,
    pub threshold: f32,
    pub bifurcation_over: Option<Entity>,
    pub bifurcation_under: Option<Entity>,
}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AwaitingChildOperationList;

//ES COMPONENT PORQ PUEDE HABER UNO PARA ARBUSTOS, OTRO PARA ARBOLES, ETC
//VA EN UNA ENTIDAD PROPIA ASI ES QUERYABLE. AGREGAR MARKER COMPONENTS PARA DISTINTOS TIPOS DE VEGETACIÓN
#[derive(Component, Debug, )]
pub struct TileWeightedMap(
    pub WeightedMap<Entity>, 
);