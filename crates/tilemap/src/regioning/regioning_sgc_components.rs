#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use game_common::game_common_components::ArgsDict;

use bevy::{ecs::entity::MapEntities, };

use common::{common_components::*, };
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Deserialize, Serialize, Clone, )]
#[require(SparedFromHotReloading, AssetScoped, Replicated, Prefix::trunc("SGC"), )]
pub struct StructuredGenConfig{
    // the structure's id, not the sgc's
    structure_id: StrId,
    /// the structure's HaId, not the sgc's
    structure_hash_id: HashId,
    pub max_per_region: u32,
    pub args: ArgsDict,
}
impl StructuredGenConfig {
    pub fn new<S: AsRef<str>>(structure_id: S) -> Self {
        Self {
            structure_id: StrId::trunc(structure_id.as_ref()),
            structure_hash_id: HashId::hash(structure_id.as_ref()),
            max_per_region: 1024,
            args: ArgsDict::default(),
        }
    }
    pub fn structure_id(&self) -> &StrId {
        &self.structure_id
    }
    pub fn structure_hash_id(&self) -> HashId {
        self.structure_hash_id
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, MapEntities)]
#[relationship(relationship_target = AcceptedFilters)]
pub struct WhitelistedFilterOf {
    #[relationship] #[entities]
    pub structured_gen_cfg: Entity,
}
impl WhitelistedFilterOf{
    pub fn new(structured_gen_cfg: Entity) -> Self {
        Self { structured_gen_cfg }
    }
}

#[derive(Component, Debug, )]
#[relationship_target(relationship = WhitelistedFilterOf)]
pub struct AcceptedFilters(Vec<Entity>);
impl AcceptedFilters { pub fn entities(&self) -> &[Entity] { &self.0 } }

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
#[require(SparedFromHotReloading, AssetScoped, Replicated, Prefix::trunc("SGCsWeightedSampler"), )]
pub struct SgcsWeightedSampler;
