use bevy::{ecs::entity::MapEntities, platform::collections::HashMap, prelude::*};
use common::common_components::{StrId, Tag};
use faction_shared::FactionInstTempl;

pub use crate::faction_inst_templ::faction_inst_templ_seris::*;

common::define_entity_map_systems!(
    main_component: FactionInstTempl,
    with_filters: (),
    abbreviation: Fit,
    target: "fit",
    entity_prefix: "FIT",
    despawn_trigger: FactionInstTempl,
    id_type: common::common_components::StrId,
    assets: [(FactionInstTemplSeri, "seri.faction.inst_template", "fit.ron")],
);


#[derive(Component, Debug, Default, Clone)]
pub struct FacDefaultRelationsByTag(pub HashMap<Tag, FacRelaConfigSeri>);

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateTags(pub Vec<Tag>);

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateBitWeightMap(pub HashMap<StrId, f32>);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct PlayerJoinable;

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

#[derive(Component, Debug, Default, Clone)]
pub struct FactionTemplateRpgProfile {
    pub starting_wealth: i64,
    pub lawfulness: f32,
    pub aggression: f32,
    pub isolationism: f32,
    pub expansionism: f32,
    pub max_members: Option<u32>,
}
