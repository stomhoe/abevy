use bevy::platform::collections::HashMap;
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
