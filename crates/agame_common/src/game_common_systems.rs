use bevy::ecs::entity_disabling::Disabled;
use bevy::input::ButtonInput;

use bevy::prelude::*;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use common::common_components::EntityPrefix;
use common::common_states::ConnectionAttempt;
use common::common_states::GamePhase;

use crate::game_common_components::*;
use crate::game_common_states::*;


// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn reset_states(
    mut connection: ResMut<NextState<ConnectionAttempt>>,
    mut state: ResMut<NextState<GamePhase>>
) {
    state.set(GamePhase::default());
    connection.set(ConnectionAttempt::default());
}


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

use bevy_ecs_tilemap::DrawTilemap;

#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens, )]
pub fn z_sort_system(
    ori_query: Query<(&MyZ, Option<&YSortOrigin>), (Or<(With<Disabled>, Without<Disabled>)>,)>,

    mut with_own_z_query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>, &MyZ, Has<TilemapAnchor>), 
    Or<(Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<MyZ>)>>,
    mut with_entityzero: Query<(&mut Transform, &GlobalTransform, &EntityZeroRef), (Or<(Changed<EntityZeroRef>, Changed<GlobalTransform>)>, Without<MyZ>,)>,

    mut event_writer: EventWriter<DrawTilemap>,

) {//TODO MEJORAR
    let mut to_draw = Vec::new();

    for (ent, mut transform, global_transform, ysort_origin, z_index, is_tilemap) in with_own_z_query.iter_mut() {
        let y_pos = global_transform.translation().y - ysort_origin.cloned().unwrap_or_default().0;
        let target_z = z_index.as_float() - y_pos * YSortOrigin::Y_SORT_DIV;

        if (transform.translation.z - target_z).abs() > f32::EPSILON { 
            transform.translation.z = target_z;
            debug!(target: "zlevel", "Z-Sorting entity to z-index {}", target_z);
            if is_tilemap{
                to_draw.push(DrawTilemap(ent));
            }
        }
    }
    for (mut transform, global_transform, original_ref) in with_entityzero.iter_mut() {
        let Ok((z_index, ysort_origin)) = ori_query.get(original_ref.0) else { continue };

        let y_pos = global_transform.translation().y - ysort_origin.cloned().unwrap_or_default().0;
        let target_z = z_index.as_float() - y_pos * YSortOrigin::Y_SORT_DIV;

        if (transform.translation.z - target_z).abs() > f32::EPSILON { 
            transform.translation.z = target_z;
            debug!(target: "zlevel", "Z-Sorting entity to z-index {}", target_z);
        }
    }
    event_writer.write_batch(to_draw);
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


