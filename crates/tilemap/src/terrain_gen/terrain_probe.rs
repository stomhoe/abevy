pub mod terrain_probe_components;
pub mod terrain_probe_init_systems;
pub mod terrain_probe_resources;
pub mod terrain_probe_messages;

use bevy::prelude::*;
use common::common_states::AssetLoading;

use crate::terrain_gen::{
    TerrainGenSystems, opfilter::OpfilterSystems, terrain_probe::{
        terrain_probe_init_systems::*,
        terrain_probe_resources::*,
        terrain_probe_messages::*,
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainProbeSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_terrain_probe_template,))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            init_terrain_probes,
            map_terrain_probe_template_id_to_entity,
        ).chain().in_set(TerrainProbeSystems))
        .configure_sets(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            TerrainProbeSystems.after(OpfilterSystems).before(TerrainGenSystems),
        )
        .add_message::<TerrainProbe>()
        .add_message::<SuitablePosFound>()
        .add_message::<SearchFailed>()
    ;
}
