use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_states::{AssetsLoadingState, };
use game_common::{game_common::GameplaySystems, StatefulSessionSystems, };
use ::sprite_shared::*;

use crate::{sprite_components::*, sprite_init_systems::*, sprite_resources::*, sprite_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*,};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AcSpriteSystems;


const SPRITES_SCHEDULE: Update = Update;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<SpriteConfigSeri>::new(&["sprite.ron"]),
    ))
    .add_systems(SPRITES_SCHEDULE, (
        disable_children_sprites_of_disabled,
        (apply_offsets, apply_scales, ).run_if(on_timer(Duration::from_millis(10))),

        // server only
        (become_child_of_sprite_with_category, replace_string_ids_by_entities, 
            add_spritechildren_and_comps, ).run_if(in_state(ClientState::Disconnected))
    ).in_set(AcSpriteSystems))
    .configure_sets(SPRITES_SCHEDULE, AcSpriteSystems.in_set(StatefulSessionSystems))
    
    .add_systems(OnEnter(AssetsLoadingState::InitReplicatedEntities), (
        (init_sprite_cfgs, add_sprites_to_local_map).chain(),
    ).in_set(AcSpriteSystems)) 

    .configure_sets(OnEnter(AssetsLoadingState::InitReplicatedEntities), (
       AcSpriteSystems.before(GameplaySystems)
    ))

    
    .register_type::<SpriteSerisHandles>()
    .register_type::<SpriteConfigSeri>()
    .register_type::<SpriteCfgEntityMap>()
    .register_type::<SpriteConfigRef>()
    .register_type::<SpriteCfgAnimationsMap>()
    .register_type::<SpriteConfigsHolder>()
    .register_type::<OffsetForChildren>()
    .register_type::<SpriteConfigNotFound>()
    .register_type::<SpriteConfigUsages>()
    

    .replicate::<SpriteConfig>()
    .replicate::<SpriteConfigNotFound>()
    .replicate::<SpriteCfgAnimationsMap>()

    .replicate::<SpriteConfigRef>()
    .replicate::<OffsetForChildren>()

    .replicate_filtered::<ChildOf, With<SpriteConfig>>()

    .replicate_filtered::<Transform, With<SpriteConfig>>()

    .replicate::<SpriteConfigsHolder>()

    
    ;
}

