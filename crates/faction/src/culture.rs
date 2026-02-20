use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::AppRuleExt;
use common::common_states::AssetLoading;

use crate::{culture::{culture_init_systems::*, culture_resources::*}, faction_components::Culture};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CultureSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_culture,))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            ((init_cultures, map_culture_id_to_entity).chain()).in_set(CultureSystems),
        )
        .add_systems(
            Update,
            (
                convert_culture_strid_ref_to_ent_ref.run_if(on_timer(Duration::from_secs_f32(0.5))),
                resolve_culture_race_opinions,
            ),
        )
    .replicate::<Culture>()

    ;

}

mod culture_init_systems;
pub mod culture_resources;
