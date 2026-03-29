pub mod wildlife;
pub mod wildlife_cleanup_systems;
pub mod wildlife_seeding_systems;
pub mod wildlife_spawning_helpers;
pub use wildlife::*;
pub use wildlife_cleanup_systems::*;
pub use wildlife_seeding_systems::*;
pub use wildlife_spawning_helpers::*;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        wildlife::*,
        wildlife_cleanup_systems::*,
        wildlife_seeding_systems::*,
        wildlife_spawning_helpers::*,
    };
}
