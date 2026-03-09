use std::collections::HashMap;

use bevy::ecs::entity::{EntityHashMap, MapEntities};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use game_common::game_common_samplers::EntityCountMapWeightedSampler;
use crate::{ItemEntityMap, ItemsGeneratedOnDeathSeri};
use serde::{Deserialize, Serialize};
use sprite::prelude::*;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Default)]
#[require(Replicated, Prefix::trunc("Item"), AssetScoped, SparedFromHotReloading, Visibility, )]
pub struct Item;
impl Item {
    pub const MIN_ID_LENGTH: u8 = 1;
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, MapEntities)]
pub struct ItemSpritesConfig {
    pub sprite_cfg_per_state: HashMap<StrId, ScRef>,
    #[entities] pub dropped_sprite_cfg: ScRef,
    #[entities] pub icon_sprite_cfg: ScRef,
    pub icon_img_path: String,
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, MapEntities, )]
#[relationship(relationship_target = HeldItems)]
pub struct ItemHeldIn {
    #[relationship] #[entities]
    pub holder: Entity,
}
pub type Dropped = Without<ItemHeldIn>;

#[derive(Component, Debug, )]
#[relationship_target(relationship = ItemHeldIn)]
pub struct HeldItems(Vec<Entity>);
impl HeldItems { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct DropHeldItemsOnDowned;

#[derive(Component, Debug, Clone, )]
pub struct ItemsGeneratedOnDeath { pub sampler: EntityCountMapWeightedSampler, pub count_multiplier: f32 }

impl Default for ItemsGeneratedOnDeath {
    fn default() -> Self {
        Self { sampler: EntityCountMapWeightedSampler::default(), count_multiplier: 1.0 }
    }
}

impl ItemsGeneratedOnDeath {
    pub fn from_gen_on_death_seri(
        seri: &ItemsGeneratedOnDeathSeri,
        item_map: &ItemEntityMap,
        all_drop_seris: &HashMap<String, ItemsGeneratedOnDeathSeri>,
    ) -> Self {
        let mut visited = std::collections::HashSet::new();
        let mut weights: Vec<(EntityHashMap<u32>, f32)> = Vec::new();
        Self::append_weights_from_seri(seri, item_map, all_drop_seris, &mut visited, 1.0, &mut weights);
        Self { sampler: EntityCountMapWeightedSampler::new(&weights), count_multiplier: seri.count_multiplier.max(0.0) }
    }

    fn append_weights_from_seri(
        seri: &ItemsGeneratedOnDeathSeri,
        item_map: &ItemEntityMap,
        all_drop_seris: &HashMap<String, ItemsGeneratedOnDeathSeri>,
        visited: &mut std::collections::HashSet<String>,
        parent_weight: f32,
        out: &mut Vec<(EntityHashMap<u32>, f32)>,
    ) {
        for (item_counts, weight) in &seri.weights {
            if *weight <= 0.0 {
                continue;
            }
            let mut ent_counts = EntityHashMap::with_capacity(item_counts.len());
            for (item_id, count) in item_counts {
                let Ok(item_ent) = item_map.0.get_cloned(item_id.as_str()) else { continue };
                ent_counts.insert(item_ent, *count);
            }
            if ent_counts.is_empty() {
                continue;
            }
            out.push((ent_counts, *weight * parent_weight));
        }

        for (ref_id, ref_weight) in &seri.refs {
            if *ref_weight <= 0.0 {
                continue;
            }
            let ref_id = ref_id.trim();
            if ref_id.is_empty() || !visited.insert(ref_id.to_string()) {
                continue;
            }
            let Some(ref_seri) = all_drop_seris.get(ref_id) else {
                visited.remove(ref_id);
                continue;
            };
            Self::append_weights_from_seri(
                ref_seri,
                item_map,
                all_drop_seris,
                visited,
                parent_weight * *ref_weight,
                out,
            );
            visited.remove(ref_id);
        }
    }
}
