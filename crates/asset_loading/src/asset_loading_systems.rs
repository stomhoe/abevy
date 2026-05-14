use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_replicon::prelude::{ServerState};
use ac_input::ac_input_actions::AssetReloadAction;
use ::common::*;
use being::sex::sex_components::Sex;
use ::being_shared::*;
use ::sprite_shared::{BaseHolderRef, SpriteConfig};
use sprite_systems::SpriteConfigEntityMap;
use sprite_animation_shared::sprite_animation_components::AcAnimation;

use ::tilemap_shared::*;
use tilemap::{
    tile::tile_components::Tile,
    terrain::{
        terrprobe::opfilter::opfilter_components::OpFilter,
        operation_list::operation_list_components::OperationList,
        terrgen_components::FnlNoiseComp,
        terrprobe::terrprobe_components::TerrProbeTempl,
    },
};

#[derive(Resource, Default)]
pub struct ChangeAssetLoadingStateToFinishedTimer(pub Timer);



#[allow(unused_parens, )]
pub fn reload_assets_while_ingame(
    mut cmd: Commands,
    asset_reload: Single<&Action<AssetReloadAction>>,
    client_state: Res<State<ServerState>>,
) {
    if !***asset_reload {
        return;
    }

    if *client_state.get() != ServerState::Running {
        warn!(target: "asset_loading", "You cannot hot-reload assets as a client.");
        return;
    }
    cmd.trigger(HotReloadRequest);
}

#[allow(clippy::too_many_arguments)]
pub fn process_hot_reload_request(
    _: On<HotReloadRequest>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
    mut regpos: ResMut<tilemap_shared::ImportantRegisteredPositions>,
    mut force_all_chunks_despawn_writer: MessageWriter<ForceAllChunksDespawn>,
    client_state: Res<State<ServerState>>,
) {
    if *client_state.get() != ServerState::Running {
        warn!(target: "asset_loading", "You cannot hot-reload assets as a client.");
        return;
    }
    info!(target: "asset_loading", "Reloading hot-reloadable entities...");
    hot_loading.set(AssetHotReloadState::Ongoing);
    force_all_chunks_despawn_writer.write_default();
    regpos.clear();
    loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
) {
    hot_loading.set(AssetHotReloadState::Stopped);
}

#[allow(unused_parens)]
pub fn remap_broken_sprite_config_refs_after_hotreload(
    mut cmd: Commands,
    sprites_query: Query<(Entity, &TemplEntiRef), (Without<SpriteConfig>, With<BaseHolderRef>)>,
    str_id_query: Query<&StrId>,
    sprite_map: Res<SpriteConfigEntityMap>,
) {
    for (sprite_ent, templ_ref) in sprites_query.iter() {
        let Ok(sprite_id) = str_id_query.get(templ_ref.0) else {
            continue;
        };
        let Ok(new_cfg_ent) = sprite_map.0.get_cloned(&sprite_id) else {
            continue;
        };
        if new_cfg_ent != templ_ref.0 {
            cmd.entity(sprite_ent).insert(TemplEntiRef(new_cfg_ent));
        }
    }
}

pub fn validate_defs_after_load(
    config: Res<DefValidationConfig>,
) {
    if !config.enabled {
        return;
    }
    if !expected_types_loaded() {
        return;
    }

    match validate_global_registry() {
        Ok(_) => {
            info!(target: "def_validation", "Def validation passed");
        }
        Err(err) => {
            error!(target: "def_validation", "{err:#}");
            if config.fail_fast {
                panic!("Def validation failed, aborting startup");
            }
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

pub fn despawn_selected_asset_scoped_entities(
    mut commands: Commands,
    query: Query<Entity, (With<AssetScoped>, With<SelectedForHotReload>, common::AnyDisabling)>,
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
        if selection.tiles { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &sprite_configs {
        if selection.sprite_configs_and_animations { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &animations {
        if selection.sprite_configs_and_animations { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &operation_lists {
        if selection.terrain_oplists_and_noises { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &noises {
        if selection.terrain_oplists_and_noises { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &probes {
        if selection.probes_and_filters { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &filters {
        if selection.probes_and_filters { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &terrgen_settings {
        if selection.terrgen_settings { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &beings_inst_templates {
        if selection.beings_inst_templates { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &races {
        if selection.races { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
    for entity in &sexes {
        if selection.sexes { commands.entity(entity).try_insert_if_new(SelectedForHotReload); }
        else { commands.entity(entity).try_remove::<SelectedForHotReload>(); }
    }
}

#[allow(unused_parens)]
pub fn change_to_finished_asset_loading_state(
    _on: On<ChangeAssetLoadingStateToFinished>,
    mut cmd: Commands,
) {
    cmd.insert_resource(ChangeAssetLoadingStateToFinishedTimer(Timer::from_seconds(1., TimerMode::Once)));
}

#[allow(unused_parens)]
pub fn finish_asset_loading_after_delay(
    time: Res<Time>,
    finish_timer: If<ResMut<ChangeAssetLoadingStateToFinishedTimer>>,
    mut cmd: Commands,
    mut asset_loading_state: ResMut<NextState<AssetLoading>>,
) {
    let mut finish_timer = finish_timer.into_inner();

    finish_timer.0.tick(time.delta());
    if !finish_timer.0.is_finished() {
        return;
    }

    cmd.remove_resource::<ChangeAssetLoadingStateToFinishedTimer>();
    asset_loading_state.set(AssetLoading::Finished);
}
