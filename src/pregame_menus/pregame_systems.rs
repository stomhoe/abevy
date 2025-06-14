use bevy::prelude::*;
use crate::{pregame_menus::pregame_components::{PreGameScoped}, AppState, StateScopedCamera};

const HOVERED_BUTTON_LIGHTENING_FACTOR: f32 = 0.2;
const HOVERED_PRESSED_BUTTON_DARKENING_FACTOR: f32 = 0.2;
const PRESSED_BUTTON_DARKENING_FACTOR: f32 = 0.2;


#[derive(Component)]
struct HoldingOpt;

pub fn setup(mut commands: Commands) {
    commands.spawn(StateScopedCamera);
}






fn cleanup_pregame_scoped(
    mut commands: Commands,
    query: Query<Entity, With<PreGameScoped>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}