use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::{common_components::HashId, common_states::GameSetupType};
use dimension_shared::{Dimension, DimensionRef, DimensionRootOplist};
use game_common::game_common_components::*;
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, HashablePosVec, OplistSize};
use crate::{ terrain_gen::{terrgen_events::{ PendingOp, PosSearch, SearchFailed, StudiedOp, SuitablePosFound}, terrgen_resources::RegisteredPositions}, tile::{tile_components::*, tile_resources::*}};




#[allow(unused_parens)]
pub fn flip_tile_along_x(
    settings: Res<AaGlobalGenSettings>,
    mut query: Query<(AnyOf<(&mut TileFlip, &mut Sprite)>, &GlobalTilePos, /*Option<&HeldSprites>*/), (Added<GlobalTilePos>, With<FlipAlongX>, Or<(With<Disabled>, Without<Disabled>)>)>,
) {

    for ((tile_flip, sprite), initial_pos) in query.iter_mut() {
        if let Some(mut flip) = tile_flip{
            flip.x = initial_pos.hash_true_false(&settings, 0);
        }
        else if let Some(mut sprite) = sprite {
            sprite.flip_x = initial_pos.hash_true_false(&settings, 0);
        }
    }
}



#[allow(unused_parens)]
/// WARNING: BORRA DISABLED ANTE CAMBIO DE GLOBALTILEPOS, ENTITYZEROREF O CHILDOF, O SI SE AGREGA REPLICATED
pub fn tile_readjust_transform(
    mut cmd: Commands,
    ezero_query: Query<&Transform, (With<Disabled>, Without<EntityZeroRef>, Without<GlobalTilePos>)>,
    parent_query: Query<(&GlobalTransform, ), ()>,
    mut query: Query<(Entity, &mut Transform, &GlobalTilePos, Option<&ChildOf>, &EntityZeroRef, Has<Replicated>, Has<KeepDisabled>),(With<Tile>, Or<(Changed<GlobalTilePos>, Changed<EntityZeroRef>, Changed<ChildOf>, Added<Replicated>)>, Or<(Without<Disabled>, With<Disabled>, )>)>,
    //NO JUNTAR LOS ORS, NO ES EQUIVALENTE
    state: Res<State<GameSetupType>>,
) {//TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    let is_host = state.get() != &GameSetupType::AsJoiner;


    for (ent, mut transform, global_pos, child_of, ezero_ref, replicated, keep_disabled) in query.iter_mut() {
        let transl_from_global_pos = global_pos.to_translation(transform.translation.z);

        let ezero_transform = ezero_query.get(ezero_ref.0).ok().cloned().unwrap_or_default();

        if let Some(child_of) = child_of && (is_host || !replicated) {
            if let Ok((parent_global_transform, )) = parent_query.get(child_of.parent()) {

                transform.translation = transl_from_global_pos - parent_global_transform.translation() + ezero_transform.translation;
            }
        } else if is_host || !replicated {
            transform.translation = transl_from_global_pos + ezero_transform.translation;
        }

        if false == keep_disabled {
             cmd.entity(ent).try_remove::<(Disabled, )>();
        }
       
    }
}


