use bevy::prelude::*;
use faction_shared::Faction;


common::define_entity_map_systems!(
    main_component: Faction,
    with_filters: (),
    abbreviation: Faction,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Faction,
    id_type: common::common_components::StrId,
);
