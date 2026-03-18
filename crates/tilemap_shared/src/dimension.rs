use std::{hash::{Hash, }};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
#[allow(unused_imports, )]
use bevy::platform::collections::{HashSet, HashMap};
use bevy::{ecs::entity::{EntityHashSet, MapEntities}, };
use common::common_tag_components::TagSet;

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, Hash, PartialEq, )]
#[require(SparedFromHotReloading, Replicated, AssetScoped, Prefix::trunc("DIMENSION"),  )]
pub struct Dimension;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, )]
pub struct Gravity(pub f32);
impl Default for Gravity {
    fn default() -> Self {
        Self(9.81)
    }
}
impl Gravity {
    pub fn mass_to_newtons(&self, mass_kg: f32) -> f32 {
        mass_kg.max(0.0) * self.0.max(0.0)
    }
}


common::define_entity_map_systems!(
    Dimension,
    DimensionSeri, "seri.dimension", "dimension.ron"
);
impl Dimension{
    pub fn overworld() -> StrId{
        StrId::trunc("ow")
    }
}

#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct DimensionSeri {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_dimension_gravity")]
    pub gravity: f32,
    /// this dimension's tags, used for whoever needs it
    #[serde(default)]
    pub tags: HashSet<String>,

    #[serde(default)]
    pub whitelisted_structure_gen_tags: Vec<String>,
    #[serde(default)]
    pub blacklisted_structure_gen_tags: Vec<String>,
}
fn default_dimension_gravity() -> f32 { 9.81 }

impl DimensionStrIdRef {

    pub fn overworld_fallback() -> Self {
        warn!("Using overworld fallback for DimensionStrIdRef");
        DimensionStrIdRef(Dimension::overworld())
    }
}



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct DimensionSystems;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, MapEntities)]
#[relationship(relationship_target = RootInDimensions)]
pub struct DimensionRootOplist(#[relationship]#[entities]pub Entity);

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = DimensionRootOplist)]
pub struct RootInDimensions(EntityHashSet);



#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
pub struct MultipleDimensionStringRefs(Vec<String>);

impl MultipleDimensionStringRefs {
    pub fn new(strings: Vec<String>) -> Self {
        let filtered = strings.into_iter().filter(|s| !s.is_empty()).collect();
        MultipleDimensionStringRefs(filtered)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }
}

#[derive(Component, Debug, Default, Serialize, Deserialize, MapEntities, Clone)]
pub struct MultipleDimensionRefs(#[entities] pub EntityHashSet,);

#[derive(Debug, Message)]
pub struct ReassignDimensionToEntity (pub Entity);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct WhitelistedStructureGenTags(pub TagSet);


#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct BlacklistedStructureGenTags(pub TagSet);
