use bevy::prelude::*;
pub use crate::terrain::operation_list::operation_list_seris::*;

use crate::terrain::operation_list::operation_list_components::OperationList;

common::define_entity_map_systems!(
    main_component: OperationList,
    with_filters: (),
    abbreviation: OperationList,
    target: "",
    entity_prefix: "operation_list",
    despawn_trigger: OperationList,
    id_type: common::common_components::StrId,
    assets: [(OpListSeri, "seri.tilemap.operation_list", "oplist.ron")]
);

#[derive(Resource, Default, Clone)]
pub struct TgCompiledOpLists(pub Vec<OpListSeri>);
