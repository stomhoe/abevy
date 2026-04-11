use bevy::prelude::*;
use common::common_components::{AddHashIdFromStrId, DisplayName, StrId, Tag};
use common::common_id_components::HashId;
use game_common::game_common_string_components::Description;
use game_common::game_common_components::TemplHashIdRef;
use faction_shared::FactionInstTempl;

use crate::{
    culture::culture_resources::CultureStrIdRef,
    faction_inst_templ::faction_inst_templ_resources::*,
};

pub fn init_faction_inst_templates(
    mut cmd: Commands,
    fit_map: Res<FactionInstTemplEntityMap>,
) {
    if !fit_map.0.is_empty() {
        return;
    }

    for seri in load_faction_inst_templ_seri_defs() {
        let str_id = match StrId::new_with_result(seri.id.trim(), 0) {
            Ok(str_id) => str_id,
            Err(err) => {
                error!(
                    target: "faction_inst_templ_init",
                    "Skipping faction template with invalid id '{}': {}",
                    seri.id,
                    err,
                );
                continue;
            }
        };
        let mut ecmd = cmd.spawn((FactionInstTempl, str_id.clone(), AddHashIdFromStrId, TemplHashIdRef(HashId::from(str_id.as_str()))));

        if !seri.display_name.trim().is_empty() {
            ecmd.insert(DisplayName(seri.display_name.clone()));
        }
        if let Some(description) = seri.description.as_ref() {
            ecmd.insert(Description(description.clone()));
        }

        if !seri.tags.is_empty() {
            ecmd.insert(FactionTemplateTags(
                seri.tags.iter().map(Tag::trunc).collect(),
            ));
        }

        if !seri.default_relationships_by_tag.is_empty() {
            let relation_config = seri
                .default_relationships_by_tag
                .iter()
                .map(|(tag, relation)| (Tag::trunc(tag), relation.clone()))
                .collect();
            ecmd.insert(FacDefaultRelationsByTag(relation_config));
        }

        if !seri.culture_id.trim().is_empty() {
            let culture_str_id = match StrId::new_with_result(seri.culture_id.trim(), 0) {
                Ok(culture_str_id) => culture_str_id,
                Err(err) => {
                    error!(
                        target: "faction_inst_templ_init",
                        "Faction template '{}' has invalid culture_id '{}': {}",
                        str_id,
                        seri.culture_id,
                        err,
                    );
                    continue;
                }
            };
            ecmd.insert(CultureStrIdRef(culture_str_id));
        }

        if !seri.bit_weightmap.is_empty() {
            let bit_weightmap = seri
                .bit_weightmap
                .iter()
                .filter_map(|(bit_id, weight)| {
                    let bit_str_id = match StrId::new_with_result(bit_id.trim(), 0) {
                        Ok(bit_str_id) => bit_str_id,
                        Err(err) => {
                            error!(
                                target: "faction_inst_templ_init",
                                "Faction template '{}' has invalid bit id '{}' in bit_weightmap: {}",
                                str_id,
                                bit_id,
                                err,
                            );
                            return None;
                        }
                    };
                    Some((bit_str_id, (*weight).max(0.0)))
                })
                .collect();
            ecmd.insert(FactionTemplateBitWeightMap(bit_weightmap));
        }

        if seri.player_joinable {
            ecmd.insert(PlayerJoinable);
        }
        ecmd.insert(FactionTemplateRpgProfile {
            starting_wealth: seri.starting_wealth,
            lawfulness: seri.lawfulness,
            aggression: seri.aggression,
            isolationism: seri.isolationism,
            expansionism: seri.expansionism,
            max_members: seri.max_members,
        });
    }
}
