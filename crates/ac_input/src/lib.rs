pub mod ac_input;
pub use ac_input::*;
pub use paste;

pub mod ac_input_being_actions;
pub mod ac_input_contexts;
pub mod ac_input_game_actions;
pub mod ac_input_actions;
pub mod player_action_request_macros;
pub mod player_action_requests;
mod ac_input_systems;

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::{
        ac_input::*,
        ac_input_actions::*,
        ac_input_contexts::*,
        player_action_request_macros::*,
        player_action_requests::*,
        ac_input_systems::*,
    };
}
