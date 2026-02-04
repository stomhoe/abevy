use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use common::common_states::AssetLoading;
use common::{define_entity_map_systems, entity_map_macros::*, common_components::StrId};

use crate::sex::sex_components::*;
use crate::sex::{sex_init_systems::*, sex_resources::*};

define_entity_map_systems!(
    SexEntityMap,
    StrId,
    Sex
);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SexSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            RonAssetPlugin::<SexSerialization>::new(&["sex.ron"]),
            plugin_sex_entity_map,
        ))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities), 
            (
                (init_sexes, map_sex_entity_map_id_to_entity).chain()
            ).in_set(SexSystems)
        )

        .register_type::<SexSerialization>();
}

mod sex_init_systems;
pub mod sex_components;
pub mod sex_resources;
