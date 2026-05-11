use ::sprite_shared::*;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use bevy_replicon::prelude::*;
use common::{
    common_states::{AssetLoading}
};
use game_common::{
    StatefulSessionSystems, game_common::GameplaySystems,
};
use ::tilemap_shared::*;

#[allow(unused_imports, )]
use crate::{
    sprite_build_systems::*,
    sprite_config_init_systems::*,
    sprite_resources::*,
    sprite_sampler::{SpriteSamplerSystems, sprite_sampler_systems::*},
    sprite_systems::*,
    sprite_scale_systems::*,
    sprite_rotation_systems::*,
    sprite_offset_systems::*,
    y_sort_settings::load_y_sort_settings,
    y_sort_system::*,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AcSpriteSystems;


#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::sprite_sampler::plugin,
        plugin_sprite_config,
        sprite_scale_offset::plugin,
    ))
    .add_systems(
        FixedUpdate,
        (
            sprite_change_detection,
            disable_held_sprites_of_disabled,
            (
                apply_offsets,
                apply_rotations,
                apply_scales,
            ).after(sprite_change_detection)
            .run_if(on_message::<SpriteChangedScaleOrOffsetOrParent>),
            become_child_of_sprite_with_tag,
            add_spritechildren_and_comps,
        )
            .in_set(AcSpriteSystems),
    )
    .add_systems(
        PostUpdate,
        y_sort_system
            .after(TransformSystems::Propagate)
            .in_set(StatefulSessionSystems),
    )
    .add_systems(Update, replace_sampler_string_ids_by_entities)
    .configure_sets(
        FixedUpdate,
        AcSpriteSystems.in_set(StatefulSessionSystems),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        ((load_y_sort_settings, init_sprite_configs, map_sprite_config_id_to_entity).chain())
            .in_set(AcSpriteSystems),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            AcSpriteSystems.before(SpriteSamplerSystems),
            SpriteSamplerSystems.before(GameplaySystems),
        ),
    )
    .replicate::<ZSettings>()
    .replicate::<AcZ>()
    .replicate::<MappedAnimations>()
    .replicate::<SpriteLoopSfx>()
    .replicate::<SpriteTimedSfx>()
    .replicate::<OffsetForChildren>()
    .replicate::<Rotation>()
    .replicate::<ScsToBuild>()
    .replicate::<BecomeChildOfSpriteWithTag>()
    .replicate::<CardinalDirectionAffectsRotation>()
    .replicate::<SpriteHoriNormalDist>()
    .replicate::<SpriteVertNormalDist>()
    .replicate::<SpriteGlobalNormalDist>()

    .replicate::<YSortOrigin>()
    .replicate_once::<AddUpAnimAndScAcZ>()
    .replicate::<BaseHolderRef>()
    .replicate_once::<MovementBased>()
    .replicate_once::<GroundingBased>()
    .replicate_once::<ExcludedFromNormalSizeModifier>()

    .replicate_filtered::<ChildOf, With<SpriteConfig>>()
    .replicate_filtered::<Transform, With<SpriteConfig>>()

    .replicate_filtered::<ChildOf, With<BaseHolderRef>>()
    .register_type::<YSortOrigin>()

    .add_message::<SpriteChangedScaleOrOffsetOrParent>()

    ;
}
