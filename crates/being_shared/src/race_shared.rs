use bevy::{ecs::entity::{EntityHashMap, EntityHashSet, MapEntities}, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::*;
use common::common_components::*;
use serde::{Deserialize, Serialize};
use tilemap_shared::tilemap_shared_samplers::{HashIdWeightedSampler, SpriteGlobalNormalDist};

use crate::WanderSeri;
use common::common_tag_components::TagSet;
use tilemap_shared::*;

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Race"), AssetScoped, SelectedForHotReload)]
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

#[derive(Component, Debug, Default, Clone)]
pub struct SexesSampler(pub HashIdWeightedSampler);

impl SexesSampler {
    pub fn new(weights: &Vec<(HashId, f32)>) -> (Self, Vec<usize>) {
        let (sampler, negative_indices) = HashIdWeightedSampler::new(weights);
        (Self(sampler), negative_indices)
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

#[derive(serde::Deserialize, Asset, TypePath, Debug)]
#[serde(default)]
pub struct RaceSeri {
    pub id: String,
    pub tags: HashSet<String>,
    pub name: String,
    pub body_or_sampler: String,
    pub name_generator: Option<String>,
    pub icon_path: String,
    pub description: String,
    pub demonym: String,
    pub singular: String,
    pub plural: String,
    pub sentient: bool,
    pub fallback_sprites_to_sample: Vec<String>,
    pub scale_hp_and_strength_with_size: bool,
    pub size_variation: NormalDistSeri,
    pub hori_variation: NormalDistSeri,
    pub vert_variation: NormalDistSeri,
    pub sets_of_choosable_sprites: Vec<(String, HashSet<String>)>,
    pub can_walk_on: HashSet<String>,
    pub produces_step_sfx: bool,
    pub footstep_sfx: RaceFootstepSfxSeri,
    pub walk_speeds_on_tiles: HashMap<String, f32>,
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    pub predator: crate::PredatorSeri,
    pub wander: WanderSeri,
    pub melee_interaction_zone: InteractionZoneSeri,
    pub collision_zone: InteractionZoneSeri,
    pub pack_size_min_max: (u32, u32),
    pub spawn_pack_size_normal_dist: NormalDistSeri,
    pub pack_spawn_radius: u8,
    pub belongs_to_packs: Vec<String>,
    pub biome_affinity: HashMap<String, f32>,
    pub whitelisted_spawn_tile_tags: HashSet<String>,
    pub blacklisted_spawn_tile_tags: HashSet<String>,
    pub spawn_pack_entity: bool,
}

impl RaceSeri {
    pub fn tags_with_my_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}
impl Default for RaceSeri {
    fn default() -> Self {
        Self {
            id: String::default(),
            tags: HashSet::default(),
            name: String::default(),
            body_or_sampler: String::default(),
            name_generator: None,
            icon_path: String::default(),
            description: String::default(),
            demonym: String::default(),
            singular: String::default(),
            plural: String::default(),
            sentient: false,
            fallback_sprites_to_sample: Vec::default(),
            scale_hp_and_strength_with_size: true,
            size_variation: NormalDistSeri::default(),
            hori_variation: NormalDistSeri::default(),
            vert_variation: NormalDistSeri::default(),
            sets_of_choosable_sprites: Vec::default(),
            can_walk_on: HashSet::default(),
            produces_step_sfx: true,
            footstep_sfx: RaceFootstepSfxSeri::default(),
            walk_speeds_on_tiles: HashMap::default(),
            whitelisted_tiles_for_spawning: HashSet::default(),
            blacklisted_tiles_for_spawning: HashSet::default(),
            predator: crate::PredatorSeri::default(),
            wander: WanderSeri::default(),
            melee_interaction_zone: InteractionZoneSeri::default(),
            collision_zone: InteractionZoneSeri::default(),
            pack_size_min_max: (0, 0),
            spawn_pack_size_normal_dist: NormalDistSeri::default(),
            pack_spawn_radius: crate::PackSpawnRadius::default().0,
            belongs_to_packs: Vec::default(),
            biome_affinity: HashMap::default(),
            whitelisted_spawn_tile_tags: HashSet::default(),
            blacklisted_spawn_tile_tags: HashSet::default(),
            spawn_pack_entity: true,
        }
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


common::define_entity_map_systems!(
    main_component: Race,
    with_filters: (),
    abbreviation: Race,
    target: common::log_targets::ENTITY_MAP_SYSTEM,
    entity_prefix: "",
    despawn_trigger: Race,
    id_type: common::common_components::StrId,
    assets: [(RaceSeri, "seri.being.race", "race.ron")],
);
