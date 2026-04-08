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
    body_build_systems::*,
    body_templ_init_systems::*,
};

pub mod body_components;
pub mod bodypart;
pub mod body_resources;
pub mod body_seris;
pub mod body_sampler;
pub mod bodytree;
mod body_systems;
mod body_hp_systems;
mod body_build_systems;
mod body_templ_init_systems;
#[allow(unused_imports)] pub use body_components::*;
#[allow(unused_imports)] pub use body_resources::*;
#[allow(unused_imports)] pub use body_seris::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        body_sampler::plugin,
        bodypart::plugin,
        bodytree::plugin,
        plugin_body,
    ))
    .add_systems(
        Update,
        (
            update_body_weight_sum,
            apply_damage.run_if(on_message::<IncHealthDamageOrHeal>),
            refresh_template_bodyparts_users_list,
            update_bodypart_max_hp_map.run_if(on_timer(core::time::Duration::from_millis(200))),
            set_bodypart_as_missing_if_0_hp,
            update_body_health_from_parts.run_if(on_timer(core::time::Duration::from_millis(200))),
            apply_bodypart_hp_regen,
            ensure_pain_slowdown_modifiers,
        ),
    )
    .add_systems(
        Update,
        build_bodys_on_beings.in_set(HostSystems).in_set(ModifierSystems),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            (init_templ_bodys, ApplyDeferred, map_body_id_to_entity).chain().in_set(BodySystems),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            crate::body::bodypart::BodypartSystems.before(BodySystems),
            crate::body::bodypart::BodypartSystems.before(bodytree::BodyTreeSystems),
            bodytree::BodyTreeSystems.before(BodySystems),
            BodySystems.before(body_sampler::BodySamplerSystems),
        ),
    )
    .replicate::<Body>()
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
