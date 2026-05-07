use bevy::prelude::*;
use bitvec::prelude::*;
use common::common_components::HashId;
use serde::{Deserialize, Serialize};

pub use crate::being_components::*;
pub use crate::entities_at_gpos::*;
pub use crate::tile::*;
pub use crate::tilemap_components::*;
pub use crate::tilemap_shared_samplers::*;
pub use crate::tilemap_messages::*;
pub use crate::tilemap_nav::*;
pub use crate::tilemap_param_sets::*;
pub use crate::terrgen_components::*;
pub use crate::chunking_shared_components::*;
pub use crate::chunking_shared_resources::*;
#[allow(unused_imports)] pub use crate::chunking_shared_messages::*;
pub use crate::dimension::*;
pub use crate::tilemap_positioning::*;
pub use crate::directions::*;
pub use crate::regioning_shared::*;
pub use crate::regioning_messages::*;
#[allow(unused_imports)]
pub use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
pub use bevy::ecs::entity::{EntityHashMap, EntityHashSet};

#[derive(Resource, Debug, Clone, Copy, Deserialize, Serialize, Reflect)]
#[serde(default)]
pub struct ZSettings {
    pub y_mult: f32,
    pub y_sort_mult: f32,
    pub sprite_z: f32,
    pub tile_z_unset: f32,
}

impl Default for ZSettings {
    fn default() -> Self {
        Self {
            y_mult: 1e-2,
            y_sort_mult: 1e-6,
            sprite_z: 1000.0,
            tile_z_unset: -10.0,
        }
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct AcZ(pub f32);
impl AcZ {
    pub fn new(z: f32) -> Self {
        Self(z)
    }
    pub fn used_float(&self, y_sort_settings: &ZSettings) -> f32 {
        self.0 * y_sort_settings.y_mult
    }
}
impl PartialEq for AcZ {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for AcZ {}
impl std::hash::Hash for AcZ {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

#[derive(Resource, Debug, Default)]
pub struct CardinalDirAtGpos (
    pub HashMap<(HashId, GlobalTilePos), CardinalDirection>,
);
impl CardinalDirAtGpos {
    pub fn resolve_tile_direction(
        &self,
        hash_id_query: &Query<&HashId, common::AnyDisabling>,
        templ_ent: Entity,
        gpos: GlobalTilePos,
        fallback: CardinalDirection,
    ) -> CardinalDirection {
        let Ok(hash_id) = hash_id_query.get(templ_ent) else {
            return fallback;
        };
        self
            .0
            .get(&(*hash_id, gpos))
            .copied()
            .unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGposMask(pub BitArr!(for ChunkPos::CHUNK_AREA));
impl ChunkGposMask {
    pub fn is_empty(&self) -> bool {
        self.0.as_bitslice().count_ones() == 0
    }

    pub fn count_set(&self) -> usize {
        self.0.as_bitslice().count_ones()
    }

    pub fn is_set(&self, bit_idx: usize) -> bool {
        self.0.as_bitslice().get(bit_idx).is_some_and(|bit| *bit)
    }

    pub fn set_bit(&mut self, bit_idx: usize) {
        self.0.as_mut_bitslice().set(bit_idx, true);
    }

    pub fn clear_bit(&mut self, bit_idx: usize) {
        self.0.as_mut_bitslice().set(bit_idx, false);
    }

    pub fn set_gpos(&mut self, chunk_pos: ChunkPos, gpos: GlobalTilePos) {
        let Some(bit_idx) = chunk_pos.bit_index_in_chunk(gpos) else {
            return;
        };
        self.set_bit(bit_idx);
    }

    pub fn clear_gpos(&mut self, chunk_pos: ChunkPos, gpos: GlobalTilePos) {
        let Some(bit_idx) = chunk_pos.bit_index_in_chunk(gpos) else {
            return;
        };
        self.clear_bit(bit_idx);
    }
}

impl Default for ChunkGposMask {
    fn default() -> Self {
        Self(BitArray::ZERO)
    }
}
