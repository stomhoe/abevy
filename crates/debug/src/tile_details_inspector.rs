use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use common::common_tag_components::TagSet;
use common::common_components::StrId;
use game_common::game_common_components::TemplEntiRef;
use bevy_ecs_tilemap::map::TilemapId;
use tilemap::tile::tile_components::{TileStrId};
use tilemap_shared::DeleteOtherTilesInSamePos;
use tilemap_shared::SafeDespawn;

use debug_shared::{DebugSelectedEntities, DubugWindowsVisibility};

#[allow(unused_parens)]
pub fn tile_details_inspector(world: &mut World) {
    let selected_entities = world.resource::<DebugSelectedEntities>();

    let multi_tiles_active = world
        .get_resource::<debug_shared::ClickInspectorState>()
        .is_some_and(|state| state.mult_tile_windows);

    if multi_tiles_active {
        let mut tiles: Vec<Entity> = selected_entities.selected_tiles.iter().copied().collect();
        if tiles.is_empty() {
            if let Some(selected_tile_entity) = selected_entities.selected_tile.or(selected_entities.selected_exempted_entity) {
                tiles.push(selected_tile_entity);
            }
        }
        tiles.sort_unstable_by_key(|entity| entity.index());

        if !tiles.is_empty() {
            let mut egui_context_query = world.query_filtered::<
                &bevy_inspector_egui::bevy_egui::EguiContext,
                With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
            >();
            let Some(egui_context) = egui_context_query.iter(world).next() else {
                return;
            };
            let mut egui_context = egui_context.clone();
            let screen_rect = egui_context.get_mut().content_rect();
            let world_ptr = world as *mut World;
            let mut is_open = true;
            let mut remove_tile_requested = None;
            let mut clear_all_requested = false;

            egui::Window::new("Selected Tile Details")
                .default_width(780.0)
                .default_height(520.0)
                .default_pos([screen_rect.right() - 800.0, screen_rect.top() + 10.0])
                .open(&mut is_open)
                .vscroll(true)
                .show(egui_context.get_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Tile Details");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear all selections").clicked() {
                                clear_all_requested = true;
                            }
                        });
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.columns(tiles.len(), |columns| {
                            for (column, entity) in columns.iter_mut().zip(tiles.iter().copied()) {
                                column.push_id(entity, |ui| {
                                    render_tile_details_column(ui, world, world_ptr, entity, &mut remove_tile_requested, false);
                                });
                            }
                        });
                    });
                });

            if clear_all_requested {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_tiles.clear();
                    selected_entities.selected_tile = None;
                    selected_entities.selected_exempted_entity = None;
                }
            }

            if let Some(tile_to_remove) = remove_tile_requested {
                if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                    selected_entities.selected_tiles.remove(&tile_to_remove);
                    if selected_entities.selected_tile == Some(tile_to_remove) {
                        selected_entities.selected_tile = selected_entities.selected_tiles.iter().next().copied();
                    }
                    if selected_entities.selected_tiles.is_empty() {
                        selected_entities.selected_tile = None;
                    }
                }
            }

            if !is_open {
                if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
                    window_visible.tile_details = false;
                }
            }
            return;
        }
    }

    let selected_tile_entity = selected_entities.selected_tile.or(selected_entities.selected_exempted_entity);

    let Some(selected_tile_entity) = selected_tile_entity else {
        return;
    };

    let window_visible = world.resource::<DubugWindowsVisibility>();

    if !window_visible.tile_details {
        return;
    }

    // Try to get the TileStrId from the referenced EntityZero
    let tile_str_id = if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            if let Ok(templ_entity) = world.get_entity(templ_ref.0) {
                if let Some(str_id) = templ_entity.get::<TileStrId>() {
                    Some(format!("{}", str_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut delete_other_tiles_here = None;
    let mut delete_other_tiles_templ = None;
    let mut tags_here = None;
    let mut tags_templ = None;
    let mut referenced_templ_entity = None;
    let mut referenced_tilemap_entity = None;
    if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        delete_other_tiles_here = entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
        tags_here = entity_ref.get::<TagSet>().cloned();
        if let Some(tilemap_id) = entity_ref.get::<TilemapId>() {
            referenced_tilemap_entity = Some(tilemap_id.0);
        }
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            referenced_templ_entity = Some(templ_ref.0);
            if let Ok(templ_entity_ref) = world.get_entity(templ_ref.0) {
                delete_other_tiles_templ = templ_entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
                tags_templ = templ_entity_ref.get::<TagSet>().cloned();
            }
        }
    }

    let mut egui_context_query = world
        .query_filtered::<&bevy_inspector_egui::bevy_egui::EguiContext, With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>>();

    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };

    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let world_ptr = world as *mut World;
    let mut is_open = true;

    egui::Window::new("Selected Tile Details")
        .default_width(600.0)
        .default_height(500.0)
        .default_pos([screen_rect.right() - 620.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            let mut remove_tile_requested = None;
            render_tile_details_column(
                ui,
                world,
                world_ptr,
                selected_tile_entity,
                &mut remove_tile_requested,
                true,
            );
        });

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.tile_details = false;
        }
    }
}

fn render_tile_details_column(
    ui: &mut egui::Ui,
    world: &mut World,
    world_ptr: *mut World,
    selected_tile_entity: Entity,
    remove_tile_requested: &mut Option<Entity>,
    show_clear_selection_button: bool,
) {
    let tile_str_id = if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            if let Ok(templ_entity) = world.get_entity(templ_ref.0) {
                templ_entity.get::<TileStrId>().map(|str_id| format!("{}", str_id))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut delete_other_tiles_here = None;
    let mut delete_other_tiles_templ = None;
    let mut tags_here = None;
    let mut tags_templ = None;
    let mut referenced_templ_entity = None;
    let mut referenced_tilemap_entity = None;
    if let Ok(entity_ref) = world.get_entity(selected_tile_entity) {
        delete_other_tiles_here = entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
        tags_here = entity_ref.get::<TagSet>().cloned();
        if let Some(tilemap_id) = entity_ref.get::<TilemapId>() {
            referenced_tilemap_entity = Some(tilemap_id.0);
        }
        if let Some(templ_ref) = entity_ref.get::<TemplEntiRef>() {
            referenced_templ_entity = Some(templ_ref.0);
            if let Ok(templ_entity_ref) = world.get_entity(templ_ref.0) {
                delete_other_tiles_templ = templ_entity_ref.get::<DeleteOtherTilesInSamePos>().cloned();
                tags_templ = templ_entity_ref.get::<TagSet>().cloned();
            }
        }
    }

    ui.horizontal(|ui| {
        if let Some(str_id) = tile_str_id {
            ui.heading(format!("Tile: {}", str_id));
        } else {
            ui.heading(format!("Entity: {:?}", selected_tile_entity));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add_sized([24.0, 24.0], egui::Button::new(egui::RichText::new("X").strong().size(18.0))).clicked() {
                *remove_tile_requested = Some(selected_tile_entity);
            }
        });
    });
    ui.separator();

    ui.collapsing("Full Entity Components", |ui| {
        if world.get_entity(selected_tile_entity).is_ok() {
            unsafe {
                bevy_inspector::ui_for_entity(&mut *world_ptr, selected_tile_entity, ui);
            }
        } else {
            ui.label("Selected tile entity missing");
        }
    });
    ui.separator();

    ui.label("Manual component details");
    render_tagset_section(ui, "TagSet on selected entity", tags_here.as_ref());
    render_delete_other_tiles_section(
        ui,
        "DeleteOtherTilesInSamePos on selected entity",
        delete_other_tiles_here.as_ref(),
    );

    if let Some(templ_entity) = referenced_templ_entity {
        let templ_str_id = world
            .get_entity(templ_entity)
            .ok()
            .and_then(|entity_ref| entity_ref.get::<StrId>().cloned());
        let templ_label = format!(
            "TemplEntiRef: {} {:?}",
            templ_str_id.map_or_else(|| "<no-strid>".to_string(), |str_id| str_id.to_string()),
            templ_entity,
        );

        ui.separator();
        ui.collapsing(templ_label, |ui| {
            if world.get_entity(templ_entity).is_ok() {
                unsafe {
                    bevy_inspector::ui_for_entity(&mut *world_ptr, templ_entity, ui);
                }
            } else {
                ui.label("TemplEntiRef target entity missing");
            }

            render_tagset_section(ui, "TagSet on EntityZero", tags_templ.as_ref());
            render_delete_other_tiles_section(
                ui,
                "DeleteOtherTilesInSamePos on EntityZero",
                delete_other_tiles_templ.as_ref(),
            );
        });
    }

    if let Some(tilemap_entity) = referenced_tilemap_entity {
        ui.separator();
        ui.collapsing("Tilemap Entity Details", |ui| {
            ui.label(format!("TilemapId target: {:?}", tilemap_entity));
            if world.get_entity(tilemap_entity).is_ok() {
                unsafe {
                    bevy_inspector::ui_for_entity(&mut *world_ptr, tilemap_entity, ui);
                }
            } else {
                ui.label("TilemapId target entity missing");
            }
        });
    }

    ui.separator();
    if ui.button("Despawn").clicked() {
        world
            .resource_mut::<Messages<SafeDespawn>>()
            .write(SafeDespawn {
                tile_ent: selected_tile_entity,
                remove_u16_index: true,
            });
        if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
            if selected_entities.selected_tile == Some(selected_tile_entity) {
                selected_entities.selected_tile = selected_entities
                    .selected_tiles
                    .iter()
                    .copied()
                    .find(|entity| *entity != selected_tile_entity);
            }
            selected_entities.selected_tiles.remove(&selected_tile_entity);
            if selected_entities.selected_exempted_entity == Some(selected_tile_entity) {
                selected_entities.selected_exempted_entity = None;
            }
            if selected_entities.selected_tile == Some(selected_tile_entity) {
                selected_entities.selected_tile = None;
            }
        }
    }
    if show_clear_selection_button {
        if ui.button("Clear Selection").clicked() {
            if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
                selected_entities.selected_tile = None;
                selected_entities.selected_tiles.clear();
                selected_entities.selected_exempted_entity = None;
            }
        }
    }
}

fn render_tagset_section(ui: &mut egui::Ui, title: &str, tags: Option<&TagSet>) {
    ui.collapsing(title, |ui| {
        let Some(tags) = tags else {
            ui.label("Missing");
            return;
        };
        if tags.is_empty() {
            ui.label("Empty");
            return;
        }

        let mut values: Vec<String> = tags.iter().map(|tag| format!("{:?}", tag)).collect();
        values.sort_unstable();
        ui.label(format!("count: {}", values.len()));
        ui.label(values.join(", "));
    });
}

fn render_delete_other_tiles_section(
    ui: &mut egui::Ui,
    title: &str,
    spec: Option<&DeleteOtherTilesInSamePos>,
) {
    ui.collapsing(title, |ui| {
        let Some(spec) = spec else {
            ui.label("Missing");
            return;
        };

        let mut spared_z: Vec<f32> = spec.spared_z.iter().map(|z| z.0).collect();
        spared_z.sort_by(|a, b| a.total_cmp(b));
        let mut targeted_z: Vec<f32> = spec.targeted_z.iter().map(|z| z.0).collect();
        targeted_z.sort_by(|a, b| a.total_cmp(b));

        let mut spared_tags: Vec<String> = spec.spared_tags.iter().map(|tag| format!("{:?}", tag)).collect();
        spared_tags.sort_unstable();
        let mut targeted_tags: Vec<String> = spec.targeted_tags.iter().map(|tag| format!("{:?}", tag)).collect();
        targeted_tags.sort_unstable();

        ui.label(format!("priority: {:.3}", spec.priority));
        ui.label(format!("extra_radius: {}", spec.extra_radius));
        ui.label(format!("displacement: ({}, {})", spec.displacement.x, spec.displacement.y));
        ui.label(format!("spared_z: {:?}", spared_z));
        ui.label(format!("targeted_z: {:?}", targeted_z));
        ui.label(format!("spared_tags: {:?}", spared_tags));
        ui.label(format!("targeted_tags: {:?}", targeted_tags));
        ui.label(format!("is_empty(): {}", spec.is_empty()));
    });
}
