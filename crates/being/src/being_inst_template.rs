use std::time::Duration;

use ::being_shared::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::*;
use common::{common_states::AssetLoading, };

use crate::{being_inst_template::{being_inst_template_components::*, being_inst_template_init_systems::*, being_inst_template_build_systems::*, being_inst_template_resources::*}, };


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BeingInstTemplateSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_being_inst_template,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            (init_being_templates, map_being_inst_template_id_to_entity).chain()
        ).in_set(BeingInstTemplateSystems)
    )


    .add_systems(Update, (
        build_being_from_being_inst_template_ref,
        convert_bit_strid_ref_to_ent_ref.run_if(on_timer(Duration::from_secs_f32(0.5))),
    ))

    ;
}

mod being_inst_template_init_systems;
mod being_inst_template_build_systems;
pub mod being_inst_template_components;
pub mod being_inst_template_resources;
