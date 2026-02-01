use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use tilemap::terrain_gen::terrgen_components::{FnlNoiseComp, Terrgen};
use tilemap::terrain_gen::terrgen_operaton_list_components::*;
use ::tilemap_shared::*;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn terrgen_editor_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut selected_entities: ResMut<DebugSelectedEntities>,
    mut queries: ParamSet<(
        Query<(Entity, Option<&Name>, &OperationList)>,
        Query<&mut OperationList>,
        Query<&mut FnlNoiseComp, With<Terrgen>>,
        Query<(Entity, Option<&Name>, &FnlNoiseComp), With<Terrgen>>,
    )>,
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

    // Pre-collect noise data to avoid borrow conflicts
    let noise_data: Vec<(Entity, String)> = queries.p3().iter()
        .map(|(ent, name, _)| {
            let label = if let Some(n) = name {
                format!("{}", n)
            } else {
                format!("{:?}", ent)
            };
            (ent, label)
        })
        .collect();

    // Pre-collect operationlist data
    let operationlist_vec: Vec<(Entity, String)> = queries.p0().iter()
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
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Terrgen Editor");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✖").clicked() {
                        window_visible.terrgen_editor = false;
                    }
                });
            });
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

            // Side-by-side layout for editors
            ui.columns(2, |columns| {
                // LEFT COLUMN: OperationList Editor
                columns[0].heading("Operations:");

                if let Some(oplist_entity) = selected_entities.selected_operationlist {
                    if let Ok(mut oplist) = queries.p1().get_mut(oplist_entity) {
                        let trunk_len = oplist.trunk.len();
                        let mut remove_op_idx = None;

                        for idx in 0..trunk_len {
                            let op_str = oplist.trunk[idx].0.as_ref().to_string();
                            let var_idx = oplist.trunk[idx].2;

                            columns[0].horizontal(|ui| {
                                ui.label(format!("Op {}: {} (Var{})", idx, op_str, var_idx));

                                if idx > 0 && ui.button("⬆").clicked() {
                                    oplist.trunk.swap(idx, idx - 1);
                                }

                                if idx < trunk_len - 1 && ui.button("⬇").clicked() {
                                    oplist.trunk.swap(idx, idx + 1);
                                }

                                if ui.button("✕").clicked() {
                                    remove_op_idx = Some(idx);
                                }
                            });

                            columns[0].horizontal(|ui| {
                                ui.label("  Operands:");
                                if ui.button("+ Add").clicked() {
                                    oplist.trunk[idx].1.push(Operand {
                                        complement: false,
                                        element: OperandElement::default(),
                                    });
                                }
                            });

                            let op_count = oplist.trunk[idx].1.len();
                            let mut remove_opd_idx = None;

                            for opd_idx in 0..op_count {
                                let mut removed = false;
                                columns[0].horizontal(|ui| {
                                    if ui.button("✕").clicked() {
                                        removed = true;
                                    }

                                    ui.checkbox(&mut oplist.trunk[idx].1[opd_idx].complement, "Complement");
                                    ui.label(format!("Opd {}:", opd_idx));

                                    let current_type = match &oplist.trunk[idx].1[opd_idx].element {
                                        OperandElement::Value(_) => "Value",
                                        OperandElement::StackArray(_) => "StackArray",
                                        OperandElement::NoiseEntity(_, _, _, _) => "NoiseEntity",
                                        OperandElement::HashPos(_) => "HashPos",
                                        OperandElement::PoissonDisk(_) => "PoissonDisk",
                                    };

                                    let combo_id = (idx as u64) * 1000 + (opd_idx as u64);
                                    egui::ComboBox::from_id_salt(combo_id)
                                        .selected_text(current_type)
                                        .width(100.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::Value(0.0), "Value");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::StackArray(0), "StackArray");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::NoiseEntity(Entity::PLACEHOLDER, Default::default(), false, 0), "NoiseEntity");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::HashPos(0), "HashPos");
                                            ui.selectable_value(&mut oplist.trunk[idx].1[opd_idx].element, OperandElement::PoissonDisk(PoissonDisk::new(1, 0).unwrap_or_default()), "PoissonDisk");
                                        });
                                });

                                columns[0].horizontal(|ui| {
                                    if ui.button("✕").clicked() {
                                        removed = true;
                                    }

                                    match &mut oplist.trunk[idx].1[opd_idx].element {
                                        OperandElement::Value(v) => {
                                            ui.label("Val:");
                                            ui.add(egui::DragValue::new(v).speed(0.1));
                                        }
                                        OperandElement::StackArray(idx_val) => {
                                            ui.label("Idx:");
                                            ui.add(egui::DragValue::new(idx_val).speed(1.0));
                                        }
                                        OperandElement::NoiseEntity(entity_ref, _range, _complementary, seed) => {
                                            let noise_label = noise_data
                                                .iter()
                                                .find(|(ent, _)| *ent == *entity_ref)
                                                .map(|(_, label)| label.clone())
                                                .unwrap_or_else(|| "None".to_string());

                                            let noise_combo_id = (idx as u64) * 10000 + (opd_idx as u64) * 100 + 50;
                                            egui::ComboBox::from_id_salt(noise_combo_id)
                                                .selected_text(&noise_label)
                                                .width(120.0)
                                                .show_ui(ui, |ui| {
                                                    for (noise_ent, noise_label) in noise_data.iter() {
                                                        ui.selectable_value(entity_ref, *noise_ent, noise_label);
                                                    }
                                                });

                                            ui.add(egui::DragValue::new(seed).speed(1.0).prefix("S:"));
                                        }
                                        OperandElement::HashPos(hash) => {
                                            ui.label("Hash:");
                                            ui.add(egui::DragValue::new(hash).speed(1.0));
                                        }
                                        OperandElement::PoissonDisk(_) => {
                                            ui.label("PoissonDisk");
                                        }
                                    }
                                });

                                if removed {
                                    remove_opd_idx = Some(opd_idx);
                                }
                            }

                            if let Some(opd_idx) = remove_opd_idx {
                                oplist.trunk[idx].1.remove(opd_idx);
                            }
                        }

                        if let Some(op_idx) = remove_op_idx {
                            oplist.trunk.remove(op_idx);
                        }

                        columns[0].separator();

                        if columns[0].button("+ Add Operation").clicked() {
                            oplist.trunk.push((Operation::Add, vec![], 0));
                        }

                        columns[0].separator();
                        columns[0].heading(format!("Bifurcations: {}", oplist.bifurcations.len()));
                        for (bif_idx, bifur) in oplist.bifurcations.iter().enumerate() {
                            if let Some(oplist_ent) = bifur.oplist {
                                columns[0].label(format!("Bif {}: OpList({:?}), Tiles: {}",
                                    bif_idx, oplist_ent, bifur.tiles.len()));
                            } else {
                                columns[0].label(format!("Bif {}: No OpList, Tiles: {}",
                                    bif_idx, bifur.tiles.len()));
                            }
                        }
                    }
                }

                // RIGHT COLUMN: Noise Component Editor
                columns[1].heading("Noise Component:");

                columns[1].separator();

                if let Some(noise_entity) = selected_entities.selected_noise {
                    if let Ok(mut noise_comp) = queries.p2().get_mut(noise_entity) {
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

                        columns[1].separator();
                        columns[1].heading("Noise Type");
                        let current_type = format!("{:?}", noise_comp.0.noise_type);
                        columns[1].label(format!("Current: {}", current_type));

                        columns[1].separator();
                        columns[1].heading("Fractal Settings");
                        let current_fractal = format!("{:?}", noise_comp.0.fractal_type);
                        columns[1].label(format!("Type: {}", current_fractal));

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
                        columns[1].heading("Cellular Settings");
                        let current_cellular_dist = format!("{:?}", noise_comp.0.cellular_distance_function);
                        columns[1].label(format!("Dist: {}", current_cellular_dist));

                        let current_cellular_return = format!("{:?}", noise_comp.0.cellular_return_type);
                        columns[1].label(format!("Return: {}", current_cellular_return));

                        columns[1].horizontal(|ui| {
                            ui.label("Jitter:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.cellular_jitter_modifier, 0.0..=2.0).step_by(0.01));
                        });

                        columns[1].separator();
                        columns[1].heading("Domain Warp");
                        let current_warp_type = format!("{:?}", noise_comp.0.domain_warp_type);
                        columns[1].label(format!("Type: {}", current_warp_type));

                        columns[1].horizontal(|ui| {
                            ui.label("Amplitude:");
                            ui.add(egui::Slider::new(&mut noise_comp.0.domain_warp_amp, 0.0..=2.0).step_by(0.01));
                        });
                    }
                }
            })
        });
}
