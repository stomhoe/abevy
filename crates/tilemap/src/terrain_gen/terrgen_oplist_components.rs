
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use fnl::{FastNoiseLite, NoiseSampleRange};

use serde::{Deserialize, Serialize};
use tilemap_shared::{AaGlobalGenSettings, ChunkPos, GlobalTilePos, HashablePosVec};
use std::hash::{Hasher, Hash};
use crate::tile::tile_components::*;

use {common::common_components::*, };
use strum_macros::{AsRefStr, Display, };
use std::ops::{Index, IndexMut};


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, )]
pub struct PoissonDisk { pub min_distance: u8, pub seed: u64, }
impl PoissonDisk {
    pub fn new(min_distance: u8, seed: u64) -> Result<Self, BevyError> {

        let max = 5;

        if min_distance > max {
            return Err(BevyError::from(format!("min_distance must be <= {}", max)));
        } else if min_distance == 0 {
            return Err(BevyError::from("min_distance must be > 0"));
        }
        Ok(Self { min_distance, seed })
    }
    pub fn sample<T: HashablePosVec>(&self, settings: &AaGlobalGenSettings, tile_pos: T, oplist_size: OplistSize) -> f32 {

        let val = tile_pos.normalized_hash_value(settings, self.seed);
        let added_sample_distance_x = oplist_size.x() as i32;
        let added_sample_distance_y = oplist_size.y() as i32;

        for dy in -(self.min_distance as i32)..=(self.min_distance as i32) {
            for dx in -(self.min_distance as i32)..=(self.min_distance as i32) {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // Only check within circle of radius min_distance
                if dx * dx + dy * dy > (self.min_distance as i32).pow(2) {
                    continue;
                }
                // Calculate the neighbor's position by offsetting the current tile position
                let neighbor_x = tile_pos.x() + dx + added_sample_distance_x;
                let neighbor_y = tile_pos.y() + dy + added_sample_distance_y;
                let neighbor_pos = GlobalTilePos(IVec2::new(neighbor_x, neighbor_y));
                let neighbor_val = neighbor_pos.normalized_hash_value(settings, self.seed);
                if neighbor_val > val {
                    return 0.0;
                }
            }
        }
        val 
    }
}
impl Default for PoissonDisk { fn default() -> Self { Self { min_distance: 1, seed: 0 } } }



#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct ChunkRef(pub Entity);

#[derive(Debug, Deserialize, Serialize, Clone, Reflect, MapEntities)]
pub struct Bifurcation{#[entities] pub oplist: Option<Entity>, #[entities] pub tiles: Vec<Entity>,}


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
#[require(EntityPrefix::new("OpList"), Replicated, SessionScoped, AssetScoped, TgenScoped)]
pub struct OperationList {

    pub trunk: Vec<(Operation, Vec<Operand>, u8)>,
    pub bifurcations: Vec<Bifurcation>,
}

// impl MapEntities for OperationList {
//     fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
//         for (_, operands, _) in self.trunk.iter_mut() {
//             for operand in operands.iter_mut() {
//                 if let Operand::NoiseEntity(ent, _, _, _) = operand {
//                     *ent = entity_mapper.get_mapped(*ent);
//                 }
//             }
//         }
//         for bifur in self.bifurcations.iter_mut() {
//             bifur.oplist = bifur.oplist.map(|oplist_entity| entity_mapper.get_mapped(oplist_entity));
//             bifur.tiles.iter_mut().for_each(|tile_entity| *tile_entity = entity_mapper.get_mapped(*tile_entity));
//         }

//     }
// }

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct OplistSize(UVec2);

impl OplistSize {
    pub fn new([x, y]: [u32; 2]) -> Result<Self, BevyError> {
        if x <= 0 || y <= 0 {
            return Err(BevyError::from("OplistSize dimensions must be > 0"));
        }
        let max = 4;
        if x > max || y > max {
            return Err(BevyError::from(format!("OplistSize dimensions must be <= {}", max)));
        }
        Ok(Self(UVec2::new(x, y)))
    }
    pub fn x(&self) -> u32 { self.0.x }
    pub fn y(&self) -> u32 { self.0.y }
    pub fn inner(&self) -> UVec2 { self.0 }
    pub fn size(&self) -> usize { (self.x() * self.y()) as usize }
}
impl Default for OplistSize { fn default() -> Self { Self(UVec2::ONE) } }


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect )]
pub struct VariablesArray(pub [f32; Self::SIZE as usize]);

impl VariablesArray {
    pub const SIZE: u8 = 16;
}

impl Index<u8> for VariablesArray {type Output = f32;
    fn index(&self, index: u8) -> &Self::Output {unsafe { self.0.get_unchecked(index as usize) }}
}

impl IndexMut<u8> for VariablesArray {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {unsafe { self.0.get_unchecked_mut(index as usize) }}
}

#[derive(Debug, Deserialize, Serialize, Clone, AsRefStr, Display, PartialEq, Reflect, )]
#[allow(non_camel_case_types)]
pub enum Operation {
    Add, Subtract, Multiply, MultiplyOpo, Divide, Min, Max, Average, Abs, MultiplyNormalized, MultiplyNormalizedAbs, i_Max, Linear
}


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, MapEntities)]
pub enum Operand {
    StackArray(u8),
    Value(f32),
    NoiseEntity(#[entities]Entity, NoiseSampleRange, bool, i32),
    HashPos(u64),
    PoissonDisk(PoissonDisk),
}
impl Operand {
    pub fn new_poisson_disk(min_distance: u8, seed: u64) -> Result<Self, BevyError> {
        PoissonDisk::new(min_distance, seed).map(Self::PoissonDisk)
    }
}
impl Default for Operand { fn default() -> Self { Self::Value(0.0) } }
impl From<f32> for Operand { fn from(v: f32) -> Self { Self::Value(v) } }


