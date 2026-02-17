use bevy::prelude::*;
use serde::Deserialize;

use crate::terrain::operation_list::operation_list_components::OperationList;

common::define_entity_map_systems!(
    OperationList,
    (),
    OperationList,
    "operation_list",
    "",
    OperationList,
    common::common_components::StrId
);

#[derive(Resource, Default)]
pub struct OpListSerisHandles {
    pub handles: Vec<Handle<OpListSeri>>,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct OpListSeri {
    pub id: String,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub debug_vars: Vec<String>,
    pub root_in_dimensions: Vec<String>,
    /// oplist id, produced tiles
    pub bifs: Vec<(String, Vec<String>)>,
    pub size: Option<[u32; 2]>,
    /// Expression tree representation (slot-free system)
    pub expr_tree: crate::terrain::terrgen_expression::ExprOpList,
}
impl OpListSeri {
    pub fn is_root(&self) -> bool {
        self.root_in_dimensions.iter().any(|s| !s.is_empty())
    }

    pub fn is_expr_based(&self) -> bool {
        true
    }
}
