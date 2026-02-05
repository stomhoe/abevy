use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use avian2d::prelude::PhysicsPlugins;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use tracing::Level;
#[allow(unused_imports)] use bevy::ecs::error::{panic, error, warn, };
use common::log_targets;
use tilemap_shared::GlobalTilePos;

const LOG_ERROR: &str = "error";
const LOG_WARN: &str = "warn";
const LOG_INFO: &str = "info";
const LOG_DEBUG: &str = "debug";

fn build_filter() -> String {
    format!(
        "info,\
        {}={},{}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        \
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        \
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},\
        {}={},",
        log_targets::NAGA, LOG_ERROR,
        log_targets::WGPU_HAL, LOG_ERROR,
        log_targets::BEVY_ECS_TILEMAP, LOG_WARN,
        log_targets::WGPU_CORE, LOG_ERROR,
        log_targets::BEVY_EGUI, LOG_WARN,
        log_targets::BEVY_REPLICON, LOG_WARN,
        log_targets::BEVY_RENDER, LOG_WARN,
        log_targets::BEVY_APP, LOG_WARN,
        log_targets::COSMIC_TEXT, LOG_WARN,
        log_targets::OFFSET_ALLOCATOR, LOG_WARN,
        log_targets::BEVY_ASSET_LOADER, LOG_WARN,
        log_targets::BEVY_ECS_RELATIONSHIP, LOG_ERROR,
        log_targets::CALLOOP_LOOP_LOGIC, LOG_ERROR,
        
        log_targets::PORTAL_INIT, LOG_WARN,
        log_targets::POSITION_SEARCH, LOG_INFO,
        log_targets::CHILDRENSPRITE_INIT, LOG_INFO,

        log_targets::OPLIST_INIT, LOG_WARN,
        log_targets::TERRGEN_INIT, LOG_WARN,
        log_targets::SGC_INIT, LOG_WARN,

        log_targets::TERRGEN_SYSTEM, LOG_INFO,
        log_targets::TERRGEN_PROCESS, LOG_INFO,
        log_targets::STRUCTURE_SPAWN, LOG_WARN,
        log_targets::TILEMAP_SYSTEM, LOG_WARN,
        log_targets::TILEMAP_SYSTEM, LOG_WARN,
        log_targets::GPOS_MAP, LOG_WARN,

        log_targets::CHUNK_DESPAWN, LOG_INFO,

        log_targets::DEBUG, LOG_INFO,

        log_targets::Z_LEVEL_SYSTEM, LOG_INFO,
        log_targets::MOVEMENT_SYSTEM, LOG_WARN,
        "grid_movement", LOG_DEBUG,
        log_targets::SPRITE_INIT, LOG_WARN,
        log_targets::SPRITE_BUILD, LOG_INFO,
        log_targets::SPRITE_SYSTEM, LOG_WARN,
        log_targets::SPRITE_SYSTEM, LOG_WARN,

        log_targets::BEING_CONTROL, LOG_INFO,
        log_targets::GAME_INIT, LOG_WARN,
        
        log_targets::SPRITE_ANIMATION_INIT, LOG_WARN,
        log_targets::ENTITY_ZERO_SYSTEM, LOG_INFO,
        log_targets::DUNGEONING_SYSTEM, LOG_WARN,
        log_targets::SGC_CHUNK_CLAIM, LOG_WARN,

        log_targets::TILE_INIT, LOG_INFO,
        log_targets::ASSET_LOAD, LOG_WARN,
        log_targets::TILEMAP_LOAD, LOG_WARN,
        log_targets::DIMENSION_LOAD, LOG_WARN,
        log_targets::CONTROL, LOG_WARN,
        log_targets::BEING_SYSTEM, LOG_WARN,
        log_targets::FACTION_SYSTEM, LOG_WARN,
        log_targets::Z_SORT_SYSTEM, LOG_WARN,
        log_targets::INSPECTOR, LOG_WARN,
    )
}
//Get-ChildItem target\debug -Recurse -Filter "tilemap*" | Remove-Item -Force

/*TRACING
cargo run -r --features bevy/trace_tracy,bevy/debug

/usr/local/bin/tracy-profiler
*/

fn main() {
    
    App::new()
        .set_error_handler(warn)
        .add_plugins((
            DefaultPlugins
            .set(//https://bevy-logging.github.io/chapter_5.html
                LogPlugin {
                    filter: build_filter(),
                    level: Level::TRACE,
                    ..Default::default()
                })
            .set(ImagePlugin::default_nearest(),),
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
            PhysicsPlugins::default().with_length_unit(GlobalTilePos::TILE_SIZE_PXS.x as f32),
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
            color_sampler::plugin,
        ))
        .run()

    ;
}
/*
Log Levels
    trace (Not printed by default) (PARA MENSAJES MUY SPAM)
    debug (Not printed by default) (MENSAJES Q SON SOLO PARA DEBUGGEAR PERO NO SPAMMEAN)
    info (default level that is printed) (MENSAJES IMPORTANTES QUE NO SPAMMEAN NI SON ERRORES)
    warn (ADVERTENCIAS, el programa funciona bien pero te avisa que algo podría estar mal)
    error (ERRORES, algo claramente está mal)
    None (you turned off logging)


Get-ChildItem target\debug -Recurse -File |
    Where-Object { $_.Name -like "tilemap*" -or $_.Name -like "libtilemap*" -or $_.Name -like "argentum_coop*" } |
    Remove-Item -Force

*/


