use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use common::common_components::HashId;
use multiplayer_shared::multiplayer_events::CharacterCreationRequest;
use ::being_shared::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct CharacterCreationSelection {
    pub race: Option<HashId>,
    pub class: Option<HashId>,
    pub sprites: HashMap<HashId, HashId>,
    pub pets: HashMap<HashId, u32>,
    pub drafts: HashMap<HashId, CharacterCreationDraft>,
}

#[derive(Clone, Debug, Default)]
pub struct CharacterCreationDraft {
    pub class: Option<HashId>,
    pub sprites: HashMap<HashId, HashId>,
    pub pets: HashMap<HashId, u32>,
}

impl CharacterCreationSelection {
    pub fn pets_fit(&self, set: &PetSetData, additional: Option<HashId>) -> bool {
        let mut total = 0u32;
        let mut cost = 0u32;
        let mut per_race = HashMap::with_capacity(set.pets.len());
        for (id, count) in self.pets.iter().map(|(&id, &count)| (id, count)).chain(additional.map(|id| (id, 1))) {
            let Some(pet) = set.pets.iter().find(|pet| pet.id == id) else { return false; };
            total = total.saturating_add(count);
            cost = cost.saturating_add(pet.cost.saturating_mul(count));
            let race_count = per_race.entry(pet.race_id).or_insert(0u32);
            *race_count = race_count.saturating_add(count);
            if total > set.max_selected || cost > set.budget || set.per_race_max.get(&pet.race_id).is_some_and(|max| *race_count > *max) { return false; }
        }
        true
    }

    pub fn switch_race(&mut self, race: HashId) {
        if self.race == Some(race) { return; }
        if let Some(previous) = self.race {
            self.drafts.insert(previous, CharacterCreationDraft {
                class: self.class.take(),
                sprites: std::mem::take(&mut self.sprites),
                pets: std::mem::take(&mut self.pets),
            });
        }
        let draft = self.drafts.remove(&race).unwrap_or_default();
        self.race = Some(race);
        self.class = draft.class;
        self.sprites = draft.sprites;
        self.pets = draft.pets;
    }

    pub fn to_request(&self) -> Option<CharacterCreationRequest> {
        let (Some(race), Some(class)) = (self.race, self.class) else { return None; };
        let mut pets = Vec::with_capacity(self.pets.values().sum::<u32>() as usize);
        for (pet, count) in &self.pets {
            pets.extend(std::iter::repeat_n(*pet, *count as usize));
        }
        Some(CharacterCreationRequest {
            race,
            class,
            sprites: self.sprites.iter().map(|(group, sprite)| (*group, *sprite)).collect(),
            pets,
        })
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterCreationButton {
    Race(HashId),
    Class(HashId),
    ToggleRaceDropdown,
    ToggleClassDropdown,
    ToggleSpriteGroup(HashId),
    Sprite { group: HashId, sprite: HashId },
    Pet { pet: HashId, delta: i8 },
    Cancel,
}

#[derive(Component, Clone, Copy)]
pub struct SpriteGroupMenu(pub HashId);

#[derive(Component, Clone, Copy)]
pub struct SpriteGroupAnchor(pub HashId);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum CharacterCreationDropdownKind {
    Race,
    Class,
}

#[derive(Component, Clone, Copy)]
pub struct CharacterCreationDropdownMenu(pub CharacterCreationDropdownKind);

#[derive(Component, Clone, Copy)]
pub struct CharacterCreationDropdownAnchor(pub CharacterCreationDropdownKind);

#[derive(Component, Clone)]
pub struct CharacterCreationRoot;

#[derive(Component)]
pub struct CharacterCreationColumns;

#[derive(Component)]
pub struct CharacterCreationScrollArea;

#[derive(Component)]
pub struct CharacterCreationPopup(pub Entity);

#[derive(Component)]
pub struct CharacterCreationSummary;

#[derive(Component)]
pub struct CharacterCreationAppearanceChoices;

#[derive(Component)]
pub struct CharacterCreationIdentityChoices;

#[derive(Component)]
pub struct CharacterCreationPetChoices;

#[derive(Component)]
pub struct CharacterCreationCompanionsSection;

#[derive(Component)]
pub struct CompanionHover(pub HashId);

#[derive(Component)]
pub struct CompanionTooltip;

#[derive(Component, Clone)]
pub struct SpriteChoiceIcon {
    pub image: Handle<Image>,
    pub frame: Option<usize>,
    pub rows: u32,
    pub columns: u32,
}

#[derive(Component)]
pub struct SpriteChoiceIconPrepared;

