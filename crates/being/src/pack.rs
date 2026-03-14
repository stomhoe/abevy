use bevy::prelude::*;
#[allow(unused_imports, )]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;

use crate::pack::{pack_init_systems::*, pack_resources::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PackSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_pack,))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            ((init_packs, map_pack_id_to_entity).chain()).in_set(PackSystems),
        );
}

mod pack_init_systems;
pub mod pack_components;
pub mod pack_resources;
pub mod pack_seris;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{pack_components::*, pack_resources::*, pack_seris::*};
}
