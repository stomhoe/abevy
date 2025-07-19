use bevy::math::U16Vec2;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::Spritesheet;

pub const DOWN: &str = "_down";
pub const LEFT: &str = "_left";
pub const UP: &str = "_up";
pub const RIGHT: &str = "_right";
pub const IDLE: &str = "_idle";
pub const WALK: &str = "_walk";
pub const SWIM : &str = "_swim";
pub const FLY: &str = "_fly";
pub const IDLE_DOWN: &str = "_idle_down";
pub const IDLE_UP: &str = "_idle_up";
pub const IDLE_LEFT: &str = "_idle_left";
pub const IDLE_RIGHT: &str = "_idle_right";
pub const BODY: &str = "_body";
pub const HEAD: &str = "_head";
pub const BODY_IDLE_DOWN: &str = "_body_idle_down";
pub const BODY_IDLE_UP: &str = "_body_idle_up";
pub const BODY_IDLE_LEFT: &str = "_body_idle_left";
pub const BODY_IDLE_RIGHT: &str = "_body_idle_right";
pub const WALK_DOWN: &str = "_walk_down";
pub const WALK_UP: &str = "_walk_up";
pub const WALK_LEFT: &str = "_walk_left";
pub const WALK_RIGHT: &str = "_walk_right";

pub const BASE_BODY_SPRITESHEET_SIZE: U16Vec2 = U16Vec2::new(6, 4);
pub const BASE_BODY_FRAME_SIZE: U16Vec2 = U16Vec2::new(25, 45);

pub const BASE_HEAD_SPRITESHEET_SIZE: U16Vec2 = U16Vec2::new(1, 4);
pub const BASE_HEAD_FRAME_SIZE: U16Vec2 = U16Vec2::new(17, 16);

pub fn base_humanoid_spritesheet() -> Spritesheet {Spritesheet::new(BASE_BODY_SPRITESHEET_SIZE.x as usize, BASE_BODY_SPRITESHEET_SIZE.y as usize)}