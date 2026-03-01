pub mod opfilter_components;
pub mod opfilter_init_systems;
pub mod opfilter_resources;
pub mod opfilter_seris;

use bevy::prelude::*;
use common::common_states::AssetLoading;
#[allow(unused_imports, )]
use crate::terrain::{TerrainGenSystems, terrprobe::opfilter::{
    opfilter_components::*, opfilter_init_systems::*, opfilter_resources::*
}};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct OpfilterSystems;

#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_op_filter
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        (init_opfilters, map_op_filter_id_to_entity,)
    ).chain().in_set(OpfilterSystems))

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities),
        OpfilterSystems.before(TerrainGenSystems)
    )
    ;
}
