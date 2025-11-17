use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, prelude::*};
use bevy_ecs_tilemap::tiles::{TileBundle, TileColor, TilePos};
use common::common_components::HashId;
use dimension_shared::DimensionRef;
use ::tilemap_shared::*;
use std::mem::take;
use game_common::{game_common_components::*, game_common_components_samplers::EntiWeightedSampler};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::{terrain_gen::{terrgen_oplist_components::VariablesArray, }, tile::tile_components::*};
    

/*
.add_message::<TestEvent>(),
mut event_writer: MessageWriter<TestEvent>,
mut event_reader: MessageReader<TestEvent>,
*/




#[derive(Debug, Clone)]
pub enum SearchPattern {
    ///Probe direction
    Radial(Option<f32>),
    Spiral(u32, u32, IVec2, GlobalTilePos, bool),
}
impl SearchPattern {
    pub fn new_radial() -> Self { SearchPattern::Radial(None) }
    pub fn new_spiral(start_pos: GlobalTilePos) -> Self {
        SearchPattern::Spiral(1, 0, IVec2::new(0, 1), start_pos, false)
    }
}




#[derive(Message, Debug, Clone)]
pub struct PosSearch {
    pub dimension_hash_id: i32,
    pub studied_op_ent: Entity,
    pub step_size: u16,
    pub curr_iteration_batch_i: i16,//se puede cambiar a otra cosa para empezar alejado del centro
    pub max_batches: u16,
    pub iterations_per_batch: u16,
    pub search_pattern: SearchPattern,
}
impl PosSearch{
    pub fn portal_pos_search(dimension_hash_id: HashId, studied_op: Entity, search_start_pos: GlobalTilePos) -> PosSearch {
        PosSearch {
            dimension_hash_id: dimension_hash_id.into_i32(),
            step_size: 1,
            curr_iteration_batch_i: 0,
            max_batches: 100,
            iterations_per_batch: 1000,
            search_pattern: SearchPattern::new_spiral(search_start_pos),
            studied_op_ent: studied_op,
        }
    }
}



#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: Entity, pub dim_ref: DimensionRef, pub pos: GlobalTilePos, pub dimension_hash_id: i32,
    pub variables: VariablesArray, pub studied_op_ent: Entity//TODO: HACER LAS StudiedOp ENTITIES? (Y PONER StudiedOpRef en su lugar)
}


#[derive(Debug, Clone, Component)]
pub struct StudiedOp{
    pub root_oplist: Entity,
    pub checked_oplist: Entity,
    pub op_i: i8,
    pub lim_below: f32,
    pub lim_above: f32,
    pub search_start_pos: GlobalTilePos,
}
impl Hash for StudiedOp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.root_oplist.hash(state);
        self.checked_oplist.hash(state);
        self.op_i.hash(state);
        self.lim_below.to_bits().hash(state);
        self.lim_above.to_bits().hash(state);
    }
}
impl PartialEq for StudiedOp {
    fn eq(&self, other: &Self) -> bool {
        self.root_oplist == other.root_oplist &&
        self.checked_oplist == other.checked_oplist &&
        self.op_i == other.op_i &&
        self.lim_below.to_bits() == other.lim_below.to_bits() &&
        self.lim_above.to_bits() == other.lim_above.to_bits()
    }
}
impl Eq for StudiedOp {}

/*
mut event_writer: MessageWriter<ToClients<ClientSpawnTile>>,
*/


#[derive(Debug, Clone, Message, )]
pub struct SuitablePosFound { pub studied_op_ent: Entity, pub val: f32, pub found_pos: GlobalTilePos, }


#[derive(Debug, Clone, Message, )]
pub struct SearchFailed (pub Entity);

