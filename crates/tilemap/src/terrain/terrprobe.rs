pub mod terrprobe_components;
pub mod terrprobe_init_systems;
pub mod terrprobe_resources;
pub mod terrprobe_messages;
pub mod terrprobe_macros;
pub mod terrprobe_systems;
pub mod terrprobe_pattern_concentric;
pub mod terrprobe_pattern_spiral;
pub mod terrprobe_pattern_chunk;
pub mod terrprobe_pattern_region;
pub mod opfilter;

use bevy::prelude::*;
use bevy_replicon::prelude::ClientState;
use common::common_states::AssetLoading;
use ::tilemap_shared::*;

use crate::terrain::{
    TerrainGenSystems, terrprobe::{
        opfilter::OpfilterSystems, terrprobe_components::*, terrprobe_init_systems::*, terrprobe_messages::*, terrprobe_resources::*, terrprobe_systems::*
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainProbeSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_terr_probe_templ,))
        .add_systems(Update, (
            search_suitable_positions.run_if(in_state(ClientState::Disconnected)),
        ))
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
        .add_message::<SampledValuesCollected>()
        .add_message::<SearchFailed>()

    ;
}
