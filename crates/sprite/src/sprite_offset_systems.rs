use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
use ::common::*;
use game_common::game_common_components::*;
use ::sprite_shared::*;
use ::tilemap_shared::tilemap_shared_samplers::*;

use crate::{sprite_systems::SpriteChangedScaleOrOffsetOrParent};
use ::tilemap_shared::directions::*;

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut reader: MessageReader<SpriteChangedScaleOrOffsetOrParent>,
    ______str_id_query: Query<&StrId>,
    mut sprite_query: Query<(
        &mut Transform,
        &BaseHolderRef,
        &ChildOf,
        Option<&TemplEntiRef>,
        Option<&AutoCorrectOffsetBasedOnParentSizeResults>,
        Has<DontNegativizeOffset>,
        Option<&Offset2D>,
    ), (Without<Templ>, )>,
    sprite_config_query: Query<(
        Option<&TagSet>,
        Option<&Offset2D>,
        Option<&OffsetSideways>,
        Option<&OffsetUpDown>, Option<&OffsetUp>, Option<&OffsetDown>,
    ),()>,
    offset_for_children_query: Query<&OffsetForChildren>,
    parent_sprite_query: Query<&TemplEntiRef>,
    base_query: Query<&CardinalDirection>,
    base_normal_dist_query: Query<(Option<&SpriteGlobalNormalDistResult>, Option<&SpriteHoriNormalDistResult>, Option<&SpriteVertNormalDistResult>)>,
    base_tile_flip_query: Query<&TileFlip>,
) {
    for (msg, _) in reader.par_read() {
        let sprite_ent = msg.0;
        let Ok((
            mut transform, baseholder, child_of, sprite_config_ref, auto_correct_offset_based_on_parent_size_results, dont_negativize_offset, own_offset,
        )) = sprite_query.get_mut(sprite_ent) else {
            continue;
        };



        let mut total_offset = Offset2D::default();

        if let Some(TemplEntiRef(my_templ_ent)) = sprite_config_ref.cloned() {
            let Ok((my_cats, templ_offset, offset_sideways, offset_updown, offset_up, offset_down)) = sprite_config_query.get(my_templ_ent)
            else {
                error_once!("Failed to get sprite config entity {:?}", my_templ_ent);
                transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                continue;
            };

            total_offset += own_offset.cloned().unwrap_or(templ_offset.cloned().unwrap_or_default());

            if let Ok(direction) = base_query.get(baseholder.base) {
                match direction {
                    CardinalDirection::West => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    CardinalDirection::East => {
                        total_offset += offset_sideways.cloned().unwrap_or_default();
                    },
                    CardinalDirection::North => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_up.cloned().unwrap_or_default();
                    },
                    CardinalDirection::South => {
                        total_offset += offset_updown.cloned().unwrap_or_default();
                        total_offset += offset_down.cloned().unwrap_or_default();
                    }
                }
                if let Some(my_cats) = my_cats {//DEJAR ASÍ, NO DES-ANIDAR
                    if let Ok(&TemplEntiRef(parent_sprite_templ_ent)) = (parent_sprite_query.get(child_of.parent())){
                        if let Ok(offset_for_children) = offset_for_children_query.get(parent_sprite_templ_ent) {
                            for (offset_cat, &(offset, _dir)) in offset_for_children.0.iter() {
                                if my_cats.0.contains(offset_cat) {
                                    total_offset += offset;
                                }
                            }
                        }
                    }
                }
            }
        } else{
            total_offset += own_offset.cloned().unwrap_or_default();
        }

        if let Some(_auto_correct) = auto_correct_offset_based_on_parent_size_results {
            let Ok((global_dist, hori_dist, vert_dist)) = base_normal_dist_query.get(baseholder.base) else {
                error_once!("Failed to get parent normal dist results for offset autocorrection on sprite entity {:?}", sprite_ent);
                if transform.translation.xy() != total_offset.0 {
                    transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                }
                continue;
            };

            let global_mult = global_dist.map(|value| value.0).unwrap_or(1.0);
            let hori_mult = hori_dist.map(|value| value.0).unwrap_or(1.0);
            let vert_mult = vert_dist.map(|value| value.0).unwrap_or(1.0);
            let offset_scale = Vec2::new(global_mult * hori_mult, global_mult * vert_mult);
            let correction = total_offset.0 * (offset_scale - Vec2::ONE) * 1.;
            total_offset += Offset2D::from(correction);
        }
        

        if !dont_negativize_offset {
            if let Ok(tile_flip) = base_tile_flip_query.get(baseholder.base) {
                if tile_flip.x {
                    total_offset.0.x = -total_offset.0.x.abs();
                }
                if tile_flip.y {
                    total_offset.0.y = -total_offset.0.y.abs();
                }
            }
        }
        if transform.translation.xy() != total_offset.0 {
            transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
        }
    }
}
