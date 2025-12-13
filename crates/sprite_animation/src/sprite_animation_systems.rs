

use bevy_replicon::prelude::*;
use being_shared::{Grounding, ControlledBy};
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::*;
use common::{common_components::{ImageHolder, StrId}, common_states::GameSetupType};
use game_common::game_common_components::{Directionable, FacingDirection};
use player::player_components::*;
use sprite::sprite_components::*;
use sprite_animation_shared::{AnimationHandle, AnimationSheet, AnimationState, MoveAnimActive};

use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*};



#[allow(unused_parens, )]
pub fn animate_sprite(
    mut cmd: Commands,
    base: Query<(&HeldSprites, Option<&FacingDirection>, Option<&MoveAnimActive>, &Grounding, ), (
        Or<(
            Changed<HeldSprites>, 
            Changed<FacingDirection>, 
            Changed<MoveAnimActive>, 
            Changed<Grounding>,
        )>,
    )>,

    mut sprites_query: Query<(Entity, Option<&Sprite>, Option<&SpritesheetAnimation>, &SpriteConfigRef, Option<&AnimationState>,/*poner Option para la play speed, aca no en animation*/
), >,
    
    spriteconfig: Query<(&SpriteCfgAnimationsMap, Has<Directionable>, Has<MovementBased>, Has<GroundingBased>, ), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,
    
    query: Query<(&StrId, &AnimationHandle, &AnimationSheet, ),()>,
    
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    images: Res<Assets<Image>>, 
) {

    for (held_sprites, direction, moving, grounding) in base.iter() {
        for held_sprite in held_sprites.sprite_ents() {
            let Ok((ent, prev_sprite, prev_animation, sprite_cfg_ref, state_id, )) = sprites_query.get_mut(*held_sprite) 
            else { error!(target: "sprite_animation", "Failed to get sprite entity {:?}", held_sprite); continue };

            let Ok((sprite_cfg_animations_map, directionable, movement_based, grounding_based, )) = spriteconfig.get(sprite_cfg_ref.0)
            else { error!(target: "sprite_animation", "Failed to get SpriteConfigRef entity {:?}", sprite_cfg_ref.0); continue };

            let anim_type = AnimType {
                direction: if directionable { direction.copied().unwrap_or_default() } else { FacingDirection::default() },
                moving: if movement_based { moving.copied().unwrap_or_default() } else { MoveAnimActive::default() },
                grounding: if grounding_based { *grounding } else { Grounding::default() },
                state_id: state_id.cloned(),
            };
            debug!(target: "sprite_animation", "Determined AnimType {:?} for sprite entity {:?}", anim_type, ent);

            let Some(anim_ent) = sprite_cfg_animations_map.0.get(&anim_type) else {
                warn!(target: "sprite_animation", "No animation found for AnimType {:?} in SpriteCfgAnimationsMap for entity {:?}", anim_type, ent);
                continue;
            };

            let Ok((_, anim_handle, anim_sheet, )) = query.get(*anim_ent) else {
                error!(target: "sprite_animation", "Failed to get animation data for animation entity {:?}", anim_ent);
                continue;
            };

            let Some(sprite) = anim_sheet.0.with_loaded_image(&images) else {
                error!(target: "sprite_animation", "Failed to create sprite for animation entity {:?} because image is not loaded yet.", anim_ent);
                continue;
            };
            let sprite = sprite.sprite(&mut atlas_layouts);

            // if let Some(prev_sprite) = prev_sprite {
            //     if prev_sprite.image != sprite.image {
            //         cmd.entity(ent).try_insert((sprite,));
            //     }
                
            // } else {
            // }
            
            if let Some(prev_animation) = prev_animation {
                if prev_animation.animation != anim_handle.0 {
                    cmd.entity(ent).try_insert((sprite, SpritesheetAnimation::new(anim_handle.0.clone())));
                }
            } else {
                cmd.entity(ent).try_insert((sprite, SpritesheetAnimation::new(anim_handle.0.clone())));
            }
        }
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