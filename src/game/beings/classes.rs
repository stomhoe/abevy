#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::beings::classes::{
    classes_components::*,
    classes_resources::*,
    classes_systems::*,
    //classes::classes_events::*,
};

use crate::AppState;
use crate::game;
mod classes_systems;
mod classes_components;
mod classes_resources;
//mod classes_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClassesSystems;

pub struct ClassesPlugin;
#[allow(unused_parens)]
impl Plugin for ClassesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::StatefulGameSession), (init_classes).in_set(game::GameDataInitSystems))
            .replicate::<Class>()
        ;
    }
}