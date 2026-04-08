use ::tilemap_shared::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use common::common_components::HashId;
use ::being_shared::*;

#[derive(Resource, Debug, Default)]
pub struct PortalCrossingIndex {
    portal_locations: EntityHashMap<(HashId, GlobalTilePos)>,
    portals_by_dimension: HashMap<HashId, Vec<Entity>>,
    portals_by_tile: HashMap<(HashId, GlobalTilePos), Vec<Entity>>,
}

impl PortalCrossingIndex {
    fn remove_from_bucket(bucket: &mut Vec<Entity>, portal_ent: Entity) {
        if let Some(index) = bucket.iter().position(|&ent| ent == portal_ent) {
            bucket.swap_remove(index);
        }
    }

    fn remove_portal(&mut self, portal_ent: Entity) {
        let Some((dimension, tile_pos)) = self.portal_locations.remove(&portal_ent) else {
            return;
        };
        let mut drop_dimension_bucket = false;
        if let Some(bucket) = self.portals_by_dimension.get_mut(&dimension) {
            Self::remove_from_bucket(bucket, portal_ent);
            drop_dimension_bucket = bucket.is_empty();
        }
        if drop_dimension_bucket {
            self.portals_by_dimension.remove(&dimension);
        }
        let mut drop_tile_bucket = false;
        if let Some(bucket) = self.portals_by_tile.get_mut(&(dimension, tile_pos)) {
            Self::remove_from_bucket(bucket, portal_ent);
            drop_tile_bucket = bucket.is_empty();
        }
        if drop_tile_bucket {
            self.portals_by_tile.remove(&(dimension, tile_pos));
        }
    }

    fn insert_portal(&mut self, portal_ent: Entity, dimension: HashId, gpos: GlobalTilePos) {
        self.portal_locations.insert(portal_ent, (dimension, gpos));
        self.portals_by_dimension.entry(dimension).or_default().push(portal_ent);
        self.portals_by_tile.entry((dimension, gpos)).or_default().push(portal_ent);
    }

    pub fn update_portal(&mut self, portal_ent: Entity, dimension: HashId, gpos: GlobalTilePos) {
        self.remove_portal(portal_ent);
        self.insert_portal(portal_ent, dimension, gpos);
    }

    pub fn portals_at_tile(&self, dimension: HashId, gpos: GlobalTilePos) -> Option<&[Entity]> {
        self.portals_by_tile.get(&(dimension, gpos)).map(Vec::as_slice)
    }

    pub fn portals_in_dimension(&self, dimension: HashId) -> Option<&[Entity]> {
        self.portals_by_dimension.get(&dimension).map(Vec::as_slice)
    }
}

#[allow(unused_parens, )]
pub fn rebuild_portal_crossing_index(
    mut index: ResMut<PortalCrossingIndex>,
    portals_query: Query<
        (Entity, &DimensionRef, &GlobalTilePos),
        (
            With<PortalTo>,
            Or<(Changed<PortalTo>, Changed<DimensionRef>, Changed<GlobalTilePos>)>,
            Without<Being>,
        ),
    >,
    mut removed_portals: RemovedComponents<PortalTo>,
) {
    for portal_ent in removed_portals.read() {
        index.remove_portal(portal_ent);
    }
    for (portal_ent, &dimension_ref, &gpos) in portals_query.iter() {
        index.update_portal(portal_ent, dimension_ref.0, gpos);
    }
}
