use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::ac_input_actions::*;

pub fn spawn_player_input_context(mut commands: Commands) {
    commands.spawn((
        PlayerInputContext,
        actions!(PlayerInputContext[
            (Action::<ToggleSimulationAction>::new(), bindings![KeyCode::Space]),
            (
                Action::<DebugIncreaseSpeedAction>::new(),
                bindings![KeyCode::NumpadAdd],
            ),
            (
                Action::<DebugDecreaseSpeedAction>::new(),
                bindings![KeyCode::NumpadSubtract],
            ),
            (
                Action::<DebugToggleHotReloadWindowAction>::new(),
                bindings![KeyCode::F12],
            ),
            (
                Action::<DebugToggleMainMenuAction>::new(),
                bindings![KeyCode::F11],
            ),
            (Action::<HotReloadAction>::new(), bindings![KeyCode::KeyR]),
            (Action::<AssetReloadAction>::new(), bindings![KeyCode::F6]),
            (Action::<ToggleInspectorAction>::new(), bindings![KeyCode::Escape]),
            (
                Action::<CameraZoomAction>::new(),
                Bindings::spawn(Spawn((Binding::mouse_wheel(), SwizzleAxis::YXZ))),
            ),
        ]),
    ));
}
