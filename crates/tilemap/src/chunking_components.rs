#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::prelude::Replicated;
use bevy_replicon_renet::renet::RenetServer;
use debug_unwraps::{DebugUnwrapErrExt, DebugUnwrapExt};
use dimension::dimension_components::DimensionRef;
use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled}, platform::collections::HashMap, prelude::*};

use crate::{terrain_gen::{terrgen_components::OplistSize, terrgen_resources::GlobalGenSettings}, tile::tile_components::{GlobalTilePos, HashPosEntiWeightedSampler, InitialPos, Tile, TileRef, TilemapChild},};


use common::{common_components::*, };

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkInitState>, SessionScoped, )]
pub struct ChunkInitState;
impl ChunkInitState {
    pub const SIZE: UVec2 = UVec2 { x: 12, y: 12 };
}

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
#[require(Visibility::Hidden)]
pub struct UninitializedChunk;

#[derive(Component, Debug)]
#[require(ChunkInitState)]
pub struct TilesReady;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
pub struct LayersReady;

#[derive(Component, Debug, Default,)]
#[require(ChunkInitState)]
pub struct InitializedChunk;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Reflect, Default, )]
pub struct ProducedTiles(#[entities] pub Vec<Entity>);

impl ProducedTiles {
    pub fn new_with_chunk_capacity() -> Self {
        let chunk_area = ChunkInitState::SIZE.element_product();
        let cap = chunk_area + chunk_area / 8;//TODO CALCULAR PROMEDIO DE TILES POR CHUNK
        ProducedTiles(Vec::with_capacity(cap as usize))
    }
    pub fn new<I>(entities: I) -> Self where I: IntoIterator, I::Item: Into<Entity>, {
        ProducedTiles(entities.into_iter().map(Into::into).collect())
    }

    pub fn produced_tiles(&self) -> &[Entity] { &self.0 }

    pub fn push(&mut self, entity: Entity) {self.0.push(entity);}

    #[allow(unused_parens, )]
    fn insert_tile_recursive(
        &mut self,
        tiling_ent: Entity,
        cmd: &mut Commands,
        global_pos: GlobalTilePos,
        pos_within_chunk: TilePos,
        weight_maps: &Query<(&HashPosEntiWeightedSampler,), ()>,
        tilemap_child: &Query<(Has<TilemapChild>, Option<&Transform>), (With<Tile>, With<Disabled>)>,//NO MUTAR
        gen_settings: &GlobalGenSettings,
        oplist_size: OplistSize,
        dimension_ref: DimensionRef,
        is_server: bool,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample(gen_settings, global_pos) {
                //info!("Inserting tile {:?} at {:?} with pos within chunk {:?}", tiling_ent, global_pos, pos_within_chunk);

                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile at {:?} with pos within chunk {:?}", global_pos, pos_within_chunk);
                    return;
                }

                self.insert_tile_recursive( tiling_ent, cmd, global_pos, pos_within_chunk, weight_maps, 
                    tilemap_child, gen_settings, oplist_size, dimension_ref, is_server, depth + 1);
            }
        } else {//TODO EXTRAER A UN SISTEMA TODAS LAS COSAS DE SPAWNING
            let tile_ent = cmd.entity(tiling_ent).clone_and_spawn().try_insert(
                (TileRef(tiling_ent), dimension_ref, InitialPos(global_pos)))
                .remove::<DisplayName>()
                .id();
            unsafe {
            let (tilemap_child, transform) = tilemap_child.get(tiling_ent).debug_expect_unchecked("asdasda");

            if tilemap_child {
                trace!("Inserting tile {:?} at {:?} with pos within chunk {:?}", tiling_ent, global_pos, pos_within_chunk);
                cmd.entity(tile_ent).try_insert((oplist_size, pos_within_chunk, global_pos));
                self.0.push(tile_ent);
            } 
            else if is_server {
                trace!("Inserting tile {:?} at {:?} with pos within chunk {:?}, but it is not a TilemapChild", tiling_ent, global_pos, pos_within_chunk);
                cmd.entity(tile_ent).try_insert((Replicated, ChildOf(dimension_ref.0), )).try_remove::<Tile>().try_remove::<Disabled>();
                let displacement: Vec3 = Into::<Vec2>::into(global_pos).extend(0.0);
                info!("Displacement for tile {:?} is {:?}", tile_ent, displacement);
                if let Some(transform) = transform {
                    cmd.entity(tile_ent).try_insert(Transform::from_translation( transform.translation + displacement));
                } 
            }
    
            }
        }
    }

    pub fn insert_clonespawned_with_pos(
        &mut self,
        tiling_ents: &Vec<Entity>,
        cmd: &mut Commands,
        global_pos: GlobalTilePos,
        pos_within_chunk: TilePos,
        weight_maps: &Query<(&HashPosEntiWeightedSampler,), ()>,
        tilemap_child: &Query<(Has<TilemapChild>, Option<&Transform>), (With<Tile>, With<Disabled>)>,
        gen_settings: &GlobalGenSettings,
        oplist_size: OplistSize,
        dimension_ref: DimensionRef,
        is_server: bool
    ) {
        for tile in tiling_ents.iter().cloned() {
            self.insert_tile_recursive(tile, cmd, global_pos, pos_within_chunk, weight_maps, tilemap_child, gen_settings, oplist_size, dimension_ref, is_server, 0);
        }
    }
}





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct PendingOperations(pub i32);





#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct ActivatingChunks(#[entities] pub EntityHashSet,);



#[derive(Component, Default, Clone, Deserialize, Serialize, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct ChunkPos(pub IVec2);

impl ChunkPos {
    pub fn new(x: i32, y: i32) -> Self { Self(IVec2::new(x, y)) }
    pub fn x(&self) -> i32 { self.0.x }
    pub fn y(&self) -> i32 { self.0.y }
}

impl std::fmt::Debug for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkPos({}, {})", self.0.x, self.0.y)
    }
}

impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0.x, self.0.y)
    }
}

impl ChunkPos {
    pub fn to_pixelpos(&self) -> Vec2 {
        self.0.as_vec2() * Tile::PIXELS.as_vec2() * ChunkInitState::SIZE.as_vec2()
    }
    pub fn to_tilepos(&self) -> GlobalTilePos {
        GlobalTilePos(self.0 * ChunkInitState::SIZE.as_ivec2())
    }
}

impl From<GlobalTilePos> for ChunkPos {
    fn from(global_tile_pos: GlobalTilePos) -> Self {
        ChunkPos(global_tile_pos.0.div_euclid(ChunkInitState::SIZE.as_ivec2()))
    }
}

impl From<Vec2> for ChunkPos {
    fn from(pixel_pos: Vec2) -> Self {
        ChunkPos(pixel_pos.as_ivec2().div_euclid(Tile::PIXELS.as_ivec2() * ChunkInitState::SIZE.as_ivec2()))
    }
}

impl std::ops::Add for ChunkPos {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output { ChunkPos(self.0 + rhs.0) }
}
impl std::ops::Sub for ChunkPos {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output { ChunkPos(self.0 - rhs.0) }
}


