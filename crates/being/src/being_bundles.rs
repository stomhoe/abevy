#[allow(unused_imports)]
use bevy::{
    ecs::entity::{EntityHashSet, MapEntities},
    platform::collections::HashMap,
    prelude::*,
};

use movement::{movement_components::GridLockedMovement, prelude::MovementRemoveOnFreezeBundle};
use tilemap::prelude::*;
use tilemap_shared::{ChunkPos, DimensionRef, GlobalTilePos, };

use crate::{being_components::*, nav::RetainedChasePathSnapshot};



#[derive(Bundle, Debug)]
pub struct BeingBundle(
    pub Being,
    pub DimensionRef,
    pub Transform,
    pub tilemap_shared::GlobalTilePos,
    pub GridLockedMovement,
);
impl BeingBundle {
    pub fn new(dimension_ref: DimensionRef, tile_pos: GlobalTilePos) -> Self {
        Self(
            Being::default(),
            dimension_ref,
            Transform::from_translation(tile_pos.to_translation(0.0)),
            tile_pos,
            GridLockedMovement {
                visual_origin_tile: tile_pos.0,
                ..default()
            },
        )
    }
}

#[derive(Bundle, Debug, )]
pub struct RemoveOnFreeze(
    pub Name,
    pub ActivateChunksAround,
    pub ActivatingChunks,
    pub RetainedChasePathSnapshot,
    pub Transform,
    pub GlobalTransform,
    pub DimensionRef,
    pub ChunkPos,
    pub Visibility,
    pub MovementRemoveOnFreezeBundle,

);//en vez de esto, hacer un estado serializado?
