
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::common_states::{LocalAssetsLoadingState, ReplicatedAssetsLoadingState};
use game_common::{game_common::GameplaySystems, SpriteSystemsSet};
use sprite_shared::sprite_shared::{SpriteCfgEntityMap, SpriteHolderRef};

use bevy_asset_loader::prelude::*;

use crate::{sprite_init_systems::*, sprite_resources::*, sprite_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*,};




pub fn plugin(app: &mut App) {
    app
    .add_plugins((
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

    .register_type::<SpriteCfgEntityMap>()
    .register_type::<SpriteHolderRef>()

    .configure_loading_state(
        LoadingStateConfig::new(LocalAssetsLoadingState::InProcess)
        .load_collection::<SpriteSerisHandles>()
        // .load_collection::<AnimSerisHandles>()
        // .load_collection::<RaceSerisHandles>()

        .finally_init_resource::<SpriteCfgEntityMap>()
    )

    ;
}

