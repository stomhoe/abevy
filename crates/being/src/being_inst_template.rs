use ::being_shared::*;
use bevy::{prelude::*, };
use common::{common_states::AssetLoading, };

use crate::{being_inst_template::{ being_inst_template_init_systems::* } };


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
        convert_bit_strid_ref_to_ent_ref,
    ))

    ;
}

mod being_inst_template_init_systems;
