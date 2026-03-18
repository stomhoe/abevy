use bevy::ecs::entity::{EntityHashMap, EntityHashSet, MapEntities};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use game_common::game_common_samplers::{EntityWeightedSampler, SpriteGlobalNormalDist};
use serde::{Deserialize, Serialize};
use tilemap_shared::BlacklistedTags;

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

#[derive(Component, Debug, Clone)]
pub struct WanderConfig {
    pub dir_secs_min: f32,
    pub dir_secs_max: f32,
    pub move_secs_min: f32,
    pub move_secs_max: f32,
    pub halt_secs_min: f32,
    pub halt_secs_max: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub avoid_tile_tags: BlacklistedTags,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ProducesStepSfx;

#[derive(Component, Debug, Default, Clone)]
pub struct RaceFootstepSfxConfig {
    pub paths: Vec<String>,
    pub disable_tile_step_sfx: bool,
}
