use std::hash::Hasher;

use bevy::{math::{U16Vec2, U8Vec2}, prelude::*};
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};



pub const Z_DIVISOR: i32 = 10000;

pub const GRASS_Z_LEVEL: i32 = 10 * Z_DIVISOR;



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

