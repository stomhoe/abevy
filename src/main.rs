use bevy::{log::LogPlugin, prelude::*,};
use avian2d::prelude::PhysicsPlugins;
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use ac_input::ac_input_actions::ToggleInspectorAction;
use sprite_systems::AcSpriteSystems;
use tracing::Level;
#[allow(unused_imports)] use bevy::ecs::error::{panic, error, warn, };
use common::log_targets;
use common::common_states::AssetLoading;
use item_systems::ItemSystems;
use tilemap::prelude::TilingSystems;
use tilemap_shared::GlobalTilePos;

const ERROR: &str = "error";
const WARN: &str = "warn";
const INFO: &str = "info";
const DEBUG: &str = "debug";
const TRACE: &str = "trace";

#[derive(Resource, Default)]
struct InspectorVisibility(bool);

fn build_filter() -> String {
    format!(
        "info,\
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
        {}={},\
        {}={},\
        {}={},\
        {}={},",
        log_targets::NAGA, ERROR,
        log_targets::WGPU_HAL, ERROR,
        log_targets::BEVY_ECS_TILEMAP, WARN,
        log_targets::WGPU_CORE, ERROR,
        log_targets::BEVY_EGUI, WARN,
        log_targets::BEVY_REPLICON, WARN,
        log_targets::BEVY_RENDER, WARN,
        log_targets::BEVY_APP, WARN,
        log_targets::COSMIC_TEXT, WARN,
        log_targets::OFFSET_ALLOCATOR, WARN,
        log_targets::BEVY_ASSET_LOADER, WARN,
        log_targets::BEVY_ECS_RELATIONSHIP, ERROR,
        log_targets::CALLOOP_LOOP_LOGIC, ERROR,

        log_targets::PORTAL_INIT, WARN,
        log_targets::POSITION_SEARCH, INFO,
        log_targets::CHILDRENSPRITE_INIT, INFO,

        log_targets::OPLIST_INIT, WARN,
        log_targets::TERRGEN_INIT, WARN,
        log_targets::SGC_INIT, WARN,

        log_targets::TERRGEN_SYSTEM, INFO,
        log_targets::TERRGEN_PROCESS, INFO,
        log_targets::STRUCTURE_SPAWN, WARN,
        log_targets::TILEMAP_SYSTEM, INFO,
        log_targets::GPOS_MAP, WARN,

        log_targets::CHUNK_DESPAWN, INFO,

        log_targets::DEBUG, DEBUG,

        log_targets::Z_LEVEL_SYSTEM, INFO,
        log_targets::MOVEMENT_SYSTEM, TRACE,
        "sprite_sampler_systems", TRACE,
        log_targets::SPRITE_INIT, WARN,
        log_targets::SPRITE_BUILD, INFO,
        log_targets::SPRITE_SYSTEM, WARN,

        log_targets::BEING_CONTROL, INFO,
        log_targets::GAME_INIT, DEBUG,

        log_targets::SPRITE_ANIMATION_INIT, WARN,
        log_targets::SPRITE_ANIMATION_SYSTEM, WARN,
        log_targets::ENTITY_ZERO_SYSTEM, INFO,
        log_targets::DUNGEONING_SYSTEM, WARN,
        log_targets::SGC_CHUNK_CLAIM, WARN,

        log_targets::TILE_INIT, INFO,
        log_targets::ASSET_LOAD, WARN,
        log_targets::TILEMAP_LOAD, WARN,
        log_targets::DIMENSION_LOAD, WARN,
        log_targets::CONTROL, WARN,
        log_targets::BEING_SYSTEM, INFO,
        log_targets::FACTION_SYSTEM, WARN,
        log_targets::Z_SORT_SYSTEM, INFO,
        log_targets::ENTITY_MAP_SYSTEM, WARN,
        log_targets::INSPECTOR, WARN,
        log_targets::RIVER_SYSTEM, WARN,
        log_targets::ITEM_SYSTEM, INFO,
    )
}
//Get-ChildItem target\debug -Recurse -Filter "tilemap*" | Remove-Item -Force

/*TRACING
cargo run -r --features bevy/trace_tracy,bevy/debug

/usr/local/bin/tracy-profiler
*/

fn main() {

    App::new()
        .init_resource::<InspectorVisibility>()
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
            WorldInspectorPlugin::default().run_if(|visible: Res<InspectorVisibility>| visible.0),
            PhysicsPlugins::default().with_length_unit(GlobalTilePos::TILE_SIZE_PXS.x as f32),
        ))
        .add_systems(Update, toggle_inspector_visibility)
        .add_plugins((
            multiplayer_shared::plugin, //VA ARRIBA
            host::plugin,
            client::plugin,
            ac_audio::plugin,
        ))
        .add_plugins((
            asset_loading::plugin,
            common::plugin,
            ac_input::plugin,
            game_common::plugin,
            ui_shared::plugin,
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
            sprite_systems::plugin,
            modifier_systems::plugin,
            item_systems::plugin,
            tilemap::plugin,
            setup_screen::plugin,
            pregame_screen::plugin,
            color_sampler::plugin,
        ))
        .configure_sets(
            OnEnter(AssetLoading::SpawnReplicatedEntities),(
            ItemSystems.before(TilingSystems).after(AcSpriteSystems),
            ))
        .add_plugins((wildlife::plugin,))
        .run()

    ;
}

fn toggle_inspector_visibility(
    toggle_events: Single<&ActionEvents, With<Action<ToggleInspectorAction>>>,
    mut visible: ResMut<InspectorVisibility>,
) {
    if toggle_events.contains(ActionEvents::START) {
        visible.0 = !visible.0;
    }
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
