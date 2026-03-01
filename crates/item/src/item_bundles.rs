use bevy::prelude::*;
use common::common_components::*;
use game_common::game_common_components::*;
use game_common::game_common_string_components::*;

#[derive(Bundle)]
pub struct ToDenyOnItemClone(
    EntityZero,
    AddHashIdFromStrId,
    HashId,
    GameCommonStringComponentsBundle,
    Children,
);
