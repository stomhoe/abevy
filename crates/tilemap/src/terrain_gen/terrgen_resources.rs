use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use common::types::HashIdToEntityMap;




#[derive(Resource, Debug, Reflect)]
pub struct GlobalGenSettings {
    
    pub seed: i32,
    pub c_decrease_per_1km: f32,
    pub world_size: Option<u32>,
}
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            c_decrease_per_1km: 15.0, //esto debería usarse para reducir o incrementar threshold
            world_size: None 
        }
    }
}

#[derive(Resource, Debug, Default )]
pub struct TerrGenEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default )]
pub struct OpListEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource)]
pub struct NoiseSerisHandles {
    #[asset(path = "ron/tilemap/terrgen/noise", collection(typed))]
    pub handles: Vec<Handle<NoiseSerialization>>,
}
#[derive(serde::Deserialize, Asset, TypePath, )]
pub struct NoiseSerialization {
    pub id: String,
    /// Default is 0.01
    pub frequency: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2S, 2: Cellular, 3: Perlin, 4: ValueCubic, 5: Value
    pub noise_type: Option<u32>,
    /// 0: None, 1: FBm, 2: Ridged, 3: PingPong, 4: DomainWarpProgressive, 5: DomainWarpIndependent,
    pub fractal_type: Option<u32>,
    /// Default is 3
    pub octaves: Option<u8>,
    /// Default is 2.0
    pub lacunarity: Option<f32>,
    /// Default is 0.5
    pub gain: Option<f32>,
    /// Default is 0.0
    pub weighted_strength: Option<f32>,
    /// Default is 2.0
    pub ping_pong_strength: Option<f32>,
    /// 0: Euclidean, 1: EuclideanSq, 2: Manhattan, 3: Hybrid
    pub cellular_distance_function: Option<u32>,
    /// 0: CellValue, 1: Distance, 2: Distance2, 3: Distance2Add, 4: Distance2Sub, 5: Distance2Mul, 6: Distance2Div
    pub cellular_return_type: Option<u32>,
    /// Default is 1.0
    pub cellular_jitter: Option<f32>,
    /// 0: OpenSimplex2, 1: OpenSimplex2Reduced, 2: BasicGrid
    pub domain_warp_type: Option<u32>,
    /// Default is 1.0
    pub domain_warp_amp: Option<f32>,
}





#[derive(AssetCollection, Resource)]
pub struct OpListSerisHandles {
    #[asset(path ="ron/tilemap/terrgen/oplist", collection(typed))]
    pub handles: Vec<Handle<OpListSerialization>>,
}
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub struct OpListSerialization {
    pub id: String,
    pub root: bool,
    pub operation_operands: HashMap<String, Vec<String>>,
    pub bifurcation_over: String,
    pub threshold: f32,
    pub bifurcation_under: String,
    pub tiles: Vec<String>,
}