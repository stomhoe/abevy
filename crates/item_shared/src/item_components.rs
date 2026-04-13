use std::collections::HashMap;

use bevy::ecs::entity::{EntityHashMap, EntityHashSet, EntityMapper, MapEntities};
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::common_components::*;
use common::common_tag_components::TagSet;
use tilemap_shared::tilemap_shared_samplers::EntityCountMapWeightedSampler;
use crate::{ItemEntityMap, GeneratedItemsSeri, SlottedItemHolderSeri};
use serde::{Deserialize, Serialize};
use ::sprite_shared::*;
use tilemap_shared::SnapTransformToGpos;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Default)]
#[require(Replicated, Prefix::trunc("Item"), AssetScoped, Visibility, SnapTransformToGpos::OnChange, )]
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
pub type DroppedItem = (With<Item>, Without<ItemHeldIn>);

#[derive(Component, Debug, )]
#[relationship_target(relationship = ItemHeldIn)]
pub struct HeldItems(Vec<Entity>);

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct DropHeldItemsOnDowned;
//puede ser lvl o strength o afeccion. no aplica para ponerlo en backpack
#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct WearRequirements;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct MeleeStrikeTool;
//.replicate::<MeleeStrikeTool>()

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone)]
pub struct SlotableIn(pub TagSet);

#[derive(Component, Debug, Clone, )]
pub struct ItemsGeneratedOnDeath { pub sampler: EntityCountMapWeightedSampler, pub count_multiplier: f32 }

impl Default for ItemsGeneratedOnDeath {
    fn default() -> Self {
        Self { sampler: EntityCountMapWeightedSampler::default(), count_multiplier: 1.0 }
    }
}

impl ItemsGeneratedOnDeath {
    pub fn from_gen_on_death_seri(
        seri: &GeneratedItemsSeri,
        item_map: &ItemEntityMap,
        all_drop_seris: &HashMap<String, GeneratedItemsSeri>,
    ) -> Self {
        let mut visited = std::collections::HashSet::new();
        let mut weights: Vec<(EntityHashMap<u32>, f32)> = Vec::new();
        Self::append_weights_from_seri(seri, item_map, all_drop_seris, &mut visited, 1.0, &mut weights);
        let (sampler, negative_items) = EntityCountMapWeightedSampler::new(&weights);
        for negative_item in negative_items {
            error!(target: "item_components", "Weighted sampler {} encountered a negative weight for value {:?}; rejected", &seri.id, negative_item);
        }
        Self { sampler, count_multiplier: seri.count_multiplier.max(0.0) }
    }

    fn append_weights_from_seri(
        seri: &GeneratedItemsSeri,
        item_map: &ItemEntityMap,
        all_drop_seris: &HashMap<String, GeneratedItemsSeri>,
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

#[derive(Component, Debug, Default, Clone)]
/// don't complicate further, define external systems if needed to reject based on extra special conditions
pub struct SlottedItemHolder(pub HashMap<Tag, (EntityHashSet, u32)>);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InsertIntoSlotError {
    LimitReached,
    SlotNotPresent,
}

impl SlottedItemHolder {
    pub fn new(seri: &SlottedItemHolderSeri) -> Self {
        let mut out = HashMap::default();
        for (slot, &limit) in &seri.slots {
            out.insert(Tag::from(slot.as_str()), (EntityHashSet::default(), limit));
        }
        Self(out)
    }

    pub fn insert_into_slot(&mut self, slot: Tag, entity: Entity) -> Result<(), InsertIntoSlotError> {
        let Some((entities, limit)) = self.0.get_mut(&slot) else {
            return Err(InsertIntoSlotError::SlotNotPresent);
        };
        if entities.len() as u32 >= *limit && !entities.contains(&entity) {
            return Err(InsertIntoSlotError::LimitReached);
        }
        entities.insert(entity);
        Ok(())
    }
}

impl MapEntities for SlottedItemHolder {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        for (entities, _) in self.0.values_mut() {
            let mut mapped_entities = EntityHashSet::default();
            for &entity in entities.iter() {
                mapped_entities.insert(entity_mapper.get_mapped(entity));
            }
            *entities = mapped_entities;
        }
    }
}
