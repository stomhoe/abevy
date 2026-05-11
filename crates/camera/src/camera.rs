use bevy::prelude::*;
use bevy_lit::{directional_light::DirectionalLight2d, prelude::Lighting2dPlugin};
use bevy_replicon::prelude::*;
use common::common_states::GamePhase;

use crate::{camera_systems::*, lighting_systems::*};
use game_common::game_common::GameplaySystems;



pub fn plugin(app: &mut App) {
    app
    .add_plugins((Lighting2dPlugin, ))
    .add_systems(Startup, spawn_camera)
    .add_systems(OnExit(GamePhase::ActiveGame), disable_lighting)
    .add_systems(Update, (enable_lighting, sync_dir_light_angle).in_set(GameplaySystems))
    .add_systems(Update, (
        delete_prev_camera_target, camera_follow_target, camera_zoom_system, hide_noncurrent_dimension,
    ).in_set(GameplaySystems))
    ;
}
