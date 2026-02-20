use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::common_components::{StrId, Tag};
use serde::{Deserialize, Serialize};

use crate::faction_components::FactionInstTempl;

common::define_entity_map_systems!(
    FactionInstTempl,
    (),
    Fit,
    "fit",
    "FIT",
    FactionInstTempl,
    common::common_components::StrId,
    FactionInstTemplSeri, "seri.faction.inst_template", "fit.ron",
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FacRelaConfigSeri {
    #[serde(default)]
    pub base_opinion: f32,
    #[serde(default)]
    pub hostility_bias: f32,
    #[serde(default = "default_trust")]
    pub trust: f32,
    #[serde(default)]
    pub aid_chance: f32,
}
impl Default for FacRelaConfigSeri {
    fn default() -> Self {
        Self {
            base_opinion: 0.0,
            hostility_bias: 0.0,
            trust: default_trust(),
            aid_chance: 0.0,
        }
    }
}
fn default_trust() -> f32 {
    0.5
}

#[derive(Component, Debug, Default, Clone)]
pub struct FacDefaultRelationsByTag(pub HashMap<Tag, FacRelaConfigSeri>);

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateTags(pub Vec<Tag>);

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateBitWeightMap(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
pub struct PlayerJoinable(pub bool);

#[derive(Component, Debug, Clone, Copy, MapEntities)]
pub struct FactionInstancedFromTemplate(#[entities] pub Entity);

#[derive(Component, Debug, Default, Clone)]
pub struct FactionInstanceTemplateId(pub StrId);

#[derive(Component, Debug, Clone, MapEntities)]
pub struct FactionInstanceRef(#[entities] pub Entity);

#[derive(Component, Debug, Default, Clone, MapEntities)]
pub struct SpawnFactionInstanceFromTemplate {
    #[entities]
    pub requester: Option<Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct FactionInstTemplatePool(pub HashMap<StrId, Vec<Entity>>);
impl FactionInstTemplatePool {
    pub fn push(&mut self, template_id: &StrId, ent: Entity) {
        self.0.entry(template_id.clone()).or_default().push(ent);
    }

    pub fn remove(&mut self, template_id: &StrId, ent: Entity) {
        if let Some(pool) = self.0.get_mut(template_id) {
            pool.retain(|other| *other != ent);
            if pool.is_empty() {
                self.0.remove(template_id);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Asset, TypePath, Default, Debug)]
pub struct FactionInstTemplSeri {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub default_relationships_by_tag: HashMap<String, FacRelaConfigSeri>,
    #[serde(default)]
    pub culture_id: String,
    #[serde(default = "default_true")]
    pub player_joinable: bool,
    #[serde(default)]
    pub bit_weightmap: HashMap<String, f32>,
    #[serde(default)]
    pub starting_wealth: i64,
    #[serde(default)]
    pub lawfulness: f32,
    #[serde(default)]
    pub aggression: f32,
    #[serde(default)]
    pub isolationism: f32,
    #[serde(default)]
    pub expansionism: f32,
    #[serde(default)]
    pub max_members: Option<u32>,
}

fn default_true() -> bool {
    true
}

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateRpgProfile {
    pub starting_wealth: i64,
    pub lawfulness: f32,
    pub aggression: f32,
    pub isolationism: f32,
    pub expansionism: f32,
    pub max_members: Option<u32>,
}
