use bevy::prelude::*;
use common::{common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::{sprite_components::*, sprite_systems::SpriteChanged};
use ::tilemap_shared::directions::*;

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut reader: MessageReader<SpriteChanged>,
    mut sprite_query: Query<(
        &mut Transform,
        &BaseHolderRef,
        &ChildOf,
        Option<&EntityZeroRef>,
        Option<&Offset2D>,
    ), (Without<EntityZero>, )>,
    sprite_config_query: Query<(
        Option<&TagSet>,
        Option<&Offset2D>,
        Option<&OffsetSideways>,
        Option<&OffsetUpDown>, Option<&OffsetUp>, Option<&OffsetDown>,
    ),()>,
    offset_for_children_query: Query<&OffsetForChildren>,
    parent_sprite_query: Query<&EntityZeroRef>,
    base_query: Query<&CardinalDirection>,
) {
    for (msg, _) in reader.par_read() {
        let sprite_ent = msg.0;
        let Ok((
            mut transform, baseholder, child_of, sprite_config_ref, offset,
        )) = sprite_query.get_mut(sprite_ent) else {
            continue;
        };
        let mut total_offset = Offset2D::default();

        if let Some(EntityZeroRef(sprite_config)) = sprite_config_ref.cloned() {
            let Ok((my_cats, offset, offset_sideways, offset_updown, offset_up, offset_down)) = sprite_config_query.get(sprite_config)
            else {
                error_once!("Failed to get sprite config entity {:?}", sprite_config);
                transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                continue;
            };

            total_offset += offset.cloned().unwrap_or_default();

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
                if let Some(my_cats) = my_cats {
                    let Ok(&EntityZeroRef(ezero_ent)) = parent_sprite_query.get(child_of.parent()) else {
                        continue;
                    };
                    let Ok(offset_for_children) = offset_for_children_query.get(ezero_ent) else {
                        continue;
                    };
                    for (offset_cat, &(offset, _dir)) in offset_for_children.0.iter() {
                        if my_cats.0.contains(offset_cat) {
                            total_offset += offset;
                        }
                    }
                }
            }
        } else{
            total_offset += offset.cloned().unwrap_or_default();
        }
        if transform.translation.xy() != total_offset.0 {
            transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
        }
    }
}
