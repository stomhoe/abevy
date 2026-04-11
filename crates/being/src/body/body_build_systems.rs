use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use common::common_components::HashId;
use common::common_components::*;
use game_common::game_common_components::{Templ, TemplHashIdRef, TemplEntiRef};
use common::log_targets::BODY_BUILD;
use modifier_shared::modifier_components::{ApplyMode, BaseValue, CurrEffectiveValue, ModifierTarget};
use modifier_shared::modifier_types::BloodCapacity;
use modifier_shared::modifier_components::{AppliedModifiers, ModifierSynergies};

use crate::body::BodyRef;
use crate::body::body_resources::BodyEntityMap;
use crate::body::body_templ_init_systems::distribute_budgets_among_bodyparts_based_on_weights_and_forcings;
use crate::body::bodytree::{BodyTreeRef, BodyTreeTemplateEntityMap};
use crate::body::body_components::*;

#[derive(SystemParam)]
pub struct BuildBodysOnBeingsQueries<'w, 's> {
    consumer_beings_query: Query<'w, 's, (Entity, &'static BodyRef), (With<Being>, Added<BodyRef>, Without<Templ>, Without<Race>, Without<BeingInstTemplate>)>,
    body_map: Res<'w, BodyEntityMap>,
    bodytree_map: Res<'w, BodyTreeTemplateEntityMap>,
    body_hash_query: Query<'w, 's, &'static HashId, With<Body>>,
    bodypart_hash_query: Query<'w, 's, &'static HashId, With<Bodypart>>,
    templ_tree_bodyparts_query: Query<'w, 's, (&'static BodypartChildrenBodyparts,), (With<Templ>,)>,
    root_bodypart_query: Query<'w, 's, (), (With<TreeRoot>,)>,
    toclone_query: Query<'w, 's, (&'static BodypartChildrenBodyparts,), (With<Templ>,)>,
    body_totals_query: Query<'w, 's, (&'static StrId, &'static StatBudgetsToDistributeAmongBodyPartsOfTemplBody), (With<Body>, With<Templ>,)>,
    bodytree_ref_query: Query<'w, 's, &'static BodyTreeRef, (With<Body>, With<Templ>,)>,
    forced_query: Query<'w, 's, &'static BodypartForcedStats>,
    weighted_query: Query<'w, 's, &'static BodypartWeightedDistribution>,
    synergy_query: Query<'w, 's, &'static ModifierSynergies>,
    display_name_query: Query<'w, 's, (&'static DisplayName, Has<Templ>)>,
}

#[derive(SystemParam)]
pub struct BuildBodysOnBeingsLocals<'s> {
    cloned_parts_to_source: Local<'s, Vec<(Entity, Entity)>>,
}

#[allow(unused_parens, )]
pub fn build_bodys_on_beings(
    mut cmd: Commands,
    queries: BuildBodysOnBeingsQueries,
    mut locals: BuildBodysOnBeingsLocals,
) {
    let BuildBodysOnBeingsQueries {
        consumer_beings_query,
        body_map,
        bodytree_map,
        body_hash_query,
        bodypart_hash_query,
        templ_tree_bodyparts_query,
        root_bodypart_query,
        toclone_query,
        body_totals_query,
        bodytree_ref_query,
        forced_query,
        weighted_query,
        synergy_query,
        display_name_query,
    } = queries;
    let BuildBodysOnBeingsLocals {
        cloned_parts_to_source,
    } = &mut locals;
    for (being_ent, tree_to_build, ) in consumer_beings_query.iter() {
        let Ok(body_templ_ent) = body_map.0.get_cloned(tree_to_build.0) else {
            error!(target: BODY_BUILD, "Body template hash {:?} could not be resolved", tree_to_build.0);
            continue;
        };
        let Ok((body_id, totals_to_distribute)) = body_totals_query.get(body_templ_ent) else {
            error!(target: BODY_BUILD, "Body template {} is missing distributed totals; skipping build for {}", entity_dbg(body_templ_ent, &display_name_query), entity_dbg(being_ent, &display_name_query));
            continue;
        };
        let blood_capacity = totals_to_distribute
            .0
            .get_opt(BodypartStat::STAT_BLOOD_CAPACITY)
            .copied()
            .unwrap_or_default()
            .max(0.0);
        let Ok(bodytree_ref) = bodytree_ref_query.get(body_templ_ent) else {
            error!(target: BODY_BUILD, "Body template {} is missing BodyTreeRef; skipping build for {}", entity_dbg(body_templ_ent, &display_name_query), entity_dbg(being_ent, &display_name_query));
            continue;
        };
        let Ok(source_tree_ent) = bodytree_map.0.get_cloned(bodytree_ref.0) else {
            error!(target: BODY_BUILD, "Body template {} references missing body tree hash {:?}", entity_dbg(body_templ_ent, &display_name_query), bodytree_ref.0);
            continue;
        };
        trace!(target: BODY_BUILD, "Building body '{}' for being {} using source tree {}", body_id, entity_dbg(being_ent, &display_name_query), entity_dbg(source_tree_ent, &display_name_query));
        let Ok(&body_hash) = body_hash_query.get(body_templ_ent) else {
            error!(target: BODY_BUILD, "Body template {} has no HashId", entity_dbg(body_templ_ent, &display_name_query));
            continue;
        };

        let body_ent = cmd.spawn((
            BodyOf { being: being_ent },
            ChildOf(being_ent),
            TemplEntiRef(body_templ_ent),
            TemplHashIdRef(body_hash),
            BodySums {
                blood_capacity,
                blood: blood_capacity,
                ..Default::default()
            },
        )).id();
        if blood_capacity > 0.0 {
            cmd.spawn((
                ModifierTarget(body_ent),
                BaseValue(blood_capacity),
                CurrEffectiveValue(blood_capacity),
                ApplyMode::Add,
                BloodCapacity,
                ChildOf(body_ent),
            ));
        }
        let Ok((templ_bodyparts, )) = templ_tree_bodyparts_query.get(source_tree_ent) else {
            error!(target: BODY_BUILD, "Body tree {} has no BodypartChildrenBodyparts; skipping {}", entity_dbg(source_tree_ent, &display_name_query), entity_dbg(being_ent, &display_name_query));
            continue;
        };
        let root_templ_bodypart = templ_bodyparts
            .iter()
            .find(|&templ_bodypart| root_bodypart_query.get(templ_bodypart).is_ok());
        let Some(root_templ_bodypart) = root_templ_bodypart else {
            error!(target: BODY_BUILD, "Body tree {} has no TreeRoot bodypart; skipping {}", entity_dbg(source_tree_ent, &display_name_query), entity_dbg(being_ent, &display_name_query));
            continue;
        };
        trace!(target: BODY_BUILD, "Selected root source bodypart {} for being {}", entity_dbg(root_templ_bodypart, &display_name_query), entity_dbg(being_ent, &display_name_query));

        cloned_parts_to_source.clear();
        let source_part_count = templ_bodyparts.iter().count();
        cloned_parts_to_source.reserve(source_part_count);

        let Some(new_root_ent) = walk_and_clone_tree(
            &mut cmd,
            root_templ_bodypart,
            &toclone_query,
            &bodypart_hash_query,
            cloned_parts_to_source,
            None,
            body_ent,
            &display_name_query,
        ) else {
            continue;
        };
        trace!(target: BODY_BUILD, "Root clone {} finished for being {}; attached to body {}", entity_dbg(new_root_ent, &display_name_query), entity_dbg(being_ent, &display_name_query), entity_dbg(body_ent, &display_name_query));

        distribute_budgets_among_bodyparts_based_on_weights_and_forcings(
            &mut cmd,
            body_id,
            &cloned_parts_to_source,
            body_ent,
            totals_to_distribute,
            &forced_query,
            &weighted_query,
            &synergy_query,
        );
    }
}

fn walk_and_clone_tree(
    cmd: &mut Commands,
    templtree_curr_node_ent: Entity,
    ref_of_bpart_toclone_query: &Query<(&BodypartChildrenBodyparts, ), (With<Templ>, )>,
    bodypart_hash_query: &Query<&HashId, With<Bodypart>>,
    cloned_parts_to_source: &mut Vec<(Entity, Entity)>,
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
    let Ok(&source_part_hash) = bodypart_hash_query.get(templtree_curr_node_ent) else {
        error!(target: BODY_BUILD, "Bodypart template {} has no HashId while cloning for body {}", entity_dbg(templtree_curr_node_ent, display_name_query), entity_dbg(body_ent, display_name_query));
        return None;
    };
    cmd.entity(cloned_bodypart_ent).insert((
        BodypartChildOfBodypart { parent_bodypart },
        ChildOf(body_ent),
        TemplEntiRef(templtree_curr_node_ent),
        TemplHashIdRef(source_part_hash),
        Name::default(),
    ));
    cloned_parts_to_source.push((cloned_bodypart_ent, templtree_curr_node_ent));
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
                bodypart_hash_query,
                cloned_parts_to_source,
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
