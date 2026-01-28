use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, }, prelude::*};

use crate::{chunking_resources::AaChunkRangeSettings, regioning::regioning_components::ChunksActiveInRegion, };
use ::tilemap_shared::*;


use common::{common_components::*, };

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, Reflect, )]
#[relationship(relationship_target = ChunksActiveInRegion)]
pub struct Chunk {
    #[relationship]
    pub region_ent: Entity,
}


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect)]
pub struct SaveTile {
    pub chunk_pos: ChunkPos,//NO HACE FALTA PORQ EL CHUNKPOS SE PUEDE CALCULAR A PARTIR DE GLOBAL POS
}



#[derive(Component, Debug, Reflect, Default,)]
pub struct TilesToSave(pub EntityHashSet);
impl TilesToSave { pub fn entities(&self) -> &EntityHashSet { &self.0 } }





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TerrGenOpsLaunched;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct ReadyForTerrgen;

#[derive(Component, Debug, Reflect)]
pub struct ActivatingChunks {
    pub reactivation_timer: Timer,
    pub entities: Vec<Entity>,
}

impl ActivatingChunks {
    pub fn new(chunkrange: &AaChunkRangeSettings) -> Self { 
        Self {
            entities: Vec::with_capacity((chunkrange.approximate_number_of_chunks(1.2)) as usize),
            reactivation_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }

}

