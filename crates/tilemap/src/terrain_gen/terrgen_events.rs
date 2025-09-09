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
.add_event::<TestEvent>(),
mut event_writer: EventWriter<TestEvent>,
mut event_reader: EventReader<TestEvent>,
*/

#[derive(Bundle, Debug, Clone, Reflect)]
pub struct TileHelperStruct{
    pub ezero: EntityZeroRef,
    pub global_pos: GlobalTilePos,
    pub dim_ref: DimensionRef,
    pub oplist_size: OplistSize,
    pub tile_bundle: TileBundle,
    pub initial_pos: InitialPos,
}

pub type CollectedTiles = Vec<(Entity, TileHelperStruct)>;

#[derive(Debug, Event, Clone, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct MassCollectedTiles  (pub CollectedTiles);
impl MassCollectedTiles {

    pub fn add_tiles_from_ezeros(
        &mut self,
        cmd: &mut Commands,
        ezeros: impl IntoIterator<Item = EntityZeroRef>,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
    ) -> Vec<Entity> {
        let ezeros_iter = ezeros.into_iter();
        let mut spawned = Vec::with_capacity(ezeros_iter.size_hint().0);
        spawned.extend(ezeros_iter.map(|ezero| {
            self.clonespawn_and_push_tile(cmd, ezero, global_pos, dim_ref, oplist_size)
        }));
        spawned
    }
    pub fn clonespawn_and_push_tile(
        &mut self,
        cmd: &mut Commands,
        ezero: EntityZeroRef,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
    ) -> Entity {
        let tile_instance = cmd.entity(ezero.0).clone_and_spawn_with(|builder|{
            builder.deny::<ToDenyOnTileClone>();
            //builder.deny::<BundleToDenyOnReleaseBuild>();
        }).id();
        let tile_bundle = TileBundle {
            position: global_pos.to_tilepos(oplist_size), ..Default::default()
        };
        let helper = TileHelperStruct {
            ezero,
            global_pos,
            dim_ref,
            oplist_size,
            tile_bundle,
            initial_pos: InitialPos(global_pos),
        };
        self.0.push((tile_instance, helper));
        tile_instance
    }

    fn collect_tiles_rec(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        global_pos: GlobalTilePos,
        dim_ref: DimensionRef,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>,
        gen_settings: &AaGlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample_with_pos(gen_settings, global_pos) {
                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }
                self.collect_tiles_rec(cmd, tiling_ent, global_pos, dim_ref, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else {
            self.clonespawn_and_push_tile(cmd, EntityZeroRef(tiling_ent), global_pos, dim_ref, oplist_size);
        }
    }
    pub fn collect_tiles(&mut self, 
        cmd: &mut Commands,
        bif_tiles: &Vec<Entity>, ev: &PendingOp, oplist_size: OplistSize, weight_maps: &Query<(&EntiWeightedSampler,), ()>, gen_settings: &AaGlobalGenSettings,
    )  {
        for tile in bif_tiles.iter().cloned() {
            self.collect_tiles_rec(cmd, tile, ev.pos, ev.dim_ref, oplist_size, weight_maps, gen_settings, 0);
        }
    }

}



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




#[derive(Event, Debug, Clone)]
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



#[derive(Event, Debug, Clone)]
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
mut event_writer: EventWriter<ToClients<ClientSpawnTile>>,
*/


#[derive(Debug, Clone, Event, )]
pub struct SuitablePosFound { pub studied_op_ent: Entity, pub val: f32, pub found_pos: GlobalTilePos, }


#[derive(Debug, Clone, Event, )]
pub struct SearchFailed (pub Entity);

