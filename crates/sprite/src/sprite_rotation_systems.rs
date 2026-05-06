use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
use ::sprite_shared::*;
use game_common::game_common_components::TemplEntiRef;
use ::tilemap_shared::directions::*;

use crate::sprite_systems::SpriteChangedScaleOrOffsetOrParent;

#[allow(unused_parens)]
pub fn apply_rotations(
    mut reader: MessageReader<SpriteChangedScaleOrOffsetOrParent>,
    mut sprite_query: Query<(
        &mut Transform,
        &TemplEntiRef,
        Has<CardinalDirectionAffectsRotation>,
        Has<NegativizeRotationOnTileFlip>,
    ),>,
    rotation_query: Query<&Rotation>,
    baseholder_query: Query<&BaseHolderRef>,
    direction_query: Query<&CardinalDirection>,
    tileflip_query: Query<&TileFlip>,
) {
    for (msg, _) in reader.par_read() {
        let Ok((mut transform, &TemplEntiRef(spritecfg_ent), card_dir_affects, negativize_on_tile_flip)) = sprite_query.get_mut(msg.0) else {
            continue;
        };

        let Some(mut total_rotation) = sprite_rotation_for_entity(msg.0, spritecfg_ent, &rotation_query) else {
            continue;
        };

        let Ok(sprite_baseholder) = baseholder_query.get(msg.0) else {
            continue;
        };

        if card_dir_affects && let Ok(base_direction) = direction_query.get(sprite_baseholder.base) {
            total_rotation += Rotation::from(match base_direction {
                CardinalDirection::West => std::f32::consts::FRAC_PI_2,
                CardinalDirection::North => std::f32::consts::PI,
                CardinalDirection::East => -std::f32::consts::FRAC_PI_2,
                CardinalDirection::South => 0.0,
            });
        };

        if negativize_on_tile_flip
            && let Ok(tile_flip) = tileflip_query.get(sprite_baseholder.base)
            && tile_flip.x
        {
            total_rotation = Rotation::from(-total_rotation.as_f32());
        }


        let total_rotation = Quat::from_rotation_z(total_rotation.as_radians());
        if transform.rotation != total_rotation {
            transform.rotation = total_rotation;
        }
    }
}

fn sprite_rotation_for_entity(
    sprite_ent: Entity,
    spritecfg_ent: Entity,
    rotation_query: &Query<&Rotation>,
) -> Option<Rotation> {
    rotation_query
        .get(sprite_ent)
        .ok()
        .copied()
        .or_else(|| rotation_query.get(spritecfg_ent).ok().copied())
}