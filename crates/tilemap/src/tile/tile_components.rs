#[allow(unused_imports, )]
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
pub use bevy_ecs_tilemap::tiles::*;
use common::common_components::*;

use tilemap_shared::{define_weightedsampler, impl_weighted_sampler_serialization};
use rand::RngExt;

use ::tilemap_shared::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
//Don't add RequiredComponents here because it is forced onto clones and when removed it despawns the new entity
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 1;
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct U16TileIndex(pub u16);


pub type TileStrId = StrId;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct TileChildSprite;

#[derive(Component, Debug, Clone, Copy)]
pub struct TileChildSpriteOccluder;

define_weightedsampler!(TileStepSfx, Vec<String>, "TileStepSfx");
impl_weighted_sampler_serialization!(TileStepSfx, Vec<String>);

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TileStepSfxConfig {
    pub prevent_repeat: bool,
}
impl Default for TileStepSfxConfig {
    fn default() -> Self {
        Self {
            prevent_repeat: true,
        }
    }
}


//TODO HACER Q LAS TILES CAMBIEN AUTOMATICAMENTE DE TINTE SEGUN VALOR DE NOISES RELEVANTES COMO HUMEDAD O LO Q SEA
//SE PUEDE MODIFICAR EL SHADER PARA Q TOME OTRO VEC3 DE COLOR MÁS COMO PARÁMETRO Y SE LE MULTIPLIQUE AL PIXEL DE LA TEXTURA SAMPLEADO



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct BlocksProjectiles;

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct CorpsePose {
    pub rotation: Quat,
    pub offset: Vec3,
}
