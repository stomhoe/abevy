use std::collections::HashSet;

#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{DrawTilemap, anchor::TilemapAnchor};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use game_common::game_common_components::{EntityZero, EntityZeroRef, CardinalDirection};
use ::sprite_shared::{sprite_scale_offset::*, *};

use crate::sprite_components::*;


#[derive(Message, Debug, Clone, Hash, PartialEq, Eq)]
pub struct SpriteChanged(pub Entity);

#[allow(unused_parens)]
pub fn sprite_change_detection(
    sprite_query: Query<(Entity), (Or<(Changed<Scale2D>, Changed<ScaleLookUpDown>, Changed<ScaleSideways>, Changed<EntityZeroRef> , Changed<Offset2D>, Added<Sprite>)>)>,
    baseholder_query: Query<(&HeldSprites), (Or<(Changed<CardinalDirection>, Changed<HeldSprites>, Changed<HeldSprites>)>)>,
    mut writer: MessageWriter<SpriteChanged>,
    mut changed: Local<HashSet<SpriteChanged>>,
)
{
    for sprite_ent in sprite_query.iter() {
        changed.insert(SpriteChanged(sprite_ent));
    }
    for held_sprites in baseholder_query.iter() {
        for &sprite_ent in held_sprites.entities() {
            changed.insert(SpriteChanged(sprite_ent));
        }
    }
    writer.write_batch(changed.drain());
}


#[allow(unused_parens)]
pub fn disable_children_sprites_of_disabled(mut cmd: Commands,
    ezero_bases: Query<(&HeldSprites),(With<EntityZero>, Added<Disabled>)>,
    non_ezero_bases: Query<(&HeldSprites),(Without<EntityZero>,)>,
    mut removed: RemovedComponents<Disabled>,
) {
    let mut disableds = Vec::new();
    for (held_sprites) in ezero_bases.iter() {
        for &sprite_ent in held_sprites.entities() {
            disableds.push((sprite_ent, Disabled));
            //trace!(target: "sprite_systems", "Disabled sprite entity {:?} as its base entity was disabled", sprite_ent);
        }
    }
    for ent in removed.read() {
        if let Ok((held_sprites)) = non_ezero_bases.get(ent) {
            for &sprite_ent in held_sprites.entities() {
                cmd.entity(sprite_ent).try_remove::<Disabled>();
                //trace!(target: "sprite_systems","Re-enabled sprite entity {:?} as its base entity {:?} was re-enabled", sprite_ent, ent);
            }
        }
    }
    cmd.try_insert_batch(disableds);
}



#[allow(unused_parens, )]
pub fn z_sort_system(

    mut query: Query<(Entity, &mut Transform, &GlobalTransform, Option<&YSortOrigin>,
        AnyOf<(&AcZ, &EntityZeroRef)>, Has<TilemapAnchor>, &ChildOf, ),
        (Or<(Changed<EntityZeroRef>, Changed<GlobalTransform>, Changed<YSortOrigin>, Changed<AcZ>, Changed<ChildOf>,)>,
        Or<(With<Sprite>, With<TilemapAnchor>, )>)>,

    parent_sprite_query: Query<&Sprite, (common::AnyDisabling,)>,

    ezero_query: Query<(&AcZ, Option<&YSortOrigin>), ()>,

    mut mw_draw_tmap: MessageWriter<DrawTilemap>,

) {//TODO MEJORAR
    let mut to_draw = Vec::new();

    for (ent, mut transform, global_transform, ysort_origin, (maybe_z_index, ezero_ref), is_tilemap, child_of) in query.iter_mut() {

        let (maybe_z_index, maybe_ysort_origin) = if let Some(ezero_ref) = ezero_ref
            && let Ok((ezero_z_index, ezero_ysort_origin)) = ezero_query.get(ezero_ref.0)
        {
            (Some(ezero_z_index.clone()), ezero_ysort_origin.cloned())
        } else if let Some(z_index) = maybe_z_index.cloned() {
            (Some(z_index), ysort_origin.cloned())
        } else {
            (None, None)
        };

        let y_pos = global_transform.translation().y - maybe_ysort_origin.unwrap_or_default().0;

        let use_y_sort = (maybe_ysort_origin.is_some() && parent_sprite_query.get(child_of.0).is_err()) as i32 as f32;

        let target_z = maybe_z_index.unwrap_or_default().used_float() - use_y_sort * y_pos * YSortOrigin::Y_SORT_DIV;

        if (transform.translation.z - target_z).abs() > f32::EPSILON {//NO TOCAR
            transform.translation.z = target_z;
            trace!(target: "zlevel", "Set entity {:?} to z {}", ent, target_z);
            if is_tilemap {
                to_draw.push(DrawTilemap(ent));
            }
        }
    }

    mw_draw_tmap.write_batch(to_draw);
}
