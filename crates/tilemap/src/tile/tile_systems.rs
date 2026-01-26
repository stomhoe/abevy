
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, futures_lite::future};
use bevy_ecs_tilemap::{map::TilemapId, tiles::TileFlip};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::{AnyDisabling, HashId}, common_tag_components::TagSet};
use dimension_shared::{DimensionRef, PrevDimensionRef};
use game_common::game_common_components::*;
use ::sprite_shared::*;
use tilemap_shared::{GlobalGenSettings, GlobalTilePos, HashablePosVec, PrevGlobalTilePos, OplistSize};
use crate::{ chunking_components::ChunkTmapsMap, tile::{tile_components::*, tile_messages::GlobalTilePosChanged, tile_resources::{TileAsyncTasks, TileGposAddition, TileGposTaskResult, TilesAtGpos}}, };

#[allow(unused_parens)]
pub fn flip_tile_horizontally_based_on_initial_pos_hash(
    settings: Single<&GlobalGenSettings>,
    dim_hash_query: Query<&HashId, AnyDisabling>,
    mut query: Query<(AnyOf<(&mut TileFlip, &mut Sprite, &HeldSprites, &Children)>, &InitialPos, Option<&DimensionRef>), 
    (Changed<InitialPos>, With<FlipHorizontallyBasedOnHash>, AnyDisabling, Without<EntityZero>, )>,
    mut sprites_query: Query<(&mut Sprite), (AnyDisabling,  Without<InitialPos>, )>,
) {
    query.iter_mut().for_each(|((tile_flip, sprite, held_sprites, children), initial_pos, dimension_ref)| {
        let dimension_hash = dimension_ref
        .and_then(|dim_ref| dim_hash_query.get(dim_ref.0).ok())
        .cloned()
        .unwrap_or_default();
        
        let should_flip = initial_pos.0.hash_true_false(&settings, dimension_hash, 0);
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
    });
}
#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn spritetile_readjust_transform_to_match_globalpos(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Transform, &GlobalTilePos, Option<&mut Visibility>, Option<&ChildOf>, &EntityZeroRef, Has<Replicated>, Has<KeepDisabled>),
    (Or<(Changed<GlobalTilePos>, Changed<EntityZeroRef>, Changed<ChildOf>, Added<Replicated>)>, 
    AnyDisabling, Without<EntityZero>)>,
    //NO JUNTAR LOS ORS, NO ES EQUIVALENTE
    parent_query: Query<(&GlobalTransform, ), ()>,
    state: Res<State<ClientState>>,
) {//TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    let is_host = *state.get() == ClientState::Disconnected;
    query.iter_mut().for_each(|(ent, mut transform, global_pos, visibility, child_of, ezero_ref, replicated, keep_disabled)| {
        let transl_from_global_pos = global_pos.to_translation(transform.translation.z);
        
        let parent_global_transl = if let Some(child_of) = child_of {
            if let Ok((parent_global_transform, )) = parent_query.get(child_of.parent()) {
                parent_global_transform.translation()
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };
        if is_host || !replicated {
            transform.translation = transl_from_global_pos - parent_global_transl;
        }
        if false == keep_disabled {
            cmd.entity(ent).try_remove::<(Disabled, )>();
        }
        if let Some(visibility) = visibility {//para arreglar un bug de q no se ve
            *visibility.into_inner() = visibility.clone();
        }
    });
}

#[allow(unused_parens)]
pub fn emit_global_tile_pos_change(
    mut query: Query<(Entity, &mut PrevGlobalTilePos, &mut PrevDimensionRef, &GlobalTilePos, &DimensionRef),(Or<(Changed<GlobalTilePos>, Changed<DimensionRef>)>, Without<EntityZero>, )>,
    mut mwriter: MessageWriter<GlobalTilePosChanged>,
) {
    let mut write = Vec::new();
    for (entity, mut prev_tile_pos, mut prev_dim_ref, global_tile_pos, dimension_ref) in query.iter_mut() {
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
    mut map: ResMut<TilesAtGpos>,
    query: Query<(Entity, &DimensionRef, &GlobalTilePos, &TilePos, Option<&TilemapId>, Option<&OplistSize>),(AnyDisabling, Without<EntityZero>, )>,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
) {
    let iter = changed_pos.read().map(|msg| msg.entity);
    let mut entities = Vec::with_capacity(iter.size_hint().0);
    entities.extend(iter);
    
    map.reserve_capacity(entities.len());
    
    query.iter_many(entities).for_each(|(ent, &dimension_ref, &gpos, &tpos, tilemap_id, oplist_size)| {
        map.insert(ent, dimension_ref, gpos, tpos, tilemap_id.copied(), );         
    });
}

pub fn remove_tile_from_gpos_map_on_despawn(
    mut removed_tiles: RemovedComponents<Tile>,
    mut tmap_query: Query<(&mut TileStorage,), (AnyDisabling, )>,
    mut map: ResMut<TilesAtGpos>,
) {
    for tile_ent in removed_tiles.read() {
        let Some((tile_pos, tilemap_id)) = map.remove_entity_and_get_data(tile_ent)
        else {
            continue;
        };
        // Remove from OplistSize positions if applicable
        // if let Some(oplist_size) = oplist_size {
        //     for dy in 0..oplist_size.x() {
        //         for dx in 0..oplist_size.y() {
        //             if dx == 0 && dy == 0 { continue; } // Already handled above
        //             let offset_pos = gpos + IVec2::new(dx as i32, dy as i32);
        //             if let Some(ents_vec) = map.map.get_mut(&(dim, offset_pos)) {
        //                 if let Some(i) = ents_vec.iter().position(|&e| e == removed_tile.entity) {
        //                     ents_vec.swap_remove(i);
        //                     if ents_vec.is_empty() {
        //                         map.map.remove(&(dim, offset_pos));
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        if let Ok((mut tile_storage, )) = tmap_query.get_mut(tilemap_id.0) {
            if let Some(stored_tile_entity) = tile_storage.get(&tile_pos) {
                if stored_tile_entity == tile_ent {
                    tile_storage.remove(&tile_pos);
                }
            }
        }
    }
}

#[allow(unused_parens)]//problema: aunque se despawnee la tile va a ser procesada en process_tiles_pre
pub fn despawn_if_not_excepted(mut cmd: Commands, 
    ezero_query: Query<(Option<&AcZ>, Option<&DeleteOtherTiles>), (With<EntityZero>, AnyDisabling, )>,
    changed_query: Query<(Entity, &DimensionRef, &GlobalTilePos, &EntityZeroRef, Option<&TagSet>, Option<&DeleteOtherTiles>),(Or<(Changed<DimensionRef>, Changed<GlobalTilePos>)>, AnyDisabling, Without<EntityZero>, )>,
    otile_query: Query<(&EntityZeroRef, Option<&TagSet>, Option<&DeleteOtherTiles>), (AnyDisabling, Without<EntityZero>, )>,
    map: Res<TilesAtGpos>,
    mut async_tasks: ResMut<TileAsyncTasks>,
) {
    async_tasks.despawn_tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            for ent in result {
                cmd.entity(ent).try_despawn();
            }
            false
        } else {
            true
        }
    });
    
    if !async_tasks.despawn_tasks.is_empty() {
        return;
    }
    
    let mut inputs = Vec::new();
    //TODO: chequear en la EntityZero si tiene DeleteOtherTiles
    changed_query.iter().for_each(|(newtile_ent, &dim, &gpos, ezero_ref, newtile_tag_hashset, newtile_delete_others_excp)| {
        let Ok((newtile_z, ezero_newtile_delete_others_excp)) = ezero_query.get(ezero_ref.0) else {
            warn!(target: "d", "Failed to get EntityZero for tile entity {:?}, skipping despawn check", newtile_ent);
            return;
        };
        let Some(newtile_z) = newtile_z else {
            warn!(target: "tilemap", "Tile entity {:?} has no AcZ, skipping despawn check", newtile_ent);
            return;
        };
        
        let mut others = Vec::new();
        if let Some(otile_ents) = map.map.get(&(dim, gpos)) {
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
                
                let otile_delete_others_excp = otile_delete_others_excp.or(ezero_otile_delete_others_excp).cloned();
                others.push(OtherTileInfo {
                    ent: otile_ent,
                    z: *otile_z,
                    tags: otile_tag_hashset.cloned(),
                    delete_others_excp: otile_delete_others_excp,
                });
            });
        }
        
        if others.is_empty() {
            return;
        }
        
        let newtile_delete_others_excp = newtile_delete_others_excp.or(ezero_newtile_delete_others_excp).cloned();
        inputs.push(DespawnCandidateInput {
            newtile_ent,
            newtile_z: *newtile_z,
            newtile_tags: newtile_tag_hashset.cloned(),
            newtile_delete_others_excp,
            others,
        });
    });
    
    if inputs.is_empty() {
        return;
    }
    
    let task_pool = AsyncComputeTaskPool::get();
    async_tasks.despawn_tasks.push(task_pool.spawn(async move {
        process_despawn_if_not_excepted_batch(inputs)
    }));
}

#[derive(Clone)]
struct DespawnCandidateInput {
    newtile_ent: Entity,
    newtile_z: AcZ,
    newtile_tags: Option<TagSet>,
    newtile_delete_others_excp: Option<DeleteOtherTiles>,
    others: Vec<OtherTileInfo>,
}

#[derive(Clone)]
struct OtherTileInfo {
    ent: Entity,
    z: AcZ,
    tags: Option<TagSet>,
    delete_others_excp: Option<DeleteOtherTiles>,
}

fn process_despawn_if_not_excepted_batch(inputs: Vec<DespawnCandidateInput>) -> Vec<Entity> {
    let mut to_despawn = Vec::new();
    for input in inputs {
        let mut despawn_newtile = false;
        for other in input.others {
            if other.ent == input.newtile_ent {
                continue;
            }
            
            if let Some(newtile_delete_others_excp) = input.newtile_delete_others_excp.as_ref() {
                if newtile_delete_others_excp.spared_z.contains(&other.z) {
                    // spared by z
                } else if other
                .tags
                .as_ref()
                .is_some_and(|tags| newtile_delete_others_excp.spared_tags.intersects(tags))
                {
                    // spared by tag
                } else {
                    to_despawn.push(other.ent);
                    continue;
                }
            }
            
            if let Some(otile_delete_others_excp) = other.delete_others_excp.as_ref() {
                if otile_delete_others_excp.spared_z.contains(&input.newtile_z) {
                    continue;
                } else if input
                .newtile_tags
                .as_ref()
                .is_some_and(|tags| otile_delete_others_excp.spared_tags.intersects(tags))
                {
                    continue;
                } else {
                    despawn_newtile = true;
                }
            }
        }
        
        if despawn_newtile {
            to_despawn.push(input.newtile_ent);
        }
    }
    
    to_despawn
}
