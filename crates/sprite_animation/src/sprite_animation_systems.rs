

use bevy_replicon::prelude::*;
use being_shared::{Grounding, ControlledBy};
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::*;
use common::{common_components::StrId, common_states::GameSetupType};
use game_common::game_common_components::{Directionable, FacingDirection};
use player::player_components::*;
use sprite::sprite_components::*;
use sprite_animation_shared::{AnimationState, MoveAnimActive};

use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*};


#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens)]
pub fn propagate_holder_state(
    bases_query: Query<(&FacingDirection, &MoveAnimActive, &Grounding, &HeldSprites), (Or<(Changed<Grounding>, Changed<MoveAnimActive>, Changed<FacingDirection>, Changed<HeldSprites>)>,) >,
    mut sprite_query: Query<(&StrId, &SpriteConfigRef, &mut AnimationState,), (Without<ExcludedFromBaseAnimPickingSystem>,)>,
    sprite_config_query: Query<(&SpriteCfgAnimationsMap),(Or<(With<Disabled>, Without<Disabled>)>,)>,
) { 
        for (facing_direction, move_anim_active, grounding, held_sprites) in bases_query.iter() {
            let moving = move_anim_active.0;
            info !(target: "sprite_animation", "Updating animation state for held sprites: {:?}", held_sprites.sprite_ents());
            for held_sprite in held_sprites.sprite_ents() {
                let Ok((_str_id, sprite_config_ref, mut anim_state)) = sprite_query.get_mut(*held_sprite) 
                else { continue };

                let Ok(sprite_cfg_animations_map) = sprite_config_query.get(sprite_config_ref.0) 
                else { continue };
        }
    }
}

//#[bevy_simple_subsecond_system::hot]

#[allow(unused_parens, )]
pub fn animate_sprite(
    mut commands: Commands,
    
    mut sprites_query: Query<(Entity, &SpriteBaseHolderRef, &SpriteConfigRef, Option<&AnimationState>,
    ), (Or<(Changed<AnimationState>, Changed<FacingDirection>)>)>,
    
    spriteconfig: Query<(&SpriteCfgAnimationsMap, Has<Directionable>, Has<MovementBased>, Has<GroundingBased>, ), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,
    
    spriteholder: Query<(Option<&FacingDirection>, Option<&MoveAnimActive>, &Grounding, )>,
    
    aserver: Res<AssetServer>, mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (ent, spriteholder_ref, sprite_cfg_ref, state_id, ) in sprites_query.iter_mut() {
        
        let Ok((direction, moving, &grounding)) = spriteholder.get(spriteholder_ref.base)
        else {
            error!("Failed to get SpriteHolderRef base entity {:?}", spriteholder_ref.base);
            continue;
        };

        let Ok((sprite_cfg_animations_map, directionable, movement_based, grounding_based, )) = spriteconfig.get(sprite_cfg_ref.0)
        else {
            error!("Failed to get SpriteConfigRef entity {:?}", sprite_cfg_ref.0);
            continue;
        };
        
        let anim_type = AnimType {
            direction: if directionable { direction.copied().unwrap_or_default() } else { FacingDirection::default() },
            moving: if movement_based { moving.copied().unwrap_or_default() } else { MoveAnimActive::default() },
            grounding: if grounding_based { grounding } else { Grounding::default() },
            state_id: state_id.cloned(),
        };

        let Some(ent) = sprite_cfg_animations_map.0.get(&anim_type) else {
            warn!(target: "sprite_animation", "No animation found for AnimType {:?} in SpriteCfgAnimationsMap for entity {:?}", anim_type, ent);
            continue;
        };

    }
}


#[allow(unused_parens)]
pub fn update_animstate_for_clients(
    mut cmd: Commands,
    connected: Query<&Player, Without<OfSelf>>,
    started_query: Query<(Entity, &MoveAnimActive, Option<&StrId>), (Changed<MoveAnimActive>)>,
    controller: Query<&ControlledBy>,
){
    if connected.is_empty() { return; }

    for (being_ent, &moving, id) in started_query.iter() {
        let moving = moving.0;
        let event_data = MoveStateUpdated {being_ent, moving};
        if let Ok(controller) = controller.get(being_ent) {
            cmd.server_trigger(ToClients {
                mode: SendMode::BroadcastExcept(bevy_replicon::prelude::ClientId::Client(controller.client)),
                message: event_data,
            });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client);
        }
        else {
            cmd.server_trigger(ToClients { mode: SendMode::Broadcast, message: event_data, });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
}

// //#[cfg(not(feature = "headless_server"))]
// #[allow(unused_parens, )]
// pub fn client_receive_moving_anim(
//     trigger: On<MoveStateUpdated>, mut query: Query<&mut MoveAnimActive>,
//     client: Option<Res<RenetClient>>,
// ) {
//     if client.is_none() {return;}
    
//     let MoveStateUpdated { being_ent, moving } = trigger.event().clone();
//     info!(target: "sprite_animation", "Received moving {} for entity {:?}", moving, being_ent);

//     if let Ok(mut move_anim) = query.get_mut(being_ent) {
//         move_anim.0 = moving;
//     } else {
//         warn!("Received moving state for entity {:?} that does not exist in this client.", being_ent);
//     }
// }