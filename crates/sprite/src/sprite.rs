use std::time::Duration;

use ::sprite_shared::*;
#[allow(unused_imports)]
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{
    common_states::{AppState, AssetLoading},
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
};

#[allow(unused_imports)]
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AcSpriteSystems;

const SPRITES_SCHEDULE: FixedUpdate = FixedUpdate;

#[allow(unused_parens)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        sprite_sampler::plugin,
        plugin_sprite_config,
    ))
    .add_systems(
        SPRITES_SCHEDULE,
        (
            disable_children_sprites_of_disabled,
            (apply_offsets, apply_scales).run_if(on_timer(Duration::from_millis(100))),
            z_sort_system.run_if(on_timer(Duration::from_millis(50))),
            // server only
            (
                become_child_of_sprite_with_tag, //replace_string_ids_by_entities,
                add_spritechildren_and_comps,
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
        SPRITES_SCHEDULE,
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
    .register_type::<AcZ>()
    .register_type::<SpriteSerisHandles>()
    .register_type::<SpriteConfigSeri>()
    .register_type::<MappedAnimations>()
    .register_type::<EguiSpriteConfigsHolder>()
    .register_type::<OffsetForChildren>()
    .register_type::<SpriteConfigNotFound>()
    .replicate::<AcZ>()
    .replicate::<SpriteConfig>()
    .replicate::<SpriteConfigNotFound>()
    .replicate::<MappedAnimations>()
    .replicate::<YSortOrigin>()
    //.replicate::<SpriteConfigRef>()
    .replicate::<OffsetForChildren>()
    .replicate_filtered::<ChildOf, With<SpriteConfig>>()
    .replicate_filtered::<Transform, With<SpriteConfig>>()
    .replicate::<EguiSpriteConfigsHolder>();
}
