pub use sprite::*;

pub mod sprite;
pub mod sprite_resources;
pub mod sprite_sampler;
pub mod sprite_components {
    pub use sprite_shared::sprite_components::*;
}

mod sprite_systems;
mod sprite_scale_systems;
mod sprite_offset_systems;
mod sprite_config_init_systems;
mod sprite_build_systems;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        sprite::*,
        sprite_components::*,
        sprite_resources::*,
        sprite_sampler::*,
        sprite_sampler::prelude::*,
        sprite_systems::*,
        sprite_scale_systems::*,
        sprite_offset_systems::*,
        sprite_config_init_systems::*,
        sprite_build_systems::*,
    };
}
