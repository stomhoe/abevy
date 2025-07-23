use bevy::math::UVec2;
#[allow(unused_imports)] use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::Spritesheet;

pub const BASE_BODY_SPRITESHEET_SIZE: UVec2 = UVec2::new(6, 4);
pub const BASE_BODY_FRAME_SIZE: UVec2 = UVec2::new(25, 45);

pub const BASE_HEAD_SPRITESHEET_SIZE: UVec2 = UVec2::new(1, 4);
pub const BASE_HEAD_FRAME_SIZE: UVec2 = UVec2::new(17, 16);

pub fn base_humanoid_spritesheet() -> Spritesheet {Spritesheet::new(BASE_BODY_SPRITESHEET_SIZE.x as usize, BASE_BODY_SPRITESHEET_SIZE.y as usize)}


//pub fn 