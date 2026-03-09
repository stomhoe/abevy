use bevy::prelude::*;
use being::body::{Bodies, BodyDamage};
use game_common::game_common_components::{
    Dead,
    DespawnOnDeath,
    EntityZero,
    Health,
    HealthDamage,
};
use tilemap::prelude::tile_components::Tile;
use tilemap_shared::SafeDespawn;

pub fn apply_health_damage(
    mut reader: MessageReader<HealthDamage>,
    mut health_query: Query<&mut Health, Without<EntityZero>>,
    body_query: Query<&Bodies, Without<EntityZero>>,
    mut body_damage_writer: MessageWriter<BodyDamage>,
    mut body_damage_messages: Local<Vec<BodyDamage>>,
) {
    for msg in reader.read() {
        if msg.amount <= 0.0 {
            continue;
        }
        let mut sent_to_direct_health = false;
        if let Ok(mut health) = health_query.get_mut(msg.entity) {
            if health.0 > 0.0 {
                health.0 = (health.0 - msg.amount).max(0.0);
                sent_to_direct_health = true;
            }
        }
        if sent_to_direct_health {
            continue;
        }
        let Ok(target_bodies) = body_query.get(msg.entity) else {
            continue;
        };
        let Some(&body) = target_bodies.entities().first() else {
            continue;
        };
        body_damage_messages.push(BodyDamage {
            body,
            amount: msg.amount,
        });
    }
    body_damage_writer.write_batch(body_damage_messages.drain(..));
}

pub fn mark_dead_by_health(
    mut cmd: Commands,
    query: Query<(Entity, &Health, Has<Dead>), (Without<EntityZero>, Changed<Health>)>,
) {
    for (entity, health, is_dead) in query.iter() {
        if health.0 <= 0.0 {
            if !is_dead {
                cmd.entity(entity).try_insert(Dead);
            }
            continue;
        }
        if is_dead {
            cmd.entity(entity).try_remove::<Dead>();
        }
    }
}

pub fn despawn_entities_on_death(
    query: Query<
        (Entity, Has<Tile>),
        (
            Without<EntityZero>,
            With<DespawnOnDeath>,
            Changed<Dead>,
        ),
    >,
    mut writer: MessageWriter<SafeDespawn>,
    mut messages: Local<Vec<SafeDespawn>>,
) {
    for (entity, _is_tile) in query.iter() {
        messages.push(SafeDespawn(entity));
    }
    writer.write_batch(messages.drain(..));
}
