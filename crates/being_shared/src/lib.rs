pub use being_shared::*;
pub use common::common_components::Grounding;

pub mod being_shared;
pub mod being_inst_templ_shared;
pub mod being_shared_messages;
pub mod being_shared_resources;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        being_shared::*,
        being_inst_templ_shared::*,
        being_shared_messages::*,
        being_shared_resources::*,
        Grounding,
    };
}
