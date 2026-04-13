use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use common::log_targets::DEBUG;

use camera::camera_components::CameraTarget;
use game_common::game_common_components::TemplEntiRef;
use sprite_shared::AcZ;
use tilemap_shared::{DimensionRef, GlobalTilePos, SafeDespawn, TileGatheringParamSet};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility, TileClickRemoverState};

fn cursor_tile_pick_context(
    contexts: &mut EguiContexts,
    mouse: &ButtonInput<MouseButton>,
    windows: &Query<&Window, (With<PrimaryWindow>, )>,
    camera_query: &Query<(&Camera, &GlobalTransform, ), (Without<CameraTarget>, )>,
    camera_target_query: &Query<&DimensionRef, (With<CameraTarget>, )>,
) -> Option<(DimensionRef, GlobalTilePos)> {
    if !mouse.just_pressed(MouseButton::Left) {
        return None;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return None;
    };
    if ctx.wants_pointer_input() {
        return None;
    }
    let Ok(window) = windows.single() else {
        return None;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return None;
    };
    let Some(&dim_ref) = camera_target_query.iter().next() else {
        return None;
    };
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return None;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return None;
    };
    let clicked_gpos = GlobalTilePos::from(world_pos + GlobalTilePos::TILE_SIZE_PXS.as_vec2() * 0.5);
    Some((dim_ref, clicked_gpos))
}

pub(crate) fn set_tile_click_remover_active(
    active: bool,
    state: &mut TileClickRemoverState,
    window_visible: &mut DubugWindowsVisibility,
) {
    window_visible.tile_click_remover = active;
    state.reset_inactivity_timer();
}

#[allow(unused_parens, )]
pub fn capture_world_tile_click_removal(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, (With<PrimaryWindow>, )>,
    camera_query: Query<(&Camera, &GlobalTransform, ), (Without<CameraTarget>, )>,
    camera_target_query: Query<&DimensionRef, (With<CameraTarget>, )>,
    mut tile_gathering: TileGatheringParamSet,
    acz_query: Query<&AcZ>,
    templ_ref_query: Query<&TemplEntiRef>,
    mut state: ResMut<TileClickRemoverState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut despawn_writer: MessageWriter<SafeDespawn>,
    mut despawn_msgs: Local<Vec<SafeDespawn>>,
) {
    if !window_visible.tile_click_remover {
        state.reset_inactivity_timer();
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        set_tile_click_remover_active(false, &mut state, &mut window_visible);
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    if state.inactivity_timer.tick(time.delta()).just_finished() {
        set_tile_click_remover_active(false, &mut state, &mut window_visible);
        trace!(target: DEBUG, "Tile click remover auto-deactivated after 10s without despawning a tile");
        return;
    }

    let Some((dim_ref, clicked_gpos)) = cursor_tile_pick_context(
        &mut contexts,
        &mouse,
        &windows,
        &camera_query,
        &camera_target_query,
    ) else {
        return;
    };

    let mut tile_ents = tile_gathering.gather_tiles(dim_ref, clicked_gpos).to_vec();
    tile_ents.sort_unstable_by(|left, right| {
        let left_acz = acz_for_entity(*left, &acz_query, &templ_ref_query);
        let right_acz = acz_for_entity(*right, &acz_query, &templ_ref_query);
        right_acz
            .partial_cmp(&left_acz)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.index().cmp(&right.index()))
    });
    tile_ents.dedup();

    if tile_ents.is_empty() {
        return;
    }
    if tile_ents.len() == 1 && !state.despawn_last_tile {
        return;
    }

    let Some(tile_ent) = tile_ents.first().copied() else {
        return;
    };
    despawn_msgs.push(SafeDespawn {
        tile_ent,
        remove_u16_index: true,
    });
    state.reset_inactivity_timer();

    if selected_entities.selected_tile == Some(tile_ent) {
        selected_entities.selected_tile = None;
    }
    if selected_entities.selected_exempted_entity == Some(tile_ent) {
        selected_entities.selected_exempted_entity = None;
    }

    debug!(
        target: DEBUG,
        "Tile click remover despawned {:?} at dim {:?} gpos {:?} from {} tiles",
        tile_ent,
        dim_ref,
        clicked_gpos,
        tile_ents.len(),
    );

    despawn_writer.write_batch(despawn_msgs.drain(..));
}

fn acz_for_entity(
    entity: Entity,
    acz_query: &Query<&AcZ>,
    templ_ref_query: &Query<&TemplEntiRef>,
) -> f32 {
    acz_query
        .get(entity)
        .map(|acz| acz.0)
        .ok()
        .or_else(|| {
            templ_ref_query
                .get(entity)
                .ok()
                .and_then(|templ_ref| acz_query.get(templ_ref.0).ok().map(|acz| acz.0))
        })
        .unwrap_or(f32::NEG_INFINITY)
}
