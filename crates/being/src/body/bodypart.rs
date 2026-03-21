#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use ::being_shared::*;

pub use bodypart_resources::*;
pub use bodypart_seris::*;

pub mod bodypart_resources;
pub mod bodypart_seris;
mod bodypart_init_systems;
use crate::body::bodypart::bodypart_init_systems::*;

use common::common_states::AssetLoading;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodypartSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_bodypart,
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
    (init_bodyparts, map_bodypart_id_to_entity).chain().in_set(BodypartSystems),
    ))
    //.register_type::<Bodyparts>()
    .replicate::<BodypartChildOfBodypart>()
    .replicate::<TreeRoot>()
    .replicate::<Vital>()
    .replicate::<Missing>()
    .replicate::<BodypartDamage>()
    ;
}
