#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(AssetCollection, Resource, Default, Reflect)]
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
    pub rows_cols: Option<(usize, usize)>, //rows, cols
    //None: forward, Some(true): backward, Some(false): ping-pong
    pub dir: Option<bool>, 
    pub reps: Option<usize>, //0: infinite, n>0: n repetitions
    pub dur_frame: Option<u32>, //milliseconds
    pub dur_rep: Option<u32>, //milliseconds
    pub offset: Option<[f32; 2]>,
    pub scale: Option<[f32; 2]>,
    pub y_sort: Option<f32>,
    pub z: i32,
    pub color: Option<[u8; 4]>, 
}


#[derive(Deserialize, Serialize, Reflect, Default, Clone)]
/// Configuration for a sprite animation sequence.
/// Animation config for a row or column.
/// - `target`: Row/column index.
/// - `is_row`: True for row, false for column.
/// - `partial`: Optional [start, end] indices.
/// - `dir`: None=forward, Some(true)=backward, Some(false)=ping-pong.
/// - `reps`: None=infinite, Some(n)=fixed repeats.
/// - `dur_frame`: Optional ms per frame.
/// - `dur_rep`: Optional ms per repetition.
pub struct ClipConfig {
    pub target: usize,
    pub is_row: bool, 
    pub partial: Option<(usize, usize)>, 
    pub dir: Option<bool>, 
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>, 
    pub dur_rep: Option<u32>,
}

