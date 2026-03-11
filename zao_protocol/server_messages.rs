use bevy::ecs::entity::MapEntities;
use bevy::prelude::Entity;
use bevy_replicon::prelude::Channel;
use serde::{Deserialize, Serialize};


use super::types::{AoHeading, AoTilePos};

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize, MapEntities)]
pub struct AoSyncGridPosition {
    #[entities]
    pub entity: Entity,
    pub tile_pos: AoTilePos,
    pub input_seq: u32,
}

impl AoSyncGridPosition {
    pub const CHANNEL: Channel = Channel::Unordered;
}

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize, MapEntities)]
pub struct AoSyncGridStep {
    #[entities]
    pub entity: Entity,
    pub tile_pos: AoTilePos,
    pub visual_origin_tile: AoTilePos,
    pub dir: AoTilePos,
    pub input_seq: u32,
    pub step_ticks_total: u16,
}

impl AoSyncGridStep {
    pub const CHANNEL: Channel = Channel::Unreliable;
}

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize, MapEntities)]
pub struct AoSyncGridHeading {
    #[entities]
    pub entity: Entity,
    pub heading: AoHeading,
    pub input_seq: u32,
}

impl AoSyncGridHeading {
    pub const CHANNEL: Channel = Channel::Ordered;
}

#[derive(Debug, Clone, Copy, Message, Serialize, Deserialize, MapEntities)]
pub struct AoForceMove {
    #[entities]
    pub entity: Entity,
    pub dir: AoTilePos,
}

impl AoForceMove {
    pub const CHANNEL: Channel = Channel::Unordered;
}
