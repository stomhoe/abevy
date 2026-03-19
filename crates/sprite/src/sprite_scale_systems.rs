
#[allow(unused_imports)] use bevy::prelude::*;
use game_common::game_common_components::{EntityZeroRef, };
use game_common::game_common_samplers::*;
use ::sprite_shared::{sprite_scale_offset::*, };
use ::tilemap_shared::directions::*;

use crate::sprite_components::*;
use crate::sprite_systems::SpriteChanged;


#[allow(unused_parens)]
pub fn apply_scales(
    mut reader: MessageReader<SpriteChanged>,
    //puede ser un meshtexture2d en vez de un sprite(para aplicar shaders)
    mut sprite_que: Query<(&mut Transform, Option<&mut Sprite>, &BaseHolderRef, &EntityZeroRef,
        Option<&Scale2D>, Option<&ScaleLookUpDown>, Option<&ScaleSideways>,
    ),>,
    sprite_config_query: Query<(Option<&FlipHorizIfDir>, Option<&Scale2D>, Option<&ScaleLookUpDown>, Option<&ScaleSideways>,  ), ()>,
    baseholder_query: Query<(Option<&CardinalDirection>, Option<&SpriteGlobalNormalDistResult>, Option<&SpriteHoriNormalDistResult>, Option<&SpriteVertNormalDistResult>)>,
) {
    for (msg, _) in reader.par_read() {
        let Ok((
            mut transform, sprite, spriteholder, &EntityZeroRef(spritecfg_ent),
             scale, scale_look_up_down, scale_look_sideways,
        )) = sprite_que.get_mut(msg.0) else { continue };
        let mut total_scale = scale.copied().unwrap_or_default();

        let (ref_flip_horiz_if_dir, ref_scale, ref_scale_updown, ref_scale_sideways) =
            sprite_config_query.get(spritecfg_ent)
                .map(|(a, b, c, d)| (a, b, c, d))
                .unwrap_or((None, None, None, None));

        total_scale *= ref_scale.copied().unwrap_or_default();

        let Ok((base_direction, ref_sprite_global_normal_dist_result, ref_sprite_hori_normal_dist_result, ref_sprite_vert_normal_dist_result)) = baseholder_query.get(spriteholder.base) else { continue };
        let global_mult = ref_sprite_global_normal_dist_result.map(|v| v.0).unwrap_or(1.0);
        let hori_mult = ref_sprite_hori_normal_dist_result.map(|v| v.0).unwrap_or(1.0);
        let vert_mult = ref_sprite_vert_normal_dist_result.map(|v| v.0).unwrap_or(1.0);
        total_scale *= Scale2D::from((global_mult * hori_mult, global_mult * vert_mult));
        let Some(base_direction) = base_direction else { continue };

        match base_direction {
            CardinalDirection::West => {
                total_scale *= ref_scale_sideways.copied().unwrap_or_default() * scale_look_sideways.copied().unwrap_or_default();

                if let Some(&flip_horiz) = ref_flip_horiz_if_dir {
                    if let Some(mut sprite) = sprite {
                        sprite.flip_x = match flip_horiz {
                            FlipHorizIfDir::Left => true, _ => true,
                        };
                    }
                }
            },
            CardinalDirection::East => {
                total_scale *= ref_scale_sideways.copied().unwrap_or_default() * scale_look_sideways.copied().unwrap_or_default();

                if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                    if let Some(mut sprite) = sprite {
                        sprite.flip_x = match flip_horiz {
                            FlipHorizIfDir::Left => false, _ => true,
                        };
                    }
                }
            },
            CardinalDirection::North => {
                total_scale *= ref_scale_updown.copied().unwrap_or_default() * scale_look_up_down.copied().unwrap_or_default();
                if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                    if let Some(mut sprite) = sprite {
                        sprite.flip_x = match flip_horiz {
                            FlipHorizIfDir::Any => true, _ => false,
                        };
                    }
                }
            },
            CardinalDirection::South => {
                total_scale *= ref_scale_updown.copied().unwrap_or_default() * scale_look_up_down.copied().unwrap_or_default();
                if let Some(flip_horiz) = ref_flip_horiz_if_dir {
                    if let Some(mut sprite) = sprite {
                        sprite.flip_x = match flip_horiz {
                            FlipHorizIfDir::Any => true, _ => false,
                        };
                    }
                }
            },
        }
        let total_scale_vec2 = total_scale.as_vec2();
        if total_scale_vec2.x == 0.0 || total_scale_vec2.y == 0.0 {
            warn!("total_scale is zero for sprite entity");
        }
        if total_scale_vec2.x > 0.0 && total_scale_vec2.y > 0.0 && total_scale_vec2 != transform.scale.xy() {
            transform.scale.x = total_scale_vec2.x;
            transform.scale.y = total_scale_vec2.y;
        }
    }
}
