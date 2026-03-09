use bevy::prelude::*;
use being::body::{Bodies, BodyDamage};
use game_common::game_common_components::{
    Dead,
    DespawnOnDeath,
    EntityZero,
    EntityZeroRef,
    Health,
    HealthDamage,
};
use item_shared::{Item, ItemsDroppedOnDeath, ToDenyOnItemClone};
use sprite::prelude::ScsToBuild;
use tilemap::prelude::tile_components::Tile;
use tilemap_shared::{DimensionRef, GlobalTilePos};
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

pub fn spawn_items_on_death(
    mut cmd: Commands,
    query: Query<
        (
            Option<&DimensionRef>,
            Option<&GlobalTransform>,
            Option<&GlobalTilePos>,
            &ItemsDroppedOnDeath,
        ),
        (Without<EntityZero>, With<Dead>, With<DespawnOnDeath>, Changed<Dead>),
    >,
    item_cfg_query: Query<&item_shared::ItemSpritesConfig, (With<Item>, With<EntityZero>)>,
) {
    let mut rng = rand::rng();
    for (dim_ref, global_transform, tile_pos, dropped_on_death) in query.iter() {
        let Some(item_counts) = dropped_on_death.0.sample_with_rng(&mut rng) else {
            continue;
        };
        let drop_pos = global_transform
            .map(|transform| transform.translation().xy())
            .or_else(|| tile_pos.map(GlobalTilePos::to_pixelpos))
            .unwrap_or_default();
        let drop_z = global_transform
            .map(|transform| transform.translation().z)
            .unwrap_or_default();

        for (item_ezero, base_count) in item_counts {
            let dropped_sprite_cfg = item_cfg_query
                .get(item_ezero)
                .ok()
                .map(|cfg| cfg.dropped_sprite_cfg.0)
                .filter(|&cfg_ent| cfg_ent != Entity::PLACEHOLDER);
            let scaled_count = (base_count as f32 * dropped_on_death.1).max(0.0);
            let drop_count = scaled_count.round() as u32;
            for _ in 0..drop_count {
                let item_instance = cmd
                    .entity(item_ezero)
                    .clone_and_spawn_with_opt_out(|builder| {
                        builder.deny::<ToDenyOnItemClone>();
                    })
                    .id();
                let mut item_cmd = cmd.entity(item_instance);
                item_cmd.insert((
                    Item,
                    EntityZeroRef(item_ezero),
                    Transform::from_translation(drop_pos.extend(drop_z)),
                    GlobalTransform::default(),
                ));
                if let Some(cfg_ent) = dropped_sprite_cfg {
                    let mut scs_to_build = ScsToBuild::with_capacity(1);
                    scs_to_build.0.insert(cfg_ent);
                    item_cmd.insert(scs_to_build);
                }
                let Some(&dim_ref) = dim_ref else { continue };
                item_cmd.insert((dim_ref, ChildOf(dim_ref.0)));
            }
        }
    }
}
