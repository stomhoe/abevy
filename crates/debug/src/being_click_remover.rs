use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use common::log_targets::DEBUG;

use camera::camera_components::CameraTarget;
use tilemap_shared::DimensionRef;

use crate::click_picker_window::cursor_world_pick_context;
use debug_shared::{BeingClickRemoverState, DubugWindowsVisibility, DebugSelectedEntities};

pub(crate) fn set_being_click_remover_active(
    active: bool,
    state: &mut BeingClickRemoverState,
    window_visible: &mut DubugWindowsVisibility,
) {
    window_visible.being_click_remover = active;
    state.reset_inactivity_timer();
}

#[allow(unused_parens, )]
pub fn capture_world_being_click_removal(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), Without<CameraTarget>>,
    camera_target_query: Query<&DimensionRef, With<CameraTarget>>,
    beings_at_gpos: Res<tilemap_shared::BeingsAtGpos>,
    mut state: ResMut<BeingClickRemoverState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut cmd: Commands,
) {
    if !window_visible.being_click_remover {
        state.reset_inactivity_timer();
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        set_being_click_remover_active(false, &mut state, &mut window_visible);
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    if state.inactivity_timer.tick(time.delta()).just_finished() {
        set_being_click_remover_active(false, &mut state, &mut window_visible);
        trace!(target: DEBUG, "Being click remover auto-deactivated after 10s without despawning a being");
        return;
    }

    let Some((dim_ref, clicked_gpos)) = cursor_world_pick_context(
        &mut contexts,
        &mouse,
        &windows,
        &camera_query,
        &camera_target_query,
    ) else {
        return;
    };
    let Some(&being_ent) = beings_at_gpos.get_beings_at_pos(dim_ref, clicked_gpos).first() else {
        return;
    };

    cmd.entity(being_ent).try_despawn();
    state.reset_inactivity_timer();

    if selected_entities.selected_being == Some(being_ent) {
        selected_entities.selected_being = None;
    }

    debug!(
        target: DEBUG,
        "Being click remover despawned {:?} at dim {:?} gpos {:?}",
        being_ent,
        dim_ref,
        clicked_gpos,
    );
}
