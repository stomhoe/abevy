use bevy::{
    prelude::*,
};
use faction_shared::Culture;
use crate::culture::culture_seris::*;

common::define_entity_map_systems!(
    main_component: Culture,
    with_filters: (),
    abbreviation: Culture,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Culture,
    id_type: common::common_components::StrId,
    assets: [(CultureSeri, "seri.faction.culture", "culture.ron")],
);
