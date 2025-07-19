
use bevy::{platform::collections::HashMap, prelude::*, sprite};

use crate::game::{being::{race::race_components::*, sprite::animation_resources::IdSpriteDataEntityMap}, game_components::{Description, DisplayName}};

//CASO DE USO: RECIBIS UN PAQUETE ONLINE SOLO CON NID Y TENES Q VER A Q ENTITY SE REFIERE
#[derive(Resource, Debug, Default )]
pub struct IdRaceEntityMap {
    map: HashMap<String, Entity>,
}



#[allow(unused_parens)]
impl IdRaceEntityMap {
    pub fn new_race_from_seri(
        &mut self,
        cmd: &mut Commands,
        race_seri_handle: Handle<RaceSeri>,
        race_seri_assets: &mut Assets<RaceSeri>,
        sprites_map: &IdSpriteDataEntityMap,
    ) -> Entity {
        use std::mem::take;
        let race_seri = race_seri_assets.get_mut(&race_seri_handle).unwrap();

        let race_id = RaceId::new(take(&mut race_seri.id));
        let disp_name = DisplayName(take(&mut race_seri.name));
        let description = Description(take(&mut race_seri.description));
        let demonym = Demonym(take(&mut race_seri.demonym));
        let plural = PluralDenomination(
            race_seri
                .plural.take()
                .unwrap_or_else(|| format!("{}s", race_seri.singular.clone()))
        );
        let singular = SingularDenomination(take(&mut race_seri.singular));
        let sprites_pool = SpritesPool(sprites_map.get_entities(&race_seri.sprites_pool));
        let selectable_sprites = SelectableSprites(sprites_map.get_entities(&race_seri.selectable_sprites));
        

        let entity = cmd.spawn((
            race_id, 
            disp_name, description, demonym, singular, plural,
            sprites_pool, selectable_sprites
        )).id();

        if let Some(males_ratio) = race_seri.sexes {
            cmd.entity(entity).insert(MalesRatio(males_ratio));
        }

        self.map.insert(race_seri.id.clone(), entity);
        entity
        
    }
    
    pub fn get_entity<S: Into<String>>(&self, race_id: S) -> Option<Entity> {
        self.map.get(&race_id.into()).copied()
    }
}

