#[allow(unused_imports)]
use bevy::{
    ecs::entity::{EntityHashSet, MapEntities},
    platform::collections::HashMap,
    prelude::*,
};

use ::movement::*;
use tilemap::chunking::chunking_components::*;
use tilemap_shared::{LoadChunksAround, ChunkPos, DimensionRef, GlobalTilePos, };
use ::being_shared::*;

use crate::{being_nav::RetainedChasePathSnapshot};



#[derive(Bundle, Debug)]
pub struct BeingBundle(
    pub Being,
    pub DimensionRef,
    pub ChildOf,
    pub Transform,
    pub tilemap_shared::GlobalTilePos,
    pub GridLockedMovement,
    pub GridLockedMovementVisual,
);
impl BeingBundle {
    pub fn new(dimension_ref: DimensionRef, tile_pos: GlobalTilePos) -> Self {
        Self(
            Being::default(),
            dimension_ref,
            ChildOf(dimension_ref.0),
            Transform::from_translation(tile_pos.to_translation(0.0)),
            tile_pos,
            GridLockedMovement {
                ..default()
            },
            GridLockedMovementVisual {
                visual_origin_tile: tile_pos.0,
                ..default()
            },
        )
    }
}


#[derive(Bundle, Debug, )]
pub struct RemoveOnEnterSemiRealSimMode(
    pub Name,
    pub LoadChunksAround,
    pub ActivatingChunks,
    pub RetainedChasePathSnapshot,
    pub StepDistanceSfxState,
    /*
        pub PendingTileCorrection,
        pub Visibility,
        pub sprite_animation_shared::MoveAnimActive,
        pub GridLockedMovementVisual,
     */
);

#[derive(Bundle, Debug, )]
pub struct RemoveOnEnterFakeSimMode(
    pub Transform,
    pub GlobalTransform,
    pub RemoveOnEnterSemiRealSimMode,
    pub DimensionRef,
    pub ChunkPos,
    pub Visibility,
    pub StepDistanceSfxState,
    pub GridLockedMovement,
    pub InputMoveDir,
    pub FinalNormMoveDir,
    pub tilemap_shared::SnapTransformToGpos,
);

#[derive(Bundle, Debug, )]
pub struct ReinsertOnUnfreeze(
    pub Name,
    pub Transform,
    pub GlobalTransform,
    pub DimensionRef,
    pub ChunkPos,
    pub Visibility,
    pub GridLockedMovement,
    pub GridLockedMovementVisual,
);
impl ReinsertOnUnfreeze {
    pub fn new(msg: tilemap_shared::ChunkLoaded) -> Self {
        Self(
            Name::default(),
            Transform::default(),
            GlobalTransform::default(),
            msg.dimension,
            msg.chunk_pos,
            Visibility::default(),
            GridLockedMovement::default(),
            GridLockedMovementVisual::default(),
        )
    }
}
