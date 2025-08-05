use bevy::math::U8Vec2;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use crate::game::tilemap::{chunking_components::TilesReady, tile::tile_components::*};



pub const TILE_SIZE_PXS: UVec2 = UVec2 { x: 64, y: 64 };
