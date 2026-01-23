use being::being_components::Being;
use bevy::prelude::*;
use common::common_states::*;
use dimension_shared::{DimensionEntityMap, DimensionStrIdRef};
use sprite::sprite_resources::SpriteCfgEntityMap;
use sprite_animation_shared::AnimationLibrary;
use tilemap::{chunking_components::ActivatingChunks, terrain_gen::terrgen_resources::*, tile::{tile_resources::*, tile_sampler_resources::TileWeightedSamplersMap, tile_shader::tile_shader_resources::TileShaderEntityMap}};


#[allow(unused_parens, )]
pub fn reload_assets_ingame(
    mut cmd: Commands, 
    keys: Res<ButtonInput<KeyCode>>,
    beings_query: Query<(Entity), (With<Being>,)>,
    mut chunks_query: Query<&mut ActivatingChunks>,
    mut loading_state: ResMut<NextState<AssetLoading>>,
    mut hot_loading: ResMut<NextState<TerrainHotReloading>>,
    mut tiling_map: ResMut<TileEzerosMap>,
    mut oplist_map: ResMut<OpListEntityMap>,
    mut sc_map: ResMut<SpriteCfgEntityMap>,
    mut dimension_entity_map: ResMut<DimensionEntityMap>,
    mut terr_gen_map: ResMut<TerrGenEntityMap>,
    mut tile_shader_map: ResMut<TileShaderEntityMap>,

    mut tile_samplers_map: ResMut<TileWeightedSamplersMap>,

    mut regpos: ResMut<RegisteredPositions>,
    mut library: ResMut<AnimationLibrary>,
) {
    if keys.pressed(KeyCode::KeyR) {
        info!(target: "asset_loading", "Reloading assets...");
        beings_query.iter().for_each(|(being_ent)| {
            cmd.entity(being_ent).try_remove::<(ChildOf,)>();
        });

        for (mut activating_chunks) in chunks_query.iter_mut() {
            activating_chunks.0.clear();
        }
        hot_loading.set(TerrainHotReloading::DespawnAll);

        //cmd.insert_resource(AnimationLibrary::default());
        

        tile_samplers_map.0.clear();
        tile_shader_map.0.clear();

        tiling_map.0.clear();

        sc_map.0.clear();
        
        oplist_map.0.clear();
        terr_gen_map.0.clear();
        dimension_entity_map.0.clear();
        library.0.clear();
        regpos.0.clear();
    
        loading_state.set(AssetLoading::LocalInProcess);
    }
}
#[allow(unused_parens, )]
pub fn moveon_to_replicated(
    mut loading_state: ResMut<NextState<AssetLoading>>,
) {
    loading_state.set(AssetLoading::LoadingReplicatedCollections);
}
#[allow(unused_parens, )]
pub fn on_assets_loaded(
    mut cmd: Commands,
    mut hot_loading: ResMut<NextState<TerrainHotReloading>>,
    beings_query: Query<(Entity), (With<Being>, Without<ChildOf>)>,
) {
    hot_loading.set(TerrainHotReloading::KeepAlive);

    beings_query.iter().for_each(|(being_ent)| {
        cmd.entity(being_ent).insert(DimensionStrIdRef::overworld_fallback());
    });
}

