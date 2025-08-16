use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::common_components::StrId;


#[derive(Resource, Default)]
pub struct ImageSizeMap(pub HashMap<Handle<Image>, U16Vec2>);

#[derive(Resource, Debug, Default )]
pub struct GlobalEntityMap(pub HashMap<String, Entity>);
impl GlobalEntityMap {
    pub fn insert<S: Into<String>>(&mut self, id: S, entity: Entity) {
        self.0.insert(id.into(), entity);
    }
    
    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }

    #[allow(dead_code)]
    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

