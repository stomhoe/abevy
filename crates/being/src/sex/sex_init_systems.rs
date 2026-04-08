#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_string_components::*;

#[allow(unused_imports, )]
use crate::sex::{SexEntityMap, sex_components::*, sex_resources::*};

pub fn init_sexes(
    mut cmd: Commands,
) {
    for sex_seri in load_sex_seri_defs() {
            let str_id = StrId::trunc(&sex_seri.id);

            let ingame_name = DisplayName(sex_seri.name.clone());
            let description = sex_seri.description.as_ref().map(|d| Description(d.clone()));

            let mut entity_cmds = cmd.spawn((Sex, str_id.clone(), AddHashIdFromStrId, ingame_name));

            if let Some(desc) = description {
                entity_cmds.insert(desc);
            }

            trace!(target: "sex_init", "Initialized sex '{}' with entity {:?}", str_id, entity_cmds.id());
    }
}
