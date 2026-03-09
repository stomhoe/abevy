use crate::{
    tile::{tile_components::*, tile_messages::*},

    tilemap_resources::*,
};
use ::sprite_shared::prelude::*;
use avian2d::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::TilemapId, tiles::TileFlip};
use bevy_replicon::prelude::*;
use game_common::game_common_components::*;
use ::tilemap_shared::*;

#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn spritetile_snap_transform_to_global_pos(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Transform, &GlobalTilePos, Option<&mut Visibility>, Option<&ChildOf>, &EntityZeroRef, Has<Replicated>, Has<KeepDisabled>),
        (Or<(Changed<GlobalTilePos>, Changed<EntityZeroRef>, Changed<ChildOf>, Added<Replicated>)>, common::AnyDisabling, Without<EntityZero>, Without<TilemapAnchor>, With<Tile>)>,
    //NO JUNTAR LOS ORS, NO ES EQUIVALENTE
    parent_query: Query<&GlobalTransform, common::AnyDisabling>,
    state: Res<State<ClientState>>,
) {
    //TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    let is_host = *state.get() == ClientState::Disconnected;
    query.iter_mut().for_each(
        |(
            ent,
            mut transform,
            global_pos,
            visibility,
            child_of,
            _ezero_ref,
            replicated,
            keep_disabled,
        )| {
            let transl_from_global_pos = global_pos.to_translation(transform.translation.z);

            let parent_global_transl = child_of
                .and_then(|co| parent_query.get(co.parent()).ok())
                .map(|t| t.translation())
                .unwrap_or(Vec3::ZERO);

            if is_host || !replicated {
                transform.translation = transl_from_global_pos - parent_global_transl;
            }
            if false == keep_disabled {
                cmd.entity(ent).try_remove::<(Disabled,)>();
            }
            if let Some(visibility) = visibility {
                //para arreglar un bug de q no se ve
                *visibility.into_inner() = visibility.clone();
            }
        },
    );
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
            Without<EntityZero>, With<Tile>,
        ),
    >,
    mut mwriter: MessageWriter<GlobalTilePosChanged>,
    mut changed: Local<Vec<GlobalTilePosChanged>>,
) {
    changed.reserve(query.iter().size_hint().0);
    for (entity, prev_tile_pos, global_tile_pos, &dimension_ref) in
        query.iter_mut()
    {
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
        (Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef),
        (common::AnyDisabling, Without<EntityZero>, Without<TilemapId>),
    >,
    ezero_size_query: Query<&SizeInTiles, (common::AnyDisabling)>,
    mut entities: Local<EntityHashSet>,
) {
    entities.reserve(changed_pos.len());
    for changed_pos in changed_pos.read() {
        let Some(old) = changed_pos.old else {
            entities.insert(changed_pos.entity);
            continue;
        };
        let size = query
            .get(changed_pos.entity)
            .ok()
            .and_then(|(_, _, _, ezero_ref)| ezero_size_query.get(ezero_ref.0).ok().copied())
            .unwrap_or_default();
        map.remove_tile(old.dim, old.gpos, changed_pos.entity, size);
        entities.insert(changed_pos.entity);
    }
    for ent in entities.drain() {
        let Ok((ent, &dimension_ref, &gpos, ezero_ref)) = query.get(ent) else { continue };
        let size = ezero_size_query.get(ezero_ref.0).copied().unwrap_or_default();
        map.insert(ent, dimension_ref, gpos, size);
    }
}

#[allow(unused_parens)]
pub fn add_projectile_colliders_to_tiles(
    mut cmd: Commands,
    query: Query<
        (Entity, &GlobalTilePos, Option<&OplistSize>),
        (Added<BlocksProjectiles>, With<Tile>, Without<EntityZero>),
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
