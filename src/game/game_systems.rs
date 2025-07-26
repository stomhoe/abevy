use bevy::asset::AssetServer;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::common::common_components::GameZindex;
use crate::game::being::being_components::{Being, ControlledBy, PlayerDirectControllable, TargetSpawnPos, };
use crate::game::being::sprite::sprite_components::SpriteDatasChildrenStringIds;
use crate::game::faction::faction_components::SelfFaction;
use crate::game::game_components::*;
use crate::game::game_resources::*;
use crate::game::player::player_components::{CameraTarget, CreatedCharacter, Player, SelfPlayer};
use crate::game::{SimulationState};

#[allow(unused_parens)]
pub fn placeholder_character_creation(mut cmd: Commands, mut query: Query<(),(With<Player>)>) {
}


pub fn spawn_player_beings(
    mut commands: Commands,
    players: Query<(Entity, &CreatedCharacter, Option<&SelfPlayer>), (With<Player>)>,
) {

    for (player_ent, created_character, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_character);

        commands.entity(created_character.0).remove::<ChildOf>();


        commands.entity(created_character.0).insert((
            ControlledBy(player_ent),
            PlayerDirectControllable,
            TargetSpawnPos::new(0.0, 0.0),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),//PROVISORIO
            //HACER Q OTRO SYSTEMA AGREGUE CAMERATARGET AL BEING CONTROLADO
            SpriteDatasChildrenStringIds::new(["humanhe0", "humanbo0"]),
            SelfFaction(),
        ));

        if self_player.is_some() {
            info!("Spawning self player being:");

        } 

        commands.entity(player_ent).remove::<CreatedCharacter>();
    }
}



pub fn toggle_simulation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<SimulationState>>, mut next_state: ResMut<NextState<SimulationState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match current_state.get() {
            SimulationState::Paused => {
                println!("Switching to Running state");
                next_state.set(SimulationState::Running)
            },
            SimulationState::Running => {
                println!("Switching to Paused state");
                next_state.set(SimulationState::Paused)
            },
        }
    }
}




pub fn force_z_index(mut query: Query<(&mut Transform, &GameZindex)>,) {
    for (mut transform, z_index) in query.iter_mut() {
        transform.translation.z = z_index.0 as f32 * 1e-9;
        //println!("transform {}", transform.translation);
    }
}


// fn hit_detection(
//     mut commands: Commands,
//     being_query: Query<(Entity, &Transform), (Without<PhysicallyImmune>, With<Health>)>,
//     bullet_query: Query<&Transform, With<Bullet>>
// ) {
//     for (entity, enemy_transform) in being_query.iter() {
//         for bullet_transform in bullet_query.iter() {
//             // Your collision check
//             if false {
//                 commands.entity(entity).despawn();
//             }
//         }
//     }
// }


pub fn debug_system(mut commands: Commands, query: Query<(Entity, &Transform), With<Being>>, cam_query: Query<&Transform, With<Camera>>) {
   
    
}
pub fn tick_time_based_multipliers(mut cmd: Commands, time: Res<Time>, mut query: Query<&mut TimeBasedMultiplier>) {
    for mut multiplier in query.iter_mut() { multiplier.timer.tick(time.delta()); }
}