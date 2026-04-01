#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::Templ;
use game_common::game_common_string_components::Description;
use modifier_shared::modifier_item_types::*;
use modifier_shared::modifier_helpers::spawn_modifier;
use ::sprite_shared::ScRef;
use sprite_systems::SpriteConfigEntityMap;
use ::item_shared::*;
use std::collections::HashMap;
use common::log_targets::ITEM_SYSTEM;

#[allow(unused_parens)]
pub fn init_items(
    mut cmd: Commands,
    item_map: Res<ItemEntityMap>,
    holders: Query<Entity, With<EguiItemsHolder>>,
    sprite_cfg_map: Option<Res<SpriteConfigEntityMap>>,
) {
    if !item_map.0.is_empty() {
        return;
    }

    let holder = if let Ok(holder) = holders.single() {
        holder
    } else {
        cmd.spawn((EguiItemsHolder,)).id()
    };

    for seri in load_item_seri_defs() {
        let Ok(str_id) = StrId::new_with_result(seri.id.clone(), Item::MIN_ID_LENGTH) else {
            continue;
        };

        let item_ent = cmd.spawn_empty().id();
        let resolve_sprite_cfg_ref = |id: &str, label: &str| -> ScRef {
            let Some(sprite_cfg_map) = sprite_cfg_map.as_ref() else {
                error!(
                    target: ITEM_SYSTEM,
                    "Item '{}' could not resolve {} sprite cfg '{}': SpriteConfigEntityMap missing",
                    str_id,
                    label,
                    id.trim(),
                );
                return ScRef(Entity::PLACEHOLDER);
            };
            let trimmed = id.trim();
            if trimmed.is_empty() {
                return ScRef(Entity::PLACEHOLDER);
            }
            let Ok(ent) = sprite_cfg_map.0.get_cloned(StrId::trunc(trimmed)) else {
                error!(
                    target: ITEM_SYSTEM,
                    "Item '{}' could not resolve {} sprite cfg '{}'",
                    str_id,
                    label,
                    trimmed,
                );
                return ScRef(Entity::PLACEHOLDER);
            };
            ScRef(ent)
        };
        let sprite_cfg_per_state: HashMap<StrId, ScRef> = seri
            .equip_sprite_cfg_ids
            .iter()
            .filter_map(|id| {
                let trimmed = id.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    let sc_ref = resolve_sprite_cfg_ref(trimmed, "equip");
                    Some((StrId::trunc(trimmed), sc_ref))
                }
            })
            .collect();
        let dropped_sprite_cfg = resolve_sprite_cfg_ref(seri.dropped_sprite_cfg_id.trim(), "dropped");
        let icon_sprite_cfg = resolve_sprite_cfg_ref(seri.icon_sprite_cfg_id.trim(), "icon");
        if dropped_sprite_cfg.0 == Entity::PLACEHOLDER && icon_sprite_cfg.0 == Entity::PLACEHOLDER {
            error!(
                target: ITEM_SYSTEM,
                "Item '{}' has neither dropped nor icon sprite cfg resolved; ground instances will be invisible",
                str_id,
            );
        }
        let mut entity_cmd = cmd.entity(item_ent);
        entity_cmd.insert((
            Item,
            ItemSpritesConfig {
                sprite_cfg_per_state,
                dropped_sprite_cfg,
                icon_sprite_cfg,
                icon_img_path: seri.icon_img_path.trim().to_string(),
            },
            str_id.clone(),
            Prefix::trunc("Item"),
            AddHashIdFromStrId,
            Templ,
            ChildOf(holder),
        ));
        if !seri.name.trim().is_empty() {
            entity_cmd.insert(DisplayName::new(seri.name.trim()));
        }
        if !seri.description.trim().is_empty() {
            entity_cmd.insert(Description(seri.description.trim().to_string()));
        }
        if !seri.icon_img_path.trim().is_empty() {
            if let Ok(icon_path) = PathHolder::new(seri.icon_img_path.trim().to_string()) {
                entity_cmd.insert(icon_path);
            }
        }

        let mut tags = TagSet::default();
        tags.insert(Tag::trunc(str_id.as_str()));
        for tag in &seri.tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                continue;
            }
            tags.insert(Tag::trunc(trimmed));
        }
        entity_cmd.insert(tags);

        spawn_modifier::<MassKg>(&mut cmd, item_ent, seri.mass.max(0.0));
        spawn_modifier::<Encumberance>(&mut cmd, item_ent, seri.encumberance.max(0.0));
        spawn_modifier::<Bulk>(&mut cmd, item_ent, seri.bulk.max(0.0));
        spawn_modifier::<Durability>(&mut cmd, item_ent, seri.durability.max(0.0));
        spawn_modifier::<MaxDurability>(&mut cmd, item_ent, seri.max_durability.max(0.0));
        spawn_modifier::<MarketValue>(&mut cmd, item_ent, seri.market_value.max(0.0));
        spawn_modifier::<Warmth>(&mut cmd, item_ent, seri.warmth);
        spawn_modifier::<ArmorBlunt>(&mut cmd, item_ent, seri.armor_blunt);
        spawn_modifier::<ArmorSharp>(&mut cmd, item_ent, seri.armor_sharp);
        spawn_modifier::<ArmorFire>(&mut cmd, item_ent, seri.armor_fire);
        spawn_modifier::<StackLimit>(&mut cmd, item_ent, seri.stack_limit.max(1) as f32);
    }
}
