
use bevy::{log::LogPlugin, prelude::*,};
use avian2d::prelude::PhysicsPlugins;
#[allow(unused_imports, )] use bevy_fps_counter::FpsCounterPlugin;
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
    #[allow(unused)]const error: &str = "error";
    #[allow(unused)]const warn: &str = "warn";
    #[allow(unused)]const info: &str = "info";
    #[allow(unused)]const debug: &str = "debug";
    #[allow(unused)]const trace: &str = "trace";
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
        (log_targets::CHILDRENSPRITE_INIT, info),
        (log_targets::OCCLUDER_INIT, info),
        (log_targets::BIOME_INIT, warn),

        (log_targets::OPLIST_INIT, warn),
        (log_targets::TERRGEN_INIT, info),
        (log_targets::TERRPROBE_INIT, warn),
        (log_targets::SGC_INIT, warn),
        (log_targets::REGION_SYSTEM, warn),
        (log_targets::SGC_CHUNK_OFFER, warn),
        (log_targets::SGC_CHUNK_CLAIM, warn),

        (log_targets::TERRGEN_SYSTEM, warn),
        (log_targets::MACROCHUNK_BIOME, warn),
        (log_targets::TERRGEN_PROCESS, info),
        (log_targets::STRUCTURE_SPAWN, info),
        (log_targets::TILEMAP_SYSTEM, info),
        (log_targets::GPOS_MAP, warn),
        (log_targets::CHUNK_DESPAWN, info),
        (log_targets::CHUNK_VISIBILITY, warn),
        (log_targets::CHUNK_ACTIVATION, warn),

        (log_targets::DEBUG, warn),

        (log_targets::MOVEMENT_SYSTEM, warn),
        (log_targets::SPRITE_SAMPLER_SYSTEM, warn),
        (log_targets::SPRITE_INIT, warn),
        (log_targets::SPRITE_BUILD, info),
        (log_targets::SPRITE_SYSTEM, warn),

        (log_targets::BEING_CONTROL, warn),
        (log_targets::BODY_BUILD, warn),
        (log_targets::BODY_HP_SYSTEM, warn),
        (log_targets::BODY_ENERGY_SYSTEM, warn),
        (log_targets::GAME_COMMON_SYSTEM, info),
        (log_targets::GAME_INIT, warn),
        (log_targets::LIGHTING_INIT, warn),

        (log_targets::SPRITE_ANIMATION_INIT, warn),
        (log_targets::SPRITE_ANIMATION_SYSTEM, warn),
        (log_targets::ENTITY_ZERO_SYSTEM, info),
        (log_targets::DUNGEONING_SYSTEM, warn),

        (log_targets::TILE_INIT, info),
        (log_targets::ASSET_LOADING, info),
        (log_targets::TILEMAP_LOAD, info),
        (log_targets::DIMENSION_LOADING, warn),
        (log_targets::DEF_VALIDATION, info),
        (log_targets::CONTROL, warn),
        (log_targets::BEING_TEMPLATE_INIT, info),
        (log_targets::BEING_BUILD, info),
        (log_targets::BEING_SYSTEM, warn),
        (log_targets::BEING_MELEE_SYSTEMS, warn),
        (log_targets::FACTION_SYSTEM, warn),
        (log_targets::ENTITY_MAP_SYSTEM, warn),
        (log_targets::INSPECTOR, warn),
        (log_targets::RIVER_SYSTEM, warn),
        (log_targets::RIVER_BUILD_SYSTEM, warn),
        (log_targets::ITEM_SYSTEM, warn),
        (log_targets::WILDLIFE_SYSTEM, warn),
    ] {
        filter.push(',');
        filter.push_str(target);
        filter.push('=');
        filter.push_str(level);
    }
    filter
}

/*TRACING
cargo run -r --features bevy/warn_tracy,bevy/debug

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
                    level: Level::WARN,
                    ..Default::default()
                })
            .set(ImagePlugin::default_nearest(),)
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // Immediate = No VSync, uncapped fps
                    // Mailbox = Fast VSync, low latency
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }
        )   
            ,
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(|visible: Res<InspectorVisibility>| visible.0),
            PhysicsPlugins::default().with_length_unit(GlobalTilePos::TILE_SIZE_PXS.x as f32),
            //FpsCounterPlugin,
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
    warn (Not printed by default) (PARA MENSAJES MUY SPAM)
    debug (Not printed by default) (MENSAJES Q SON SOLO PARA DEBUGGEAR PERO NO SPAMMEAN)
    info (default level that is printed) (MENSAJES IMPORTANTES QUE NO SPAMMEAN NI SON ERRORES)
    warn (ADVERTENCIAS, el programa funciona bien pero te avisa que algo podría estar mal)
    error (ERRORES, algo claramente está mal)
    None (you turned off logging)

*/
