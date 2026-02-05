use bevy::prelude::*;
use bevy_replicon::prelude::{ServerState};
use common::{common_components::{AssetScoped, SparedFromHotReloading}, common_states::*};

use tilemap::{terrain_gen::terrgen_resources::*};
use tilemap_shared::ForceAllChunksDespawn;


#[allow(unused_parens, )]
pub fn reload_assets_while_ingame(
    keys: Res<ButtonInput<KeyCode>>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,

    mut regpos: ResMut<tilemap::tilemap_resources::ImportantRegisteredPositions>,
    mut force_all_chunks_despawn_writer: MessageWriter<ForceAllChunksDespawn>,
    client_state: Res<State<ServerState>>,
) {
    if keys.pressed(KeyCode::KeyR) {
        
        if *client_state.get() != ServerState::Running {
            warn!(target: "asset_loading", "You cannot hot-reload assets as a client.");
            return;
        }
        info!(target: "asset_loading", "Reloading assets...");


        
        hot_loading.set(AssetHotReloadState::Ongoing);
        force_all_chunks_despawn_writer.write_default();

        regpos.clear();
    
        loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
    }
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
) {
    hot_loading.set(AssetHotReloadState::Stopped);
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
    query: Query<Entity, (With<AssetScoped>, Without<SparedFromHotReloading>, common::AnyDisabling)>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}