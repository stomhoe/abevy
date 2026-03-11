
use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
use bevy::prelude::*;
use ac_audio::AcAudioSystems;
use common::common_states::AssetLoading;
use game_common::game_common::SimRunningSystems;
use sprite_systems::AcSpriteSystems;
use ::sprite_animation_shared::*;
use crate::{sprite_animation_init_systems::*, sprite_animation_systems::*};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteAnimationSystems;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        SpritesheetAnimationPlugin,
        plugin_ac_animation,
    ))
    .add_systems(Update, ((
           animate_sprite,
           update_animstate_for_clients
               .run_if(in_state(ServerState::Running))
               .after(animate_sprite),
           client_receive_moving_anim
               .run_if(in_state(ClientState::Connected))
               .run_if(on_message::<SyncMoveState>)
               .after(update_animstate_for_clients),
        ).in_set(SpriteAnimationSystems),
    ))
    .add_systems(FixedUpdate, ((//está en fixed update para q no le afecte lo de SimRunningSystems del SpriteAnimationSystems
            init_animation_sheet_and_handle,
        ).in_set(SpriteAnimationSystems),
    ))
    .configure_sets(Update, (SpriteAnimationSystems.in_set(SimRunningSystems).after(AcAudioSystems),))

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        SpriteAnimationSystems.before(AcSpriteSystems)
    ))

    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        (init_animations, map_ac_animation_id_to_entity).chain()
    ).in_set(SpriteAnimationSystems))

    .add_mapped_server_message::<SyncMoveState>(Channel::Ordered)

    .add_message::<BeingChangedMoveState>()

    .replicate_once::<AnimExtraState>()
    .replicate_once::<MoveAnimActive>()
    .replicate_filtered::<ChildOf, With<AcAnimation>>()
    .replicate::<AnimationSeri>()
    .replicate::<ClipStartFrames>()
    .replicate::<AlternatingStartFramesConfig>()
    .replicate_once::<PlayingSpeed>()


    //.replicate::<AlternatingStartFramesState>()
    ;
}
