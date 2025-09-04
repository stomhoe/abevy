use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{ecs::{entity::MapEntities, entity_disabling::Disabled}, platform::collections::{HashMap, HashSet}};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{HashId, StrId};
use dimension_shared::DimensionRootOplist;
use game_common::game_common_components::*;
use player::player_components::{HostPlayer, OfSelf, Player};
use rand_distr::StudentT;
use tilemap_shared::{AaGlobalGenSettings, GlobalTilePos, HashablePosVec};
use crate::{ terrain_gen::terrgen_events::{PendingOp, PosSearch, SearchFailed, StudiedOp, SuitablePosFound}, tile::{tile_components::*, tile_resources::*}};




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

//sys

#[derive(serde::Deserialize, Event, serde::Serialize, Clone, MapEntities)]
pub struct SyncTilesToServer {

}

/* No olvidarse de agregarlo al Plugin del módulo
           .add_client_trigger::<SyncTilesToServer>(Channel::Ordered)
           .add_mapped_client_trigger::<SyncTilesToServer>(Channel::Ordered)
*/

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO -----------------------------
//                                                       ^^^^
#[allow(unused_parens)]// ESBOZO
pub fn esbozo_add_tile_instances_to_map(mut cmd: Commands, 
    mut tile_instances_map: ResMut<TileInstancesEntityMap>,
    mut query: Query<(Entity, &InitialPos, &MyZ, &TileRef), (Added<InitialPos>, Or<(With<Disabled>, Without<Disabled>)>)>,
    mut oritile_query: Query<(&StrId), (With<Tile>, )>
) {
    for (ent, initial_pos, my_z, tile_ref) in query.iter() {
        let mut hasher = DefaultHasher::new();
        initial_pos.hash(&mut hasher);
        my_z.hash(&mut hasher);
        tile_ref.hash(&mut hasher);
        let Ok(oritile_strid) = oritile_query.get(tile_ref.0) else {
            continue;
        };

        let hash = HashId::new(hasher.finish());

        let _ = tile_instances_map.0.insert_with_hash(hash, ent);
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
pub fn instantiate_portal(mut cmd: Commands,
    new_portals: Query<(Entity, &StrId, &PortalTemplate, &GlobalTilePos),(Or<(With<Disabled>, Without<Disabled>)>, Without<SearchingForSuitablePos>)>,
    pending_search: Query<(Entity, &StrId, &PortalTemplate, &GlobalTilePos),(Or<(With<Disabled>, Without<Disabled>)>, With<SearchingForSuitablePos>)>,
    mut ew_pending_ops: EventWriter<PosSearch>,
    mut ereader_search_failed: EventReader<SearchFailed>,
    mut ereader_search_successful: EventReader<SuitablePosFound>,

) {
    let mut started_searches: HashMap<StudiedOp, Entity> = HashMap::new();

    for (ent, str_id, portal_template, &global_pos) in new_portals.iter() {

        let studied_op = portal_template.to_studied_op(global_pos);
        let pos_search = PosSearch::portal_pos_search(studied_op.clone());
        cmd.entity(ent).insert(SearchingForSuitablePos);
        ew_pending_ops.write(pos_search);
        started_searches.insert(studied_op, ent);
    }


    let mut successful_searches: HashSet<StudiedOp> = HashSet::new();

    'successful_searches: for search_successful_ev in ereader_search_successful.read() {
        let studied_op = search_successful_ev.studied_op.clone();
        if successful_searches.contains(&studied_op) {
            continue 'successful_searches;
        }

        if let Some(ent) = started_searches.remove(&studied_op) {
            cmd.entity(ent).remove::<(SearchingForSuitablePos, PortalTemplate)>();
            successful_searches.insert(studied_op);
            continue 'successful_searches;
        }

        for (ent, str_id, portal_template, &global_pos) in pending_search.iter() {
            if studied_op == portal_template.to_studied_op(global_pos) {
                info!("Found suitable pos for portal tile {}", str_id);
                successful_searches.insert(studied_op);
                cmd.entity(ent).remove::<(SearchingForSuitablePos, PortalTemplate)>();
                continue 'successful_searches;
            }
        }
    }

    for ev in ereader_search_failed.read() {
        if successful_searches.contains(&ev.0) {
            continue;
        }
        for (ent, str_id, portal_template, &global_pos) in pending_search.iter() {

            if ev.0 == portal_template.to_studied_op(global_pos) {
                error!("Failed to find suitable pos for portal tile {}", str_id);
                cmd.entity(ent).remove::<SearchingForSuitablePos>();

            }
        }
    }
}

