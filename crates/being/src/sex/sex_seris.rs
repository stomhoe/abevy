use bevy::prelude::*;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct SexSeri {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}
