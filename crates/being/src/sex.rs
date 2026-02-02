use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use common::common_states::AssetLoading;

use crate::sex::{sex_init_systems::*, sex_resources::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SexSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((RonAssetPlugin::<SexSerialization>::new(&["sex.ron"])))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities), 
            (
                (init_sexes, map_sex_id_to_entity).chain()
            ).in_set(SexSystems)
        )
        .add_systems(Update, (
            map_sex_id_to_entity,
        ))
        .init_resource::<SexEntityMap>()
        .register_type::<SexSerialization>();
}

mod sex_init_systems;
pub mod sex_components;
pub mod sex_resources;
