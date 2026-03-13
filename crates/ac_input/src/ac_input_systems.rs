use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use game_common::game_common_states::SimulationState;
use player::player_components::{Mine, Player};

use crate::ac_input_actions::*;

pub fn toggle_simulation(
    toggle_events: Single<&ActionEvents, With<Action<ToggleSimulationAction>>>,
    current_state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    if !toggle_events.contains(ActionEvents::START) {
        return;
    }

    match current_state.get() {
        SimulationState::Paused => {
            info!("Switching to Running state");
            next_state.set(SimulationState::Running)
        }
        SimulationState::Running => {
            info!("Switching to Paused state");
            next_state.set(SimulationState::Paused)
        }
    }
}

pub fn spawn_input_contexts(mut commands: Commands) {
    let holder = commands.spawn(InputContextsHolder).id();
    commands.spawn((
        DebugInputContext,
        ChildOf(holder),
        actions!(DebugInputContext[
            (Action::<ToggleSimulationAction>::new(), bindings![KeyCode::Space]),
            (
                Action::<DebugIncreaseSpeedAction>::new(),
                bindings![KeyCode::NumpadAdd, KeyCode::Equal],
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

pub fn add_being_input_context(
    mut commands: Commands,
    my_player_query: Query<Entity, (With<Mine>, With<Player>, Without<Actions<BeingDirectControlInputContext>>)>,
    players_query: Query<Entity, With<Player>>,
    mut removed_mine_query: RemovedComponents<Mine>,
) {
    for player_ent in my_player_query.iter() {
        commands.entity(player_ent).try_insert((
            BeingDirectControlInputContext,
            ::bevy::prelude::related!(bevy_enhanced_input::prelude::Actions<BeingDirectControlInputContext>[(Action::<DcWasdAction>::new(),DeadZone::default(),Bindings::spawn((Cardinal::wasd_keys(),Cardinal::arrows(),Cardinal::dpad(),Axial::left_stick(),)),),(Action::<DcMeleeAttackAction>::new(),bindings![KeyCode::ControlLeft],),(Action::<DcItemPickupAction>::new(),bindings![KeyCode::KeyQ],)]),
        ));
    }
    for removed_mine in removed_mine_query.read() {
        let Ok(player_ent) = players_query.get(removed_mine) else {
            continue;
        };
        commands.entity(player_ent).try_remove::<Actions<BeingDirectControlInputContext>>();
    }
}
