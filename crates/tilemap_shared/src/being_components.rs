use bevy::prelude::*;
use common::{common_tag_components::TagSet, prelude::Tag};

// Re-export for backward compatibility
pub use common::common_tag_components::{WhitelistedTags, BlacklistedTags};

#[derive(Component, Debug, Default, Clone)]
pub struct WhitelistedSpawnTileTags(pub WhitelistedTags);
impl WhitelistedSpawnTileTags {
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(WhitelistedTags::new(tags))
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct BlacklistedSpawnTileTags(pub BlacklistedTags);
impl BlacklistedSpawnTileTags {
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(BlacklistedTags::new(tags))
    }
}