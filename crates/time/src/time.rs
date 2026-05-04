
use bevy::prelude::*;
use bevy_replicon::prelude::ServerState;
use common::common_states::GamePhase;
use game_common::{GameplaySystems, HostSystems};

use crate::time_resources::*;
use crate::time_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClockSystems;

pub struct ClockPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app
            .configure_sets(Update, (ClockSystems.in_set(GameplaySystems), ClockSystems.in_set(HostSystems)))
            .init_resource::<CurrYear>()
            .init_resource::<CurrSeason>()
            .init_resource::<CurrDay>()
            .init_resource::<CurrHour>()
            .init_resource::<CurrMin>()
            .init_resource::<SimTimeScale>()
            .init_resource::<InGameTiming>()
            .add_systems(
                Update,
                (
                    pass_time,
                    reduce_remaining_days,
                )
                    .in_set(ClockSystems)
                    .run_if(in_state(GamePhase::ActiveGame).and(in_state(ServerState::Running))),
            )
        ;
    }
}
