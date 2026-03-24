use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use common::common_states::AssetLoading;

use crate::faction_inst_templ::{
    faction_inst_templ_build_systems::*,
    faction_inst_templ_init_systems::*,
    faction_inst_templ_resources::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct FactionInstTemplateSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_faction_inst_templ,))
        .init_resource::<FactionInstTemplatePool>()
        .add_systems(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            ((init_faction_inst_templates, map_faction_inst_templ_id_to_entity).chain())
                .in_set(FactionInstTemplateSystems),
        )
        .add_systems(
            Update,
            (
                convert_fit_strid_ref_to_ent_ref.run_if(on_timer(Duration::from_secs_f32(0.5))),
                spawn_faction_instance_from_template,
                track_spawned_faction_instances,
            ),
        )
        .add_observer(remove_faction_instance_from_pool_on_despawn);
}

mod faction_inst_templ_build_systems;
mod faction_inst_templ_init_systems;
pub mod faction_inst_templ_resources;
pub mod faction_inst_templ_seris;
