use being::being_components::Being;
use bevy::prelude::*;
use common::{common_components::{AnyDisabling, AssetScoped, SparedFromHotReloading}, common_states::*};
use dimension_shared::{DimensionStrIdRef};

use tilemap::{chunking_components::ActivatingChunks, terrain_gen::terrgen_resources::*};


#[allow(unused_parens, )]
pub fn reload_assets_while_ingame(
    mut cmd: Commands, 
    keys: Res<ButtonInput<KeyCode>>,
    beings_query: Query<(Entity), (With<Being>,)>,
    mut chunks_query: Query<&mut ActivatingChunks>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,

    mut regpos: ResMut<RegisteredPositions>,
) {
    if keys.pressed(KeyCode::KeyR) {
        info!(target: "asset_loading", "Reloading assets...");
        beings_query.iter().for_each(|(being_ent)| {
            cmd.entity(being_ent).try_remove::<(ChildOf,)>();
        });

        for (mut activating_chunks) in chunks_query.iter_mut() {
            activating_chunks.entities.clear();
        }
        hot_loading.set(AssetHotReloadState::Ongoing);

        regpos.registered.clear();
    
        loading_state.set(AssetLoading::LoadingAssetsIntoHandles);
    }
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut cmd: Commands,
    mut hot_loading: ResMut<NextState<AssetHotReloadState>>,
    beings_query: Query<(Entity), (With<Being>, Without<ChildOf>)>,
    mut game_state: ResMut<NextState<GamePhase>>,

) {
    
    beings_query.iter().for_each(|(being_ent)| {
        cmd.entity(being_ent).try_insert(DimensionStrIdRef::overworld_fallback());
    });
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