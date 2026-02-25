use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: DimensionRootOplist, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity, pub requester: Entity, pub max_emitted_results: u16, pub mark_last_success_in_batch: bool}
impl PendingOp {
    pub fn filtered_op_is_placeholder(&self) -> bool {
        self.filtered_op == Entity::PLACEHOLDER
    }
}
