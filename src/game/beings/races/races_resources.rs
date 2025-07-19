use bevy::{platform::collections::HashMap, prelude::*};

use crate::{common::common_components::{DisplayName, Sid}, game::beings::races::races_components::{Race, RaceDto}};

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default )]
pub struct IdRaceEntityMap {
    map: HashMap<String, Entity>,
}


#[allow(unused_parens)]
impl IdRaceEntityMap {
    // pub fn new_race<S: Into<String>, B: Bundle, T: Into<String>>(&mut self, cmd: &mut Commands, 
    //     race_id: S, 
    //     name: T, 
    //     bundle: B, ) -> Entity {
    //     let entity = cmd.spawn((Race::new(race_id.into()), bundle, DisplayName(name.into()))).id();
    //     self.map.insert(race_id.into(), entity);
    //     entity
    // }

    pub fn new_race_from_dto(
        &mut self,
        cmd: &mut Commands,
        race_dto_handle: Handle<RaceDto>,
        race_dto_assets: &Assets<RaceDto>,
    ) -> Option<Entity> {
        if let Some(race_dto) = race_dto_assets.get(&race_dto_handle) {
            let race = Race::new(race_dto.id.clone());
            let name = DisplayName(race_dto.name.clone());
            let entity = cmd.spawn((race, name)).id();
            self.map.insert(race_dto.id.clone(), entity);
            Some(entity)
        } else {
            None
        }
    }
    
    pub fn get_entity<S: Into<String>>(&self, race_id: S) -> Option<Entity> {
        self.map.get(&race_id.into()).copied()
    }
}

