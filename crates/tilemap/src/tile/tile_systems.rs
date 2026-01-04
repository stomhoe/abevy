
use bevy::ecs::entity_disabling::Disabled;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::{map::TilemapId, tiles::TileFlip};
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use dimension_shared::{DimensionRef, PrevDimensionRef};
use game_common::game_common_components::*;
use ::sprite_shared::*;
use tilemap_shared::{AcGlobalGenSettings, GlobalTilePos, HashablePosVec, PrevGlobalTilePos, OplistSize};
use crate::{ chunking_components::LayersMap, tile::{tile_components::*, tile_messages::GlobalTilePosChanged, tile_resources::TilesAtGpos}, tilemap_resources::DeleteOthersExceptZLevels};

#[allow(unused_parens)]
pub fn flip_tile_horizontally_based_on_initial_pos_hash(
    settings: Single<&AcGlobalGenSettings>,
    mut query: Query<(AnyOf<(&mut TileFlip, &mut Sprite, &HeldSprites, &Children)>, &InitialPos, ), (Changed<InitialPos>, With<FlipHorizontallyBasedOnHash>, Or<(With<Disabled>, Without<Disabled>)>, Without<EntityZero>, )>,
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
    ezero_query: Query<&Transform, (With<EntityZero>, Without<GlobalTilePos>, Or<(With<Disabled>, Without<Disabled>)>,)>,
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
pub fn add_tile_to_gpos_map(
    mut map: ResMut<TilesAtGpos>,
    query: Query<(Entity, &DimensionRef, &GlobalTilePos, Option<&OplistSize>),(Changed<GlobalTilePos>, Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    mut removed_tiles: RemovedComponents<Tile>,
    mut changed_pos: MessageReader<GlobalTilePosChanged>,
) {
    for (ent, dimension_ref, gpos, oplist_size) in query.iter() {
        map.0.entry((*dimension_ref, *gpos)).or_default().push(ent);
        if let Some(oplist_size) = oplist_size {
            //TODO llenar todas las posiciones que ocupe la tile

        }
    }
    for removed_tile in removed_tiles.read() {
        if let Ok((_, &dim, &gpos, oplist_size)) = query.get(removed_tile) {
            if let Some(ents_vec) = map.0.get_mut(&(dim, gpos)) {
                if let Some(i) = ents_vec.iter().position(|&e| e == removed_tile) {
                    ents_vec.swap_remove(i);
                }
            }
        }
    }
    for msg in changed_pos.read() {
        if let Some(ents_vec) = map.0.get_mut(&(msg.old_dim, msg.old_gpos)) {
            if let Some(i) = ents_vec.iter().position(|&e| e == msg.entity) {
                ents_vec.swap_remove(i);
            }
        }
    }
}


#[allow(unused_parens)]
pub fn despawn_if_not_excepted(mut cmd: Commands, 
    query: Query<(Entity, &DimensionRef, &GlobalTilePos, &DeleteOthersExceptZLevels, &AcZ,),(Added<InitialPos>, Or<(Without<Disabled>, With<Disabled>)>, Without<EntityZero>, )>,
    map: Res<TilesAtGpos>,
) {
    for (newtile_ent, &dim, &gpos, newtile_delete_others_excp, new_tile_z, ) in query.iter() {
        
        if let Some(otile_ents) = map.0.get(&(dim, gpos)) {
            for &otile_ent in otile_ents.iter() {
                if otile_ent == newtile_ent {
                    continue ;
                }
                let Ok((_, _, _, otile_delete_others_excp, otile_z, )) = query.get(otile_ent) else {
                    continue ;
                };

                if let Some(newtile_delete_others_excp) = &newtile_delete_others_excp.0 {
                    if newtile_delete_others_excp.contains(otile_z) {
                        continue ;
                    } else{
                        cmd.entity(otile_ent).try_despawn();
                        continue ;
                    }
                }

                let Some(otile_delete_others_excp) = &otile_delete_others_excp.0 else {
                    continue ;
                };
                if otile_delete_others_excp.contains(new_tile_z) {
                    continue ;
                } else{
                    cmd.entity(newtile_ent).try_despawn();
                }
            }
        }
    
        if newtile_delete_others_excp.0.is_none() {
            cmd.entity(newtile_ent).try_remove::<DeleteOthersExceptZLevels>();
        }
    }
}
