
use bevy::ecs::entity_disabling::Disabled;
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_states::{AssetsLoadingState, };
use game_common::{game_common::GameplaySystems, StatefulSessionSystems, };

use crate::{sprite_components::*, sprite_init_systems::*, sprite_resources::*, sprite_scale_offset_components::*, sprite_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*,};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystems;


const SPRITES_SCHEDULE: Update = Update;


pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<SpriteConfigSeri>::new(&["sprite.ron"]),
    ))
    .add_systems(SPRITES_SCHEDULE, (
        disable_children_sprites_of_disabled,
        (apply_offsets, apply_scales, ).chain(),

        // server only
        (become_child_of_sprite_with_category, replace_string_ids_by_entities, 
            add_spritechildren_and_comps, ).run_if(in_state(ClientState::Disconnected))
    ).in_set(SpriteSystems))
    .configure_sets(SPRITES_SCHEDULE, SpriteSystems.in_set(StatefulSessionSystems))
    
    .add_systems(OnEnter(AssetsLoadingState::ReplicatedFinished), (
        (init_sprite_cfgs, add_sprites_to_local_map).chain(),
    ).in_set(SpriteSystems)) 

    .register_type::<BaseHolderRef>()
    .register_type::<HeldSprites>()
    .register_type::<SpriteSerisHandles>()
    .register_type::<SpriteConfigSeri>()
    .register_type::<SpriteCfgEntityMap>()
    .register_type::<SpriteConfigRef>()
    .register_type::<Offset2D>()
    .register_type::<OffsetUpDown>()
    .register_type::<OffsetDown>()
    .register_type::<OffsetUp>()
    .register_type::<OffsetSideways>()
    .register_type::<OffsetForChildren>()
    .register_type::<Scale2D>()
    .register_type::<ScaleLookUp>()
    .register_type::<ScaleLookDown>()
    .register_type::<ScaleLookUpDown>()
    .register_type::<ScaleSideways>()
    .register_type::<SpriteCfgAnimationsMap>()
    .register_type::<SpriteConfigStrIds>()
    .register_type::<SpriteConfigsHolder>()
    
    //.add_server_event::<SpriteCfgEntityMap>(Channel::Unordered).make_event_independent::<SpriteCfgEntityMap>().add_observer(client_map_server_sprite_cfgs)

    .replicate::<SpriteConfig>()
    .replicate::<SpriteCfgAnimationsMap>()
    .replicate::<MovementBased>()
    .replicate::<GroundingBased>()
    .replicate::<SpriteConfigRef>()
    .replicate::<BaseHolderRef>()
    .replicate_filtered::<ChildOf, Or<(With<SpriteConfig>, With<BaseHolderRef>, With<SpriteConfigRef>)>>()
    .replicate_once_filtered::<Transform, Or<(With<SpriteConfig>, With<BaseHolderRef>, With<SpriteConfigRef>)>>()
    .replicate::<SpriteConfigsHolder>()
    .replicate::<Offset2D>()
    .replicate::<OffsetUpDown>()
    .replicate::<OffsetDown>()
    .replicate::<OffsetUp>()
    .replicate::<OffsetSideways>()
    .replicate::<OffsetForChildren>()
    .replicate::<Scale2D>()
    .replicate::<ScaleLookUp>()
    .replicate::<ScaleLookDown>()
    .replicate::<ScaleLookUpDown>()
    .replicate::<ScaleSideways>()
    
    ;
}

