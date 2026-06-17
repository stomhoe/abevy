use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::*;
use common::common_components::SettingsEntity;
use game_common::game_common_states::SimulationState;
use debug_shared::DebugUiConfig;
use player_shared::player_components::{Mine, Player};
use serde::{Deserialize, Serialize};

use crate::ac_input_actions::*;

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct ClientToggleSimulationRequest;

#[derive(Deserialize, Message, Serialize, Clone, MapEntities)]
pub struct SyncSimulationState {
    pub state: SimulationState,
}

pub fn toggle_simulation(
    toggle_events: Single<&ActionEvents, With<Action<ToggleSimulationAction>>>,
    current_state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
    debug_ui_config: Query<&DebugUiConfig, With<SettingsEntity>>,
    client_state: Res<State<ClientState>>,
    server_state: Res<State<ServerState>>,
    mut client_toggle_request_writer: MessageWriter<ClientToggleSimulationRequest>,
    mut sync_state_writer: MessageWriter<ToClients<SyncSimulationState>>,
) {
    if !toggle_events.contains(ActionEvents::START) {
        return;
    }

    let next_state_value = match current_state.get() {
        SimulationState::Paused => SimulationState::Running,
        SimulationState::Running => SimulationState::Paused,
    };

    let Ok(debug_ui_config) = debug_ui_config.single() else {
        return;
    };
    if *client_state.get() == ClientState::Connected {
        if !debug_ui_config.client_debug {
            return;
        }
        client_toggle_request_writer.write(ClientToggleSimulationRequest);
        return;
    }

    info!("Switching to {:?} state", next_state_value);
    next_state.set(next_state_value.clone());
    if *server_state.get() == ServerState::Running {
        sync_state_writer.write(ToClients {
            mode: SendMode::Broadcast,
            message: SyncSimulationState {
                state: next_state_value,
            },
        });
    }
}

#[allow(unused_parens, )]
pub fn receive_toggle_simulation_request(
    mut requests: MessageReader<FromClient<ClientToggleSimulationRequest>>,
    current_state: Res<State<SimulationState>>,
    mut next_state: ResMut<NextState<SimulationState>>,
    mut sync_state_writer: MessageWriter<ToClients<SyncSimulationState>>,
) {
    let mut should_toggle = false;
    for _ in requests.read() {
        should_toggle = true;
        break;
    }
    if !should_toggle {
        return;
    }

    let next_state_value = match current_state.get() {
        SimulationState::Paused => SimulationState::Running,
        SimulationState::Running => SimulationState::Paused,
    };

    info!("Received client toggle request, switching to {:?} state", next_state_value);
    next_state.set(next_state_value.clone());
    sync_state_writer.write(ToClients {
        mode: SendMode::Broadcast,
        message: SyncSimulationState {
            state: next_state_value,
        },
    });
}

#[allow(unused_parens, )]
pub fn receive_simulation_state_from_server(
    mut messages: MessageReader<SyncSimulationState>,
    mut next_state: ResMut<NextState<SimulationState>>,
) {
    for message in messages.read() {
        next_state.set(message.state.clone());
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
            ::bevy::prelude::related!(bevy_enhanced_input::prelude::Actions<BeingDirectControlInputContext>[(Action::<DcWasdAction>::new(),DeadZone::default(),Bindings::spawn((Cardinal::wasd_keys(),Cardinal::arrows(),Cardinal::dpad(),Axial::left_stick(),)),),(Action::<DcMeleeAttackAction>::new(),bindings![KeyCode::ControlLeft],),(Action::<DcItemPickupAction>::new(),bindings![KeyCode::KeyQ],),(Action::<DcDebugIncreaseSpeedAction>::new(),bindings![KeyCode::NumpadAdd,KeyCode::Equal],),(Action::<DcDebugDecreaseSpeedAction>::new(),bindings![KeyCode::NumpadSubtract],)]),
        ));
    }
    for removed_mine in removed_mine_query.read() {
        let Ok(player_ent) = players_query.get(removed_mine) else {
            continue;
        };
        commands.entity(player_ent).try_remove::<Actions<BeingDirectControlInputContext>>();
    }
}
