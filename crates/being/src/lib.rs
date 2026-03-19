

pub mod being;
pub use being::*;

pub mod being_components;
mod being_messages;
mod being_systems;
mod being_melee_systems;
mod being_control_systems;
mod being_portal_systems;
mod being_behavior_systems;
mod being_on_chunk_despawn_systems;
mod being_interaction_zone_helper;
pub mod nav;
mod being_build_systems;
pub mod being_bundles;

pub mod being_inst_template;

pub mod race;
pub mod pack;
pub mod sex;
pub mod body;



#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        being::*,
        being_components::*,
        being_messages::*,
        being_systems::*,
        being_melee_systems::*,
        being_control_systems::*,
        being_portal_systems::*,
        being_behavior_systems::*,
        being_on_chunk_despawn_systems::*,
        nav::being_nav_systems::*,
        nav::being_nav_components::*,
        nav::being_nav_resources::*,
        nav::being_nav_structs::*,
        nav::being_nav_helpers::*,
        being_build_systems::*,
        being_bundles::*,
        being_inst_template::*,
        race::*,
        pack::*,
        sex::*,
        body::*,
    };
}
