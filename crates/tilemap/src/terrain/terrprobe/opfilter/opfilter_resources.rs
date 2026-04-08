use bevy::{prelude::*};

pub use crate::terrain::terrprobe::opfilter::opfilter_seris::*;

use crate::terrain::terrprobe::opfilter::opfilter_components::OpFilter;

common::define_entity_map_systems!(
    main_component: OpFilter,
    with_filters: (Without<crate::terrain::terrprobe::terrprobe_components::TerrProbeTempl>, With<game_common::Templ>),
    abbreviation: OpFilter,
    target: "opfilter",
    entity_prefix: "opfilter",
    despawn_trigger: OpFilter,
    id_type: common::common_components::StrId,
    assets: [(OpFilterSeri, "seri.tilemap.of", "of.ron")],
);
