use bevy::platform::collections::HashMap;
#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct BodyWeightedSamplerSeri {
    pub id: String,
    pub weights: Vec<(String, f32)>,
    pub extra: Option<HashMap<String, String>>,
}
