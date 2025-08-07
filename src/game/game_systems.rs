use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::*;
use crate::common::common_components::{DisplayName, MyZ};
use crate::game::being::being_components::{Being, ControlledBy, PlayerDirectControllable, TargetSpawnPos, };
use crate::game::being::modifier::modifier_components::ModifierCategories;
use crate::game::being::sprite::sprite_components::SpriteConfigStringIds;
use crate::game::faction::faction_components::{BelongsToFaction, Faction};
use crate::game::faction::faction_resources::FactionEntityMap;
use crate::game::tilemap::terrain_gen::terrgen_resources::{OpListEntityMap, TerrGenEntityMap};
use crate::game::{game_components::*, ReplicatedAssetsLoadingState};
use crate::game::game_resources::*;
use crate::game::player::player_components::{CreatedCharacter, HostPlayer, OfSelf, Player};
use crate::game::{SimulationState};

#[allow(unused_parens)]
pub fn placeholder_character_creation(cmd: Commands, query: Query<(),(With<Player>)>) {
}

#[allow(unused_parens)]
pub fn remove_server_resources(mut cmd: Commands, ) {
    cmd.remove_resource::<TerrGenEntityMap>();
    cmd.remove_resource::<OpListEntityMap>();


}

pub fn server_or_singleplayer_setup(mut cmd: Commands, 
    mut assets_loading_state: ResMut<NextState<ReplicatedAssetsLoadingState>>,
    mut fac_map: ResMut<FactionEntityMap>)
{
    assets_loading_state.set(ReplicatedAssetsLoadingState::InProcess);

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
            SpriteConfigStringIds::new(["humanhe0", "humanbo0"]),
            belongs_to_fac.clone(),
        ));

        if self_player.is_some() {
            info!("Spawning self player being:");

        } 
        commands.entity(player_ent).remove::<CreatedCharacter>();
    }

}


#[allow(unused_parens)]
pub fn host_on_player_added(mut cmd: Commands, query: Query<(Entity, &DisplayName),(Added<DisplayName>, With<Player>)>) {
    for (player_ent, player_name) in query.iter() {
        let being = cmd.spawn((Being, DisplayName::new(player_name.0.clone()),)).id();
        cmd.entity(player_ent).insert((CreatedCharacter(being),));
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

pub fn update_transform_z(mut query: Query<(&mut Transform, &MyZ), (Changed<MyZ>,)>) {
    for (mut transform, z_index) in query.iter_mut() { transform.translation.z = z_index.div_1e9(); }
}


pub fn tick_time_based_multipliers(time: Res<Time>, mut query: Query<&mut TimeBasedMultiplier, Without<ModifierCategories>>) {
    for mut multiplier in query.iter_mut() { multiplier.timer.tick(time.delta()); }
}


pub fn update_img_sizes_on_load(mut events: EventReader<AssetEvent<Image>>, assets: Res<Assets<Image>>, mut map: ResMut<ImageSizeMap>,) {
    for ev in events.read() {
        match ev {
            AssetEvent::Added { id } => {
                if let Some(img) = assets.get(*id) {
                    let img_size = UVec2::new(img.texture_descriptor.size.width, img.texture_descriptor.size.height);
                    map.0.insert(Handle::Weak(id.clone()), img_size.as_u16vec2());
                }
            },
            _ => {}
        }
    }
}