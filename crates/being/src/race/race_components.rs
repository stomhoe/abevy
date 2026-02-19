use bevy::{ecs::entity::{EntityHashSet, MapEntities}, platform::collections::HashSet};
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_samplers::EntityWeightedSampler;
use serde::{Deserialize, Serialize};

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
