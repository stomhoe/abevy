#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

pub use body_part_components::*;
pub use body_part_resources::*;
pub use body_part_seris::*;

pub mod body_part_components;
pub mod body_part_resources;
pub mod body_part_seris;
mod body_part_init_systems;

pub use body_part_init_systems::*;
use common::common_states::AssetLoading;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodyPartSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        plugin_body_part,
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
    (init_body_parts, map_body_part_id_to_entity).chain().in_set(BodyPartSystems),
    ))
    //.register_type::<BodyParts>()
    .replicate::<BodyPart>()
    .replicate::<BodyPartOf>()
    .replicate::<BodyPartParent>()
    .replicate::<BodyRootPart>()
    .replicate::<BodyPartVital>()
    .replicate::<BodyPartMissing>()
    .replicate::<BodyPartDamage>()
    ;
}

#[allow(unused_imports, ambiguous_glob_reexports)]
pub mod prelude {
    pub use super::{
        body_part_components::*,
        body_part_resources::*,
        body_part_seris::*,
    };
}
