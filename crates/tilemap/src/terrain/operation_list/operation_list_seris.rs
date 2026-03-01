use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Clone)]
pub struct OpListSeri {
    pub id: String,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub debug_vars: Vec<String>,
    pub root_in_dimensions: Vec<String>,
    pub bifs: Vec<OpListBifSeri>,
    pub size: Option<[u32; 2]>,
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
