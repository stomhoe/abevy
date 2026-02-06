
use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_asset_loader::prelude::*;
use crate::race::Race;

#[derive(AssetCollection, Resource, Default, )]
pub struct RaceSerisHandles {
    #[asset(path = "ron/being/race", collection(typed))]
    pub handles: Vec<Handle<RaceSerialization>>,
}



#[derive(serde::Deserialize, Asset, Default, Debug, Reflect)]
pub struct RaceSerialization {
    pub id: String,
    pub name: String,
    pub name_generator: Option<String>,
    pub icon_path: Option<String>,
    pub description: Option<String>,
    pub demonym: Option<String>,
    pub singular: Option<String>,
    pub plural: Option<String>,
    pub sexes: HashMap<String, (u32, Vec<String>)>,//id, (weight, spriteents_to_sampl (may be empty, in that case fallback is used)
    pub sentient: Option<bool>,
    pub fallback_sprites_to_sample: Vec<String>,

    /// Each vec entry is a tuple of (group_name (face, hair, skin color) and the choosable sprites. from the group (HashSet<String>) you are supposed to choose only one). fallbacks to sprites_to_sample when offering sprites to select to player if None
    pub sets_of_choosable_sprites: Option<Vec<(String, HashSet<String>)>>,
    /// global_caloric_burn_rate_multiplier
    pub caloric_burn_rate_multiplier: Option<f32>,
    pub can_walk_on: Option<HashSet<String>>,
    pub walk_speeds_on_tiles: Option<HashMap<String, f32>>,

    pub whitelisted_tiles_for_spawning: Option<HashSet<String>>,
    pub blacklisted_tiles_for_spawning: Option<HashSet<String>>,

    ///can be a body sampler
    pub body_tree: String,
}

common::define_entity_map_systems!(
    common::common_components::StrId,
    Race
);
