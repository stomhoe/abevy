use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::ecs::{entity::MapEntities, entity_disabling::Disabled};
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileFlip;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
#[allow(unused_imports)] use bevy_asset_loader::prelude::*;
use common::common_components::{HashId, StrId};
use game_common::game_common_components::MyZ;
use player::player_components::{HostPlayer, OfSelf, Player};
use crate::{terrain_gen::terrgen_resources::AaGlobalGenSettings, tile::{tile_components::*, tile_resources::*}};




#[allow(unused_parens)]
pub fn flip_tile_along_x(
    settings: Res<AaGlobalGenSettings>,
    mut query: Query<AnyOf<(&mut TileFlip, &mut Sprite)>, (Added<InitialPos>, With<FlipAlongX>)>
) {
    for (mut flip, mut sprite) in query.iter_mut() {

    }
}

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
