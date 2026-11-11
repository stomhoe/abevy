use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui::RelativeCursorPosition;
use bevy::input::mouse::*;
use rand::seq::IteratorRandom;
use ::being_shared::*;
use common::common_components::*;
use common::log_targets::GAME_INIT;
use game_common::game_common_states::GameSetupScreen;
use multiplayer_shared::multiplayer_events::*;
use game_common::game_common_string_components::*;

use super::character_creation_components::*;

#[allow(unused_parens, )]
pub fn reset_character_creation(mut selection: ResMut<CharacterCreationSelection>) {
    *selection = CharacterCreationSelection::default();
}

#[allow(unused_parens, )]
pub fn update_companion_tooltip(
    root: Query<(&ComputedNode, &UiGlobalTransform, ), (With<CharacterCreationRoot>, )>,
    rows: Query<(&CompanionHover, &RelativeCursorPosition, &ComputedNode, &UiGlobalTransform, ), (Without<CharacterCreationRoot>, )>,
    races: Query<(&HashId, &Description, ), (With<Race>, )>,
    tooltip: Query<(Entity, &ComputedNode, ), (With<CompanionTooltip>, )>,
    mut nodes: Query<(&mut Node, ), ()>,
    mut texts: Query<(&mut Text, ), ()>,
) {
    let Ok((root_node, root_transform, )) = root.single() else { return; };
    let Ok((entity, computed, )) = tooltip.single() else { return; };
    let Ok((mut node, )) = nodes.get_mut(entity) else { return; };
    let hovered = rows.iter().find(|(_, cursor, _, _)| cursor.cursor_over());
    let Some((race, _, row, transform)) = hovered else { node.display = Display::None; return; };
    let Some((_, description)) = races.iter().find(|(id, description)| **id == race.0 && !description.0.trim().is_empty()) else { node.display = Display::None; return; };
    let Ok((mut text, )) = texts.get_mut(entity) else { return; };
    if text.0 != description.0 { text.0.clone_from(&description.0); }
    let scale = root_node.inverse_scale_factor();
    let viewport = root_node.size() * scale;
    let origin = root_transform.translation * scale - viewport * 0.5;
    let position = transform.translation * scale - origin;
    let row_size = row.size() * scale;
    let size = computed.size() * scale;
    node.display = Display::Flex;
    node.max_width = Val::Px(320_f32.min((viewport.x - 16.).max(0.)));
    node.left = Val::Px((position.x + row_size.x * 0.5 + 8.).min(viewport.x - size.x - 8.).max(8.));
    node.top = Val::Px((position.y - row_size.y * 0.5).min(viewport.y - size.y - 8.).max(8.));
}

#[allow(unused_parens, )]
pub fn resize_character_creation(
    root: Query<(&ComputedNode, ), (With<CharacterCreationRoot>, )>,
    columns: Query<(Entity, &Children, ), (With<CharacterCreationColumns>, )>,
    mut nodes: Query<(&mut Node, ), ()>,
    mut positions: Query<(&mut ScrollPosition, ), ()>,
) {
    let Ok((computed, )) = root.single() else { return; };
    let stacked = computed.size().x * computed.inverse_scale_factor() < 900.;
    for (entity, children) in &columns {
        let overflow = if stacked { Overflow::scroll_y() } else { Overflow::clip() };
        if let Ok((mut node, )) = nodes.get_mut(entity) {
            if node.overflow != overflow {
                node.overflow = overflow;
                if let Ok((mut position, )) = positions.get_mut(entity) { position.0 = Vec2::ZERO; }
            }
        }
        for (index, child) in children.iter().enumerate() {
            let Ok((mut node, )) = nodes.get_mut(child) else { continue; };
            let basis = if stacked { Val::Percent(100.) } else { Val::Px(0.) };
            let grow = if stacked { 0. } else if index == 0 { 42. } else { 58. };
            let height = if stacked { Val::Auto } else { Val::Percent(100.) };
            let overflow = if stacked { Overflow::visible() } else { Overflow::scroll_y() };
            if node.flex_basis == basis && node.flex_grow == grow && node.height == height && node.overflow == overflow { continue; }
            node.flex_basis = basis;
            node.flex_grow = grow;
            node.flex_shrink = 0.;
            node.height = height;
            node.min_height = Val::Px(0.);
            node.overflow = overflow;
            if let Ok((mut position, )) = positions.get_mut(child) { position.0 = Vec2::ZERO; }
        }
    }
}

#[allow(unused_parens, )]
pub fn fill_character_selection_defaults(
    mut selection: ResMut<CharacterCreationSelection>,
    races: Query<(&HashId, &StrId, &SetsOfPlayerMonoChoosableSprites, &common::TagSet, ), (With<Race>, With<PlayerSelectableRace>, )>,
    classes: Query<(&HashId, &StrId, &ClassRequirements, Option<&ClassData>, ), (With<Class>, With<PlayerSelectableClass>, )>,
    sprite_ids: Query<(&HashId, ), ()>,
    pet_sets: Query<(&HashId, &PetSetData, ), (With<PetSet>, )>,
) {
    let selected_class = selection.class.and_then(|selected| classes.iter().find(|(id, _, _, _)| **id == selected));
    let compatible_race = |tags: &common::TagSet| selected_class.is_none_or(|(_, _, requirements, _)| requirements.allowed_race_tags.is_empty() || requirements.allowed_race_tags.iter().any(|allowed| tags.iter().any(|tag| HashId::from(tag.as_ref()) == *allowed)));
    let race = selection.race.and_then(|selected| races.iter().find(|(id, _, _, _)| **id == selected));
    let race = if selection.race.is_some() {
        race
    } else {
        races.iter().find(|(_, id, _, tags)| id.as_str() == "human" && compatible_race(tags))
            .or_else(|| races.iter().filter(|(_, _, _, tags)| compatible_race(tags)).min_by(|(_, left, _, _), (_, right, _, _)| left.as_str().cmp(right.as_str())))
    };
    let Some((&race_id, race_name, groups, tags)) = race else {
        error_once!(target: GAME_INIT, "Cannot generate character defaults: selected race is unavailable or no selectable races are loaded");
        return;
    };
    if selection.race.is_none() {
        selection.race = Some(race_id);
    }
    if selection.class.is_some() && !selected_class.is_some_and(|(_, _, requirements, _)| requirements.allowed_race_tags.is_empty() || requirements.allowed_race_tags.iter().any(|allowed| tags.iter().any(|tag| HashId::from(tag.as_ref()) == *allowed))) {
        debug!(target: GAME_INIT, "Discarding unavailable or incompatible class for race '{}'", race_name);
        selection.class = None;
        selection.pets.clear();
    }
    if selection.class.is_none() {
        let class = classes.iter()
            .filter(|(_, _, requirements, _)| requirements.allowed_race_tags.is_empty() || requirements.allowed_race_tags.iter().any(|allowed| tags.iter().any(|tag| HashId::from(tag.as_ref()) == *allowed)))
            .min_by(|(_, left, _, _), (_, right, _, _)| left.as_str().cmp(right.as_str()));
        let Some((&class_id, _, _, _)) = class else {
            error_once!(target: GAME_INIT, "Cannot generate character defaults: race '{}' has no compatible selectable class", race_name);
            return;
        };
        selection.class = Some(class_id);
    }
    let mut rng = rand::rng();
    let valid_sprite = |group_id: &HashId, selected: &HashId| groups.0.iter().any(|(group, options)| group.hash_id() == *group_id && options.iter().any(|&entity| sprite_ids.get(entity).is_ok_and(|(id, )| id == selected)));
    if selection.sprites.iter().any(|(group, sprite)| !valid_sprite(group, sprite)) {
        selection.sprites.retain(|group, sprite| valid_sprite(group, sprite));
        debug!(target: GAME_INIT, "Discarded unavailable appearance choices for race '{}'", race_name);
    }
    for (group, options) in &groups.0 {
        if selection.sprites.contains_key(&group.hash_id()) { continue; }
        let Some(sprite) = options.iter().choose(&mut rng) else {
            error_once!(target: GAME_INIT, "Cannot generate character defaults: race '{}' sprite group '{}' has no options", race_name, group);
            return;
        };
        let Ok((&sprite_id, )) = sprite_ids.get(*sprite) else {
            error_once!(target: GAME_INIT, "Cannot generate character defaults: race '{}' sprite group '{}' has an option without HashId", race_name, group);
            return;
        };
        selection.sprites.insert(group.hash_id(), sprite_id);
    }
    let pet_set = selection.class.and_then(|class| classes.iter().find(|(id, _, _, _)| **id == class))
        .and_then(|(_, _, _, data)| data.and_then(|data| data.pet_set))
        .and_then(|pet_set| pet_sets.iter().find(|(id, _)| **id == pet_set).map(|(_, data)| data));
    let valid_pet = |pet: &HashId, count: &u32| *count > 0 && pet_set.is_some_and(|set| set.pets.iter().any(|entry| entry.id == *pet));
    if selection.pets.iter().any(|(pet, count)| !valid_pet(pet, count)) {
        selection.pets.retain(|pet, count| valid_pet(pet, count));
        debug!(target: GAME_INIT, "Discarded unavailable companion choices for race '{}'", race_name);
    }
    if pet_set.is_some_and(|set| !selection.pets_fit(set, None)) {
        selection.pets.clear();
        debug!(target: GAME_INIT, "Discarded companion choices exceeding the current class limits for race '{}'", race_name);
    }
}

#[allow(unused_parens, )]
pub fn send_character_selection(
    selection: Res<CharacterCreationSelection>,
    mut requests: MessageWriter<CharacterCreationRequest>,
    mut messages: Local<Vec<CharacterCreationRequest>>,
) {
    let Some(request) = selection.to_request() else {
        debug!(target: GAME_INIT, "Character selection pending: race selected={}, class selected={}, {} sprite groups", selection.race.is_some(), selection.class.is_some(), selection.sprites.len());
        return;
    };
    debug!(target: GAME_INIT, "Submitting character selection: race {}, class {}, {} sprite groups", request.race, request.class, request.sprites.len());
    messages.push(request);
    requests.write_batch(messages.drain(..));
}

#[allow(unused_parens, )]
pub fn scroll_character_creation(
    mut wheel: MessageReader<MouseWheel>,
    mut columns: Query<(&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition, Has<CharacterCreationPopup>, ), (Or<(With<CharacterCreationColumns>, With<CharacterCreationScrollArea>, With<CharacterCreationPopup>)>, )>,
) {
    for event in wheel.read() {
        let popup_hovered = columns.iter().any(|(cursor, _, _, is_popup)| is_popup && cursor.cursor_over());
        for (cursor, computed, mut position, is_popup) in &mut columns {
            if !cursor.cursor_over() { continue; }
            if !is_popup && popup_hovered { continue; }
            let distance = match event.unit { MouseScrollUnit::Line => event.y * 40., MouseScrollUnit::Pixel => event.y };
            let max_scroll = ((computed.content_size().y - computed.size().y) * computed.inverse_scale_factor()).max(0.);
            position.0.y = (position.0.y - distance).clamp(0., max_scroll);
        }
    }
}

#[allow(unused_parens, )]
pub fn position_character_creation_popups(
    mut commands: Commands,
    root: Query<(Entity, &ComputedNode, &UiGlobalTransform, ), (With<CharacterCreationRoot>, )>,
    anchors: Query<(&ComputedNode, &UiGlobalTransform, ), (Or<(With<SpriteGroupAnchor>, With<CharacterCreationDropdownAnchor>)>, )>,
    mut menus: Query<(Entity, &ChildOf, Option<&CharacterCreationPopup>, &ComputedNode, &mut Node, ), (Or<(With<SpriteGroupMenu>, With<CharacterCreationDropdownMenu>)>, )>,
) {
    let Ok((root, root_node, root_transform)) = root.single() else { return; };
    let scale = root_node.inverse_scale_factor();
    let viewport = root_node.size() * scale;
    let origin = root_transform.translation * scale - viewport * 0.5;
    for (entity, parent, popup, computed, mut node) in &mut menus {
        let anchor = popup.map_or(parent.parent(), |popup| popup.0);
        let Ok((anchor_node, transform)) = anchors.get(anchor) else { commands.entity(entity).try_despawn(); continue; };
        if popup.is_none() {
            commands.entity(entity).insert((ChildOf(root), CharacterCreationPopup(anchor), RelativeCursorPosition::default()));
        }
        if node.display == Display::None { continue; }
        let size = anchor_node.size() * scale;
        let top_left = transform.translation * scale - size * 0.5 - origin;
        let width = size.x.max(280.).min((viewport.x - 16.).max(0.));
        let below = (viewport.y - top_left.y - size.y - 8.).max(0.);
        let above = (top_left.y - 8.).max(0.);
        let height_limit = below.max(above).min(viewport.y * 0.7);
        node.width = Val::Px(width);
        node.min_width = Val::Px(0.);
        node.max_width = Val::Px(width);
        node.max_height = Val::Px(height_limit);
        node.overflow = Overflow::scroll_y();
        node.left = Val::Px(top_left.x.clamp(8., (viewport.x - width - 8.).max(8.)));
        node.top = Val::Px(if below >= above { top_left.y + size.y } else { (top_left.y - (computed.size().y * scale).min(height_limit)).max(8.) });
    }
}

#[allow(unused_parens, )]
pub fn character_creation_button_interaction(
    mut selection: ResMut<CharacterCreationSelection>,
    mut next_screen: ResMut<NextState<GameSetupScreen>>,
    buttons: Query<&CharacterCreationButton, (Changed<Pressed>, With<Pressed>, )>,
    mut dropdown_controls: ParamSet<(
        Query<(&SpriteGroupMenu, &mut Node, &mut GlobalZIndex), ()>,
        Query<(&SpriteGroupAnchor, &mut ZIndex), (Without<SpriteGroupMenu>, )>,
        Query<(&CharacterCreationDropdownMenu, &mut Node, &mut GlobalZIndex), ()>,
        Query<(&CharacterCreationDropdownAnchor, &mut ZIndex), (Without<CharacterCreationDropdownMenu>, )>,
    )>,
    classes: Query<(&HashId, &ClassData), (With<Class>, With<PlayerSelectableClass>, )>,
    pet_sets: Query<(&HashId, &PetSetData), (With<PetSet>, )>,
) {
    for button in &buttons {
        match *button {
            CharacterCreationButton::ToggleRaceDropdown | CharacterCreationButton::ToggleClassDropdown => {
                for (_, mut node, mut z_index) in &mut dropdown_controls.p0() {
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                let kind = if matches!(*button, CharacterCreationButton::ToggleRaceDropdown) { CharacterCreationDropdownKind::Race } else { CharacterCreationDropdownKind::Class };
                let mut opened = false;
                for (menu, mut node, mut z_index) in &mut dropdown_controls.p2() {
                    if menu.0 == kind && node.display == Display::None {
                        opened = true;
                    }
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                for (_, mut z_index) in &mut dropdown_controls.p3() {
                    z_index.0 = 0;
                }
                if opened {
                    for (menu, mut node, mut z_index) in &mut dropdown_controls.p2() {
                        if menu.0 == kind {
                            node.display = Display::Flex;
                            z_index.0 = 200;
                        }
                    }
                    for (anchor, mut z_index) in &mut dropdown_controls.p3() {
                        if anchor.0 == kind {
                            z_index.0 = 200;
                        }
                    }
                }
            }
            CharacterCreationButton::ToggleSpriteGroup(group) => {
                for (_, mut node, mut z_index) in &mut dropdown_controls.p2() {
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                let mut opened = false;
                for (menu, mut node, mut z_index) in &mut dropdown_controls.p0() {
                    if menu.0 == group && node.display == Display::None {
                        opened = true;
                    }
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                for (_, mut z_index) in &mut dropdown_controls.p1() {
                    z_index.0 = 0;
                }
                if opened {
                    for (menu, mut node, mut z_index) in &mut dropdown_controls.p0() {
                        if menu.0 == group {
                            node.display = Display::Flex;
                            z_index.0 = 200;
                        }
                    }
                    for (anchor, mut z_index) in &mut dropdown_controls.p1() {
                        if anchor.0 == group {
                            z_index.0 = 200;
                        }
                    }
                }
            }
            CharacterCreationButton::Race(race) => {
                selection.switch_race(race);
                debug!(target: GAME_INIT, "Selected race {}: restored {} appearance choices and {} companion types", race, selection.sprites.len(), selection.pets.len());
                for (_, mut node, mut z_index) in &mut dropdown_controls.p2() {
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                for (_, mut z_index) in &mut dropdown_controls.p3() { z_index.0 = 0; }
            }
            CharacterCreationButton::Class(class) => {
                selection.class = Some(class);
                selection.pets.clear();
                for (_, mut node, mut z_index) in &mut dropdown_controls.p2() {
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                for (_, mut z_index) in &mut dropdown_controls.p3() { z_index.0 = 0; }
            }
            CharacterCreationButton::Sprite { group, sprite } => {
                selection.sprites.insert(group, sprite);
                for (_, mut node, mut z_index) in &mut dropdown_controls.p0() {
                    node.display = Display::None;
                    z_index.0 = 0;
                }
                for (_, mut z_index) in &mut dropdown_controls.p1() { z_index.0 = 0; }
            }
            CharacterCreationButton::Pet { pet, delta } => {
                let Some(class) = selection.class else { continue; };
                let Some((_, class_data)) = classes.iter().find(|(id, _)| **id == class) else { continue; };
                let Some(pet_set_id) = class_data.pet_set else { continue; };
                let Some((_, pet_set)) = pet_sets.iter().find(|(id, _)| **id == pet_set_id) else { continue; };
                if delta < 0 {
                    if let Some(count) = selection.pets.get_mut(&pet) {
                        *count = count.saturating_sub(delta.unsigned_abs() as u32);
                        if *count == 0 {
                            selection.pets.remove(&pet);
                        }
                    }
                } else {
                    if selection.pets_fit(pet_set, Some(pet)) {
                        *selection.pets.entry(pet).or_default() += 1;
                    }
                }
            }
            CharacterCreationButton::Cancel => {
                next_screen.set(GameSetupScreen::GameSettings);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_drafts_restore_and_submit_only_the_active_race() {
        let mut app = App::new();
        app.init_resource::<CharacterCreationSelection>()
            .add_message::<CharacterCreationRequest>()
            .add_systems(Update, (send_character_selection, update_companion_tooltip));
        let human = HashId::from("human");
        let dwarf = HashId::from("dwarf");
        let class = HashId::from("warrior");
        let group = HashId::from("Head");
        let head = HashId::from("human_head");
        let pet = HashId::from("wolf");
        {
            let mut selection = app.world_mut().resource_mut::<CharacterCreationSelection>();
            selection.switch_race(human);
            selection.class = Some(class);
            selection.sprites.insert(group, head);
            selection.pets.insert(pet, 2);
            selection.switch_race(dwarf);
            assert!(selection.class.is_none() && selection.sprites.is_empty() && selection.pets.is_empty());
            selection.class = Some(HashId::from("archer"));
            selection.sprites.insert(group, HashId::from("dwarf_head"));
            selection.switch_race(human);
            selection.switch_race(human);
        }
        app.update();
        let requests: Vec<_> = app.world_mut().resource_mut::<Messages<CharacterCreationRequest>>().drain().collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].race, human);
        assert_eq!(requests[0].class, class);
        assert_eq!(requests[0].sprites, vec![(group, head)]);
        assert_eq!(requests[0].pets, vec![pet, pet]);
        app.world_mut().resource_mut::<CharacterCreationSelection>().switch_race(dwarf);
        assert_eq!(app.world().resource::<CharacterCreationSelection>().class, Some(HashId::from("archer")));
        app.add_systems(Update, reset_character_creation.after(send_character_selection));
        app.update();
        let selection = app.world().resource::<CharacterCreationSelection>();
        assert!(selection.race.is_none() && selection.drafts.is_empty());
    }

    #[test]
    fn character_sheet_reflows_without_shrinking_text() {
        let mut app = App::new();
        app.add_systems(Update, resize_character_creation);
        let root = app.world_mut().spawn((CharacterCreationRoot, ComputedNode::default())).id();
        let columns = app.world_mut().spawn((CharacterCreationColumns, Node::default(), ChildOf(root))).id();
        let left = app.world_mut().spawn((Node::default(), ChildOf(columns))).id();
        let right = app.world_mut().spawn((Node::default(), ChildOf(columns))).id();

        for (width, height, inverse_scale, stacked) in [(1280., 720., 1., false), (1280., 360., 1., false), (1024., 768., 1., false), (800., 360., 1., true), (1600., 1200., 0.5, true), (1280., 720., 1., false)] {
            app.world_mut().entity_mut(root).insert(ComputedNode { size: Vec2::new(width, height), inverse_scale_factor: inverse_scale, ..default() });
            app.update();
            let left = app.world().get::<Node>(left).unwrap();
            let right = app.world().get::<Node>(right).unwrap();
            assert_eq!(left.flex_basis, if stacked { Val::Percent(100.) } else { Val::Px(0.) });
            assert_eq!(right.flex_basis, left.flex_basis);
            assert_eq!(left.flex_grow, if stacked { 0. } else { 42. });
            assert_eq!(right.flex_grow, if stacked { 0. } else { 58. });
            assert_eq!(left.height, if stacked { Val::Auto } else { Val::Percent(100.) });
            assert_eq!(right.height, left.height);
            assert_eq!(left.overflow, if stacked { Overflow::visible() } else { Overflow::scroll_y() });
            assert_eq!(right.overflow, left.overflow);
            assert_eq!(app.world().get::<Node>(columns).unwrap().overflow, if stacked { Overflow::scroll_y() } else { Overflow::clip() });
        }
    }

    #[test]
    fn short_columns_scroll_to_the_end_without_scrolling_under_open_popups() {
        let mut app = App::new();
        app.add_message::<MouseWheel>().add_systems(Update, scroll_character_creation.run_if(on_message::<MouseWheel>));
        let column = app.world_mut().spawn((CharacterCreationScrollArea, RelativeCursorPosition { cursor_over: true, ..default() }, ComputedNode { size: Vec2::new(400., 200.), content_size: Vec2::new(400., 700.), inverse_scale_factor: 1., ..default() }, ScrollPosition::default())).id();
        let wheel = || MouseWheel { unit: MouseScrollUnit::Line, x: 0., y: -100., window: Entity::PLACEHOLDER, phase: bevy::input::touch::TouchPhase::Moved };
        app.world_mut().write_message(wheel());
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(column).unwrap().0.y, 500.);
        app.world_mut().get_mut::<ScrollPosition>(column).unwrap().0.y = 0.;
        let popup = app.world_mut().spawn((CharacterCreationPopup(column), RelativeCursorPosition { cursor_over: true, ..default() }, ComputedNode { size: Vec2::new(300., 150.), content_size: Vec2::new(300., 450.), inverse_scale_factor: 1., ..default() }, ScrollPosition::default())).id();
        app.world_mut().write_message(wheel());
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(column).unwrap().0.y, 0.);
        assert_eq!(app.world().get::<ScrollPosition>(popup).unwrap().0.y, 300.);
    }

    #[test]
    fn defaults_fill_missing_choices_and_preserve_selected_sprites() {
        let mut app = App::new();
        app.init_resource::<CharacterCreationSelection>()
            .add_message::<CharacterCreationRequest>()
            .add_systems(Update, (fill_character_selection_defaults, send_character_selection.run_if(resource_changed::<CharacterCreationSelection>)).chain());
        let human = HashId::from("human");
        let elf = HashId::from("elf");
        let head_group = StrId::trunc("Head");
        let body_group = StrId::trunc("Body");
        let head_id = HashId::from("head_choice");
        let picked_head = HashId::from("picked_head");
        let body_id = HashId::from("body_choice");
        let head = app.world_mut().spawn(head_id).id();
        let alternate_head = app.world_mut().spawn(picked_head).id();
        let body = app.world_mut().spawn(body_id).id();
        for (race, name) in [(human, "human"), (elf, "elf")] {
            app.world_mut().spawn((Race, PlayerSelectableRace, race, StrId::trunc(name), common::TagSet::new([name]), SetsOfPlayerMonoChoosableSprites(vec![
                (head_group.clone(), [head, alternate_head].into_iter().collect()),
                (body_group.clone(), [body].into_iter().collect()),
            ])));
        }
        let elf_class = HashId::from("a_elf_only");
        let pet = HashId::from("picked_pet");
        let pet_set = HashId::from("test_pets");
        app.world_mut().spawn((PetSet, pet_set, PetSetData { max_selected: 2, budget: 2, per_race_max: default(), pets: vec![PetSetEntry { id: pet, name: "Pet".into(), race_id: human, cost: 1 }] }));
        app.world_mut().spawn((Class, PlayerSelectableClass, elf_class, StrId::trunc("a_elf_only"), ClassRequirements { allowed_race_tags: vec![elf], ..default() }, ClassData { pet_set: Some(pet_set), ..default() }));
        let all_races_class = HashId::from("z_all_races");
        app.world_mut().spawn((Class, PlayerSelectableClass, all_races_class, StrId::trunc("z_all_races"), ClassRequirements::default()));

        app.update();

        let selection = app.world().resource::<CharacterCreationSelection>();
        assert_eq!(selection.race, Some(human));
        assert_eq!(selection.class, Some(all_races_class));
        assert_eq!(selection.sprites.len(), 2);
        assert_eq!(app.world_mut().resource_mut::<Messages<CharacterCreationRequest>>().drain().count(), 1);
        app.update();
        assert!(app.world().resource::<Messages<CharacterCreationRequest>>().is_empty());

        {
            let mut selection = app.world_mut().resource_mut::<CharacterCreationSelection>();
            selection.race = Some(elf);
            selection.class = None;
            selection.sprites.clear();
            selection.sprites.insert(head_group.hash_id(), picked_head);
            selection.pets.insert(pet, 2);
        }
        app.update();

        let selection = app.world().resource::<CharacterCreationSelection>();
        assert_eq!(selection.race, Some(elf));
        assert_eq!(selection.class, Some(elf_class));
        assert_eq!(selection.sprites.get(&head_group.hash_id()), Some(&picked_head));
        assert_eq!(selection.sprites.get(&body_group.hash_id()), Some(&body_id));
        assert_eq!(selection.pets.get(&pet), Some(&2));

        app.world_mut().resource_mut::<CharacterCreationSelection>().race = None;
        app.update();
        assert_eq!(app.world().resource::<CharacterCreationSelection>().race, Some(elf));
    }

    #[test]
    fn selection_changes_submit_without_button_clicks() {
        let mut app = App::new();
        app.init_resource::<CharacterCreationSelection>()
            .add_message::<CharacterCreationRequest>()
            .add_systems(Update, send_character_selection.run_if(resource_changed::<CharacterCreationSelection>));
        let race = HashId::from("human");
        let class = HashId::from("warrior");
        let group = HashId::from("Head");
        let head = HashId::from("humahe1");
        {
            let mut selection = app.world_mut().resource_mut::<CharacterCreationSelection>();
            selection.race = Some(race);
            selection.sprites.insert(group, head);
        }
        app.update();
        assert!(app.world().resource::<Messages<CharacterCreationRequest>>().is_empty());

        app.world_mut().resource_mut::<CharacterCreationSelection>().class = Some(class);
        app.update();
        let requests: Vec<_> = app.world_mut().resource_mut::<Messages<CharacterCreationRequest>>().drain().collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sprites, vec![(group, head)]);

        app.update();
        assert!(app.world().resource::<Messages<CharacterCreationRequest>>().is_empty());

        let next_head = HashId::from("humahe2");
        app.world_mut().resource_mut::<CharacterCreationSelection>().sprites.insert(group, next_head);
        app.update();
        let requests: Vec<_> = app.world_mut().resource_mut::<Messages<CharacterCreationRequest>>().drain().collect();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].sprites, vec![(group, next_head)]);
    }
}
