pub mod game;
pub use game::*;
pub mod game_init_systems;


#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        game::*,
        game_init_systems::*,
    };
}
