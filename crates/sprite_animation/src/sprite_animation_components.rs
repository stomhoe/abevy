#[allow(unused_imports)] use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Clone, Debug, Deserialize, Serialize)]
pub struct ClipStartFrames(pub Vec<usize>);

/// Component to store alternating start frames configuration for each clip.
/// Each clip can have two alternating start frames that switch on each animation start.
#[derive(Component, Default, Clone, Debug, Deserialize, Serialize)]
pub struct AlternatingStartFramesConfig(pub Vec<Option<(usize, usize)>>);

/// Component to track the alternating start frame state for clips.
/// For each clip with alternating_start_frames, tracks which frame index (0 or 1) to use next.
#[derive(Component, Default, Clone, Debug, Deserialize, Serialize)]
pub struct AlternatingStartFramesState(pub Vec<usize>);

/// Component to control whether animation progress is saved and restored.
/// Some(true): save and restore animation progress between clips.
/// Some(false) or None: don't save animation progress.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct SaveAnimationProgress;
