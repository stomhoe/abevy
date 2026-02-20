#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_replicon::prelude::*;
use common::{common_components::*, common_tag_components::TagSet};
use game_common::game_common_components::{EntityZero, EntityZeroRef};

use crate::body::{
    body_tree_components::*, body_part::body_part_components::*, body_part::body_part_resources::*,
    body_tree_resources::*,
};

#[allow(unused_parens)]
pub fn init_ezero_body_trees(
    mut cmd: Commands,
    body_map: Res<BodyTreeEntityMap>,
    part_map: Res<BodyPartEntityMap>,
) {
    if !body_map.0.is_empty() {
        return;
    }

    for mut seri in load_body_tree_seri_defs() {

        let body_id = match StrId::new_with_result(seri.id, 3) {
            Ok(id) => id,
            Err(e) => {
                let err = BevyError::from(format!("Failed to create StrId for BodyConfig: {}", e));
                error!(target: "body_init", "{}", err);
                continue;
            }
        };

        let body_tree_ent = cmd.spawn_empty().id();
        cmd.entity(body_tree_ent).insert((
            body_id.clone(),
            BodyTree,
            BodyTreeMassKg(seri.mass_kg.max(0.0)),
            BodyTreeDistributedTotals {
                hp_capacity: seri.hp_capacity.max(0.0),
                hp_regen_rate: seri.hp_regen_rate.max(0.0),
                blood_capacity: seri.blood_capacity.max(0.0),
                blood_pumping: seri.blood_pumping.max(0.0),
                walk_speed: seri.walk_speed.max(0.0),
                swim_speed: seri.swim_speed.max(0.0),
                fly_speed: seri.fly_speed.max(0.0),
                manipulation: seri.manipulation.max(0.0),
                vision: seri.vision.max(0.0),
                pain_sensitivity: seri.pain_sensitivity.max(0.0),
                caloric_burn_rate: seri.caloric_burn_rate.max(0.0),
                caloric_capacity: seri.caloric_capacity.max(0.0),
            },
            EntityZero,
        ));

        if seri.name.trim().is_empty() {
            cmd.entity(body_tree_ent)
                .insert(DisplayName::trunc(body_id.as_str()));
        } else {
            cmd.entity(body_tree_ent)
                .insert(DisplayName::trunc(seri.name));
        }

        if !seri.tags.is_empty() {
            cmd.entity(body_tree_ent).insert(TagSet::new(&seri.tags));
        }

        let root_node = std::mem::take(&mut seri.root);
        let root_id = StrId::trunc(root_node.part_id.as_str());

        let root_ent = walk_body_tree(
            &mut cmd,
            &part_map,
            body_tree_ent,
            &body_id,
            root_node,
            None,
        );

        if let Some(root_ent) = root_ent {
            cmd.entity(root_ent).insert(BodyRootPart);
        } else {
            warn!(target: "body_init", "BodyConfig '{}' root part '{}' not found", body_id, root_id);
        }
    }
}

fn walk_body_tree(
    cmd: &mut Commands,
    part_map: &Res<BodyPartEntityMap>,
    body_ent: Entity,
    body_id: &StrId,
    node: BodyNodeSeri,
    parent_ent: Option<Entity>,
) -> Option<Entity> {
    let part_id = StrId::trunc(node.part_id.as_str());
    let Ok(part_ent) = part_map.0.get_cloned(&part_id) else {
        warn!(target: "body_init", "BodyPart '{}' not found in BodyPartCfgEntityMap for body '{}', skipping", part_id, body_id);
        return None;
    };

    cmd.entity(part_ent).insert((
        BodyPartOf { body: body_ent },
        ChildOf(body_ent),
        EntityZeroRef(part_ent),
    ));

    let label = node.label_override.trim();
    if !label.is_empty() {
        cmd.entity(part_ent).insert(DisplayName::trunc(label));
    }

    if let Some(parent_ent) = parent_ent {
        cmd.entity(part_ent)
            .insert(BodyPartParent { parent: parent_ent });
    }

    for child in node.children {
        walk_body_tree(cmd, part_map, body_ent, body_id, child, Some(part_ent));
    }

    Some(part_ent)
}
