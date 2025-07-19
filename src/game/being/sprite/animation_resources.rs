use bevy::platform::collections::HashMap;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::game_components::DisplayName, game::being::sprite::sprite_components::{SpriteDataId, SpriteDataSeri}};




#[derive(Resource, Debug, Default )]
pub struct IdSpriteDataEntityMap {map: HashMap<String, Entity>,}


#[allow(unused_parens)]
impl IdSpriteDataEntityMap {
    pub fn new_spritedata_from_seri(
        &mut self,
        cmd: &mut Commands,
        sprite_data_seri_handle: Handle<SpriteDataSeri>,
        sprite_data_seri_assets: &Assets<SpriteDataSeri>,
    ) -> Option<Entity> {
        if let Some(sprite_data_seri) = sprite_data_seri_assets.get(&sprite_data_seri_handle) {
            let spritedata = SpriteDataId::new(sprite_data_seri.id.clone());
            let name = DisplayName(sprite_data_seri.name.clone());
            let entity = cmd.spawn((spritedata, name)).id();
            self.map.insert(sprite_data_seri.id.clone(), entity);
            Some(entity)
        } else {
            None
        }
    }
    
    pub fn get_entity<S: Into<String>>(&self, spritedata_id: S) -> Option<Entity> {
        self.map.get(&spritedata_id.into()).copied()
    }

    pub fn get_entities(&self, spritedatas: &Vec<String>) -> Vec<Entity> {
        let mut entities = Vec::new();
        for spritedata in spritedatas {
            if let Some(entity) = self.get_entity(spritedata) {
                entities.push(entity);
            }
        }
        entities
    }
}



