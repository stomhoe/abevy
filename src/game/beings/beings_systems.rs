use bevy::prelude::*;

use crate::game::{beings::beings_components::{Being, InputMoveDirection}, game_components::Nid, game_resources::NidEntityMap};



pub fn handle_movement(
    time: Res<Time>,
    mut query: Query<(&InputMoveDirection, &mut Transform), With<Being>>,
) {
    for (input_move_direction, mut transform) in query.iter_mut() {
        let speed = 1000.0;
        let delta = time.delta_secs();
        let movement = input_move_direction.0 * speed * delta;
        transform.translation += movement;
    }
}


//ESTO Q SOLO LO HAGA EL HOST, EN EL CASO DE CLIENTES, DEBEN ENVIARLE UN PAQUETE AL HOST PARA QUE LO HAGA
// Y EL HOST DEBE ENVIAR DE VUELTA EL NID DEL BEING CREADO, PARA EVITAR DESYNCS DEBIDO A QUE EL SPAWNEO DE UN BEING SE ADELANTE A OTRO EN EL MISMO FRAME, Y CADA CLIENT LES ASIGNE LAS NIDS FLIPPEADAS
// pub fn host_on_being_spawned(
//     mut commands: Commands,
//     new_being: Query<Entity, Without<Nid>>,
//     mut nid_entity_map: ResMut<NidEntityMap>,
    
// ) {
//     for entity in new_being.iter() {
//         let nid: Nid = nid_entity_map.new_entity(&mut commands, entity);
//         commands.entity(entity).insert(nid);
//     }
// }

// pub fn client_on_being_spawned(
//     mut commands: Commands,
//     new_being: Query<Entity,  Without<Nid>>,
//     mut nid_entity_map: ResMut<NidEntityMap>,
    
// ) {
//     for entity in new_being.iter() {
//         let nid: Nid = nid_entity_map.new_entity(&mut commands, entity);
//         commands.entity(entity).insert(nid);
//     }
// }

