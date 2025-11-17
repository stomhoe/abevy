
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
        (apply_offsets, apply_scales, (become_child_of_sprite_with_category.run_if(in_state(ClientState::Disconnected)), 
            insert_sprite_to_instance,).chain()),

        (replace_string_ids_by_entities, add_spritechildren_and_comps, ).run_if(in_state(ClientState::Disconnected))
    ).in_set(SpriteSystems))
    .configure_sets(SPRITES_SCHEDULE, SpriteSystems.in_set(StatefulSessionSystems))
    
    .add_systems(OnEnter(AssetsLoadingState::LocalFinished), (
        (init_sprite_cfgs, add_sprites_to_local_map).chain(),
    ).in_set(SpriteSystems)) 

    .replicate_with((
        (RuleFns::<ChildOf>::default(), ReplicationMode::OnChange),
        (RuleFns::<Transform>::default(), ReplicationMode::Once),
        (RuleFns::<SpriteHolderRef>::default(), ReplicationMode::OnChange),
        (RuleFns::<SpriteConfigRef>::default(), ReplicationMode::OnChange),
    ))

    // .replicate_with((
    //     (RuleFns::<SpriteHolderRef>::default(), ReplicationMode::OnChange),
    //     (RuleFns::<ChildOf>::default(), ReplicationMode::OnChange),
    // ))

    .register_type::<SpriteHolderRef>()
    .register_type::<HeldSprites>()
    .register_type::<SpriteSerisHandles>()
    .register_type::<SpriteConfigSeri>()
    .register_type::<SpriteCfgEntityMap>()
    .register_type::<SpriteHolderRef>()
    .register_type::<SpriteConfigRef>()
    .register_type::<Offset2D>().register_type::<OffsetUpDown>().register_type::<OffsetDown>()
    .register_type::<OffsetUp>()
    .register_type::<OffsetSideways>()
    .register_type::<Scale2D>()
    .register_type::<ScaleLookUp>()
    .register_type::<ScaleLookDown>()
    .register_type::<ScaleLookUpDown>()
    .register_type::<ScaleSideways>()
    //.add_server_event::<SpriteCfgEntityMap>(Channel::Unordered).make_event_independent::<SpriteCfgEntityMap>().add_observer(client_map_server_sprite_cfgs)

   
    ;
}

