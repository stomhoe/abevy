
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy::ecs::entity::MapEntities;

use ::tilemap_shared::*;



#[derive(Debug, Message)]
pub struct SavedTileHadChunkDespawn (pub Entity);

#[derive(Debug, Message)]
pub struct GlobalTilePosChanged {
    pub entity: Entity,
    pub old: Option<PrevPos>,
}



// #[derive(Debug, Deserialize, Message, Serialize, MapEntities, Hash, PartialEq, Eq, Clone)]
// pub struct SpawnSyncTile  {
//     #[entities] pub orig_ref: Entity, pub oplist_size: OplistSize, pub dim: DimensionRef, pub global_pos: GlobalTilePos, pub serv_tile_ent: Entity
// }
// impl SpawnSyncTile {
//     pub fn new(orig_ref: Entity, oplist_size: OplistSize, dim: DimensionRef, global_pos: GlobalTilePos, serv_tile_ent: Entity)
//     -> Self {Self { orig_ref, oplist_size, dim, global_pos, serv_tile_ent }}
// }
