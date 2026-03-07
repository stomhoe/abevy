pub mod player;
pub use player::*;

pub mod player_components;
pub mod player_resources;
mod player_systems;
#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        player::*,
        player_components::*,
        player_resources::*,
        player_systems::*,
    };
}
