use bevy::prelude::*;
use strum_macros::EnumCount;

use crate::{game::{being::{race::{race_components::RaceSeri, race_resources::*, race_systems::*}, sprite::SpriteSystemsSet}, LocalAssetsLoadingState, GameDataInitSystems}, AppState};
use bevy_common_assets::ron::RonAssetPlugin;


// Module race
pub mod race_components;
pub mod race_resources;
mod race_systems;
pub mod race_constants;
pub mod race_utils;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RaceSystemsSet;

pub struct RacePlugin;
#[allow(unused_parens)]
impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((RonAssetPlugin::<RaceSeri>::new(&["race.ron"])))
            //.add_systems(OnEnter(AppState::StatefulGameSession), ().in_set(GameDataInitSystems).in_set(RaceSystemsSet))
            .add_systems(OnEnter(LocalAssetsLoadingState::Complete), init_races.in_set(RaceSystemsSet))
            .init_resource::<RaceIdEntityMap>()

            .configure_sets(OnEnter(LocalAssetsLoadingState::Complete), RaceSystemsSet.after(SpriteSystemsSet))
            .configure_sets(Update, RaceSystemsSet.after(SpriteSystemsSet))

            
                
        
        ;
    }
} 
