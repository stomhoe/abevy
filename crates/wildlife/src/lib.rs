pub mod wildlife;
pub mod wildlife_resources;
pub mod wildlife_spawning_systems;
pub use wildlife::*;
pub use wildlife_resources::*;
pub use wildlife_spawning_systems::*;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        wildlife::*,
        wildlife_resources::*,
        wildlife_spawning_systems::*,
    };
}
