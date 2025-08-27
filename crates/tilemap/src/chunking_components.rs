#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_replicon::prelude::Replicated;
use bevy_replicon_renet::renet::RenetServer;
use debug_unwraps::{DebugUnwrapErrExt, DebugUnwrapExt};
use game_common::{game_common_components::DimensionRef, game_common_components_samplers::EntiWeightedSampler};
use superstate::{SuperstateInfo};
use serde::{Deserialize, Serialize};
use bevy::{ecs::{entity::EntityHashSet, entity_disabling::Disabled}, platform::collections::HashMap, prelude::*};
use tilemap_shared::{AaGlobalGenSettings, ChunkPos, GlobalTilePos};

use crate::{terrain_gen::{terrgen_oplist_components::OplistSize,}, tile::tile_components::{InitialPos, Tile, TileRef, TilemapChild},};


use common::{common_components::*, };

#[derive(Component, Default)]
#[require(SuperstateInfo<ChunkInitState>, SessionScoped, )]
pub struct ChunkInitState;

#[derive(Component, Debug, Default, )]
#[require(ChunkInitState)]
#[require(Visibility::Hidden)]
pub struct UninitializedChunk;

#[derive(Component, Debug, Default,)]
#[require(ChunkInitState)]
pub struct TilesInstantiated;


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
        let chunk_area = ChunkPos::CHUNK_SIZE.element_product();
        let cap = chunk_area + chunk_area / 8;//TODO CALCULAR PROMEDIO DE TILES POR CHUNK
        ProducedTiles(Vec::with_capacity(cap as usize))
    }
    pub fn new<I>(entities: I) -> Self where I: IntoIterator, I::Item: Into<Entity>, {
        ProducedTiles(entities.into_iter().map(Into::into).collect())
    }

    pub fn produced_tiles(&self) -> &[Entity] { &self.0 }

    pub fn push(&mut self, entity: Entity) {self.0.push(entity);}

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Entity> { self.0.iter_mut() }


    #[allow(unused_parens, )]
    fn insert_tile_recursive(
        &mut self,
        cmd: &mut Commands,
        tiling_ent: Entity,
        tile_pos: TilePos,
        global_pos: GlobalTilePos,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>,
        gen_settings: &AaGlobalGenSettings,
        depth: u32
    ) {
        if let Ok((wmap, )) = weight_maps.get(tiling_ent) {
            if let Some(tiling_ent) = wmap.sample_with_pos(gen_settings, global_pos) {

                if depth > 6 {
                    warn!("Tile insertion depth exceeded 6, stopping recursion for tile {:?}", tiling_ent);
                    return;
                }

                self.insert_tile_recursive( cmd, tiling_ent, tile_pos, global_pos, oplist_size, weight_maps, gen_settings, depth + 1);
            }
        } else { 
            let tile_ent = cmd.entity(tiling_ent).clone_and_spawn_with(|builder|{
                builder.deny::<(/*DisplayName, StrId*/)>();
            })
             .try_insert((
                Disabled, TilemapChild,
                tile_pos, TileRef(tiling_ent), InitialPos(global_pos), global_pos, oplist_size))
            .id();
            self.0.push(tile_ent);

        }
    }

    pub fn insert_as_instanced_tiles(
        &mut self,
        cmd: &mut Commands,
        tiling_ents: &Vec<Entity>,
        pos_within_chunk: TilePos,
        global_pos: GlobalTilePos,
        oplist_size: OplistSize,
        weight_maps: &Query<(&EntiWeightedSampler,), ()>,
        gen_settings: &AaGlobalGenSettings,
    ) {
        for tile in tiling_ents.iter().cloned() {
            self.insert_tile_recursive(cmd, tile, pos_within_chunk, global_pos, oplist_size, weight_maps, gen_settings, 0);
        }
    }
}





#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Reflect)]
pub struct PendingOperations(pub i32);





#[derive(Component, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct ActivatingChunks(#[entities] pub EntityHashSet,);




            // unsafe {
            // let (tilemap_child, transform) = tilemap_child.get(tiling_ent).debug_expect_unchecked("asdasda");

            // if tilemap_child {
            //     trace!("Inserting tile {:?} at {:?} with pos within chunk {:?}", tiling_ent, global_pos, pos_within_chunk);
            //     cmd.entity(tile_ent).try_insert((oplist_size, pos_within_chunk, global_pos));
            //     self.0.push(tile_ent);
            // } 
            // else if is_server {
            //     trace!("Inserting tile {:?} at {:?} with pos within chunk {:?}, but it is not a TilemapChild", tiling_ent, global_pos, pos_within_chunk);
            //     cmd.entity(tile_ent).try_insert((Replicated, ChildOf(dimension_ref.0), )).try_remove::<Tile>().try_remove::<Disabled>();
            //     let displacement: Vec3 = Into::<Vec2>::into(global_pos).extend(0.0);
            //     info!("Displacement for tile {:?} is {:?}", tile_ent, displacement);
            //     if let Some(transform) = transform {
            //         cmd.entity(tile_ent).try_insert(Transform::from_translation( transform.translation + displacement));
            //     } 
            // }
    
            // }