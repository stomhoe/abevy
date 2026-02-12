use bevy::prelude::*;

use crate::sex::sex_components::Sex;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct SexSeri {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}



common::define_entity_map_systems!(
    Sex,
    SexSeri, "ron/being/sex", "sex.ron",
);
