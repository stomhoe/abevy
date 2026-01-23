#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use serde::{Deserialize, Serialize};
use bevy::ecs::entity::MapEntities;



use common::{common_components::*, };


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(AssetScoped, Prefix::trunc("EguiSgcHolder"), Replicated, SessionScoped, TgenHotLoadingScoped)]
pub struct EguiSgcHolder;

#[derive(Component, Debug, Deserialize, Serialize, Clone, Reflect)]
#[require(Replicated, Prefix::trunc("SGC"), AssetScoped, SessionScoped, TgenHotLoadingScoped, )]
pub struct StructuredGenConfig{
    pub structure_id: StrId,
    pub hash: HashId,
    pub max_per_region: u32,
    pub args: Vec<String>,
}
impl Default for StructuredGenConfig {
    fn default() -> Self {
        Self { structure_id: StrId::default(), hash: HashId::default(), max_per_region: 1024, args: Vec::new()  }
    }
}

#[derive(Component, Debug, Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq, Reflect, MapEntities)]
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

#[derive(Component, Debug, Reflect)]
#[relationship_target(relationship = WhitelistedFilterOf)]
pub struct AcceptedFilters(Vec<Entity>);
impl AcceptedFilters { pub fn entities(&self) -> &[Entity] { &self.0 } }


#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, Reflect)]
#[require(Replicated, Prefix::trunc("SGCsEntityWeightedMap"), AssetScoped, SessionScoped, TgenHotLoadingScoped, )]
pub struct SgcsEntityWeightedMap;

