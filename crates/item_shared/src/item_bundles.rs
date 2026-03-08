use bevy::prelude::*;
use common::common_components::*;
use game_common::game_common_components::*;
use game_common::game_common_string_components::*;
use game_common::prelude::EntityZeroCloneDeny;
use crate::ItemSpritesConfig;

#[derive(Bundle)]
pub struct ToDenyOnItemClone(
    EntityZeroCloneDeny,
    Children,
    ItemSpritesConfig,
);
