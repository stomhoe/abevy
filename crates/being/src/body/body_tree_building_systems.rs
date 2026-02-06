#[allow(unused_imports)]
use bevy::prelude::*;
#[allow(unused_imports)]
use game_common::game_common_components::{EntityZero, EntityZeroRef};
use modifier::modifier_components::ModifierTarget;

use crate::being_components::*;
use crate::body::{body_components::*, body_part::body_part_components::*};
pub fn build_body_tree(
    mut cmd: Commands,
    query: Query<
        (Entity, &BodyTreeToBuild),
        (With<Being>, Added<BodyTreeToBuild>, Without<EntityZero>),
    >,
    toclone_query: Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    children_query: Query<&Children>,
    modifier_target_query: Query<&ModifierTarget>,
) {
    for (being_ent, tree_to_build) in query.iter() {
        if let Some(new_root_ent) = walk_and_clone_tree(
            &mut cmd,
            tree_to_build.0,
            &toclone_query,
            &children_query,
            None,
            &modifier_target_query,
            being_ent,
        ) {
            cmd.entity(new_root_ent)
                .try_insert(BodyPartOf { body: being_ent });
        }

        cmd.entity(being_ent).remove::<BodyTreeToBuild>();
    }
}

fn walk_and_clone_tree(
    cmd: &mut Commands,
    ezerotree_curr_node_ent: Entity,
    ref_of_bpart_toclone_query: &Query<(&EntityZeroRef, &BodyPartOf, Option<&BodyPartChildren>)>,
    bodypart_modifiers_query: &Query<&Children>,
    parent_cloned_ent: Option<Entity>,
    modifier_target_query: &Query<&ModifierTarget>,
    being_ent: Entity,
) -> Option<Entity> {
    let Ok((ezero_ref, ezero_body_part_of, bodypart_children)) =
        ref_of_bpart_toclone_query.get(ezerotree_curr_node_ent)
    else {
        return None;
    };
    let bodypart_2b_cloned_ent = ezero_ref.0;

    let cloned_bodypart_ent = cmd
        .entity(bodypart_2b_cloned_ent)
        .clone_and_spawn_with_opt_out(|builder| {
            builder.deny::<EntityZero>();
        })
        .id();

    if let Ok(children) = bodypart_modifiers_query.get(bodypart_2b_cloned_ent) {
        for modifier_ent in children.iter() {
            cmd.entity(modifier_ent)
                .clone_and_spawn_with_opt_out(|builder| {
                    builder.deny::<(EntityZero, ModifierTarget)>();
                })
                .try_insert(ChildOf(cloned_bodypart_ent));
            if let Ok(modifier_target) = modifier_target_query.get(modifier_ent) {
                let target = if modifier_target.0 == bodypart_2b_cloned_ent {
                    cloned_bodypart_ent
                } else {
                    being_ent
                };

                cmd.entity(modifier_ent).try_insert(ModifierTarget(target));
            }
        }
    }

    if let Some(parent_cloned) = parent_cloned_ent {
        cmd.entity(cloned_bodypart_ent).insert((
            BodyPartParent {
                parent: parent_cloned,
            },
            ChildOf(parent_cloned),
        ));
    }
    if let Some(bodypart_children) = bodypart_children {
        for ezero_child_bodypart_ent in bodypart_children.iter() {
            walk_and_clone_tree(
                cmd,
                ezero_child_bodypart_ent,
                ref_of_bpart_toclone_query,
                &bodypart_modifiers_query,
                Some(cloned_bodypart_ent),
                &modifier_target_query,
                being_ent,
            );
        }
    }

    Some(cloned_bodypart_ent)
}
