use bevy::{math::U16Vec2, platform::collections::HashMap};
#[allow(unused_imports)] use bevy::prelude::*;

pub const METER: f32 = 64.0; 
pub const KILOMETER: f32 = METER * 1000.0; 


use crate::game::game_components::Nid;



#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ImageNid (pub u32);

/* No olvidarse de inicializarlo en el Plugin del módulo
 .init_resource::<ImageNidEntityMap>()
*/
#[derive(Resource, Debug, Default)]
pub struct NidImageMap {
    map: HashMap<ImageNid, Handle<Image>>,
}
#[allow(unused_parens)]
impl NidImageMap {
    pub fn insert(&mut self, nid: ImageNid, image: Handle<Image>) {
        self.map.insert(nid, image);
    }

    pub fn get_image(&self, nid: ImageNid) -> Option<&Handle<Image>> {
        self.map.get(&nid)
    }
}


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