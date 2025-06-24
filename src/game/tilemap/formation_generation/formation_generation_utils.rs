use std::hash::Hasher;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::*;
use strum_macros::{Display, EnumCount, EnumIter, VariantNames};

use crate::game::tilemap::{formation_generation::{formation_generation_components::NoiseComp, formation_generation_resources::WorldGenSettings}, tilemap_resources::{chunkpos_to_contpos, TILE_SIZE_PXS, CHUNK_SIZE}};



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

pub struct TileInfo{
    pub layer_z: i16,
    pub pos_within_chunk: UVec2,
    pub used_handle: Handle<Image>,//NO SÉ SI USAR ESTO O UNA ID O ALGO ASÍ EN VEZ DE ESTE SHARED POINTER
    pub flip: TileFlip,
    pub color: TileColor,
    pub needs_y_sort: bool,
}

#[allow(unused_parens)]
pub fn gather_all_tiles2spawn_within_chunk (
    mut commands: &mut Commands, 
    asset_server: &AssetServer, 
    noise_query: Query<&NoiseComp>, 
    gen_settings: &WorldGenSettings,
    chunk_pos: IVec2,) -> Vec<TileInfo> {
    
    let mut tiles2spawn: Vec<TileInfo> = Vec::new();
    
    println!("chunk_pos gather {} ", chunk_pos);
    println!("chunk_contpos gather {} ", chunkpos_to_contpos(chunk_pos)/TILE_SIZE_PXS.as_vec2());
    for x in 0..CHUNK_SIZE.x { 
        for y in 0..CHUNK_SIZE.y {
            let pos_within_chunk = UVec2::new(x, y);
            
            let x = x as i32 + chunk_pos.x; let y = y as i32;
            
            let worldgrid_pos = chunkpos_to_contpos(chunk_pos);
            let asd = Vec2::new(x as f32,y as f32)/CHUNK_SIZE.as_vec2() + worldgrid_pos;

            
            //el super match o lo q sea

            let asd: Handle<Image> = asset_server.load("textures/world/bushes/bush.png");
            
            let tileinfo = TileInfo {
                layer_z: 1,
                pos_within_chunk,
                used_handle: asd,
                flip: TileFlip::default(),
                color: TileColor::default(),
                needs_y_sort: false,
            };
            
            tiles2spawn.push(tileinfo);   
    }}
    tiles2spawn
}
