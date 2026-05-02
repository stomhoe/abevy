use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use game_common::Dead;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;
use faction_shared::BelongsToAPlayerFaction;
use tilemap_shared::*;
use crate::being_inst_templ_shared::{DontExtendBitSpawnBlacklist, DontExtendBitSpawnWhitelist, DontExtendRaceSpawnBlacklist, DontExtendRaceSpawnWhitelist};
use crate::{RaceEntityMap, RaceRef};

#[derive(Component, Debug, Default, Clone)]
pub struct ComputedLocally;

#[derive(Component, Debug, Copy, Clone, Default, Deserialize, Serialize)]
#[require(Prefix::trunc("Being"),)]
pub struct Being;
impl Being {
    pub const Z_LEVEL: f32 = 1_000.;

    fn resolve_spawn_tile_tag_source(
        being_ent: Entity,
        bit_ent: Option<Entity>,
        race_ent: Option<Entity>,
        race_map: &RaceEntityMap,
        bit_race_query: &Query<&RaceRef>,
        spawn_tile_tags_query: &Query<(
            Option<&WhitelistedSpawnTileTags>,
            Option<&BlacklistedSpawnTileTags>,
        )>,
    ) -> Option<Entity> {
        let has_spawn_tags = |ent: Entity| {
            let Ok((whitelist, blacklist)) = spawn_tile_tags_query.get(ent) else {
                return false;
            };
            whitelist.is_some() || blacklist.is_some()
        };

        if has_spawn_tags(being_ent) {
            return Some(being_ent);
        }

        if let Some(bit_ent) = bit_ent {
            if has_spawn_tags(bit_ent) {
                return Some(bit_ent);
            }
        }

        race_ent.or(
            bit_ent.and_then(|bit_ent| {
                bit_race_query
                    .get(bit_ent)
                    .ok()
                    .and_then(|race_ref| race_map.0.get_cloned(race_ref.0).ok())
            }),
        )
    }

    fn apply_spawn_tile_tags(
        source_ent: Entity,
        spawn_tile_tags_query: &Query<(
            Option<&WhitelistedSpawnTileTags>,
            Option<&BlacklistedSpawnTileTags>,
        )>,
        whitelisted_tags: &mut WhitelistedSpawnTileTags,
        blacklisted_tags: &mut BlacklistedSpawnTileTags,
    ) -> bool {
        let Ok((source_whitelist, source_blacklist)) = spawn_tile_tags_query.get(source_ent) else {
            return false;
        };
        let has_any = source_whitelist.is_some() || source_blacklist.is_some();
        if !has_any {
            return false;
        }
        if let Some(source_whitelist) = source_whitelist {
            whitelisted_tags.extend_from(source_whitelist);
        }
        if let Some(source_blacklist) = source_blacklist {
            blacklisted_tags.extend_from(source_blacklist);
        }
        true
    }

    pub fn select_spawn_tile_tag_filters(
        being_ent: Entity,
        bit_ent: Option<Entity>,
        race_ent: Option<Entity>,
        race_map: &RaceEntityMap,
        bit_race_query: &Query<&RaceRef>,
        spawn_tile_tags_query: &Query<(Option<&WhitelistedSpawnTileTags>, Option<&BlacklistedSpawnTileTags>, )>,
        being_spawn_tag_extension_query: &Query<(Has<DontExtendBitSpawnWhitelist>, Has<DontExtendBitSpawnBlacklist>, Has<DontExtendRaceSpawnWhitelist>, Has<DontExtendRaceSpawnBlacklist>, )>,
        whitelisted_tags: &mut WhitelistedSpawnTileTags,
        blacklisted_tags: &mut BlacklistedSpawnTileTags,
    ) {
        whitelisted_tags.clear();
        blacklisted_tags.clear();

        let Some(source_ent) = Self::resolve_spawn_tile_tag_source(
            being_ent,
            bit_ent,
            race_ent,
            race_map,
            bit_race_query,
            spawn_tile_tags_query,
        ) else {
            return;
        };

        let Ok((dont_extend_bit_spawn_whitelist, dont_extend_bit_spawn_blacklist, dont_extend_race_spawn_whitelist, dont_extend_race_spawn_blacklist)) = being_spawn_tag_extension_query.get(being_ent) else {
            return;
        };

        let resolved_race_ent = race_ent.or(
            bit_ent.and_then(|bit_ent| {
                bit_race_query
                    .get(bit_ent)
                    .ok()
                    .and_then(|race_ref| race_map.0.get_cloned(race_ref.0).ok())
            }),
        );
        if !Self::apply_spawn_tile_tags(source_ent, spawn_tile_tags_query, whitelisted_tags, blacklisted_tags) {
            return;
        }
        let dont_extend_bit_spawn_tags = dont_extend_bit_spawn_whitelist || dont_extend_bit_spawn_blacklist;
        let dont_extend_race_spawn_tags = dont_extend_race_spawn_whitelist || dont_extend_race_spawn_blacklist;

        if source_ent == being_ent {
            if !dont_extend_bit_spawn_tags && let Some(bit_ent) = bit_ent {
                let _ = Self::apply_spawn_tile_tags(bit_ent, spawn_tile_tags_query, whitelisted_tags, blacklisted_tags);
            }
            if !dont_extend_race_spawn_tags && let Some(race_ent) = resolved_race_ent {
                let _ = Self::apply_spawn_tile_tags(race_ent, spawn_tile_tags_query, whitelisted_tags, blacklisted_tags);
            }
        } else if bit_ent.is_some_and(|bit_ent| source_ent == bit_ent) && !dont_extend_race_spawn_tags && let Some(race_ent) = resolved_race_ent {
            let _ = Self::apply_spawn_tile_tags(race_ent, spawn_tile_tags_query, whitelisted_tags, blacklisted_tags);
        }
        blacklisted_tags.retain(|tag| !whitelisted_tags.contains(tag.clone()));
    }
}
pub type AliveBeing = (With<Being>, Without<Dead>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct HumanControlled;

#[derive(Component, Debug, Default, Clone)]
pub struct AiAutoMeleeTargets(pub Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct LodLevel(pub u8);

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

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyWeightSum(pub f32);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct MainCharacter;

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
    pub EntityHashMap<SampleSpritesamplers>,
);

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

#[derive(Component, Debug, Clone, Copy, )]
pub struct NaturalSpawnOrigin(pub ChunkPos);


pub type BeingOfPlayerFaction = (With<Being>, With<BelongsToAPlayerFaction>);

pub type LoadedBeing = (With<Being>, Without<Unloaded>);
pub type UnloadedBeing = (With<Being>, With<Unloaded>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(Replicated, )]//HACER Q MEJOR ESTO SE REGISTRE EN EL CHUNK PARA NO TENER QUE QUERYEAR TODA TILE O BIENG ADENTRO
pub struct PreventsChunkUnloading;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct ChunkPersistersWithin(pub u32);
