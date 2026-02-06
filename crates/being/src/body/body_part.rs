#[allow(unused_imports)] use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy_replicon::prelude::*;

pub use body_part_components::*;
pub use body_part_resources::*;

pub mod body_part_components;
pub mod body_part_resources;
mod body_part_init_systems;

pub use body_part_init_systems::*;
use common::common_states::AssetLoading;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct BodyPartSystems;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
		RonAssetPlugin::<BodyPartSeri>::new(&["bodypart.ron"]),

        plugin_body_part_entity_map,
    ))
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
		(init_body_parts, map_body_part_entity_map_id_to_entity).chain().in_set(BodyPartSystems),
    ))
    .add_systems(Update, map_body_part_entity_map_id_to_entity)
    .register_type::<BodyPart>()
    .register_type::<BodyPartOf>()
    .register_type::<BodyPartParent>()
    .register_type::<BodyPartDepth>()
    .register_type::<BodyPartKind>()
    .register_type::<BodyPartVital>()
    .register_type::<BodyPartMissing>()
    .register_type::<BodyPartDamage>()
    .replicate::<BodyPart>()
    .replicate::<BodyPartOf>()
    .replicate::<BodyPartParent>()
    .replicate::<BodyRootPart>()
    .replicate::<BodyPartVital>()
    .replicate::<BodyPartMissing>()
    .replicate::<BodyPartDamage>()
    ;
}
