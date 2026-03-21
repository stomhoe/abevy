use bevy::prelude::*;
use game_common::EntityZeroCloneDeny;
use crate::ItemSpritesConfig;

#[derive(Bundle)]
pub struct ToDenyOnItemClone(
    EntityZeroCloneDeny,
    Children,
    ItemSpritesConfig,
);
