
use bevy::{platform::collections::HashMap, prelude::*, sprite};
use bevy_asset_loader::prelude::*;

use crate::{common::common_components::DisplayName, game::{being::{race::race_components::*, sprite::sprite_resources::SpriteDataIdEntityMap}, game_components::Description}};

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default )]
pub struct RaceIdEntityMap {
    map: HashMap<String, Entity>,
}

#[allow(unused_parens)]
impl RaceIdEntityMap {
    pub fn new_race_from_seri(
        &mut self, cmd: &mut Commands,
        handle: Handle<RaceSeri>,
        assets: &mut Assets<RaceSeri>,
        sprites_map: &SpriteDataIdEntityMap,
    ) {
        
        if let Some(mut race_seri) = assets.remove(handle.id()){
            use std::mem::take;

            let enti_name = Name::new(format!("race id:'{}' '{}'", race_seri.id.clone(), race_seri.name.clone()));
            let race_id = RaceId::new(take(&mut race_seri.id));
            let ingame_name = DisplayName::new(take(&mut race_seri.name));
            let description = Description(take(&mut race_seri.description));
            let demonym = Demonym(take(&mut race_seri.demonym));
            let plural = PluralDenomination(
                race_seri
                    .plural.take()
                    .unwrap_or_else(|| format!("{}s", race_seri.singular.clone()))
            );
            let singular = SingularDenomination(take(&mut race_seri.singular));
            let sprites_pool = SpritesPool(sprites_map.get_entities(&race_seri.sprite_pool));
            let selectable_sprites = SelectableSprites(sprites_map.get_entities(&race_seri.selectable_sprites));
            
            let entity = cmd.spawn((
                race_id, 
                enti_name,
                ingame_name,
                description,
                demonym,
                plural,
                singular,
                sprites_pool,
                selectable_sprites,
            )).id();

            if ! race_seri.sexes.is_empty() {
                cmd.entity(entity).insert(Sexes::new(take(&mut race_seri.sexes)));
            }

            self.map.insert(race_seri.id.clone(), entity);
        } else {
            warn!("RaceSeri with handle {:?} not found in assets", handle);
        }
    }
    
    pub fn get_entity<S: Into<String>>(&self, race_id: S) -> Option<Entity> {
        self.map.get(&race_id.into()).copied()
    }
}

#[derive(AssetCollection, Resource)]
pub struct RaceSerisHandles {
    #[asset(path = "race", collection(typed))]
    pub handles: Vec<Handle<RaceSeri>>,
}


