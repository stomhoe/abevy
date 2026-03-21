
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy::platform::collections::HashSet;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;
use faction_shared::BelongsToAPlayerFaction;
use tilemap_shared::{BlacklistedSpawnTileTags, WhitelistedSpawnTileTags};

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
pub struct Sentient;


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
pub struct MappedSpritesToSample(
    /// sexent - samplespriteents
    pub EntityHashMap<SampleSpriteEnts>,
);

#[derive(Component, Debug, Deserialize, Serialize, Clone)]
pub struct Predator {
    pub own_races: HashSet<StrId>,
    pub territorialism: f32,
    pub pack_size_min: u32,
    pub pack_size_max: u32,
    pub do_not_hunt_tags: TagSet,
    pub prey_body_size_ratio_tolerance: f32,
}
impl Default for Predator {
    fn default() -> Self {
        Self {
            own_races: HashSet::default(),
            territorialism: 0.0,
            pack_size_min: 1,
            pack_size_max: 1,
            do_not_hunt_tags: TagSet::default(),
            prey_body_size_ratio_tolerance: -1.0,
        }
    }
}

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
pub struct PredatorHuntThreshold(pub f32);
impl Default for PredatorHuntThreshold {
    fn default() -> Self {
        Self(40.0)
    }
}
impl PredatorHuntThreshold {
    pub const SERI_SENTINEL: f32 = f32::NEG_INFINITY;
    pub fn is_configured_in_seri(value: f32) -> bool {
        value > Self::SERI_SENTINEL
    }
}


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

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FactionLeader {
    #[entities]
    pub being: Entity,
}


pub type PlayerBeing = (With<Being>, With<BelongsToAPlayerFaction>);

pub type LoadedBeing = (With<Being>, Without<Unloaded>);
pub type UnloadedBeing = (With<Being>, With<Unloaded>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct PackMemberRank;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(Replicated, )]//HACER Q MEJOR ESTO SE REGISTRE EN EL CHUNK PARA NO TENER QUE QUERYEAR TODA TILE O BIENG ADENTRO
pub struct PreventsChunkUnloading;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct ChunkPersistersWithin(pub u32);
