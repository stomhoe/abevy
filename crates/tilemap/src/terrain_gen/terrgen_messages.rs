use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Message, Debug, Clone)]
pub struct PendingOp {pub oplist: DimensionRootOplist, pub dimension_ref: DimensionRef, pub gpos: GlobalTilePos,
    pub filtered_op: Entity, pub max_emitted_results: u16}
