use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy_common_assets::ron::RonAssetPlugin;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use common::{common_components::StrId, common_states::{AppState, AssetLoading }, define_entity_map_systems};
use game_common::{StatefulSessionSystems, game_common::GameplaySystems, game_common_components::EntityZero };
use ::sprite_shared::*;

use crate::{sprite_building_systems::*, sprite_components::*, sprite_config_init_systems::*, sprite_resources::*, sprite_sampler::{self, SpriteSamplerSystems, sprite_sampler_systems::*}, sprite_systems::*};

#[allow(unused_imports)] use {bevy::prelude::*,};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct AcSpriteSystems;

define_entity_map_systems!(
    SpriteCfgEntityMap,
    StrId,
    SpriteConfig,
    (With<EntityZero>, )
);
const SPRITES_SCHEDULE: Update = Update;

#[allow(unused_parens, )]
pub fn plugin(app: &mut App) {
    app
    .add_plugins((
        RonAssetPlugin::<SpriteConfigSeri>::new(&["sprite.ron"]),
        sprite_sampler::plugin,
        plugin_sprite_cfg_entity_map,
    ))
    .add_systems(SPRITES_SCHEDULE, (
        disable_children_sprites_of_disabled,
        (apply_offsets, apply_scales, ).run_if(on_timer(Duration::from_millis(10))),
        z_sort_system,
        // server only
        (become_child_of_sprite_with_tag, replace_string_ids_by_entities, 
            add_spritechildren_and_comps, ).run_if(in_state(ClientState::Disconnected)
            .and(in_state(AppState::StatefulGameSession)))
            
    ).in_set(AcSpriteSystems))
    .add_systems(Update, replace_sampler_string_ids_by_entities)
    .configure_sets(SPRITES_SCHEDULE, AcSpriteSystems.in_set(StatefulSessionSystems))
    
    .add_systems(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        (init_sprite_configs, map_sprite_cfg_entity_map_id_to_entity).chain()
    ).in_set(AcSpriteSystems)) 

    .configure_sets(OnEnter(AssetLoading::SpawnReplicatedEntities), (
        AcSpriteSystems.before(SpriteSamplerSystems),
        SpriteSamplerSystems.before(GameplaySystems)
    ))
    
    
    .register_type::<AcZ>()
    .register_type::<SpriteSerisHandles>()
    .register_type::<SpriteConfigSeri>()
    .register_type::<MappedAnimations>()
    .register_type::<SpriteConfigsHolder>()
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

    .replicate::<SpriteConfigsHolder>()

    
    ;
}

