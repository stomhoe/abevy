use std::hash::Hasher;

use bevy::prelude::*;
use strum_macros::{Display, EnumCount, EnumIter, VariantNames};



#[derive(Clone, Copy, EnumCount, Debug, Display, EnumIter, VariantNames, Hash)] 
pub enum BaseZLevels {Soil = 0, Water=40, Floor=80, Stain=160, Structure=200, Roof=240,}
impl Default for BaseZLevels {fn default() -> Self {Self::Soil}}


impl BaseZLevels{
  pub fn new(i: i32) -> Self {
    match i {
      0 => Self::Soil,
      1 => Self::Water,
      2 => Self::Floor,
      3 => Self::Stain,
      4 => Self::Structure,
      5 => Self::Roof,
      _ => panic!("TileZLevel::new: invalid i={}", i),
    }
  }
}


