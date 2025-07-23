use bevy::prelude::*;
use bevy::window::PrimaryWindow;
// use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
// use bevy_renet::{RenetClientPlugin, RenetServerPlugin};

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
            DefaultPlugins, 
            //bevy::remote::RemotePlugin::default(), bevy::remote::http::RemoteHttpPlugin::default(),
            GamePlugin, 
            MenuPlugin, 
            MyUiPlugin,
            
        ))
      
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
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
