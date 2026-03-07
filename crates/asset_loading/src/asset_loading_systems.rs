use bevy::prelude::*;
use bevy_replicon::prelude::{ServerState};
use common::{common_components::{AssetScoped, HotReload}, common_states::*};
use being::race::race_components::Race;
use being::sex::sex_components::Sex;
use being_shared::BeingInstTemplate;
use sprite::sprite_components::SpriteConfig;
use sprite_animation_shared::sprite_animation_components::AcAnimation;
use tilemap_shared::GlobalGenSettings;

use tilemap_shared::ForceAllChunksDespawn;
use tilemap::{
    tile::tile_components::Tile,
    terrain::{
        terrprobe::opfilter::opfilter_components::OpFilter,
        operation_list::operation_list_components::OperationList,
        terrgen_components::FnlNoiseComp,
        terrprobe::terrprobe_components::TerrProbeTempl,
    },
};


#[allow(unused_parens, )]
pub fn reload_assets_while_ingame(
    keys: Res<ButtonInput<KeyCode>>,
    mut hot_reload_request: ResMut<HotReloadRequest>,
    client_state: Res<State<ServerState>>,
) {
    if keys.pressed(KeyCode::F6) {
        if *client_state.get() != ServerState::Running {
            warn!(target: "asset_loading", "You cannot hot-reload assets as a client.");
            return;
        }
        hot_reload_request.requested = true;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn process_hot_reload_request(
    mut hot_reload_request: ResMut<HotReloadRequest>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
    mut regpos: ResMut<tilemap::tilemap_resources::ImportantRegisteredPositions>,
    mut force_all_chunks_despawn_writer: MessageWriter<ForceAllChunksDespawn>,
    client_state: Res<State<ServerState>>,
) {
    if !hot_reload_request.requested {
        return;
    }
    if *client_state.get() != ServerState::Running {
        warn!(target: "asset_loading", "You cannot hot-reload assets as a client.");
        hot_reload_request.requested = false;
        return;
    }
    info!(target: "asset_loading", "Reloading hot-reloadable entities...");
    hot_loading.set(AssetHotReloadState::Ongoing);
    force_all_chunks_despawn_writer.write_default();
    regpos.clear();
    loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
    hot_reload_request.requested = false;
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
) {
    hot_loading.set(AssetHotReloadState::Stopped);
}

pub fn validate_defs_after_load(
    mut runtime: ResMut<common::def_db::DefValidationRuntime>,
    config: Res<common::def_db::DefValidationConfig>,
) {
    if !config.enabled || runtime.completed {
        return;
    }
    if !common::def_db::expected_types_loaded() {
        return;
    }

    runtime.attempted = true;
    match common::def_db::validate_global_registry() {
        Ok(_) => {
            info!(target: "def_validation", "Def validation passed");
            runtime.completed = true;
        }
        Err(err) => {
            error!(target: "def_validation", "{err:#}");
            if config.fail_fast {
                panic!("Def validation failed, aborting startup");
            }
            runtime.completed = true;
        }
    }
}


pub fn despawn_asset_scoped_entities(
    mut commands: Commands,
    query: Query<Entity, (With<AssetScoped>, common::AnyDisabling)>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

pub fn despawn_asset_scoped_entities_except_spared(
    mut commands: Commands,
    query: Query<Entity, (With<AssetScoped>, With<HotReload>, common::AnyDisabling)>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_hot_reload_markers(
    mut commands: Commands,
    selection: Res<HotReloadSelection>,
    tiles: Query<Entity, (With<AssetScoped>, With<Tile>, common::AnyDisabling)>,
    sprite_configs: Query<Entity, (With<AssetScoped>, With<SpriteConfig>, common::AnyDisabling)>,
    animations: Query<Entity, (With<AssetScoped>, With<AcAnimation>, common::AnyDisabling)>,
    operation_lists: Query<Entity, (With<AssetScoped>, With<OperationList>, common::AnyDisabling)>,
    noises: Query<Entity, (With<AssetScoped>, With<FnlNoiseComp>, common::AnyDisabling)>,
    probes: Query<Entity, (With<AssetScoped>, With<TerrProbeTempl>, common::AnyDisabling)>,
    filters: Query<Entity, (With<AssetScoped>, With<OpFilter>, common::AnyDisabling)>,
    terrgen_settings: Query<Entity, (With<AssetScoped>, With<GlobalGenSettings>, common::AnyDisabling)>,
    beings_inst_templates: Query<Entity, (With<AssetScoped>, With<BeingInstTemplate>, common::AnyDisabling)>,
    races: Query<Entity, (With<AssetScoped>, With<Race>, common::AnyDisabling)>,
    sexes: Query<Entity, (With<AssetScoped>, With<Sex>, common::AnyDisabling)>,
) {
    for entity in &tiles {
        if selection.tiles { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &sprite_configs {
        if selection.sprite_configs_and_animations { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &animations {
        if selection.sprite_configs_and_animations { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &operation_lists {
        if selection.terrain_oplists_and_noises { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &noises {
        if selection.terrain_oplists_and_noises { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &probes {
        if selection.probes_and_filters { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &filters {
        if selection.probes_and_filters { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &terrgen_settings {
        if selection.terrgen_settings { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &beings_inst_templates {
        if selection.beings_inst_templates { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &races {
        if selection.races { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
    for entity in &sexes {
        if selection.sexes { commands.entity(entity).try_insert(HotReload); }
        else { commands.entity(entity).remove::<HotReload>(); }
    }
}
