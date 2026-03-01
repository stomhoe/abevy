#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::EntityZero;
use game_common::game_common_string_components::Description;
use modifier::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierTarget};
use modifier::modifier_item_types::*;
use ::item_shared::*;

fn spawn_item_modifier<T: Component + Default>(cmd: &mut Commands, item: Entity, value: f32) {
    if value == 0.0 {
        return;
    }
    cmd.spawn((
        ModifierTarget(item),
        BaseValue(value),
        CurrEffectiveValue(value),
        ApplyMode::Add,
        T::default(),
        ChildOf(item),
    ));
}

#[allow(unused_parens)]
pub fn init_items(mut cmd: Commands, item_map: Res<ItemEntityMap>, holders: Query<Entity, With<EguiItemsHolder>>) {
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
        let mut entity_cmd = cmd.entity(item_ent);
        entity_cmd.insert((
            Item {
                equip_sprite_cfg_ids: seri
                    .equip_sprite_cfg_ids
                    .iter()
                    .filter_map(|id| {
                        let trimmed = id.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(StrId::trunc(trimmed))
                        }
                    })
                    .collect(),
                dropped_sprite_cfg_id: StrId::trunc(seri.dropped_sprite_cfg_id.trim()),
                icon_sprite_cfg_id: StrId::trunc(seri.icon_sprite_cfg_id.trim()),
                icon_img_path: seri.icon_img_path.trim().to_string(),
            },
            str_id.clone(),
            Prefix::trunc("Item"),
            AddHashIdFromStrId,
            EntityZero,
            ChildOf(holder),
        ));
        if !seri.name.trim().is_empty() {
            entity_cmd.insert(DisplayName::new(seri.name.trim()));
        }
        if !seri.description.trim().is_empty() {
            entity_cmd.insert(Description(seri.description.trim().to_string()));
        }
        if !seri.icon_img_path.trim().is_empty() {
            if let Ok(icon_path) = ImagePathHolder::new(seri.icon_img_path.trim().to_string()) {
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

        spawn_item_modifier::<MassKg>(&mut cmd, item_ent, seri.mass.max(0.0));
        spawn_item_modifier::<Encumberance>(&mut cmd, item_ent, seri.encumberance.max(0.0));
        spawn_item_modifier::<Bulk>(&mut cmd, item_ent, seri.bulk.max(0.0));
        spawn_item_modifier::<Durability>(&mut cmd, item_ent, seri.durability.max(0.0));
        spawn_item_modifier::<MaxDurability>(&mut cmd, item_ent, seri.max_durability.max(0.0));
        spawn_item_modifier::<MarketValue>(&mut cmd, item_ent, seri.market_value.max(0.0));
        spawn_item_modifier::<Warmth>(&mut cmd, item_ent, seri.warmth);
        spawn_item_modifier::<ArmorBlunt>(&mut cmd, item_ent, seri.armor_blunt);
        spawn_item_modifier::<ArmorSharp>(&mut cmd, item_ent, seri.armor_sharp);
        spawn_item_modifier::<ArmorFire>(&mut cmd, item_ent, seri.armor_fire);
        spawn_item_modifier::<StackLimit>(&mut cmd, item_ent, seri.stack_limit.max(1) as f32);
    }
}
