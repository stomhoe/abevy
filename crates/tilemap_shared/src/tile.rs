pub use crate::tile::adj_retex::*;
pub use crate::tile::based_on_hash::*;
pub use crate::tile::delete_other_tiles::*;
pub use crate::interaction_zones::*;
pub use crate::portal::*;
pub use crate::tile::terr_blend::*;
pub use crate::tile::tile_distancing::*;
pub use crate::tile::tile_components::*;
pub use crate::tile::tile_shared_seris::*;

pub mod adj_retex;
pub mod based_on_hash;
pub mod delete_other_tiles;
pub mod terr_blend;
pub mod tile_distancing;
pub mod tile_components;
pub mod tile_resources;
pub mod tile_shared_seris;

/*
use serde::{Serialize, Deserialize};
#[allow(unused_imports)] use bevy::platform::collections::HashMap;
use bevy::{ecs::entity::MapEntities, prelude::*};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
//Don't add RequiredComponents here because it is forced onto clones and when removed it despawns the new entity
pub struct Tile;
impl Tile {
    pub const MIN_ID_LENGTH: u8 = 1;
}

common::define_entity_map_systems!(
    main_component: Tile,
    with_filters: (With<Templ>, common::AnyDisabling),
    abbreviation: Tile,
    target: "",
    entity_prefix: "tile",
    despawn_trigger: Tile,
    id_type: common::common_components::StrId,
    assets: [(TileSeri, "seri.tilemap.tile", "tile.ron")],
    templ_enti_ref_sync: (common::AnyDisabling,),
);
*/