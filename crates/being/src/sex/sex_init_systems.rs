#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_string_components::*;

use crate::sex::{SexEntityMap, sex_components::*, sex_resources::*};

pub fn init_sexes(
    mut cmd: Commands,
    mut seris_handles: ResMut<SexSerisHandles>,
    mut assets: ResMut<Assets<SexSeri>>,
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
