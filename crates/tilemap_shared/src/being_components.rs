use bevy::prelude::*;

// Re-export for backward compatibility
pub use common::common_tag_components::{WhitelistedTags, BlacklistedTags};

#[derive(Component, Debug, Default, Clone)]
pub struct WhitelistedSpawnTileTags(pub WhitelistedTags);
impl WhitelistedSpawnTileTags {
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(WhitelistedTags::new(tags))
    }

    pub fn as_ref(&self) -> WhitelistedSpawnTileTagsRef<'_> {
        WhitelistedSpawnTileTagsRef(&self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WhitelistedSpawnTileTagsRef<'a>(pub &'a WhitelistedTags);

#[derive(Component, Debug, Default, Clone)]
pub struct BlacklistedSpawnTileTags(pub BlacklistedTags);
impl BlacklistedSpawnTileTags {
    pub fn new<S: AsRef<str>>(tags: impl IntoIterator<Item = S>) -> Self {
        Self(BlacklistedTags::new(tags))
    }

    pub fn as_ref(&self) -> BlacklistedSpawnTileTagsRef<'_> {
        BlacklistedSpawnTileTagsRef(&self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlacklistedSpawnTileTagsRef<'a>(pub &'a BlacklistedTags);
