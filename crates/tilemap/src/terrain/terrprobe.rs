pub mod terrprobe_components;
pub mod terrprobe_resources;
pub mod terrprobe_seris;
pub mod terrprobe_tpt_parser;
pub mod terrprobe_init_systems;
pub mod terrprobe_messages;
pub mod terrprobe_macros;
pub mod terrprobe_systems;
pub mod terrprobe_pattern_concentric;
pub mod terrprobe_pattern_chunk;
pub mod terrprobe_pattern_region;
pub mod opfilter;

use bevy_replicon::shared::replication::rules::AppRuleExt;
pub use terrprobe_components::*;
pub use terrprobe_resources::*;
#[allow(unused_imports)] pub use terrprobe_seris::*;
pub use terrprobe_messages::*;
pub use terrprobe_init_systems::init_terrain_probes;
#[allow(unused_imports)] pub use terrprobe_macros::*;
pub use terrprobe_systems::{search_suitable_positions, SearchParams, };
pub use terrprobe_pattern_concentric::*;
pub use terrprobe_pattern_chunk::*;
pub use terrprobe_pattern_region::*;
pub use opfilter::*;

use bevy::prelude::*;
use common::common_states::AssetLoading;
use crate::terrain::TerrainGenSystems;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TerrainProbeSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((plugin_terr_probe_templ,))
        .add_systems(Update, (
            search_suitable_positions,
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
        .replicate_filtered::<ChildOf, With<TerrProbeTempl>>()

    ;
}
