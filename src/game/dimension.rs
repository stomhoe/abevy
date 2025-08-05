use bevy_common_assets::ron::RonAssetPlugin;
use bevy_replicon::prelude::AppRuleExt;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::dimension::{
    dimension_systems::*,
    dimension_components::*,
    dimension_resources::*,
//    dimension_constants::*,
//    dimension_events::*,
//    dimension_layout::*,
};
mod dimension_systems;
pub mod dimension_components;
pub mod dimension_resources;
//mod dimension_constants;
//mod dimension_events;
//mod dimension_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DimensionSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,Dimension)) del módul !!
pub struct DimensionPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for DimensionPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(DimensionSystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            .add_plugins((
            // SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
                //RonAssetPlugin::<DimensionSeri>::new(&["dimension.ron"]),
            ))
            .replicate::<Dimension>()
        ;
    }
}