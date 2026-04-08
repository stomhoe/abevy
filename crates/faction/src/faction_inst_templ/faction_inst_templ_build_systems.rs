use bevy::prelude::*;
use common::common_components::{AddHashIdFromStrId, HashId, StrId};

use faction_shared::Faction;

use crate::{
    culture::culture_resources::CultureRef,
    faction_inst_templ::faction_inst_templ_resources::*,
};

#[allow(clippy::type_complexity)]
pub fn spawn_faction_instance_from_template(
    mut cmd: Commands,
    requests: Query<(Entity, &FitRef, Option<&SpawnFactionInstanceFromTemplate>), Added<FitRef>>,
    fit_map: Res<FactionInstTemplEntityMap>,
    fit_query: Query<(
        &StrId,
        Option<&FactionTemplateTags>,
        Option<&FacDefaultRelationsByTag>,
        Option<&CultureRef>,
        Has<PlayerJoinable>,
        Option<&FactionTemplateBitWeightMap>,
        Option<&FactionTemplateRpgProfile>,
    )>,
) {
    if requests.is_empty() {
        return;
    }

    let mut requester_links = Vec::new();

    for (_request_ent, fit_ref, spawn_request) in requests.iter() {
        let Ok(fit_ent) = fit_map.0.get_cloned(fit_ref.0) else {
            continue;
        };
        let Ok((
            template_id,
            template_tags,
            relation_by_tag,
            culture_ref,
            is_player_joinable,
            bit_weightmap,
            rpg_profile,
        )) = fit_query.get(fit_ent)
        else {
            continue;
        };

        let faction_ent = cmd.spawn_empty().id();
        let instance_id = StrId::trunc(format!("{}_{}", template_id.as_str(), faction_ent.index()));
        let instance_hash = HashId::from(instance_id.as_str());

        let mut ins = cmd.entity(faction_ent);
        ins.insert((
            Faction,
            instance_id,
            instance_hash,
            AddHashIdFromStrId,
            FactionInstancedFromTemplate(fit_ent),
            FactionInstanceTemplateId(template_id.clone()),
        ));

        if let Some(template_tags) = template_tags {
            ins.insert(template_tags.clone());
        }
        if let Some(relation_by_tag) = relation_by_tag {
            ins.insert(relation_by_tag.clone());
        }
        if let Some(culture_ref) = culture_ref {
            ins.insert(*culture_ref);
        }
        if is_player_joinable {
            ins.insert(PlayerJoinable);
        }
        if let Some(bit_weightmap) = bit_weightmap {
            ins.insert(bit_weightmap.clone());
        }
        if let Some(rpg_profile) = rpg_profile {
            ins.insert(rpg_profile.clone());
        }

        if let Some(spawn_request) = spawn_request {
            if let Some(requester_ent) = spawn_request.requester {
                requester_links.push((requester_ent, FactionInstanceRef(faction_ent)));
            }
        }
    }

    cmd.try_insert_batch(requester_links);
}

pub fn track_spawned_faction_instances(
    mut pool: ResMut<FactionInstTemplatePool>,
    query: Query<(Entity, &FactionInstanceTemplateId), Added<FactionInstancedFromTemplate>>,
) {
    for (faction_ent, template_id) in query.iter() {
        pool.push(&template_id.0, faction_ent);
    }
}

pub fn remove_faction_instance_from_pool_on_despawn(
    trigger: On<Despawn, Faction>,
    query: Query<&FactionInstanceTemplateId>,
    mut pool: ResMut<FactionInstTemplatePool>,
) {
    if let Ok(template_id) = query.get(trigger.entity) {
        pool.remove(&template_id.0, trigger.entity);
    }
}
