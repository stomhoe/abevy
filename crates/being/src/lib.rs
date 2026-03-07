pub mod being;
pub use being::*;

pub mod being_components;
mod being_systems;
mod being_behavior_systems;
mod being_build_systems;
pub mod being_bundles;

pub mod being_inst_template;

pub mod race;
pub mod sex;
pub mod body;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        being::*,
        being_components::*,
        being_systems::*,
        being_behavior_systems::*,
        being_build_systems::*,
        being_bundles::*,
        being_inst_template::*,
        race::*,
        sex::*,
        body::*,
    };
}
