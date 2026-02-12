use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct EntityZero;

#[derive(
    Component, Debug, Clone, Deserialize, Serialize, Reflect, Copy, PartialEq, Eq, Hash, MapEntities,
)]
pub struct EntityZeroRef(#[entities] pub Entity);
