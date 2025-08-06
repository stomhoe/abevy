

use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_replicon_renet::renet::RenetServer;
use bevy_spritesheet_animation::prelude::*;

use crate::{common::common_components::StrId, game::{being::{being_components::ControlledBy, movement::movement_components::*, sprite::{animation_constants::*, animation_resources::*, sprite_components::*, }}, game_components::FacingDirection,}};


#[allow(unused_parens)]
pub fn init_animations(
    mut anim_handles: ResMut<AnimSerisHandles>,
    mut assets: ResMut<Assets<AnimationSeri>>,
    mut library: ResMut<AnimationLibrary>,
) {
    use std::mem::take;
    let handles_vec = take(&mut anim_handles.handles);
    for handle in handles_vec {
        if let Some(mut seri) = assets.remove(&handle) 
            {
                let spritesheet = Spritesheet::new(seri.sheet_rows_cols[1] as usize, seri.sheet_rows_cols[0] as usize);

                let clip: Clip = if seri.is_row {
                match seri.partial {
                    Some([start, end]) => Clip::from_frames(
                    spritesheet.row_partial(seri.target as usize, (start as usize)..=(end as usize))
                    ),
                    None => Clip::from_frames(spritesheet.row(seri.target as usize)),
                }
                } else {
                    match seri.partial {
                        Some([start, end]) => Clip::from_frames(
                        spritesheet.column_partial(seri.target as usize, (start as usize)..=(end as usize))
                        ),
                        None => Clip::from_frames(spritesheet.column(seri.target as usize)),
                    }
                };
                let mut animation = Animation::from_clip(library.register_clip(clip));
                animation.set_repetitions(AnimationRepeat::Loop);
                let animation_id = library.register_animation(animation);
                //info!(target: "sprite_animation", "Registered animation: {}", seri.id);
                library.name_animation(animation_id, take(&mut seri.id)).unwrap();
            }
    }
}

//#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens)]
pub fn change_anim_state_string(
    mut sprite_query: Query<(&StrId, &SpriteHolderRef, &SpriteConfigRef, &mut AnimationState,), 
    (Without<ExcludedFromBaseAnimPickingSystem>)>,
    sprite_config_query: Query<(Option<&WalkAnim>, Option<&SwimAnim>, Option<&FlyAnim>,),
    (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,

    parents_query: Query<(Has<VoluntarilyMoving>, &Altitude)>,
) {
    for (_str_id, sprite_holder, sprite_config_ref, mut anim_state) in sprite_query.iter_mut() {
        if let Ok((has_walk_anim, has_swim_anim, has_fly_anim)) = sprite_config_query.get(sprite_config_ref.0) {
            if let Ok((moving, curr_parent_altitude)) = parents_query.get(sprite_holder.base) {
                match (moving, curr_parent_altitude, has_walk_anim, has_swim_anim, has_fly_anim) {
                    (_any_move, _any_alti, None, None, None) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },
                    (true, Altitude::OnGround, Some(_), _, _) => {
                        anim_state.set_walk();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_walk for {:?}", _str_id);}
                    },
                    (true, Altitude::Swimming, _, Some(_), _) => {
                        anim_state.set_swim();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_swim for {:?}", _str_id);}
                    },
                    (true, Altitude::Floating, _, _, Some(_)) => {
                        anim_state.set_fly();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_fly for {:?}", _str_id);}
                    },
                    (false, Altitude::OnGround, _, _, _) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },
                    (false, Altitude::Swimming, _, Some(has_swim_anim), _) => {
                        if has_swim_anim.use_still {
                            anim_state.set_idle();
                            //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                        } else {
                            anim_state.set_swim();
                            //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_swim for {:?}", _str_id);}
                        }
                    },
                    (false, Altitude::Floating, _, _, Some(has_fly_anim)) => {
                        if has_fly_anim.use_still {
                            anim_state.set_idle();
                            //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                        } else {
                            anim_state.set_fly();
                            //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_fly for {:?}", _str_id);}
                        }
                    },
                    (true, Altitude::Floating, Some(_has_walk), _, None) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },
                    (true, Altitude::Swimming, Some(_has_walk), None, _) => {
                        anim_state.set_walk();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_walk for {:?}", _str_id);}
                    },
                    (true, Altitude::OnGround, None, Some(_has_fly), None) => {
                        anim_state.set_fly();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_fly for {:?}", _str_id);}
                    },
                    (true, Altitude::OnGround, None, None, Some(_has_swim)) => {
                        anim_state.set_swim();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_swim for {:?}", _str_id);}
                    },
                    (true, Altitude::OnGround, None, Some(_has_fly), Some(_has_swim)) => {
                        anim_state.set_fly();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_fly for {:?}", _str_id);}
                    },
                    (true, Altitude::Swimming, None, None, Some(_fly)) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },
                    (true, Altitude::Floating, None, Some(_swim), None) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },
                    (false, _curr_alt, _any_walk, _any_swim, _any_fly) => {
                        anim_state.set_idle();
                        //if _str_id.as_str() != "humanhe0" {info!("AnimState: set_idle for {:?}", _str_id);}
                    },    
                }
            }
        }
    }
      
}


#[allow(unused_parens)]
pub fn on_anim_state_change(//hacer cada 50ms, no poner changed porq se puede perder el paquete y puede quedarse corriendo
    mut cmd: Commands,
    query: Query<(Entity, &SpriteHolderRef, &AnimationState), (Changed<AnimationState>,)>,
    controlled_by: Query<&ControlledBy>,
)
{
    for (sprite_ent, sprite_holder, anim_state) in query.iter() {
        let event_data = AnimStateUpdated {
            sprite_ent, anim_state: anim_state.clone(),
        };
        if let Ok(controller) = controlled_by.get(sprite_holder.base) {
            cmd.server_trigger(ToClients {
                mode: SendMode::BroadcastExcept(controller.player),
                event: event_data.clone(),
            });
        }
        else {
            cmd.server_trigger(ToClients {
                mode: SendMode::Broadcast,
                event: event_data.clone(),
            });
        }
    }
}

pub fn on_receive_anim_state_from_server(
    trigger: Trigger<AnimStateUpdated>,
    mut query: Query<&mut AnimationState, >,
    server: Option<Res<RenetServer>>,
) {
    if server.is_some() { return; }

    
    let AnimStateUpdated { sprite_ent, anim_state } = trigger.event().clone();
    
    if let Ok(mut state) = query.get_mut(sprite_ent) {
        *state = anim_state;
        //info!("Received animation state for entity {:?}: {:?}", sprite_ent, state);
    } else {
        warn!("Received animation state for entity {:?} that does not exist or is not a sprite.", sprite_ent);
    }
}


pub fn animate_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &SpriteHolderRef, &SpriteConfigRef,
        Option<&mut SpritesheetAnimation>,
        Option<&AnimationState>,
    ), With<Sprite>>,
    cfg_query: Query<(&AnimationIdPrefix, Has<Directionable>), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,
    parents: Query<(Option<&FacingDirection>, )>,
    library: Res<AnimationLibrary>,
) {
    for (ent, spriteholder_ref, sprite_cfg_ref, sheet_anim, anim_state, ) in query.iter_mut() {
        if let Ok(direction) = parents.get(spriteholder_ref.base) {
            
            if let Ok((prefix, directionable)) = cfg_query.get(sprite_cfg_ref.0) {
                let prefix = prefix.0.as_str();
                let direction_str = if directionable {
                    direction.0.map(|dir| dir.as_suffix()).unwrap_or("")
                } else {
                    ""
                };
                let animation_name = format!("{}{}{}", prefix, anim_state.map(|f| f.0.as_str()).unwrap_or(""), direction_str);
                //info!(target: "sprite_animation", "Entity {:?} animation name: '{}'", ent, animation_name);

                if let Some(animation_id) = library.animation_with_name(animation_name.clone()) {
                    if let Some(mut sheet_anim) = sheet_anim {
                        if sheet_anim.animation_id != animation_id {
                            sheet_anim.switch(animation_id);
                            //info!(target: "sprite_animation", "Switched animation for entity {:?} to '{}'", ent, animation_name);
                        } 
                        sheet_anim.speed_factor = 1.0;//AJUSTAR EN OTRO SISTEMA DISTINTO
                    } else {
                        let new_anim = SpritesheetAnimation::from_id(animation_id);
                        commands.entity(ent).insert(new_anim);
                        //info!(target: "sprite_animation", "Inserted new animation for entity {:?} with name '{}'", ent, animation_name);
                    }
                } else {
                    commands.entity(ent).remove::<SpritesheetAnimation>();
                    //warn!(target: "sprite_animation", "Animation with name '{}' not found in library.", animation_name);
                }
            }    }
    }
      
}


