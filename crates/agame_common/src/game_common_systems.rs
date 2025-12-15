use bevy::ecs::bundle;
use bevy::ecs::entity_disabling::Disabled;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::render::sync_world::RenderEntity;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_replicon::prelude::ClientState;
use bevy_replicon::prelude::Replicated;
use common::common_components::ImagePathHolder;
use common::common_states::ConnectionAttempt;
use common::common_states::GamePhase;
use ::sprite_shared::*;
use crate::game_common_components::*;
use crate::game_common_states::*;
use bevy_ecs_tilemap::DrawTilemap;

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


#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens, )]
pub fn z_sort_system(
    ori_query: Query<(&MyZ, Option<&YSortOrigin>), (Or<(With<Disabled>, Without<Disabled>)>,)>,

    mut with_own_z_query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>, &MyZ, Has<TilemapAnchor>), 
    Or<(Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<MyZ>)>>,
    mut with_entityzero: Query<(&mut Transform, &GlobalTransform, &EntityZeroRef), (Or<(Changed<EntityZeroRef>, Changed<GlobalTransform>)>, Without<MyZ>,)>,

    mut event_writer: MessageWriter<DrawTilemap>,

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

#[allow(unused_parens)]
pub fn disable_ezeros(mut cmd: Commands, 
    query: Query<(Entity),(With<EntityZero>, Without<Disabled>)>,
) {
    let mut batch = Vec::with_capacity(query.iter().count());
    for ent in query.iter() { 
        batch.push((ent, Disabled, ));
    }
    cmd.insert_batch(batch);
}


#[derive(Bundle)]
struct BaseDeny( EntityZero, BaseHolderRef, Disabled, ImagePathHolder);

#[allow(unused_parens)]
pub fn clone_ezero_children_ents(mut cmd: Commands, 
    query: Query<(Entity, &EntityZeroRef, Has<Replicated>, Has<Persisted>),
    (Changed<EntityZeroRef>, Or<(Without<Disabled>, With<Disabled>)>)>,

    ezero: Query<(&Children, Option<&HeldSprites>, ),(Or<(Without<Disabled>, With<Disabled>)>)>,
    client_state: Res<State<ClientState>>,
) {
    let mut new_child_of = Vec::new();
    let mut new_base_holder_ref = Vec::new();

    let is_client = *client_state.get() != ClientState::Disconnected;

    for (new_ent, ezero_ref, is_replicated, is_persisted) in query.iter() {
        let Ok((ezero_children, ezero_held_sprites)) = ezero.get(ezero_ref.0) 
        else { continue };

        let is_replicated = (is_replicated || is_persisted);

        if is_client && is_replicated {
            continue;
        }

        for child_to_clone in ezero_children.iter() {
            let cloned_child = cmd.entity(child_to_clone).clone_and_spawn_with_opt_out(
                move |builder|{ builder.deny::<(EntityZero, BaseHolderRef, Disabled, ImagePathHolder)>();
                    if ! is_replicated{
                        builder.deny::<Replicated>();
                    }
                }
            ).id();
            new_child_of.push((cloned_child, (ChildOf(new_ent), EntityZeroRef(child_to_clone))));


            debug!(target: "entity_zero", "Cloned child {:?} of EntityZero {:?} as child of {:?}", cloned_child, ezero_ref.0, new_ent);

            if let Some(ezero_held_sprites) = ezero_held_sprites {
                if ezero_held_sprites.entities().contains(&child_to_clone) {
                    new_base_holder_ref.push((cloned_child, BaseHolderRef { base: new_ent,  }, ));
                }
            }
        }
    }
    cmd.try_insert_batch(new_child_of);
    cmd.try_insert_batch(new_base_holder_ref);
}
