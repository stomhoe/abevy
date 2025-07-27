use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
// use bevy_renet::{RenetClientPlugin, RenetServerPlugin};

use crate::common::common_components::{DisplayName, EntityPrefix};
use crate::game::GamePlugin;
use crate::pregame_menus::{MenuPlugin};
use crate::ui::MyUiPlugin;
mod game;
pub mod pregame_menus;
pub mod common;
pub mod ui;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {#[default]PreGame, StatefulGameSession, }

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            GamePlugin, 
            MenuPlugin, 
            MyUiPlugin,
            EguiPlugin::default(), WorldInspectorPlugin::new()
        ))
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, set_entity_name)
        .run()
    ;
}

pub fn spawn_camera(mut commands: Commands, window_query: Single<&Window, With<PrimaryWindow>>) {
    let _window = window_query;

    commands.spawn((
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform {
            ..default()
        },
    ));
}


#[allow(unused_parens)]
pub fn set_entity_name(mut cmd: Commands, mut query: Query<(Entity, &EntityPrefix, &DisplayName),(Changed<EntityPrefix>, Changed<DisplayName>)>) {
    for (ent, prefix, disp_name) in query.iter_mut() {
        let new_name = format!("{}('{}')", prefix, disp_name.0);
        cmd.entity(ent).insert(Name::new(new_name));
    }
}
