#[derive(Component, Asset, TypePath, Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct WanderSeri {
    pub dir_secs_min: f32,
    pub dir_secs_max: f32,
    pub move_secs_min: f32,
    pub move_secs_max: f32,
    pub halt_secs_min: f32,
    pub halt_secs_max: f32,
    pub speed_min: f32,
    pub speed_max: f32,
    pub avoid_tile_tags: HashSet<String>,
    pub avoid_bit_tags: HashMap<String, AvoidBeingSpec>,
    pub avoid_race_tags: HashMap<String, AvoidBeingSpec>,
    pub avoid_pack_tags: HashMap<String, AvoidBeingSpec>,
    pub max_drift: f32,
    pub allow_hard_flips_to_return: bool,
    pub wander_around_leader: bool,
    pub avoid_being_tags: HashMap<String, AvoidBeingSpec>,
    pub avoid_blacklisted_spawn_tiles: bool,
    pub pack_orbit_radius: f32,
    pub pack_orbit_retarget_secs_min: f32,
    pub pack_orbit_retarget_secs_max: f32,
}
impl Default for WanderSeri {
    fn default() -> Self {
        Self {
            dir_secs_min: 1.0,
            dir_secs_max: 2.0,
            move_secs_min: 0.8,
            move_secs_max: 2.2,
            halt_secs_min: 0.2,
            halt_secs_max: 1.0,
            speed_min: 0.4,
            speed_max: 0.6,
            avoid_tile_tags: HashSet::default(),
            avoid_bit_tags: HashMap::default(),
            avoid_race_tags: HashMap::default(),
            avoid_pack_tags: HashMap::default(),
            max_drift: 30.0,
            allow_hard_flips_to_return: false,
            wander_around_leader: false,
            avoid_being_tags: HashMap::default(),
            avoid_blacklisted_spawn_tiles: false,
            pack_orbit_radius: 70.0,
            pack_orbit_retarget_secs_min: 30.,
            pack_orbit_retarget_secs_max: 240.,
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct AvoidBeingSpec {
    pub radius: f32,
    pub strength: f32,
}
impl AvoidBeingSpec {
    pub const DEFAULT_RADIUS: f32 = 18.0;
    pub const DEFAULT_STRENGTH: f32 = 0.85;

    pub fn strongest_entity_avoidance(&self, delta: Vec2, distance: f32, move_speed: f32) -> Vec2 {
        if self.radius <= 0.0 || distance <= 0.0 || distance > self.radius {
            return Vec2::ZERO;
        }
        let pull = ((self.radius - distance) / self.radius).clamp(0.0, 1.0);
        delta.normalize_or_zero() * (move_speed * self.strength * pull * pull)
    }
}
impl Default for AvoidBeingSpec {
    fn default() -> Self {
        Self { radius: Self::DEFAULT_RADIUS, strength: Self::DEFAULT_STRENGTH, }
    }
}
impl WanderSeri {
    pub fn resolve_wander_avoid_tile_tags(
        &self,
        has_avoid_blacklisted_spawn_tiles: bool,
        bit_ref: Option<&crate::BitRef>,
        race_ref: Option<&crate::RaceRef>,
        blacklisted_spawn_tile_tags_query: &Query<&BlacklistedSpawnTileTags>,
    ) -> BlacklistedTags {
        let mut avoid_tile_tags = BlacklistedTags::new(&self.avoid_tile_tags);
        if !has_avoid_blacklisted_spawn_tiles {
            return avoid_tile_tags;
        }
        if let Some(bit_ref) = bit_ref {
            if let Ok(bit_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(bit_ref.0) {
                if !bit_blacklisted_spawn_tile_tags.0.is_empty() {
                    avoid_tile_tags.extend_from(&bit_blacklisted_spawn_tile_tags.0);
                    return avoid_tile_tags;
                }
            }
        }
        if let Some(race_ref) = race_ref {
            if let Ok(race_blacklisted_spawn_tile_tags) = blacklisted_spawn_tile_tags_query.get(race_ref.0) {
                avoid_tile_tags.extend_from(&race_blacklisted_spawn_tile_tags.0);
            }
        }
        avoid_tile_tags
    }

    pub fn sample_pack_orbit_target(
        &self,
        rng: &mut impl Rng,
        center: GlobalTilePos,
    ) -> GlobalTilePos {
        let radius = self.pack_orbit_radius.round().max(0.0) as i32;
        if radius <= 0 {
            return center;
        }
        for _ in 0..8 {
            let offset = IVec2::new(
                rng.random_range(-radius..=radius),
                rng.random_range(-radius..=radius),
            );
            if offset != IVec2::ZERO {
                return GlobalTilePos(center.0 + offset);
            }
        }
        center
    }

    pub fn sanitized(mut self) -> Self {
        self.dir_secs_min = self.dir_secs_min.max(0.01);
        self.dir_secs_max = self.dir_secs_max.max(self.dir_secs_min);
        self.move_secs_min = self.move_secs_min.max(0.01);
        self.move_secs_max = self.move_secs_max.max(self.move_secs_min);
        self.halt_secs_min = self.halt_secs_min.max(0.01);
        self.halt_secs_max = self.halt_secs_max.max(self.halt_secs_min);
        self.speed_min = self.speed_min.max(0.0);
        self.speed_max = self.speed_max.max(self.speed_min);
        self.max_drift = self.max_drift.max(0.0);
        self.pack_orbit_radius = self.pack_orbit_radius.max(0.0);
        self.pack_orbit_retarget_secs_min = self.pack_orbit_retarget_secs_min.max(0.0);
        self.pack_orbit_retarget_secs_max = self.pack_orbit_retarget_secs_max.max(self.pack_orbit_retarget_secs_min);
        self.avoid_bit_tags = self
            .avoid_bit_tags
            .into_iter()
            .map(|(tag, spec)| {
                (
                    tag,
                    AvoidBeingSpec {
                        radius: spec.radius.max(0.0),
                        strength: spec.strength.max(0.0),
                    },
                )
            })
            .collect();
        self.avoid_race_tags = self
            .avoid_race_tags
            .into_iter()
            .map(|(tag, spec)| {
                (
                    tag,
                    AvoidBeingSpec {
                        radius: spec.radius.max(0.0),
                        strength: spec.strength.max(0.0),
                    },
                )
            })
            .collect();
        self.avoid_pack_tags = self
            .avoid_pack_tags
            .into_iter()
            .map(|(tag, spec)| {
                (
                    tag,
                    AvoidBeingSpec {
                        radius: spec.radius.max(0.0),
                        strength: spec.strength.max(0.0),
                    },
                )
            })
            .collect();
        self.avoid_being_tags = self
            .avoid_being_tags
            .into_iter()
            .map(|(tag, spec)| {
                (
                    tag,
                    AvoidBeingSpec {
                        radius: spec.radius.max(0.0),
                        strength: spec.strength.max(0.0),
                    },
                )
            })
            .collect();
        self
    }

    pub fn is_disabled(&self) -> bool {
        self.dir_secs_min == 0.0
            && self.dir_secs_max == 0.0
            && self.move_secs_min == 0.0
            && self.move_secs_max == 0.0
            && self.halt_secs_min == 0.0
            && self.halt_secs_max == 0.0
            && self.speed_min == 0.0
            && self.speed_max == 0.0
            && self.avoid_tile_tags.is_empty()
            && self.avoid_bit_tags.is_empty()
            && self.avoid_race_tags.is_empty()
            && self.avoid_pack_tags.is_empty()
            && self.max_drift == 0.0
            && self.pack_orbit_radius == 0.0
            && self.pack_orbit_retarget_secs_min == 0.0
            && self.pack_orbit_retarget_secs_max == 0.0
            && !self.wander_around_leader
            && self.avoid_being_tags.is_empty()
            && !self.avoid_blacklisted_spawn_tiles
    }

    pub fn avoid_being_radius_for(&self, tag: &str) -> f32 {
        self.avoid_being_tags
            .get(tag)
            .map(|spec| spec.radius)
            .unwrap_or(AvoidBeingSpec::DEFAULT_RADIUS)
    }

    pub fn avoid_being_strength_for(&self, tag: &str) -> f32 {
        self.avoid_being_tags
            .get(tag)
            .map(|spec| spec.strength)
            .unwrap_or(AvoidBeingSpec::DEFAULT_STRENGTH)
    }

    pub fn max_avoid_being_radius(&self) -> f32 {
        self.avoid_being_tags
            .values()
            .map(|spec| spec.radius)
            .chain(self.avoid_bit_tags.values().map(|spec| spec.radius))
            .chain(self.avoid_race_tags.values().map(|spec| spec.radius))
            .chain(self.avoid_pack_tags.values().map(|spec| spec.radius))
            .fold(AvoidBeingSpec::DEFAULT_RADIUS, f32::max)
    }

    pub fn sample_pack_orbit_timer_secs(&self, rng: &mut impl Rng) -> f32 {
        if self.pack_orbit_retarget_secs_max <= 0.0 {
            return 0.0;
        }
        sample_seconds(rng, self.pack_orbit_retarget_secs_min, self.pack_orbit_retarget_secs_max)
    }
}
#[derive(Component, Debug, Clone, Deserialize, Serialize, Reflect, Default)]
#[require(BehavorialNavState, )]
pub struct WanderState {
    pub(crate) dir: Vec2,
    pub(crate) dir_secs_left: f32,
    pub(crate) speed_mult: f32,
    pub(crate) halting: bool,
    pub(crate) phase_secs_left: f32,
    pub(crate) pack_orbit_secs_left: f32,
    pub(crate) pack_orbit_target: Option<GlobalTilePos>,
    #[serde(default)]
    #[reflect(ignore)]
    pub(crate) pack_return_dir: Option<CardinalDirection>,
    pub(crate) lod_level: u8,
    pub(crate) lod_secs_left: f32,
    pub(crate) lod_accum_secs: f32,
}
impl WanderState {
    pub fn new(rng: &mut impl Rng, cfg: &WanderSeri) -> Self {
        let mut state = Self::default();
        state.initialize(rng, cfg);
        state
    }

    pub fn is_uninitialized(&self) -> bool {
        self.dir_secs_left <= 0.0
            && self.phase_secs_left <= 0.0
            && self.pack_orbit_secs_left <= 0.0
            && self.lod_secs_left <= 0.0
            && self.dir == Vec2::ZERO
            && self.speed_mult == 0.0
            && self.pack_return_dir.is_none()
    }

    pub fn initialize(&mut self, rng: &mut impl Rng, cfg: &WanderSeri) {
        self.dir = CardinalDirection::random(rng).to_dir_vec().as_vec2();
        self.dir_secs_left = sample_seconds(rng, cfg.dir_secs_min, cfg.dir_secs_max);
        self.speed_mult = sample_seconds(rng, cfg.speed_min, cfg.speed_max);
        self.halting = false;
        self.phase_secs_left = sample_seconds(rng, cfg.move_secs_min, cfg.move_secs_max);
        self.pack_orbit_secs_left = cfg.sample_pack_orbit_timer_secs(rng);
        self.pack_orbit_target = None;
        self.pack_return_dir = None;
        self.lod_level = 0;
        self.lod_secs_left = 0.0;
        self.lod_accum_secs = 0.0;
    }

    pub fn advance_motion(
        &mut self,
        dt: f32,
        rng: &mut impl Rng,
        cfg: &WanderSeri,
    ) -> Vec2 {
        self.dir_secs_left -= dt;
        if self.dir_secs_left <= 0.0 {
            if let Some(return_dir) = self.pack_return_dir.take() {
                self.dir = return_dir.to_dir_vec().as_vec2();
                self.dir_secs_left = sample_seconds(rng, 1.0, 1.5);
            } else {
                self.dir = pick_wander_dir(rng, CardinalDirection::from_dir_vec(self.dir.as_ivec2())).to_dir_vec().as_vec2();
                self.dir_secs_left = sample_seconds(rng, cfg.dir_secs_min, cfg.dir_secs_max);
            }
        }

        self.phase_secs_left -= dt;
        if self.phase_secs_left <= 0.0 {
            self.halting = !self.halting;
            if self.halting {
                self.phase_secs_left = sample_seconds(rng, cfg.halt_secs_min, cfg.halt_secs_max);
            } else {
                self.phase_secs_left = sample_seconds(rng, cfg.move_secs_min, cfg.move_secs_max);
                self.speed_mult = sample_seconds(rng, cfg.speed_min, cfg.speed_max);
            }
        }

        if self.halting {
            Vec2::ZERO
        } else {
            self.dir * self.speed_mult
        }
    }

    pub fn advance_lod(&mut self, dt: f32, lod_level: u8, lod_interval_secs: f32) -> Option<f32> {
        self.lod_accum_secs += dt.max(0.0);
        if self.lod_level != lod_level {
            self.lod_level = lod_level;
            self.lod_secs_left = lod_interval_secs.max(0.0);
            let elapsed = self.lod_accum_secs;
            self.lod_accum_secs = 0.0;
            return Some(elapsed);
        }
        if lod_level == 0 {
            self.lod_secs_left = 0.0;
            let elapsed = self.lod_accum_secs;
            self.lod_accum_secs = 0.0;
            return Some(elapsed);
        }
        if self.lod_secs_left > 0.0 {
            self.lod_secs_left = (self.lod_secs_left - dt).max(0.0);
        }
        if self.lod_secs_left > 0.0 {
            return None;
        }
        self.lod_secs_left = lod_interval_secs.max(0.0);
        let elapsed = self.lod_accum_secs;
        self.lod_accum_secs = 0.0;
        Some(elapsed)
    }

    pub fn pack_orbit_pull(
        &mut self,
        dt: f32,
        rng: &mut impl Rng,
        cfg: &WanderSeri,
        pack_center: GlobalTilePos,
        gpos: GlobalTilePos,
        mut is_target_allowed: impl FnMut(GlobalTilePos) -> bool,
    ) -> Vec2 {
        if cfg.pack_orbit_radius <= 0.0 {
            return Vec2::ZERO;
        }
        self.pack_orbit_secs_left -= dt;
        if self.pack_orbit_target.is_none() || self.pack_orbit_secs_left <= 0.0 {
            self.pack_orbit_target = sample_pack_orbit_target_filtered(rng, cfg, pack_center, &mut is_target_allowed);
            self.pack_orbit_secs_left = cfg.sample_pack_orbit_timer_secs(rng);
        }
        let Some(orbit_target) = self.pack_orbit_target else {
            return Vec2::ZERO;
        };
        if !is_target_allowed(orbit_target) {
            self.pack_orbit_target = None;
            return Vec2::ZERO;
        }
        let orbit_delta = orbit_target.0 - gpos.0;
        let orbit_distance = orbit_delta.as_vec2().length();
        if orbit_distance <= 0.5 {
            return Vec2::ZERO;
        }
        let pull = ((orbit_distance - cfg.pack_orbit_radius).max(0.0)
            / cfg.pack_orbit_radius.max(1.0))
            .clamp(0.0, 1.0);
        orbit_delta.as_vec2().normalize_or_zero() * (self.speed_mult * 0.75 * pull)
    }

    pub fn current_speed_mult_or_zero(&self) -> f32 {
        self.speed_mult.max(0.0)
    }

    pub fn has_pack_return_dir(&self) -> bool {
        self.pack_return_dir.is_some()
    }

    pub fn maybe_apply_pack_drift_return(
        &mut self,
        rng: &mut impl Rng,
        cfg: &WanderSeri,
        gpos: GlobalTilePos,
        pack_center: GlobalTilePos,
    ) -> Option<Vec2> {
        if !cfg.max_drift.is_finite() {
            return None;
        }

        let speed_mult = self.current_speed_mult_or_zero();
        if speed_mult <= 0.0 {
            return None;
        }

        let pack_delta = pack_center.0 - gpos.0;
        let current_step = self.dir.as_ivec2();
        if current_step == IVec2::ZERO {
            return None;
        }

        let current_dist_sq = pack_delta.x.saturating_mul(pack_delta.x).saturating_add(pack_delta.y.saturating_mul(pack_delta.y));
        let current_next_dist_sq = pack_drift_candidate_distance_sq(gpos, pack_center, current_step);
        if current_next_dist_sq < current_dist_sq {
            self.pack_return_dir = None;
            return None;
        }
        if cfg.allow_hard_flips_to_return {
            self.pack_return_dir = None;
            return Some(cardinal_step_toward(pack_delta).as_vec2() * (speed_mult * 2.0));
        }

        let current_dir = CardinalDirection::from_dir_vec(current_step);
        let adjacent_dirs = [
            current_dir.next_clockwise(),
            current_dir.next_clockwise().next_clockwise().next_clockwise(),
        ];
        let mut chosen_dir = None;
        let mut chosen_dist_sq = i32::MAX;
        for candidate_dir in adjacent_dirs {
            let candidate_step = candidate_dir.to_dir_vec();
            let candidate_dist_sq = pack_drift_candidate_distance_sq(gpos, pack_center, candidate_step);
            if candidate_dist_sq < chosen_dist_sq {
                chosen_dir = Some(candidate_dir);
                chosen_dist_sq = candidate_dist_sq;
            }
        }

        let Some(chosen_dir) = chosen_dir else {
            return None;
        };

        self.dir = chosen_dir.to_dir_vec().as_vec2();
        self.dir_secs_left = sample_seconds(rng, 1.0, 1.5);
        self.pack_return_dir = if chosen_dist_sq < current_dist_sq {
            None
        } else {
            Some(CardinalDirection::from_dir_vec(cardinal_step_toward(pack_delta)))
        };
        Some(self.dir * speed_mult)
    }
}

fn pick_wander_dir(rng: &mut impl Rng, avoid_dir: CardinalDirection) -> CardinalDirection {
    let candidates = [
        CardinalDirection::South,
        CardinalDirection::West,
        CardinalDirection::North,
        CardinalDirection::East,
    ];
    let opposite_dir = avoid_dir.opposite_dir();
    let mut choices = [CardinalDirection::South; 4];
    let mut choice_count = 0usize;
    for candidate in candidates {
        if candidate != avoid_dir && candidate != opposite_dir {
            choices[choice_count] = candidate;
            choice_count += 1;
        }
    }
    if choice_count == 0 {
        return avoid_dir;
    }
    choices[rng.random_range(0..choice_count)]
}

fn sample_seconds(rng: &mut impl Rng, min: f32, max: f32) -> f32 {
    let min = min.max(0.01);
    let max = max.max(min);
    if max == min {
        min
    } else {
        rng.random_range(min..max)
    }
}

fn sample_pack_orbit_target_filtered(
    rng: &mut impl Rng,
    cfg: &WanderSeri,
    pack_center: GlobalTilePos,
    is_target_allowed: &mut impl FnMut(GlobalTilePos) -> bool,
) -> Option<GlobalTilePos> {
    const MAX_CANDIDATE_TRIES: usize = 24;
    for _ in 0..MAX_CANDIDATE_TRIES {
        let candidate = cfg.sample_pack_orbit_target(rng, pack_center);
        if is_target_allowed(candidate) {
            return Some(candidate);
        }
    }
    if is_target_allowed(pack_center) {
        return Some(pack_center);
    }
    None
}

fn cardinal_step_toward(delta: IVec2) -> IVec2 {
    if delta == IVec2::ZERO {
        IVec2::ZERO
    } else if delta.x.abs() >= delta.y.abs() {
        IVec2::new(delta.x.signum(), 0)
    } else {
        IVec2::new(0, delta.y.signum())
    }
}

fn pack_drift_candidate_distance_sq(
    gpos: GlobalTilePos,
    pack_center: GlobalTilePos,
    step: IVec2,
) -> i32 {
    let candidate_delta = pack_center.0 - (gpos.0 + step);
    candidate_delta.x.saturating_mul(candidate_delta.x).saturating_add(candidate_delta.y.saturating_mul(candidate_delta.y))
}

impl AvoidBeingSpec {
    pub fn strongest_avoidance_spec<'a>(
        avoid_tags: &'a HashMap<String, AvoidBeingSpec>,
        threat_tags: &TagSet,
    ) -> Option<&'a AvoidBeingSpec> {
        let mut strongest: Option<&AvoidBeingSpec> = None;
        for tag in threat_tags.iter() {
            let tag_str = tag.as_str();
            let Some(spec) = avoid_tags.get(tag_str) else {
                continue;
            };
            let Some(best_spec) = strongest else {
                strongest = Some(spec);
                continue;
            };
            if spec.strength > best_spec.strength
                || (spec.strength == best_spec.strength && spec.radius > best_spec.radius)
            {
                strongest = Some(spec);
            }
        }
        strongest
    }
}

#[derive(Component, Debug, Default, Deserialize, Serialize, Copy, Clone)]
pub struct DoAvoidBlacklistedSpawnTilesForWander;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy::reflect::Reflect;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::being_shared_nav_states::BehavorialNavState;
use common::common_tag_components::TagSet;
use ::tilemap_shared::*;
