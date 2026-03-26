use bevy::{ecs::entity::{EntityHashMap, EntityHashSet, MapEntities}, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::tilemap_shared_samplers::{EntityWeightedSampler, SpriteGlobalNormalDist};

use crate::WanderSeri;
use common::common_tag_components::TagSet;
use tilemap_shared::*;

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Race"), AssetScoped, HotReload)]
pub struct Race;

/// do not insert this into beings
#[derive(Component, Debug, Default, Clone)]
#[component(map_entities)]
pub struct SetsOfPlayerMonoChoosableSprites(#[entities] pub Vec<(StrId, EntityHashSet)>);

impl MapEntities for SetsOfPlayerMonoChoosableSprites {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.0.iter_mut().for_each(|(_, entities)| {
            let mut entities_to_update: Vec<Entity> = entities.iter().copied().collect();
            entities_to_update.iter_mut().for_each(|entity| {
                *entity = entity_mapper.get_mapped(*entity);
            });
            entities.clear();
            entities.extend(entities_to_update);
        });
    }
}

#[derive(Component, Debug, Default, MapEntities, Clone)]
pub struct SexesSampler(#[entities] pub EntityWeightedSampler);

impl SexesSampler {
    pub fn new(weights: &Vec<(Entity, f32)>) -> Self {
        Self(EntityWeightedSampler::new(&weights))
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct SexSizeVariationsBySex(pub EntityHashMap<SpriteGlobalNormalDist>);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ProducesStepSfx;

#[derive(Component, Debug, Default, Clone)]
pub struct RaceFootstepSfxConfig {
    pub paths: Vec<String>,
    pub disable_tile_step_sfx: bool,
}

#[derive(serde::Deserialize, Asset, TypePath, Default, Debug)]
pub struct RaceSeri {
    pub id: String,
    #[serde(default)]
    pub tags: HashSet<String>,
    pub name: String,
    pub body_or_sampler: String,
    pub name_generator: Option<String>,
    #[serde(default)]
    pub icon_path: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub demonym: String,
    #[serde(default)]
    pub singular: String,
    #[serde(default)]
    pub plural: String,
    #[serde(default)]
    pub sentient: bool,
    pub fallback_sprites_to_sample: Vec<String>,
    #[serde(default = "default_true")]
    pub scale_hp_and_strength_with_size: bool,
    #[serde(default)]
    pub size_variation: NormalDistSeri,
    #[serde(default)]
    pub hori_variation: NormalDistSeri,
    #[serde(default)]
    pub vert_variation: NormalDistSeri,
    #[serde(default)]
    pub sets_of_choosable_sprites: Vec<(String, HashSet<String>)>,
    #[serde(default)]
    pub can_walk_on: HashSet<String>,
    #[serde(default = "default_true")]
    pub produces_step_sfx: bool,
    #[serde(default)]
    pub footstep_sfx: RaceFootstepSfxSeri,
    #[serde(default)]
    pub walk_speeds_on_tiles: HashMap<String, f32>,
    #[serde(default)]
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    #[serde(default)]// these are not hunted/attacked even if hungry and carnivore
    pub friend_races: HashSet<String>,
    #[serde(default)]
    pub predator_territorialism: f32,
    #[serde(default = "default_u32_1_1")]
    pub predator_pack_size_range: (u32, u32),
    #[serde(default)]
    pub predator_dont_hunt: HashSet<String>,
    #[serde(default = "default_prey_body_size_ratio_tolerance")]
    pub predator_prey_kg_ratio_over_us_tolerance: f32,
    #[serde(default = "default_predator_hunt_threshold")]
    pub predator_hunt_threshold: f32,
    #[serde(default = "default_detection_vision_cone_sentinel")]
    pub detection_vision_cone_range_tiles: f32,
    #[serde(default = "default_detection_vision_cone_sentinel")]
    pub detection_vision_cone_half_angle_deg: f32,
    #[serde(default)]
    pub wander: WanderSeri,
    #[serde(default = "tilemap_shared::sentinel_melee_interaction_zone")]
    pub melee_interaction_zone: InteractionZoneSeri,
    #[serde(default = "tilemap_shared::sentinel_collision_zone")]
    pub collision_zone: InteractionZoneSeri,
    #[serde(default)]//targets for already-spawned packs
    pub pack_size_min_max: (u32, u32),
    #[serde(default)]//if this race wins sampling, it will spawn with a pack size drawn from this distribution
    pub spawn_pack_size_normal_dist: NormalDistSeri,
    #[serde(default)]//additive membership into already-defined packs
    pub belongs_to_packs: Vec<String>,
    #[serde(default)]
    pub biome_affinity: HashMap<String, f32>,
    #[serde(default)]
    pub whitelisted_spawn_tile_tags: HashSet<String>,
    #[serde(default)]
    pub blacklisted_spawn_tile_tags: HashSet<String>,
    #[serde(default = "default_true")]
    pub spawn_pack_entity: bool,
}

impl RaceSeri {
    pub fn tags_with_my_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct RaceFootstepSfxSeri {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub disable_tile_step_sfx: bool,
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct RaceSexEntrySeri {//TODO fix usage
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub sprites: Vec<String>,
    pub size_variation: Option<NormalDistSeri>,
}

fn default_true() -> bool { true }
fn default_predator_hunt_threshold() -> f32 { crate::PredatorHuntThreshold::SERI_SENTINEL }
fn default_detection_vision_cone_sentinel() -> f32 { crate::DetectionVisionCone::SERI_SENTINEL }
fn default_u32_1_1() -> (u32, u32) { (1, 1) }
fn default_prey_body_size_ratio_tolerance() -> f32 { -1.0 }

common::define_entity_map_systems!(
    Race,
    RaceSeri, "seri.being.race", "race.ron",
);
