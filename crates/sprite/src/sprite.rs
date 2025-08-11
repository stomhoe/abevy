
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::states::LocalAssetsLoadingState;
use game_common::game_common::GameplaySystems;


use crate::{sprite_init_systems::*, sprite_resources::*, sprite_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*,};



//mod animation_events;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SpriteSystemsSet;


pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        SpritesheetAnimationPlugin::default(), 
        RonAssetPlugin::<SpriteConfigSeri>::new(&["sprite.ron"]),
    ))
    .add_systems(Update, (
        (apply_offsets, apply_scales, ).in_set(GameplaySystems),

        (
            (replace_string_ids_by_entities, add_spritechildren_and_comps, ).run_if(server_or_singleplayer), 
            become_child_of_sprite_with_category, insert_sprite_to_instance,
        ).run_if(
            in_state(LocalAssetsLoadingState::Finished)
        )
    ).in_set(SpriteSystemsSet))
    
    .add_systems(OnEnter(LocalAssetsLoadingState::Finished), (
        init_sprite_cfgs.before(add_sprites_to_local_map,),
        add_sprites_to_local_map,
    ).in_set(SpriteSystemsSet)) 



    

    ;
}

