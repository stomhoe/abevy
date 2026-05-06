use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};
use common::common_components::HashId;

use ::being_shared::{Being, LocalHumanControlled};
use ::tilemap_shared::{Dimension, DimensionRef};

use crate::debug_resources::DubugWindowsVisibility;

fn dimension_label(name: Option<&Name>, dim_ref: DimensionRef) -> String {
    name.map(|name| name.to_string())
        .unwrap_or_else(|| format!("{:?}", dim_ref))
}

#[allow(unused_parens, )]
pub fn dimension_changer_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    mut controlled_dim_query: Query<&mut DimensionRef, (With<Being>, LocalHumanControlled, )>,
    dimension_query: Query<(&HashId, Option<&Name>, ), (With<Dimension>, )>,
) {
    if !window_visible.dimension_changer {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let Ok(mut controlled_dim_ref) = controlled_dim_query.single_mut() else {
        let mut open = window_visible.dimension_changer;
        egui::Window::new("DimensionChanger")
            .default_size([360.0, 240.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("No local human-controlled being is available.");
            });
        window_visible.dimension_changer = open;
        return;
    };

    let current_dim_ref = *controlled_dim_ref;
    let mut dimensions_by_label: BTreeMap<String, DimensionRef> = BTreeMap::default();
    for (hash_id, name, ) in dimension_query.iter() {
        let dim_ref = DimensionRef(*hash_id);
        let label = format!("{} ({:?})", dimension_label(name, dim_ref), dim_ref);
        dimensions_by_label.insert(label, dim_ref);
    }

    let mut open = window_visible.dimension_changer;
    egui::Window::new("DimensionChanger")
        .default_size([420.0, 360.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("Current dimension: {:?}", current_dim_ref));
            ui.separator();

            if dimensions_by_label.is_empty() {
                ui.label("No dimensions are currently loaded.");
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (label, dim_ref) in dimensions_by_label.iter() {
                    let is_current = *dim_ref == current_dim_ref;
                    let response = ui.selectable_label(is_current, label);
                    if response.clicked() {
                        *controlled_dim_ref = *dim_ref;
                    }
                }
            });
        });
    window_visible.dimension_changer = open;
}
