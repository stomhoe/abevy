use bevy::prelude::*;
use game_common::prelude::EntityZeroCloneDeny;
use crate::ItemSpritesConfig;

#[derive(Bundle)]
pub struct ToDenyOnItemClone(
    EntityZeroCloneDeny,
    Children,
    ItemSpritesConfig,
);
