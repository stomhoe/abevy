use bevy::ecs::entity_disabling::Disabled;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_replicon::prelude::ClientState;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use ::sprite_shared::sprite_scale_offset::AllScalesAndOffsets;
use ::sprite_shared::*;
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
pub fn tick_time_based_multipliers(time: Res<Time>, 
    mut query: Query<(&mut TimeBasedMultiplier, Option<&TickMultFactor>, Option<&TickMultFactors>)>) {
    for (mut multiplier, tick_mult_factor, tick_mult_factors) in query.iter_mut() {
        let mut factor = tick_mult_factor.map(|f| f.value()).unwrap_or(1.0);
        if let Some(factors) = tick_mult_factors {
            factor *= factors.0.iter().map(|f| f.value()).product::<f32>();
        }
        multiplier.timer.tick(time.delta().mul_f32(factor));
    }
}


#[derive(Bundle)]
struct DenyForClonedEntityZeroChildren( EntityZero, BaseHolderRef, Disabled, ImagePathHolder, AcZ, YSortOrigin, AllScalesAndOffsets, StrId, );

#[allow(unused_parens)]
pub fn clone_ezero_children_ents(mut cmd: Commands, 
    query: Query<(Entity, &EntityZeroRef, Has<Replicated>, Has<Persisted>),
    (Changed<EntityZeroRef>, common::AnyDisabling)>,

    ezero: Query<(&Children, Option<&HeldSprites>, ),(common::AnyDisabling)>,
    client_state: Res<State<ClientState>>,
) {
    let mut new_child_of = Vec::new();
    let mut new_base_holder_ref = Vec::new();

    let is_client = *client_state.get() != ClientState::Disconnected;
    query.iter().for_each(|(new_ent, ezero_ref, is_replicated, is_persisted)| {
        let Ok((ezero_children, ezero_held_sprites)) = ezero.get(ezero_ref.0) 
        else { return };

        let is_replicated = (is_replicated || is_persisted);

        if is_client && is_replicated {
            return;
        }

        ezero_children.iter().for_each(|child_to_clone| {
            let cloned_child = cmd.entity(child_to_clone).clone_and_spawn_with_opt_out(
                move |builder|{ builder.deny::<DenyForClonedEntityZeroChildren>();
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
        });
    });
    cmd.try_insert_batch(new_child_of);
    cmd.try_insert_batch(new_base_holder_ref);
}

#[allow(unused_parens)]
pub fn despawn_sprites_without_childof(mut cmd: Commands, 
    query: Query<(Entity),(With<Sprite>, Without<ChildOf>, common::AnyDisabling)>,
) {
    query.iter().for_each(|sprite_ent| {
        cmd.entity(sprite_ent).try_despawn()
    });
}


/// DEACTIVATE THIS SYSTEM IN RELEASE BUILDS !!!!
#[allow(unused_parens)]
pub fn set_entity_name(
    ezeros_query: Query<AnyOf<(&Prefix, &StrId, &StrId20B, &DisplayName, &EntityZeroRef)>, (With<EntityZero>, common::AnyDisabling)>,
    mut changers_query: Query<(&mut Name, AnyOf<(&Prefix, &StrId, &StrId20B, &DisplayName, &EntityZeroRef)>), 
    (
        Or<(Changed<Prefix>, Changed<StrId>, Changed<StrId20B>, Changed<DisplayName>, Changed<EntityZeroRef>)>, 
    common::AnyDisabling)>,
) {
    for (mut name, (e_prefix, strid, strid20b, display_name, ezero_ref)) in changers_query.iter_mut() {
        let mut prefix = e_prefix.map(|p| p.as_str());
        let mut sid = strid.map(|s| s.as_str());
        let mut sid20 = strid20b.map(|s| s.as_str());
        let mut display_name = display_name;

        let mut ezero_id = String::new();
        if let Some(ezero_ref) = ezero_ref {
            if let Ok((z_prefix, z_strid, z_strid20, z_display_name, _)) =
            ezeros_query.get(ezero_ref.0)
            {
            if prefix.is_none() {
                prefix = z_prefix.map(|p| p.as_str());
            }
            if sid.is_none() {
                sid = z_strid.map(|s| s.as_str());
            }
            if sid20.is_none() {
                sid20 = z_strid20.map(|s| s.as_str());
            }
            if display_name.is_none() {
                display_name = z_display_name;
            }
            }
            ezero_id = format!("{:?}", ezero_ref.0);
        }

        let prefix = prefix.unwrap_or("");
        let sid = sid.unwrap_or("");
        let sid20 = sid20.unwrap_or("");

        let mut new_name = String::with_capacity(
            prefix.len()
            + 1
            + sid.len()
            + sid20.len()
            + ezero_id.len()
            + display_name.map(|d| d.0.len() + 2).unwrap_or(0),
        );

        new_name.push_str(prefix);
        new_name.push(' ');
        new_name.push_str(sid);
        new_name.push_str(sid20);
        if !ezero_id.is_empty() {
            new_name.push_str(&ezero_id);
        }

        if let Some(dn) = display_name {
            new_name.push(' ');
            new_name.push_str(dn.0.as_str());
        }

        name.set(new_name);
    }
}



#[allow(unused_parens)]
pub fn tick_despawn_timers(mut cmd: Commands, 
    mut query: Query<(Entity, &mut DespawnTimer),()>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            cmd.entity(entity).try_despawn();
        }
    }
}


#[allow(unused_parens)]
pub fn tick_sim_despawn_timers(mut cmd: Commands, 
    mut query: Query<(Entity, &mut SimDespawnTimer),()>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            cmd.entity(entity).try_despawn();
        }
    }
}
