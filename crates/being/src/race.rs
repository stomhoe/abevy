use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use common::common_states::AssetLoading;
use sprite::AcSpriteSystems;

use crate::race::{race_init_systems::*, race_resources::*};


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystems;
#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((RonAssetPlugin::<RaceSerialization>::new(&["race.ron"])))
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities), 
            (
                (init_races,
                map_race_id_to_entity).chain()
            ).in_set(RaceSystems)
        )
        .add_systems(Update, (
            map_race_id_to_entity,
        ))
        .init_resource::<RaceEntityMap>()

        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), RaceSystems.after(AcSpriteSystems))

        
            
    
    ;
}

mod race_init_systems;
pub mod race_components;
pub mod race_resources;