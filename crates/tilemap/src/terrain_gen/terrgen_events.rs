use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, prelude::*};
use dimension_shared::DimensionRef;
use ::tilemap_shared::*;
use std::mem::take;
use game_common::{game_common_components::*, game_common_components_samplers::EntiWeightedSampler};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

use crate::{terrain_gen::terrgen_oplist_components::VariablesArray, tile::tile_components::*};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect, )]
pub enum OplistCollectedTiles {
    Array([Entity; 4]), Batch(Vec<Entity>),
}
impl OplistCollectedTiles {
    pub fn new(tile:Entity) -> Self {
        let mut arr = [Entity::PLACEHOLDER; 4];
        arr[0] = tile;
        Self::Array(arr)   
    }

    pub fn iter(&self) -> OplistCollectedTilesIter<'_> {
        match self {
            OplistCollectedTiles::Array(arr) => OplistCollectedTilesIter::Array { arr, idx: 0 },
            OplistCollectedTiles::Batch(vec) => OplistCollectedTilesIter::Batch { vec, idx: 0 },
        }
    }
    pub fn iter_mut(&mut self) -> OplistCollectedTilesIterMut<'_> {
        match self {
            OplistCollectedTiles::Array(arr) => OplistCollectedTilesIterMut::Array { arr: arr.as_mut_slice(), idx: 0 },
            OplistCollectedTiles::Batch(vec) => OplistCollectedTilesIterMut::Batch { vec: vec.as_mut_slice(), idx: 0 },
        }
    }
    pub fn len(&self) -> usize {
        match self {
            OplistCollectedTiles::Array(arr) => arr.iter().filter(|e| **e != Entity::PLACEHOLDER).count(),
            OplistCollectedTiles::Batch(vec) => vec.len(),
        }
    }
}
pub enum OplistCollectedTilesIterMut<'a> { Array { arr: &'a mut [Entity], idx: usize }, Batch { vec: &'a mut [Entity], idx: usize }, }
impl<'a> Iterator for OplistCollectedTilesIterMut<'a> {
    type Item = &'a mut Entity;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OplistCollectedTilesIterMut::Array { arr, idx } => {
                while *idx < arr.len() {
                    let i = *idx; *idx += 1;
                    let ptr = &mut arr[i] as *mut Entity;
                    unsafe { if *ptr != Entity::PLACEHOLDER { return Some(&mut *ptr); } }
                }
                None
            }
            OplistCollectedTilesIterMut::Batch { vec, idx } => {
                if *idx < vec.len() {
                    let i = *idx; *idx += 1;
                    let ptr = &mut vec[i] as *mut Entity;
                    unsafe { Some(&mut *ptr) }
                } else { None }
            }
        }
    }
}
pub enum OplistCollectedTilesIter<'a> { Array { arr: &'a [Entity; 4], idx: usize }, Batch { vec: &'a Vec<Entity>, idx: usize },}

impl<'a> Iterator for OplistCollectedTilesIter<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OplistCollectedTilesIter::Array { arr, idx } => {
                while *idx < arr.len() {
                    let ent = arr[*idx]; *idx += 1;
                    if ent != Entity::PLACEHOLDER { return Some(ent); }
                }
                None
            }
            OplistCollectedTilesIter::Batch { vec, idx } => {
                if *idx < vec.len() {
                    let ent = vec[*idx]; *idx += 1; Some(ent)
                } else { None }
            }
        }
    }
}

impl Default for OplistCollectedTiles { fn default() -> Self { Self::Array([Entity::PLACEHOLDER; 4]) } }

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
    pub studied_op: StudiedOp,
    pub step_size: u32,
    pub curr_iteration_batch_i: u32,//se puede cambiar a otra cosa para empezar alejado del centro
    pub max_batches: u32,
    pub iterations_per_batch: u32,
    pub search_pattern: SearchPattern,
}
impl PosSearch{
    pub fn portal_pos_search(studied_op: StudiedOp) -> PosSearch {
        PosSearch {
            step_size: 1,
            curr_iteration_batch_i: 0,
            max_batches: 100,
            iterations_per_batch: 1000,
            search_pattern: SearchPattern::new_spiral(studied_op.search_start_pos),
            studied_op,
        }
    }
}



#[derive(Event, Debug)]
pub struct PendingOp {pub oplist: Entity, pub chunk_ent: Entity, pub pos: GlobalTilePos, 
    pub variables: VariablesArray, pub studied_op: Option<StudiedOp>
}
impl Default for PendingOp {
    fn default() -> Self {
        Self { oplist: Entity::PLACEHOLDER, chunk_ent: Entity::PLACEHOLDER, pos: GlobalTilePos(IVec2::ZERO), variables: VariablesArray::default(), studied_op: None}
    }
}
#[derive(Debug, Clone, )]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Event, )]
pub struct InstantiatedTiles { pub chunk: Entity, pub tiles: OplistCollectedTiles, pub retransmission_count: u16 }
impl InstantiatedTiles {
    #[allow(unused_parens, )]
    fn insert_tile_recursive(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        global_pos: GlobalTilePos,
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
                self.insert_tile_recursive( cmd, tiling_ent, global_pos, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else {

            let tile_ent = cmd.entity(tiling_ent).clone_and_spawn_with(|builder|{
                builder.deny::<BundleToDenyOnTileClone>();
            })
            .try_insert((
                ChunkOrTilemapChild, 
                global_pos.to_tilepos(oplist_size), TileRef(tiling_ent), InitialPos(global_pos), global_pos, oplist_size))
            .id();

            // Insert into the array if there's space, otherwise switch to Batch
            match &mut self.tiles {
                OplistCollectedTiles::Array(arr) => {
                    if let Some(slot) = arr.iter_mut().find(|e| **e == Entity::PLACEHOLDER) {
                        *slot = tile_ent;
                    } else {
                        // No space left, convert to Batch
                        let mut batch = arr.iter().cloned().filter(|e| *e != Entity::PLACEHOLDER).collect::<Vec<_>>();
                        batch.push(tile_ent);
                        self.tiles = OplistCollectedTiles::Batch(batch);
                    }
                }
                OplistCollectedTiles::Batch(vec) => { vec.push(tile_ent); }
            }
        }
    }
    pub fn from_op(cmd: &mut Commands, precursor: &PendingOp,  tiling_ents: &Vec<Entity>, oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>, gen_settings: &AaGlobalGenSettings,
    ) -> Self {
        let mut instance = Self { chunk: precursor.chunk_ent, ..Default::default() };
        for tile in tiling_ents.iter().cloned() {
            instance.insert_tile_recursive(cmd, tile, precursor.pos, oplist_size, weight_maps, gen_settings, 0);
        }
        instance
    }
    pub fn from_tile(cmd: &mut Commands, chunk: Entity, tile_ref: TileRef, global_pos: GlobalTilePos, oplist_size: OplistSize) -> Self {
        if tile_ref.0 == Entity::PLACEHOLDER {
            warn!("Tried to instantiate tile with placeholder entity");
            return Self { chunk, ..Default::default() };
        }

        let tile_pos = global_pos.to_tilepos(oplist_size);

        let tile_ent = cmd.entity(tile_ref.0).clone_and_spawn_with(|builder|{
            builder.deny::<BundleToDenyOnTileClone>();
        })
        .try_insert((
            ChunkOrTilemapChild,
            tile_pos, tile_ref, InitialPos(global_pos), global_pos, oplist_size))

        .id();

        Self { chunk, tiles: OplistCollectedTiles::new(tile_ent), ..Default::default() }
    }


    pub fn take_tiles(&mut self) -> OplistCollectedTiles {take(&mut self.tiles)}

}
impl Default for InstantiatedTiles { fn default() -> Self { Self { chunk: Entity::PLACEHOLDER, tiles: OplistCollectedTiles::default(), retransmission_count: 0 } } }


#[derive(Debug, Clone, Hash, PartialEq, Eq, Reflect, Event, )]
pub struct ProcessedTiles { pub chunk: Entity, pub tiles: OplistCollectedTiles }

#[derive(Deserialize, Event, Serialize, )]
pub struct NewlyRegPos (pub Entity, pub OplistSize, pub (DimensionRef, GlobalTilePos));

impl MapEntities for NewlyRegPos {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.0 = entity_mapper.get_mapped(self.0);

        self.2.0 = DimensionRef(entity_mapper.get_mapped(self.2.0.0));
    }
}

#[derive(Debug, Clone, Event, )]
pub struct SuitablePosFound { pub studied_op: StudiedOp, pub val: f32, pub found_pos: GlobalTilePos, }


#[derive(Debug, Clone, Event, )]
pub struct SearchFailed (pub StudiedOp);

#[derive(Bundle)]
pub struct BundleToDenyOnTileClone(///EN EL FUTURO DEJAR EL DISABLED EN LOS CLONES, POR AHORA DENYEAR PARA DEBUGGEAR TILES HUERFANAS
    MinDistancesMap, KeepDistanceFrom, ChildOf, /*DisplayName, StrId*/
);
