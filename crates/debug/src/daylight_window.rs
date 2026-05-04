use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use camera::camera_components::CameraTarget;
use camera::DaylightSettings;
use tilemap_shared::{DimensionDaylightSeri, DimensionEntityMap, DimensionRef};

use crate::debug_resources::DubugWindowsVisibility;

fn edit_rgb(ui: &mut egui::Ui, label: &str, rgb: &mut [f32; 3]) {
    ui.horizontal(|ui| {
        ui.monospace(label);
        ui.add(egui::DragValue::new(&mut rgb[0]).speed(0.01));
        ui.add(egui::DragValue::new(&mut rgb[1]).speed(0.01));
        ui.add(egui::DragValue::new(&mut rgb[2]).speed(0.01));
    });
}

fn edit_scalar(ui: &mut egui::Ui, label: &str, value: &mut f32, speed: f64) {
    ui.horizontal(|ui| {
        ui.monospace(label);
        ui.add(egui::DragValue::new(value).speed(speed));
    });
}

#[allow(unused_parens)]
pub fn daylight_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    mut daylight_query: Query<&mut DimensionDaylightSeri>,
) {
    if !window_visible.daylight {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let screen_rect = ctx.content_rect();
    let mut open = window_visible.daylight;

    let Ok(camera_dimension) = camera_dimension.single() else {
        return;
    };
    let Ok(dimension_ent) = dimension_map.0.get_cloned(camera_dimension.0) else {
        return;
    };
    let Ok(mut daylight) = daylight_query.get_mut(dimension_ent) else {
        return;
    };
    daylight.normalize();
    let daylight_preview: DaylightSettings = (*daylight).into();

    egui::Window::new("Daylight")
        .default_pos([screen_rect.right() - 320.0, screen_rect.top() + 10.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("Day progress: {:.1}%", daylight_preview.day_progress() * 100.0));
            edit_scalar(ui, "Day length (min)", &mut daylight.day_length_minutes, 0.1);
            edit_scalar(ui, "Current time (min)", &mut daylight.time_of_day_minutes, 0.1);
            ui.separator();
            edit_rgb(ui, "Ambient color", &mut daylight.ambient_color_rgb);
            edit_rgb(ui, "Night color", &mut daylight.night_color_rgb);
            edit_rgb(ui, "Dawn/Dusk color", &mut daylight.dawn_dusk_color_rgb);
            edit_scalar(ui, "Day curve exponent", &mut daylight.day_curve_exponent, 0.01);
            edit_scalar(ui, "Dawn/Dusk curve exponent", &mut daylight.dawn_dusk_curve_exponent, 0.01);
            edit_scalar(ui, "Ambient min brightness factor", &mut daylight.ambient_min_brightness_factor, 0.01);
            edit_scalar(ui, "Ambient max brightness factor", &mut daylight.ambient_max_brightness_factor, 0.01);
            edit_scalar(ui, "Ambient brightness", &mut daylight.ambient_brightness, 0.01);

            daylight.normalize();
        });

    window_visible.daylight = open;
}