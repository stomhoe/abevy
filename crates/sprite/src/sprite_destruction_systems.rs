use std::mem::take;

use bevy::{ecs::entity_disabling::Disabled, platform::collections::{HashMap, HashSet}, };
#[allow(unused_imports)] use bevy::prelude::*;
#[allow(unused_imports)] use bevy_replicon::prelude::*;
use bevy_spritesheet_animation::prelude::Animation;
use game_common::game_common_components::{Categories, Directionable, EntityZero, MyZ};
use ::sprite_shared::{sprite_scale_offset::Offset2D, *};

use crate::{sprite_components::*, sprite_resources::*, };

// ----------------------> NO OLVIDARSE DE AGREGARLO AL Plugin DEL MÓDULO <-----------------------------
//                                                       ^^^^
// #[allow(unused_parens)]
// pub fn on_sprite_config_despawn(mut cmd: Commands, 
//     mut removed_configs: RemovedComponents<SpriteConfig>,
//     mut query: Query<(&SpriteConfigUsages),(Or<(With<Disabled>, Without<Disabled>)>,)>,
// ) {
//     for ent in removed_configs.read() {
//         if let Ok(usages) = query.get(ent) {
//             for &usage_ent in usages.entities() {
//             }
//         }
//     }
// }
