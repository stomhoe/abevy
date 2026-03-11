use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Deserialize, TypePath, Clone, Default, Debug)]
pub struct InteractionZoneSeri {
    #[serde(default)]
    pub offset_positions: Vec<(i8, i8)>,
    #[serde(default)]
    pub radius_offset: Vec<(f32, (f32, f32))>,
}
