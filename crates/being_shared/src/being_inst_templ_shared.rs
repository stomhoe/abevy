use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use bevy_replicon::prelude::Replicated;
use tilemap_shared::tilemap_shared_samplers::NormalDistSeri;
use common::common_components::*;
use common::common_tag_components::TagSet;
use serde::{Deserialize, Serialize};
use crate::WanderSeri;
use tilemap_shared::InteractionZoneSeri;

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone, )]
#[require(Replicated, Prefix::trunc("BeingInstTemplate"), AssetScoped, SelectedForHotReload)]
pub struct BeingInstTemplate{
    pub points: u32,
    pub extra_health_multiplier: f32,
}
impl BeingInstTemplate {
    pub fn default_pack_spawn_radius() -> u8 {
        crate::PackSpawnRadius::default().0
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DontExtendBitSpawnWhitelist;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DontExtendBitSpawnBlacklist;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DontExtendRaceSpawnWhitelist;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DontExtendRaceSpawnBlacklist;

#[derive(serde::Deserialize, Asset, TypePath, Debug)]
#[serde(default)]
pub struct BitSeri {
    pub id: String,
    pub tags: HashSet<String>,
    pub points: u32,
    pub fallback_faction: String,
    pub consecutive_name_weighted_distributions: Vec<Vec<(String, f32)>>,
    pub race: String,
    pub scs_samplers: Vec<String>,
    pub sprites_scale_ranges: HashMap<String, (f32, f32)>,
    pub size_variation: NormalDistSeri,
    pub hori_variation: NormalDistSeri,
    pub vert_variation: NormalDistSeri,
    pub health_multiplier: f32,
    pub body: String,
    pub recruitment_difficulty: i32,
    pub whitelisted_tiles_for_spawning: HashSet<String>,
    pub blacklisted_tiles_for_spawning: HashSet<String>,
    pub predator: crate::PredatorSeri,
    pub melee_attack_zone: InteractionZoneSeri,
    pub collision_zone: InteractionZoneSeri,
    pub spawn_pack_size_normal_dist: NormalDistSeri,
    pub pack_spawn_radius: u8,
    pub belongs_to_packs: Vec<String>,
    pub biome_affinity: HashMap<String, f32>,
    pub whitelisted_spawn_tile_tags: HashSet<String>,
    pub blacklisted_spawn_tile_tags: HashSet<String>,
    pub dont_extend_from_bit_spawn_whitelist: bool,
    pub dont_extend_from_bit_spawn_blacklist: bool,
    pub dont_extend_from_race_spawn_whitelist: bool,
    pub dont_extend_from_race_spawn_blacklist: bool,
    pub spawn_pack_entity: bool,
    pub wander: WanderSeri,
}

impl BitSeri {
    pub fn tags_and_own_id(&self) -> TagSet {
        TagSet::new(self.tags.iter().chain(std::iter::once(&self.id)))
    }
}

impl Default for BitSeri {
    fn default() -> Self {
        Self {
            id: String::default(),
            tags: HashSet::default(),
            points: 0,
            fallback_faction: String::default(),
            consecutive_name_weighted_distributions: Vec::default(),
            race: String::default(),
            scs_samplers: Vec::default(),
            sprites_scale_ranges: HashMap::default(),
            size_variation: NormalDistSeri::default(),
            hori_variation: NormalDistSeri::default(),
            vert_variation: NormalDistSeri::default(),
            health_multiplier: 1.0,
            body: String::default(),
            recruitment_difficulty: 0,
            whitelisted_tiles_for_spawning: HashSet::default(),
            blacklisted_tiles_for_spawning: HashSet::default(),
            predator: crate::PredatorSeri::default(),
            melee_attack_zone: InteractionZoneSeri::default(),
            collision_zone: InteractionZoneSeri::default(),
            spawn_pack_size_normal_dist: NormalDistSeri::default(),
            pack_spawn_radius: crate::PackSpawnRadius::default().0,
            belongs_to_packs: Vec::default(),
            biome_affinity: HashMap::default(),
            whitelisted_spawn_tile_tags: HashSet::default(),
            blacklisted_spawn_tile_tags: HashSet::default(),
            dont_extend_from_bit_spawn_whitelist: false,
            dont_extend_from_bit_spawn_blacklist: false,
            dont_extend_from_race_spawn_whitelist: false,
            dont_extend_from_race_spawn_blacklist: false,
            spawn_pack_entity: true,
            wander: WanderSeri::default(),
        }
    }
}
common::define_entity_map_systems!(
    main_component: BeingInstTemplate,
    with_filters: (),
    abbreviation: Bit,
    target: "bit",
    entity_prefix: "BIT",
    despawn_trigger: BeingInstTemplate,
    id_type: common::common_components::StrId,
    assets: [(BitSeri, "seri.being.inst_templ", "bit.ron")],
);
