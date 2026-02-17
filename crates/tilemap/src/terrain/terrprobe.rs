pub mod terrprobe_components;
pub mod terrprobe_init_systems;
pub mod terrprobe_resources;
pub mod terrprobe_messages;

use bevy::prelude::*;
use common::common_states::AssetLoading;

use crate::terrain::{
    TerrainGenSystems, opfilter::OpfilterSystems, terrprobe::{
        terrprobe_init_systems::*,
        terrprobe_resources::*,
        terrprobe_messages::*,
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainProbeSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_terr_probe_templ,))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            init_terrain_probes,
            map_terr_probe_templ_id_to_entity,
        ).chain().in_set(TerrainProbeSystems))
        .configure_sets(
            OnEnter(AssetLoading::SpawnReplicatedEntities),
            TerrainProbeSystems.after(OpfilterSystems).before(TerrainGenSystems),
        )
        .add_message::<TerrProbeJob>()
        .add_message::<SuitablePosFound>()
        .add_message::<SearchFailed>()
    ;
}
