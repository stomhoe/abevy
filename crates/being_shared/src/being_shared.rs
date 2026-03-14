
use bevy::{ecs::entity::EntityHashMap, prelude::*};
use bevy::platform::collections::HashSet;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use common::common_tag_components::TagSet;
use game_common::game_common_samplers::EntityWeightedSampler;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;

#[derive(Component, Debug, Default, Clone)]
pub struct ComputedLocally;

#[derive(Component, Debug, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Being;
impl Being {
    pub const Z_LEVEL: f32 = 1_000.;
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
pub struct BodyCollisionRadius(pub u32);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
pub struct BodyTreeWeightSum(pub f32);

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq)]
pub struct MainCharacter{#[entities] created_by: Entity}

#[derive(Component, Debug, Default, Clone, Copy, Hash, PartialEq)]
pub struct InfiniteMorale;

#[derive(Component, Default, Deserialize, Serialize, Clone)]
pub struct PlayerDirectControllable;

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
impl Followers {pub fn entities(&self) -> &Vec<Entity> {&self.0}}

#[derive(Component, Debug, Clone)]
pub struct LearningMultiplier(pub EntityHashMap<f32>);

#[derive(Component, Debug, Default, Clone)]
pub struct TargetSpawnPos(pub Vec2);//NO SÉ SI PONERLE UN FIELD Q SEA LA DIMENSIÓN
impl TargetSpawnPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Reflect, MapEntities, Copy, Clone, )]
#[relationship(relationship_target = CreatedCharacters)]
#[require(PlayerDirectControllable, )]
pub struct CharacterCreatedBy {
    #[relationship] #[entities] pub player: Entity,
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = CharacterCreatedBy)]
pub struct CreatedCharacters(Vec<Entity>);
impl CreatedCharacters { pub fn entities(&self) -> &[Entity] { &self.0 } }


#[derive(Component, Debug, Clone, )]
pub struct MappedSpritesToSample(
    /// sexent - samplespriteents
    pub EntityHashMap<SampleSpriteEnts>,
);

#[derive(Resource, Debug, Default, Clone)]
pub struct BiomeHidPackSamplers(pub bevy::platform::collections::HashMap<HashId, EntityWeightedSampler>);

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
    pub const SERI_SENTINEL: f32 = -1.0;
    pub fn is_configured_in_seri(value: f32) -> bool {
        value > Self::SERI_SENTINEL
    }
}
