use bevy::prelude::*;
use ::tilemap_shared::*;
use crate::terrain::{
    terrprobe::{terrprobe_components::TerrProbeTempl, terrprobe_messages::TerrProbeJob},
    terrgen_messages::PendingOp,
};

pub fn process_radial_pattern(
    pos_search: TerrProbeJob,
    templ: &TerrProbeTempl,
    root_oplist: DimensionRootOplist,
    curr_iteration_batch_i: i16,
    new_pending_ops: &mut Vec<PendingOp>,
    new_pos_searches: &mut Vec<TerrProbeJob>,
    search_failed: &mut Vec<Entity>,
) {
    crate::terrain::terrprobe::terrprobe_pattern_sun::process_sun_pattern(
        pos_search,
        templ,
        root_oplist,
        curr_iteration_batch_i,
        new_pending_ops,
        new_pos_searches,
        search_failed,
    );
}
