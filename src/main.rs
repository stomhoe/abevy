use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use tracing::Level;
#[allow(unused_imports)] use bevy::ecs::error::{panic, error, warn, };

pub const FILTER: &str = 
concat!(
    "info,",
    "naga=error,","wgpu_hal=error,",
    "bevy_ecs_tilemap=warn,",
    "wgpu_core=error,",
    "bevy_egui=warn,",
    "bevy_replicon=warn,",
    "bevy_render=warn,",
    "bevy_app=warn,",
    "cosmic_text=warn,",
    "offset_allocator=warn,",
    "bevy_asset_loader=warn,",
    "bevy_ecs::relationship=error,",
    "calloop::loop_logic=error,",
    
    "portal_init=debug,",
    

    "tilemap::terrain_gen::terrgen_systems=info,",
    "terrgen_process=info,",
    "structure_spawn=warn,",
    "tilemap_systems=warn,",
    "tilemap=warn,",
    "add2gposmap=warn,",

    "zlevel=info,",
    "movement=warn,",
    "sprite_init=warn,",
    "sprite_building=warn,",
    "sprite_systems=warn,",
    "sprite_systems=warn,",

    "being_control=warn,",
    "game_init_systems=info,",
    
    "sprite_animation_init=warn,",
    "entity_zero=info,",
    "dungeoning=warn,",
    "sgc_chunk_claim=warn,",

    "tile_init=info,",
    "asset_loading=warn,",
    "tiling_loading=warn,",
    "dimension_loading=warn,",
    "control=warn,",
    "being=warn,",
    "faction=warn,",
    "zsort=warn,",
    "inspector=warn,",
);
//Get-ChildItem target\debug -Recurse -Filter "tilemap*" | Remove-Item -Force

/*
cargo run --release --features bevy/trace_tracy,bevy/debug

/usr/local/bin/tracy-profiler


*/


fn main() {
    
    App::new()
        .set_error_handler(warn)
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
            sprite_shared::plugin,
            shader::plugin,
            debug::plugin,
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
            color_sample::plugin,
        ))
        .run()

    ;
}
/* ç
Log Levels
    Trace (Not printed by default) (PARA MENSAJES MUY SPAM)
    Debug (Not printed by default) (MENSAJES Q SON SOLO PARA DEBUGGEAR PERO NO SPAMMEAN)
    Info (default level that is printed)
    Warn
    Error
    None (you turned off logging)


Get-ChildItem target\debug -Recurse -File |
    Where-Object { $_.Name -like "tilemap*" -or $_.Name -like "libtilemap*" -or $_.Name -like "argentum_coop*" } |
    Remove-Item -Force

*/


