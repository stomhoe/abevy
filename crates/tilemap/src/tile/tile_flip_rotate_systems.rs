use crate::tile::{tile_components::*, tile_messages::*, tile_resources::*};
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
use game_common::game_common_components::*;
use ::sprite_shared::*;
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn flip_tile_based_on_initial_pos_hash(
    settings: Query<&GlobalGenSettings>,
    mut tile_query: Query<
        (&mut TileFlip, &InitialPos, &TileRef),
        (Changed<InitialPos>, Without<Templ>, common::AnyDisabling,),
    >,
    templ_query: Query<(
            Has<FlipHorizontallyBasedOnHash>,
            Has<FlipVerticallyBasedOnHash>,
            Has<FlipDiagonallyBasedOnHash>,
        ),
        (),
    >,
    tile_map: Res<TileEntityMap>,
) {
    if tile_query.is_empty() {
        return;
    }
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    tile_query.iter_mut().for_each(
        |(mut tile_flip, initial_pos, templ_ref)| {
            let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
                return;
            };
            let Ok((do_flip_hori, do_flip_vert, do_flip_diag)) = templ_query.get(templ_ent) else {
                return;
            };

            let dimension_hash = initial_pos.dim.0;

            if do_flip_hori {
                tile_flip.x = initial_pos.pos.hash_true_false(settings, dimension_hash, 0);
            }
            if do_flip_vert {
                tile_flip.y = initial_pos.pos.hash_true_false(settings, dimension_hash, 1);
            }
            if do_flip_diag {
                tile_flip.d = initial_pos.pos.hash_true_false(settings, dimension_hash, 2);
            }
        },
    );
}

#[allow(unused_parens)]
pub fn rotate_tile_based_on_initial_pos_hash(
    mut cmd: Commands,
    settings: Query<&GlobalGenSettings>,
    mut tile_query: Query<
        (
            Entity,
            Option<&mut CardinalDirection>,
            Option<&mut Transform>,
            &InitialPos,
            &TileRef,
        ),
        (
            Changed<InitialPos>,
            common::AnyDisabling,
            Without<Templ>,
        ),
    >,
    templ_query: Query<(Has<RotateTransform>), (With<ChangeFacingDirectionBasedOnHash>,),
    >,
    tile_map: Res<TileEntityMap>,
) {
    if tile_query.is_empty() {
        return;
    }
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    for (ent, direction, maybe_transform, initial_pos, templ_ref) in
        tile_query.iter_mut()
    {
        let Ok(templ_ent) = tile_map.0.get_cloned(templ_ref.0) else {
            continue;
        };
        let Ok((do_transform_rotate)) = templ_query.get(templ_ent) else {
            continue;
        };
        let dimension_hash = initial_pos.dim.0;

        let hash_u8 = (initial_pos.pos.hash_value(settings, dimension_hash, 3) % 4) as u8;
        let new_direction = CardinalDirection::from(hash_u8);

        if let Some(mut direction) = direction {
            *direction = new_direction;
        } else {
            cmd.entity(ent).insert(new_direction);
        }

        if do_transform_rotate {
            if let Some(mut transform) = maybe_transform {
                let angle = match new_direction {
                    CardinalDirection::South => 0.0,
                    CardinalDirection::West => std::f32::consts::FRAC_PI_2,
                    CardinalDirection::North => std::f32::consts::PI,
                    CardinalDirection::East => -std::f32::consts::FRAC_PI_2,
                };
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

#[allow(unused_parens)]
pub fn sync_sprite_flips_with_tileflip(
    tile_query: Query<
        (Entity, &TileFlip, Option<&HeldSprites>, Option<&Children>),
        (
            Or<(Changed<TileFlip>, Changed<HeldSprites>, Changed<Children>)>,
            With<Tile>,
            Without<Templ>,
            common::AnyDisabling,
        ),
    >,
    mut sprites_query: Query<&mut Sprite, (common::AnyDisabling, )>,
    mut occluder_query: Query<(), (With<TileChildSpriteOccluder>, common::AnyDisabling, )>,
    mut flipped_transform_query: Query<&mut FlippedTransform, ()>,
    mut cmd: Commands,
) {
    for (tile_ent, tile_flip, held_sprites, children) in tile_query.iter() {
        if let Ok(mut my_sprite) = sprites_query.get_mut(tile_ent) {
            my_sprite.flip_x = tile_flip.x;
            my_sprite.flip_y = tile_flip.y;
        }
        if let Some(held_sprites) = held_sprites {
            held_sprites.iter().for_each(|sprite_entity| {
                if let Ok(mut sprite) = sprites_query.get_mut(sprite_entity) {
                    sprite.flip_x = tile_flip.x;
                    sprite.flip_y = tile_flip.y;
                }
            });
        }
        if let Some(children) = children {
            children.iter().for_each(|child| {
                if let Ok(mut sprite) = sprites_query.get_mut(child) {
                    sprite.flip_x = tile_flip.x;
                    sprite.flip_y = tile_flip.y;
                }
                if occluder_query.get_mut(child).is_ok() {
                    let flipped_transform = FlippedTransform { x: tile_flip.x, y: tile_flip.y };
                    if let Ok(mut flip_transform) = flipped_transform_query.get_mut(child) {
                        *flip_transform = flipped_transform;
                    } else {
                        cmd.entity(child).try_insert(flipped_transform);
                    }
                }
            });
        }
    }
}

#[allow(unused_parens, )]
pub fn track_non_default_tile_cardinal_direction_changes(
    query: Query<
        (&CardinalDirection, &GlobalTilePos, &TileRef),
        (With<Tile>, Without<Templ>, Changed<CardinalDirection>, common::AnyDisabling, ),
    >,
    mut card_at_gpos: ResMut<CardinalDirAtGpos>,
) {
    for (&card_dir, &gpos, templ) in query.iter() {
        if card_dir == CardinalDirection::default() {
            continue;
        }
        card_at_gpos.0.insert((templ.0, gpos), card_dir);
    }
}

#[allow(unused_parens, )]
pub fn sync_cardinal_dir_at_gpos_on_gpos_change(
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    query: Query<
        (&CardinalDirection, &GlobalTilePos, &TileRef),
        (With<Tile>, Without<Templ>, common::AnyDisabling, ),
    >,
    mut card_at_gpos: ResMut<CardinalDirAtGpos>,
) {
    for changed in changed_pos.read() {
        let Some(old) = changed.old else {
            continue;
        };
        let Ok((&card_dir, &gpos, templ)) = query.get(changed.entity) else {
            continue;
        };
        if card_dir == CardinalDirection::default() {
            card_at_gpos.0.remove(&(templ.0, old.gpos));
        } else {
            card_at_gpos.0.remove(&(templ.0, old.gpos));
            card_at_gpos.0.insert((templ.0, gpos), card_dir);
        }
    }
}
