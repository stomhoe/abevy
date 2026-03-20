pub mod operation_list_components;
pub mod operation_list_init_systems;
pub mod operation_list_resources;
pub mod operation_list_seris;
pub mod operation_list_script;

use common::common_states::AssetLoading;
pub use operation_list_components::*;
pub use operation_list_resources::*;
#[allow(unused_imports, )] pub use operation_list_seris::*;
pub use operation_list_script::*;

use bevy::prelude::*;
use crate::terrain::operation_list_init_systems::*;


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct OperationListSystems;

#[allow(unused_parens, path_statements)]
pub fn plugin(app: &mut App) {
    app
        .add_plugins((
            plugin_operation_list,
        ))
        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            (

                cache_tg_oplists,
                init_oplists_from_assets,
                map_operation_list_id_to_entity,
                init_oplists_bifurcations,
                cycle_detection,
                assign_rootoplist_to_dimensions,

            ).chain(),
            ).in_set(OperationListSystems)
        )
    ;
}
