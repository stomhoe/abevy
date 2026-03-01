use crate::tile::tile_components::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
use common::common_components::HashId;
use game_common::game_common_components::*;
use ::sprite_shared::*;
use ::tilemap_shared::*;

#[allow(unused_parens)]
pub fn flip_tile_based_on_initial_pos_hash(
    settings: Query<&GlobalGenSettings>,
    mut tile_query: Query<
        (&mut TileFlip, &InitialPos, &EntityZeroRef, Option<&DimensionRef>),
        (
            Changed<InitialPos>,
            common::AnyDisabling,
            Without<EntityZero>,
        ),
    >,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
    ezero_query: Query<
        (
            Has<FlipHorizontallyBasedOnHash>,
            Has<FlipVerticallyBasedOnHash>,
            Has<FlipDiagonallyBasedOnHash>,
        ),
        (),
    >,
) {
    if tile_query.is_empty() {
        return;
    }
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    tile_query.iter_mut().for_each(
        |(mut tile_flip, initial_pos, ezero_ref, dimension_ref)| {

            let Ok((do_flip_hori, do_flip_vert, do_flip_diag)) = ezero_query.get(ezero_ref.0) else {
                return;
            };

            let dimension_hash = dimension_ref
                .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
                .cloned()
                .unwrap_or_default();


            if do_flip_hori {
                let should_flip = initial_pos.0.hash_true_false(settings, dimension_hash, 0);
                tile_flip.x = should_flip;
            }

            if do_flip_vert {
                let should_flip = initial_pos.0.hash_true_false(settings, dimension_hash, 1);

                tile_flip.y = should_flip;
            }

            if do_flip_diag {
                let should_flip = initial_pos.0.hash_true_false(settings, dimension_hash, 2);
                tile_flip.d = should_flip;
            }
        },
    );
}

#[allow(unused_parens)]
pub fn rotate_tile_based_on_initial_pos_hash(
    mut cmd: Commands,
    settings: Query<&GlobalGenSettings>,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
    mut tile_query: Query<
        (
            Entity,
            Option<&mut CardinalDirection>,
            Option<&mut Transform>,
            &InitialPos,
            &EntityZeroRef,
            Option<&DimensionRef>,
        ),
        (
            Changed<InitialPos>,
            common::AnyDisabling,
            Without<EntityZero>,
        ),
    >,
    ezero_query: Query<(Has<RotateCardinallyBasedOnHash>, Has<TransformBasedCardRotation>,),
        (),
    >,
) {
    if tile_query.is_empty() {
        return;
    }
    let Ok(settings) = settings.single() else {
        error_once!("Failed to get global gen settings");
        return;
    };
    for (ent, direction, maybe_transform, initial_pos, ezero_ref, dimension_ref) in
        tile_query.iter_mut()
    {
        let Ok((do_rotate, do_transform_rotate)) = ezero_query.get(ezero_ref.0) else {
            continue;
        };
        if !do_rotate {
            continue;
        }
        let dimension_hash = dimension_ref
            .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
            .cloned()
            .unwrap_or_default();

        let hash_u8 = (initial_pos.0.hash_value(settings, dimension_hash, 3) % 4) as u8;
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
            Without<EntityZero>,
            common::AnyDisabling,
        ),
    >,
    mut sprites_query: Query<&mut Sprite, (common::AnyDisabling, )>,
) {
    for (tile_ent, tile_flip, held_sprites, children) in tile_query.iter() {
        if let Ok(mut my_sprite) = sprites_query.get_mut(tile_ent) {
            my_sprite.flip_x = tile_flip.x;
            my_sprite.flip_y = tile_flip.y;
        }
        if let Some(held_sprites) = held_sprites {
            held_sprites.entities().iter().for_each(|&sprite_entity| {
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
            });
        }
    }
}
