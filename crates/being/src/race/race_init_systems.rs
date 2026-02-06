use bevy::platform::collections::HashSet;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_string_components::*;
use sprite::sprite_resources::SpriteCfgEntityMap;

use sex::sex_resources::SexEntityMap;
use crate::{race::{race_components::*, race_resources::*}, sex, };

pub fn init_races(
    mut cmd: Commands,
    mut seris_handles: ResMut<RaceSerisHandles>,
    mut assets: ResMut<Assets<RaceSerialization>>,
    sprite_map: Res<SpriteCfgEntityMap>,
    sexes_map: Res<SexEntityMap>,
) {
    use std::mem::take;
    for handle in take(&mut seris_handles.handles) {
        if let Some(mut race_seri) = assets.remove(handle.id()) {
            let str_id = StrId::trunc(&race_seri.id);

            let ingame_name = DisplayName(race_seri.name.clone());
            let description = race_seri.description.as_ref().map(|d| Description(d.clone()));
            let demonym = race_seri.demonym.as_ref().map(|d| Demonym(d.clone().into()));
            
            let singular_str = race_seri.singular.unwrap_or_else(|| race_seri.name.clone());
            let plural_str = race_seri.plural
                .unwrap_or_else(|| format!("{}s", singular_str));
            let singular = SingularDenomination(singular_str.into());
            let plural = PluralDenomination(plural_str.into());

            let sprites_pool = {
                let mut pool = Vec::new();
                for sprite_id in &race_seri.sprite_pool {
                    match sprite_map.0.get_cloned(sprite_id) {
                        Ok(entity) => pool.push(entity),
                        Err(_) => warn!(target: "race_init", "Race '{}' sprite pool member '{}' not found", str_id, sprite_id),
                    }
                }
                if !pool.is_empty() {
                    Some(SpritesPool(pool))
                } else {
                    warn!(target: "race_init", "Race '{}' has empty sprite pool after lookup", str_id);
                    None
                }
            };

            let selectable_sprites = {
                if let Some(sprite_ids) = &race_seri.selectable_sprites {
                    let valid_ids: HashSet<_> = sprite_ids
                        .iter()
                        .map(|id| id.trim())
                        .filter(|id| !id.is_empty())
                        .collect();
                    
                    if valid_ids.is_empty() {
                        None
                    } else {
                        let mut sprites = Vec::new();
                        for sprite_id in valid_ids {
                            match sprite_map.0.get_cloned(sprite_id) {
                                Ok(entity) => sprites.push(entity),
                                Err(_) => warn!(target: "race_init", "Race '{}' selectable sprite '{}' not found", str_id, sprite_id),
                            }
                        }
                        if !sprites.is_empty() {
                            Some(PlayerSelectableSprites(sprites))
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            };

            let mut entity_cmds = cmd.spawn((Race, str_id.clone(), ingame_name, singular, plural));
            
            if let Some(desc) = description {
                entity_cmds.insert(desc);
            }
            if let Some(dem) = demonym {
                entity_cmds.insert(dem);
            }
            if let Some(pool) = sprites_pool {
                entity_cmds.insert(pool);
            }
            if let Some(selectable) = selectable_sprites {
                entity_cmds.insert(selectable);
            }

            let entity = entity_cmds.id();

            if !race_seri.sexes.is_empty() {
                let mut sex_entities_weights: Vec<(Entity, f32)> = Vec::new();
                for (sex_id, weight) in &race_seri.sexes {
                    match sexes_map.0.get_cloned(sex_id) {
                        Ok(sex_entity) => {
                            sex_entities_weights.push((sex_entity, *weight as f32));
                        }
                        Err(_) => {
                            warn!(target: "race_init", "Race '{}' sex '{}' not found in SexEntityMap", str_id, sex_id);
                        }
                    }
                }
                if !sex_entities_weights.is_empty() {
                    let sex_sampler = SexesSampler::new(&sex_entities_weights);
                    cmd.entity(entity).insert(sex_sampler);
                }
            }
            
            trace!(target: "race_init", "Initialized race '{}' with entity {:?}", str_id, entity);
        }
    }
}


