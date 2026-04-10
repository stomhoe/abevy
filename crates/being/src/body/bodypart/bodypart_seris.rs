#[allow(unused_imports, )] use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use item_shared::item_seris::SlottedItemHolderSeri;
use modifier_shared::modifier_seris::ModifierSynergySeri;
use tilemap_shared::tilemap_shared_samplers::NormalDistSeri;


#[derive(Asset, serde::Deserialize, TypePath, Debug, Clone)]
#[serde(default)]
/// TODO hacer que el peso/hitpoints de cada bodypart se le pueda aplicar un multiplier por el body size del animal para reducir o aumentar su respectivo valor. asi no hay que crear tantas bodyparts similares que lo unico que cambia es el peso y hp y la blood capacity
pub struct BodypartSeri {
    pub id: String,
    pub name: String,
    pub parent: String,
    pub slots: SlottedItemHolderSeri,
    pub tags: Vec<String>,
    pub depth: String,
    pub vital: bool,
    pub bleed_rate: f32,
    pub caloric_burn_rate_multipier: f32,
    pub forced_stats: HashMap<String, f32>,
    pub weighted_stats: HashMap<String, f32>,
    pub synergies: HashMap<String, ModifierSynergySeri>,
    pub extra_modifiers_on_body_holder: HashMap<String, (String, String)>,
}

impl Default for BodypartSeri {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            parent: String::new(),
            slots: Default::default(),
            tags: Default::default(),
            depth: String::new(),
            vital: false,
            bleed_rate: 0.0,
            caloric_burn_rate_multipier: 1.0,
            forced_stats: Default::default(),
            weighted_stats: Default::default(),
            synergies: Default::default(),
            extra_modifiers_on_body_holder: Default::default(),
        }
    }
}

#[derive(Asset, serde::Deserialize, TypePath, Debug, Clone)]
#[serde(default)]
pub struct AttackToolSeri {
    pub cooldown: f32,
    pub attack_damage: NormalDistSeri,
}

impl AttackToolSeri {
    pub fn sentinel() -> Self {
        Self {
            cooldown: f32::NAN,
            attack_damage: NormalDistSeri::default(),
        }
    }

    pub fn is_sentinel(&self) -> bool {
        self.cooldown.is_nan() && self.attack_damage.is_sentinel()
    }
}

impl Default for AttackToolSeri {
    fn default() -> Self {
        Self::sentinel()
    }
}
