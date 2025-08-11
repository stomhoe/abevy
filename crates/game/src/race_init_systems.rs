#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use crate::{common::common_components::{DisplayName, EntityPrefix, StrId}, game::{being::{race::{
   race_components::*, race_resources::*, 
   //race_events::*,
}, sprite::sprite_resources::SpriteCfgEntityMap}, game_components::Description}};


pub fn init_races(
    mut cmd: Commands,
    mut seris_handles: ResMut<RaceSerisHandles>,
    mut assets: ResMut<Assets<RaceSeri>>,
    sprite_map: Res<SpriteCfgEntityMap>,
) -> Result {
    let mut result: Result = Ok(());
    use std::mem::take;
    for handle in take(&mut seris_handles.handles) {
         if let Some(mut race_seri) = assets.remove(handle.id()){

            let str_id = match StrId::new(race_seri.id) {
                Ok(id) => id,
                Err(e) => {
                    let e: BevyError = BevyError::from(format!("Failed to create StrId for race {:?}: {}", race_seri.name, e));
                    error!("{}", e);
                    result = Err(e.into());
                    continue;
                }
            };

            let ingame_name = DisplayName(race_seri.name);
            let description = Description(race_seri.description);
            let demonym = Demonym(race_seri.demonym.into());
            let plural = PluralDenomination(race_seri.plural.unwrap_or(format!("{}s", race_seri.singular)).into());
            let singular = SingularDenomination(race_seri.singular.into());
            let sprites_pool = match sprite_map.0.get_multiple(&race_seri.sprite_pool) {
                Ok(pool) => SpritesPool(pool),
                Err(e) => return Err(e.into()),
            };
            let selectable_sprites = match sprite_map.0.get_multiple(&race_seri.selectable_sprites) {
                Ok(sprites) => SelectableSprites(sprites),
                Err(e) => return Err(e.into()),
            };

            let entity = cmd.spawn((
                str_id, 
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
        } 
    }
    result
}

pub fn add_races_to_map(
    mut race_map: ResMut<RaceEntityMap>,
    query: Query<(Entity, &StrId, &EntityPrefix), Added<Race>>,
) -> Result{
    let mut result: Result = Ok(());
    for (entity, str_id, entity_prefix) in query.iter() {
        match race_map.0.insert(str_id.clone(), entity, entity_prefix) {
            Ok(()) => (),
            Err(e) => {
                let err: BevyError = BevyError::from(format!("Failed to insert race into map: {}", e));
                result = Err(err.into());
            }
        }
    }
    result
}

