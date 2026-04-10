
#[allow(unused_imports)] use bevy::prelude::*;
use common::common_tag_components::AddSameHashedTags;

use {common::common_components::*, };
use serde::{Deserialize, Serialize};
use crate::chunking::macro_chunk_components::BiomeTagWeightAtMacrochunk;


#[derive(Debug, Deserialize, Serialize, Clone, Reflect)]
pub struct Bifurcation{
    pub oplist: Option<HashId>,
    #[reflect(ignore)]
    pub tiles: Vec<HashId>,
    #[reflect(ignore)]
    #[serde(skip)]
    pub biome_tags: Vec<BiomeTagWeightAtMacrochunk>,
}
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[require(Prefix::trunc("OpList"), AssetScoped, SelectedForHotReload, AddSameHashedTags, AddHashIdFromStrId)]
pub struct OperationList {
    #[reflect(ignore)]
    pub expr_tree: crate::terrain::terrgen_expression::ExprOpList,
    /// Variable names to keep in runtime debug capture for this oplist.
    #[reflect(ignore)]
    pub hash_ids_mapped_to_strids: HashIdMap<StrId>,
    pub bifurcations: Vec<Bifurcation>,
}

impl Default for OperationList {
    fn default() -> Self {
        Self {
            expr_tree: crate::terrain::terrgen_expression::ExprOpList {
                assignments: Vec::new(),
                output: crate::terrain::terrgen_expression::Expr::Literal(0.0),
            },
            hash_ids_mapped_to_strids: HashIdMap::default(),
            bifurcations: Vec::new(),
        }
    }
}
