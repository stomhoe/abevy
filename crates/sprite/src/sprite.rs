use ::sprite_shared::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use common::{
    common_states::{AppState, AssetLoading}
};
use game_common::{
    StatefulSessionSystems, game_common::GameplaySystems,
};

use crate::{
    sprite_build_systems::*,
    sprite_components::*,
    sprite_config_init_systems::*,
    sprite_resources::*,
    sprite_sampler::{self, SpriteSamplerSystems, sprite_sampler_systems::*},
    sprite_systems::*,
    sprite_scale_systems::*,
    sprite_offset_systems::*,
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
            disable_children_sprites_of_disabled,
            (
                apply_offsets
                    .after(sprite_change_detection)
                    .run_if(on_message::<SpriteChanged>),
                apply_scales
                    .after(sprite_change_detection)
                    .run_if(on_message::<SpriteChanged>),
            ),
            z_sort_system,
            (
                become_child_of_sprite_with_tag,
                add_spritechildren_and_comps,
                remap_broken_sprite_config_refs_after_hotreload,
            )
            .run_if(
                in_state(ClientState::Disconnected)
                    .and(in_state(AppState::StatefulGameSession)),
            ),
        )
            .in_set(AcSpriteSystems),
    )
    .add_systems(Update, replace_sampler_string_ids_by_entities)
    .configure_sets(
        FixedUpdate,
        AcSpriteSystems.in_set(StatefulSessionSystems),
    )
    .add_systems(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        ((init_sprite_configs, map_sprite_config_id_to_entity).chain())
            .in_set(AcSpriteSystems),
    )
    .configure_sets(
        OnEnter(AssetLoading::SpawnReplicatedEntities),
        (
            AcSpriteSystems.before(SpriteSamplerSystems),
            SpriteSamplerSystems.before(GameplaySystems),
        ),
    )
    .replicate::<AcZ>()
    .replicate::<MappedAnimations>()
    .replicate::<SpriteLoopSfx>()
    .replicate::<SpriteTimedSfx>()
    .replicate::<OffsetForChildren>()

    .replicate::<YSortOrigin>()
    .replicate::<BaseHolderRef>()
    .replicate::<MovementBased>()
    .replicate::<GroundingBased>()
    .replicate::<ExcludedFromNormalSizeModifier>()

    .replicate_filtered::<ChildOf, With<SpriteConfig>>()
    .replicate_filtered::<Transform, With<SpriteConfig>>()

    .replicate_filtered::<ChildOf, With<BaseHolderRef>>()

    .add_message::<SpriteChanged>()

    ;
}
