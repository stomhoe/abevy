use bevy::{platform::collections::HashMap, prelude::*};

use crate::game::{beings::races::races_components::Race, };

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default )]
pub struct RaceEntityMap {
    map: HashMap<Race, Entity>,
    nid: u32, // next_nid
}


#[allow(unused_parens)]
impl RaceEntityMap {
    pub fn new_race<B: Bundle>(&mut self, commands: &mut Commands, bundle: B) -> Entity {
        let race = Race::new(self.nid);
        let entity = commands.spawn((Race::new(self.nid), bundle)).id();
        self.map.insert(race, entity);
        self.nid += 1;
        entity
    }

    pub fn get_entity(&self, race: Race) -> Option<Entity> {
        self.map.get(&race).copied()
    }
}

