use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use camera::camera_components::CameraTarget;
use sprite_shared::AcZ;
use std::cmp::Ordering;
use tilemap::tile::tile_components::TileStrId;
use tilemap::tile::tile_resources::TileRef;
use tilemap::tile::TileEntityMap;
use tilemap_shared::{DimensionRef, GlobalTilePos, TileGatheringParamSet};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility, WorldTileClickInspectorState};

const PICKER_GRID_RADIUS: i32 = 1;
const PICKER_GRID_SIDE: usize = 3;

#[allow(unused_parens)]
pub fn capture_world_tile_click_selection(
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), Without<CameraTarget>>,
    camera_target_query: Query<&DimensionRef, With<CameraTarget>>,
    mut tile_gathering: TileGatheringParamSet,
    acz_query: Query<&AcZ>,
    tile_ref_query: Query<&TileRef>,
    tile_map: Res<TileEntityMap>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut state: ResMut<WorldTileClickInspectorState>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if !state.enabled || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((dim_ref, center_gpos)) = cursor_world_pick_context(
        &mut contexts,
        &mouse,
        &windows,
        &camera_query,
        &camera_target_query,
    ) else {
        return;
    };

    state.clicked_dim = Some(dim_ref.0);
    state.clicked_gpos = Some(center_gpos);

    let center_tiles = sorted_tiles_at_gpos(
        dim_ref,
        center_gpos,
        &mut tile_gathering,
        &acz_query,
        &tile_ref_query,
        &tile_map,
    );
    if let Some(center_tile) = center_tiles.first().copied() {
        select_tile_for_details(selected_entities.as_mut(), window_visible.as_mut(), center_tile);
    } else {
        selected_entities.selected_tile = None;
        selected_entities.selected_exempted_entity = None;
    }

    selected_entities.selected_tiles.clear();
}

#[allow(unused_parens)]
pub fn world_tile_click_picker_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut state: ResMut<WorldTileClickInspectorState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut tile_gathering: TileGatheringParamSet,
    acz_query: Query<&AcZ>,
    tile_ref_query: Query<&TileRef>,
    tile_map: Res<TileEntityMap>,
    templ_str_id_query: Query<&TileStrId, With<game_common::game_common_components::Templ>>,
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
    let currently_selected_tile = selected_entities.selected_tile;

    egui::Window::new("🖱️ TileGpos Click Picker")
        .default_size([720.0, 520.0])
        .open(&mut open)
        .vscroll(true)
        .show(ctx, |ui| {
            let Some(clicked_dim) = state.clicked_dim else {
                ui.label("No tile selected yet.");
                return;
            };
            let Some(center_gpos) = state.clicked_gpos else {
                ui.label("No tile selected yet.");
                return;
            };

            ui.label(format!("Center -> dim {:?}, gpos {}", clicked_dim, center_gpos));
            ui.separator();

            egui::Grid::new("tile_picker_grid")
                .num_columns(PICKER_GRID_SIDE)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for row in 0..PICKER_GRID_SIDE {
                        for col in 0..PICKER_GRID_SIDE {
                            let gpos = grid_cell_gpos(center_gpos, row, col);
                            let mut tiles = sorted_tiles_at_gpos(
                                DimensionRef(clicked_dim),
                                gpos,
                                &mut tile_gathering,
                                &acz_query,
                                &tile_ref_query,
                                &tile_map,
                            );
                            let count = tiles.len();

                            ui.group(|ui| {
                                ui.set_min_size(egui::vec2(160.0, 150.0));
                                ui.vertical(|ui| {
                                    ui.label(format!("{} tile(s)", count));

                                    if tiles.is_empty() {
                                        ui.label("No tiles at this gpos.");
                                    } else {
                                        egui::ScrollArea::vertical()
                                            .max_height(92.0)
                                            .auto_shrink([false; 2])
                                            .show(ui, |ui| {
                                                for entity in tiles.drain(..) {
                                                    let label = tile_label(entity, &tile_ref_query, &tile_map, &templ_str_id_query);
                                                    let is_selected = currently_selected_tile == Some(entity);
                                                    if ui.selectable_label(is_selected, label).clicked() {
                                                        select_tile_for_details(selected_entities.as_mut(), window_visible.as_mut(), entity);
                                                    }
                                                }
                                            });
                                    }
                                });
                            });

                        }
                        ui.end_row();
                    }
                });

        });

    window_visible.world_tile_click_picker = open;
}

fn cursor_world_pick_context(
    contexts: &mut EguiContexts,
    mouse: &ButtonInput<MouseButton>,
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_query: &Query<(&Camera, &GlobalTransform), Without<CameraTarget>>,
    camera_target_query: &Query<&DimensionRef, With<CameraTarget>>,
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

fn sorted_tiles_at_gpos(
    dim_ref: DimensionRef,
    gpos: GlobalTilePos,
    tile_gathering: &mut TileGatheringParamSet,
    acz_query: &Query<&AcZ>,
    tile_ref_query: &Query<&TileRef>,
    tile_map: &Res<TileEntityMap>,
) -> Vec<Entity> {
    let mut tiles = tile_gathering.gather_tiles(dim_ref, gpos).to_vec();
    tiles.sort_unstable_by(|left, right| {
        let left_acz = effective_acz_for_entity(*left, acz_query, tile_ref_query, tile_map);
        let right_acz = effective_acz_for_entity(*right, acz_query, tile_ref_query, tile_map);
        right_acz
            .partial_cmp(&left_acz)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.index().cmp(&right.index()))
    });
    tiles.dedup();
    tiles
}

fn effective_acz_for_entity(
    entity: Entity,
    acz_query: &Query<&AcZ>,
    tile_ref_query: &Query<&TileRef>,
    tile_map: &Res<TileEntityMap>,
) -> f32 {
    if let Ok(tile_ref) = tile_ref_query.get(entity)
        && let Ok(templ_entity) = tile_map.0.get_cloned(tile_ref.0)
        && let Ok(acz) = acz_query.get(templ_entity)
    {
        return acz.0;
    }

    acz_query.get(entity).map(|acz| acz.0).ok().unwrap_or(f32::NEG_INFINITY)
}

fn tile_label(
    entity: Entity,
    tile_ref_query: &Query<&TileRef>,
    tile_map: &Res<TileEntityMap>,
    templ_str_id_query: &Query<&TileStrId, With<game_common::game_common_components::Templ>>,
) -> String {
    let Some(tile_ref) = tile_ref_query.get(entity).ok().copied() else {
        return format!("{:?}", entity);
    };
    let Some(templ_entity) = tile_map.0.get_cloned(tile_ref.0).ok() else {
        return format!("{:?}", entity);
    };
    let Some(tile_str_id) = templ_str_id_query.get(templ_entity).ok() else {
        return format!("{:?}", entity);
    };
    tile_str_id.as_str().to_string()
}

fn grid_cell_gpos(center_gpos: GlobalTilePos, row: usize, col: usize) -> GlobalTilePos {
    let dx = col as i32 - PICKER_GRID_RADIUS;
    let dy = PICKER_GRID_RADIUS - row as i32;
    GlobalTilePos(center_gpos.0 + IVec2::new(dx, dy))
}

fn select_tile_for_details(
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
    tile_entity: Entity,
) {
    selected_entities.selected_tile = Some(tile_entity);
    selected_entities.selected_exempted_entity = None;
    selected_entities.selected_tiles.clear();
    window_visible.tile_details = true;
}
