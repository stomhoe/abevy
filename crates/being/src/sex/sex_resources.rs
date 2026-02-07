use bevy::prelude::*;

use crate::sex::sex_components::Sex;

#[derive(serde::Deserialize, Asset, Reflect, Default, Debug)]
pub struct SexSerialization {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

pub type SexSeri = SexSerialization;


common::define_entity_map_systems!(
    Sex,
    SexSeri, "ron/being/sex", "sex.ron",
);