use bevy::asset::AssetServer;
use bevy::ecs::world::OnDespawn;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::window::PrimaryWindow;
use bevy::prelude::*;
use crate::common::common_components::GameZindex;
use crate::game::beings::beings_components::{Being, ControlledBySelf, PlayerDirectControllable};
use crate::game::factions::factions_components::SelfFaction;
use crate::game::game_resources::NidEntityMap;
use crate::game::player::player_components::{CameraTarget, Player};
use crate::game::{SimulationState};

pub fn spawn_player_beings(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();
    println!("Spawning player beings at window size");

    commands.spawn((
        Sprite {
            image: asset_server.load("textures\\wear\\moss_short_tunic_icon.png"),
            ..default()
        },
        Being,
        PlayerDirectControllable,
        ControlledBySelf,
        CameraTarget,
        Transform::from_translation(Vec3::new(
            window.width() / 2.0,
            window.height() / 2.0,
            0.0,
        )),
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


// pub fn host_on_entity_spawned(
//     mut commands: Commands,
//     new_being: Query<Entity, Added<Ser>>,
//     mut being_entity_map: ResMut<NidEntityMap>,
    
// ) {
//     for entity in new_being.iter() {
//         being_entity_map.new_entity(&mut commands, entity);
        
//         info!("New being spawned with entity: {:?}", entity);
//     }
// }