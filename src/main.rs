use bevy::{input::common_conditions::input_toggle_active, log::LogPlugin, prelude::*,};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use tracing::Level;
#[allow(unused_imports)] use bevy::ecs::error::{panic, error, warn, };
use common::log_targets;

fn build_filter() -> String {
    format!(
        "info,\
        {}=error,{}=error,\
        {}=warn,\
        {}=error,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=error,\
        {}=error,\
        \
        {}=warn,\
        {}=info,\
        {}=info,\
        \
        {}=warn,\
        {}=warn,\
        {}=warn,\
        \
        {}=info,\
        {}=info,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        \
        {}=info,\
        \
        {}=info,\
        \
        {}=info,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        \
        {}=warn,\
        {}=info,\
        \
        {}=warn,\
        {}=info,\
        {}=warn,\
        {}=warn,\
        \
        {}=info,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,\
        {}=warn,",
        log_targets::NAGA,
        log_targets::WGPU_HAL,
        log_targets::BEVY_ECS_TILEMAP,
        log_targets::WGPU_CORE,
        log_targets::BEVY_EGUI,
        log_targets::BEVY_REPLICON,
        log_targets::BEVY_RENDER,
        log_targets::BEVY_APP,
        log_targets::COSMIC_TEXT,
        log_targets::OFFSET_ALLOCATOR,
        log_targets::BEVY_ASSET_LOADER,
        log_targets::BEVY_ECS_RELATIONSHIP,
        log_targets::CALLOOP_LOOP_LOGIC,
        
        log_targets::PORTAL_INIT,
        log_targets::POSITION_SEARCH,
        log_targets::CHILDRENSPRITE_INIT,

        log_targets::OPLIST_INIT,
        log_targets::TERRGEN_INIT,
        log_targets::SGC_INIT,

        log_targets::TERRGEN_SYSTEM,
        log_targets::TERRGEN_PROCESS,
        log_targets::STRUCTURE_SPAWN,
        log_targets::TILEMAP_SYSTEM,
        log_targets::TILEMAP_SYSTEM,
        log_targets::GPOS_MAP,

        log_targets::CHUNK_DESPAWN,

        log_targets::DEBUG,

        log_targets::Z_LEVEL_SYSTEM,
        log_targets::MOVEMENT_SYSTEM,
        log_targets::SPRITE_INIT,
        log_targets::SPRITE_BUILD,
        log_targets::SPRITE_SYSTEM,
        log_targets::SPRITE_SYSTEM,

        log_targets::BEING_CONTROL,
        log_targets::GAME_INIT,
        
        log_targets::SPRITE_ANIMATION_INIT,
        log_targets::ENTITY_ZERO_SYSTEM,
        log_targets::DUNGEONING_SYSTEM,
        log_targets::SGC_CHUNK_CLAIM,

        log_targets::TILE_INIT,
        log_targets::ASSET_LOAD,
        log_targets::TILEMAP_LOAD,
        log_targets::DIMENSION_LOAD,
        log_targets::CONTROL,
        log_targets::BEING_SYSTEM,
        log_targets::FACTION_SYSTEM,
        log_targets::Z_SORT_SYSTEM,
        log_targets::INSPECTOR,
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


