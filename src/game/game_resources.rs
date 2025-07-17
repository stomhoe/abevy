use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

pub const METER: f32 = 64.0; 
pub const KILOMETER: f32 = METER * 1000.0; 


use crate::game::game_components::Nid;


#[derive(Resource, )]
// CADA JUGADOR DEBE TENERLO RELLENO YA QUE CADA JUGADOR TIENE SU PROPIO ENTITY PARA CADA ENTIDAD
pub struct NidEntityMap {
    map: HashMap<Nid, Entity>,
    next_nid: u64, // next_nid
}
impl Default for NidEntityMap {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            next_nid: 1, 
       }
    }
}

#[allow(unused_parens)]
impl NidEntityMap {
    pub fn new_entity(&mut self, commands: &mut Commands, new_entity: Entity) -> Nid {
        let nid = Nid::new(self.next_nid);
        commands.entity(new_entity).insert(nid);
        self.map.insert(nid, new_entity);
        nid
    }

    pub fn get_entity(&self, being_nid: Nid) -> Option<Entity> {
        self.map.get(&being_nid).copied()
    }

    // ESTO LO USAN LOS CLIENTS PARA SINCRONIZARSE RESPECTO LA NID Q QUIERE EL SERVER
    pub fn insert(&mut self, being_nid: Nid, entity: Entity) {
        self.map.insert(being_nid, entity);
    }
}

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
