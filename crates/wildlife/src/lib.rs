pub mod wildlife;
pub mod terrgen_natural_spawning;
pub use wildlife::*;
pub use terrgen_natural_spawning::*;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        wildlife::*,
        terrgen_natural_spawning::*,
    };
}
