pub use body_tree_components::*;
pub use body_part::body_part_components::*;
pub use body_tree_resources::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::game_common::ModifierSystems;
use crate::body::{
    body_tree_resources::*,
    body_systems::*,
    body_tree_build_systems::*,
    body_tree_ezero_init_systems::*,
};
use body_sampler::BodySamplerSystems;

pub mod body_tree_components;
pub mod body_part;
pub mod body_tree_resources;
pub mod body_sampler;
mod body_systems;
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
            apply_body_damage,
            sync_body_part_missing,
            update_body_health_from_parts,
            apply_pain_slowdown,
            build_body_tree,
        )
        .in_set(ModifierSystems),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            init_ezero_body_trees.in_set(BodySystems),
            map_body_tree_id_to_entity.in_set(BodySystems),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            crate::body::body_part::BodyPartSystems.before(BodySystems),
            BodySystems.before(BodySamplerSystems),
        ),
    )
    .register_type::<BodyTreeSerisHandles>()
    .register_type::<BodyTreeSeri>()
    .register_type::<BodyHealth>()
    .register_type::<BodyDead>()
    .register_type::<BodyDamage>()

    .replicate::<BodyTree>()
    .replicate::<BodyHealth>()
    .replicate::<BodyDead>()
    .add_message::<BodyDamage>();
}
