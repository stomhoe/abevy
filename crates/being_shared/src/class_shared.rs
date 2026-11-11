use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use common::common_components::*;
use serde::{Deserialize, Serialize, de::{self, *}};

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Class"), AssetScoped, SelectedForHotReload)]
pub struct Class;

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ClassRef(pub HashId);

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[require(Replicated)]
pub struct PlayerSelectableClass;

#[derive(Component, Serialize, Deserialize, Clone)]
#[require(Replicated, Prefix::trunc("Ability"), AssetScoped, SelectedForHotReload)]
pub struct Ability;

#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct AbilityRef(pub HashId);

#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClassRequirements {
    pub allowed_race_tags: Vec<HashId>,
    pub required_abilities: Vec<HashId>,
    pub required_spells: Vec<HashId>,
    pub allowed_weapons: Vec<HashId>,
}

#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClassData {
    pub pet_set: Option<HashId>,
    pub starting_abilities: Vec<HashId>,
    pub starting_loadout: Vec<(HashId, u32)>,
    pub starting_followers: Vec<HashId>,
    pub progression: Vec<ClassProgressionData>,
    pub race_loadouts: bevy::platform::collections::HashMap<HashId, Vec<(HashId, u32)>>,
}

impl ClassData {
    pub fn loadout_for_race(&self, race_id: HashId) -> &[(HashId, u32)] {
        self.race_loadouts.get(&race_id).map(Vec::as_slice).unwrap_or(&self.starting_loadout)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClassProgressionData {
    pub level: u32,
    pub unlock_abilities: Vec<HashId>,
}

#[derive(Component, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AbilityEffects(pub Vec<AbilityEffectData>);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AbilityEffectData {
    pub effect_id: HashId,
    pub parameters: Vec<(String, String)>,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ClassSeri {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub stats_description: String,
    pub player_selectable: bool,
    pub pet_set: Option<String>,
    pub tags: Vec<String>,
    pub allowed_race_tags: Vec<String>,
    pub required_abilities: Vec<String>,
    pub required_spells: Vec<String>,
    pub allowed_weapons: Vec<String>,
    pub starting_abilities: Vec<String>,
    pub starting_loadout: Vec<(String, u32)>,
    pub starting_followers: Vec<String>,
    pub progression: Vec<ClassProgressionSeri>,
    pub race_loadouts: bevy::platform::collections::HashMap<String, Vec<(String, u32)>>,
}

#[derive(Debug, Clone, Default)]
pub struct ClassProgressionSeri {
    pub level: u32,
    pub unlock_abilities: Vec<String>,
}

impl<'de> Deserialize<'de> for ClassProgressionSeri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProgressionVisitor;

        impl<'de> Visitor<'de> for ProgressionVisitor {
            type Value = ClassProgressionSeri;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a progression entry: (level [unlock_abilities])")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let Some(level) = sequence.next_element()? else {
                    return Err(de::Error::invalid_length(0, &self));
                };
                let Some(unlock_abilities) = sequence.next_element()? else {
                    return Err(de::Error::invalid_length(1, &self));
                };
                if sequence.next_element::<IgnoredAny>()?.is_some() {
                    return Err(de::Error::invalid_length(3, &self));
                }
                Ok(ClassProgressionSeri { level, unlock_abilities })
            }
        }

        deserializer.deserialize_tuple(2, ProgressionVisitor)
    }
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AbilitySeri {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub effects: Vec<AbilityEffectSeri>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AbilityEffectSeri {
    pub effect_id: String,
    pub parameters: Vec<(String, String)>,
}
