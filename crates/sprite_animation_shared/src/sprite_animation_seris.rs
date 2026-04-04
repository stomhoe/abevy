use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Deserialize, Asset, TypePath, Default, Clone)]
pub struct MultipleAnimationSeri(pub Vec<AnimationSeri>);

#[derive(Component, Debug, Deserialize, Serialize, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CardinalRotation {
    #[default]
    None,
    West,
    North,
    East,
}
impl CardinalRotation {
    pub fn angle(&self) -> Option<f32> {
        match self {
            CardinalRotation::None => None,
            CardinalRotation::West => Some(std::f32::consts::FRAC_PI_2),
            CardinalRotation::North => Some(std::f32::consts::PI),
            CardinalRotation::East => Some(-std::f32::consts::FRAC_PI_2),
        }
    }
}

#[derive(Component, Deserialize, Serialize, Asset, TypePath, Clone)]
#[serde(default)]
pub struct AnimationSeri {
    pub id: String,
    pub img_path: String,
    pub clips: Vec<ClipConfig>,
    pub anim_format_id: String,
    pub rows_cols: (usize, usize),
    pub save_animation_progress: bool,
    pub alternating_start_frames: (usize, usize),
    pub dir: u8,
    pub reps: u32,
    pub dur_frame: u32,
    pub dur_rep: u32,
    pub offset: [f32; 2],
    pub scale: [f32; 2],
    pub y_sort: f32,
    pub z: f32,
    pub paused: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub cardinal_rotation: CardinalRotation,
    pub speed: f32,
    pub sound_effects: Vec<String>,
    pub sound_effects_every_n_frames: f32,
    #[serde(default)]
    pub color: Option<[u8; 4]>,
}

impl Default for AnimationSeri {
    fn default() -> Self {
        Self {
            id: String::default(),
            img_path: String::default(),
            clips: Vec::default(),
            anim_format_id: String::default(),
            rows_cols: (1, 1),
            save_animation_progress: false,
            alternating_start_frames: Self::UNSET_ALTERNATING_START_FRAMES,
            dir: Self::DIR_UNSET,
            reps: Self::REPS_UNSET,
            dur_frame: Self::DUR_FRAME_UNSET,
            dur_rep: Self::DUR_REP_UNSET,
            offset: [0.0, 0.0],
            scale: [1.0, 1.0],
            y_sort: Self::Y_SORT_UNSET,
            z: f32::NAN,
            paused: false,
            flip_x: false,
            flip_y: false,
            cardinal_rotation: CardinalRotation::default(),
            speed: 1.0,
            sound_effects: Vec::default(),
            sound_effects_every_n_frames: 1.0,
            color: None,
        }
    }
}

impl AnimationSeri {
    pub const DIR_UNSET: u8 = 0;
    pub const DIR_BACKWARDS: u8 = 1;
    pub const DIR_PINGPONG: u8 = 2;
    pub const REPS_UNSET: u32 = 0;
    pub const DUR_FRAME_UNSET: u32 = 0;
    pub const DUR_REP_UNSET: u32 = 0;
    pub const Y_SORT_UNSET: f32 = f32::NAN;
    pub const UNSET_ALTERNATING_START_FRAMES: (usize, usize) = (usize::MAX, usize::MAX);

    pub fn anim_format_id(&self) -> Option<&str> {
        if self.anim_format_id.is_empty() {
            None
        } else {
            Some(self.anim_format_id.as_str())
        }
    }

    pub fn alternating_start_frames(&self) -> Option<(usize, usize)> {
        if self.alternating_start_frames == Self::UNSET_ALTERNATING_START_FRAMES {
            None
        } else {
            Some(self.alternating_start_frames)
        }
    }

    pub fn dir(&self) -> Option<bool> {
        match self.dir {
            Self::DIR_BACKWARDS => Some(true),
            Self::DIR_PINGPONG => Some(false),
            _ => None,
        }
    }

    pub fn reps(&self) -> Option<usize> {
        if self.reps == Self::REPS_UNSET {
            None
        } else {
            Some(self.reps as usize)
        }
    }

    pub fn dur_frame(&self) -> Option<u32> {
        if self.dur_frame == Self::DUR_FRAME_UNSET {
            None
        } else {
            Some(self.dur_frame)
        }
    }

    pub fn dur_rep(&self) -> Option<u32> {
        if self.dur_rep == Self::DUR_REP_UNSET {
            None
        } else {
            Some(self.dur_rep)
        }
    }

    pub fn y_sort(&self) -> Option<f32> {
        if self.y_sort.is_nan() {
            None
        } else {
            Some(self.y_sort)
        }
    }

    pub fn color(&self) -> Option<[u8; 4]> {
        self.color
    }
}

#[derive(Deserialize, Serialize, TypePath, Clone)]
#[serde(default)]
pub struct ClipConfig {
    pub target: usize,
    pub is_row: bool,
    pub partial: (usize, usize),
    pub start_frame: usize,
    pub dir: u8,
    pub reps: u32,
    pub dur_frame: u32,
    pub dur_rep: u32,
}

impl Default for ClipConfig {
    fn default() -> Self {
        Self {
            target: 0,
            is_row: false,
            partial: Self::UNSET_PARTIAL,
            start_frame: Self::START_FRAME_UNSET,
            dir: Self::DIR_UNSET,
            reps: Self::REPS_UNSET,
            dur_frame: Self::DUR_FRAME_UNSET,
            dur_rep: Self::DUR_REP_UNSET,
        }
    }
}

impl ClipConfig {
    pub const DIR_UNSET: u8 = 0;
    pub const DIR_BACKWARDS: u8 = 1;
    pub const DIR_PINGPONG: u8 = 2;
    pub const REPS_UNSET: u32 = 0;
    pub const DUR_FRAME_UNSET: u32 = 0;
    pub const DUR_REP_UNSET: u32 = 0;
    pub const UNSET_PARTIAL: (usize, usize) = (usize::MAX, usize::MAX);
    pub const START_FRAME_UNSET: usize = usize::MAX;

    pub fn partial(&self) -> Option<(usize, usize)> {
        if self.partial == Self::UNSET_PARTIAL {
            None
        } else {
            Some(self.partial)
        }
    }

    pub fn start_frame(&self) -> Option<usize> {
        if self.start_frame == Self::START_FRAME_UNSET {
            None
        } else {
            Some(self.start_frame)
        }
    }

    pub fn dir(&self) -> Option<bool> {
        match self.dir {
            Self::DIR_BACKWARDS => Some(true),
            Self::DIR_PINGPONG => Some(false),
            _ => None,
        }
    }

    pub fn reps(&self) -> Option<usize> {
        if self.reps == Self::REPS_UNSET {
            None
        } else {
            Some(self.reps as usize)
        }
    }

    pub fn dur_frame(&self) -> Option<u32> {
        if self.dur_frame == Self::DUR_FRAME_UNSET {
            None
        } else {
            Some(self.dur_frame)
        }
    }

    pub fn dur_rep(&self) -> Option<u32> {
        if self.dur_rep == Self::DUR_REP_UNSET {
            None
        } else {
            Some(self.dur_rep)
        }
    }
}
