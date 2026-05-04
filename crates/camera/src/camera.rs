use bevy::prelude::*;
use bevy_firefly::prelude::FireflyPlugin;
use common::common_states::GamePhase;

use crate::camera_systems::*;
use game_common::game_common::GameplaySystems;



pub fn plugin(app: &mut App) {
    app
    .add_plugins((FireflyPlugin, ))
    .add_systems(Startup, spawn_camera)
    .add_systems(OnExit(GamePhase::ActiveGame), disable_firefly_lighting)
    .add_systems(Update, enable_firefly_lighting.run_if(in_state(GamePhase::ActiveGame)).in_set(GameplaySystems).after(camera_follow_target).before(sync_firefly_lighting))
    .add_systems(Update, sync_firefly_lighting.run_if(in_state(GamePhase::ActiveGame)).in_set(GameplaySystems).after(camera_follow_target))
    .add_systems(Update, (
        delete_prev_camera_target, camera_follow_target, camera_zoom_system, hide_nonvisualized_dimension,
    ).in_set(GameplaySystems))
    ;
}
