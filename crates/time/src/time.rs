
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_common::{GameplaySystems, HostSystems};

use time_shared::*;
use crate::time_systems::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ClockSystems;

pub struct ClockPlugin;
#[allow(unused_parens, path_statements, )]
impl Plugin for ClockPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<CurrYear>()
        .init_resource::<CurrSeason>()
        .init_resource::<CurrDay>()
        .init_resource::<CurrHour>()
        .init_resource::<CurrMin>()
        .init_resource::<InGameTiming>()
        .replicate::<SimTimeScale>()
        .add_systems(
            Update,
            (
                pass_time,
                reduce_remaining_days,
            )
            .in_set(ClockSystems)
        )
        .configure_sets(Update, (ClockSystems.in_set(GameplaySystems).in_set(HostSystems)))
        ;
    }
}
