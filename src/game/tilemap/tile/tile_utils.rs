use bevy::math::U8Vec2;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::tilemap::{chunking_components::TilesReady, tile::tile_components::*};

// pub fn clone_add_tilepos_and_push(
//     commands: &mut Commands, 
//     tiles2spawn: &mut TilesReady,
//     pos_within_chunk: U8Vec2, 
//     tilepos: IVec2,
//     entity: Entity,
// ) {
//     let entity = commands.entity(entity).clone_and_spawn().insert((
//         TilePos::new(pos_within_chunk.x as u32, pos_within_chunk.y as u32),
//         GlobalTilePos(tilepos),//NO SÉ SI METERLE ESTO O NO, PERO HACE CADA TILE MÁS FÁCILMENTE QUERYABLE POR DISTANCIA
//     )).id();
//     tiles2spawn.0.push(entity);
// }


// pub fn new_tile<B: Bundle>(
//     commands: &mut Commands, 
//     pos_within_chunk: U8Vec2, 
//     bundle: B,
// ) -> Entity {
//     commands.spawn((
//         TilePos::new(pos_within_chunk.x as u32, pos_within_chunk.y as u32),
//         bundle,
//     )).id()
// }

