use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;

use crate::game::faction::faction_components::*;



#[derive(Resource, )]
pub struct FactionEntityMap {
    map: HashMap<Faction, Entity>,
    nid: u32, // next_nid
}
impl Default for FactionEntityMap {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            nid: 0, 
       }
    }
}

#[allow(unused_parens)]
impl FactionEntityMap {
    pub fn new_faction<B: Bundle>(&mut self, commands: &mut Commands, bundle: B) -> Entity {
        let faction = Faction::new(self.nid);
        let entity = commands.spawn((Faction::new(self.nid), bundle)).id();
        self.map.insert(faction, entity);
        self.nid += 1;
        entity
    }

    pub fn get_entity(&self, faction: Faction) -> Option<Entity> {
        self.map.get(&faction).copied()
    }
}