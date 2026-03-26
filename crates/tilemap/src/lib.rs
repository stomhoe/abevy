#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

pub use tilemap::*;
pub mod tilemap;

mod tilemap_systems;
mod tilemap_despawn_systems;
mod tilemap_structs;
mod tilemap_terrbl_systems;
pub mod chunking;
pub mod tilemap_bundles;
pub mod tilemap_resources;

pub mod regioning;
pub mod tile;
pub mod terrain;
