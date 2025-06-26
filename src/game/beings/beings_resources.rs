use bevy::{platform::collections::HashMap, prelude::*};

use crate::game::{beings::beings_components::*};



// NO OLVIDARSE DE INICIALIZARLO EN EL Plugin DEL MÓDULO <--------
#[derive(Resource, )]
pub struct BeingEntityMap (
    pub HashMap<Being, Entity>,
    u32,//next_nid
);
impl Default for BeingEntityMap {
    fn default() -> Self {
        Self {
            0: HashMap::default(),
            1: 0, 
        }
    }
}

#[allow(unused_parens)]
impl BeingEntityMap {
    pub fn new_being<B: Bundle>(&mut self, commands: &mut Commands, bundle: B) -> Entity {
        let nid = self.1;
        self.1 += 1;
        let being = Being(nid);
        let entity = commands.spawn((Being(nid), bundle)).id();
        self.0.insert(being, entity);
        entity
    }

    pub fn get_entity(&self, being: Being) -> Option<Entity> {
        self.0.get(&being).copied()
    }
}