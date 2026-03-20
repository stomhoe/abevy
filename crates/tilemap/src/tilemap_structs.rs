use bevy::{math::U16Vec2, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use sprite_shared::AcZ;
use std::{collections::HashMap, mem::take};
use tilemap_shared::*;

use crate::tile::tile_shader::tile_shader_components::TileShaderRef;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapKey {
    pub dim_ref: DimensionRef,
    pub chunk_pos: ChunkPos,
    pub ac_z: AcZ,
    pub tile_size: U16Vec2,
    pub shader_ref: Option<TileShaderRef>,
}
impl MapKey {
    pub fn new(
        dim_ref: DimensionRef,
        chunk_pos: ChunkPos,
        ac_z: AcZ,
        tile_size: U16Vec2,
        shader_ref: Option<TileShaderRef>,
    ) -> Self {
        Self { dim_ref, chunk_pos, ac_z, tile_size, shader_ref }
    }
    pub fn shader_ref(&self) -> Option<TileShaderRef> { self.shader_ref }
}

#[derive(Debug, Clone)]
pub struct MapStruct {
    pub tmap_ent: Entity,
    pub texture: TilemapTexture,
    pub storage: TileStorage,
    pub tmap_hash_id_map: HashIdToTexIndex,
}
impl MapStruct {
    pub fn take_texture(&mut self) -> TilemapTexture { take(&mut self.texture) }
    pub fn take_storage(&mut self) -> TileStorage { take(&mut self.storage) }
    pub fn take_hash_id_map(&mut self) -> HashIdToTexIndex { take(&mut self.tmap_hash_id_map) }
}

#[derive(Resource, Debug, Default)]
pub struct TmapMap(pub HashMap<MapKey, MapStruct>);

#[derive(Component, Debug, Default)]
pub struct NeedsTerrblRefresh;
