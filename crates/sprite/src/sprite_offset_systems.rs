use bevy::prelude::*;
use common::{common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::{sprite_components::*, sprite_systems::SpriteChanged};
use ::tilemap_shared::directions::*;

#[allow(unused_parens, )]
pub fn apply_offsets(
    mut cmd: Commands,
    mut reader: MessageReader<SpriteChanged>,
    mut sprite_query: Query<(
        &mut Transform,
        &BaseHolderRef,
        &ChildOf,
        Option<&EntityZeroRef>,
        Option<&Offset2D>,
        Has<SpriteConfigNotFound>,
    ), (Without<EntityZero>, )>,
    sprite_config_query: Query<(
        Option<&TagSet>,
        Option<&Offset2D>,
        Option<&OffsetSideways>,
        Option<&OffsetUpDown>, Option<&OffsetUp>, Option<&OffsetDown>,
        Option<&OffsetForChildren>,
    ),()>,
    parent_sprite_query: Query<&EntityZeroRef>,
    base_query: Query<&CardinalDirection>,
) {
    for (msg, _) in reader.par_read() {
        let sprite_ent = msg.0;
        let Ok((
            mut transform, baseholder, child_of, sprite_config_ref,
            offset, has_sprite_config_not_found
        )) = sprite_query.get_mut(sprite_ent) else {
            continue;
        };
        let mut total_offset = Offset2D::default();

        if let Some(EntityZeroRef(sprite_config)) = sprite_config_ref.cloned() {
            let Ok((my_cats, offset, offset_sideways, offset_updown, offset_up, offset_down, _)) = sprite_config_query.get(sprite_config)
            else {
                if !has_sprite_config_not_found {
                    error!("Failed to get sprite config for entity {:?}", sprite_config);
                    cmd.entity(sprite_ent).try_insert(SpriteConfigNotFound);
                }
                transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
                continue;
            };
            if has_sprite_config_not_found {
                cmd.entity(sprite_ent).try_remove::<SpriteConfigNotFound>();
            }
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
                    let Ok((.., offset_for_children)) = sprite_config_query.get(ezero_ent) else {
                        continue;
                    };
                    let Some(offset_for_children) = offset_for_children else {
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
        transform.translation.x = total_offset.0.x; transform.translation.y = total_offset.0.y;
    }
}
