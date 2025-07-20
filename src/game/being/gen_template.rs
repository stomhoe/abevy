use bevy_common_assets::ron::RonAssetPlugin;

#[allow(unused_imports)] use {bevy::prelude::*, superstate::superstate_plugin};

use crate::game::being::gen_template::{
//    gen_template_systems::*,
    gen_template_components::*,
//    gen_template_resources::*,
//    gen_template_constants::*,
//    gen_template_events::*,
//    gen_template_layout::*,
};
//mod gen_template_systems;
mod gen_template_components;
//mod gen_template_resources;
//mod gen_template_constants;
//mod gen_template_events;
//mod gen_template_layout;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GenTemplateSystems;

//PLU ¡¡ NO OLVIDARSE DE METERLO EN .add_plugins((,BeingGenTemplates)) del módulo being !!
pub struct GenTemplatePlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for GenTemplatePlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, (somesystem, ).in_set(BeingGenTemplatesSystems).in_set(IngameSystems))
            //.add_systems(OnEnter(SomeStateType::Literal), (setup, ))
            //.init_resource::<RESOURCE_NAME>()
            .add_plugins((
                RonAssetPlugin::<GenTemplateSeri>::new(&["gentmpl.ron"])
              //SomePlugin, 
            // superstate_plugin::<SuperState, (Substate1, Substate2)>
            ))
        ;
    }
}