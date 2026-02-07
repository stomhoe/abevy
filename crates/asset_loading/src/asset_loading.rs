

use bevy_replicon::prelude::ClientState;
use common::common_states::*;
use bevy_asset_loader::prelude::*;
use game_common::{GameplaySystems, };

use crate::asset_loading_systems::*;

#[allow(unused_imports)] use {bevy::prelude::*, };



#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AssetHotReloading;


#[allow(unused_parens, path_statements, )]
pub fn plugin(app: &mut App) {
    app
        .init_state::<AssetLoading>()
        .init_state::<AssetHotReloadState>()

        .add_systems(Update, (
            reload_assets_while_ingame,
        ))
        .add_systems(OnExit(AppState::StatefulGameSession),
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(ClientState::Connecting),
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(AssetLoading::LoadingAssetsIntoHandles),
            despawn_asset_scoped_entities_except_spared
        )
        .add_systems(OnEnter(AssetLoading::NotStarted),
            despawn_asset_scoped_entities
        )
        // Don't use OnExit(AssetLoading::SpawnReplicatedEntities) because clients aren't in that state

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            on_assets_loaded.in_set(AssetHotReloading)
        ))
        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            AssetHotReloading.run_if(in_state(AssetHotReloadState::Ongoing)).after(GameplaySystems)
        ))
        .add_loading_state(
            LoadingState::new(AssetLoading::LoadingAssetsIntoHandles).continue_to_state(AssetLoading::SpawnReplicatedEntities)
        );
}
