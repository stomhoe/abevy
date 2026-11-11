use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use ::being_shared::*;
use ::sprite_animation_shared::*;
use ::sprite_shared::*;
use common::common_components::*;
use game_common::game_common_states::GameSetupScreen;
use game_common::game_common_string_components::*;
use tilemap_shared::*;
use ui_shared::ui_components::ButtonBackgroundStyle;

use super::character_creation_components::*;

use ui_shared::ui_styles::*;
use ui_shared::ui_functions::*;
use std::cmp::Ordering;
const MUTED_TEXT: Color = Color::srgb(0.82, 0.76, 0.65);
const BRONZE: Color = Color::srgb(0.50, 0.39, 0.27);

fn natural_name_cmp(left: &str, right: &str) -> Ordering {
    fn split_suffix(name: &str) -> (&str, Option<u64>) {
        let suffix_start = name
            .char_indices()
            .rev()
            .take_while(|(_, character)| character.is_ascii_digit())
            .last()
            .map(|(index, _)| index);
        match suffix_start.filter(|&index| index < name.len()) {
            Some(index) => {
                let (prefix, suffix) = name.split_at(index);
                (prefix, suffix.parse::<u64>().ok())
            }
            None => (name, None),
        }
    }
    let (left_prefix, left_number) = split_suffix(left);
    let (right_prefix, right_number) = split_suffix(right);
    match left_prefix.cmp(right_prefix) {
        Ordering::Equal => match (left_number, right_number) {
            (Some(left_number), Some(right_number)) => left_number.cmp(&right_number).then_with(|| left.cmp(right)),
            _ => left.cmp(right),
        },
        ordering => ordering,
    }
}

fn font(asset_server: &AssetServer, bold: bool, size: f32) -> TextFont {
    TextFont { font: asset_server.load(if bold { RETRO_FONT_BOLD } else { RETRO_FONT }).into(), font_size: FontSize::Px(size), ..default() }
}


fn button_style(selected: bool) -> ButtonBackgroundStyle {
    ButtonBackgroundStyle::new(
        Some(if selected { Color::srgb(0.28, 0.26, 0.22) } else { BUTTON_BG_NORMAL }),
        Some(BUTTON_BG_HOVERED),
        Some(BUTTON_BG_PRESSED),
    )
}


fn inner_frame(commands: &mut Commands, parent: Entity, inset: bool) {
    commands.spawn((ChildOf(parent), Node {
        position_type: PositionType::Absolute, left: Val::Px(1.), right: Val::Px(1.), top: Val::Px(1.), bottom: Val::Px(1.),
        border: UiRect::all(Val::Px(1.)), ..default()
    }, if inset { inset_border() } else { bevel_border() }, Pickable::IGNORE));
}

fn choice_button(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parent: Entity,
    label: String,
    action: CharacterCreationButton,
    selected: bool,
) {
    sprite_choice_button(commands, parent, label, action, None, None, 1, 1, 0., asset_server, selected);
}

fn selection_dropdown(
    commands: &mut Commands,
    parent: Entity,
    label: String,
    kind: CharacterCreationDropdownKind,
    selected_icon: Option<SpriteIconData>,
    asset_server: &AssetServer,
    options: impl IntoIterator<Item = (HashId, String, Option<SpriteIconData>)>,
    selected_id: Option<HashId>,
) {
    let wrapper = commands.spawn((
        ChildOf(parent),
        Node { width: Val::Percent(100.), position_type: PositionType::Relative, ..default() },
        CharacterCreationDropdownAnchor(kind),
        ZIndex(0),
    )).id();
    let menu = commands.spawn((
        ChildOf(wrapper),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Percent(100.),
            left: Val::Px(0.),
            width: Val::Auto,
            min_width: Val::Px(230.),
            max_width: Val::Vw(40.),
            border: UiRect::all(Val::Px(1.)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.),
            padding: UiRect::all(Val::Px(5.)),
            ..default()
        },
        BackgroundColor(FIELD),
        BorderColor::all(BRONZE),
        ZIndex(0),
        CharacterCreationDropdownMenu(kind),
        GlobalZIndex(0),
    )).id();
    let toggle = match kind {
        CharacterCreationDropdownKind::Race => CharacterCreationButton::ToggleRaceDropdown,
        CharacterCreationDropdownKind::Class => CharacterCreationButton::ToggleClassDropdown,
    };
    if let Some((path, frame, rows, columns)) = selected_icon {
        sprite_choice_button(commands, wrapper, label, toggle, Some(path), frame, rows, columns, 48., asset_server, false);
    } else {
        choice_button(commands, asset_server, wrapper, label, toggle, false);
    }
    for (id, name, icon) in options {
        if let Some((path, frame, rows, columns)) = icon {
            sprite_choice_button(commands, menu, name, match kind {
                CharacterCreationDropdownKind::Race => CharacterCreationButton::Race(id),
                CharacterCreationDropdownKind::Class => CharacterCreationButton::Class(id),
            }, Some(path), frame, rows, columns, 24., asset_server, selected_id == Some(id));
        } else {
            choice_button(commands, asset_server, menu, name, match kind {
                CharacterCreationDropdownKind::Race => CharacterCreationButton::Race(id),
                CharacterCreationDropdownKind::Class => CharacterCreationButton::Class(id),
            }, selected_id == Some(id));
        }
    }
}

fn pet_control_button(commands: &mut Commands, asset_server: &AssetServer, parent: Entity, label: String, action: CharacterCreationButton) {
    let slot = commands.spawn((ChildOf(parent), Node { width: Val::Px(34.), flex_shrink: 0., ..default() })).id();
    choice_button(commands, asset_server, slot, label, action, false);
}

fn sprite_choice_button(
    commands: &mut Commands,
    parent: Entity,
    label: String,
    action: CharacterCreationButton,
    path: Option<String>,
    frame: Option<usize>,
    rows: u32,
    columns: u32,
    icon_size: f32,
    asset_server: &AssetServer,
    selected: bool,
) {
    let dropdown = matches!(action, CharacterCreationButton::ToggleRaceDropdown | CharacterCreationButton::ToggleClassDropdown | CharacterCreationButton::ToggleSpriteGroup(_));
    let command_button = matches!(action, CharacterCreationButton::Cancel | CharacterCreationButton::Pet { .. });
    let style = if dropdown {
        ButtonBackgroundStyle::new(Some(DROPDOWN_FIELD), Some(Color::srgb(1., 1., 0.95)), Some(Color::srgb(0.7, 0.72, 0.66)))
    } else {
        button_style(selected)
    };
    let button = commands.spawn((ChildOf(parent), Node {
        width: Val::Percent(100.), min_width: Val::Px(0.), min_height: Val::Px(36.), flex_shrink: 0.,
        position_type: PositionType::Relative, align_items: AlignItems::Center,
        justify_content: if command_button { JustifyContent::Center } else { JustifyContent::FlexStart },
        border: UiRect::all(Val::Px(if dropdown || command_button { 2. } else { 1. })),
        border_radius: if dropdown { BorderRadius::ZERO } else { panel_radius() },
        padding: UiRect { left: Val::Px(8.), right: Val::Px(if dropdown { 40. } else { 8. }), top: Val::Px(3.), bottom: Val::Px(3.) },
        column_gap: Val::Px(10.), ..default()
    }, action, bevy::ui_widgets::Button::default(), bevy::picking::hover::Hovered(false),
        BackgroundColor(style.normal()), style,
        if dropdown { inset_border() } else { bevel_border() },
    )).id();
    spawn_sprite_icon(commands, button, path, frame, rows, columns, icon_size, asset_server);
    commands.spawn((ChildOf(button), Node { min_width: Val::Px(0.), flex_shrink: if command_button || dropdown { 0. } else { 1. }, ..default() },
        Text::new(label), font(asset_server, selected || command_button, 18.),
        TextColor(if dropdown { DROPDOWN_TEXT } else { TEXT }), TextLayout::no_wrap(), Pickable::IGNORE));
    if dropdown {
        let arrow = commands.spawn((ChildOf(button), Node {
            position_type: PositionType::Absolute, right: Val::Px(2.), top: Val::Px(2.), bottom: Val::Px(2.),
            width: Val::Px(28.), border: UiRect::all(Val::Px(2.)), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default()
        }, BackgroundColor(FORM), bevel_border(), Pickable::IGNORE)).id();
        let glyph = commands.spawn((ChildOf(arrow), Node { width: Val::Px(9.), height: Val::Px(5.), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }, Pickable::IGNORE)).id();
        for width in [9., 7., 5., 3., 1.] {
            commands.spawn((ChildOf(glyph), Node { width: Val::Px(width), height: Val::Px(1.), flex_shrink: 0., ..default() }, BackgroundColor(TEXT), Pickable::IGNORE));
        }
    }
    if command_button {
        commands.entity(button).insert(control_sheen());
        inner_frame(commands, button, false);
    }
}

type SpriteIconData = (String, Option<usize>, u32, u32);

fn spawn_sprite_icon(
    commands: &mut Commands,
    parent: Entity,
    path: Option<String>,
    frame: Option<usize>,
    rows: u32,
    columns: u32,
    icon_size: f32,
    asset_server: &AssetServer,
) {
    let Some(path) = path else { return; };
    let image = asset_server.load(path);
    let slot = commands.spawn((ChildOf(parent), Node {
        width: Val::Px(icon_size), height: Val::Px(icon_size), flex_shrink: 0.,
        align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default()
    }, Pickable::IGNORE)).id();
    commands.spawn((ChildOf(slot), Node { width: Val::Px(icon_size), height: Val::Px(icon_size), ..default() },
        ImageNode::new(image.clone()), SpriteChoiceIcon { image, frame, rows, columns }, Pickable::IGNORE));
}

fn sprite_icon_data(
    entity: Entity,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
) -> Option<SpriteIconData> {
    sprite_icon_data_inner(entity, sprites, animation_map, animations, true)
}

fn sprite_icon_data_south_idle(
    entity: Entity,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
) -> Option<SpriteIconData> {
    sprite_icon_data_inner(entity, sprites, animation_map, animations, false)
}

fn sprite_icon_data_inner(
    entity: Entity,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
    allow_raw_image_fallback: bool,
) -> Option<SpriteIconData> {
    let Ok((_, _, image_path, mapped_animations)) = sprites.get(entity) else { return None; };
    let south_animation = mapped_animations
        .and_then(|mapped| mapped.0.iter().find_map(|(kind, animation)| (
            kind.direction == CardinalDirection::South
                && !kind.moving.get()
                && kind.state_id.is_none()
        ).then_some(animation)))
        .and_then(|animation| animation_map.0.get_cloned(*animation).ok())
        .and_then(|animation| animations.get(animation).ok())
        .map(|(animation, clips)| (
            clips.0.first().copied().unwrap_or(0),
            animation.rows_cols.0 as u32,
            animation.rows_cols.1 as u32,
            animation.img_path.clone(),
        ));
    let path = south_animation.as_ref()
        .map(|(_, _, _, path)| path.clone())
        .or_else(|| allow_raw_image_fallback.then(|| image_path.map(|path| path.path().to_string())).flatten())?;
    Some((
        path,
        south_animation.as_ref().map(|(frame, _, _, _)| *frame),
        south_animation.as_ref().map(|(_, rows, _, _)| *rows).unwrap_or(1),
        south_animation.as_ref().map(|(_, _, columns, _)| *columns).unwrap_or(4),
    ))
}

fn first_race_head_icon(
    race: HashId,
    races: &Query<(&HashId, &DisplayName, &SetsOfPlayerMonoChoosableSprites), (With<Race>, With<PlayerSelectableRace>, )>,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
) -> Option<SpriteIconData> {
    let (_, _, groups) = races.iter().find(|(id, _, _)| **id == race)?;
    let (_, options) = groups.0.iter().find(|(group, _)| group.as_str().eq_ignore_ascii_case("head"))?;
    let mut entities = options.iter().copied().collect::<Vec<_>>();
    entities.sort_by(|left, right| {
        sprites
            .get(*left)
            .ok()
            .map(|(_, name, _, _)| name.0.as_str())
            .cmp(&sprites.get(*right).ok().map(|(_, name, _, _)| name.0.as_str()))
            .then_with(|| left.index().cmp(&right.index()))
    });
    let entity = entities.into_iter().next()?;
    sprite_icon_data(entity, sprites, animation_map, animations)
}

fn section(commands: &mut Commands, asset_server: &AssetServer, parent: Entity, title: &str) -> Entity {
    let section = commands.spawn((ChildOf(parent), Node {
        position_type: PositionType::Relative, flex_direction: FlexDirection::Column, flex_shrink: 0.,
        min_width: Val::Px(0.), width: Val::Percent(100.), row_gap: Val::Px(14.),
        padding: UiRect::all(Val::Px(18.)), ..default()
    }, BackgroundColor(PANEL))).id();
    carved_frame(commands, section, 10.);
    commands.spawn((ChildOf(section), Node {
        width: Val::Percent(100.), ..default()
    }, retro_caption(asset_server, title, 22.), TextLayout::new(Justify::Center, LineBreak::WordBoundary)));
    section
}

fn description_box(commands: &mut Commands, asset_server: &AssetServer, parent: Entity, description: Option<&Description>, stats: Option<&StatsDescription>) {
    if !description.is_some_and(|text| !text.0.trim().is_empty()) && !stats.is_some_and(|text| !text.0.trim().is_empty()) { return; }
    let container = commands.spawn((ChildOf(parent), Node { width: Val::Percent(100.), min_width: Val::Px(0.), padding: UiRect::all(Val::Px(12.)), flex_direction: FlexDirection::Column, row_gap: Val::Px(10.), border_radius: panel_radius(), ..default() }, BackgroundColor(INSET))).id();
    for (text, is_stats) in [(description.map(|text| text.0.as_str()), false), (stats.map(|text| text.0.as_str()), true)] {
        let Some(text) = text.filter(|text| !text.trim().is_empty()) else { continue; };
        commands.spawn((ChildOf(container), Node { width: Val::Percent(100.), min_width: Val::Px(0.), ..default() }, Text::new(text), font(asset_server, false, 18.), TextColor(if is_stats { MUTED_TEXT } else { TEXT }), TextLayout::new(Justify::Left, LineBreak::WordBoundary)));
    }
}

fn populate_identity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    selection: &CharacterCreationSelection,
    identity: Entity,
    races: &Query<(&HashId, &DisplayName, &SetsOfPlayerMonoChoosableSprites), (With<Race>, With<PlayerSelectableRace>, )>,
    race_texts: &Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Race>, With<PlayerSelectableRace>, )>,
    classes: &Query<(&HashId, &DisplayName, &ClassData), (With<Class>, With<PlayerSelectableClass>, )>,
    class_texts: &Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Class>, With<PlayerSelectableClass>, )>,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
) {
    let mut available_races = races.iter().collect::<Vec<_>>();
    available_races.sort_by(|(_, left, _), (_, right, _)| natural_name_cmp(&left.0, &right.0));
    let mut available_classes = classes.iter().collect::<Vec<_>>();
    available_classes.sort_by(|(_, left, _), (_, right, _)| natural_name_cmp(&left.0, &right.0));
    let race_section = section(commands, asset_server, identity, "Race");
    selection_dropdown(commands, race_section,
        selection.race.and_then(|race| available_races.iter().find(|(id, _, _)| **id == race).map(|(_, name, _)| name.0.clone())).unwrap_or_else(|| "Select race".into()),
        CharacterCreationDropdownKind::Race,
        selection.race.and_then(|race| first_race_head_icon(race, races, sprites, animation_map, animations)),
        asset_server,
        available_races.iter().map(|(id, name, _)| (**id, name.0.clone(), first_race_head_icon(**id, races, sprites, animation_map, animations))),
        selection.race);
    if let Some((_, description, stats)) = selection.race.and_then(|race| race_texts.iter().find(|(id, _, _)| **id == race)) {
        description_box(commands, asset_server, race_section, description, stats);
    }
    let class_section = section(commands, asset_server, identity, "Class");
    selection_dropdown(commands, class_section,
        selection.class.and_then(|class| available_classes.iter().find(|(id, _, _)| **id == class).map(|(_, name, _)| name.0.clone())).unwrap_or_else(|| "Select class".into()),
        CharacterCreationDropdownKind::Class, None, asset_server,
        available_classes.iter().map(|(id, name, _)| (**id, name.0.clone(), None)), selection.class);
    if let Some((_, description, stats)) = selection.class.and_then(|class| class_texts.iter().find(|(id, _, _)| **id == class)) {
        description_box(commands, asset_server, class_section, description, stats);
    }
}

#[allow(unused_parens, )]
fn populate_dynamic_choices(
    commands: &mut Commands,
    asset_server: &AssetServer,
    selection: &CharacterCreationSelection,
    appearance: Entity,
    companions: Entity,
    races: &Query<(&HashId, &DisplayName, &SetsOfPlayerMonoChoosableSprites), (With<Race>, With<PlayerSelectableRace>, )>,
    classes: &Query<(&HashId, &DisplayName, &ClassData), (With<Class>, With<PlayerSelectableClass>, )>,
    sprites: &Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    sprite_entities: &Query<(Entity, &HashId), (With<SpriteConfig>, )>,
    animation_map: &Res<AcAnimationEntityMap>,
    animations: &Query<(&AnimationSeri, &ClipStartFrames), ()>,
    pet_sets: &Query<(&HashId, &PetSetData), (With<PetSet>, )>,
) {
    if let Some(selected_race) = selection.race
        && let Some((_, _, selectable_groups)) = races.iter().find(|(id, _, _)| **id == selected_race)
    {
        for (group, options) in &selectable_groups.0 {
            let group_section = commands.spawn((ChildOf(appearance), Node { width: Val::Percent(100.), flex_direction: FlexDirection::Column, row_gap: Val::Px(8.), margin: UiRect::bottom(Val::Px(8.)), ..default() })).id();
            commands.spawn((ChildOf(group_section), Text::new(group.as_str()), font(asset_server, true, 18.), TextColor(MUTED_TEXT)));
            let group_wrapper = commands.spawn((
                ChildOf(group_section),
                Node { width: Val::Percent(100.), position_type: PositionType::Relative, ..default() },
                SpriteGroupAnchor(group.hash_id()),
                ZIndex(0),
            )).id();
            let group_menu = commands.spawn((
                ChildOf(group_wrapper),
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.),
                    left: Val::Px(0.),
                    width: Val::Auto,
                    min_width: Val::Px(280.),
                    max_width: Val::Vw(45.),
                    border: UiRect::all(Val::Px(1.)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.),
                    padding: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                BackgroundColor(FIELD),
                BorderColor::all(BRONZE),
                ZIndex(0),
                SpriteGroupMenu(group.hash_id()),
                GlobalZIndex(0),
            )).id();
            let mut option_ids = options.iter().copied().collect::<Vec<_>>();
            option_ids.sort_by(|left, right| {
                let left_name = sprites.get(*left).ok().map(|(_, name, _, _)| name.0.as_str());
                let right_name = sprites.get(*right).ok().map(|(_, name, _, _)| name.0.as_str());
                match (left_name, right_name) {
                    (Some(left_name), Some(right_name)) => natural_name_cmp(left_name, right_name),
                    (left_name, right_name) => left_name.cmp(&right_name),
                }
            });
            let selected_option = selection.sprites.get(&group.hash_id()).and_then(|selected| option_ids.iter().find(|option| sprites.get(**option).ok().is_some_and(|(id, _, _, _)| id == selected))).copied();
            let selected_name = selected_option.and_then(|option| sprites.get(option).ok()).map(|(_, name, _, _)| name.0.as_str()).unwrap_or("Select");
            if let Some(selected_option) = selected_option.and_then(|option| sprite_icon_data(option, sprites, animation_map, animations)) {
                sprite_choice_button(
                    commands,
                    group_wrapper,
                    selected_name.to_string(),
                    CharacterCreationButton::ToggleSpriteGroup(group.hash_id()),
                    Some(selected_option.0),
                    selected_option.1,
                    selected_option.2,
                    selected_option.3,
                    48.,
                    asset_server,
                    false,
                );
            } else {
                choice_button(commands, asset_server, group_wrapper, selected_name.to_string(), CharacterCreationButton::ToggleSpriteGroup(group.hash_id()), false);
            }
            for option in option_ids {
                let Ok((sprite_id, sprite_name, image_path, _)) = sprites.get(option) else { continue; };
                let icon_data = sprite_icon_data(option, sprites, animation_map, animations);
                sprite_choice_button(
                    commands,
                    group_menu,
                    sprite_name.0.clone(),
                    CharacterCreationButton::Sprite { group: group.hash_id(), sprite: *sprite_id },
                    icon_data.as_ref().map(|data| data.0.clone()).or_else(|| image_path.map(|path| path.path().to_string())),
                    icon_data.as_ref().and_then(|data| data.1),
                    icon_data.as_ref().map(|data| data.2).unwrap_or(1),
                    icon_data.as_ref().map(|data| data.3).unwrap_or(4),
                    24.,
                    asset_server,
                    selection.sprites.get(&group.hash_id()) == Some(sprite_id),
                );
            }
        }
    }
    if let Some(selected_class) = selection.class
        && let Some((_, _, class_data)) = classes.iter().find(|(id, _, _)| **id == selected_class)
        && let Some(pet_set_id) = class_data.pet_set
        && let Some((_, pet_set)) = pet_sets.iter().find(|(id, _)| **id == pet_set_id)
    {
        for pet in &pet_set.pets {
            let row = commands.spawn((
                ChildOf(companions),
                CompanionHover(pet.race_id), RelativeCursorPosition::default(),
                Node { width: Val::Percent(100.), min_width: Val::Px(0.), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(8.), padding: UiRect::all(Val::Px(8.)), ..default() }, BackgroundColor(FIELD),
            )).id();
            let pet_icon = sprite_entities.iter()
                .find(|(_, id)| **id == pet.race_id)
                .and_then(|(entity, _)| sprite_icon_data_south_idle(entity, sprites, animation_map, animations));
            if let Some((path, frame, rows, columns)) = pet_icon {
                spawn_sprite_icon(commands, row, Some(path), frame, rows, columns, 64., asset_server);
            }
            commands.spawn((
                ChildOf(row),
                Node { flex_grow: 1., min_width: Val::Px(0.), ..default() },
                Text::new(pet.name.clone()),
                font(asset_server, false, 18.),
                TextColor(TEXT),
            ));
            pet_control_button(commands, asset_server, row, "-".into(), CharacterCreationButton::Pet { pet: pet.id, delta: -1 });
            commands.spawn((ChildOf(row), Text::new(selection.pets.get(&pet.id).copied().unwrap_or_default().to_string()), font(asset_server, false, 20.), TextColor(TEXT), TextLayout::no_wrap(), Node { min_width: Val::Px(24.), ..default() }));
            pet_control_button(commands, asset_server, row, "+".into(), CharacterCreationButton::Pet { pet: pet.id, delta: 1 });
        }
        commands.spawn((
            ChildOf(companions),
            Text::new(format!("Selected pets: {} / {}", selection.pets.values().sum::<u32>(), pet_set.max_selected)),
            font(asset_server, false, 18.),
            TextColor(MUTED_TEXT),
            CharacterCreationSummary,
        ));
    }
}

#[allow(unused_parens, )]
pub fn do_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selection: Res<CharacterCreationSelection>,
    races: Query<(&HashId, &DisplayName, &SetsOfPlayerMonoChoosableSprites), (With<Race>, With<PlayerSelectableRace>, )>,
    race_texts: Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Race>, With<PlayerSelectableRace>, )>,
    classes: Query<(&HashId, &DisplayName, &ClassData), (With<Class>, With<PlayerSelectableClass>, )>,
    class_texts: Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Class>, With<PlayerSelectableClass>, )>,
    sprites: Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    sprite_entities: Query<(Entity, &HashId), (With<SpriteConfig>, )>,
    animation_map: Res<AcAnimationEntityMap>,
    animations: Query<(&AnimationSeri, &ClipStartFrames), ()>,
    pet_sets: Query<(&HashId, &PetSetData), (With<PetSet>, )>,
) {
    let root = commands.spawn_scene(bsn! {
        Node { position_type: PositionType::Absolute, left: Val::Px(0.), top: Val::Px(0.), width: Val::Percent(100.), height: Val::Percent(100.), padding: UiRect::axes(Val::Vw(3.), Val::Vh(4.)) }
    }).insert((ImageNode { color: Color::srgb(0.76, 0.75, 0.72), image_mode: NodeImageMode::Stretch, visual_box: VisualBox::BorderBox, ..ImageNode::new(asset_server.load("apocalypse.jpg")) }, ui_shared::ui_components::MenuBackground, CharacterCreationRoot, GlobalZIndex(100), DespawnOnExit::<GameSetupScreen>(GameSetupScreen::CharacterCreation))).id();
    commands.spawn((ChildOf(root), CompanionTooltip, GlobalZIndex(500), Pickable::IGNORE,
        Node { display: Display::None, position_type: PositionType::Absolute, max_width: Val::Px(320.), padding: UiRect::all(Val::Px(12.)), border: UiRect::all(Val::Px(2.)), ..default() },
        BackgroundColor(Color::srgb(0.10, 0.09, 0.07)), bevel_border(), Text::default(), font(&asset_server, false, 18.), TextColor(TEXT)));
    let card = commands.spawn((ChildOf(root), Node {
        width: Val::Percent(100.), height: Val::Percent(100.), min_width: Val::Px(0.), min_height: Val::Px(0.),
        padding: UiRect::all(Val::Px(28.)), flex_direction: FlexDirection::Column, row_gap: Val::Px(24.), ..default()
    }, BackgroundColor(FORM))).id();
    carved_frame(&mut commands, card, 20.);
    commands.spawn((ChildOf(card), Node { width: Val::Percent(100.), min_height: Val::Px(36.), flex_shrink: 0., ..default() }, retro_caption(&asset_server, "Character creation", 28.), TextLayout::new(Justify::Center, LineBreak::WordBoundary)));
    let columns = commands.spawn((ChildOf(card), Node {
        width: Val::Percent(100.), flex_grow: 1., flex_basis: Val::Px(0.), min_height: Val::Px(0.), min_width: Val::Px(0.),
        column_gap: Val::Px(16.), row_gap: Val::Px(16.), flex_wrap: FlexWrap::Wrap, align_content: AlignContent::Start, align_items: AlignItems::Start, overflow: Overflow::scroll_y(), ..default()
    }, CharacterCreationColumns, RelativeCursorPosition::default())).id();
    let identity = commands.spawn((ChildOf(columns), CharacterCreationIdentityChoices, CharacterCreationScrollArea, RelativeCursorPosition::default(), Node {
        flex_basis: Val::Px(0.), flex_grow: 42., min_width: Val::Px(0.), min_height: Val::Px(0.), flex_direction: FlexDirection::Column, row_gap: Val::Px(16.), overflow: Overflow::scroll_y(), ..default()
    })).id();
    let right = commands.spawn((ChildOf(columns), CharacterCreationScrollArea, RelativeCursorPosition::default(), Node {
        flex_basis: Val::Px(0.), flex_grow: 58., min_width: Val::Px(0.), min_height: Val::Px(0.), flex_direction: FlexDirection::Column, row_gap: Val::Px(16.), overflow: Overflow::scroll_y(), ..default()
    })).id();
    let appearance = section(&mut commands, &asset_server, right, "Appearance");
    let companions = section(&mut commands, &asset_server, right, "Companions");
    commands.entity(companions).insert(CharacterCreationCompanionsSection);
    let has_companions = selection.class.and_then(|class| classes.iter().find(|(id, _, _)| **id == class)).and_then(|(_, _, data)| data.pet_set).is_some_and(|pet_set| pet_sets.iter().any(|(id, data)| *id == pet_set && !data.pets.is_empty()));
    if !has_companions {
        commands.entity(companions).entry::<Node>().and_modify(|mut node| node.display = Display::None);
    }
    populate_identity(&mut commands, &asset_server, &selection, identity, &races, &race_texts, &classes, &class_texts, &sprites, &animation_map, &animations);
    let dynamic_appearance = commands.spawn((ChildOf(appearance), CharacterCreationAppearanceChoices, Node { width: Val::Percent(100.), flex_direction: FlexDirection::Column, row_gap: Val::Px(12.), ..default() })).id();
    let dynamic_companions = commands.spawn((ChildOf(companions), CharacterCreationPetChoices, Node { width: Val::Percent(100.), flex_direction: FlexDirection::Column, row_gap: Val::Px(8.), ..default() })).id();
    populate_dynamic_choices(&mut commands, &asset_server, &selection, dynamic_appearance, dynamic_companions, &races, &classes, &sprites, &sprite_entities, &animation_map, &animations, &pet_sets);
    let actions = commands.spawn((ChildOf(card), Node { width: Val::Percent(100.), min_height: Val::Px(60.), flex_shrink: 0., padding: UiRect::top(Val::Px(16.)), border: UiRect::top(Val::Px(1.)), justify_content: JustifyContent::FlexEnd, ..default() }, BorderColor::all(BRONZE))).id();
    let back = commands.spawn((ChildOf(actions), Node { width: Val::Px(210.), min_width: Val::Px(210.), flex_shrink: 0., ..default() })).id();
    choice_button(&mut commands, &asset_server, back, "Back to lobby".into(), CharacterCreationButton::Cancel, false);
}

#[allow(unused_parens, )]
pub fn rebuild_layout_after_selection_change(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selection: Res<CharacterCreationSelection>,
    identity: Query<Entity, (With<CharacterCreationIdentityChoices>, )>,
    appearance: Query<Entity, (With<CharacterCreationAppearanceChoices>, )>,
    companions: Query<Entity, (With<CharacterCreationPetChoices>, )>,
    mut companions_section: Query<(&mut Node, ), (With<CharacterCreationCompanionsSection>, )>,
    races: Query<(&HashId, &DisplayName, &SetsOfPlayerMonoChoosableSprites), (With<Race>, With<PlayerSelectableRace>, )>,
    race_texts: Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Race>, With<PlayerSelectableRace>, )>,
    classes: Query<(&HashId, &DisplayName, &ClassData), (With<Class>, With<PlayerSelectableClass>, )>,
    class_texts: Query<(&HashId, Option<&Description>, Option<&StatsDescription>), (With<Class>, With<PlayerSelectableClass>, )>,
    sprites: Query<(&HashId, &DisplayName, Option<&PathHolder>, Option<&MappedAnimations>), ()>,
    sprite_entities: Query<(Entity, &HashId), (With<SpriteConfig>, )>,
    animation_map: Res<AcAnimationEntityMap>,
    animations: Query<(&AnimationSeri, &ClipStartFrames), ()>,
    pet_sets: Query<(&HashId, &PetSetData), (With<PetSet>, )>,
) {
    if !selection.is_changed() {
        return;
    }
    let Ok(appearance) = appearance.single() else { return; };
    let Ok(companions) = companions.single() else { return; };
    let Ok(identity) = identity.single() else { return; };
    let Ok((mut companions_node, )) = companions_section.single_mut() else { return; };
    let has_companions = selection.class.and_then(|class| classes.iter().find(|(id, _, _)| **id == class)).and_then(|(_, _, data)| data.pet_set).is_some_and(|pet_set| pet_sets.iter().any(|(id, data)| *id == pet_set && !data.pets.is_empty()));
    companions_node.display = if has_companions { Display::Flex } else { Display::None };
    commands.entity(appearance).despawn_related::<Children>();
    commands.entity(companions).despawn_related::<Children>();
    commands.entity(identity).despawn_related::<Children>();
    populate_identity(&mut commands, &asset_server, &selection, identity, &races, &race_texts, &classes, &class_texts, &sprites, &animation_map, &animations);
    populate_dynamic_choices(&mut commands, &asset_server, &selection, appearance, companions, &races, &classes, &sprites, &sprite_entities, &animation_map, &animations, &pet_sets);
}

#[allow(unused_parens, )]
pub fn prepare_sprite_choice_icons(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut icons: Query<(Entity, &SpriteChoiceIcon, &mut ImageNode, &mut Node, ), (Without<SpriteChoiceIconPrepared>, )>,
) {
    for (entity, icon, mut image_node, mut node) in &mut icons {
        let Some(image) = images.get(&icon.image) else { continue; };
        let size = image.size();
        // Match the runtime animation atlas: sprite sheets may contain a few
        // padding pixels, so frame dimensions are integer-divided rather than
        // requiring the image dimensions to divide evenly.
        let (columns, rows) = if icon.columns > 0 && icon.rows > 0 && size.x >= icon.columns && size.y >= icon.rows {
            (icon.columns, icon.rows)
        } else if size.x >= 4 && size.x % 4 == 0 {
            (4, 1)
        } else {
            (1, 1)
        };
        let frame_size = Vec2::new((size.x / columns) as f32, (size.y / rows) as f32);
        let (Val::Px(width), Val::Px(height)) = (node.width, node.height) else { continue; };
        if frame_size.min_element() <= 0. { continue; }
        let scale = (width / frame_size.x).min(height / frame_size.y);
        node.width = Val::Px(frame_size.x * scale);
        node.height = Val::Px(frame_size.y * scale);
        if columns > 1 || rows > 1 {
            let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(size.x / columns, size.y / rows),
                columns,
                rows,
                None,
                None,
            ));
            image_node.texture_atlas = Some(TextureAtlas { layout, index: icon.frame.unwrap_or(0) });
        }
        commands.entity(entity).insert(SpriteChoiceIconPrepared);
    }
}
