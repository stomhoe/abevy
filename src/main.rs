use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use bevy_simple_subsecond_system::SimpleSubsecondPlugin;
use tracing::Level;



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
            SimpleSubsecondPlugin::default(),
            host::plugin,
            client::plugin,
            common::plugin,
            tilemap::plugin,
            setup_screen::plugin,
            ui_shared::plugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape))
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


