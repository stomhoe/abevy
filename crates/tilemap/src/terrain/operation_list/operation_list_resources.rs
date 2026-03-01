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
    common::common_components::StrId,
    OpListSeri, "seri.tilemap.operation_list", "oplist.ron",
);

#[derive(Deserialize, Asset, TypePath, Clone)]
pub struct OpListSeri {
    pub id: String,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub debug_vars: Vec<String>,
    pub root_in_dimensions: Vec<String>,
    pub bifs: Vec<OpListBifSeri>,
    pub size: Option<[u32; 2]>,
    /// Expression tree representation (slot-free system)
    pub expr_tree: crate::terrain::terrgen_expression::ExprOpList,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OpListBifSeri {
    pub oplist: String,
    pub tiles: Vec<String>,
    #[serde(default)]
    pub biome_tags: Vec<OpListBifBiomeTagSeri>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OpListBifBiomeTagSeri {
    pub tag: String,
    #[serde(default = "default_biome_tag_weight")]
    pub weight: f32,
}

fn default_biome_tag_weight() -> f32 { 1.0 }

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
