use core::f32;

use being_shared::{ControlledBy, ControlledLocally, HumanControlled};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy::ecs::entity::EntityHashSet;

use modifier::{modifier_components::*, modifier_move_components::*, modifier_types::*};
use player::{player_components::*, player_resources::KeyboardInputMappings};

use crate::{movement_components::*, movement_messages::SendMoveInput};

pub fn update_human_move_input(
    keys: Res<ButtonInput<KeyCode>>,
    input_mappings: Res<KeyboardInputMappings>,
    mut move_input: Query<(&mut InputDirection, &HumanControlled), With<ControlledLocally>>,
) {
    let mut input_dir = Vec2::ZERO;
    if keys.pressed(input_mappings.move_up) {
        input_dir.y += 1.0;
    }
    if keys.pressed(input_mappings.move_down) {
        input_dir.y -= 1.0;
    }
    if keys.pressed(input_mappings.move_left) {
        input_dir.x -= 1.0;
    }
    if keys.pressed(input_mappings.move_right) {
        input_dir.x += 1.0;
    }

    if input_dir != Vec2::ZERO {
        input_dir = input_dir.normalize();
    }

    for (mut input_direction, human_controlled) in move_input.iter_mut() {
        if human_controlled.0 && input_direction.0 != input_dir {
            trace!(target: "movement", "Updating human move input");
            input_direction.0 = input_dir;
        }
    }
}

pub fn send_move_input_to_server(
    mut event_writer: MessageWriter<SendMoveInput>,
    move_input: Query<(Entity, &InputDirection), (Changed<InputDirection>, With<ControlledLocally>)>,
) {
    let mut to_write = Vec::new();
    for (being_ent, input_dir) in move_input.iter() {
        trace!(target: "movement", "Sending move input for entity {:?} with vector {:?}", being_ent, input_dir.0);
        to_write.push(SendMoveInput { being_ent, vec: input_dir.0 });
    }
    event_writer.write_batch(to_write);
}

pub fn receive_move_input_from_client(
    mut events: MessageReader<FromClient<SendMoveInput>>,
    mut controlled_beings_query: Query<(&mut InputDirection, &ControlledBy)>,
) {
    for from_client in events.read() {
        let SendMoveInput { vec: new_vec, being_ent } = from_client.message.clone();

        if let Ok((mut input_dir, controlled_by)) = controlled_beings_query.get_mut(being_ent) {
            let Some(client_entity) = from_client.client_id.entity() else { continue; };

            if controlled_by.client == client_entity {
                if input_dir.0 != new_vec {
                    input_dir.0 = new_vec;
                }
            } else {
                warn!(
                    "Client tried to control a being not controlled by them: {} (controlled_by.client: {:?}, from_client.client_entity: {:?})",
                    being_ent, controlled_by.client, client_entity
                );
            }
        } else {
            warn!("Client tried to control a being that does not exist in server or is not controllable {}", being_ent);
        }
    }
}

pub fn process_input_direction_modifiers(
    state: Res<State<ClientState>>,
    mut being_query: Query<
        (Entity, &AppliedModifiers, &InputDirection, &mut MoveVecMag, Has<ControlledLocally>),
    >,
    modifiers_query: Query<
        (Entity, &ModifierTarget, &CurrEffectiveValue, &ApplyMode, Has<InvertMovement>),
    >,
) {
    let is_client = state.get() == &ClientState::Connected;

    for (being_ent, applied, input_dir, mut move_state, controlled_locally) in being_query.iter_mut() {
        if is_client && !controlled_locally { continue; }

        let mut invert_sum: f32 = 0.0;
        let mut invert_scale: f32 = 1.0;

        let mut effects = EntityHashSet::default();
        applied.entities().iter().for_each(|&ent| { effects.insert(ent); });

        for (modifier_ent, target, ..) in modifiers_query.iter() {
            if target.0 == being_ent {
                effects.insert(modifier_ent);
            }
        }

        for effect in effects.iter() {
            if let Ok((_, _, &CurrEffectiveValue(val), optype, invert)) = modifiers_query.get(*effect) {
                match optype {
                    ApplyMode::Add if invert => invert_sum += val,
                    ApplyMode::Mul if invert => invert_scale *= val.max(0.0),
                    _ => {}
                }
            }
        }

        move_state.norm_move_dir = if input_dir.0 == Vec2::ZERO {
            Vec2::ZERO
        } else {
            let dir = input_dir.0;
            if invert_sum * invert_scale > 1.0 { -dir } else { dir }
        };
    }
}
