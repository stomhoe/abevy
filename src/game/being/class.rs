#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

use crate::game::being::class::{
    class_components::*,
    class_systems::*,
    //classes::classes_events::*,
};

use crate::AppState;
use crate::game;
mod class_systems;
mod class_components;
mod class_resources;
//mod classes_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClassesSystems;

pub struct ClassPlugin;
#[allow(unused_parens)]
impl Plugin for ClassPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::StatefulGameSession), (init_classes).in_set(game::GameDataInitSystems))
            .replicate::<Class>()
        ;
    }
}