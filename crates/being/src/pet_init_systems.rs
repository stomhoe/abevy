use ::being_shared::*;
use bevy::prelude::*;
use common::common_components::*;
use common::def_db::discover_assets_files_by_suffixes;
use common::log_targets::CLASS_INIT;
use game_common::game_common_string_components::Description;
use crate::being_def_parser::parse_typed_def;

fn load_pet_sets() -> Vec<PetSetSeri> {
    let Ok(mut files) = discover_assets_files_by_suffixes(&[".petset"]) else {
        error!(target: CLASS_INIT, "Failed discovering pet set definitions");
        return Vec::new();
    };
    files.sort_by(|(a, _), (b, _)| a.precedence_rank().cmp(&b.precedence_rank()).then_with(|| a.rel_path.cmp(&b.rel_path)));
    files.into_iter().filter_map(|(_, path)| {
        let Ok(content) = std::fs::read_to_string(&path) else {
            error!(target: CLASS_INIT, "Failed reading pet set definition '{}'", path.display());
            return None;
        };
        match parse_typed_def::<PetSetSeri>(&content) {
            Ok(definition) => Some(definition),
            Err(err) => {
                error!(target: CLASS_INIT, "Failed parsing pet set definition '{}': {err}", path.display());
                None
            }
        }
    }).collect()
}

#[allow(unused_parens, )]
pub fn init_pet_sets(mut cmd: Commands) {
    for definition in load_pet_sets() {
        let Ok(id) = StrId::new_with_result(definition.id.trim(), 0) else { continue; };
        let mut entity_cmds = cmd.spawn((
            PetSet,
            id.clone(),
            id.hash_id(),
            DisplayName(definition.name),
            PetSetData {
                max_selected: definition.max_selected,
                budget: definition.budget,
                per_race_max: definition.per_race_max.iter().map(|(race, max)| (HashId::from(race), *max)).collect(),
                pets: definition.pets.into_iter().map(|pet| PetSetEntry {
                    id: HashId::from(pet.id.clone()),
                    name: pet.id.replace('_', " "),
                    race_id: HashId::from(pet.race_id),
                    cost: pet.cost,
                }).collect(),
            },
        ));
        if !definition.description.trim().is_empty() {
            entity_cmds.insert(Description(definition.description));
        }
    }
}
