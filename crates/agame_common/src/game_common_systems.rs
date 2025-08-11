use bevy::input::ButtonInput;
use bevy::prelude::*;

use crate::game_common_components::*;
use crate::game_common_states::*;




pub fn toggle_simulation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<SimulationState>>, mut next_state: ResMut<NextState<SimulationState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match current_state.get() {
            SimulationState::Paused => {
                info!("Switching to Running state");
                next_state.set(SimulationState::Running)
            },
            SimulationState::Running => {
                info!("Switching to Paused state");
                next_state.set(SimulationState::Paused)
            },
        }
    }
}

pub fn update_transform_z(mut query: Query<(&mut Transform, &MyZ), (Changed<MyZ>,)>) {
    for (mut transform, z_index) in query.iter_mut() {
        let new_z = z_index.as_float();
        if transform.translation.z != new_z {
            debug!(target: "zlevel", "Updating transform z-index to {}", new_z);
            transform.translation.z = new_z;
        }
    }
}


pub fn tick_time_based_multipliers(time: Res<Time>, mut query: Query<(&mut TimeBasedMultiplier, Option<&TickMultFactor>, Option<&TickMultFactors>)>) {
    for (mut multiplier, tick_mult_factor, tick_mult_factors) in query.iter_mut() {
        let mut factor = tick_mult_factor.map(|f| f.value()).unwrap_or(1.0);
        if let Some(factors) = tick_mult_factors {
            factor *= factors.0.iter().map(|f| f.value()).product::<f32>();
        }
        multiplier.timer.tick(time.delta().mul_f32(factor));
    }
}


