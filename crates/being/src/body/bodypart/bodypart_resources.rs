use bevy::prelude::*;
use game_common::game_common_components::Templ;
pub use crate::body::bodypart::bodypart_seris::*;
use ::being_shared::*;

common::define_entity_map_systems!(
    main_component: Bodypart,
    with_filters: (With<Templ>, Without<BodypartChildOfBodypart>, Without<BodypartChildrenBodyparts>),
    abbreviation: Bodypart,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Bodypart,
    id_type: common::common_components::StrId,
    assets: [(BodypartSeri, "seri.being.body.part", "bodypart.ron")],
);
