pub mod asset_loading;

pub use asset_loading::*;
mod asset_loading_systems;


#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        asset_loading::*,
        asset_loading_systems::*,
    };
}
