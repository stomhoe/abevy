#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::{tilemap::formation_generation::{formation_generation_resources::*, formation_generation_systems::*}, GamePhase, IngameSystems};
//use crate::game::parent_module_name::formation_generation::formation_generation_systems::*;
//use crate::game::parent_module_name::formation_generation::formation_generation_components::*;
//use crate::game::parent_module_name::formation_generation::formation_generation_resources::*;
//use crate::game::parent_module_name::formation_generation::formation_generation_layout::*;
//use crate::game::parent_module_name::formation_generation::formation_generation_events::*;
mod formation_generation_systems;
pub mod formation_generation_components;
pub mod formation_generation_resources;
pub mod formation_generation_utils;
//mod formation_generation_events;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FormationGenerationSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,FormationGeneration)) del módulo parent_module_name !!
pub struct FormationGenerationPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for FormationGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(FormationGenerationSystems).in_set(IngameSystems))
            .add_systems(OnEnter(GamePhase::InGame), (setup, ))
            .init_resource::<WorldGenSettings>()
            .init_resource::<Textures>()
        ;
    }
}