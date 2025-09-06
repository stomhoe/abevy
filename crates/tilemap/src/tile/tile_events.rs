
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy::ecs::entity::MapEntities;

use dimension_shared::DimensionRef;
use serde::{Deserialize, Serialize};
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, OplistSize};
use std::hash::{Hasher, Hash};
use std::collections::hash_map::DefaultHasher;

use crate::tile::tile_components::*;

use {common::common_components::*, };

#[derive(Debug, Event)]
pub struct SavedTileHadChunkDespawn (pub Entity);


// #[derive(Debug, Deserialize, Event, Serialize, MapEntities, Hash, PartialEq, Eq, Clone)]
// pub struct SpawnSyncTile  { 
//     #[entities] pub orig_ref: Entity, pub oplist_size: OplistSize, pub dim: DimensionRef, pub global_pos: GlobalTilePos, pub serv_tile_ent: Entity 
// }
// impl SpawnSyncTile {
//     pub fn new(orig_ref: Entity, oplist_size: OplistSize, dim: DimensionRef, global_pos: GlobalTilePos, serv_tile_ent: Entity)
//     -> Self {Self { orig_ref, oplist_size, dim, global_pos, serv_tile_ent }}
// }