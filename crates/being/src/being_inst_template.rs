use ::being_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use common::{common_components::StrId, common_states::AssetLoading, define_entity_map_systems};

use crate::{being_inst_template::{being_inst_template_components::*, being_inst_template_init_systems::*, being_inst_template_build_systems::*, being_inst_template_resources::*}, };


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BeingInstTemplateSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<BitSerialization>::new(&["being_template.ron"]),
        plugin_bit_entity_map,
    ))
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities), 
        (
            (init_being_templates, map_bit_entity_map_id_to_entity).chain()
        ).in_set(BeingInstTemplateSystems)
    )
    

    .add_systems(Update, (
        build_being_from_being_inst_template_ref,
        convert_strid_to_ent,
    ))
    .register_type::<BitSerialization>()
    .register_type::<BitRef>()

    .replicate::<BeingInstTemplate>()
    .replicate::<BitRef>()
    .replicate::<BitStrIdRef>()
    .replicate::<BitHealthMultiplier>()

    ;
}       

mod being_inst_template_init_systems;
mod being_inst_template_build_systems;
pub mod being_inst_template_components;
pub mod being_inst_template_resources;