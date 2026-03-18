use bevy::{
    ecs::entity::EntityHashMap,
    platform::collections::HashMap,
    prelude::*,
};
use common::common_components::{Prefix, StrId, Tag};

use crate::faction_components::Culture;
pub use crate::culture::culture_seris::*;

common::define_entity_map_systems!(
    Culture,
    CultureSeri, "seri.faction.culture", "culture.ron",
);

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
