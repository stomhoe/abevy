use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_ecs_tilemap::tiles::TileColor;

use crate::being_details_inspector::part_label;
use camera::camera_components::CameraTarget;
use common::common_components::{DisplayName, StrId};
use sprite_shared::AcZ;
use std::cmp::Ordering;
use tilemap::tile::tile_components::TileStrId;
use tilemap::tile::tile_resources::TileRef;
use tilemap::tile::TileEntityMap;
use tilemap_shared::{BeingsAtGpos, DimensionRef, GlobalTilePos, TileGatheringParamSet};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility, WorldTileClickInspectorState};

const PICKER_MIN_SIDE: usize = 3;
const PICKER_MAX_SIDE: usize = 15;

#[allow(unused_parens)]
pub fn capture_world_tile_click_selection(
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), Without<CameraTarget>>,
    camera_target_query: Query<&DimensionRef, With<CameraTarget>>,
    beings_at_gpos: Res<BeingsAtGpos>,
    mut tile_gathering: TileGatheringParamSet,
    acz_query: Query<&AcZ>,
    tile_ref_query: Query<&TileRef>,
    tile_map: Res<TileEntityMap>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut state: ResMut<WorldTileClickInspectorState>,
    mut window_visible: ResMut<DubugWindowsVisibility>,
) {
    if !state.enabled || !state.picking_enabled || !mouse.just_pressed(MouseButton::Left) {
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
        state.picker_side,
        &mut tile_gathering,
        &acz_query,
        &tile_ref_query,
        &tile_map,
    );
    if center_tiles.first().is_none() {
        selected_entities.selected_tile = None;
        selected_entities.selected_exempted_entity = None;
    } else {
        if state.auto_open_tile_details {
            if let Some(center_tile) = center_tiles.first().copied() {
                select_tile_for_details(state.as_mut(), selected_entities.as_mut(), window_visible.as_mut(), center_tile);
            }
        }
        if state.auto_open_being_details {
            if let Some(center_being) = beings_at_gpos.get_beings_at_pos(dim_ref, center_gpos).first().copied() {
                select_being_for_details(state.as_mut(), selected_entities.as_mut(), window_visible.as_mut(), center_being);
            }
        }
    }

    if !state.mult_tile_windows {
        selected_entities.selected_tiles.clear();
    }
}

#[allow(unused_parens)]
pub fn click_picker_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut state: ResMut<WorldTileClickInspectorState>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut cmd: Commands,
    mut tile_gathering: TileGatheringParamSet,
    beings_at_gpos: Res<BeingsAtGpos>,
    acz_query: Query<&AcZ>,
    tile_ref_query: Query<&TileRef>,
    tile_map: Res<TileEntityMap>,
    display_name_query: Query<&DisplayName>,
    str_id_query: Query<&StrId>,
    templ_str_id_query: Query<&TileStrId, With<game_common::game_common_components::Templ>>,
    tile_color_query: Query<&TileColor>,
) {
    if !window_visible.click_picker {
        state.enabled = false;
        clear_center_highlight(&mut cmd, state.as_mut());
        return;
    }

    state.enabled = true;
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = window_visible.click_picker;
    let currently_selected_tile = selected_entities.selected_tile;
    let currently_selected_being = selected_entities.selected_being;
    let picker_side = state.picker_side.clamp(PICKER_MIN_SIDE, PICKER_MAX_SIDE);

    egui::Window::new("🖱️ Click Picker")
        .default_size([720.0, 520.0])
        .resizable([true, false])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut state.picking_enabled, "Picking");
                    if ui.checkbox(&mut state.mult_being_windows, "mult being windows").changed() {
                        if state.mult_being_windows {
                            if let Some(selected_being) = selected_entities.selected_being.or(selected_entities.selected_exempted_entity) {
                                selected_entities.selected_beings.insert(selected_being);
                            }
                        } else {
                            selected_entities.selected_beings.clear();
                        }
                    }
                    if ui.checkbox(&mut state.mult_tile_windows, "mult tile windows").changed() {
                        if state.mult_tile_windows {
                            if let Some(selected_tile) = selected_entities.selected_tile.or(selected_entities.selected_exempted_entity) {
                                selected_entities.selected_tiles.insert(selected_tile);
                            }
                        } else {
                            selected_entities.selected_tiles.clear();
                        }
                    }
                    ui.checkbox(&mut state.auto_open_being_details, "auto-open details of beings at clicked gpos");
                    ui.checkbox(&mut state.auto_open_tile_details, "auto-open details of tiles at clicked gpos");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.label("Picker size");
                        let mut picker_size = picker_side as u32;
                        if ui.add(egui::Slider::new(&mut picker_size, PICKER_MIN_SIDE as u32..=PICKER_MAX_SIDE as u32)).changed() {
                            let mut size = picker_size as usize;
                            if size % 2 == 0 {
                                size = size.saturating_add(1).min(PICKER_MAX_SIDE);
                            }
                            state.picker_side = size.clamp(PICKER_MIN_SIDE, PICKER_MAX_SIDE);
                        }
                        ui.label(format!("{} x {}", state.picker_side, state.picker_side));
                    });
                });
            });

            let Some(clicked_dim) = state.clicked_dim else {
                ui.separator();
                ui.label("-");
                clear_center_highlight(&mut cmd, state.as_mut());
                return;
            };
            let Some(center_gpos) = state.clicked_gpos else {
                ui.separator();
                ui.label("-");
                clear_center_highlight(&mut cmd, state.as_mut());
                return;
            };

            ui.separator();

            let grid_width = ui.available_width().max(1.0);
            let cell_size = (grid_width / picker_side as f32).max(1.0);
            let grid_size = egui::vec2(grid_width, cell_size * picker_side as f32);
            let (grid_rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
            let cell_spacing = 0.0;
            let cell_inner = cell_size - cell_spacing;

            for row in 0..picker_side {
                for col in 0..picker_side {
                    let gpos = grid_cell_gpos(center_gpos, row, col, picker_side);
                    let mut tiles = sorted_tiles_at_gpos(
                        DimensionRef(clicked_dim),
                        gpos,
                        picker_side,
                        &mut tile_gathering,
                        &acz_query,
                        &tile_ref_query,
                        &tile_map,
                    );
                    let mut beings = beings_at_gpos.get_beings_at_pos(DimensionRef(clicked_dim), gpos).to_vec();
                    beings.sort_unstable_by_key(|entity| entity.index());

                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            grid_rect.left() + col as f32 * cell_size,
                            grid_rect.top() + row as f32 * cell_size,
                        ),
                        egui::vec2(cell_inner, cell_inner),
                    );

                    ui.allocate_ui_at_rect(cell_rect, |ui| {
                        ui.set_min_size(egui::vec2(cell_inner, cell_inner));
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.vertical(|ui| {
                            egui::ScrollArea::vertical()
                                .max_height(cell_inner.max(48.0))
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.label("Tiles");
                                    if tiles.is_empty() {
                                        ui.label("-");
                                    } else {
                                        for entity in tiles.drain(..) {
                                            let label = tile_label(entity, &tile_ref_query, &tile_map, &templ_str_id_query);
                                            let is_selected = selected_entities.selected_tiles.contains(&entity) || currently_selected_tile == Some(entity);
                                            if ui.selectable_label(is_selected, label).clicked() {
                                                toggle_tile_selection(
                                                    state.as_mut(),
                                                    selected_entities.as_mut(),
                                                    window_visible.as_mut(),
                                                    entity,
                                                );
                                            }
                                        }
                                    }

                                    ui.label("Beings");
                                    if beings.is_empty() {
                                        ui.label("-");
                                    } else {
                                        for entity in beings.drain(..) {
                                            let label = part_label(
                                                entity,
                                                display_name_query.get(entity).ok(),
                                                str_id_query.get(entity).ok(),
                                            );
                                            let is_selected = selected_entities.selected_beings.contains(&entity) || currently_selected_being == Some(entity);
                                            if ui.selectable_label(is_selected, label).clicked() {
                                                toggle_being_selection(
                                                    state.as_mut(),
                                                    selected_entities.as_mut(),
                                                    window_visible.as_mut(),
                                                    entity,
                                                );
                                            }
                                        }
                                    }
                                });
                        });
                    });
                }
            }

            let center_tiles = sorted_tiles_at_gpos(DimensionRef(clicked_dim), center_gpos, picker_side, &mut tile_gathering, &acz_query, &tile_ref_query, &tile_map);
            sync_center_highlight(
                &mut cmd,
                state.as_mut(),
                center_tiles.first().copied(),
                &tile_color_query,
            );

            if state.mult_tile_windows || state.mult_being_windows {
                ui.separator();
                ui.label("Multi-select is active; details are shown in the combined details window.");
            }

        });

    window_visible.click_picker = open;
    if !open {
        state.enabled = false;
        clear_center_highlight(&mut cmd, state.as_mut());
    }
}

pub(crate) fn cursor_world_pick_context(
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
    picker_side: usize,
    tile_gathering: &mut TileGatheringParamSet,
    acz_query: &Query<&AcZ>,
    tile_ref_query: &Query<&TileRef>,
    tile_map: &Res<TileEntityMap>,
) -> Vec<Entity> {
    let _ = picker_side;
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

fn grid_cell_gpos(center_gpos: GlobalTilePos, row: usize, col: usize, picker_side: usize) -> GlobalTilePos {
    let radius = (picker_side / 2) as i32;
    let dx = col as i32 - radius;
    let dy = radius - row as i32;
    GlobalTilePos(center_gpos.0 + IVec2::new(dx, dy))
}

fn select_tile_for_details(
    state: &mut WorldTileClickInspectorState,
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
    tile_entity: Entity,
) {
    if state.mult_tile_windows {
        selected_entities.selected_tiles.insert(tile_entity);
        selected_entities.selected_tile = Some(tile_entity);
        window_visible.tile_details = true;
        return;
    }

    selected_entities.selected_tile = Some(tile_entity);
    selected_entities.selected_exempted_entity = None;
    selected_entities.selected_tiles.clear();
    window_visible.tile_details = true;
}

fn select_being_for_details(
    state: &mut WorldTileClickInspectorState,
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
    being_entity: Entity,
) {
    if state.mult_being_windows {
        selected_entities.selected_beings.insert(being_entity);
        selected_entities.selected_being = Some(being_entity);
        window_visible.being_details = true;
        return;
    }

    selected_entities.selected_being = Some(being_entity);
    selected_entities.selected_beings.clear();
    selected_entities.selected_being_interaction_zone = None;
    selected_entities.selected_being_bodypart = None;
    selected_entities.show_full_being_components = false;
    window_visible.being_details = true;
}

fn toggle_tile_selection(
    state: &mut WorldTileClickInspectorState,
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
    tile_entity: Entity,
) {
    if state.mult_tile_windows {
        if !selected_entities.selected_tiles.insert(tile_entity) {
            selected_entities.selected_tiles.remove(&tile_entity);
            selected_entities.selected_tile = selected_entities.selected_tiles.iter().next().copied();
        } else {
            selected_entities.selected_tile = Some(tile_entity);
        }
        window_visible.tile_details = !selected_entities.selected_tiles.is_empty();
        return;
    }

    if selected_entities.selected_tile == Some(tile_entity) {
        selected_entities.selected_tile = None;
        window_visible.tile_details = false;
    } else {
        selected_entities.selected_tile = Some(tile_entity);
        selected_entities.selected_tiles.clear();
        window_visible.tile_details = true;
    }
}

fn toggle_being_selection(
    state: &mut WorldTileClickInspectorState,
    selected_entities: &mut DebugSelectedEntities,
    window_visible: &mut DubugWindowsVisibility,
    being_entity: Entity,
) {
    if state.mult_being_windows {
        if !selected_entities.selected_beings.insert(being_entity) {
            selected_entities.selected_beings.remove(&being_entity);
            selected_entities.selected_being = selected_entities.selected_beings.iter().next().copied();
        } else {
            selected_entities.selected_being = Some(being_entity);
        }
        window_visible.being_details = !selected_entities.selected_beings.is_empty();
        return;
    }

    if selected_entities.selected_being == Some(being_entity) {
        selected_entities.selected_being = None;
        window_visible.being_details = false;
    } else {
        selected_entities.selected_being = Some(being_entity);
        selected_entities.selected_beings.clear();
        selected_entities.selected_being_interaction_zone = None;
        selected_entities.selected_being_bodypart = None;
        selected_entities.show_full_being_components = false;
        window_visible.being_details = true;
    }
}

fn clear_center_highlight(cmd: &mut Commands, state: &mut WorldTileClickInspectorState) {
    let Some(center_tile) = state.highlighted_center_tile.take() else {
        state.highlighted_center_tile_original_color = None;
        return;
    };

    if let Some(original_color) = state.highlighted_center_tile_original_color.take() {
        cmd.entity(center_tile).insert(original_color);
    } else {
        cmd.entity(center_tile).remove::<TileColor>();
    }
}

fn sync_center_highlight(
    cmd: &mut Commands,
    state: &mut WorldTileClickInspectorState,
    center_tile: Option<Entity>,
    tile_color_query: &Query<&TileColor>,
) {
    if state.highlighted_center_tile == center_tile {
        return;
    }

    clear_center_highlight(cmd, state);

    let Some(center_tile) = center_tile else {
        return;
    };

    state.highlighted_center_tile = Some(center_tile);
    state.highlighted_center_tile_original_color = tile_color_query.get(center_tile).copied().ok();
    cmd.entity(center_tile).insert(TileColor::from(Color::srgb(0.0, 1.0, 1.0)));
}
