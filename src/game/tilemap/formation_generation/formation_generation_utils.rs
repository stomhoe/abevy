use std::hash::Hasher;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::*;
use strum_macros::{Display, EnumCount, EnumIter, VariantNames};

use crate::game::tilemap::{formation_generation::{formation_generation_components::FnlComp, formation_generation_resources::WorldGenSettings}, tilemap_resources::{chunkpos_to_pixelpos, chunkpos_to_tilepos, pixelpos_to_tilepos, CHUNK_SIZE, TILE_SIZE_PXS}};



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

#[derive(Debug, Default)]
pub struct TileInfo{
    pub layer_z: i16,
    pub pos_within_chunk: UVec2,
    pub used_handle: Handle<Image>,//NO SÉ SI USAR ESTO O UNA ID O ALGO ASÍ EN VEZ DE ESTE SHARED POINTER
    pub flip: TileFlip,
    pub color: TileColor,
    pub needs_y_sort: bool,
    pub visible: TileVisible,
    pub entity: Option<Entity>, 
}

#[allow(unused_parens)]
pub fn gather_all_tiles2spawn_within_chunk (
    mut commands: &mut Commands, 
    asset_server: &AssetServer, 
    noise_query: Query<&FnlComp>, 
    gen_settings: &WorldGenSettings,
    chunk_pos: IVec2,) -> Vec<TileInfo> {
    
    let mut tiles2spawn: Vec<TileInfo> = Vec::new();
    
    for x in 0..CHUNK_SIZE.x { 
        for y in 0..CHUNK_SIZE.y {
            let pos_within_chunk = UVec2::new(x, y);
            let tilepos = chunkpos_to_tilepos(chunk_pos) + pos_within_chunk.as_ivec2();
            add_tiles_for_tilepos(&mut tiles2spawn, asset_server, noise_query, tilepos, pos_within_chunk);
    }}
    tiles2spawn
}

fn add_tiles_for_tilepos(tiles2spawn: &mut Vec<TileInfo>, asset_server: &AssetServer, 
    noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: UVec2,
) {
    let asd: Handle<Image> = asset_server.load("textures/world/bushes/bush.png");




    let tileinfo = TileInfo {
        layer_z: 1,
        pos_within_chunk,
        used_handle: asd,
        flip: TileFlip::default(),
        color: TileColor::default(),
        needs_y_sort: false,
        entity: None,
        visible: TileVisible::default(),
    };
    
    tiles2spawn.push(tileinfo);   
}