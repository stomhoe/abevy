use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_simple_subsecond_system::SimpleSubsecondPlugin;
use tracing::Level;



#[allow(unused_imports)] use bevy::ecs::error::{panic, error, warn, GLOBAL_ERROR_HANDLER, };

pub const FILTER: &str = 
concat!(
    "error,",
    "terrgen=warn,",
    "zlevel=warn,",
    "movement=warn,",
    "sprite_animation=warn,",
    "sprite_loading=trace,",
    "sprite_building=trace,",
    "asset_loading=warn,",
    "tiling_loading=warn,",
);


fn main() {
    GLOBAL_ERROR_HANDLER.set(warn).expect("Error handler can only be set once, globally.");
    
    App::new()
        .add_plugins((
            DefaultPlugins
            .set(//https://bevy-logging.github.io/chapter_5.html
                LogPlugin {
                    filter: FILTER.to_string(),
                    level: Level::TRACE,
                    ..Default::default()
                })
            .set(ImagePlugin::default_nearest(),),
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
            SimpleSubsecondPlugin::default(),
            multiplayer_shared::plugin,
        ))
        .add_plugins((
            host::plugin,
            client::plugin,
            common::plugin,
            asset_loading::plugin,
            game_common::plugin,
            game::plugin,
            camera::plugin,
            sprite_animation::plugin,
            movement::plugin,
            sprite::plugin,
            modifier::plugin,
            tilemap::plugin,
            setup_screen::plugin,
            pregame_screen::plugin,
            ui_shared::plugin,
            
        ))
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


