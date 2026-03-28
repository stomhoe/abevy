use crate::{
    tile::{tile_components::*, tile_messages::*},

};
use avian2d::prelude::*;
use bevy::ecs::entity::EntityHashSet;

use bevy::prelude::*;
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::TilemapId, };
#[allow(unused_imports, )]use bevy_replicon::prelude::*;
use game_common::game_common_components::*;
use ::tilemap_shared::*;

pub type ExcludedComps = (Without<Templ>, Without<TilemapAnchor>, Without<TilePos>);

#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn snap_transform_to_gpos(
    mut cmd: Commands,
    mut main_query: Query<
        (Entity, Option<&mut Transform>, &GlobalTilePos, Option<&mut Visibility>, Option<&ChildOf>, ),
        (With<SnapTransformToGpos>, common::AnyDisabling, ExcludedComps),
    >,
    gpos_state_query: Query<
        (Entity, Ref<GlobalTilePos>, &SnapTransformToGpos, ),
        (common::AnyDisabling, Changed<GlobalTilePos>, ExcludedComps),
    >,
    parent_query: Query<&GlobalTransform, common::AnyDisabling>,
    mut ents_to_process: Local<EntityHashSet>,
) {
    //TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    for (ent, gpos, snap_on_gpos, ) in gpos_state_query {
        let should_snap = match snap_on_gpos {
            SnapTransformToGpos::OnChange => gpos.is_changed(),
            SnapTransformToGpos::OnAdd => gpos.is_added(),
        };
        if should_snap {
            ents_to_process.insert(ent);
        }
    }
    let mut iter = main_query.iter_many_mut(ents_to_process.drain());
    while let Some((
        ent,
        transform,
        global_pos,
        visibility,
        child_of,
    )) = iter.fetch_next() {
        let z = transform.as_ref().map(|t| t.translation.z).unwrap_or_default();
        let transl_from_global_pos = global_pos.to_translation(z);

        let parent_global_transl = child_of
            .and_then(|co| parent_query.get(co.parent()).ok())
            .map(|t| t.translation())
            .unwrap_or(Vec3::ZERO);

        let local_translation = transl_from_global_pos - parent_global_transl;
        if let Some(mut transform) = transform {
            transform.translation = local_translation;
        } else {
            cmd.entity(ent).try_insert(Transform::from_translation(local_translation));
        }

        if let Some(visibility) = visibility {
            //DON'T REMOVE, FIXES A BUG
            *visibility.into_inner() = visibility.clone();
        }
    }
}
#[allow(unused_parens)]
pub fn emit_global_tile_pos_change(
    mut cmd: Commands,
    mut query: Query<
        (
            Entity, Option<&mut PrevPos>,
            &GlobalTilePos, &DimensionRef,
        ),
        (
            Or<(Changed<GlobalTilePos>, Changed<DimensionRef>)>,
            Without<Templ>, With<Tile>,
        ),
    >,
    mut mwriter: MessageWriter<GlobalTilePosChanged>,
    mut changed: Local<Vec<GlobalTilePosChanged>>,
) {
    let iter = query.iter();
    changed.reserve(iter.size_hint().1.unwrap_or(iter.size_hint().0));
    for (entity, prev_tile_pos, global_tile_pos, &dimension_ref) in query.iter_mut() {
        let old = prev_tile_pos.as_deref().map(|prev| (prev.dim, prev.gpos));
        if old != Some((dimension_ref, *global_tile_pos)) {
            changed.push(GlobalTilePosChanged {
                entity,
                old: old.map(|(dim, gpos)| PrevPos { gpos, dim }),
            });
            if let Some(mut prev_tile_pos) = prev_tile_pos {
                prev_tile_pos.gpos = *global_tile_pos;
                prev_tile_pos.dim = dimension_ref;
            } else {
                cmd.entity(entity).try_insert(PrevPos {
                    gpos: *global_tile_pos,
                    dim: dimension_ref,
                });
            }
        }
    }
    mwriter.write_batch(changed.drain(..));
}

#[allow(unused_parens)]
pub fn add_spawned_tiles_to_gpos_map(
    mut map: ResMut<SpriteTilesAtGpos>,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos, &TemplEntiRef),
        (common::AnyDisabling, Without<Templ>, Without<TilemapId>),
    >,
    interaction_zones_query: Query<&InteractionZones, common::AnyDisabling>,
    mut entities: Local<EntityHashSet>,
) {
    entities.reserve(changed_pos.len());
    for changed_pos in changed_pos.read() {
        let Some(old) = changed_pos.old else {
            entities.insert(changed_pos.entity);
            continue;
        };
        let interaction_zones = query
            .get(changed_pos.entity)
            .ok()
            .and_then(|(_, _, _, templ_ref)| interaction_zones_query.get(templ_ref.0).ok());
        map.remove_tile(old.dim, old.gpos, changed_pos.entity, interaction_zones);
        entities.insert(changed_pos.entity);
    }
    for ent in entities.drain() {
        let Ok((ent, &dimension_ref, &gpos, templ_ref)) = query.get(ent) else { continue };
        let interaction_zones = interaction_zones_query.get(templ_ref.0).ok();
        map.insert(ent, dimension_ref, gpos, interaction_zones);
    }
}

#[allow(unused_parens)]
pub fn add_projectile_colliders_to_tiles(
    mut cmd: Commands,
    query: Query<
        (Entity, &GlobalTilePos, Option<&OplistSize>),
        (Added<BlocksProjectiles>, With<Tile>, Without<Templ>),
    >,
) {
    for (ent, gpos, oplist_size) in query.iter() {
        let size = oplist_size.map(|size| size.inner()).unwrap_or(UVec2::ONE);
        let tile_size = Vec2::new(
            GlobalTilePos::TILE_SIZE_PXS.x as f32 * size.x as f32,
            GlobalTilePos::TILE_SIZE_PXS.y as f32 * size.y as f32,
        );
        let transform = Transform::from_translation(gpos.to_translation(0.0));

        cmd.entity(ent).try_insert((
            RigidBody::Static,
            Collider::rectangle(tile_size.x, tile_size.y),
            transform,
            GlobalTransform::default(),
        ));
    }
}
