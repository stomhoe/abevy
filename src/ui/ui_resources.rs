
use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct InputOutputMap(HashMap<Entity, Entity>);