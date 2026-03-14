pub mod wildlife;
pub mod natural_spawning;
pub use wildlife::*;
pub use natural_spawning::*;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        wildlife::*,
        natural_spawning::*,
    };
}
