use bevy::prelude::*;

use crate::sex::sex_components::Sex;
pub use crate::sex::sex_seris::*;


common::define_entity_map_systems!(
    main_component: Sex,
    with_filters: (),
    abbreviation: Sex,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Sex,
    id_type: common::common_components::StrId,
    assets: [(SexSeri, "seri.being.sex", "sex.ron")],
);
