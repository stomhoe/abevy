use bevy::{platform::collections::HashMap, prelude::*};

use crate::game::beings::races::races_components::*;

#[derive(Resource, Default)]
pub struct RaceDatabase (
    pub HashMap<RaceNid, Entity>
);

