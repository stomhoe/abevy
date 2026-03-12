use bevy_enhanced_input::prelude::*;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ToggleSimulationAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct DebugIncreaseSpeedAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct DebugDecreaseSpeedAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct DebugToggleHotReloadWindowAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct DebugToggleMainMenuAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct HotReloadAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct AssetReloadAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ToggleInspectorAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct CameraZoomAction;
