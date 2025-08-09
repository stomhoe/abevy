use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_simple_subsecond_system::SimpleSubsecondPlugin;
use tracing::Level;

use crate::{
    common::*,
    game::{multiplayer::MpPlugin, GamePlugin},
    pregame_menus::MenuPlugin,
    ui::MyUiPlugin,
};
mod game;
pub mod pregame_menus;
pub mod common;
pub mod ui;
#[allow(unused_imports)] use bevy::ecs::error::{panic, error, GLOBAL_ERROR_HANDLER, };


pub const FILTER: &str = 
concat!(
    "error,",
    "terrgen=warn,",
    "zlevel=warn,",
    "sprite_animation=trace,",
    "sprite_loading=trace,",
    "sprite_building=trace",
);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {#[default]PreGame, StatefulGameSession, }
fn main() {
    GLOBAL_ERROR_HANDLER.set(panic ).expect("Error handler can only be set once, globally.");
    
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(//https://bevy-logging.github.io/chapter_5.html
                LogPlugin {//sin espacios
                    filter: FILTER.to_string(),
                    level: Level::TRACE,
                    ..Default::default()
                })
            .set(ImagePlugin::default_nearest(),),
            
            MpPlugin,
            SimpleSubsecondPlugin::default(),
            CommonPlugin,
            GamePlugin, 
            MenuPlugin, 
            MyUiPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape))
        ))
        .init_state::<AppState>()
        .run()

    ;
}
/* 
Log Levels
    Trace (Not printed by default) (PARA MENSAJES MUY SPAM)
    Debug (Not printed by default) (MENSAJES Q SON SOLO PARA DEBUGGEAR PERO NO SPAMMEAN)
    Info (default level that is printed)
    Warn
    Error
    None (you turned off logging)
*/


