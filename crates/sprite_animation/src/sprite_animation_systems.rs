

use bevy_replicon::prelude::*;
use being_shared::{BeingAltitude, ControlledBy};
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_replicon_renet::renet::RenetClient;
use bevy_spritesheet_animation::prelude::*;
use common::{common_components::StrId, common_states::GameSetupType};
use game_common::game_common_components::{Directionable, FacingDirection};
use player::player_components::*;
use sprite::sprite_components::*;

use crate::{sprite_animation_components::*, sprite_animation_events::MoveStateUpdated, sprite_animation_resources::*};


#[allow(unused_parens)]
pub fn init_animations(
    mut cmd: Commands,
    mut anim_handles: ResMut<AnimSerisHandles>,
    mut assets: ResMut<Assets<AnimationSeri>>,
    mut library: ResMut<AnimationLibrary>,
    //usar state
) {
    use std::mem::take;
    // if ! library.is_empty(){
    //     cmd.remove_resource::<AnimationLibrary>();
    // }



    for handle in take(&mut anim_handles.handles) {
        let Some(seri) = assets.remove(&handle) else { continue };
        
        let sheet = Spritesheet::new(seri.sheet_rows_cols[1], seri.sheet_rows_cols[0]);

        let clip: Clip = if seri.is_row {
            match seri.partial {
                Some([start, end]) => Clip::from_frames(
                    sheet.row_partial(seri.target, start..=end)
                ),
                None => Clip::from_frames(sheet.row(seri.target)),
            }
        } else {
            match seri.partial {
                Some([start, end]) => Clip::from_frames(
                    sheet.column_partial(seri.target, start..=end)
                ),
                None => Clip::from_frames(sheet.column(seri.target)),
            }
        };      
        let mut animation = Animation::from_clip(library.register_clip(clip));
        animation.set_repetitions(AnimationRepeat::Loop);
        let animation_id = library.register_animation(animation);

        if let Ok(()) = library.name_animation(animation_id, seri.id.clone()) {
            info!(target: "sprite_animation", "Registered animation: {}", seri.id);
        }else {
            error!(target: "sprite_animation", "Animation with same name already present: {}", seri.id);
        }
    }
}

//#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens)]
pub fn update_animstate(
    parents_query: Query<(&MoveAnimActive, &BeingAltitude, &HeldSprites), (Or<(Changed<BeingAltitude>, Changed<MoveAnimActive>, Changed<HeldSprites>)>,)>,
    mut sprite_query: Query<(&StrId, &SpriteConfigRef, &mut AnimationState,), (Without<ExcludedFromBaseAnimPickingSystem>,)>,
    sprite_config_query: Query<(Option<&WalkAnim>, Option<&SwimAnim>, Option<&FlyAnim>, ),(Or<(With<Disabled>, Without<Disabled>)>,)>,
) { 
        for (moving, curr_parent_altitude, held_sprites) in parents_query.iter() {
            let moving = moving.0;
            info !(target: "sprite_animation", "Updating animation state for held sprites: {:?}", held_sprites.sprite_ents());
            for held_sprite in held_sprites.sprite_ents() {
                let Ok((_str_id, sprite_config_ref, mut anim_state)) = sprite_query.get_mut(*held_sprite) 
                else { continue };

                let Ok((has_walk_anim, has_swim_anim, has_fly_anim)) = sprite_config_query.get(sprite_config_ref.0) 
                else { continue };

                match (moving, curr_parent_altitude, has_walk_anim, has_swim_anim, has_fly_anim) {
                    (_any_move, _any_alti, None, None, None) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },
                    (true, BeingAltitude::OnGround, Some(_), _, _) => {
                        anim_state.set_walk();
                        trace!(target: "sprite_animation", "AnimState: set_walk for {:?}", _str_id);
                    },
                    (true, BeingAltitude::Swimming, _, Some(_), _) => {
                        anim_state.set_swim();
                        trace!(target: "sprite_animation", "AnimState: set_swim for {:?}", _str_id);
                    },
                    (true, BeingAltitude::Floating, _, _, Some(_)) => {
                        anim_state.set_fly();
                        trace!(target: "sprite_animation", "AnimState: set_fly for {:?}", _str_id);
                    },
                    (false, BeingAltitude::OnGround, _, _, _) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },
                    (false, BeingAltitude::Swimming, _, Some(has_swim_anim), _) => {
                        if has_swim_anim.use_still {
                            anim_state.set_idle();
                            trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                        } else {
                            anim_state.set_swim();
                            trace!(target: "sprite_animation", "AnimState: set_swim for {:?}", _str_id);
                        }
                    },
                    (false, BeingAltitude::Floating, _, _, Some(has_fly_anim)) => {
                        if has_fly_anim.use_still {
                            anim_state.set_idle();
                            trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                        } else {
                            anim_state.set_fly();
                            trace!(target: "sprite_animation", "AnimState: set_fly for {:?}", _str_id);
                        }
                    },
                    (true, BeingAltitude::Floating, Some(_has_walk), _, None) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },
                    (true, BeingAltitude::Swimming, Some(_has_walk), None, _) => {
                        anim_state.set_walk();
                        trace!(target: "sprite_animation", "AnimState: set_walk for {:?}", _str_id);
                    },
                    (true, BeingAltitude::OnGround, None, Some(_has_fly), None) => {
                        anim_state.set_fly();
                        trace!(target: "sprite_animation", "AnimState: set_fly for {:?}", _str_id);
                    },
                    (true, BeingAltitude::OnGround, None, None, Some(_has_swim)) => {
                        anim_state.set_swim();
                        trace!(target: "sprite_animation", "AnimState: set_swim for {:?}", _str_id);
                    },
                    (true, BeingAltitude::OnGround, None, Some(_has_fly), Some(_has_swim)) => {
                        anim_state.set_fly();
                        trace!(target: "sprite_animation", "AnimState: set_fly for {:?}", _str_id);
                    },
                    (true, BeingAltitude::Swimming, None, None, Some(_fly)) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },
                    (true, BeingAltitude::Floating, None, Some(_swim), None) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },
                    (false, _curr_alt, _any_walk, _any_swim, _any_fly) => {
                        anim_state.set_idle();
                        trace!(target: "sprite_animation", "AnimState: set_idle for {:?}", _str_id);
                    },    
                }

                
            
        }
    }
}

//#[bevy_simple_subsecond_system::hot]



pub fn animate_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &SpriteHolderRef, &SpriteConfigRef,
        Option<&mut SpritesheetAnimation>, Option<&AnimationState>,
    ), (With<Sprite>, )>,
    cfg_query: Query<(&AnimationIdPrefix, Has<Directionable>, Option<&FlipHorizIfDir>), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,
    spriteholder_direction: Query<(&FacingDirection, )>,
    library: Option<Res<AnimationLibrary>>,
) {
    let Some(library) = library else {
        error!(target: "animate_sprite", "AnimationLibrary not found, skipping animation update.");
        return;
    };
    for (ent, spriteholder_ref, sprite_cfg_ref, sheet_anim, moving_anim, ) in query.iter_mut() {
        
        let direction = spriteholder_direction.get(spriteholder_ref.base);

        let Ok((prefix, directionable, flip_horiz)) = cfg_query.get(sprite_cfg_ref.0) else {
            trace!(target: "animate_sprite", "Failed to get config for SpriteConfigRef {:?}", sprite_cfg_ref.0);
            continue;
        };

        let prefix = prefix.0.as_str();
        let direction_str = if directionable {
            direction
                .ok()
                .map(|(facing_direction,)| facing_direction.as_ref())
                .unwrap_or("")
        } 
        else {""};
        
        let animation_name = format!("{}_{}_{}", prefix, moving_anim.map(|f| f.0.as_str()).unwrap_or(""), direction_str);
        trace!(target: "animate_sprite", "Entity {:?} concatted animation name: '{}'", ent, animation_name);

        if let Some(animation_id) = library.animation_with_name(animation_name.clone()) {
            if let Some(mut sheet_anim) = sheet_anim {
                if sheet_anim.animation_id != animation_id {
                    sheet_anim.switch(animation_id);
                    trace!(target: "animate_sprite", "Switched animation for entity {:?} to '{}'", ent, animation_name);
                } 
                sheet_anim.speed_factor = 1.0;//AJUSTAR EN OTRO SISTEMA DISTINTO
            } else {
                let new_anim = SpritesheetAnimation::from_id(animation_id);
                commands.entity(ent).insert(new_anim);
                trace!(target: "animate_sprite", "Inserted new animation for entity {:?} with name '{}'", ent, animation_name);
            }
        } else{ 
            error!(target: "animate_sprite", "Animation with name '{}' not found in library.", animation_name);
            if let Some(mut sheet_anim) = sheet_anim {
                sheet_anim.playing = false;
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
                mode: SendMode::BroadcastExcept(controller.client),
                event: event_data,
            });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} {} to all clients except {:?}", moving, being_ent, id.cloned().unwrap_or_default(), controller.client);
        }
        else {
            cmd.server_trigger(ToClients { mode: SendMode::Broadcast, event: event_data, });
            info!(target: "sprite_animation", "Sending moving {} for entity {:?} to all clients", moving, being_ent);
        }
    }
}

//#[cfg(not(feature = "headless_server"))]
#[allow(unused_parens, )]
pub fn client_receive_moving_anim(
    trigger: Trigger<MoveStateUpdated>, mut query: Query<&mut MoveAnimActive>,
    client: Option<Res<RenetClient>>,
) {
    if client.is_none() {return;}
    
    let MoveStateUpdated { being_ent, moving } = trigger.event().clone();
    info!(target: "sprite_animation", "Received moving {} for entity {:?}", moving, being_ent);

    if let Ok(mut move_anim) = query.get_mut(being_ent) {
        move_anim.0 = moving;
    } else {
        warn!("Received moving state for entity {:?} that does not exist in this client.", being_ent);
    }
}