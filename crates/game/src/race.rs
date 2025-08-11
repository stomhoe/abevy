use bevy::prelude::*;

use crate::game::{being::{race::{race_components::RaceSeri, race_resources::*, race_init_systems::*}, sprite::SpriteSystemsSet}, ReplicatedAssetsLoadingState};
use bevy_common_assets::ron::RonAssetPlugin;


mod race_init_systems;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystemsSet;

pub struct RacePlugin;
#[allow(unused_parens)]
impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RonAssetPlugin::<RaceSeri>::new(&["race.ron"])))
            //.add_systems(OnEnter(AppState::StatefulGameSession), ().in_set(GameDataInitSystems).in_set(RaceSystemsSet))
            .add_systems(
                OnEnter(ReplicatedAssetsLoadingState::Finished), 
                (
                    init_races.before(add_races_to_map),
                    add_races_to_map
                ).in_set(RaceSystemsSet)
            )
            .init_resource::<RaceEntityMap>()

            .configure_sets(OnEnter(ReplicatedAssetsLoadingState::Finished), RaceSystemsSet.after(SpriteSystemsSet))
            .configure_sets(Update, RaceSystemsSet.after(SpriteSystemsSet))

            
                
        
        ;
    }
} 
