
use bevy::ecs::entity::MapEntities;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_tag_components::AddSameHashedTags;
use fnl::{FastNoiseLite, NoiseSampleRange};

use serde::{Deserialize, Serialize};
use ::tilemap_shared::*;

use {common::common_components::*, };
use strum_macros::{AsRefStr, Display, };
use std::ops::{Index, IndexMut};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(AssetScoped, Prefix::trunc("EguiOplistHolder"), Replicated, )]
pub struct EguiOplistHolder;


#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct ChunkRef(pub Entity);

#[derive(Debug, Deserialize, Serialize, Clone, Reflect, MapEntities)]
pub struct Bifurcation{
    #[entities] pub oplist: Option<Entity>, 
    #[entities]pub tiles: Vec<Entity>,
}
#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
#[require(Prefix::trunc("OpList"), Replicated, AssetScoped, AddSameHashedTags)]
#[component(map_entities)]
pub struct OperationList {

    pub trunk: Vec<(Operation, Vec<Operand>, u8)>,
    pub bifurcations: Vec<Bifurcation>,
}

impl MapEntities for OperationList {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        for (_, operands, _) in self.trunk.iter_mut() {
            for operand in operands.iter_mut() {
                if let OperandElement::NoiseEntity(ref mut ent, _, _, _) = operand.element {
                    *ent = entity_mapper.get_mapped(*ent);
                }
            }
        }
        for bifur in self.bifurcations.iter_mut() {
            bifur.oplist = bifur.oplist.map(|oplist_entity| entity_mapper.get_mapped(oplist_entity));
            bifur.tiles.iter_mut().for_each(|tile_entity| *tile_entity = entity_mapper.get_mapped(*tile_entity));
        }
    }
}

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
    Add, Subtract, Multiply, MultiplyOpo, Divide, Min, Max, Average, Abs, MultiplyNormalized, MultiplyNormalizedAbs, i_Max, Linear, i_Norm, Clamp
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, MapEntities)]
pub struct Operand {
    pub complement: bool,
    pub element: OperandElement,
}


#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Reflect, MapEntities)]
pub enum OperandElement {
    StackArray(u8),
    Value(f32),
    NoiseEntity(#[entities]Entity, NoiseSampleRange, bool, i32),
    HashPos(u64),
    PoissonDisk(PoissonDisk),
}
impl OperandElement {
    pub fn new_poisson_disk(min_distance: u8, seed: u64) -> Result<Self, BevyError> {
        PoissonDisk::new(min_distance, seed).map(Self::PoissonDisk)
    }
}
impl Default for OperandElement { fn default() -> Self { Self::Value(0.0) } }
impl From<f32> for OperandElement { fn from(v: f32) -> Self { Self::Value(v) } }


