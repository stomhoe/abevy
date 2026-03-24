use bevy::prelude::*;
use common::common_components::{DisplayName, StrId, Tag};
use game_common::game_common_string_components::Description;
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
        let str_id = StrId::trunc(&seri.id);
        let mut ecmd = cmd.spawn((FactionInstTempl, str_id.clone()));

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
            ecmd.insert(CultureStrIdRef(StrId::trunc(&seri.culture_id)));
        }

        if !seri.bit_weightmap.is_empty() {
            let bit_weightmap = seri
                .bit_weightmap
                .iter()
                .map(|(bit_id, weight)| (StrId::trunc(bit_id), (*weight).max(0.0)))
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
