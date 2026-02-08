use crate::{
    tile::{tile_components::*, tile_messages::GlobalTilePosChanged},
    tilemap_components::*,
    tilemap_resources::*,
};
use ::sprite_shared::*;
use avian2d::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_asset_loader::prelude::*;
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::TilemapId, tiles::TileFlip};
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{AnyDisabling, common_components::HashId, common_tag_components::TagSet};
use dimension_shared::{PrevDimensionRef};
use game_common::game_common_components::*;
use tilemap_shared::{
    ChunkPos, DimensionRef, GlobalGenSettings, GlobalTilePos, HashablePosVec, LoadedChunks, OplistSize, PrevGlobalTilePos, SpriteTilesAtGpos, TileGatheringParamSet
};

#[allow(unused_parens)]
pub fn flip_tile_horizontally_based_on_initial_pos_hash(
    settings: Query<&GlobalGenSettings>,
    dim_hash_query: Query<&HashId, common::AnyDisabling>,
    mut query: Query<
        (
            AnyOf<(&mut TileFlip, &mut Sprite, &HeldSprites, &Children)>,
            &InitialPos,
            Option<&DimensionRef>,
        ),
        (
            Changed<InitialPos>,
            With<FlipHorizontallyBasedOnHash>,
            common::AnyDisabling,
            Without<EntityZero>,
        ),
    >,
    mut sprites_query: Query<(&mut Sprite), (common::AnyDisabling, Without<InitialPos>)>,
) {
    if query.is_empty() {
        return;
    }
    let Ok(settings) = settings.single() else {
        error!("Failed to get global gen settings");
        return;
    };
    query.iter_mut().for_each(
        |((tile_flip, sprite, held_sprites, children), initial_pos, dimension_ref)| {
            let dimension_hash = dimension_ref
                .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
                .cloned()
                .unwrap_or_default();

            let should_flip = initial_pos.0.hash_true_false(settings, dimension_hash, 0);
            if let Some(mut flip) = tile_flip {
                flip.x = should_flip;
            }
            if let Some(mut sprite) = sprite {
                sprite.flip_x = should_flip;
            }
            if let Some(held_sprites) = held_sprites {
                held_sprites.entities().iter().for_each(|&sprite_entity| {
                    if let Ok(mut sprite) = sprites_query.get_mut(sprite_entity) {
                        sprite.flip_x = should_flip;
                    }
                });
            }
            if let Some(children) = children {
                children.iter().for_each(|child| {
                    if let Ok(mut sprite) = sprites_query.get_mut(child) {
                        sprite.flip_x = should_flip;
                    }
                });
            }
        },
    );
}
#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn spritetile_snap_transform_to_global_pos(
    mut cmd: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &GlobalTilePos,
            Option<&mut Visibility>,
            Option<&ChildOf>,
            &EntityZeroRef,
            Has<Replicated>,
            Has<KeepDisabled>,
        ),
        (
            Or<(
                Changed<GlobalTilePos>,
                Changed<EntityZeroRef>,
                Changed<ChildOf>,
                Added<Replicated>,
            )>,
            common::AnyDisabling,
            Without<EntityZero>,
            Without<TilemapAnchor>,
            With<Tile>,
        ),
    >,
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
    mut query: Query<
        (
            Entity,
            &mut PrevGlobalTilePos,
            &mut PrevDimensionRef,
            &GlobalTilePos,
            &DimensionRef,
        ),
        (
            Or<(Changed<GlobalTilePos>, Changed<DimensionRef>)>,
            Without<EntityZero>,
            With<Tile>,
        ),
    >,
    mut mwriter: MessageWriter<GlobalTilePosChanged>,
) {
    let mut write = Vec::new();
    for (entity, mut prev_tile_pos, mut prev_dim_ref, global_tile_pos, dimension_ref) in
        query.iter_mut()
    {
        write.push(GlobalTilePosChanged {
            entity,
            old_gpos: prev_tile_pos.0,
            old_dim: DimensionRef(prev_dim_ref.0),
        });
        prev_tile_pos.0 = *global_tile_pos;
        prev_dim_ref.0 = dimension_ref.0;
    }
    mwriter.write_batch(write);
}

#[allow(unused_parens)]
pub fn add_spawned_tiles_to_gpos_map(
    mut map: ResMut<SpriteTilesAtGpos>,
    query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
        ),
        (common::AnyDisabling, Without<EntityZero>, Without<TilemapId>),
    >,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
) {
    let iter = changed_pos.read().map(|msg| msg.entity);
    let mut entities = Vec::with_capacity(iter.size_hint().0);
    entities.extend(iter);

    map.reserve_capacity(entities.len());

    query.iter_many(entities).for_each(
        |(ent, &dimension_ref, &gpos, )| {
            map.insert(ent, dimension_ref, gpos,);
        },
    );
}

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------

#[allow(unused_parens)]
pub fn on_spritetile_despawn(
    trig: On<Despawn, (Tile, Transform, SpriteTile)>,
    query: Query<(&DimensionRef, &GlobalTilePos, ), (Without<TilemapId>, Without<TilePos>, Without<EntityZero>, AnyDisabling)>,
    mut spritetiles_at_gpos: ResMut<SpriteTilesAtGpos>,
) {
    let Ok((&dim_ref, &gpos)) = query.get(trig.entity) else {
        return;
    };
    spritetiles_at_gpos.remove_tile(dim_ref, gpos, trig.entity);
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

pub fn despawn_if_not_excepted(
    mut cmd: Commands,
    ezero_query: Query<
        (Option<&AcZ>, Option<&DeleteOtherTiles>),
        (With<EntityZero>, common::AnyDisabling),
    >,
    changed_query: Query<
        (
            Entity,
            &DimensionRef,
            &GlobalTilePos,
            &EntityZeroRef,
            Option<&TagSet>,
            Option<&DeleteOtherTiles>,
        ),
        (
            Or<(Changed<DimensionRef>, Changed<GlobalTilePos>)>,
            common::AnyDisabling,
            Without<EntityZero>,
        ),
    >,
    otile_query: Query<
        (&EntityZeroRef, Option<&TagSet>, Option<&DeleteOtherTiles>),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    registered_positions: Res<ImportantRegisteredPositions>,
    tmap_chunk_params: TileGatheringParamSet,
) {
    changed_query.iter().for_each(|(newtile_ent, &dim, &gpos, ezero_ref, newtile_tag_hashset, newtile_delete_others_excp)| {
        let Ok((newtile_z, ezero_newtile_delete_others_excp)) = ezero_query.get(ezero_ref.0) else {
            warn!(target: common::DEBUG_TILE, "Failed to get EntityZero for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let Some(newtile_z) = newtile_z else {
            warn!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", newtile_ent);
            return;
        };

        let otile_ents = tmap_chunk_params.gather_tiles_at(dim, gpos);
        if !otile_ents.is_empty() {
            otile_ents.iter().for_each(|&otile_ent| {
                if otile_ent == newtile_ent {
                    return;
                }
                let Ok((ezero_ref, otile_tag_hashset, otile_delete_others_excp)) = otile_query.get(otile_ent) else {
                    trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);
                    return;
                };
                let Ok((otile_z, ezero_otile_delete_others_excp)) = ezero_query.get(ezero_ref.0) else {
                    trace!(target: "tilemap", "Failed to get EntityZero for tile entity {:?}, skipping despawn check", otile_ent);
                    return;
                };
                let Some(otile_z) = otile_z else {
                    trace!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", otile_ent);
                    return;
                };


                let newtile_delete_others_excp = newtile_delete_others_excp.or(ezero_newtile_delete_others_excp);
                if let Some(newtile_delete_others_excp) = newtile_delete_others_excp {
                    if newtile_delete_others_excp.spared_z.contains(otile_z) {
                        return;
                    }
                    else if let Some(otile_tag_hashset) = otile_tag_hashset
                    && newtile_delete_others_excp.spared_tags.intersects(otile_tag_hashset)
                    {
                        return;
                    }
                    else {
                        trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to new tile entity {:?}", otile_ent, gpos, dim, newtile_ent);
                        // Don't despawn if the tile's EntityZero is registered or exempted
                        if !registered_positions.is_pos_registered(*ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&otile_ent) {
                            cmd.entity(otile_ent).try_despawn();
                        }
                        return;
                    }
                }

                let otile_delete_others_excp = otile_delete_others_excp.or(ezero_otile_delete_others_excp);
                if let Some(otile_delete_others_excp) = otile_delete_others_excp {
                    if otile_delete_others_excp.spared_z.contains(newtile_z) {
                        return;
                    }
                    else if let Some(newtile_tag_hashset) = newtile_tag_hashset
                    && otile_delete_others_excp.spared_tags.intersects(newtile_tag_hashset)
                    {
                        return;
                    }

                    else {
                        trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                        // Don't despawn if the new tile's EntityZero is registered or exempted
                        if !registered_positions.is_pos_registered(*ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&newtile_ent) {
                            cmd.entity(newtile_ent).try_despawn();
                        }
                    }
                }
            });
        }
    });
}

#[allow(unused_parens)]
pub fn make_spritetile_child_of_chunk(
    mut cmd: Commands,
    query: Query<
        (
            Entity,
            &GlobalTilePos,
            &DimensionRef,
            Has<Persisted>,
            Has<PortalTo>,
        ),
        (
            With<Tile>,
            Without<TilemapId>,
            common::AnyDisabling,
            Without<EntityZero>,
            Without<ChildOf>,
        ),
    >,
    loaded_chunks: Res<LoadedChunks>,
) {
    let mut child_ofs = Vec::new();
    query.iter().for_each(|(ent, &global_pos, &dim_ref, to_persist, portal_to)| {
        let chunk_pos: ChunkPos = global_pos.into();

        if to_persist {
            child_ofs.push((ent, ChildOf(dim_ref.0)));
        } else {
            let Some(&chunk) = loaded_chunks.0.get(&(dim_ref, chunk_pos)) else {
                if portal_to {
                    error!(target: "tilemap", "Portal tile entity {:?} at gpos {:?} in dimension {:?} has no loaded chunk to be child of!", ent, global_pos, dim_ref);
                    error!(target: "tilemap", "Portal tile entity {:?} at gpos {:?} in dimension {:?} has no loaded chunk to be child of!", ent, global_pos, dim_ref);
                    error!(target: "tilemap", "Portal tile entity {:?} at gpos {:?} in dimension {:?} has no loaded chunk to be child of!", ent, global_pos, dim_ref);

                }

                cmd.entity(ent).try_despawn();
                return;
            };
            child_ofs.push((ent, ChildOf(chunk)));
        }
    });
    cmd.try_insert_batch(child_ofs);
}

#[allow(unused_parens)]
//todo que se triggeree con un evento cuando el tilemap esté listo, sino puede q las tiles adyacentes no esten cargadas todavia
pub fn tile_adjacency_retexturing_system(
    mut cmd: Commands,
    ezero_query: Query<(&EntityZero), (common::AnyDisabling,)>,
    tilemap_query: Query<(&TileStorage, &HashIdToTexIndex), ()>,
    mut tile_query: Query<(&EntityZeroRef, &mut TileTextureIndex), ()>,
) {
    for mut item in tile_query.iter_mut() {}
}
