use std::hash::Hasher;

use bevy::{math::{U16Vec2, U8Vec2}, prelude::*};
use bevy_ecs_tilemap::{map::TilemapTileSize, tiles::*};



pub const Z_DIVISOR: i32 = 10000;

pub const GRASS_Z_LEVEL: i32 = 10 * Z_DIVISOR;

