use bevy::prelude::*;

use crate::{DiagonalCardinalDirection, DimensionRef, GlobalTilePos};

#[derive(Message, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RecheckTileAdjacency {
    pub dim: DimensionRef,
    pub gpos: GlobalTilePos,
}
impl RecheckTileAdjacency {
    pub fn append_all_adjacent_pos(msgs: &mut Vec<RecheckTileAdjacency>, dim: DimensionRef, base_pos: GlobalTilePos,) {
        for dir in DiagonalCardinalDirection::ALL_DIRS {
            msgs.push(RecheckTileAdjacency {
                dim,
                gpos: base_pos.adjacent_dir(dir),
            });
        }
    }
}

#[derive(Message, Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// Despawn with removal from SpriteTilesAtGpos (if spritetile) and tile adjacency recheck
pub struct SafeDespawn { pub tile_ent: Entity, pub remove_u16_index: bool }
