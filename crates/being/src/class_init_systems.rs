use ::being_shared::*;
use bevy::prelude::*;
use common::common_components::*;
use common::{def_db::discover_assets_files_by_suffixes, log_targets::CLASS_INIT};
use game_common::game_common_string_components::Description;
use crate::being_def_parser::parse_typed_def;

fn load_defs<T: for<'de> serde::Deserialize<'de>>(suffix: &str) -> Vec<T> {
    let Ok(mut files) = discover_assets_files_by_suffixes(&[suffix]) else {
        error!(target: CLASS_INIT, "Failed discovering class definition files with suffix '{suffix}'");
        return Vec::new();
    };
    files.sort_by(|(a, _), (b, _)| a.precedence_rank().cmp(&b.precedence_rank()).then_with(|| a.rel_path.cmp(&b.rel_path)));
    files.into_iter().filter_map(|(_, path)| {
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                error!(target: CLASS_INIT, "Failed reading class definition '{}': {err}", path.display());
                return None;
            }
        };
        match parse_typed_def::<T>(&content) {
            Ok(definition) => Some(definition),
            Err(err) => {
                error!(target: CLASS_INIT, "Failed parsing class definition '{}': {err}", path.display());
                None
            }
        }
    }).collect()
}

#[allow(unused_parens, )]
pub fn init_classes(mut cmd: Commands) {
    for definition in load_defs::<ClassSeri>(".class") {
        let Ok(id) = StrId::new_with_result(definition.id.trim(), 0) else { continue; };
        let mut entity_cmds = cmd.spawn((
            Class,
            id.clone(),
            id.hash_id(),
            DisplayName(definition.name),
            Tags(definition.tags.iter().map(HashId::from).collect()),
            ClassRequirements {
                allowed_race_tags: definition.allowed_race_tags.iter().map(HashId::from).collect(),
                required_abilities: definition.required_abilities.iter().map(HashId::from).collect(),
                required_spells: definition.required_spells.iter().map(HashId::from).collect(),
                allowed_weapons: definition.allowed_weapons.iter().map(HashId::from).collect(),
            },
            ClassData {
                pet_set: definition.pet_set.as_ref().map(HashId::from),
                starting_abilities: definition.starting_abilities.iter().map(HashId::from).collect(),
                starting_loadout: definition.starting_loadout.iter().map(|(item, amount)| (HashId::from(item), *amount)).collect(),
                starting_followers: definition.starting_followers.iter().map(HashId::from).collect(),
                progression: definition.progression.iter().map(|entry| ClassProgressionData {
                    level: entry.level,
                    unlock_abilities: entry.unlock_abilities.iter().map(HashId::from).collect(),
                }).collect(),
                race_loadouts: definition.race_loadouts.iter().map(|(race, loadout)| (
                    HashId::from(race),
                    loadout.iter().map(|(item, amount)| (HashId::from(item), *amount)).collect(),
                )).collect(),
            },
        ));
        if definition.player_selectable {
            entity_cmds.insert(PlayerSelectableClass);
        }
        if !definition.description.trim().is_empty() {
            entity_cmds.insert(Description(definition.description));
        }
        if !definition.stats_description.trim().is_empty() {
            entity_cmds.insert(StatsDescription(definition.stats_description));
        }
    }
}

#[allow(unused_parens, )]
pub fn init_abilities(mut cmd: Commands) {
    for definition in load_defs::<AbilitySeri>(".ability") {
        let Ok(id) = StrId::new_with_result(definition.id.trim(), 0) else { continue; };
        cmd.spawn((
            Ability,
            id.clone(),
            id.hash_id(),
            DisplayName(definition.name),
            AbilityEffects(definition.effects.into_iter().map(|effect| AbilityEffectData {
                effect_id: HashId::from(effect.effect_id),
                parameters: effect.parameters,
            }).collect()),
        ));
    }
}
