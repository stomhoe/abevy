use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use camera::camera_components::CameraTarget;
use ::being_shared::*;
use common::log_targets::DEBUG;
use tilemap_shared::{DimensionRef, GlobalTilePos, BeingsAtGpos};

use crate::debug_resources::{
    BeingTileClickInspectorState,
    DebugBeingNavUiState,
    DebugSelectedEntities,
    DubugWindowsVisibility,
};

fn cursor_being_pick_context(
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

pub(crate) fn set_being_click_picker_active(
    active: bool,
    state: &mut BeingTileClickInspectorState,
    window_visible: &mut DubugWindowsVisibility,
) {
    window_visible.being_tile_click_picker = active;
    if active {
        state.reset_inactivity_timer();
    } else {
        state.clear_selection();
        state.reset_inactivity_timer();
    }
}

#[allow(unused_parens, )]
pub fn capture_world_being_click_selection(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, (With<PrimaryWindow>, )>,
    camera_query: Query<(&Camera, &GlobalTransform, ), (Without<CameraTarget>, )>,
    camera_target_query: Query<&DimensionRef, (With<CameraTarget>, )>,
    beings_at_gpos: Res<BeingsAtGpos>,
    nav_ui_state: Res<DebugBeingNavUiState>,
    mut state: ResMut<BeingTileClickInspectorState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if nav_ui_state.track_new_being {
        return;
    }
    if !window_visible.being_tile_click_picker {
        state.clear_selection();
        state.reset_inactivity_timer();
        return;
    }
    if mouse.just_pressed(MouseButton::Right) {
        set_being_click_picker_active(false, &mut state, &mut window_visible);
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    if state.inactivity_timer.tick(time.delta()).just_finished() {
        set_being_click_picker_active(false, &mut state, &mut window_visible);
        return;
    }

    let Some((dim_ref, clicked_gpos)) = cursor_being_pick_context(
        &mut contexts,
        &mouse,
        &windows,
        &camera_query,
        &camera_target_query,
    ) else {
        return;
    };
    let mut beings = beings_at_gpos.get_beings_at_pos(dim_ref, clicked_gpos).to_vec();
    beings.sort_unstable_by_key(|entity| entity.index());
    let Some(selected_being) = select_next_being(
        beings.as_slice(),
        state.last_clicked_dim == Some(dim_ref.0) && state.last_clicked_gpos == Some(clicked_gpos),
        state.last_selected_being,
    ) else {
        return;
    };

    state.last_clicked_dim = Some(dim_ref.0);
    state.last_clicked_gpos = Some(clicked_gpos);
    state.last_selected_being = Some(selected_being);
    state.reset_inactivity_timer();

    select_being_details(selected_being, &mut selected_entities, &mut window_visible);
    debug!(
        target: DEBUG,
        "Being click picker selected {:?} at dim {:?} gpos {:?} from {} beings",
        selected_being,
        dim_ref,
        clicked_gpos,
        beings.len(),
    );
}

#[allow(unused_parens, )]
pub fn capture_world_being_nav_selection(
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, (With<PrimaryWindow>, )>,
    camera_query: Query<(&Camera, &GlobalTransform, ), (Without<CameraTarget>, )>,
    camera_target_query: Query<&DimensionRef, (With<CameraTarget>, )>,
    beings_at_gpos: Res<BeingsAtGpos>,
    nav_state_query: Query<(Entity, Has<BehavorialNavState>, ), (With<Being>, )>,
    mut nav_ui_state: ResMut<DebugBeingNavUiState>,
    mut nav_debug: ResMut<DebuggingBeingNav>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if !nav_ui_state.track_new_being {
        return;
    }

    let Some((dim_ref, clicked_gpos)) = cursor_being_pick_context(
        &mut contexts,
        &mouse,
        &windows,
        &camera_query,
        &camera_target_query,
    ) else {
        return;
    };
    let mut beings = beings_at_gpos.get_beings_at_pos(dim_ref, clicked_gpos).to_vec();
    beings.sort_unstable_by_key(|entity| entity.index());
    let Some(selected_being) = select_next_being(
        beings.as_slice(),
        nav_ui_state.last_clicked_dim == Some(dim_ref.0) && nav_ui_state.last_clicked_gpos == Some(clicked_gpos),
        nav_ui_state.last_selected_being,
    ) else {
        return;
    };

    let Ok((_, has_nav_state)) = nav_state_query.get(selected_being) else {
        return;
    };
    if !has_nav_state {
        nav_ui_state.last_clicked_dim = Some(dim_ref.0);
        nav_ui_state.last_clicked_gpos = Some(clicked_gpos);
        nav_ui_state.last_selected_being = Some(selected_being);
        return;
    }

    nav_ui_state.last_clicked_dim = Some(dim_ref.0);
    nav_ui_state.last_clicked_gpos = Some(clicked_gpos);
    nav_ui_state.last_selected_being = Some(selected_being);

    if nav_debug.track_being(selected_being) {
        window_visible.being_nav_log = true;
    }
}

pub(crate) fn select_next_being(
    beings: &[Entity],
    repeat_click_on_same_tile: bool,
    last_selected_being: Option<Entity>,
) -> Option<Entity> {
    let Some((&first_being, _)) = beings.split_first() else {
        return None;
    };
    if !repeat_click_on_same_tile {
        return Some(first_being);
    }

    let Some(last_selected_being) = last_selected_being else {
        return Some(first_being);
    };
    let Some(last_idx) = beings.iter().position(|&being| being == last_selected_being) else {
        return Some(first_being);
    };
    Some(beings[(last_idx + 1) % beings.len()])
}

fn select_being_details(
    entity: Entity,
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
) {
    selected_entities.selected_being = Some(entity);
    selected_entities.selected_being_bodypart = None;
    selected_entities.show_full_being_components = false;
    selected_entities.selected_exempted_entity = None;
    selected_entities.selected_tile = None;
    window_visible.tile_details = false;
    window_visible.being_details = true;
}
