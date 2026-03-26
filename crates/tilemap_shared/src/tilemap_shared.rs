use bevy::prelude::*;
use common::common_components::HashId;
use serde::{Deserialize, Serialize};

pub use crate::being_components::*;
pub use crate::entities_at_gpos::*;
pub use crate::tile::*;
pub use crate::tilemap_components::*;
pub use crate::tilemap_shared_samplers::*;
pub use crate::tilemap_messages::*;
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
    const Z_MULT: f32 = 1e-3;

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
