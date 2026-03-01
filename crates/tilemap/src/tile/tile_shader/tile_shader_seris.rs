#[allow(unused_imports)]
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Asset, TypePath, Default)]
pub struct ShaderTerrblSeri {
    pub id: String,
}
