use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{HashId, };
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


/*
- HACER Q NINGUNA TILE SEA REPLICATED POR DEFECTO, PERO Q SE PUEDA SYNQUEAR CON UN EVENTO MANDADO DESDE EL CLIENTE, Q LE PASA EL HASH TE LA TILE. EL SERVER LA BUSCA EN SU TileInstancesEntityMap DEL CHUNK (EL CLIENTE NO DEBE ACTUALIZAR RELLENAR LOS TileInstancesEntityMap DEL CHUNK), Y LE RESPONDE AL CLIENTE CON UN EVENTO Q CONTIENE LA ENTIDAD, EL CLIENTE LA MAPEA. HACERLO UN SISTEMA ON-DEMAND
- HACER Q TODOS LOS SISTEMAS STATEFUL DE TILES SE EJECUTEN SERVER-SIDE ONLY (TRAMPAS P. EJ), EL CLIENTE NO EJECUTA ESTOS SISTEMAS ASÍ Q NO LE IMPORTA EL ESTADO INTERNO DE LAS TILES, EL CLIENTE SOLO LE MANDA INPUTS P EJ CUANDO LA ATACA Y EL SERVER RECIBE EL INPUT Y ACTUALIZA EL ESTADO.
- 

*/




// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
#[allow(unused_parens)]
pub fn tile_readjust_transform(
    parent_query: Query<(&GlobalTransform, ), ()>,
    mut query: Query<(&mut Transform, &GlobalTilePos, Option<&ChildOf>, ),(With<Tile>, Added<GlobalTilePos>, Or<(Without<Disabled>, With<Disabled>, With<EntityZeroRef>)>)>,
) {//TODO HACER UN SISTEMA PARA SALVAGUARDAR LOS OFFSETS
    for (mut transform, global_pos, child_of, ) in query.iter_mut() {
        let transl_from_global_pos = global_pos.to_translation(transform.translation.z);
        if let Some(child_of) = child_of {
            if let Ok((parent_global_transform, )) = parent_query.get(child_of.parent()) {
                transform.translation += transl_from_global_pos - parent_global_transform.translation();
            }
        } else {
            transform.translation += transl_from_global_pos;
        }
    }
}
