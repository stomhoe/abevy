#[allow(unused_imports, ambiguous_glob_reexports)] use {bevy::prelude::*, };

pub use color_sampler::*;

pub mod color_sampler;
pub mod color_sampler_seris;
pub mod color_sampler_components;
pub mod color_sampler_resources;
mod color_sampler_systems;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        color_sampler::*,
        color_sampler_seris::*,
        color_sampler_components::*,
        color_sampler_resources::*,
        color_sampler_systems::*,
    };
}
