
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
use regioning::plugin as regioning_plugin;
use tilemap::tile::TilingSystems;
use tilemap_shared::GlobalTilePos;
use time::ClockPlugin;


#[derive(Resource, Default)]
struct InspectorVisibility(bool);

#[allow(non_upper_case_globals, )]
fn build_filter() -> String {
    const error: &str = "error";
    const warn: &str = "warn";
    const info: &str = "info";
    const debug: &str = "debug";
    const trace: &str = "trace";
    let mut filter = String::from("info");
    for (target, level) in [
        (log_targets::NAGA, error),
        (log_targets::WGPU_HAL, error),
        (log_targets::BEVY_ECS_TILEMAP, warn),
        (log_targets::WGPU_CORE, error),
        (log_targets::BEVY_EGUI, warn),
        (log_targets::BEVY_REPLICON, warn),
        (log_targets::BEVY_RENDER, warn),
        (log_targets::BEVY_APP, warn),
        (log_targets::COSMIC_TEXT, warn),
        (log_targets::OFFSET_ALLOCATOR, warn),
        (log_targets::BEVY_ASSET_LOADER, warn),
        (log_targets::BEVY_ECS_RELATIONSHIP, error),
        (log_targets::BEVY_ERROR_HANDLER, error),
        (log_targets::CALLOOP_LOOP_LOGIC, error),
        (log_targets::BEVY_W_INIT, warn),

        (log_targets::PORTAL_INIT, warn),
        (log_targets::POSITION_SEARCH, info),
        (log_targets::CHILDRENSPRITE_INIT, trace),
        (log_targets::OCCLUDER_INIT, info),
        (log_targets::BIOME_INIT, debug),

        (log_targets::OPLIST_INIT, warn),
        (log_targets::TERRGEN_INIT, info),
        (log_targets::TERRPROBE_INIT, debug),
        (log_targets::SGC_INIT, debug),
        (log_targets::REGION_SYSTEM, debug),
        (log_targets::SGC_CHUNK_OFFER, debug),
        (log_targets::SGC_CHUNK_CLAIM, warn),

        (log_targets::TERRGEN_SYSTEM, debug),
        (log_targets::MACROCHUNK_BIOME, trace),
        (log_targets::TERRGEN_PROCESS, info),
        (log_targets::STRUCTURE_SPAWN, info),
        (log_targets::TILEMAP_SYSTEM, info),
        (log_targets::GPOS_MAP, warn),
        (log_targets::CHUNK_DESPAWN, info),
        (log_targets::CHUNK_VISIBILITY, warn),
        (log_targets::CHUNK_ACTIVATION, debug),

        (log_targets::DEBUG, debug),

        (log_targets::MOVEMENT_SYSTEM, debug),
        (log_targets::SPRITE_SAMPLER_SYSTEM, warn),
        (log_targets::SPRITE_INIT, warn),
        (log_targets::SPRITE_BUILD, info),
        (log_targets::SPRITE_SYSTEM, warn),

        (log_targets::BEING_CONTROL, debug),
        (log_targets::BODY_BUILD, debug),
        (log_targets::BODY_HP_SYSTEM, trace),
        (log_targets::BODY_ENERGY_SYSTEM, debug),
        (log_targets::GAME_COMMON_SYSTEM, info),
        (log_targets::GAME_INIT, debug),
        (log_targets::LIGHTING_INIT, debug),

        (log_targets::SPRITE_ANIMATION_INIT, debug),
        (log_targets::SPRITE_ANIMATION_SYSTEM, debug),
        (log_targets::ENTITY_ZERO_SYSTEM, info),
        (log_targets::DUNGEONING_SYSTEM, debug),

        (log_targets::TILE_INIT, info),
        (log_targets::ASSET_LOADING, info),
        (log_targets::TILEMAP_LOAD, info),
        (log_targets::DIMENSION_LOADING, warn),
        (log_targets::DEF_VALIDATION, info),
        (log_targets::CONTROL, warn),
        (log_targets::BEING_TEMPLATE_INIT, info),
        (log_targets::BEING_BUILD, info),
        (log_targets::BEING_SYSTEM, debug),
        (log_targets::BEING_MELEE_SYSTEMS, debug),
        (log_targets::FACTION_SYSTEM, warn),
        (log_targets::ENTITY_MAP_SYSTEM, debug),
        (log_targets::INSPECTOR, warn),
        (log_targets::RIVER_SYSTEM, info),
        (log_targets::ITEM_SYSTEM, debug),
        (log_targets::WILDLIFE_SYSTEM, debug),
    ] {
        filter.push(',');
        filter.push_str(target);
        filter.push('=');
        filter.push_str(level);
    }
    filter
}

/*TRACING
cargo run -r --features bevy/trace_tracy,bevy/debug

tracy-profiler
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
            common_systems::plugin,
            ac_input::plugin,
            game_common::plugin,
            ui_shared::plugin,
            shader::plugin,
            debug::plugin,
        ))
        .add_plugins((
            game::plugin,
            being::plugin,
            player::player::plugin,
            faction::plugin,
            dimension::plugin,
            camera::plugin,
            sprite_animation::plugin,
            movement::plugin,
            sprite_systems::plugin,
            modifier_systems::plugin,
            item_systems::plugin,
            tilemap::plugin,
            regioning_plugin,
        ))
        .add_plugins((
            setup_screen::plugin,
            pregame_screen::plugin,
            color_sampler::plugin,
        ))
        .add_plugins((ClockPlugin, ))
        .configure_sets(
            OnEnter(AssetLoading::SpawnReplicatedEntities),(
            ItemSystems
                .before(TilingSystems)
                .after(AcSpriteSystems),
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

*/
