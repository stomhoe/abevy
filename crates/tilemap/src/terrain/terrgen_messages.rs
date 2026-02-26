use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy)]
pub struct PendingOpMatrixSpec {
    pub min: GlobalTilePos,
    pub matrix_size: UVec2,
    pub spacing: u16,
}

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: DimensionRootOplist, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity, pub requester: Entity, pub max_emitted_results: u32, pub mark_last_success_in_batch: bool, pub matrix_spec: Option<PendingOpMatrixSpec>}
impl PendingOp {
    pub fn filtered_op_is_placeholder(&self) -> bool {
        self.filtered_op == Entity::PLACEHOLDER
    }
}
