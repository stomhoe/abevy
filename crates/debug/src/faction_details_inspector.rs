use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector;
use ::being_shared::BeingMembers;
use common::common_components::{DisplayName, StrId};
use faction::faction_resources::FactionRef;
use faction_shared::*;
use game_common::Templ;
use player_shared::player_components::Mine;

use crate::debug_resources::{DebugSelectedEntities, DubugWindowsVisibility};

fn entity_label(entity: Entity, display_name: Option<&DisplayName>, str_id: Option<&StrId>) -> String {
    if let Some(display_name) = display_name {
        if !display_name.0.is_empty() {
            return format!("{} ({:?})", display_name.0, entity);
        }
    }
    if let Some(str_id) = str_id {
        if !str_id.as_str().is_empty() {
            return format!("{} ({:?})", str_id, entity);
        }
    }
    format!("{:?}", entity)
}

#[allow(unused_parens)]
pub fn faction_details_inspector(world: &mut World) {
    let Some(window_visible) = world.get_resource::<DubugWindowsVisibility>() else {
        return;
    };
    if !window_visible.faction_details {
        return;
    }

    let Some(selected_entities) = world.get_resource::<DebugSelectedEntities>() else {
        return;
    };
    let mut selected_faction = selected_entities.selected_faction;
    let mut show_full_components = selected_entities.show_full_faction_components;

    let mut egui_context_query = world.query_filtered::<
        &bevy_inspector_egui::bevy_egui::EguiContext,
        With<bevy_inspector_egui::bevy_egui::PrimaryEguiContext>,
    >();
    let Some(egui_context) = egui_context_query.iter(world).next() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let screen_rect = egui_context.get_mut().content_rect();

    let mut display_name_query = world.query::<&DisplayName>();
    let mut str_id_query = world.query::<&StrId>();
    let mut faction_list_query = world.query_filtered::<
        (Entity, Option<&DisplayName>, Option<&StrId>, ),
        (With<Faction>, Without<Templ>, ),
    >();
    let mut player_members_query = world.query::<&PlayerMembers>();
    let mut being_members_query = world.query::<&BeingMembers>();
    let mut mine_query = world.query::<&Mine>();
    let mut faction_ref_query = world.query::<&FactionRef>();
    let mut group_player_authority_query = world.query::<&GroupPlayerAuthority>();
    let mut relation_ship_query = world.query::<&RelationShip>();
    let mut relation_ship_status_query = world.query::<&RelationShipStatus>();
    let mut inter_faction_event_query = world.query::<&InterFactionEvent>();
    let mut inclination_query = world.query::<&Inclination>();

    let mut faction_entries = Vec::new();
    let iter = faction_list_query.iter(world);
    faction_entries.reserve(iter.size_hint().1.unwrap_or(iter.size_hint().0));
    for (faction_ent, display_name, str_id, ) in iter {
        faction_entries.push((
            faction_ent,
        entity_label(faction_ent, display_name, str_id),
        ));
    }

    if selected_faction.is_none()
        || !faction_entries.iter().any(|(entity, _)| Some(*entity) == selected_faction)
    {
        selected_faction = faction_entries.first().map(|(entity, _)| *entity);
    }

    let mut clear_selection = false;
    let mut is_open = true;
    let world_ptr = world as *mut World;

    egui::Window::new("Selected Faction Details")
        .default_width(720.0)
        .default_height(540.0)
        .default_pos([screen_rect.right() - 740.0, screen_rect.top() + 10.0])
        .open(&mut is_open)
        .vscroll(true)
        .show(egui_context.get_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Show Full Components").clicked() {
                    show_full_components = !show_full_components;
                }
                if ui.button("Clear Selection").clicked() {
                    clear_selection = true;
                }
            });
            ui.separator();

            if show_full_components {
                ui.label("All Components on this Faction:");
                ui.separator();
                if let Some(faction_ent) = selected_faction {
                    unsafe {
                        bevy_inspector::ui_for_entity(&mut *world_ptr, faction_ent, ui);
                    }
                } else {
                    ui.label("No faction selected.");
                }
                return;
            }

            if faction_entries.is_empty() {
                ui.label("No faction entities found.");
                return;
            }

            egui::ComboBox::from_label("Faction")
                .selected_text(
                    selected_faction
                        .and_then(|selected_faction| {
                            faction_entries
                                .iter()
                                .find(|(entity, _)| *entity == selected_faction)
                                .map(|(_, label)| label.clone())
                        })
                        .unwrap_or_else(|| "None".to_string()),
                )
                .show_ui(ui, |ui| {
                    for (faction_ent, label) in &faction_entries {
                        ui.selectable_value(&mut selected_faction, Some(*faction_ent), label);
                    }
                });

            let Some(faction_ent) = selected_faction else {
                ui.label("No faction selected.");
                return;
            };

            let faction_label = entity_label(
                faction_ent,
                display_name_query.get(world, faction_ent).ok(),
                str_id_query.get(world, faction_ent).ok(),
            );
            ui.heading(format!("Faction Entity: {}", faction_label));
            ui.label(format!("Entity: {:?}", faction_ent));
            ui.label(format!(
                "Mine: {}",
                mine_query.get(world, faction_ent).is_ok()
            ));

            if let Ok(faction_ref) = faction_ref_query.get(world, faction_ent) {
                ui.label(format!("FactionRef: {:?}", faction_ref.0));
            }

            ui.separator();

            ui.collapsing("Membership", |ui| {
                match player_members_query.get(world, faction_ent) {
                    Ok(player_members) => {
                        let members = player_members.iter().collect::<Vec<_>>();
                        ui.label(format!("PlayerMembers: {} player(s)", members.len()));
                        for member_ent in members {
                            ui.label(format!(
                                "  - {}",
                                entity_label(
                                    member_ent,
                                    display_name_query.get(world, member_ent).ok(),
                                    str_id_query.get(world, member_ent).ok(),
                                )
                            ));
                        }
                    }
                    Err(_) => {
                        ui.label("PlayerMembers: missing");
                    }
                }

                match being_members_query.get(world, faction_ent) {
                    Ok(being_members) => {
                        let members = being_members.iter().collect::<Vec<_>>();
                        ui.label(format!("BeingMembers: {} being(s)", members.len()));
                        for member_ent in members {
                            ui.label(format!(
                                "  - {}",
                                entity_label(
                                    member_ent,
                                    display_name_query.get(world, member_ent).ok(),
                                    str_id_query.get(world, member_ent).ok(),
                                )
                            ));
                        }
                    }
                    Err(_) => {
                        ui.label("BeingMembers: missing");
                    }
                }
            });

            ui.separator();

            ui.collapsing("Faction Components", |ui| {
                if let Ok(group_player_authority) = group_player_authority_query.get(world, faction_ent) {
                    ui.label(format!("GroupPlayerAuthority: {:?}", group_player_authority.player));
                } else {
                    ui.label("GroupPlayerAuthority: missing");
                }
                if let Ok(relation_ship) = relation_ship_query.get(world, faction_ent) {
                    ui.label(format!(
                        "RelationShip: source={:?} destination={:?}",
                        relation_ship.source,
                        relation_ship.destination
                    ));
                } else {
                    ui.label("RelationShip: missing");
                }
                if let Ok(relation_ship_status) = relation_ship_status_query.get(world, faction_ent) {
                    ui.label(format!("RelationShipStatus: {:?}", relation_ship_status));
                } else {
                    ui.label("RelationShipStatus: missing");
                }
                if let Ok(inter_faction_event) = inter_faction_event_query.get(world, faction_ent) {
                    ui.label(format!("InterFactionEvent: {:?}", inter_faction_event.nid()));
                } else {
                    ui.label("InterFactionEvent: missing");
                }
                if let Ok(inclination) = inclination_query.get(world, faction_ent) {
                    ui.label(format!("Inclination: {:?}", inclination));
                } else {
                    ui.label("Inclination: missing");
                }
            });
        });

    if let Some(mut selected_entities) = world.get_resource_mut::<DebugSelectedEntities>() {
        if clear_selection {
            selected_entities.selected_faction = None;
            selected_entities.show_full_faction_components = false;
        } else {
            selected_entities.selected_faction = selected_faction;
            selected_entities.show_full_faction_components = show_full_components;
        }
    }

    if !is_open {
        if let Some(mut window_visible) = world.get_resource_mut::<DubugWindowsVisibility>() {
            window_visible.faction_details = false;
        }
    }
}
