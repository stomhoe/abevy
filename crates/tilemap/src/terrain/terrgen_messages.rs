use bevy::prelude::*;
use crate::terrain::terrprobe::opfilter::opfilter_components::OpFilter;
use ::tilemap_shared::*;

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: DimensionRootOplist, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity, pub opfilter_override: Option<OpFilter>, pub requester: Entity, pub max_emitted_results: u16}
impl PendingOp {
    pub fn filtered_op_is_placeholder(&self) -> bool {
        self.filtered_op == Entity::PLACEHOLDER
    }
}
