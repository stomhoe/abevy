use std::hash::Hasher;

use bevy::{math::{U16Vec2, U8Vec2}, prelude::*};
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};
use strum_macros::{Display, EnumCount, EnumIter, VariantNames};

use crate::game::tilemap::tile_imgs::TileImgNid;


pub const Z_DIVISOR: i32 = 10000;

pub const GRASS_Z_LEVEL: i32 = 10 * Z_DIVISOR;


//hacerlo parte de una entity 
#[derive(Component, Debug, )]
pub struct TileInstantiationData{
    pub layer_z: i32,//posteriormente se divide entre 10000
    pub image_nid: TileImgNid,
    pub flip: TileFlip,
    pub color: TileColor,
    pub needs_y_sort: bool,
    pub visible: TileVisible,
}//TODO separar
impl TileInstantiationData {
    pub fn new(layer_z: i32, image_nid: TileImgNid, flip: TileFlip, color: TileColor, needs_y_sort: bool, visible: TileVisible, ) -> Self {
        Self { layer_z, image_nid, flip, color, needs_y_sort, visible,  }
    }

    pub fn layer_z(&self) -> i32 {self.layer_z}

    pub fn visible(&self) -> TileVisible {self.visible}

}


impl Default for TileInstantiationData {
    fn default() -> Self {
        Self { 
            layer_z: 0,
            image_nid: TileImgNid::default(),
            flip: TileFlip::default(),
            color: TileColor::default(),
            needs_y_sort: false,
            visible: TileVisible::default(),
        }
    }
}
#[derive(Debug, )]
pub struct UniqueTileDto{
    tile_entity: Entity,
    pos_within_chunk: U8Vec2,
    tile_inst_data_entity: Entity,
}
impl UniqueTileDto {
    pub fn new(tile_entity: Entity, pos_within_chunk: U8Vec2,  tile_inst_data_entity: Entity) -> Self {
        Self { tile_entity, pos_within_chunk, tile_inst_data_entity }
    }
    pub fn tile_entity(&self) -> Entity {self.tile_entity}
    pub fn pos_within_chunk(&self) -> TilePos {TilePos::new(self.pos_within_chunk.x as u32, self.pos_within_chunk.y as u32)}
    pub fn tile_inst_data_entity(&self) -> Entity {self.tile_inst_data_entity}
}

