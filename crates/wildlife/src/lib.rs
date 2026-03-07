pub mod wildlife;
pub use wildlife::*;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        wildlife::*,
    };
}
