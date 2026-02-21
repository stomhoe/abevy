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
        let mut totals = HashIdMap::default();
        for (key, val) in &seri.distributed_totals {
            totals.overwrite(HashId::from(key), val.max(0.0));
        }
        if !totals.contains_key(STAT_HP_CAPACITY) {
            totals.overwrite(STAT_HP_CAPACITY, 1.0);
        }
        if !totals.contains_key(STAT_HP_REGEN_RATE) {
            totals.overwrite(STAT_HP_REGEN_RATE, 1.0);
        }
        if !totals.contains_key(STAT_BLOOD_CAPACITY) {
            totals.overwrite(STAT_BLOOD_CAPACITY, 1.0);
        }
        if !totals.contains_key(STAT_VISION) {
            totals.overwrite(STAT_VISION, 1.0);
        }
        if !totals.contains_key(STAT_CALORIC_BURN_RATE) {
            totals.overwrite(STAT_CALORIC_BURN_RATE, 1.0);
        }
        if !totals.contains_key(STAT_WALK_SPEED) {
            totals.overwrite(STAT_WALK_SPEED, 300.);
        }
        cmd.entity(body_tree_ent).insert((
            body_id.clone(),
            BodyTree,
            BodyTreeMassKg(seri.mass_kg.max(0.0)),
            BodyTreeDistributedTotals(totals),
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
    let Ok(source_part_ent) = part_map.0.get_cloned(&part_id) else {
        warn!(target: "body_init", "BodyPart '{}' not found in BodyPartCfgEntityMap for body '{}', skipping", part_id, body_id);
        return None;
    };

    let tree_node_ent = cmd.spawn_empty().id();
    cmd.entity(tree_node_ent).insert((
        BodyPartOf { body: body_ent },
        ChildOf(body_ent),
        EntityZeroRef(source_part_ent),
        EntityZero,
    ));

    let label = node.label_override.trim();
    if !label.is_empty() {
        cmd.entity(tree_node_ent).insert(DisplayName::trunc(label));
    }

    if let Some(parent_ent) = parent_ent {
        cmd.entity(tree_node_ent)
            .insert(BodyPartParent { parent: parent_ent });
    }

    for child in node.children {
        walk_body_tree(cmd, part_map, body_ent, body_id, child, Some(tree_node_ent));
    }

    Some(tree_node_ent)
}
