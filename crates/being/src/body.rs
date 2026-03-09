pub use body_tree_components::*;
pub use body_part::body_part_components::*;
pub use body_tree_resources::*;
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::on_message;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{HostSystems, game_common::ModifierSystems};
use crate::body::{
    body_hp_systems::*,
    body_systems::*,
    body_tree_build_systems::*,
    body_tree_ezero_init_systems::*,
};
use body_sampler::BodySamplerSystems;

pub mod body_tree_components;
pub mod body_part;
pub mod body_tree_resources;
pub mod body_tree_seris;
pub mod body_sampler;
mod body_systems;
mod body_hp_systems;
mod body_tree_build_systems;
mod body_tree_ezero_init_systems;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        body_sampler::plugin,
        body_part::plugin,
        plugin_body_tree,
    ))
    .add_systems(
        Update,
        (
            (update_body_tree_weight_sum).in_set(ModifierSystems),
            (
                apply_body_damage.run_if(on_message::<BodyDamage>),
                sync_body_part_missing,
                update_body_health_from_parts,
                apply_pain_slowdown,
                build_body_tree,
            )
            .in_set(HostSystems)
            .in_set(ModifierSystems),
        ),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            (init_ezero_body_trees, map_body_tree_id_to_entity).chain().in_set(BodySystems),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            crate::body::body_part::BodyPartSystems.before(BodySystems),
            BodySystems.before(BodySamplerSystems),
        ),
    )

    //.replicate::<BodyTreeWeightSum>()

    .replicate::<BodyTree>()
    .replicate::<BodyOf>()
    .replicate_filtered::<ChildOf, With<BodyOf>>()
    .replicate::<BodySums>()
    .add_message::<BodyDamage>();
}

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        body_tree_components::*,
        body_part::*,
        body_tree_resources::*,
        body_tree_seris::*,
        body_sampler::*,
    };
}
