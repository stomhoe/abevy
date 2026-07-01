use crate::{
    tile::{tile_components::*},
};
use bevy::prelude::*;
use being_shared::Being;
use game_common::game_common_components::*;
use rand::RngExt;
use std::f32::consts::TAU;

use super::tile_systems::ExcludedComps;

fn corpse_rotation() -> Quat {
    let corpse_rotation_min = 15.0_f32.to_radians();
    let mut rng = rand::rng();
    loop {
        let angle = rng.random_range(0.0..TAU);
        if angle >= corpse_rotation_min && angle <= TAU - corpse_rotation_min {
            return Quat::from_rotation_z(angle);
        }
    }
}

#[allow(unused_parens)]
pub fn apply_corpse_pose_after_gpos_change(
    mut cmd: Commands,
    mut query: Query<
        (Entity, Option<&mut CorpsePose>, ),
        (
            common::AnyDisabling,
            ExcludedComps,
            Or<(Added<Dead>, (With<Dead>, Without<CorpsePose>))>,
            With<Being>, With<Dead>,
        ),
    >,
) {
    for (being_ent, pose_opt, ) in query.iter_mut() {
        if pose_opt.is_some() {
            continue;
        }

        let rotation = corpse_rotation();
        let offset = rotation * Vec3::new(0.0, -16.0, 0.0);
        let pose = CorpsePose { rotation, offset };
        cmd.entity(being_ent).try_insert(pose);
    }
}

pub fn update_corpse_transform(
    mut query: Query<(&mut Transform, &CorpsePose, ), (Changed<CorpsePose>, With<Dead>, ExcludedComps)>,
) {
    for (mut transform, pose, ) in query.iter_mut() {
        transform.rotation = pose.rotation;
        transform.translation += pose.offset;
    }
}

#[allow(unused_parens)]
pub fn remove_corpse_pose_on_removal_of_dead(
    mut cmd: Commands,
    mut removed: RemovedComponents<Dead>,
) {
    for entity in removed.read() {
        cmd.entity(entity).try_remove::<CorpsePose>();
    }
}
