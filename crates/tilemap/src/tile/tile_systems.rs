
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapId, tiles::TileFlip};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::DisabledOrNot;
use dimension_shared::{DimensionRef, PrevDimensionRef};
use game_common::game_common_components::*;
use ::sprite_shared::*;
use tilemap_shared::{AcGlobalGenSettings, GlobalTilePos, HashablePosVec, PrevGlobalTilePos, OplistSize};
use crate::{ chunking_components::ChunkTmapsMap, tile::{tile_components::*, tile_messages::GlobalTilePosChanged, tile_resources::TilesAtGpos}, };

#[allow(unused_parens)]
pub fn flip_tile_horizontally_based_on_initial_pos_hash(
    settings: Single<&AcGlobalGenSettings>,
    mut query: Query<(AnyOf<(&mut TileFlip, &mut Sprite, &HeldSprites, &Children)>, &InitialPos, ), 
    (Changed<InitialPos>, With<FlipHorizontallyBasedOnHash>, DisabledOrNot, Without<EntityZero>, )>,
    mut sprites_query: Query<(&mut Sprite), (Or<(With<Disabled>, Without<Disabled>,)>,  Without<InitialPos>, )>,
) {
    for ((tile_flip, sprite, held_sprites, children), initial_pos) in query.iter_mut() {
        if let Some(mut flip) = tile_flip{
            flip.x = initial_pos.0.hash_true_false(&settings, 0);
        }
        if let Some(mut sprite) = sprite {
            sprite.flip_x = initial_pos.0.hash_true_false(&settings, 0);
        }
        if let Some(held_sprites) = held_sprites {
            for &sprite in held_sprites.entities() {
                if let Ok((mut sprite)) = sprites_query.get_mut(sprite) {
                    sprite.flip_x = initial_pos.0.hash_true_false(&settings, 0);
                }
            }
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((mut sprite)) = sprites_query.get_mut(child) {
                    sprite.flip_x = initial_pos.0.hash_true_false(&settings, 0);
                }
            }
        }
    }
}
#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn spritetile_readjust_transform_to_match_globalpos(
    mut cmd: Commands,
    mut query: Query<(Entity, &mut Transform, &GlobalTilePos, Option<&mut Visibility>, Option<&ChildOf>, &EntityZeroRef, Has<Replicated>, Has<KeepDisabled>),
    (Or<(Changed<GlobalTilePos>, Changed<EntityZeroRef>, Changed<ChildOf>, Added<Replicated>)>, 
    Or<(Without<Disabled>, With<Disabled>, )>, Without<EntityZero>
)>,
    //NO JUNTAR LOS ORS, NO ES EQUIVALENTE
    ezero_query: Query<&Transform, (With<EntityZero>, Without<GlobalTilePos>, DisabledOrNot,)>,
    parent_query: Query<(&GlobalTransform, ), ()>,
    state: Res<State<ClientState>>,
) {//TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    let is_host = *state.get() == ClientState::Disconnected;

    for (ent, mut transform, global_pos, visibility, child_of, ezero_ref, replicated, keep_disabled) in query.iter_mut() {
        let transl_from_global_pos = global_pos.to_translation(transform.translation.z);
        let ezero_translation = match ezero_query.get(ezero_ref.0) {
            Ok(transform) => transform.translation,
            Err(_) => {
                warn!(target: "tilemap", "Failed to get EntityZeroRef {:?} for tile entity {:?}, using default Transform", ezero_ref.0, ent);
                Vec3::ZERO
            }
        };
        let parent_global_transl = if let Some(child_of) = child_of {
            if let Ok((parent_global_transform, )) = parent_query.get(child_of.parent()) {
                parent_global_transform.translation()
            } else {
                Vec3::ZERO
            }
        } else {
            Vec3::ZERO
        };
        if is_host || !replicated {// otherwise you get replicated transform if you are a client
            transform.translation = transl_from_global_pos - parent_global_transl + ezero_translation;
        }
        if false == keep_disabled {
            cmd.entity(ent).try_remove::<(Disabled, )>();
        }
        if let Some(visibility) = visibility {// DON'T REMOVE
            *visibility.into_inner() = visibility.clone();
        }
    }
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
    query: Query<(Entity, &DimensionRef, &GlobalTilePos, Option<&OplistSize>),(Changed<GlobalTilePos>, Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
) {
    for msg in changed_pos.read() {
        if let Some(ents_vec) = map.0.get_mut(&(msg.old_dim, msg.old_gpos)) {
            if let Some(i) = ents_vec.iter().position(|&e| e == msg.entity) {
                ents_vec.swap_remove(i);
            }
        }
    }
    for (ent, dimension_ref, gpos, oplist_size) in query.iter() {
        if !map.0.entry((*dimension_ref, *gpos)).or_default().contains(&ent) {
            map.0.entry((*dimension_ref, *gpos)).or_default().push(ent);
            trace!(target: "add2gposmap", "Added tile entity {:?} at gpos {:?} in dimension {:?}", ent, gpos, dimension_ref);
            if let Some(oplist_size) = oplist_size {
                //TODO llenar todas las posiciones que ocupe la tile
    
            }
        }
    }
}

pub fn remove_tile_from_gpos_map(
    removed_tile: On<Despawn, (DimensionRef, GlobalTilePos)>,
    query: Query<(&DimensionRef, &GlobalTilePos, Option<&TilemapId>, Option<&TilePos>),(Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    mut tmap_query: Query<(&mut TileStorage,), (Or<(Without<Disabled>, With<Disabled>)>, )>,
    mut map: ResMut<TilesAtGpos>,
) {
    let Ok((&dim, &gpos, tilemap_id, tile_pos)) = query.get(removed_tile.entity) else {
        trace!(target: "gposmap_remove", "Failed to get DimensionRef and GlobalTilePos for removed tile entity {:?}", removed_tile.entity);
        return;
    };
    if let Some(ents_vec) = map.0.get_mut(&(dim, gpos)) {
        if let Some(i) = ents_vec.iter().position(|&e| e == removed_tile.entity) {
            ents_vec.swap_remove(i);
            trace!(target: "gposmap_remove", "Removed tile entity {:?} at gpos {:?} in dimension {:?}", removed_tile.entity, gpos, dim);
            if ents_vec.is_empty() {
                map.0.remove(&(dim, gpos));
                trace!(target: "gposmap_remove", "Removed last tile at gpos {:?} in dimension {:?}", gpos, dim);
            }
        }
    }
    if let (Some(tilemap_id), Some(tile_pos)) = (tilemap_id, tile_pos) {
        let Ok((mut tile_storage, )) = tmap_query.get_mut(tilemap_id.0) else {
            return ;
        };
        if let Some(stored_tile_entity) = tile_storage.get(&tile_pos) {
            if stored_tile_entity == removed_tile.entity {
                tile_storage.remove(tile_pos);
            }
        }
    }
}

#[allow(unused_parens)]//problema: aunque se despawnee la tile va a ser procesada en process_tiles_pre
pub fn despawn_if_not_excepted(mut cmd: Commands, 
    acz_query: Query<&AcZ, (With<EntityZero>, DisabledOrNot, )>,
    changed_query: Query<(Entity, &DimensionRef, &GlobalTilePos, Option<&DeleteOthersExceptZLevels>, &EntityZeroRef,),(Or<(Changed<DimensionRef>, Changed<GlobalTilePos>)>, Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    otile_query: Query<(Option<&DeleteOthersExceptZLevels>, &EntityZeroRef,), (Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    map: Res<TilesAtGpos>,
) {// poner el contenido de esto en process_tiles_pre? asi se intercepta a tiempo
    for (newtile_ent, &dim, &gpos, newtile_delete_others_excp, ezero_ref, ) in changed_query.iter() {

        let Ok(new_tile_z) = acz_query.get(ezero_ref.0) else {
            warn!(target: "tilemap", "Failed to get AcZ for tile entity {:?}, skipping despawn check", newtile_ent);
            continue ;
        };

        if let Some(otile_ents) = map.0.get(&(dim, gpos)) {
            for &otile_ent in otile_ents.iter() {
                if otile_ent == newtile_ent {
                    continue;//skip self
                }
                let Ok((otile_delete_others_excp, ezero_ref, )) = otile_query.get(otile_ent) else {
                    trace!(target: "tilemap", "Failed to get prev tile entity {:?}, skipping despawn check", otile_ent);    
                    continue ;
                };
                let Ok(otile_z) = acz_query.get(ezero_ref.0) else {
                    trace!(target: "tilemap", "Failed to get AcZ for tile entity {:?}, skipping despawn check", otile_ent);
                    continue ;
                };
                if let Some(newtile_delete_others_excp) = newtile_delete_others_excp {
                    if newtile_delete_others_excp.0.contains(otile_z) {
                        continue;
                    } else {
                        trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to new tile entity {:?}", otile_ent, gpos, dim, newtile_ent);
                        cmd.entity(otile_ent).try_despawn();
                        continue;
                    }
                }

                if let Some(otile_delete_others_excp) = otile_delete_others_excp {
                    if otile_delete_others_excp.0.contains(new_tile_z) {
                        continue;
                    } else {
                        trace!(target: "tilemap", "Despawning tile entity {:?} at gpos {:?} in dimension {:?} due to old tile entity {:?}", newtile_ent, gpos, dim, otile_ent);
                        cmd.entity(newtile_ent).try_despawn();
                    }
                }
            }
        }
    }
}
