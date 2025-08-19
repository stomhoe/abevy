use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use common::common_types::HashIdToEntityMap;




#[derive(Resource, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct GlobalGenSettings {
    
    pub seed: i32,
    pub c_decrease_per_1km: f32,
    pub world_freq: f32,
    pub world_limits: Option<u32>,
}
impl Default for GlobalGenSettings {
    fn default() -> Self {
        Self { 
            seed: 0,
            c_decrease_per_1km: 15.0, //esto debería usarse para reducir o incrementar split
            world_limits: None,
            world_freq: 1e1,
        }
    }
}

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct TerrGenEntityMap(pub HashIdToEntityMap);

#[derive(Resource, Debug, Default, Reflect, )]
#[reflect(Resource, Default)]
pub struct OpListEntityMap(pub HashIdToEntityMap);


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct NoiseSerisHandles {
    #[asset(path = "ron/tilemap/terrgen/noise", collection(typed))]
    pub handles: Vec<Handle<NoiseSerialization>>,
}
#[derive(serde::Deserialize, Asset, Reflect, )]
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


#[derive(AssetCollection, Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct OpListSerisHandles {
    #[asset(path ="ron/tilemap/terrgen/oplist", collection(typed))]
    pub handles: Vec<Handle<OpListSerialization>>,
}
#[derive(serde::Deserialize, Asset, Reflect, Default)]
pub struct OpListSerialization {
    pub id: String,
    pub root_in_dimensions: Vec<String>,
    pub operation_operands: HashMap<String, Vec<String>>,
    pub bifover: String,
    pub tiles_over: Vec<String>,
    pub split: f32,
    pub bifunder: String,
    pub tiles_under: Vec<String>,
    pub size: Option<[u32; 2]>
}
impl OpListSerialization {
    pub fn is_root(&self) -> bool {
        self.root_in_dimensions.iter().any(|s| !s.is_empty())
    }
}