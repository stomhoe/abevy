

use bevy_replicon::prelude::ClientState;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::prelude::*;
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
        .init_resource::<HotReloadSelection>()
        .add_systems(Update, (
            reload_assets_while_ingame,
            finish_asset_loading_after_delay,
            sync_hot_reload_markers.run_if(resource_changed::<HotReloadSelection>),
        ))
        .add_observer(process_hot_reload_request)
        .add_observer(change_to_finished_asset_loading_state)
        .add_systems(OnExit(AppState::StatefulGameSession),
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(ClientState::Connecting),
            despawn_asset_scoped_entities
        )
        .add_systems(OnEnter(AssetLoading::LoadingAssetsIntoHandles),
            despawn_selected_asset_scoped_entities
        )
        .add_systems(OnEnter(AssetLoading::NotStarted),
            despawn_asset_scoped_entities
        )
        // Don't use OnExit(AssetLoading::SpawnReplicatedEntities) because clients aren't in that state

        .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            remap_broken_sprite_config_refs_after_hotreload,
            on_assets_loaded.in_set(AssetHotReloading),
            validate_defs_after_load.run_if(in_state(ClientState::Disconnected)),
        ))
        .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
            AssetHotReloading.run_if(in_state(AssetHotReloadState::Ongoing)).after(GameplaySystems)
        ))
        .add_loading_state(
            LoadingState::new(AssetLoading::LoadingAssetsIntoHandles).continue_to_state(AssetLoading::SpawnReplicatedEntities)
        );
}
