pub use body_components::*;
pub use body_part::body_part_components::*;
pub use body_resources::*;

pub mod body_components;
pub mod body_part;
pub mod body_resources;
pub mod body_sampler;
mod body_systems;
mod body_tree_building_systems;
mod body_tree_ezero_init_systems;

#[allow(unused_imports)]
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_states::AssetLoading;
use game_common::game_common::ModifierSystems;

use crate::body::body_part::{BodyPartSystems, init_body_parts};
use crate::body::body_resources::*;
use crate::body::body_systems::*;
use crate::body::body_tree_building_systems::*;
use crate::body::body_tree_ezero_init_systems::*;
use body_sampler::BodySamplerSystems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodySystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        RonAssetPlugin::<BodyTreeSeri>::new(&["bodytree.ron"]),
        body_sampler::plugin,
        body_part::plugin,
        plugin_body_tree_entity_map,
    ))
    .add_systems(
        Update,
        (
            //apply_body_damage,
            //sync_body_part_missing,
            //update_body_health_from_parts,
            //apply_pain_slowdown,
            build_body_tree,
        )
            .in_set(ModifierSystems),
    )
    .add_systems(Update, map_body_tree_entity_map_id_to_entity)
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            init_ezero_body_trees.in_set(BodySystems),
            map_body_tree_entity_map_id_to_entity.in_set(BodySystems),
        ),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            BodyPartSystems.before(BodySystems),
            BodySystems.before(BodySamplerSystems),
        ),
    )
    .register_type::<BodyTreeSerisHandles>()
    .register_type::<BodyTreeSeri>()
    .register_type::<BodyHealth>()
    .register_type::<BodyDead>()
    .register_type::<BodyDamage>()
    .replicate::<BodyTreesHolder>()
    .replicate::<BodyTree>()
    .replicate::<BodyHealth>()
    .replicate::<BodyDead>()
    .add_message::<BodyDamage>();
}
