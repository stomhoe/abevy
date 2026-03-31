#[allow(unused_imports)] use bevy::prelude::*;
use smallvec::SmallVec;
use tilemap_shared::*;



#[derive(Message, Debug, Clone, )]
pub struct UnfreezeBeing(pub Entity);

#[derive(Message, Debug, Clone, )]
pub struct FaithfulSimBeing(pub Entity);

#[derive(Debug, Clone, Copy, Default, )]
pub enum SquadSpawnMode {
    #[default]
    AutoFromTemplateFlag,
    ForceSpawn,
    DontSpawn,
}

#[derive(Message, Debug, Clone, )]
pub struct InstantiateTemplPackEntity {
    pub source_ent: Entity,
    pub override_being_count: Option<u16>,
    pub sampled_count_mult: Option<f32>,
    pub dim_ref: DimensionRef,
    pub member_gpos: SmallVec<[GlobalTilePos; 4]>,
    pub only_same_island: bool,
    pub squad_spawn_mode: SquadSpawnMode,
}
impl InstantiateTemplPackEntity {
    pub fn new(
        source_ent: Entity,
        override_being_count: Option<u16>,
        sampled_count_multiplier: Option<f32>,
        dim_ref: DimensionRef,
        member_gpos: impl IntoIterator<Item = GlobalTilePos>,
    ) -> Self {
        Self {
            source_ent,
            override_being_count,
            sampled_count_mult: sampled_count_multiplier,
            dim_ref,
            member_gpos: member_gpos.into_iter().collect(),
            only_same_island: false,
            squad_spawn_mode: SquadSpawnMode::default(),
        }
    }
}
