use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;


#[derive(Component, Debug, Default, )]
pub struct ActivatesChunks(pub HashSet<Entity>,);


use superstate::{SuperstateInfo};

use crate::game::tilemap::{terrain_gen::terrain_gen_utils::UniqueTileDto, chunking_resources::{CHUNK_SIZE, TILE_SIZE_PXS}};

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkInitState>)]
pub struct ChunkInitState;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
#[require(Visibility::Hidden)]
pub struct UninitializedChunk;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
pub struct TilesReady(pub Vec<UniqueTileDto>);


#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
pub struct LayersReady;

#[derive(Component, Debug, Default,)]
#[require(ChunkInitState)]
pub struct InitializedChunk;



#[derive(Component, Debug, Default, )]
pub struct ChunkPos(pub IVec2);

impl ChunkPos {
    
    pub fn to_pixelpos(&self) -> Vec2 {
         self.0.as_vec2() * TILE_SIZE_PXS.as_vec2() * CHUNK_SIZE.as_vec2() 
    }
    pub fn to_tilepos(&self) -> IVec2 {
        self.0 * CHUNK_SIZE.as_ivec2()
    }
}

pub fn contpos_to_chunkpos(contpos: Vec2) -> IVec2 {
    contpos.as_ivec2().div_euclid(TILE_SIZE_PXS.as_ivec2() * CHUNK_SIZE.as_ivec2())
}


pub fn pixelpos_to_tilepos(pixelpos: Vec2) -> IVec2 {
    pixelpos.div_euclid(TILE_SIZE_PXS.as_vec2()).as_ivec2()
}
