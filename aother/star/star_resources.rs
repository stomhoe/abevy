use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct Score {
    pub count: i32,
}