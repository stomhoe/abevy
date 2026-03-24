use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::HashMap,
    prelude::*,
};
use common::common_components::{Prefix, StrId, Tag};

#[derive(Component, Debug, Default, Clone)]
pub struct CultureBitWeightMap(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureRacesOpinionStrIds(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureRacesOpinion(pub EntityHashMap<f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct CultureTags(pub Vec<Tag>);

#[derive(Component, Debug, Clone)]
pub struct RacePrefix(pub Prefix);
impl Default for RacePrefix {
    fn default() -> Self {
        Self(Prefix::trunc("Race"))
    }
}
