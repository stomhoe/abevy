

#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::game::{being::{being_components::{Altitude, Moving}, sprite::{animation_constants::*, animation_resources::*, sprite_components::*, sprite_constants::* }}, game_components::FacingDirection,};

pub fn prepend_body_to_string(
    prefix: &str, 
    body: &str,
) -> String {
 
    format!("{}{}", prefix, body)
}

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
                    None => Clip::from_frames(
                    spritesheet.row(seri.target as usize)
                    ),
                }
                } else {
                match seri.partial {
                    Some([start, end]) => Clip::from_frames(
                    spritesheet.column_partial(seri.target as usize, (start as usize)..=(end as usize))
                    ),
                    None => Clip::from_frames(
                    spritesheet.column(seri.target as usize)
                    ),
                }
                };
                let animation = Animation::from_clip(library.register_clip(clip));
                let animation_id = library.register_animation(animation);
                info!("Registered animation: {}", seri.id); 
                library.name_animation(animation_id, take(&mut seri.id)).unwrap();
            }
    }
}




#[allow(unused_parens)]
pub fn change_anim_state_string(
    mut sprite_query: Query<(
            &ChildOf, &mut AnimationState,
            Option<&WalkAnim>, Option<&FlyAnim>, Option<&SwimAnim>,
            Option<&AnimationIdPrefix>,
        ), (Without<ExcludedFromBaseAnimPickingSystem>)>,
    parents_query: Query<(Option<&Moving>, &Altitude),>,
) {
    for (child_of, mut anim_state, has_walk_anim, has_swim_anim, has_fly_anim, prefix) in sprite_query.iter_mut() {
        if let Ok((moving, curr_parent_altitude)) = parents_query.get(child_of.parent()) {
            match (moving, curr_parent_altitude, has_walk_anim, has_swim_anim, has_fly_anim) {
                (_any_move, _any_alti, None, None, None) => {
                    anim_state.set_idle();
                    let prefix_str = prefix.map_or("", |p| p.0.as_str());
                    info!(" prefix '{}', setting to idle.", prefix_str);    },
                (Some(_move), Altitude::OnGround, Some(_), _, _) => {
                    anim_state.set_walk();
                },
                (Some(_move), Altitude::Swimming, _, Some(_), _) => {
                    anim_state.set_swim();
                },
                (Some(_move), Altitude::Floating, _, _, Some(_)) => {
                    anim_state.set_fly();
                },
                (None, Altitude::OnGround, _, _, _) => {
                    anim_state.set_idle();
                },
                (None, Altitude::Swimming, _, Some(has_swim_anim), _) => {
                    if has_swim_anim.use_still {
                        anim_state.set_idle();
                    } else {
                        anim_state.set_swim();
                    }
                },
                (None, Altitude::Floating, _, _, Some(has_fly_anim)) => {
                    if has_fly_anim.use_still {
                        anim_state.set_idle();
                    } else {
                        anim_state.set_fly();
                    }
                },
                (Some(_move), Altitude::Floating, Some(_has_walk), _, None) => {
                    anim_state.set_idle();
                },
                (Some(_move), Altitude::Swimming, Some(_has_walk), None, _) => {
                    anim_state.set_walk();
                },
                (Some(_move), Altitude::OnGround, None, Some(_has_fly), None) => {
                    anim_state.set_fly();
                },
                (Some(_move), Altitude::OnGround, None, None, Some(_has_swim)) => {
                    anim_state.set_swim();
                },
                (Some(_move), Altitude::OnGround, None, Some(_has_fly), Some(_has_swim)) => {
                    anim_state.set_fly();
                },
                (Some(_), Altitude::Swimming, None, None, Some(_fly)) => {anim_state.set_idle();},
                (Some(_), Altitude::Floating, None, Some(_swim), None) => {anim_state.set_idle();},
                (None, _curr_alt, _any_walk, _any_swim, _any_fly) => {anim_state.set_idle();},
            }
        }
    }
      
}

pub fn animate_sprite(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&mut SpritesheetAnimation>,
        Option<&AnimationIdPrefix>,
        Option<&AnimationState>,
        Option<&Directionable>,
        &ChildOf,
    ), With<Sprite>>,
    parents: Query<(Option<&FacingDirection>, )>,
    library: Res<AnimationLibrary>,
) {
    for (ent, sheet_anim, prefix, anim_state, directionable, child_of) in query.iter_mut() {
        if let Ok(direction) = parents.get(child_of.parent()) {
            
            let prefix = prefix.as_ref().map_or("", |p| p.0.as_str());
            let anim_state = anim_state.as_ref().map_or("", |s| s.0.as_str());
            let direction_str = directionable
                .and_then(|_| direction.0.map(|dir| dir.as_suffix()))
                .unwrap_or("");
            let animation_name = format!("{}{}{}", prefix, anim_state, direction_str);
            info!("Entity {:?} animation name: {}", ent, animation_name);

            if let Some(animation_id) = library.animation_with_name(animation_name.clone()) {
                if let Some(mut sheet_anim) = sheet_anim {
                    if sheet_anim.animation_id != animation_id {
                        sheet_anim.switch(animation_id);
                    }
                } else{
                    let new_anim = SpritesheetAnimation::from_id(animation_id);
                    commands.entity(ent).insert(new_anim);
                }
            }
            else{
                commands.entity(ent).remove::<SpritesheetAnimation>();
                warn!("Animation with name '{}' not found in library.", animation_name);
            }
        }
    }
      
}


