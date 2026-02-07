#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::AssetCollection;

use serde::{Deserialize, Serialize};

#[derive(bevy_asset_loader::asset_collection::AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct AnimSerisHandles {
    #[asset(path = "ron/sprite/animation", collection(typed))]
    pub handles: Vec<Handle<AnimationSerialization>>,
}



// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo

#[derive(Component, Deserialize, Serialize, Asset, Reflect, Default, )]
pub struct AnimationSerialization {
    pub id: String,
    pub img_path: String,
    pub clips: Vec<ClipConfig>,

    //optional in case you want to reuse a predefined animation format
    pub anim_format_id: Option<String>,
    pub rows_cols: Option<(usize, usize)>, //rows, cols
    pub save_animation_progress: Option<bool>, //Some(true): save progress, Some(false)/None: don't save
    //None: forward, Some(true): backward, Some(false): ping-pong
    pub dir: Option<bool>,
    pub reps: Option<usize>, //0: infinite, n>0: n repetitions
    pub dur_frame: Option<u32>, //milliseconds
    pub dur_rep: Option<u32>, //milliseconds
    pub offset: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub y_sort: Option<f32>,
    pub z: f32,
    pub color: Option<[u8; 4]>,
}


#[derive(Deserialize, Serialize, Reflect, Default, Clone)]
/// Configuration for a sprite animation sequence.
/// Animation config for a row or column.
/// - `target`: Row/column index.
/// - `is_row`: True for row, false for column.
/// - `partial`: Optional [start, end] indices.
/// - `start_frame`: Optional starting frame index (default: 0).
/// - `alternating_start_frames`: Optional (frame1, frame2) for alternating between two start frames each time animation starts.
/// - `dir`: None=forward, Some(true)=backward, Some(false)=ping-pong.
/// - `reps`: None=infinite, Some(n)=fixed repeats.
/// - `dur_frame`: Optional ms per frame.
/// - `dur_rep`: Optional ms per repetition.
pub struct ClipConfig {
    pub target: usize,
    pub is_row: bool,
    pub partial: Option<(usize, usize)>,
    pub start_frame: Option<usize>,
    pub alternating_start_frames: Option<(usize, usize)>,
    pub dir: Option<bool>,
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>,
    pub dur_rep: Option<u32>,
}
