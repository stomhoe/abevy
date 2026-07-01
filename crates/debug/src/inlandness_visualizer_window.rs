use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use bevy_replicon::prelude::{ClientState, ClientTriggerExt};

use ::being_shared::{Being, LocalHumanControlled};
use camera::camera_components::*;
use common::common_components::SettingsEntity;
use common::log_targets::DEBUG;
use tilemap::terrain::terrprobe::*;
use ::tilemap_shared::*;

use crate::debug_messages::ClientDebugTeleportBeingRequest;
use debug_shared::*;

const INLANDNESS_VISUALIZER_PROBE_ID: &str = "inlandness_visualizer_probe";
const INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE: u16 = 160;
const INLANDNESS_VISUALIZER_DEFAULT_TARGET_SAMPLE_POINTS: f32 = 12_000.0;
const INLANDNESS_VISUALIZER_DEFAULT_SAMPLE_AREA: f32 = 1.0;
const INLANDNESS_VISUALIZER_SAMPLE_AREA_RANGE: std::ops::RangeInclusive<f32> = 0.25..=25.0;

#[derive(Clone, Copy)]
pub struct ActiveTerrainProbe {
    requester: Entity,
    templ_ent: Entity,
    dimension_ref: DimensionRef,
    region_pos: RegionPos,
}

#[derive(Default)]
pub struct InlandnessVisualizerPreview {
    texture: Option<egui::TextureHandle>,
    image_size: [usize; 2],
    min_tile: IVec2,
    sample_step: IVec2,
    display_min: f32,
    display_max: f32,
    raw_min: f32,
    raw_max: f32,
    sampled_region: Option<(DimensionRef, RegionPos)>,
}

#[derive(Clone, Copy)]
pub struct InlandnessVisualizerControls {
    sampled_area: f32,
    target_sample_points: f32,
    step_size: u16,
    region_multiplier: f32,
    step_size_override: bool,
    region_multiplier_override: bool,
    auto_resample: bool,
    settings_dirty: bool,
}

impl Default for InlandnessVisualizerControls {
    fn default() -> Self {
        let (step_size, region_multiplier) = Self::defaults_for_sampled_area(
            INLANDNESS_VISUALIZER_DEFAULT_SAMPLE_AREA,
            INLANDNESS_VISUALIZER_DEFAULT_TARGET_SAMPLE_POINTS,
        );
        Self {
            sampled_area: INLANDNESS_VISUALIZER_DEFAULT_SAMPLE_AREA,
            target_sample_points: INLANDNESS_VISUALIZER_DEFAULT_TARGET_SAMPLE_POINTS,
            step_size,
            region_multiplier,
            step_size_override: false,
            region_multiplier_override: false,
            auto_resample: true,
            settings_dirty: false,
        }
    }
}

impl InlandnessVisualizerControls {
    fn defaults_for_sampled_area(sampled_area: f32, target_sample_points: f32) -> (u16, f32) {
        let sampled_area = sampled_area.max(0.0001);
        let target_sample_points = target_sample_points.max(1.0);
        let base_region_width = REGION_SIZE_IN_CHUNKS.x() as f32 * ChunkPos::CHUNK_SIZE.x as f32;
        let base_region_height = REGION_SIZE_IN_CHUNKS.y() as f32 * ChunkPos::CHUNK_SIZE.y as f32;
        let base_region_area = base_region_width * base_region_height;
        let base_region_multiplier = ((target_sample_points * INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE as f32 * INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE as f32) / base_region_area).sqrt();
        (
            (INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE as f32 * sampled_area).round().clamp(16.0, 400.0) as u16,
            (base_region_multiplier * sampled_area).clamp(1.0, 30.0),
        )
    }

    fn sync_linked_defaults(&mut self) -> bool {
        let (default_step_size, default_region_multiplier) = Self::defaults_for_sampled_area(self.sampled_area, self.target_sample_points);
        let mut changed = false;
        if !self.step_size_override && self.step_size != default_step_size {
            self.step_size = default_step_size;
            changed = true;
        }
        if !self.region_multiplier_override && (self.region_multiplier - default_region_multiplier).abs() > f32::EPSILON {
            self.region_multiplier = default_region_multiplier;
            changed = true;
        }
        changed
    }
}

impl InlandnessVisualizerPreview {
    fn clear(&mut self) {
        self.texture = None;
        self.sampled_region = None;
    }
}

struct TerrainPreviewBuild {
    image: egui::ColorImage,
    image_size: [usize; 2],
    min_tile: IVec2,
    sample_step: IVec2,
    display_min: f32,
    display_max: f32,
    raw_min: f32,
    raw_max: f32,
}

#[derive(SystemParam)]
pub struct InlandnessVisualizerResources<'w> {
    pub window_visible: ResMut<'w, DubugWindowsVisibility>,
    pub client_state: Res<'w, State<ClientState>>,
    pub terrprobe_entity_map: Res<'w, TerrProbeTemplEntityMap>,
    pub terrprobe_writer: MessageWriter<'w, TerrProbeJob>,
}

#[derive(SystemParam)]
pub struct InlandnessVisualizerQueries<'w, 's> {
    pub camera_dimension: Query<'w, 's, (Entity, &'static DimensionRef, &'static GlobalTransform, Option<&'static GlobalTilePos>), With<CameraTarget>>,
    pub debug_ui_config: Query<'w, 's, &'static DebugUiConfig, With<SettingsEntity>>,
    pub controlled_being_query: Query<'w, 's, Entity, (With<Being>, LocalHumanControlled)>,
    pub terrprobe_query: Query<'w, 's, &'static TerrProbeTempl, ()>,
    pub sampled_values_reader: MessageReader<'w, 's, SampledValuesCollected>,
    pub search_failed_reader: MessageReader<'w, 's, SearchFailed>,
}

#[derive(SystemParam)]
pub struct InlandnessVisualizerLocals<'s> {
    pub was_open: Local<'s, bool>,
    pub active_probe: Local<'s, Option<ActiveTerrainProbe>>,
    pub preview: Local<'s, InlandnessVisualizerPreview>,
    pub controls: Local<'s, InlandnessVisualizerControls>,
    pub pending_probes: Local<'s, Vec<TerrProbeJob>>,
    pub camera_region_prev: Local<'s, Option<(DimensionRef, RegionPos)>>,
    pub camera_tile_prev: Local<'s, Option<GlobalTilePos>>,
}

#[allow(unused_parens, )]
pub fn inlandness_visualizer_window(
    mut cmd: Commands,
    mut contexts: EguiContexts,
    resources: InlandnessVisualizerResources,
    queries: InlandnessVisualizerQueries,
    mut locals: InlandnessVisualizerLocals,
) {
    let InlandnessVisualizerResources {
        mut window_visible,
        client_state,
        terrprobe_entity_map,
        mut terrprobe_writer,
    } = resources;
    let InlandnessVisualizerQueries {
        camera_dimension,
        debug_ui_config,
        controlled_being_query,
        terrprobe_query,
        mut sampled_values_reader,
        mut search_failed_reader,
    } = queries;
    let InlandnessVisualizerLocals {
        was_open,
        active_probe,
        preview,
        controls,
        pending_probes,
        camera_region_prev,
        camera_tile_prev,
    } = &mut locals;
    let was_open = &mut **was_open;
    let camera_region_prev = &mut **camera_region_prev;
    let camera_tile_prev = &mut **camera_tile_prev;
    if !window_visible.inlandness_visualizer {
        if *was_open {
            if let Some(active) = active_probe.take() {
                cmd.entity(active.requester).try_despawn();
                cmd.entity(active.templ_ent).try_despawn();
            }
            preview.clear();
        }
        *was_open = false;
        *camera_region_prev = None;
        *camera_tile_prev = None;
        return;
    }

    let Ok(debug_ui_config) = debug_ui_config.single() else {
        return;
    };

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((camera_entity, &camera_dim_ref, camera_transform, camera_gpos_opt)) = camera_dimension.single() else {
        return;
    };
    let camera_tile_pos = camera_gpos_opt
        .copied()
        .unwrap_or_else(|| GlobalTilePos::from(camera_transform.translation().xy()));
    let camera_region_pos = camera_tile_pos.to_chunkpos().to_region_pos();
    let camera_region = (camera_dim_ref, camera_region_pos);
    let camera_has_gpos = camera_gpos_opt.is_some();
    let opening_now = !*was_open;
    let camera_moved = camera_tile_prev
        .as_ref()
        .map(|prev| *prev != camera_tile_pos)
        .unwrap_or(false);
    let region_changed = camera_region_prev
        .as_ref()
        .map(|prev| *prev != camera_region)
        .unwrap_or(false);
    *camera_region_prev = Some(camera_region);
    *camera_tile_prev = Some(camera_tile_pos);

    for sampled_values in sampled_values_reader.read() {
        if active_probe.as_ref().map(|p| p.requester) != Some(sampled_values.requester) {
            continue;
        }
        let Some(active) = active_probe.take() else {
            continue;
        };
        cmd.entity(active.requester).try_despawn();
        cmd.entity(active.templ_ent).try_despawn();
        if let Some(built) = build_terrain_preview(&sampled_values.matrix) {
            let texture_name = format!("inlandness_visualizer_{:?}_{:?}", active.dimension_ref.0, active.region_pos);
            preview.texture = Some(ctx.load_texture(
                texture_name,
                built.image,
                egui::TextureOptions::NEAREST,
            ));
            preview.image_size = built.image_size;
            preview.min_tile = built.min_tile;
            preview.sample_step = built.sample_step;
            preview.display_min = built.display_min;
            preview.display_max = built.display_max;
            preview.raw_min = built.raw_min;
            preview.raw_max = built.raw_max;
            preview.sampled_region = Some((active.dimension_ref, active.region_pos));
            debug!(target: DEBUG, "Inlandness visualizer sampled dim={:?} region={:?} points={}", active.dimension_ref.0, active.region_pos, sampled_values.matrix.values.len());
        } else {
            preview.clear();
            warn!(target: DEBUG, "Inlandness visualizer received empty sampled matrix for dim={:?} region={:?}", active.dimension_ref.0, active.region_pos);
        }
    }

    for failed in search_failed_reader.read() {
        if active_probe.as_ref().map(|p| p.requester) != Some(failed.0) {
            continue;
        }
        if let Some(active) = active_probe.take() {
            cmd.entity(active.requester).try_despawn();
            cmd.entity(active.templ_ent).try_despawn();
            warn!(target: DEBUG, "Inlandness visualizer probe failed for dim={:?} region={:?}", active.dimension_ref.0, active.region_pos);
        }
    }

    let mut needs_resample = opening_now || (controls.auto_resample && (camera_moved || region_changed));

    let mut open = window_visible.inlandness_visualizer;
    let screen_rect = ctx.content_rect();
    egui::Window::new("Inlandness Visualizer")
        .default_pos([screen_rect.left() + 420.0, screen_rect.top() + 120.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            let mut settings_changed = false;
            ui.label(format!("Camera region: {:?} dim {:?}", camera_region_pos, camera_dim_ref.0));
            if let Some((sample_dim, sample_region)) = preview.sampled_region {
                ui.label(format!("Preview region: {:?} dim {:?}", sample_region, sample_dim.0));
            } else {
                ui.label("Preview region: waiting for samples");
            }
            if active_probe.is_some() {
                ui.label("Sampling state: running");
            } else {
                ui.label("Sampling state: idle");
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.checkbox(&mut controls.auto_resample, "Auto-resample on move").changed() {
                    settings_changed = true;
                }
                if controls.auto_resample && controls.settings_dirty {
                    needs_resample = true;
                    controls.settings_dirty = false;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Area");
                let response = ui.add(
                    egui::Slider::new(&mut controls.sampled_area, INLANDNESS_VISUALIZER_SAMPLE_AREA_RANGE.clone())
                        .clamping(egui::SliderClamping::Always)
                        .show_value(true),
                );
                if response.changed() {
                    settings_changed = true;
                }
                let response = ui.add(
                    egui::DragValue::new(&mut controls.target_sample_points)
                        .range(1_000.0..=100_000.0)
                        .speed(50.0)
                        .suffix(" samples"),
                );
                if response.changed() {
                    settings_changed = true;
                }
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut controls.step_size_override, "Override step size").changed() {
                    settings_changed = true;
                }
                ui.add_enabled_ui(controls.step_size_override, |ui| {
                    let response = ui.add(
                        egui::Slider::new(&mut controls.step_size, 16..=400)
                            .clamping(egui::SliderClamping::Always)
                            .integer(),
                    );
                    if response.changed() {
                        settings_changed = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui.checkbox(&mut controls.region_multiplier_override, "Override region multiplier").changed() {
                    settings_changed = true;
                }
                ui.add_enabled_ui(controls.region_multiplier_override, |ui| {
                    let response = ui.add(
                        egui::Slider::new(&mut controls.region_multiplier, 1.0..=30.0)
                            .clamping(egui::SliderClamping::Always),
                    );
                    if response.changed() {
                        settings_changed = true;
                    }
                });
            });
            if controls.sync_linked_defaults() {
                settings_changed = true;
            }
            if settings_changed {
                if controls.auto_resample {
                    needs_resample = true;
                    controls.settings_dirty = false;
                } else {
                    controls.settings_dirty = true;
                }
            }
            ui.separator();
            let Some(texture) = preview.texture.as_ref() else {
                ui.label("No inlandness preview yet.");
                return;
            };
            let aspect = (preview.image_size[0] as f32 / preview.image_size[1] as f32).max(0.01);
            let mut map_w = ui.available_width().clamp(240.0, 1000.0);
            let mut map_h = (map_w / aspect).clamp(220.0, 760.0);
            if map_w / aspect > map_h {
                map_w = map_h * aspect;
            } else {
                map_h = map_w / aspect;
            }
            let (rect, response) = ui.allocate_exact_size(egui::vec2(map_w.max(120.0), map_h.max(120.0)), egui::Sense::click());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(18, 18, 18));
            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            render_region_bounds_and_camera_marker(
                &painter,
                rect,
                &preview,
                camera_region_pos,
                camera_tile_pos,
            );
            if response.clicked()
                && let Some(pointer_pos) = response.interact_pointer_pos()
                && rect.contains(pointer_pos)
                && let Some((sample_dim, _sample_region)) = preview.sampled_region
                && sample_dim == camera_dim_ref
            {
                let click_x = ((pointer_pos.x - rect.left()) / rect.width() * preview.image_size[0] as f32)
                    .floor()
                    .clamp(0.0, (preview.image_size[0].saturating_sub(1)) as f32) as usize;
                let click_y = ((pointer_pos.y - rect.top()) / rect.height() * preview.image_size[1] as f32)
                    .floor()
                    .clamp(0.0, (preview.image_size[1].saturating_sub(1)) as f32) as usize;
                let tile_x = preview.min_tile.x + (click_x as i32) * preview.sample_step.x;
                let tile_y = preview.min_tile.y + ((preview.image_size[1].saturating_sub(1) - click_y) as i32) * preview.sample_step.y;
                let target_gpos = GlobalTilePos(ivec2(tile_x, tile_y));
                cmd.entity(camera_entity).insert(Transform::from_translation(target_gpos.to_pixelpos().extend(0.0)));
                if camera_has_gpos {
                    cmd.entity(camera_entity).insert(target_gpos);
                }
                if *client_state.get() == ClientState::Connected && debug_ui_config.client_debug {
                    if let Ok(controlled_being_entity) = controlled_being_query.single() {
                        cmd.client_trigger(ClientDebugTeleportBeingRequest {
                            being_ent: controlled_being_entity,
                            gpos: target_gpos,
                        });
                    }
                }
            }
            ui.label(format!(
                "Legend: inlandness grayscale, robust range [{:.3}..{:.3}], raw range [{:.3}..{:.3}]",
                preview.display_min,
                preview.display_max,
                preview.raw_min,
                preview.raw_max
            ));
        });
    if needs_resample {
        preview.clear();
        let step_size = controls.step_size;
        let region_multiplier = controls.region_multiplier;
        queue_terrain_probe(
            &mut cmd,
            &terrprobe_entity_map,
            &terrprobe_query,
            &mut *active_probe,
            pending_probes,
            camera_dim_ref,
            camera_region_pos,
            camera_tile_pos,
            step_size,
            region_multiplier,
        );
        debug!(
            target: DEBUG,
            "Terrain visualizer resampled sampled_area={:.3} step_size={} region_multiplier={:.3} overrides(step={}, region={}) opening_now={} region_changed={}",
            controls.sampled_area,
            step_size,
            region_multiplier,
            controls.step_size_override,
            controls.region_multiplier_override,
            opening_now,
            region_changed
        );
    }
    terrprobe_writer.write_batch(pending_probes.drain(..));
    window_visible.inlandness_visualizer = open;
    *was_open = window_visible.inlandness_visualizer;
}

fn render_region_bounds_and_camera_marker(
    painter: &egui::Painter,
    rect: egui::Rect,
    preview: &InlandnessVisualizerPreview,
    region_pos: RegionPos,
    camera_tile_pos: GlobalTilePos,
) {
    let (base_min_chunk, base_max_chunk_excl) = region_pos.chunk_bounds();
    let base_min_tile = base_min_chunk.to_tilepos().0;
    let base_max_tile_excl = base_max_chunk_excl.to_tilepos().0;
    let min_tile = preview.min_tile;
    let step_x = preview.sample_step.x.max(1) as f32;
    let step_y = preview.sample_step.y.max(1) as f32;
    let image_w = preview.image_size[0].max(1) as f32;
    let image_h = preview.image_size[1].max(1) as f32;

    let map_x = |tile_x: i32| rect.left() + (((tile_x - min_tile.x) as f32 / step_x) / image_w) * rect.width();
    let map_y = |tile_y: i32| rect.bottom() - (((tile_y - min_tile.y) as f32 / step_y) / image_h) * rect.height();

    let region_rect = egui::Rect::from_min_max(
        egui::pos2(map_x(base_min_tile.x), map_y(base_max_tile_excl.y)),
        egui::pos2(map_x(base_max_tile_excl.x), map_y(base_min_tile.y)),
    );
    painter.rect_stroke(
        region_rect,
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
        egui::StrokeKind::Inside,
    );

    let camera_pos = camera_tile_pos.0;
    let camera_marker = egui::Rect::from_center_size(
        egui::pos2(map_x(camera_pos.x), map_y(camera_pos.y)),
        egui::vec2(4.0, 4.0),
    );
    painter.rect_filled(camera_marker, 0.0, egui::Color32::RED);
}

fn queue_terrain_probe(
    cmd: &mut Commands,
    terrprobe_entity_map: &Res<TerrProbeTemplEntityMap>,
    terrprobe_query: &Query<&TerrProbeTempl, ()>,
    active_probe: &mut Option<ActiveTerrainProbe>,
    pending_probes: &mut Vec<TerrProbeJob>,
    dimension_ref: DimensionRef,
    region_pos: RegionPos,
    camera_tile_pos: GlobalTilePos,
    step_size: u16,
    region_multiplier: f32,
) {
    let Ok(probe_templ_ent) = terrprobe_entity_map.0.get_cloned(INLANDNESS_VISUALIZER_PROBE_ID) else {
        error!(target: DEBUG, "Terrain visualizer missing terrprobe template '{}'", INLANDNESS_VISUALIZER_PROBE_ID);
        return;
    };
    let Ok(probe_templ) = terrprobe_query.get(probe_templ_ent) else {
        error!(target: DEBUG, "Terrain visualizer terrprobe entity {:?} has no TerrProbeTempl", probe_templ_ent);
        return;
    };
    if let Some(active) = active_probe.take() {
        cmd.entity(active.requester).despawn();
        cmd.entity(active.templ_ent).despawn();
    }
    let requester = cmd.spawn_empty().id();
    let mut probe_templ = probe_templ.clone();
    probe_templ.step_size = step_size;
    probe_templ.probe_pattern = ProbePattern::region(step_size, region_multiplier);
    let temp_probe_templ_ent = cmd.spawn(probe_templ.clone()).id();
    let mut probe = probe_templ.to_probe(temp_probe_templ_ent, dimension_ref, camera_tile_pos);
    probe.requester = requester;
    pending_probes.push(probe);
    *active_probe = Some(ActiveTerrainProbe {
        requester,
        templ_ent: temp_probe_templ_ent,
        dimension_ref,
        region_pos,
    });
    debug!(target: DEBUG, "Terrain visualizer queued probe requester={:?} dim={:?} region={:?}", requester, dimension_ref.0, region_pos);
}

fn build_terrain_preview(sampled_values: &SampledValues) -> Option<TerrainPreviewBuild> {
    if sampled_values.values.is_empty() {
        return None;
    }
    let mut min_tile = GlobalTilePos(IVec2::new(i32::MAX, i32::MAX));
    let mut max_tile_excl = GlobalTilePos(IVec2::new(i32::MIN, i32::MIN));
    let mut min_sample_val = f32::INFINITY;
    let mut max_sample_val = f32::NEG_INFINITY;
    let mut sample_values = Vec::with_capacity(sampled_values.values.len());
    for (sample_pos, sample_val_opt) in sampled_values.iter() {
        min_tile.0.x = min_tile.0.x.min(sample_pos.0.x);
        min_tile.0.y = min_tile.0.y.min(sample_pos.0.y);
        max_tile_excl.0.x = max_tile_excl.0.x.max(sample_pos.0.x + 1);
        max_tile_excl.0.y = max_tile_excl.0.y.max(sample_pos.0.y + 1);
        let Some(sample_val) = sample_val_opt else {
            continue;
        };
        min_sample_val = min_sample_val.min(sample_val);
        max_sample_val = max_sample_val.max(sample_val);
        sample_values.push(sample_val);
    }
    if !min_sample_val.is_finite() || !max_sample_val.is_finite() {
        min_sample_val = 0.0;
        max_sample_val = 0.0;
    }
    let (display_min, display_max) = robust_display_range(&mut sample_values, min_sample_val, max_sample_val);
    let (step_x, step_y) = estimate_sample_step_tiles(sampled_values);
    let width = ((max_tile_excl.0.x - min_tile.0.x + step_x - 1) / step_x).max(1) as usize;
    let height = ((max_tile_excl.0.y - min_tile.0.y + step_y - 1) / step_y).max(1) as usize;
    let mut image = egui::ColorImage::new(
        [width, height],
        vec![egui::Color32::from_rgb(18, 18, 18); width.saturating_mul(height)],
    );
    let set_cell = |img: &mut egui::ColorImage, cx: usize, cy: usize, color: egui::Color32| {
        if cx < width && cy < height {
            img.pixels[cy * width + cx] = color;
        }
    };
    for (sample_pos, sample_val_opt) in sampled_values.iter() {
        let Some(sample_val) = sample_val_opt else {
            continue;
        };
        let cell_x = ((sample_pos.0.x - min_tile.0.x) / step_x).max(0) as usize;
        let cell_y = height
            .saturating_sub(1)
            .saturating_sub(((sample_pos.0.y - min_tile.0.y) / step_y).max(0) as usize);
        set_cell(
            &mut image,
            cell_x,
            cell_y,
            sample_value_color(sample_val, display_min, display_max),
        );
    }
    Some(TerrainPreviewBuild {
        image,
        image_size: [width, height],
        min_tile: min_tile.0,
        sample_step: IVec2::new(step_x, step_y),
        display_min,
        display_max,
        raw_min: min_sample_val,
        raw_max: max_sample_val,
    })
}

fn robust_display_range(values: &mut [f32], fallback_min: f32, fallback_max: f32) -> (f32, f32) {
    if values.is_empty() {
        return (fallback_min, fallback_max);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let last = values.len().saturating_sub(1);
    let lo_i = ((last as f32) * 0.02).round() as usize;
    let hi_i = ((last as f32) * 0.98).round() as usize;
    let lo = values[lo_i.min(last)];
    let hi = values[hi_i.min(last)];
    if (hi - lo).abs() < f32::EPSILON {
        (fallback_min, fallback_max)
    } else {
        (lo, hi)
    }
}

fn sample_value_color(value: f32, min_val: f32, max_val: f32) -> egui::Color32 {
    let t = if (max_val - min_val).abs() <= f32::EPSILON {
        1.0
    } else {
        ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    };
    let g = (t * 255.0) as u8;
    egui::Color32::from_rgb(g, g, g)
}

fn estimate_sample_step_tiles(sampled_values: &SampledValues) -> (i32, i32) {
    let mut xs: Vec<i32> = sampled_values.iter().map(|(p, _)| p.0.x).collect();
    let mut ys: Vec<i32> = sampled_values.iter().map(|(p, _)| p.0.y).collect();
    xs.sort_unstable();
    ys.sort_unstable();
    xs.dedup();
    ys.dedup();
    let step_x = xs
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(1)
        .max(1);
    let step_y = ys
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|d| *d > 0)
        .min()
        .unwrap_or(1)
        .max(1);
    (step_x, step_y)
}
