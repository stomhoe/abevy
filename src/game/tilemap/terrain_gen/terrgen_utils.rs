use std::hash::Hasher;

use bevy::{math::{U16Vec2, U8Vec2}, prelude::*};
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};

use crate::game::tilemap::{chunking_components::*, tile::tile_components::GlobalTilePos};





pub const TC_RED: TileColor = TileColor(Color::srgb(1., 0., 0.));
pub const TC_GREEN: TileColor = TileColor(Color::srgb(0., 1., 0.));
pub const TC_BLUE: TileColor = TileColor(Color::srgb(0., 0., 1.));



// fn add_tiles_for_tilepos(mut cmd: &mut Commands, tiles2spawn: &mut TilesReady, 
//     noise_query: Query<&FnlComp>, tilepos: IVec2, pos_within_chunk: U8Vec2, 
//     mut clonable_tiles: Query<Entity, (With<Tileimg>, Without<TilePos>)>,
//     rng : &mut Pcg64,

// ) -> Result {
//     //si una tile es suitable para una edificación, o spawnear una village o algo, se le puede añadir un componente SuitableForVillage o algo así, para que se pueda identificar la tile. después se puede hacer un sistema q borre los árboles molestos en un cierto radio. el problema es si hay múltiples marcadas adyacentemente, en ese caso va a haber q chequear distancias a otras villages
//     let mut grass = clonable_tiles.transmute_lens_filtered::<(Entity), (With<TemperateGrass>, Without<TilePos>)>();
//     let grass = grass.query().single()?;
//     clone_add_tilepos_and_push(
//         &mut cmd, 
//         tiles2spawn, 
//         pos_within_chunk, 
//         tilepos,
//         grass, 
//     );
//     Ok(()) 
// }


//add_tiles_for_tilepos( &mut commands, &mut tiles_ready, noise_query, tilepos, pos_within_chunk, clonable_tiles, &mut rng)?;
