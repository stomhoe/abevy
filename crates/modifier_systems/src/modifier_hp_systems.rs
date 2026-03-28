use bevy::prelude::*;
use game_common::game_common_components::{Dead, DespawnOnDeath, Templ};
use tilemap::tile::tile_components::Tile;
use tilemap_shared::SafeDespawn;

pub fn despawn_entities_on_death(
    query: Query<
        (Entity, Has<Tile>),
        (
            Without<Templ>,
            With<DespawnOnDeath>,
            Changed<Dead>,
        ),
    >,
    mut writer: MessageWriter<SafeDespawn>,
    mut messages: Local<Vec<SafeDespawn>>,
) {
    for (entity, is_tile) in query.iter() {
        if is_tile {
            messages.push(SafeDespawn(entity));
        }
    }
    writer.write_batch(messages.drain(..));
}
