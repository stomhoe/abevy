use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts};

use camera::camera_components::CameraTarget;
use common::common_components::SettingsEntity;
use tilemap_shared::{DirectionalLight2dOverride, DimensionDaylightRuntime, DimensionDaylightSeri, DimensionEntityMap, DimensionRef};
use time::SimTimeScale;

use debug_shared::DubugWindowsVisibility;

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

fn edit_vec2(ui: &mut egui::Ui, label: &str, value: &mut [f32; 2], speed: f64) {
    ui.horizontal(|ui| {
        ui.monospace(label);
        ui.add(egui::DragValue::new(&mut value[0]).speed(speed));
        ui.add(egui::DragValue::new(&mut value[1]).speed(speed));
    });
}

#[allow(unused_parens)]
pub fn daylight_window(
    mut contexts: EguiContexts,
    mut window_visible: ResMut<DubugWindowsVisibility>,
    camera_dimension: Query<&DimensionRef, With<CameraTarget>>,
    dimension_map: Res<DimensionEntityMap>,
    mut daylight_query: Query<(&mut DimensionDaylightSeri, &mut DimensionDaylightRuntime)>,
    mut sim_timescale_query: Query<&mut SimTimeScale, With<SettingsEntity>>,
    mut directional_light_override: ResMut<DirectionalLight2dOverride>,
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
    let Ok((mut daylight, mut daylight_runtime)) = daylight_query.get_mut(dimension_ent) else {
        return;
    };
    let Ok(mut sim_timescale) = sim_timescale_query.single_mut() else {
        return;
    };
    daylight.normalize();
    daylight_runtime.normalize(daylight.day_length_minutes);
    egui::Window::new("Daylight")
        .default_pos([screen_rect.right() - 320.0, screen_rect.top() + 10.0])
        .resizable(true)
        .movable(true)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("Day progress: {:.1}%", daylight.day_progress(&daylight_runtime) * 100.0));
            edit_scalar(ui, "Day length (min)", &mut daylight.day_length_minutes, 0.1);
            edit_scalar(ui, "Current time (min)", &mut daylight_runtime.time_of_day_minutes, 0.1);
            edit_scalar(ui, "Minute offset (min)", &mut daylight.minute_offset, 0.1);
            edit_scalar(ui, "Sim scale", &mut sim_timescale.0, 0.01);
            sim_timescale.0 = sim_timescale.0.max(0.0);
            ui.checkbox(&mut daylight.paused_daylight, "Paused daylight");
            ui.separator();
            edit_rgb(ui, "Ambient color", &mut daylight.ambient_color_rgb);
            edit_rgb(ui, "Night color", &mut daylight.night_color_rgb);
            edit_rgb(ui, "Dawn/Dusk color", &mut daylight.dawn_dusk_color_rgb);
            edit_scalar(ui, "Day curve exponent", &mut daylight.day_curve_exponent, 0.01);
            edit_scalar(ui, "Dawn/Dusk curve exponent", &mut daylight.dawn_dusk_curve_exponent, 0.01);
            edit_scalar(ui, "Ambient min brightness factor", &mut daylight.ambient_min_brightness_factor, 0.01);
            edit_scalar(ui, "Ambient max brightness factor", &mut daylight.ambient_max_brightness_factor, 0.01);
            edit_scalar(ui, "Ambient brightness", &mut daylight.ambient_brightness, 0.01);
            ui.checkbox(&mut daylight.disable_directional_light, "Disable directional light");
            edit_rgb(ui, "Directional light color", &mut daylight.directional_light_color_rgb);

            ui.separator();
            ui.label("Directional light overrides");
            ui.horizontal(|ui| {
                ui.checkbox(&mut directional_light_override.color_enabled, "Color");
                ui.add_enabled_ui(directional_light_override.color_enabled, |ui| {
                    edit_rgb(ui, "", &mut directional_light_override.color_rgb);
                });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut directional_light_override.height_enabled, "Height");
                ui.add_enabled_ui(directional_light_override.height_enabled, |ui| {
                    edit_scalar(ui, "", &mut directional_light_override.height, 0.01);
                });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut directional_light_override.direction_enabled, "Direction");
                ui.add_enabled_ui(directional_light_override.direction_enabled, |ui| {
                    edit_vec2(ui, "", &mut directional_light_override.direction_xy, 0.01);
                });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut directional_light_override.tile_size_enabled, "Tile size");
                ui.add_enabled_ui(directional_light_override.tile_size_enabled, |ui| {
                    edit_scalar(ui, "", &mut directional_light_override.tile_size, 0.01);
                });
            });

            daylight.normalize();
            daylight_runtime.normalize(daylight.day_length_minutes);
        });

    window_visible.daylight = open;
}