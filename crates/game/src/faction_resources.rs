use bevy::prelude::*;
use common::{common_components::{DisplayName, EntityPrefix, StrId}, common_types::HashIdToEntityMap};

use crate::{faction_components::Faction, player::OfSelf};

#[derive(Resource, Reflect, )]
pub struct FactionEntityMap (pub HashIdToEntityMap);
impl FromWorld for FactionEntityMap {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashIdToEntityMap::default();
        let str_id = StrId::new("host").expect("Failed to create StrId for host faction");
        let host_faction = (Faction, str_id.clone(), DisplayName::new("Host Faction"), OfSelf);
        let entity = world.spawn(host_faction).id();
        map.insert(str_id, entity, )
        .expect("Failed to insert host faction into FactionEntityMap");
        FactionEntityMap(map)
    }
}
