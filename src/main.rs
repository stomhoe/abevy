use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
// use bevy_renet::{RenetClientPlugin, RenetServerPlugin};

use crate::common::*;
use crate::game::multiplayer::MpPlugin;
use crate::game::GamePlugin;
use crate::pregame_menus::{MenuPlugin};
use crate::ui::MyUiPlugin;
mod game;
pub mod pregame_menus;
pub mod common;
pub mod ui;
#[allow(unused_imports)] use bevy::ecs::error::{error, panic, GLOBAL_ERROR_HANDLER, };

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {#[default]PreGame, StatefulGameSession, }
fn main() {
    GLOBAL_ERROR_HANDLER.set(panic ).expect("The error handler can only be set once, globally.");
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MpPlugin,
            CommonPlugin,
            GamePlugin, 
            MenuPlugin, 
            MyUiPlugin,
            EguiPlugin::default(), WorldInspectorPlugin::new()
        ))
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        //.add_systems(Update, (, ))
        .run()
    ;
}

pub fn spawn_camera(mut commands: Commands, window_query: Single<&Window, With<PrimaryWindow>>) {
    let _window = window_query;

    commands.spawn((Camera2d::default(), Camera {hdr: true, ..default()}, Transform::default(),));
}


