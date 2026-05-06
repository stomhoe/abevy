use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use camera::camera_components::*;
use common::log_targets::DEBUG;
use tilemap::terrain::terrprobe::*;
use ::tilemap_shared::*;

use crate::debug_resources::*;

const INLANDNESS_VISUALIZER_PROBE_ID: &str = "inlandness_visualizer_probe";
const INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE: u16 = 160;
const INLANDNESS_VISUALIZER_DEFAULT_REGION_MULTIPLIER: f32 = 15.0;

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
    step_size: u16,
    region_multiplier: f32,
}

impl Default for InlandnessVisualizerControls {
    fn default() -> Self {
        Self {
            step_size: INLANDNESS_VISUALIZER_DEFAULT_STEP_SIZE,
            region_multiplier: INLANDNESS_VISUALIZER_DEFAULT_REGION_MULTIPLIER,
        }
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

#[allow(unused_parens, )]
pub fn inlandness_visualizer_window(
    mut cmd: Commands,
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    camera_dimension: Query<(&DimensionRef, &GlobalTilePos, ), (With<CameraTarget>, )>,
    terrprobe_entity_map: Res<TerrProbeTemplEntityMap>,
    terrprobe_query: Query<&TerrProbeTempl, ()>,
    mut terrprobe_writer: MessageWriter<TerrProbeJob>,
    mut sampled_values_reader: MessageReader<SampledValuesCollected>,
    mut search_failed_reader: MessageReader<SearchFailed>,
    mut was_open: Local<bool>,
    mut active_probe: Local<Option<ActiveTerrainProbe>>,
    mut preview: Local<InlandnessVisualizerPreview>,
    mut controls: Local<InlandnessVisualizerControls>,
    mut pending_probes: Local<Vec<TerrProbeJob>>,
    mut camera_region_prev: Local<Option<(DimensionRef, RegionPos)>>,
) {
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
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok((&camera_dim_ref, &camera_tile_pos, )) = camera_dimension.single() else {
        return;
    };
    let camera_region_pos = camera_tile_pos.to_chunkpos().to_region_pos();
    let camera_region = (camera_dim_ref, camera_region_pos);
    let opening_now = !*was_open;
    let region_changed = camera_region_prev
        .as_ref()
        .map(|prev| *prev != camera_region)
        .unwrap_or(false);
    *camera_region_prev = Some(camera_region);

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

    let mut needs_resample = opening_now || region_changed;

    let mut open = window_visible.inlandness_visualizer;
    let screen_rect = ctx.content_rect();
    egui::Window::new("Inlandness Visualizer")
        .default_pos([screen_rect.left() + 420.0, screen_rect.top() + 120.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
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
                ui.label("Step size");
                let response = ui.add(
                    egui::Slider::new(&mut controls.step_size, 16..=256)
                        .clamping(egui::SliderClamping::Always)
                        .integer(),
                );
                needs_resample |= response.changed();
            });
            ui.horizontal(|ui| {
                ui.label("Region multiplier");
                let response = ui.add(
                    egui::Slider::new(&mut controls.region_multiplier, 1.0..=30.0)
                        .clamping(egui::SliderClamping::Always),
                );
                needs_resample |= response.changed();
            });
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
            let (rect, _) = ui.allocate_exact_size(egui::vec2(map_w.max(120.0), map_h.max(120.0)), egui::Sense::hover());
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
        queue_terrain_probe(
            &mut cmd,
            &terrprobe_entity_map,
            &terrprobe_query,
            &mut *active_probe,
            &mut pending_probes,
            camera_dim_ref,
            camera_region_pos,
            camera_tile_pos,
            controls.step_size,
            controls.region_multiplier,
        );
        debug!(
            target: DEBUG,
            "Terrain visualizer resampled step_size={} region_multiplier={:.3} opening_now={} region_changed={}",
            controls.step_size,
            controls.region_multiplier,
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
