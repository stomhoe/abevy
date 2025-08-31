use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use common::common_states::*;
use tilemap::{chunking_components::{ActivatingChunks, }, terrain_gen::{terrgen_components::{TerrGen}, terrgen_resources::*}, tile::{tile_components::*, tile_resources::*, tile_sampler_resources::TileWeightedSamplersMap}};


#[allow(unused_parens, )]
pub fn reload_assets_ingame(
    mut cmd: Commands, 
    keys: Res<ButtonInput<KeyCode>>,
    mut chunks_query: Query<&mut ActivatingChunks>,
    mut loading_state: ResMut<NextState<AssetsLoadingState>>,
    mut hot_loading: ResMut<NextState<TerrainGenHotLoading>>,
) {
    
    if keys.pressed(KeyCode::KeyR) {
        info!(target: "asset_loading", "Reloading assets...");

        for (mut activating_chunks) in chunks_query.iter_mut() {
            activating_chunks.0.clear();
        }
        hot_loading.set(TerrainGenHotLoading::DespawnAll);

        //cmd.insert_resource(AnimationLibrary::default());
        
        //cmd.remove_resource::<SpriteCfgEntityMap>();

        cmd.remove_resource::<TileWeightedSamplersMap>();
        cmd.remove_resource::<TileShaderEntityMap>();
        cmd.remove_resource::<TileEntitiesMap>();
        
        cmd.remove_resource::<OpListEntityMap>();
        cmd.remove_resource::<TerrGenEntityMap>();
    
        //cmd.remove_resource::<SpriteCfgEntityMap>();
        loading_state.set(AssetsLoadingState::LocalInProcess);
    }
}



#[allow(unused_parens, )]
pub fn moveon_to_replicated(
    mut loading_state: ResMut<NextState<AssetsLoadingState>>,
) {
    loading_state.set(AssetsLoadingState::ReplicatedInProcess);
}

#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut hot_loading: ResMut<NextState<TerrainGenHotLoading>>,
) {
    hot_loading.set(TerrainGenHotLoading::KeepAlive);
}

