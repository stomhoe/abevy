#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct AnimSerisHandles {
    #[asset(path = "ron/sprite/animation", collection(typed))]
    pub handles: Vec<Handle<AnimationSeri>>,
}



// TODO: hacer shaders aplicables? (para meditacion por ej)
// TODO: hacer que se puedan aplicar colorses sobre máscaras como en humanoid alien races del rimworld. hacer un mapa color-algo 

#[derive(serde::Deserialize, Asset, Reflect, Default,)]
pub struct AnimationSeri {
    pub id: String,
    pub rows_cols: (usize, usize), //rows, cols
    pub img_path: String,

    pub clips: Vec<AnimConfig>,
    //None: forward, Some(true): backward, Some(false): ping-pong
    pub dir: Option<bool>, 
    pub reps: Option<usize>, //0: infinite, n>0: n repetitions
    pub dur_frame: Option<u32>, //milliseconds
    pub dur_rep: Option<u32>, //milliseconds
    pub ysort: Option<f32>,
}


#[derive(serde::Deserialize, Reflect, Default,)]
/// Configuration for a sprite animation sequence.
/// Animation config for a row or column.
/// - `target`: Row/column index.
/// - `is_row`: True for row, false for column.
/// - `partial`: Optional [start, end] indices.
/// - `dir`: None=forward, Some(true)=backward, Some(false)=ping-pong.
/// - `reps`: None=infinite, Some(n)=fixed repeats.
/// - `dur_frame`: Optional ms per frame.
/// - `dur_rep`: Optional ms per repetition.
pub struct AnimConfig {
    pub target: usize,
    pub is_row: bool, 
    pub partial: Option<(usize, usize)>, 
    pub dir: Option<bool>, 
    pub reps: Option<usize>,
    pub dur_frame: Option<u32>, 
    pub dur_rep: Option<u32>,
}



// No olvidarse de agregarlo al Plugin del módulo
// .add_client_trigger::<AnimStateUpdated>(Channel::Ordered)