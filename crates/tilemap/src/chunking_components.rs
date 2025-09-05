#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::client::event;
use debug_unwraps::{DebugUnwrapErrExt, DebugUnwrapExt};
use game_common::{game_common_components_samplers::EntiWeightedSampler};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}, prelude::*};

use crate::{tile::tile_components::*};
use ::tilemap_shared::*;


use common::{common_components::*, };

#[derive(Component, Default)]
#[require(Visibility::Hidden, SessionScoped, LayersMap, TilesToSave, )]
pub struct Chunk;

/*
           .replicate::<FollowerOf>()
           .register_type::<FollowerOf>()
           .register_type::<TilesToSave>()
*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct SaveTile {
    pub chunk_pos: ChunkPos,//NO HACE FALTA PORQ EL CHUNKPOS SE PUEDE CALCULAR A PARTIR DE GLOBAL POS
}



#[derive(Component, Debug, Reflect, Default,)]
pub struct TilesToSave(pub EntityHashSet);
impl TilesToSave { pub fn entities(&self) -> &EntityHashSet { &self.0 } }





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct OperationsLaunched;

use crate::tilemap_systems::{MapKey, MapStruct};

#[derive(Component, Default, Clone, Reflect, )]
pub struct LayersMap(pub HashMap<MapKey, MapStruct>);


#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct ActivatingChunks(#[entities] pub EntityHashSet,);


