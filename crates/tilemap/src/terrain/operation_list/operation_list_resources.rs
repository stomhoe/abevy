use bevy::prelude::*;
use serde::Deserialize;
pub use crate::terrain::operation_list::operation_list_seris::*;

use crate::terrain::operation_list::operation_list_components::OperationList;

common::define_entity_map_systems!(
    OperationList,
    (),
    OperationList,
    "operation_list",
    "",
    OperationList,
    common::common_components::StrId,
    OpListSeri, "seri.tilemap.operation_list", "oplist.ron",
);

#[derive(Resource, Default, Clone)]
pub struct TgCompiledOpLists(pub Vec<OpListSeri>);
impl OpListSeri {
    pub fn is_root(&self) -> bool {
        self.root_in_dimensions.iter().any(|s| !s.is_empty())
    }

    pub fn is_expr_based(&self) -> bool {
        true
    }
}
