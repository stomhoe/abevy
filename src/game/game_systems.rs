use bevy::asset::AssetServer;
use bevy::ecs::world::OnDespawn;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::common::common_components::GameZindex;
use crate::game::beings::beings_components::{Being, ControlledBySelf};
use crate::game::beings::beings_resources::BeingEntityMap;
use crate::game::factions::factions_components::SelfFaction;
use crate::game::player::player_components::{CameraTarget, Player};
use crate::game::{SimulationState};

pub fn spawn_player_beings(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut bmap: ResMut<BeingEntityMap>,
) {
    let window = window_query.single().unwrap();
    println!("Spawning player beings at window size");

    bmap.new_being(&mut commands, (
        Sprite {
            image: asset_server.load("textures\\wear\\moss_short_tunic_icon.png"),
            ..default()
        },
        ControlledBySelf,
        SelfFaction(),
    ));

    bmap.new_being(&mut commands, (
        Sprite {
            image: asset_server.load("textures\\wear\\moss_short_tunic_icon.png"),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            window.width() ,
            window.height(),
            0.0,
        )),
        ControlledBySelf,
        SelfFaction(),
    ));

       
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


#[derive(Component, Debug,)]
pub struct Bullet();

#[derive(Component, Debug,)]
pub struct Health(pub i32,);//SOLO PARA ENEMIGOS ULTRA BÁSICOS SIN CUERPO (GRUNTS IRRECLUTABLES PARA FARMEAR XP O LOOT)

#[derive(Component, Debug,)]
pub struct PhysicallyImmune();

#[derive(Component, Debug,)]
pub struct MagicallyInvulnerable();


pub fn force_z_index(mut query: Query<(&mut Transform, &GameZindex)>,) {
    for (mut transform, z_index) in query.iter_mut() {
        transform.translation.z = (z_index.0) as f32;
        //println!("transform {}", transform.translation);
    }
}


fn hit_detection(
    mut commands: Commands,
    being_query: Query<(Entity, &Transform), (Without<PhysicallyImmune>, With<Health>)>,
    bullet_query: Query<&Transform, With<Bullet>>
) {
    for (entity, enemy_transform) in being_query.iter() {
        for bullet_transform in bullet_query.iter() {
            // Your collision check
            if false {
                commands.entity(entity).despawn();
            }
        }
    }
}


pub fn debug_system(mut commands: Commands, query: Query<(Entity, &Transform), With<Being>>, cam_query: Query<&Transform, With<Camera>>) {
   
    
}


// pub fn HandleCollisions(mut spriteschange: Query<(&mut Transform, &Meta), With<Sprite>>)
// {
//     let mut combos = spriteschange.iter_combinations_mut();
//     while let Some([(mut trans1, mut meta1), (mut trans2, mut meta2)]) = combos.fetch_next() {
//         if(meta1.Id != meta2.Id){
//             let collision = collide(
//                     trans1.translation,
//                     trans1.scale.truncate(),
//                     trans2.translation,
//                     trans2.scale.truncate()
//                 );
//             if let Some(collision) = collision {
//                 trans1.translation.x += 256.;
//                 trans1.translation.y += 256.;
//                 println!("There was a collision!");
//             }
//         }

//     }

// }