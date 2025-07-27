use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::{common::common_components::DisplayName, game::faction::faction_components::*};



#[derive(Resource, )]
pub struct FactionEntityMap {
    map: HashMap<String, Entity>,
}
impl Default for FactionEntityMap {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
       }
    }
}

#[allow(unused_parens)]
impl FactionEntityMap {
    pub fn insert_faction<S: Into<String>, B: Bundle>(&mut self, cmd: &mut Commands, faction: Faction, name: S, bundle: B) -> Entity {
        let faction_id = faction.id().clone();
        let entity = cmd.spawn((faction, DisplayName::new(name.into()))).id();
        cmd.entity(entity).insert(bundle);
        self.map.insert(faction_id, entity);
        entity
    }

    pub fn get_entity_from_fac(&self, faction: &Faction) -> Option<Entity> {
        self.map.get(faction.id()).copied()
    }
    pub fn get_entity_from_str<S: Into<String>>(&self, faction_id: S) -> Option<Entity> {
        self.map.get(&faction_id.into()).copied()
    }
}