

use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use common::common_components::StrId;
use game_common::game_common_components::{BeingAltitude, Directionable, FacingDirection};
use sprite_shared::{animation_shared::*, sprite_shared::*};

use crate::animation_resources::*;


#[allow(unused_parens)]
pub fn init_animations(
    mut anim_handles: ResMut<AnimSerisHandles>,
    mut assets: ResMut<Assets<AnimationSeri>>,
    mut library: ResMut<AnimationLibrary>,
) {
    use std::mem::take;
    let handles_vec = take(&mut anim_handles.handles);
    for handle in handles_vec {
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
        info!(target: "sprite_animation", "Registered animation: {}", seri.id);
        library.name_animation(animation_id, seri.id).unwrap();
        
    }
}

//#[bevy_simple_subsecond_system::hot]
#[allow(unused_parens)]
pub fn update_animstate(
    parents_query: Query<(&MoveAnimActive, &BeingAltitude, &HeldSprites), (Or<(Changed<BeingAltitude>, Changed<MoveAnimActive>, Changed<HeldSprites>)>,)>,
    mut sprite_query: Query<(&StrId, &SpriteConfigRef, &mut AnimationState,), (Without<ExcludedFromBaseAnimPickingSystem>,)>,
    sprite_config_query: Query<(Option<&WalkAnim>, Option<&SwimAnim>, Option<&FlyAnim>, ),(Or<(With<Disabled>, Without<Disabled>)>,)>,
) { 
        for (&MoveAnimActive(moving), curr_parent_altitude, held_sprites) in parents_query.iter() {
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
    ), (With<Sprite>, Changed<AnimationState>)>,
    cfg_query: Query<(&AnimationIdPrefix, Has<Directionable>, Option<&FlipHorizIfDir>), (With<SpriteConfig>, Or<(With<Disabled>, Without<Disabled>)>,)>,
    spriteholder_direction: Query<(&FacingDirection, )>,
    library: Res<AnimationLibrary>,
) {
    for (ent, spriteholder_ref, sprite_cfg_ref, sheet_anim, moving_anim, ) in query.iter_mut() {
        
        let direction = spriteholder_direction.get(spriteholder_ref.base);

        let Ok((prefix, directionable, flip_horiz)) = cfg_query.get(sprite_cfg_ref.0) else {continue };

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
        } else {
            commands.entity(ent).remove::<SpritesheetAnimation>();
            error!(target: "animate_sprite", "Animation with name '{}' not found in library.", animation_name);
        }
    }
}


