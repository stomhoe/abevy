use bevy::{ecs::entity, math::U16Vec2, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_ecs_tilemap::map::TilemapTileSize;
use fastnoise_lite::FastNoiseLite;


#[derive(Resource, )]
pub struct WorldGenSettings {
    
    pub seed: i32,
    pub c_decrease_per_1km: f32,
    pub world_size: Option<u32>,
}
impl Default for WorldGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            c_decrease_per_1km: 15.0, //esto debería usarse para reducir o incrementar threshold
            world_size: None 
        }
    }
}



#[derive(Resource, Debug, Default )]
pub struct NoiseEntityMap(pub HashMap<String, Entity>);

#[allow(unused_parens)]
impl NoiseEntityMap {
    pub fn new_tile_ent_from_seri(
        &mut self, cmd: &mut Commands, handle: Handle<NoiseSeri>, assets: &mut Assets<NoiseSeri>,
    ) {
        if let Some(mut seri) = assets.remove(&handle) {
            use std::mem::take;
            if self.0.contains_key(&seri.id) {
                error!(target: "noise_loading", "NoiseSeri with id {:?} already exists in map, skipping", seri.id);
                return;
            }
            if seri.id.len() <= 2 {
                error!(target: "noise_loading", "NoiseSeri id is too short or empty, skipping");
                return;
            }

        }
    }

    pub fn get_entity<S: Into<String>>(&self, id: S) -> Option<Entity> { self.0.get(&id.into()).copied() }

    pub fn get_entities<I, S>(&self, ids: I) -> Vec<Entity> where I: IntoIterator<Item = S>, S: AsRef<str>, {
        ids.into_iter().filter_map(|id| self.0.get(id.as_ref()).copied()).collect()
    }
}

#[derive(AssetCollection, Resource)]
pub struct NoiseSerisHandles {
    #[asset(path = "ron/noise", collection(typed))]
    pub handles: Vec<Handle<NoiseSeri>>,
}
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct NoiseSeri {
    pub id: String,

}