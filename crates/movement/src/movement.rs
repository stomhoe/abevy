use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::game_common::{GameplaySystems, SimRunningSystems};

use crate::{movement_components::*, movement_events::*, movement_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MovementSystems;

const MOVEMENT_SCHEDULE: FixedUpdate = FixedUpdate;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .add_systems(MOVEMENT_SCHEDULE, (
            (process_movement_modifiers, update_facing_dir, apply_movement, update_human_move_input,
                (send_move_input_to_server, on_receive_transf_from_server).run_if(not(server_or_singleplayer)),
                receive_move_input_from_client.run_if(server_or_singleplayer)
            ).in_set(MovementSystems),
        ))
        .configure_sets(FixedUpdate, 
            (MovementSystems).in_set(SimRunningSystems)
        )
        .configure_sets(Update, 
            (MovementSystems).in_set(SimRunningSystems)
        )


        .add_mapped_client_event::<SendMoveInput>(Channel::Unreliable)

        .add_mapped_server_event::<TransformFromServer>(Channel::Unreliable)

        .register_type::<InputMoveVector>()
        .register_type::<ProcessedInputVector>()
        //.replicate_once::<GlobalTransform>()
    ;
}