use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use bevy_ecs_tilemap::tiles::TileFlip;
use common::common_components::HashId;
use serde::{Deserialize, Serialize};
use crate::directions::DiagonalCardinalDirection;

pub type VisibleResult = (HashId, Option<TileFlip>);
pub type ModuloResult = Option<u32>;

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdjMask(pub u16);
impl AdjMask {
    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn insert(&mut self, bit: AdjMask) {
        self.0 |= bit.0;
    }
    pub fn contains_all(&self, other: AdjMask) -> bool {
        (self.0 & other.0) == other.0
    }
    pub fn count_bits(&self) -> usize {
        self.0.count_ones() as usize
    }
}

pub trait DiagonalCardinalDirectionAdjMaskExt {
    fn adj_mask_bit(self) -> AdjMask;
}
impl DiagonalCardinalDirectionAdjMaskExt for DiagonalCardinalDirection {
    #[inline]
    fn adj_mask_bit(self) -> AdjMask {
        AdjMask(match self {
            DiagonalCardinalDirection::North => 1 << 0,
            DiagonalCardinalDirection::East => 1 << 1,
            DiagonalCardinalDirection::South => 1 << 2,
            DiagonalCardinalDirection::West => 1 << 3,
            DiagonalCardinalDirection::NorthEast => 1 << 4,
            DiagonalCardinalDirection::SouthEast => 1 << 5,
            DiagonalCardinalDirection::SouthWest => 1 << 6,
            DiagonalCardinalDirection::NorthWest => 1 << 7,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexRule {
    pub connect_to: HashId,
    pub required_mask: AdjMask,
    pub out: VisibleResult,
    pub match_mode: AdjRetexRuleMatchMode,
    pub mod_res_i: ModuloResult,
    pub mod_res_j: ModuloResult,
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Clone, )]
pub struct AdjRetexConfig(
    pub Vec<AdjRetexRule>,
);

impl AdjRetexConfig {
    pub fn new(seri: AdjRetexConfigSeri) -> Self {
        let mut parsed_rules = Vec::with_capacity(seri.0.len());
        for (_rule_i, rule_seri) in seri.0.into_iter().enumerate() {
            let adj_state_seri = rule_seri.adj_state;
            let mod_res_i = (rule_seri.modulo_i != u32::MAX).then_some(rule_seri.modulo_i);
            let mod_res_j = (rule_seri.modulo_j != u32::MAX).then_some(rule_seri.modulo_j);
            let out_hash_seri = rule_seri.out_id;
            let tile_flip = rule_seri.tile_flip;
            let match_mode = rule_seri.match_mode;
            let mut required_mask = AdjMask::empty();
            let mut connect_to_str: Option<String> = None;
            let mut invalid_rule = false;
            for (id_seri, dir_seri, ) in adj_state_seri.into_iter() {
                let id_trim = id_seri.trim();
                if id_trim.is_empty() {
                    invalid_rule = true;
                    break;
                }
                if let Some(existing) = &connect_to_str {
                    if existing != id_trim {
                        invalid_rule = true;
                        break;
                    }
                } else {
                    connect_to_str = Some(id_trim.to_string());
                }
                let Some(dir) = DiagonalCardinalDirection::parse(&dir_seri) else {
                    invalid_rule = true;
                    break;
                };
                required_mask.insert(dir.adj_mask_bit());
            }
            if invalid_rule {
                continue;
            }
            let Some(connect_to_str) = connect_to_str else {
                continue;
            };
            parsed_rules.push(AdjRetexRule {
                connect_to: HashId::from(connect_to_str),
                required_mask,
                out: (HashId::from(out_hash_seri), tile_flip),
                match_mode,
                mod_res_i,
                mod_res_j,
            });
        }
        Self(parsed_rules)
    }

    pub fn get_tex_in_curr_adjacency_state(&self, adj_masks_by_hid: &HashMap<HashId, AdjMask>) -> Option<VisibleResult> {
        let mut best_match: Option<(usize, VisibleResult)> = None;
        for rule in self.0.iter() {
            let current_mask = adj_masks_by_hid.get(&rule.connect_to).copied().unwrap_or_default();
            match rule.match_mode {
                AdjRetexRuleMatchMode::ExactState => {
                    if current_mask == rule.required_mask {
                        return Some(rule.out);
                    }
                }
                AdjRetexRuleMatchMode::BestMatching => {
                    if current_mask.contains_all(rule.required_mask) {
                        let reqs_len = rule.required_mask.count_bits();
                        let should_replace = match best_match {
                            Some((best_len, ..)) => reqs_len > best_len,
                            None => true,
                        };
                        if should_replace {
                            best_match = Some((reqs_len, rule.out));
                        }
                    }
                }
            }
        }
        best_match.map(|(_, visible_result)| visible_result)
    }
}

#[derive(Deserialize, Asset, TypePath, Default, )]
/// something similar to godot's autotiling
pub struct AdjRetexConfigSeri(
    pub Vec<AdjRetexRuleSeri>,
);

type NeighborTileTemplStrId = String;
type NeighborAdjacencyDirection = String;

#[derive(Deserialize, Clone, )]
pub struct AdjRetexRuleSeri {
    pub adj_state: Vec<(NeighborTileTemplStrId, NeighborAdjacencyDirection)>,
    #[serde(default = "inf_u32_default")]
    pub modulo_i: u32,
    #[serde(default = "inf_u32_default")]
    pub modulo_j: u32,
    pub out_id: String,
    #[serde(default)]
    pub tile_flip: Option<TileFlip>,
    #[serde(default)]
    pub match_mode: AdjRetexRuleMatchMode,
}

fn inf_u32_default() -> u32 {
    u32::MAX
}

#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, )]
#[serde(rename_all = "snake_case")]
pub enum AdjRetexRuleMatchMode {
    #[default]
    BestMatching,
    ExactState,
}
