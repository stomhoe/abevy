use bevy::prelude::*;

use crate::game::{tilemap::{terrain_gen::TerrainGenPlugin, tile_imgs::*, tilemap_resources::*, tilemap_systems::*, chunking_systems::*}, IngameSystems, SimRunningSystems};

mod tilemap_systems;
mod chunking_systems;
pub mod tilemap_components;
pub mod tilemap_resources;
pub mod tile_nids;

pub mod tile_imgs;

pub mod terrain_gen;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ChunkVisibSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TilemapsSystems;

pub struct MyTileMapPlugin;
#[allow(unused_parens)]
impl Plugin for MyTileMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((bevy_ecs_tilemap::TilemapPlugin, TerrainGenPlugin))
            .add_systems(FixedUpdate, (
                visit_chunks_around_activators, 
                rem_outofrange_chunks_from_activators, 
                despawn_unreferenced_chunks.before(produce_tilemaps), 
                show_chunks_around_camera, 
                hide_outofrange_chunks, 

            ).in_set(IngameSystems).in_set(ChunkVisibSystems))
            .add_systems(FixedUpdate, (
                produce_tilemaps.before(fill_tilemaps_data),
                fill_tilemaps_data,
                
            ).in_set(IngameSystems).in_set(TilemapsSystems))
            .init_resource::<LoadedChunks>()
            .init_resource::<ChunkRangeSettings>()
            .init_resource::<NidImgMap>()
            .init_resource::<NidRepeatImgMap>()
            .add_systems(Startup, (tile_imgs::setup_nid_img_map, tile_imgs::setup_rep_img_map))
        ;
    }
}