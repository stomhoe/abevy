use bevy::asset::AssetServer;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::common::common_components::{DisplayName, MyZ};
use crate::game::being::being_components::{Being, ControlledBy, PlayerDirectControllable, TargetSpawnPos, };
use crate::game::being::modifier::modifier_components::ModifierCategories;
use crate::game::being::sprite::sprite_components::SpriteDatasChildrenStringIds;
use crate::game::faction::faction_components::{BelongsToFaction, Faction};
use crate::game::faction::faction_resources::FactionEntityMap;
use crate::game::game_components::*;
use crate::game::game_resources::*;
use crate::game::player::player_components::{CameraTarget, CreatedCharacter, HostPlayer, OfSelf, Player};
use crate::game::{SimulationState};

#[allow(unused_parens)]
pub fn placeholder_character_creation(mut cmd: Commands, mut query: Query<(),(With<Player>)>) {
}


pub fn sors_setup_initial_entities(mut cmd: Commands, mut fac_map: ResMut<FactionEntityMap>) {
    let fac_ent = Faction::new(&mut cmd, &mut fac_map, "host", "Host Faction", ());
    cmd.spawn((
        OfSelf, HostPlayer,
        DisplayName::new("host"),
        BelongsToFaction(fac_ent),
    ));
}

#[allow(unused_parens, )]
pub fn spawn_player_beings(
    mut commands: Commands,
    players: Query<(Entity, &CreatedCharacter, &BelongsToFaction, Option<&OfSelf>), (With<Player>)>,
) {

    for (player_ent, created_character, belongs_to_fac, self_player) in players.iter() {
        println!("Spawning player being: {:?}", created_character);

        commands.entity(created_character.0).insert((
            ControlledBy { player: player_ent },
            PlayerDirectControllable,
            TargetSpawnPos::new(0.0, 0.0),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),//PROVISORIO
            //HACER Q OTRO SYSTEMA AGREGUE CAMERATARGET AL BEING CONTROLADO
            SpriteDatasChildrenStringIds::new(["humanhe0", "humanbo0"]),
            belongs_to_fac.clone(),
        ));

        if self_player.is_some() {
            info!("Spawning self player being:");

        } 
        commands.entity(player_ent).remove::<CreatedCharacter>();
    }

}


#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands, 
    query: Query<(Entity, &DisplayName),(Added<DisplayName>, With<Player>)>) {
    
    for (player_ent, player_name) in query.iter() {
        let being = cmd.spawn((
            Being,
            DisplayName::new(player_name.0.clone()),
        )).id();

        cmd.entity(player_ent).insert((
            CreatedCharacter(being),
        ));
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




pub fn force_z_index(mut query: Query<(&mut Transform, &MyZ), (Changed<MyZ>,)>) {
    for (mut transform, z_index) in query.iter_mut() {
        transform.translation.z = z_index.div_1e9();
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
pub fn tick_time_based_multipliers(time: Res<Time>, mut query: Query<&mut TimeBasedMultiplier, Without<ModifierCategories>>) {
    for mut multiplier in query.iter_mut() { multiplier.timer.tick(time.delta()); }
}
