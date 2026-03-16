use bevy::prelude::*;
use common::common_tag_components::TagSet;

#[derive(Component, Debug, Default, Clone)]
pub struct WhitelistedSpawnTileTags(pub TagSet);

#[derive(Component, Debug, Default, Clone)]
pub struct BlacklistedSpawnTileTags(pub TagSet);