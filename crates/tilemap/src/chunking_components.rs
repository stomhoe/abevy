use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, }, platform::collections::{HashMap, HashSet}, prelude::*};
use sprite_shared::AcZ;

use crate::{chunking_resources::AaChunkRangeSettings, regioning::regioning_components::ChunksActiveInRegion, };
use ::tilemap_shared::*;


use common::{common_components::*, };

#[derive(Component, Debug, Copy, Clone, Hash, PartialEq, Eq, Reflect, )]
#[relationship(relationship_target = ChunksActiveInRegion)]

#[require(Visibility::Hidden, AssetScoped, AppStateScoped, ChunkTmapsMap, TilesToSave, )]
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

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct TerrgenDisallowed;

use crate::tilemap_systems::{MapKey, MapStruct};

#[derive(Component, Default, Clone, Reflect, )]
pub struct ChunkTmapsMap(pub HashMap<AcZ, HashMap<MapKey, MapStruct>>);


#[derive(Component, Debug, Reflect)]
pub struct ActivatingChunks(pub Vec<Entity>,);

impl ActivatingChunks {
    pub fn new(chunkrange: &AaChunkRangeSettings) -> Self { 
        Self(Vec::with_capacity((chunkrange.approximate_number_of_chunks(1.2)) as usize)) 
    }

}