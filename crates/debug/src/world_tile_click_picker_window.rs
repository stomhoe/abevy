use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use camera::camera_components::CameraTarget;
use common::common_components::StrId;
use game_common::game_common_components::EntityZeroRef;
use param_sets::EntitiesAtGposParamSet;
use sprite_shared::prelude::AcZ;
use std::cmp::Ordering;
use tilemap_shared::{DimensionRef, GlobalTilePos, TileGatheringParamSet};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility, WorldTileClickInspectorState};

#[allow(unused_parens)]
pub fn capture_world_tile_click_selection(
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), Without<CameraTarget>>,
    camera_target_query: Query<&DimensionRef, With<CameraTarget>>,
    entities_at_gpos: EntitiesAtGposParamSet,
    mut tile_gathering: TileGatheringParamSet,
    mut state: ResMut<WorldTileClickInspectorState>,
) {
    if !state.enabled || !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some((&dim_ref,)) = camera_target_query.iter().next().map(|dim_ref| (dim_ref,)) else {
        return;
    };
    let Some((camera, camera_transform)) = camera_query.iter().next() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    let clicked_gpos = GlobalTilePos::from(world_pos + GlobalTilePos::TILE_SIZE_PXS.as_vec2() * 0.5);

    state.clicked_dim = Some(dim_ref.0);
    state.clicked_gpos = Some(clicked_gpos);
    state.entities_at_gpos.clear();
    entities_at_gpos.gather_entities_at(&mut state.entities_at_gpos, dim_ref, clicked_gpos);
    state.entities_at_gpos.extend(
        tile_gathering
            .gather_tiles_at_to_drain(dim_ref, clicked_gpos)
            .iter()
            .copied(),
    );
    state.entities_at_gpos.sort_unstable_by_key(|entity| entity.index());
    state.entities_at_gpos.dedup();
}

#[allow(unused_parens)]
pub fn world_tile_click_picker_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut state: ResMut<WorldTileClickInspectorState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    strid_query: Query<&StrId>,
    ezero_ref_query: Query<&EntityZeroRef>,
    acz_query: Query<&AcZ>,
) {
    if !window_visible.world_tile_click_picker {
        state.enabled = false;
        return;
    }
    state.enabled = true;
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = window_visible.world_tile_click_picker;

    egui::Window::new("World Tile Click Picker")
        .default_size([520.0, 380.0])
        .open(&mut open)
        .show(ctx, |ui| {
            let Some(clicked_dim) = state.clicked_dim else {
                ui.label("No tile clicked yet.");
                return;
            };
            let Some(clicked_gpos) = state.clicked_gpos else {
                ui.label("No tile clicked yet.");
                return;
            };

            ui.label(format!("Last click -> dim {:?}, gpos {:?}", clicked_dim, clicked_gpos));
            ui.label(format!("Entities at tile: {}", state.entities_at_gpos.len()));
            ui.separator();

            state.entities_at_gpos.sort_unstable_by(|left, right| {
                let left_acz = acz_for_entity(*left, &acz_query, &ezero_ref_query);
                let right_acz = acz_for_entity(*right, &acz_query, &ezero_ref_query);
                right_acz
                    .partial_cmp(&left_acz)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| left.index().cmp(&right.index()))
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for &entity in &state.entities_at_gpos {
                    let strid_label = if let Ok(str_id) = strid_query.get(entity) {
                        str_id.as_str().to_string()
                    } else if let Ok(&EntityZeroRef(ezero_entity)) = ezero_ref_query.get(entity) {
                        if let Ok(str_id) = strid_query.get(ezero_entity) {
                            str_id.as_str().to_string()
                        } else {
                            "<no StrId>".to_string()
                        }
                    } else {
                        "<no StrId>".to_string()
                    };
                    let is_selected = selected_entities.selected_exempted_entity == Some(entity);
                    let row = format!("{}  ({:?})", strid_label, entity);
                    if ui.selectable_label(is_selected, row).clicked() {
                        selected_entities.selected_exempted_entity = Some(entity);
                        window_visible.exempted_entity_details = true;
                    }
                }
            });
        });

    window_visible.world_tile_click_picker = open;
    state.enabled = open;
}

fn acz_for_entity(
    entity: Entity,
    acz_query: &Query<&AcZ>,
    ezero_ref_query: &Query<&EntityZeroRef>,
) -> f32 {
    acz_query
        .get(entity)
        .map(|acz| acz.0)
        .ok()
        .or_else(|| {
            ezero_ref_query
                .get(entity)
                .ok()
                .and_then(|ezero_ref| acz_query.get(ezero_ref.0).ok().map(|acz| acz.0))
        })
        .unwrap_or(f32::NEG_INFINITY)
}
