#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::prelude::Replicated;
use bevy_replicon_renet::renet::RenetServer;
use debug_unwraps::{DebugUnwrapErrExt, DebugUnwrapExt};
use game_common::{game_common_components_samplers::EntiWeightedSampler};
use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled}, platform::collections::HashMap, prelude::*};

use crate::{tile::tile_components::*};
use ::tilemap_shared::*;


use common::{common_components::*, };

#[derive(Component, Default)]
#[require(Visibility::Hidden, SessionScoped, LayersMap)]
pub struct Chunk;


#[derive(Component, Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Reflect, )]
pub struct Tiles (pub Vec<Entity>);
impl Tiles {
    pub fn new_with_chunk_capacity() -> Self {
        let chunk_area = ChunkPos::CHUNK_SIZE.element_product();
        let cap = chunk_area;//TODO CALCULAR PROMEDIO DE TILES POR CHUNK
        Tiles (Vec::with_capacity(cap as usize))
    }

}




#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct OperationsLaunched;

use crate::tilemap_systems::{MapKey, MapStruct};

#[derive(Component, Default, Clone, Reflect, )]
pub struct LayersMap(pub HashMap<MapKey, MapStruct>);


#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct ActivatingChunks(#[entities] pub EntityHashSet,);


