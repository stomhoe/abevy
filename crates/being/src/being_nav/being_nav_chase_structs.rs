use bevy::prelude::*;
use ::tilemap_shared::GlobalTilePos;
use std::time::Duration;

pub struct ChaserNavPlan {
    pub path_tiles: Vec<GlobalTilePos>,
    pub next_step_ix: usize,
    pub rebuild_timer: Timer,
    pub last_target_pos: Option<GlobalTilePos>,
    pub holds_at_partial_endpoint: bool,
    pub reserved_shared_goal: Option<GlobalTilePos>,
}

impl Default for ChaserNavPlan {
    fn default() -> Self {
        Self {
            path_tiles: Vec::new(),
            next_step_ix: 0,
            rebuild_timer: Timer::from_seconds(0.1, TimerMode::Once),
            last_target_pos: None,
            holds_at_partial_endpoint: false,
            reserved_shared_goal: None,
        }
    }
}

impl ChaserNavPlan {
    pub fn rebuild_interval(chaser_speed: f32, prey_speed: f32, distance: f32) -> Duration {
        let chaser_speed = chaser_speed.max(0.05);
        let speed_ratio = (prey_speed.max(0.0) / chaser_speed).clamp(0.35, 2.25);
        let distance_factor = (8.0 / distance.max(1.0)).clamp(0.45, 2.0);
        let urgency = (speed_ratio * distance_factor).clamp(0.35, 3.5);
        Duration::from_secs_f32((0.45 / urgency).clamp(0.08, 0.9))
    }

    pub fn reset(&mut self, interval: Duration) {
        self.path_tiles.clear();
        self.next_step_ix = 0;
        self.last_target_pos = None;
        self.holds_at_partial_endpoint = false;
        self.reserved_shared_goal = None;
        self.rebuild_timer = Timer::new(interval, TimerMode::Once);
    }

    pub fn clear_shared_goal(&mut self) {
        self.reserved_shared_goal = None;
    }

    pub fn clear_path_and_retry(&mut self, interval: Duration, target_pos: GlobalTilePos) {
        self.path_tiles.clear();
        self.next_step_ix = 0;
        self.last_target_pos = Some(target_pos);
        self.holds_at_partial_endpoint = false;
        self.reserved_shared_goal = None;
        self.rebuild_timer = Timer::new(interval, TimerMode::Once);
    }

    pub fn next_step(&mut self, chaser_pos: GlobalTilePos) -> Option<GlobalTilePos> {
        while self.next_step_ix < self.path_tiles.len() && self.path_tiles[self.next_step_ix] == chaser_pos {
            self.next_step_ix += 1;
        }
        self.path_tiles.get(self.next_step_ix).copied()
    }
}