#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

pub use tilemap::*;
pub mod tilemap;

mod tilemap_systems;
mod tilemap_nav_systems;
mod tilemap_despawn_systems;
mod tilemap_structs;
mod tilemap_terrbl_systems;
pub mod chunking;
pub mod tilemap_bundles;
pub mod tilemap_resources;
pub mod tile;
pub mod terrain;

pub use crate::tilemap_systems::process_tiles_pre;
