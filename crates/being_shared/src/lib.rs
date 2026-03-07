pub use being_shared::*;
use bevy::ecs::prelude::*;

pub mod being_shared;
pub mod being_inst_templ_shared;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        being_shared::*,
        being_inst_templ_shared::*,
    };
}
