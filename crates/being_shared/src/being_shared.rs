
use bevy::{ecs::entity::EntityHashMap, platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;
use faction_shared::BelongsToAPlayerFaction;
use tilemap_shared::{BlacklistedSpawnTileTags, WhitelistedSpawnTileTags};
use crate::being_shared_seris::{DEFAULT_AVOID_ENTITY_RADIUS, DEFAULT_AVOID_ENTITY_STRENGTH, WanderSeri};

#[derive(Component, Debug, Default, Clone)]
pub struct ComputedLocally;

#[derive(Component, Debug, Copy, Clone, Default, Deserialize, Serialize)]
#[require(Prefix::trunc("Being"),)]
pub struct Being;
impl Being {
    pub const Z_LEVEL: f32 = 1_000.;

    pub fn collect_spawn_tile_tag_filters(
        bit_ent: Option<Entity>,
        race_ent: Option<Entity>,
        spawn_tile_tags_query: &Query<(
            Option<&WhitelistedSpawnTileTags>,
            Option<&BlacklistedSpawnTileTags>,
        )>,
        mut bit_race_ent: impl FnMut(Entity) -> Option<Entity>,
        whitelisted_tags: &mut WhitelistedSpawnTileTags,
        blacklisted_tags: &mut BlacklistedSpawnTileTags,
    ) {
        whitelisted_tags.0.clear();
        blacklisted_tags.0.clear();

        let mut effective_race_ent = race_ent;
        if let Some(bit_ent) = bit_ent {
            let Ok((bit_whitelist, bit_blacklist)) = spawn_tile_tags_query.get(bit_ent) else {
                return;
            };
            if let Some(bit_whitelist) = bit_whitelist {
                whitelisted_tags.0.extend_from(&bit_whitelist.0);
            }
            if let Some(bit_blacklist) = bit_blacklist {
                blacklisted_tags.0.extend_from(&bit_blacklist.0);
            }
            if effective_race_ent.is_none() {
                effective_race_ent = bit_race_ent(bit_ent);
            }
        }

        let Some(race_ent) = effective_race_ent else {
            blacklisted_tags.0.retain(|tag| !whitelisted_tags.0.contains_ref(tag));
            return;
        };
        let Ok((race_whitelist, race_blacklist)) = spawn_tile_tags_query.get(race_ent) else {
            blacklisted_tags.0.retain(|tag| !whitelisted_tags.0.contains_ref(tag));
            return;
        };
        if let Some(race_whitelist) = race_whitelist {
            whitelisted_tags.0.extend_from(&race_whitelist.0);
        }
        if let Some(race_blacklist) = race_blacklist {
            blacklisted_tags.0.extend_from(&race_blacklist.0);
        }
        blacklisted_tags.0.retain(|tag| !whitelisted_tags.0.contains_ref(tag));
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct HumanControlled;

pub type LocalAiControlled = (With<ComputedLocally>, Without<HumanControlled>);
pub type LocalHumanControlled = (With<ComputedLocally>, With<HumanControlled>);

//CAN BE A BOT RUN IN THE CLIENT'S COMPUTER (P.EJ PATHFINDING)


#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = ComputedBy)]
pub struct ComputedBeings(Vec<Entity>);
impl ComputedBeings {pub fn being_ents(&self) -> &[Entity] {&self.0}}


#[derive(Component, Debug, Deserialize, Serialize, MapEntities, Clone)]
#[relationship(relationship_target = ComputedBeings)]
pub struct ComputedBy  {
    #[relationship] #[entities]
    pub client_ent: Entity,
    pub human_dc_input: bool,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
#[require(Replicated, Prefix::trunc("BeingInstTemplate"), AssetScoped, HotReload)]
pub struct BeingInstTemplate{
    pub points: u32,
    pub extra_health_multiplier: f32,
}



#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct WallPhaser;


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyTreeWeightSum(pub f32);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq)]
pub struct MainCharacter{#[entities] created_by: Entity}

#[derive(Component, Debug, Default, Clone, Copy, Hash, PartialEq)]
pub struct InfiniteMorale;

#[derive(Component, Default, Deserialize, Serialize, Clone)]
pub struct DirectControllable;

#[derive(Component, MapEntities, Clone)]
//no insertar este component si no se quiere restringir quien puede tomar control
/// entities: whitelisted players
pub struct ControlTakeoverWhitelist(#[entities] pub Vec<Entity>);//chequear si es de la misma facción antes de intentar tomar control

#[derive(Component, Debug, Copy, Clone, MapEntities)]
pub struct TouchingPortal(#[entities] pub Entity);

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, Copy, Clone, )]
#[relationship(relationship_target = Followers)]
pub struct FollowerOf {#[relationship] #[entities] pub master: Entity,}

#[derive(Component, Debug, Reflect, Clone)]
#[relationship_target(relationship = FollowerOf)]
pub struct Followers(Vec<Entity>);

#[derive(Component, Debug, Clone)]
pub struct LearningMultiplier(pub EntityHashMap<f32>);

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, Copy, Clone, )]
#[relationship(relationship_target = CreatedCharacters)]
#[require(DirectControllable, )]
pub struct CharacterCreatedBy {
    #[relationship] #[entities] pub player: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = CharacterCreatedBy)]
pub struct CreatedCharacters(Vec<Entity>);


#[derive(Component, Debug, Clone, )]
pub struct SexMappedSpritesToSample(
    /// sexent - samplespriteents
    pub EntityHashMap<SampleSpriteEnts>,
);



#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct PredatorCfg {
    pub territorialism: f32,
    pub pack_size_min: u32,
    pub pack_size_max: u32,
    pub do_not_hunt_tags: TagSet,
    pub prey_body_size_ratio_tolerance: f32,
    pub min_hunger_to_hunt: f32,
    pub min_hp_ratio_to_hunt: f32,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredatorSeri {
    #[serde(default)]
    pub territorialism: f32,
    #[serde(default)]
    pub pack_size_min: u32,
    #[serde(default)]
    pub pack_size_max: u32,
    #[serde(default)]
    pub do_not_hunt_tags: HashSet<String>,
    #[serde(default)]
    pub prey_body_size_ratio_tolerance: f32,
    #[serde(default = "default_predator_seri_uninitialized")]
    pub min_hunger_to_hunt: f32,
    #[serde(default)]
    pub min_hp_ratio_to_hunt: f32,
}
impl PredatorSeri {
    pub const SERI_UNINITIALIZED: f32 = f32::NEG_INFINITY;
    pub fn is_uninitialized(&self) -> bool {
        self.min_hunger_to_hunt == Self::SERI_UNINITIALIZED
    }
}
impl Default for PredatorSeri {
    fn default() -> Self {
        Self {
            territorialism: 0.0,
            pack_size_min: 1,
            pack_size_max: 1,
            do_not_hunt_tags: HashSet::default(),
            prey_body_size_ratio_tolerance: -1.0,
            min_hunger_to_hunt: Self::SERI_UNINITIALIZED,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}
impl Default for PredatorCfg {
    fn default() -> Self {
        Self {
            territorialism: 0.0,
            pack_size_min: 1,
            pack_size_max: 1,
            do_not_hunt_tags: TagSet::default(),
            prey_body_size_ratio_tolerance: -1.0,
            min_hunger_to_hunt: 40.0,
            min_hp_ratio_to_hunt: 0.0,
        }
    }
}
impl PredatorCfg {
    pub fn from_seri(seri: &PredatorSeri) -> Option<Self> {
        if seri.is_uninitialized() {
            return None;
        }
        let mut pack_size_min = seri.pack_size_min;
        let mut pack_size_max = seri.pack_size_max;
        if pack_size_min == 0 {
            pack_size_min = 1;
        }
        if pack_size_max < pack_size_min {
            pack_size_max = pack_size_min;
        }
        Some(Self {
            territorialism: seri.territorialism.max(0.0),
            pack_size_min,
            pack_size_max,
            do_not_hunt_tags: TagSet::new(&seri.do_not_hunt_tags),
            prey_body_size_ratio_tolerance: seri.prey_body_size_ratio_tolerance,
            min_hunger_to_hunt: seri.min_hunger_to_hunt.max(0.0),
            min_hp_ratio_to_hunt: seri.min_hp_ratio_to_hunt.clamp(0.0, 1.0),
        })
    }
}
#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct Predator;


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Hunger {
    pub curr: f32,
    pub max: f32,
    pub increase_per_sec: f32,
}
impl Default for Hunger {
    fn default() -> Self {
        Self {
            curr: 0.0,
            max: 100.0,
            increase_per_sec: 2.0,
        }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct DetectionVisionCone {
    pub range_tiles: f32,
    pub half_angle_deg: f32,
}
impl Default for DetectionVisionCone {
    fn default() -> Self {
        Self {
            range_tiles: 9.0,
            half_angle_deg: 45.0,
        }
    }
}
impl DetectionVisionCone {
    pub const SERI_SENTINEL: f32 = f32::NEG_INFINITY;
    pub fn is_configured_in_seri(range_tiles: f32, half_angle_deg: f32) -> bool {
        range_tiles > Self::SERI_SENTINEL && half_angle_deg > Self::SERI_SENTINEL
    }
}

fn default_predator_seri_uninitialized() -> f32 { PredatorSeri::SERI_UNINITIALIZED }

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
pub struct PredatorDetectedByPrey(#[entities] pub Entity);


#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, bevy::ecs::entity::MapEntities, )]
#[relationship(relationship_target = SimulatedBeingsWithin)]
#[require(Unloaded)]
pub struct BgSimulatedIn {
    #[relationship] #[entities]
    pub macro_chunk_ent: Entity,
}


#[derive(Component, Debug, )]
#[relationship_target(relationship = BgSimulatedIn)]
pub struct SimulatedBeingsWithin(Vec<Entity>);



#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
pub struct Unloaded;


pub type PlayerBeing = (With<Being>, With<BelongsToAPlayerFaction>);

pub type LoadedBeing = (With<Being>, Without<Unloaded>);
pub type UnloadedBeing = (With<Being>, With<Unloaded>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct MemberRank(pub f32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct NoSpawnGroup;

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, bevy::ecs::entity::MapEntities, )]
#[relationship(relationship_target = SquadMembers)]
pub struct SquadMemberOf(#[relationship]#[entities]pub Entity);

// current physically close distance group of beings we belong to and are currently operating with
#[derive(Component, Debug, )]
#[relationship_target(relationship = SquadMemberOf)]
pub struct SquadMembers(Vec<Entity>);


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(Replicated, )]//HACER Q MEJOR ESTO SE REGISTRE EN EL CHUNK PARA NO TENER QUE QUERYEAR TODA TILE O BIENG ADENTRO
pub struct PreventsChunkUnloading;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct ChunkPersistersWithin(pub u32);

/*
*/
#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, bevy::ecs::entity::MapEntities, )]
#[relationship(relationship_target = HuntedBy)]
pub struct Hunting {
    #[relationship] #[entities]
    pub prey: Entity,
}

#[derive(Component, Debug, )]
#[relationship_target(relationship = Hunting)]
pub struct HuntedBy(Vec<Entity>);

#[derive(Component, Debug, Clone, Deserialize, Serialize)]
pub struct WanderConfig {
    pub dir_secs_min: f32,
    pub dir_secs_max: f32,
    pub move_secs_min: f32,
    pub move_secs_max: f32,
    pub halt_secs_min: f32,
    pub halt_secs_max: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    #[serde(default)]
    pub avoid_tile_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_bit_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_race_tags: HashSet<String>,
    #[serde(default)]
    pub avoid_pack_tags: HashSet<String>,
    #[serde(default = "crate::being_shared_seris::default_nan")]
    pub max_drift: f32,
    #[serde(default)]
    pub pack_orbit_radius: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_min: f32,
    #[serde(default)]
    pub pack_orbit_retarget_secs_max: f32,
    #[serde(default)]
    pub wander_around_leader: bool,
    #[serde(default)]
    pub avoid_entity_radius: HashMap<String, f32>,
    #[serde(default)]
    pub avoid_entity_strength: HashMap<String, f32>,
}
impl Default for WanderConfig {
    fn default() -> Self {
        Self {
            dir_secs_min: 0.8,
            dir_secs_max: 2.4,
            move_secs_min: 0.9,
            move_secs_max: 2.4,
            halt_secs_min: 0.25,
            halt_secs_max: 1.4,
            speed_min: 0.2,
            speed_max: 0.6,
            avoid_tile_tags: HashSet::default(),
            avoid_bit_tags: HashSet::default(),
            avoid_race_tags: HashSet::default(),
            avoid_pack_tags: HashSet::default(),
            max_drift: f32::NAN,
            pack_orbit_radius: 0.0,
            pack_orbit_retarget_secs_min: 0.0,
            pack_orbit_retarget_secs_max: 0.0,
            wander_around_leader: false,
            avoid_entity_radius: HashMap::default(),
            avoid_entity_strength: HashMap::default(),
        }
    }
}

impl WanderConfig {
    pub fn from_seri(seri: &WanderSeri) -> Self {
        Self {
            dir_secs_min: seri.dir_secs_min,
            dir_secs_max: seri.dir_secs_max,
            move_secs_min: seri.move_secs_min,
            move_secs_max: seri.move_secs_max,
            halt_secs_min: seri.halt_secs_min,
            halt_secs_max: seri.halt_secs_max,
            speed_min: seri.speed_min,
            speed_max: seri.speed_max,
            avoid_tile_tags: seri.avoid_tile_tags.clone(),
            avoid_bit_tags: seri.avoid_bit_tags.clone(),
            avoid_race_tags: seri.avoid_race_tags.clone(),
            avoid_pack_tags: seri.avoid_pack_tags.clone(),
            max_drift: seri.max_drift,
            pack_orbit_radius: seri.pack_orbit_radius,
            pack_orbit_retarget_secs_min: seri.pack_orbit_retarget_secs_min,
            pack_orbit_retarget_secs_max: seri.pack_orbit_retarget_secs_max,
            wander_around_leader: seri.wander_around_leader,
            avoid_entity_radius: seri
                .avoid_entity_radius
                .iter()
                .map(|(tag, radius)| (tag.clone(), radius.max(0.0)))
                .collect(),
            avoid_entity_strength: seri
                .avoid_entity_strength
                .iter()
                .map(|(tag, strength)| (tag.clone(), strength.max(0.0)))
                .collect(),
        }
        .sanitized()
    }

    pub fn sanitized(mut self) -> Self {
        self.dir_secs_min = self.dir_secs_min.max(0.01);
        self.dir_secs_max = self.dir_secs_max.max(self.dir_secs_min);
        self.move_secs_min = self.move_secs_min.max(0.01);
        self.move_secs_max = self.move_secs_max.max(self.move_secs_min);
        self.halt_secs_min = self.halt_secs_min.max(0.01);
        self.halt_secs_max = self.halt_secs_max.max(self.halt_secs_min);
        self.speed_min = self.speed_min.max(0.0);
        self.speed_max = self.speed_max.max(self.speed_min);
        self.pack_orbit_radius = self.pack_orbit_radius.max(0.0);
        self.pack_orbit_retarget_secs_min = self.pack_orbit_retarget_secs_min.max(0.0);
        self.pack_orbit_retarget_secs_max = self
            .pack_orbit_retarget_secs_max
            .max(self.pack_orbit_retarget_secs_min);
        self.avoid_entity_radius = self
            .avoid_entity_radius
            .into_iter()
            .map(|(tag, radius)| (tag, radius.max(0.0)))
            .collect();
        self.avoid_entity_strength = self
            .avoid_entity_strength
            .into_iter()
            .map(|(tag, strength)| (tag, strength.max(0.0)))
            .collect();
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.dir_secs_min == 0.0
            && self.dir_secs_max == 0.0
            && self.move_secs_min == 0.0
            && self.move_secs_max == 0.0
            && self.halt_secs_min == 0.0
            && self.halt_secs_max == 0.0
            && self.speed_min == 0.0
            && self.speed_max == 0.0
            && self.avoid_tile_tags.is_empty()
            && self.avoid_bit_tags.is_empty()
            && self.avoid_race_tags.is_empty()
            && self.avoid_pack_tags.is_empty()
            && self.max_drift.is_nan()
            && self.pack_orbit_radius == 0.0
            && self.pack_orbit_retarget_secs_min == 0.0
            && self.pack_orbit_retarget_secs_max == 0.0
            && !self.wander_around_leader
            && self.avoid_entity_radius.is_empty()
            && self.avoid_entity_strength.is_empty()
    }

    pub fn avoid_entity_radius_for(&self, tag: &str) -> f32 {
        self.avoid_entity_radius
            .get(tag)
            .copied()
            .unwrap_or(DEFAULT_AVOID_ENTITY_RADIUS)
    }

    pub fn avoid_entity_strength_for(&self, tag: &str) -> f32 {
        self.avoid_entity_strength
            .get(tag)
            .copied()
            .unwrap_or(DEFAULT_AVOID_ENTITY_STRENGTH)
    }

    pub fn max_avoid_entity_radius(&self) -> f32 {
        self.avoid_entity_radius
            .values()
            .copied()
            .fold(DEFAULT_AVOID_ENTITY_RADIUS, f32::max)
    }
}



/*
*/
