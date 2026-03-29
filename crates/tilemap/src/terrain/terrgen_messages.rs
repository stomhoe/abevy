use bevy::prelude::*;
use ::tilemap_shared::*;

#[derive(Debug, Clone, Copy)]
pub struct PendingOpMatrixSpec {
    pub min: GlobalTilePos,
    pub matrix_size: UVec2,
    pub spacing: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct PendingOpInput {
    pub dimension_ref: DimensionRef,
    pub gpos: GlobalTilePos,
}

#[derive(Debug, Clone, Copy)]
pub struct PendingOpValueProbe {
    pub filtered_op: Entity,
    pub requester: Entity,
    pub max_emitted_results: u32,
    pub mark_last_success_in_batch: bool,
    pub matrix_spec: Option<PendingOpMatrixSpec>,
}

#[derive(Message, Debug, Clone)]
pub struct ChunkTerrainBuilt {
    pub chunk_ent: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct MacroChunkBiomeSampled {
    pub macro_chunk_ent: Entity,
}

#[derive(Debug, Clone, Copy)]
pub enum PendingOpPurpose {
    ChunkTerrainGen { chunk_ent: Entity },
    ValueProbe(PendingOpValueProbe),
    MacroChunkBiomeSampling { macro_chunk_ent: Entity },
}

#[derive(Message, Debug, Clone)]
pub struct PendingOp {
    pub oplist: DimensionRootOplist,
    pub input: PendingOpInput,
    pub purpose: PendingOpPurpose,
}

impl PendingOp {
    pub fn dimension_ref(&self) -> DimensionRef {
        self.input.dimension_ref
    }

    pub fn gpos(&self) -> GlobalTilePos {
        self.input.gpos
    }

    pub fn filtered_op(&self) -> Entity {
        match self.purpose {
            PendingOpPurpose::ValueProbe(probe) => probe.filtered_op,
            _ => Entity::PLACEHOLDER,
        }
    }

    pub fn requester(&self) -> Entity {
        match self.purpose {
            PendingOpPurpose::ValueProbe(probe) => probe.requester,
            _ => Entity::PLACEHOLDER,
        }
    }

    pub fn max_emitted_results(&self) -> u32 {
        match self.purpose {
            PendingOpPurpose::ValueProbe(probe) => probe.max_emitted_results,
            _ => 0,
        }
    }

    pub fn mark_last_success_in_batch(&self) -> bool {
        match self.purpose {
            PendingOpPurpose::ValueProbe(probe) => probe.mark_last_success_in_batch,
            _ => false,
        }
    }

    pub fn matrix_spec(&self) -> Option<PendingOpMatrixSpec> {
        match self.purpose {
            PendingOpPurpose::ValueProbe(probe) => probe.matrix_spec,
            _ => None,
        }
    }

    pub fn filtered_op_is_placeholder(&self) -> bool {
        self.filtered_op() == Entity::PLACEHOLDER
    }
}
