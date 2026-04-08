#[allow(unused_imports)]
use bevy::prelude::*;
use game_common::game_common_components::Templ;

use crate::body::body_components::Body;
pub use crate::body::body_seris::*;

common::define_entity_map_systems!(
    main_component: Body,
    with_filters: With<Templ>,
    abbreviation: Body,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Body,
    id_type: common::common_components::StrId,
    assets: [(BodySeri, "seri.being.body", "body.ron")],
);
