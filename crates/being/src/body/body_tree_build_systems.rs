use bevy::prelude::*;
use common::common_components::*;
use game_common::game_common_components::{Templ, TemplEntiRef};
use common::log_targets::BODY_BUILD;
use modifier_shared::modifier_components::{AppliedModifiers, ModifierSynergies};

use crate::body::BodyTreeRef;
use crate::body::{body_tree_components::*};
use ::being_shared::*;

#[allow(unused_parens, )]
pub fn build_body_trees_on_beings(
    mut cmd: Commands,
    consumer_beings_query: Query<
        (Entity, &BodyTreeRef, ),
        (With<Being>, Added<BodyTreeRef>, Without<Templ>, Without<Race>, Without<BeingInstTemplate>),
    >,
    templ_tree_bodyparts_query: Query<(&BodypartChildrenBodyparts, ), (With<Templ>,)>,
    root_bodypart_query: Query<(), (With<TreeRoot>, )>,
    toclone_query: Query<(&BodypartChildrenBodyparts, ), (With<Templ>, )>,
    display_name_query: Query<(&DisplayName, Has<Templ>),>,
) {
    for (being_ent, tree_to_build, ) in consumer_beings_query.iter() {
        trace!(target: BODY_BUILD, "Building body tree {} for being {} from source templ {}", entity_dbg(tree_to_build.0, &display_name_query), entity_dbg(being_ent, &display_name_query), entity_dbg(tree_to_build.0, &display_name_query));
        let body_ent = cmd.spawn((
            BodyOf { being: being_ent },
            ChildOf(being_ent),
            TemplEntiRef(tree_to_build.0),
            BodySums::default(),
        )).id();

        let Ok((templ_bodyparts, )) = templ_tree_bodyparts_query.get(tree_to_build.0) else {
            error!(target: BODY_BUILD, "BodyTree {:?} has no BodyRootPart; skipping source clone for owner {:?}", tree_to_build.0, being_ent);
            continue;
        };
        let root_templ_bodypart = templ_bodyparts
            .iter()
            .find(|&templ_bodypart| root_bodypart_query.get(templ_bodypart).is_ok());
        let Some(root_templ_bodypart) = root_templ_bodypart else {
            error!(target: BODY_BUILD, "BodyTree {} has no valid BodyRootPart; skipping source clone for owner {}", entity_dbg(tree_to_build.0, &display_name_query), entity_dbg(being_ent, &display_name_query));
            continue;
        };
        trace!(target: BODY_BUILD, "Selected root source body part {} for being {} from body tree {}", entity_dbg(root_templ_bodypart, &display_name_query), entity_dbg(being_ent, &display_name_query), entity_dbg(tree_to_build.0, &display_name_query));

        let Some(new_root_ent) = walk_and_clone_tree(
            &mut cmd,
            root_templ_bodypart,
            &toclone_query,
            None,
            body_ent,
            &display_name_query,
        ) else {
            continue;
        };
        trace!(target: BODY_BUILD, "Root clone {} finished for being {}; attached to body {}", entity_dbg(new_root_ent, &display_name_query), entity_dbg(being_ent, &display_name_query), entity_dbg(body_ent, &display_name_query));
    }
}

fn walk_and_clone_tree(
    cmd: &mut Commands,
    templtree_curr_node_ent: Entity,
    ref_of_bpart_toclone_query: &Query<(&BodypartChildrenBodyparts, ), (With<Templ>, )>,
    parent_node: Option<Entity>,
    body_ent: Entity,
    display_name_query: &Query<(&DisplayName, Has<Templ>),>,
) -> Option<Entity> {
    let parent_bodypart = parent_node.unwrap_or(body_ent);
    let cloned_bodypart_ent = cmd
        .entity(templtree_curr_node_ent)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<(
                Templ, Children, AppliedModifiers, ModifierSynergies, BodypartForcedStats, BodypartWeightedDistribution, ChildOf, BodypartChildOfBodypart, BodypartChildrenBodyparts
            )>();
        })
        .id();
    cmd.entity(cloned_bodypart_ent).insert((
        BodypartChildOfBodypart { parent_bodypart },
        ChildOf(body_ent),
        TemplEntiRef(templtree_curr_node_ent),
        Name::default(),
    ));
    trace!(target: BODY_BUILD, "Created clone {} from source {} for body {}", entity_dbg(cloned_bodypart_ent, display_name_query), entity_dbg(templtree_curr_node_ent, display_name_query), entity_dbg(body_ent, display_name_query));
    trace!(target: BODY_BUILD, "Assigned BodypartChildOf parent {} and ChildOf body {} to clone {}", entity_dbg(parent_bodypart, display_name_query), entity_dbg(body_ent, display_name_query), entity_dbg(cloned_bodypart_ent, display_name_query));

    if let Ok((bodypart_children, )) = ref_of_bpart_toclone_query.get(templtree_curr_node_ent) {
        trace!(target: BODY_BUILD, "Clone {} has {} bodypart-child nodes", entity_dbg(cloned_bodypart_ent, display_name_query), bodypart_children.iter().count());
        for templ_child_bodypart_ent in bodypart_children.iter() {
            trace!(target: BODY_BUILD, "Descending from clone {} into source child {}", entity_dbg(cloned_bodypart_ent, display_name_query), entity_dbg(templ_child_bodypart_ent, display_name_query));
            walk_and_clone_tree(
                cmd,
                templ_child_bodypart_ent,
                ref_of_bpart_toclone_query,
                Some(cloned_bodypart_ent),
                body_ent,
                display_name_query,
            );
        }
    };

    Some(cloned_bodypart_ent)
}

fn entity_dbg(
    entity: Entity,
    display_name_query: &Query<(&DisplayName, Has<Templ>),>,
) -> String {
    let Ok((display_name, is_templ)) = display_name_query.get(entity) else {
        return format!("{:?}", entity);
    };
    format!("{} ({:?}, templ={})", display_name, entity, is_templ)
}
