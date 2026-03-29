use ::being_shared::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy::ecs::schedule::common_conditions::on_message;
use bevy::ecs::schedule::ApplyDeferred;
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::{HostSystems, game_common::ModifierSystems};
use crate::body::{
    body_systems::*,
    body_hp_systems::*,
    body_tree_build_systems::*,
    body_tree_templ_init_systems::*,
};

pub mod body_tree_components;
pub mod bodypart;
pub mod body_tree_resources;
pub mod body_tree_seris;
pub mod body_sampler;
mod body_systems;
mod body_hp_systems;
mod body_tree_build_systems;
mod body_tree_templ_init_systems;
#[allow(unused_imports)] pub use body_tree_components::*;
#[allow(unused_imports)] pub use body_tree_resources::*;
#[allow(unused_imports)] pub use body_tree_seris::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        body_sampler::plugin,
        bodypart::plugin,
        plugin_body_tree,
    ))
    .add_systems(
        Update,
        (
            update_body_tree_weight_sum.in_set(ModifierSystems),
            (
                apply_damage.run_if(on_message::<IncHealthDamageOrHeal>),
                refresh_template_bodyparts_users_list.before(update_body_health_from_parts),
                update_bodypart_max_hp_map,
                set_bodypart_as_missing_if_0_hp,
                update_body_health_from_parts.run_if(on_timer(core::time::Duration::from_millis(200))),
                apply_bodypart_hp_regen,
                ensure_pain_slowdown_modifiers,
                build_body_trees_on_beings,
            )
            .in_set(HostSystems)
            .in_set(ModifierSystems),
        ),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            (init_templ_body_trees, ApplyDeferred, distribute_templ_body_tree_modifiers, map_body_tree_id_to_entity).chain().in_set(BodySystems),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            crate::body::bodypart::BodypartSystems.before(BodySystems),
            BodySystems.before(body_sampler::BodySamplerSystems),
        ),
    )

    //.replicate::<BodyTreeWeightSum>()

    .replicate::<BodyTree>()
    .replicate::<BodyOf>()
    .replicate_filtered::<ChildOf, With<BodyOf>>()
    .replicate_filtered::<ChildOf, With<BodypartChildOfBodypart>>()
    .replicate::<BodySums>()
    .init_resource::<BodypartMaxHpMap>()
    .init_resource::<BodypartTemplateByPart>()
    .add_message::<IncHealthDamageOrHeal>()
    //TEMPORAL
    .register_type::<BodypartChildrenBodyparts>()
    ;
}
