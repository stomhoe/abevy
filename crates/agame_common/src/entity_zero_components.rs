use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct EntityZero;

#[derive(
    Component, Clone, Deserialize, Serialize, Reflect, Copy, PartialEq, Eq, Hash, MapEntities,
)]
pub struct EntityZeroRef(#[entities] pub Entity);

impl Display for EntityZeroRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ezro({:?})", self.0)
    }
}

impl Debug for EntityZeroRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(self, f)
    }
}
