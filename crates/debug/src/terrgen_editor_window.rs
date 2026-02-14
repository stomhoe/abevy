use bevy::prelude::*;
use bevy::math::Vec3Swizzles;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use tilemap::terrain_gen::terrgen_components::{FnlNoiseComp, Terrgen};
use tilemap::terrain_gen::terrgen_operaton_list_components::OperationList;
use tilemap::terrain_gen::terrgen_expression::Expr;
use camera::camera_components::CameraTarget;
use common::common_components::HashId;
use tilemap_shared::{DimensionRef, GlobalGenSettings, GlobalTilePos};
use crate::debug_resources::{
    DebugNoiseWorkshopState, DebugSelectedEntities, DubugWindowsVisibility, NoiseCombineOp,
};

/// Format an expression tree into a readable string representation
fn format_expr(expr: &Expr, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match expr {
        Expr::Literal(v) => format!("{:.3}", v),
        Expr::Noise { complement, seed_offset, .. } => {
            let comp = if *complement { "!" } else { "" };
            format!("{}Noise(seed: {}){}", prefix, seed_offset, comp)
        }
        Expr::NoiseByName { name, complement, .. } => {
            let comp = if *complement { "!" } else { "" };
            format!("{}NoiseByName({}){}", prefix, name, comp)
        }
        Expr::Variable { name } => format!("${}", name),
        Expr::Add { left, right } => {
            format!("({} + {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::Subtract { left, right } => {
            format!("({} - {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::Multiply { left, right } => {
            format!("({} * {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::Divide { left, right } => {
            format!("({} / {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::MultiplyOpo { value } => {
            format!("*opo({})", format_expr_compact(value))
        }
        Expr::Min { values } => {
            let args = values.iter()
                .map(|v| format_expr_compact(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("min({})", args)
        }
        Expr::Max { values } => {
            let args = values.iter()
                .map(|v| format_expr_compact(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("max({})", args)
        }
        Expr::Average { values } => {
            let args = values.iter()
                .map(|v| format_expr_compact(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("avg({})", args)
        }
        Expr::Abs { value } => {
            format!("abs({})", format_expr_compact(value))
        }
        Expr::MultiplyNormalized { left, right } => {
            format!("*nm({}, {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::MultiplyNormalizedAbs { left, right } => {
            format!("*nmabs({}, {})",
                format_expr_compact(left),
                format_expr_compact(right))
        }
        Expr::IndexMax { values } => {
            let args = values.iter()
                .map(|v| format_expr_compact(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("idxmax({})", args)
        }
        Expr::IndexNorm { value, multiplier } => {
            format!("idxnorm({}, {})",
                format_expr_compact(value),
                format_expr_compact(multiplier))
        }
        Expr::Linear { values } => {
            let args = values.iter()
                .map(|v| format_expr_compact(v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("lin({})", args)
        }
        Expr::Clamp { value, min, max } => {
            format!("clamp({}, {}, {})",
                format_expr_compact(value),
                format_expr_compact(min),
                format_expr_compact(max))
        }
        Expr::Complement { value } => {
            format!("!{}", format_expr_compact(value))
        }
        Expr::HashPos { seed } => format!("hp{}", seed),
        Expr::PoissonDisk { min_dist, seed } => format!("pd{}_{}", min_dist, seed),
    }
}

/// Format an expression compactly (single line, no indentation)
fn format_expr_compact(expr: &Expr) -> String {
    match expr {
        Expr::Literal(v) => format!("{:.3}", v),
        Expr::Noise { complement, .. } => {
            if *complement { "!noise" } else { "noise" }.to_string()
        }
        Expr::NoiseByName { name, complement, .. } => {
            if *complement { format!("!{}", name) } else { name.to_string() }
        }
        Expr::Variable { name } => format!("${}", name),
        Expr::Add { left, right } => {
            format!("({} + {})", format_expr_compact(left), format_expr_compact(right))
        }
        Expr::Subtract { left, right } => {
            format!("({} - {})", format_expr_compact(left), format_expr_compact(right))
        }
        Expr::Multiply { left, right } => {
            format!("({} * {})", format_expr_compact(left), format_expr_compact(right))
        }
        Expr::Divide { left, right } => {
            format!("({} / {})", format_expr_compact(left), format_expr_compact(right))
        }
        Expr::Min { values } => {
            let args = values.iter()
                .map(format_expr_compact)
                .collect::<Vec<_>>()
                .join(", ");
            format!("min({})", args)
        }
        Expr::Max { values } => {
            let args = values.iter()
                .map(format_expr_compact)
                .collect::<Vec<_>>()
                .join(", ");
            format!("max({})", args)
        }
        Expr::IndexMax { values } => {
            let args = values.iter()
                .map(format_expr_compact)
                .collect::<Vec<_>>()
                .join(", ");
            format!("idxmax({})", args)
        }
        Expr::Linear { values } => {
            let args = values.iter()
                .map(format_expr_compact)
                .collect::<Vec<_>>()
                .join(", ");
            format!("lin({})", args)
        }
        _ => format_expr(expr, 0),
    }
}

fn combine_values(values: &[f32], op: NoiseCombineOp) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    match op {
        NoiseCombineOp::Add => values.iter().copied().sum::<f32>().clamp(0.0, 1.0),
        NoiseCombineOp::Subtract => {
            let mut acc = values[0];
            for v in &values[1..] {
                acc -= *v;
            }
            acc.clamp(0.0, 1.0)
        }
        NoiseCombineOp::Multiply => values.iter().copied().product::<f32>().clamp(0.0, 1.0),
        NoiseCombineOp::Average => (values.iter().copied().sum::<f32>() / values.len() as f32).clamp(0.0, 1.0),
        NoiseCombineOp::Max => values.iter().copied().fold(f32::NEG_INFINITY, f32::max).clamp(0.0, 1.0),
        NoiseCombineOp::Min => values.iter().copied().fold(f32::INFINITY, f32::min).clamp(0.0, 1.0),
    }
}

fn snapshot_original_noises(
    workshop: &mut DebugNoiseWorkshopState,
    noise_data: &[(Entity, String)],
    noise_query: &Query<&FnlNoiseComp, With<Terrgen>>,
) {
    for (noise_ent, _) in noise_data {
        if workshop.original_noises.contains_key(noise_ent) {
            continue;
        }
        if let Ok(noise) = noise_query.get(*noise_ent) {
            workshop.original_noises.insert(*noise_ent, noise.clone());
        }
    }
}

#[allow(unused_parens)]
pub fn terrgen_editor_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut workshop: ResMut<DebugNoiseWorkshopState>,
    oplist_query: Query<(Entity, Option<&Name>, &OperationList)>,
    noise_name_query: Query<(Entity, Option<&Name>), With<Terrgen>>,
    mut noise_queries: ParamSet<(
        Query<&FnlNoiseComp, With<Terrgen>>,
        Query<&mut FnlNoiseComp, With<Terrgen>>,
    )>,
    camera_query: Query<(&DimensionRef, &GlobalTransform), With<CameraTarget>>,
    dim_hash_query: Query<&HashId>,
    gen_settings_query: Query<&GlobalGenSettings>,
) {
    if !window_visible.terrgen_editor {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let default_x = screen_rect.left() + 700.0;
    let default_y = screen_rect.top() + 10.0;
    let mut open = window_visible.terrgen_editor;

    // Pre-collect noise data to avoid borrow conflicts
    let noise_data: Vec<(Entity, String)> = noise_name_query.iter()
        .map(|(ent, name)| {
            let fallback = format!("{:?}", ent);
            let label = name
                .map(Name::as_str)
                .unwrap_or_default()
                .to_string();
            let label = if label.is_empty() { fallback } else { label };
            (ent, label)
        })
        .collect();
    snapshot_original_noises(&mut workshop, &noise_data, &noise_queries.p0());

    // Pre-collect operationlist data
    let operationlist_vec: Vec<(Entity, String)> = oplist_query.iter()
        .map(|(ent, name, _)| {
            let label = if let Some(n) = name {
                format!("{} ({:?})", n, ent)
            } else {
                format!("OperationList ({:?})", ent)
            };
            (ent, label)
        })
        .collect();

    egui::Window::new("Terrgen Editor")
        .default_pos([default_x, default_y])
        .resizable(true)
        .movable(true)
        .default_width(1200.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.heading("Terrgen Editor");
            ui.separator();

            // Select OperationList dropdown and Noise Component selector on same row
            let oplist_label = selected_entities.selected_operationlist
                .and_then(|ent| operationlist_vec.iter().find(|(e, _)| *e == ent).map(|(_, l)| l.clone()))
                .unwrap_or_else(|| "Select OperationList".to_string());

            let noise_label = selected_entities.selected_noise
                .and_then(|ent| noise_data.iter().find(|(e, _)| *e == ent).map(|(_, l)| l.clone()))
                .unwrap_or_else(|| "None".to_string());

            ui.horizontal(|ui| {
                ui.label("OperationList:");
                egui::ComboBox::from_id_salt(0u64)
                    .selected_text(&oplist_label)
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for (entity, label) in operationlist_vec.iter() {
                            ui.selectable_value(
                                &mut selected_entities.selected_operationlist,
                                Some(*entity),
                                label,
                            );
                        }
                    });

                ui.separator();

                ui.label("Noise:");
                egui::ComboBox::from_id_salt(999u64)
                    .selected_text(&noise_label)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        for (entity, label) in noise_data.iter() {
                            ui.selectable_value(
                                &mut selected_entities.selected_noise,
                                Some(*entity),
                                label,
                            );
                        }
                    });
            });

            ui.separator();

            // AST TREE DISPLAY (if available)
            if let Some(oplist_entity) = selected_entities.selected_operationlist {
                if let Ok((_, _, oplist)) = oplist_query.get(oplist_entity) {
                    let expr_tree = &oplist.expr_tree;
                        ui.heading("AST Expression Tree:");
                        ui.label(format!("Assignments: {}", expr_tree.assignments.len()));

                        egui::Frame::default()
                            .fill(egui::Color32::from_rgb(20, 20, 20))
                            .stroke(egui::Stroke { width: 1.0, color: egui::Color32::DARK_GRAY })
                            .inner_margin(egui::Margin { left: 8, right: 8, top: 4, bottom: 4 })
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // Display assignments
                                    for (i, assignment) in expr_tree.assignments.iter().enumerate() {
                                        let expr_str = format_expr_compact(&assignment.expr);
                                        ui.horizontal(|ui| {
                                            ui.label(format!("${}", assignment.name));
                                            ui.label("=");
                                            ui.label(egui::RichText::new(&expr_str).color(egui::Color32::from_rgb(100, 200, 255)));
                                        });
                                    }

                                    // Display output expression
                                    if !expr_tree.assignments.is_empty() {
                                        ui.separator();
                                    }
                                    let output_str = format_expr_compact(&expr_tree.output);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("out").color(egui::Color32::from_rgb(100, 255, 100)).strong());
                                        ui.label("=");
                                        ui.label(egui::RichText::new(&output_str).color(egui::Color32::from_rgb(255, 255, 100)));
                                    });
                                });
                            });

                        ui.separator();

                }
            }

            // Side-by-side layout for bifurcations and noise editing
            ui.columns(2, |columns| {
                columns[0].heading("Bifurcations");
                if let Some(oplist_entity) = selected_entities.selected_operationlist {
                    if let Ok((_, _, oplist)) = oplist_query.get(oplist_entity) {
                        if oplist.bifurcations.is_empty() {
                            columns[0].label("No bifurcations defined");
                        } else {
                            for (idx, bifurcation) in oplist.bifurcations.iter().enumerate() {
                                let child_info = bifurcation
                                    .oplist
                                    .map(|child| format!("Child: {:?}", child))
                                    .unwrap_or_else(|| "Child: None".to_string());
                                let tile_info = format!("Tiles: {}", bifurcation.tiles.len());
                                columns[0].vertical(|ui| {
                                    ui.label(format!("Branch {}", idx));
                                    ui.label(child_info);
                                    ui.label(tile_info);
                                });
                                columns[0].separator();
                            }
                        }
                    }
                } else {
                    columns[0].label("Select an OperationList to inspect");
                }

                columns[1].heading("Noise Component");
                if let Some(noise_entity) = selected_entities.selected_noise {
                    if let Ok(mut noise_comp) = noise_queries.p1().get_mut(noise_entity) {
                        columns[1].horizontal(|ui| {
                            if ui.button("Reset Selected").clicked() {
                                if let Some(original) = workshop.original_noises.get(&noise_entity) {
                                    *noise_comp = original.clone();
                                }
                            }
                        });
                        columns[1].separator();
                        columns[1].horizontal(|ui| {
                            ui.label("Seed:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.seed).speed(1));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Offset X:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.offset.x).speed(1));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Offset Y:");
                            ui.add(egui::DragValue::new(&mut noise_comp.0.offset.y).speed(1));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Frequency:");
                            let mut frequency = noise_comp.0.frequency();
                            if ui.add(egui::Slider::new(&mut frequency, 0.0001..=5.0).step_by(0.0001)).changed() {
                                noise_comp.0.set_frequency(Some(frequency));
                            }
                        });
                        columns[1].separator();
                        columns[1].label(format!("Noise Type: {:?}", noise_comp.0.noise_type));
                        columns[1].separator();
                        columns[1].label(format!("Fractal Type: {:?}", noise_comp.0.fractal_type));
                        columns[1].horizontal(|ui| {
                            ui.label("Octaves:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.octaves, 1..=10));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Lacunarity:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.lacunarity, 0.1..=4.0).step_by(0.01));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Gain:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.gain, 0.0..=1.0).step_by(0.01));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Weighted Strength:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.weighted_strength, 0.0..=1.0).step_by(0.01));
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Ping Pong:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.ping_pong_strength, 0.0..=4.0).step_by(0.01));
                        });
                        columns[1].separator();
                        columns[1].label(format!("Cellular Distance: {:?}", noise_comp.0.cellular_distance_function));
                        columns[1].label(format!("Cellular Return: {:?}", noise_comp.0.cellular_return_type));
                        columns[1].horizontal(|ui| {
                            ui.label("Jitter:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.cellular_jitter_modifier, 0.0..=2.0).step_by(0.01));
                        });
                        columns[1].separator();
                        columns[1].label(format!("Domain Warp: {:?}", noise_comp.0.domain_warp_type));
                        columns[1].horizontal(|ui| {
                            ui.label("Amplitude:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.domain_warp_amp, 0.0..=2.0).step_by(0.01));
                        });
                    } else {
                        columns[1].label("Selected noise component unavailable");
                    }
                } else {
                    columns[1].label("Select a noise component to edit");
                }
            });

            ui.separator();
            ui.heading("Noise Workshop Preview");
            ui.label("Combine multiple noises, apply per-noise subtract, and preview threshold highlights.");

            if let Some((camera_dim_ref, transform)) = camera_query.iter().next() {
                if let (Ok(dim_hash), Ok(gen_settings)) = (dim_hash_query.get(camera_dim_ref.0), gen_settings_query.single()) {
                    let camera_tile = GlobalTilePos::from(transform.translation().xy());

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Combine:");
                            egui::ComboBox::from_id_salt("noise_workshop_combine")
                                .selected_text(match workshop.combine_op {
                                    NoiseCombineOp::Add => "Add",
                                    NoiseCombineOp::Subtract => "Subtract",
                                    NoiseCombineOp::Multiply => "Multiply",
                                    NoiseCombineOp::Average => "Average",
                                    NoiseCombineOp::Max => "Max",
                                    NoiseCombineOp::Min => "Min",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Add, "Add");
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Subtract, "Subtract");
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Multiply, "Multiply");
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Average, "Average");
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Max, "Max");
                                    ui.selectable_value(&mut workshop.combine_op, NoiseCombineOp::Min, "Min");
                                });
                            ui.checkbox(&mut workshop.threshold_enabled, "Highlight > threshold");
                            ui.add(egui::Slider::new(&mut workshop.threshold, 0.0..=1.0).text("threshold"));
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label("preview size");
                            ui.add(egui::Slider::new(&mut workshop.preview_size_px, 160.0..=960.0));
                            ui.label("zoom (freq x)");
                            ui.add(egui::Slider::new(&mut workshop.preview_zoom, 0.1..=20.0));
                        });
                    });

                    ui.columns(2, |columns| {
                        columns[0].heading("Noises");
                        egui::ScrollArea::vertical().max_height(220.0).show(&mut columns[0], |ui| {
                            for (noise_ent, label) in &noise_data {
                                ui.horizontal(|ui| {
                                    let mut selected = workshop.selected_noises.contains(noise_ent);
                                    if ui.checkbox(&mut selected, "").changed() {
                                        if selected {
                                            workshop.selected_noises.push(*noise_ent);
                                        } else {
                                            workshop.selected_noises.retain(|e| e != noise_ent);
                                        }
                                    }
                                    ui.label(label);
                                    let subtract = workshop.per_noise_subtract.entry(*noise_ent).or_insert(0.0);
                                    ui.add(egui::Slider::new(subtract, -1.0..=1.0).text("subtract"));
                                });
                            }
                        });

                        columns[1].heading("Preview");
                        let res = workshop.preview_samples.max(8);
                        let size_px = workshop.preview_size_px.clamp(160.0, 960.0);
                        let cell = (size_px / res as f32).max(1.0);
                        let mut preview_settings = gen_settings.clone();
                        preview_settings.world_freq *= workshop.preview_zoom.clamp(0.1, 20.0);
                        let size = egui::vec2(size_px, size_px);
                        let (rect, _) = columns[1].allocate_exact_size(size, egui::Sense::hover());
                        let painter = columns[1].painter_at(rect);

                        if workshop.selected_noises.is_empty() {
                            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 20));
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "Select noises on the list",
                                egui::FontId::proportional(12.0),
                                egui::Color32::GRAY,
                            );
                        } else {
                            for py in 0..res {
                                for px in 0..res {
                                    let gx = px as i32;
                                    let gy = -(py as i32);
                                    let sample_pos = GlobalTilePos(camera_tile.0 + IVec2::new(gx, gy));
                                    let mut values = Vec::with_capacity(workshop.selected_noises.len());
                                    for noise_ent in &workshop.selected_noises {
                                        if let Ok(noise) = noise_queries.p0().get(*noise_ent) {
                                            let subtract = workshop.per_noise_subtract.get(noise_ent).copied().unwrap_or(0.0);
                                            let v = (noise.sample(
                                                sample_pos,
                                                *dim_hash,
                                                fnl::NoiseSampleRange::ZeroToOne,
                                                false,
                                                0,
                                                &preview_settings,
                                            ) - subtract)
                                            .clamp(0.0, 1.0);
                                            values.push(v);
                                        }
                                    }
                                    let combined = combine_values(&values, workshop.combine_op);
                                    let is_hot = workshop.threshold_enabled && combined > workshop.threshold;
                                    let base = (combined * 255.0) as u8;
                                    let fill = if is_hot {
                                        egui::Color32::from_rgb(255, base / 3, base / 3)
                                    } else {
                                        egui::Color32::from_rgb(base, base, base)
                                    };

                                    let x = rect.left() + px as f32 * cell;
                                    let y = rect.top() + py as f32 * cell;
                                    let r = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));
                                    painter.rect_filled(r, 0.0, fill);
                                }
                            }
                        }
                    });
                } else {
                    ui.label("Missing dimension hash or global generation settings for preview.");
                }
            } else {
                ui.label("CameraTarget not found, preview unavailable.");
            }
        });
    window_visible.terrgen_editor = open;
}
