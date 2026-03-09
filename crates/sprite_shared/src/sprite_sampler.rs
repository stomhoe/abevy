#[allow(unused_imports)]
use bevy::prelude::*;

pub mod sprite_sampler_components;
pub mod sprite_sampler_seris;

pub use sprite_sampler_components::*;
pub use sprite_sampler_seris::*;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        sprite_sampler_components::*,
        sprite_sampler_seris::*,
    };
}
