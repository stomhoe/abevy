#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_string_components::*;

use crate::sex::{SexEntityMap, sex_components::*, sex_resources::*};

pub fn init_sexes(
    mut cmd: Commands,
    mut seris_handles: ResMut<SexSerisHandles>,
    mut assets: ResMut<Assets<SexSerialization>>,
) {
    use std::mem::take;
    for handle in take(&mut seris_handles.handles) {
        if let Some(sex_seri) = assets.remove(handle.id()) {
            let str_id = StrId::trunc(&sex_seri.id);

            let ingame_name = DisplayName(sex_seri.name.clone());
            let description = sex_seri.description.as_ref().map(|d| Description(d.clone()));

            let mut entity_cmds = cmd.spawn((Sex, str_id.clone(), ingame_name));
            
            if let Some(desc) = description {
                entity_cmds.insert(desc);
            }

            trace!(target: "sex_init", "Initialized sex '{}' with entity {:?}", str_id, entity_cmds.id());
        }
    }
}

pub fn map_sex_id_to_entity(
    mut cmd: Commands,
    map: Option<ResMut<SexEntityMap>>,
    query: Query<(Entity, Option<&Prefix>, &StrId), (Changed<StrId>, With<Sex>)>,
) {
    if let Some(mut map) = map {
        for (entity, prefix, str_id) in query.iter() {
            if let Err(prev_ent) = map.0.insert(str_id, entity) {
                if prev_ent.0 == entity {
                    continue;
                }
                error!(target: "sex_init", "{} '{}' already in SexEntityMap with entity {:?}, cannot insert entity {:?}", prefix.cloned().unwrap_or_default(), str_id, prev_ent, entity);
                cmd.entity(entity).try_despawn();
            } else {
                trace!(target: "sex_init", "Inserted sex '{}' into SexEntityMap with entity {:?}", str_id, entity);
            }
        }
    } else {
        error!(target: "sex_init", "SexEntityMap resource not found when trying to add sex to it.");
    }
}
