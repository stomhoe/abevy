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
    "warn,",
    "naga=error,",
    "wgpu_hal=error,",
    "bevy_ecs_tilemap=warn,",
    "wgpu_core=error,",
    "bevy_egui=warn,",
    "bevy_replicon=warn,",
    "bevy_render=warn,",
    "bevy_app=warn,",
    "cosmic_text=warn,",
    "offset_allocator=warn,",
    "bevy_asset_loader=warn,",

    "tilemap::terrain_gen::terrgen_systems=info,",
    "zlevel=warn,",
    "movement=warn,",
    "sprite_animation=warn,",
    "sprite_loading=trace,",
    "sprite_building=trace,",
    "sprite_systems=debug,",
    "asset_loading=warn,",
    "tiling_loading=debug,",
    "dimension_loading=debug,",
    "control=debug,",
    "being=debug,",
    "faction=debug,",
);
//Get-ChildItem target\debug -Recurse -Filter "tilemap*" | Remove-Item -Force

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
        ))
        .add_plugins((
            multiplayer_shared::plugin, //VA ARRIBA    
            host::plugin,
            client::plugin,
        ))
        .add_plugins((
            asset_loading::plugin,
            common::plugin,
            game_common::plugin,
            ui_shared::plugin,
        ))
        .add_plugins((
            game::plugin,
            being::plugin,
            player::plugin,
            faction::plugin,
            dimension::plugin,
            camera::plugin,
            sprite_animation::plugin,
            movement::plugin,
            sprite::plugin,
            modifier::plugin,
            tilemap::plugin,
            setup_screen::plugin,
            pregame_screen::plugin,
            
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


