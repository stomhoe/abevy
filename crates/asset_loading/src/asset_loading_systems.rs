use bevy::prelude::*;
use common::{common_components::{AnyDisabling, AssetScoped, SparedFromHotReloading}, common_states::*};
use dimension_shared::{DimensionStrIdRef};

use tilemap::{chunking_components::ActivatingChunks, terrain_gen::terrgen_resources::*};
use tilemap_shared::ForceAllChunksDespawn;


#[allow(unused_parens, )]
pub fn reload_assets_while_ingame(
    mut cmd: Commands, 
    keys: Res<ButtonInput<KeyCode>>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,

    mut regpos: ResMut<RegisteredPositions>,
    mut force_all_chunks_despawn_writer: MessageWriter<ForceAllChunksDespawn>,
) {
    if keys.pressed(KeyCode::KeyR) {
        info!(target: "asset_loading", "Reloading assets...");


        
        hot_loading.set(AssetHotReloadState::Ongoing);
        force_all_chunks_despawn_writer.write_default();

        regpos.clear();
    
        loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
    }
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut cmd: Commands,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
    mut game_state: ResMut<NextState<GamePhase>>,

) {
    hot_loading.set(AssetHotReloadState::Stopped);
    //game_state.set(GamePhase::ActiveGame);
}


pub fn despawn_asset_scoped_entities(
    mut commands: Commands,
    query: Query<Entity, (With<AssetScoped>, AnyDisabling)>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

pub fn despawn_asset_scoped_entities_except_spared(
    mut commands: Commands,
    query: Query<Entity, (With<AssetScoped>, Without<SparedFromHotReloading>, AnyDisabling)>,
) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}