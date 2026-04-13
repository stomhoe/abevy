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
#[allow(unused_imports)]
pub use bevy::platform::collections::{HashMap, HashSet};
#[allow(unused_imports)]
pub use bevy::ecs::entity::{EntityHashMap, EntityHashSet};

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Copy, Reflect)]
pub struct AcZ(pub f32);
impl AcZ {
    pub fn new(z: f32) -> Self {
        Self(z)
    }
    pub fn used_float(&self) -> f32 {
        self.0 * Self::Z_MULT
    }
    const Z_MULT: f32 = 1e-5;

    pub const Z_SORT_MULT: f32 = 1e-6;
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

    pub fn set_gpos(&mut self, chunk_pos: ChunkPos, gpos: GlobalTilePos) {
        let Some(bit_idx) = chunk_pos.bit_index_in_chunk(gpos) else {
            return;
        };
        self.set_bit(bit_idx);
    }
}

impl Default for ChunkGposMask {
    fn default() -> Self {
        Self(BitArray::ZERO)
    }
}
