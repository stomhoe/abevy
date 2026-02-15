use crate::{
    tile::{tile_components::*, tile_messages::*},

    tilemap_resources::*,
};
use ::sprite_shared::*;
use avian2d::prelude::*;
use bevy::ecs::entity_disabling::Disabled;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::TilemapId, tiles::TileFlip};
use bevy_replicon::prelude::*;
use common::{AnyDisabling, common_components::HashId, common_tag_components::TagSet};
use game_common::game_common_components::*;
use ::tilemap_shared::*;

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
    mut query: Query<(Entity, &mut Transform, &GlobalTilePos, Option<&mut Visibility>, Option<&ChildOf>, &EntityZeroRef, Has<Replicated>, Has<KeepDisabled>), (Or<(Changed<GlobalTilePos>, Changed<EntityZeroRef>, Changed<ChildOf>, Added<Replicated>)>, common::AnyDisabling, Without<EntityZero>, Without<TilemapAnchor>, With<Tile>)>,
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
    mut changed: Local<Vec<GlobalTilePosChanged>>,
) {
    changed.reserve(query.iter().size_hint().0);
    for (entity, mut prev_tile_pos, mut prev_dim_ref, global_tile_pos, &dimension_ref) in
        query.iter_mut()
    {
        if global_tile_pos != &prev_tile_pos.0 || dimension_ref.0 != prev_dim_ref.0 {
            changed.push(GlobalTilePosChanged {
                entity,
                old_gpos: prev_tile_pos.0,
                old_dim: DimensionRef(prev_dim_ref.0),
            });
            prev_tile_pos.0 = *global_tile_pos;
            prev_dim_ref.0 = dimension_ref.0;
        }
    }
    mwriter.write_batch(changed.drain(..));
}

#[allow(unused_parens)]
pub fn add_spawned_tiles_to_gpos_map(
    mut map: ResMut<SpriteTilesAtGpos>,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    query: Query<
        (Entity, &DimensionRef, &GlobalTilePos),
        (common::AnyDisabling, Without<EntityZero>, Without<TilemapId>),
    >,
    mut entities: Local<Vec<Entity>>,
) {
    entities.reserve(changed_pos.len());
    for changed_pos in changed_pos.read() {
        map.remove_tile(changed_pos.old_dim, changed_pos.old_gpos, changed_pos.entity);
        entities.push(changed_pos.entity);
    }
    query.iter_many(entities.drain(..)).for_each(
        |(ent, &dimension_ref, &gpos, )| {
            map.insert(ent, dimension_ref, gpos,);
        },
    );
}

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
    ezero_query: Query<
        (Option<&AcZ>, Option<&DeleteOtherTiles>),
        (With<EntityZero>, common::AnyDisabling),
    >,
    query: Query<
        (
            Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef,
            Option<&TagSet>, Option<&DeleteOtherTiles>,
        ),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    otile_query: Query<
        (&EntityZeroRef, Option<&TagSet>, Option<&DeleteOtherTiles>),
        (common::AnyDisabling, Without<EntityZero>),
    >,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
    registered_positions: Res<ImportantRegisteredPositions>,
    params: TileGatheringParamSet,
    mut otile_ents: Local<Vec<Entity>>,
    mut writer: MessageWriter<SafeDespawn>,
    mut msgs: Local<Vec<SafeDespawn>>,
) {
    query.iter_many(changed_pos.read().map(|msg| msg.entity)).for_each(|(newtile_ent, &dim, &gpos, ezero_ref, newtile_tag_hashset, newtile_delete_others_excp)| {
        let Ok((newtile_z, ezero_newtile_delete_others_excp)) = ezero_query.get(ezero_ref.0) else {
            warn!(target: common::DEBUG_TILE, "Failed to get EntityZero for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let Some(newtile_z) = newtile_z else {
            warn!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", newtile_ent);
            return;
        };
        params.gather_tiles_at(&mut *otile_ents, dim, gpos);
        otile_ents.drain(..).for_each(|otile_ent| {
            if otile_ent == newtile_ent {
                return;
            }
            let Ok((otile_ezero_ref, otile_tag_hashset, otile_delete_others_excp)) = otile_query.get(otile_ent) else {
                trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);
                return;
            };
            let Ok((otile_z, ezero_otile_delete_others_excp)) = ezero_query.get(otile_ezero_ref.0) else {
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
                    if !registered_positions.is_pos_registered(*otile_ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&otile_ent) {
                        msgs.push(SafeDespawn(otile_ent));
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
                && otile_delete_others_excp.spared_tags.intersects(newtile_tag_hashset) {
                    return;
                }
                else {
                    trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                    if !registered_positions.is_pos_registered(*ezero_ref, dim, gpos) && !registered_positions.exempted.contains(&newtile_ent) {
                        msgs.push(SafeDespawn(newtile_ent));
                    }
                }
            }
        });
    });
    writer.write_batch(msgs.drain(..));
}
#[allow(unused_parens)]
pub fn reckeck_adjacency_for(
    mut reader: MessageReader<GlobalTilePosChanged>,
    mut writer: MessageWriter<RecheckTileAdjacency>,
    tiles_query: Query<(&DimensionRef, &GlobalTilePos), (With<Tile>, common::AnyDisabling)>,
    mut msgs: Local<Vec<RecheckTileAdjacency>>,
) {
    for read in reader.read() {
        if read.old_gpos != PrevGlobalTilePos::PLACEHOLDER_I32_MAX.0 {
            RecheckTileAdjacency::append_all_adjacent_pos(&mut msgs, read.old_dim, read.old_gpos, );
        }
        if let Ok((&new_dim, &new_gpos)) = tiles_query.get(read.entity) {
            msgs.push(RecheckTileAdjacency {
                dim: new_dim,
                gpos: new_gpos,
            });
            RecheckTileAdjacency::append_all_adjacent_pos(&mut msgs, new_dim, new_gpos, );
        }
    }
    writer.write_batch(msgs.drain(..));
}
#[allow(unused_parens)]
/// should implement something similar to Godot's autotiling system
pub fn tile_adjacency_retexturing_system(
    mut reader: MessageReader<RecheckTileAdjacency>,
    mut tile_query: Query<(&EntityZeroRef, &DimensionRef, &GlobalTilePos, Option<&mut sprite_animation_shared::AnimExtraState>, Option<(&mut TileTextureIndex, &mut TileFlip, &TilemapId)>, ), ()>,
    ezero_query: Query<(&HashId, Option<&AdjRetexConfig>), ()>,
    hash2tex_query: Query<(&HashIdToTexIndex), ()>,
    params: TileGatheringParamSet,
    mut adj_tiles_ezeros_hash_ids: Local<Vec<(DiagonalCardinalDirection, HashId)>>,
    mut north_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut south_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut west_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut east_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut northeast_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut northwest_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut southeast_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut southwest_adj_tiles_ezeros: Local<Vec<Entity>>,
    mut unique_rechecks: Local<HashSet<(DimensionRef, GlobalTilePos)>>,
) {
    unique_rechecks.clear();
    for msg in reader.read() {
        let key = (msg.dim, msg.gpos);
        if !unique_rechecks.insert(key) {
            continue;
        }
        north_adj_tiles_ezeros.clear();
        params.gather_tiles_at(&mut *north_adj_tiles_ezeros, msg.dim, msg.gpos);
        let mut tiles_to_recheck: Vec<Entity> = north_adj_tiles_ezeros.drain(..).collect();

        for tile_ent in tiles_to_recheck.drain(..) {
            let Ok((ezero_ref, &dim, &gpos, ..)) = tile_query.get(tile_ent) else {
                continue;
            };
            let Ok((_, Some(adj_retex_config))) = ezero_query.get(ezero_ref.0) else {
                continue;
            };
            north_adj_tiles_ezeros.clear();
            south_adj_tiles_ezeros.clear();
            west_adj_tiles_ezeros.clear();
            east_adj_tiles_ezeros.clear();
            northeast_adj_tiles_ezeros.clear();
            northwest_adj_tiles_ezeros.clear();
            southeast_adj_tiles_ezeros.clear();
            southwest_adj_tiles_ezeros.clear();
            adj_tiles_ezeros_hash_ids.clear();

            params.gather_tiles_at(&mut *north_adj_tiles_ezeros, dim, gpos.adjacent_north());
            params.gather_tiles_at(&mut *south_adj_tiles_ezeros, dim, gpos.adjacent_south());
            params.gather_tiles_at(&mut *west_adj_tiles_ezeros, dim, gpos.adjacent_west());
            params.gather_tiles_at(&mut *east_adj_tiles_ezeros, dim, gpos.adjacent_east());
            params.gather_tiles_at(&mut *northeast_adj_tiles_ezeros, dim, gpos.adjacent_northeast());
            params.gather_tiles_at(&mut *northwest_adj_tiles_ezeros, dim, gpos.adjacent_northwest());
            params.gather_tiles_at(&mut *southeast_adj_tiles_ezeros, dim, gpos.adjacent_southeast());
            params.gather_tiles_at(&mut *southwest_adj_tiles_ezeros, dim, gpos.adjacent_southwest());

            let mut process_adjacent_tiles = |direction: DiagonalCardinalDirection, adj_tiles: &mut Vec<Entity>| {
                for adj_tile_ent in adj_tiles.drain(..) {
                    let Ok((ezero_ref, ..)) = tile_query.get(adj_tile_ent) else {
                        continue;
                    };
                    let Ok((&hid, ..)) = ezero_query.get(ezero_ref.0) else {
                        continue;
                    };
                    adj_tiles_ezeros_hash_ids.push((direction, hid));
                }
            };
            process_adjacent_tiles(DiagonalCardinalDirection::North, &mut north_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::South, &mut south_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::West, &mut west_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::East, &mut east_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthEast, &mut northeast_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::NorthWest, &mut northwest_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthEast, &mut southeast_adj_tiles_ezeros);
            process_adjacent_tiles(DiagonalCardinalDirection::SouthWest, &mut southwest_adj_tiles_ezeros);

            let Some((hid_to_use, new_flip)) = adj_retex_config.get_tex_in_curr_adjacency_state(&adj_tiles_ezeros_hash_ids) else {
                continue;
            };
            let Ok((ezero_ref, .., anim_state, tmap_tile_data)) = tile_query.get_mut(tile_ent) else {
                continue;
            };
            let Ok((&tile_hid, ..)) = ezero_query.get(ezero_ref.0) else {
                continue;
            };
            if let Some((mut tex_idx, mut flip, tmap_ent)) = tmap_tile_data {
                let Ok(hash2tex) = hash2tex_query.get(tmap_ent.0) else {
                    continue;
                };
                if let Some(new_flip) = new_flip {
                    *flip = new_flip;
                }
                if let Ok(new_tex_idx) = hash2tex.get(tile_hid, hid_to_use) {
                    *tex_idx = new_tex_idx;
                }
            } else if let Some(mut anim_state) = anim_state {
                anim_state.0 = hid_to_use;
            }
        }
    }
}

pub fn safe_despawn_tile_at(
    mut cmd: Commands,
    mut reader: MessageReader<SafeDespawn>,
    mut recheck_writer: MessageWriter<RecheckTileAdjacency>,
    loaded_chunks: Res<LoadedChunks>,
    chunk_children: Query<&Tilemaps>,
    mut tilemap_query: Query<(&SizeInTiles, &mut TileStorage, &HashIdToTexIndex)>,
    tile_query: Query<(&DimensionRef, &GlobalTilePos), (With<Tile>, common::AnyDisabling)>,
    mut rechecks: Local<Vec<RecheckTileAdjacency>>,
) {
    for &SafeDespawn(tile_ent) in reader.read() {
        let Ok((&dim, &gpos)) = tile_query.get(tile_ent) else {
            cmd.entity(tile_ent).try_despawn();
            continue;
        };

        cmd.entity(tile_ent).try_despawn();
        rechecks.push(RecheckTileAdjacency { dim, gpos });
        RecheckTileAdjacency::append_all_adjacent_pos(&mut rechecks, dim, gpos);

        let chunk_pos = gpos.to_chunkpos();
        let Some(&chunk_ent) = loaded_chunks.0.get(&(dim, chunk_pos)) else {
            continue;
        };
        let Ok(tilemaps) = chunk_children.get(chunk_ent) else {
            continue;
        };
        for &tmap_ent in tilemaps.entities() {
            let Ok((&size_in_tiles, mut storage, ..)) = tilemap_query.get_mut(tmap_ent) else {
                continue;
            };
            let tpos = gpos.to_tilepos(size_in_tiles);
            let Some(found_tile_ent) = storage.get(&tpos) else {
                continue;
            };
            if tile_ent == found_tile_ent {
                storage.remove(&tpos);
            }
        }
    }
    recheck_writer.write_batch(rechecks.drain(..));
}
