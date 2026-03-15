use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use camera::camera_components::CameraTarget;
use bevy_ecs_tilemap::tiles::TileFlip;
use game_common::game_common_components::EntityZeroRef;
use std::collections::HashSet;
use tilemap_shared::{BeingsAtGpos, CardinalDirection, DimensionRef, GlobalTilePos, ItemsAtGpos, TiledCollisionMask, TileGatheringParamSet, WalkSpeedMultIfOnTop};

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

pub struct GposMapsUiState {
    radius: i32,
    cell_px: f32,
    fit_grids_to_window: bool,
    follow_camera_target: bool,
    center_dim: Entity,
    center_x: i32,
    center_y: i32,
}
impl Default for GposMapsUiState {
    fn default() -> Self {
        Self {
            radius: 12,
            cell_px: 16.0,
            fit_grids_to_window: true,
            follow_camera_target: true,
            center_dim: Entity::PLACEHOLDER,
            center_x: 0,
            center_y: 0,
        }
    }
}

fn paint_grid(
    ui: &mut egui::Ui,
    title: &str,
    radius: i32,
    cell_px: f32,
    camera_local: Option<GlobalTilePos>,
    count_at: impl Fn(GlobalTilePos) -> usize,
) -> Option<GlobalTilePos> {
    ui.label(title);
    let side = (radius * 2 + 1).max(1) as usize;
    let grid_size = egui::vec2(side as f32 * cell_px, side as f32 * cell_px);
    let (rect, _) = ui.allocate_exact_size(grid_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let mut clicked = None;

    for row in 0..side {
        for col in 0..side {
            let dx = col as i32 - radius;
            let dy = radius - row as i32;
            let gpos = GlobalTilePos::new(dx, dy);
            let count = count_at(gpos);
            let x = rect.left() + col as f32 * cell_px;
            let y = rect.top() + row as f32 * cell_px;
            let cell_rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_px, cell_px));
            let id = ui.make_persistent_id((title, row, col));
            let response = ui.interact(cell_rect, id, egui::Sense::click());
            let fill = if count == 0 {
                egui::Color32::from_rgb(18, 18, 18)
            } else {
                let heat = (count.min(9) as u8) * 24;
                egui::Color32::from_rgb(50 + heat, 40, 40)
            };
            painter.rect_filled(cell_rect, 0.0, fill);
            painter.rect_stroke(
                cell_rect,
                0.0,
                egui::Stroke::new(0.5, egui::Color32::from_gray(40)),
                egui::StrokeKind::Inside,
            );
            if camera_local == Some(gpos) {
                painter.rect_stroke(
                    cell_rect.shrink(0.5),
                    0.0,
                    egui::Stroke::new(1.5, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );
            }
            if count > 0 && cell_px >= 12.0 {
                painter.text(
                    cell_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    count.to_string(),
                    egui::FontId::proportional(10.0),
                    egui::Color32::WHITE,
                );
            }
            if response.clicked() {
                clicked = Some(gpos);
            }
        }
    }
    clicked
}

pub fn gpos_maps_window_system(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected: ResMut<DebugSelectedEntities>,
    beings_at_gpos: Res<BeingsAtGpos>,
    items_at_gpos: Res<ItemsAtGpos>,
    mut tile_gathering: TileGatheringParamSet,
    tile_instance_query: Query<(&EntityZeroRef, &GlobalTilePos, Option<&TileFlip>, Option<&CardinalDirection>)>,
    walk_speed: Query<&WalkSpeedMultIfOnTop>,
    tile_collision_masks: Query<&TiledCollisionMask>,
    camera_target_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    being_dim_pos: Query<(&DimensionRef, &Transform)>,
    mut terrain_blocked: Local<HashSet<(i32, i32)>>,
    mut ui_state: Local<GposMapsUiState>,
) {
    if !window_visible.gpos_maps {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut open = window_visible.gpos_maps;

    if ui_state.follow_camera_target {
        if let Ok((&dim_ref, gtf)) = camera_target_query.single() {
            ui_state.center_dim = dim_ref.0;
            let center = GlobalTilePos::from(gtf.translation().xy());
            ui_state.center_x = center.0.x;
            ui_state.center_y = center.0.y;
        } else if let Some(being_ent) = selected.selected_being {
            if let Ok((&dim_ref, transform)) = being_dim_pos.get(being_ent) {
                ui_state.center_dim = dim_ref.0;
                let center = GlobalTilePos::from(transform.translation.xy());
                ui_state.center_x = center.0.x;
                ui_state.center_y = center.0.y;
            }
        }
    }
    let camera_pos = camera_target_query
        .iter()
        .next()
        .map(|(dim_ref, gtf)| (*dim_ref, GlobalTilePos::from(gtf.translation().xy())));

    egui::Window::new("GPos Blocker Maps")
        .default_size([900.0, 520.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut ui_state.follow_camera_target, "Follow camera target");
                ui.checkbox(&mut ui_state.fit_grids_to_window, "Fit grids to window");
                ui.add(egui::Slider::new(&mut ui_state.radius, 4..=64).text("radius"));
                if !ui_state.fit_grids_to_window {
                    ui.add(egui::Slider::new(&mut ui_state.cell_px, 6.0..=28.0).text("cell px"));
                } else {
                    ui.label(format!("cell px: {:.1} (auto)", ui_state.cell_px));
                }
            });

            ui.horizontal(|ui| {
                ui.label("Dimension:");
                ui.label(format!("{:?}", ui_state.center_dim));
                ui.label("Center x:");
                ui.add(egui::DragValue::new(&mut ui_state.center_x).speed(1.0));
                ui.label("Center y:");
                ui.add(egui::DragValue::new(&mut ui_state.center_y).speed(1.0));
            });

            ui.separator();

            let center = GlobalTilePos::new(ui_state.center_x, ui_state.center_y);
            let dim_ref = DimensionRef(ui_state.center_dim);
            terrain_blocked.clear();
            for y in -ui_state.radius..=ui_state.radius {
                for x in -ui_state.radius..=ui_state.radius {
                    let gpos = center + GlobalTilePos::new(x, y);
                    let mut blocked = false;
                    for &tile_ent in tile_gathering.gather_tiles_at_to_drain(dim_ref, gpos) {
                        let Ok((ezero_ref, tile_origin, tile_flip, direction)) = tile_instance_query.get(tile_ent) else { continue; };
                        if walk_speed.get(ezero_ref.0).cloned().unwrap_or_default().is_extremely_low() {
                            blocked = true;
                            break;
                        }
                        if let Ok(mask) = tile_collision_masks.get(ezero_ref.0) {
                            if mask.is_solid_at_world_pos_with_flip(
                                *tile_origin,
                                gpos,
                                tile_flip.copied().unwrap_or_default(),
                                direction.copied().unwrap_or_default(),
                            ) {
                                blocked = true;
                                break;
                            }
                        }
                    }
                    if blocked {
                        terrain_blocked.insert((x, y));
                    }
                }
            }

            let camera_local = camera_pos.and_then(|(camera_dim_ref, camera_gpos)| {
                if camera_dim_ref != dim_ref {
                    return None;
                }
                let local = GlobalTilePos(camera_gpos.0 - center.0);
                if local.0.x < -ui_state.radius
                    || local.0.x > ui_state.radius
                    || local.0.y < -ui_state.radius
                    || local.0.y > ui_state.radius
                {
                    return None;
                }
                Some(local)
            });

            ui.columns(3, |cols| {
                let side = (ui_state.radius * 2 + 1).max(1) as f32;
                if ui_state.fit_grids_to_window {
                    let cell_from_col = ((cols[0].available_width() - 2.0) / side).clamp(4.0, 32.0);
                    ui_state.cell_px = cell_from_col;
                }
                let clicked_beings = paint_grid(&mut cols[0], "BeingsAtGpos (being occupancy)", ui_state.radius, ui_state.cell_px, camera_local, |local| {
                    let gpos = center + local;
                    beings_at_gpos.beings_at_pos(dim_ref, gpos).len()
                });
                let clicked_items = paint_grid(&mut cols[1], "ItemsAtGpos (item occupancy)", ui_state.radius, ui_state.cell_px, camera_local, |local| {
                    let gpos = center + local;
                    items_at_gpos.items_at_pos(dim_ref, gpos).len()
                });
                let clicked_terrain = paint_grid(&mut cols[2], "Terrain Blocking (mask/speed<=0.01)", ui_state.radius, ui_state.cell_px, camera_local, |local| {
                    if terrain_blocked.contains(&(local.0.x, local.0.y)) { 1 } else { 0 }
                });
                if let Some(local) = clicked_beings {
                    let entities = beings_at_gpos.beings_at_pos(dim_ref, center + local);
                    if let Some(being_entity) = entities.first().copied() {
                        selected.selected_being = Some(being_entity);
                        selected.selected_being_bodypart = None;
                        selected.show_full_being_components = false;
                        window_visible.being_details = true;
                    }
                }
                if let Some(local) = clicked_items {
                    let entities = items_at_gpos.items_at_pos(dim_ref, center + local);
                    if let Some(item_entity) = entities.first().copied() {
                        selected.selected_exempted_entity = Some(item_entity);
                        selected.selected_tile = None;
                        window_visible.tile_details = true;
                    }
                }
                if let Some(local) = clicked_terrain {
                    let gpos = center + local;
                    if let Some(tile_entity) = tile_gathering.gather_tiles_at_to_drain(dim_ref, gpos).first().copied() {
                        selected.selected_tile = Some(tile_entity);
                        window_visible.tile_details = true;
                    }
                }
            });
        });
    window_visible.gpos_maps = open;
}
