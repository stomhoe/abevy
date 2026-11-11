use bevy::prelude::*;
use common::common_states::*;
use ui_shared::ui_components::*;
use ui_shared::ui_functions::*;
use ui_shared::ui_styles::*;

use crate::main_menu_components::*;

#[allow(unused_parens, )]
pub fn layout(mut commands: Commands, assets: Res<AssetServer>) {
    let root = commands.spawn((
        Node { width: Val::Percent(100.), height: Val::Percent(100.), padding: UiRect::axes(Val::Vw(5.), Val::Vh(6.)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, overflow: Overflow::scroll_y(), ..default() },
        ImageNode { color: Color::srgb(0.85, 0.83, 0.78), image_mode: NodeImageMode::Stretch, visual_box: VisualBox::BorderBox, ..ImageNode::new(assets.load("apocalypse.jpg")) }, MenuBackground,
        DespawnOnExit(PreGameState::MainMenu), DespawnOnExit(AppState::NoSession),
    )).id();
    let sheet = commands.spawn((ChildOf(root), Node {
        width: Val::Px(820.), max_width: Val::Percent(100.), flex_shrink: 0.,
        padding: UiRect::all(Val::Px(28.)), flex_direction: FlexDirection::Column, row_gap: Val::Px(24.), ..default()
    }, BackgroundColor(FORM))).id();
    carved_frame(&mut commands, sheet, 20.);
    commands.spawn((ChildOf(sheet), Node { width: Val::Percent(100.), ..default() },
        retro_caption(&assets, "Argentum Coop", 32.), TextLayout::new(Justify::Center, LineBreak::WordBoundary)));
    let frame = commands.spawn((ChildOf(sheet), Node {
        width: Val::Percent(100.),
        flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(28.), row_gap: Val::Px(24.), ..default()
    })).id();
    let local = commands.spawn((ChildOf(frame), Node {
        flex_basis: Val::Px(290.), flex_grow: 1., min_width: Val::Px(0.),
        padding: UiRect::all(Val::Px(20.)), flex_direction: FlexDirection::Column, row_gap: Val::Px(14.), ..default()
    }, BackgroundColor(PANEL))).id();
    carved_frame(&mut commands, local, 10.);
    commands.spawn((ChildOf(local), Node { width: Val::Percent(100.), ..default() }, retro_caption(&assets, "Partida", 22.), TextLayout::new(Justify::Center, LineBreak::WordBoundary)));
    commands.spawn((ChildOf(local), main_menu_button(&assets, MainMenuButton::QuickStart, "Jugar")));
    commands.spawn((ChildOf(local), main_menu_button(&assets, MainMenuButton::Host, "Crear partida")));
    let network = commands.spawn((ChildOf(frame), Node {
        flex_basis: Val::Px(290.), flex_grow: 1., min_width: Val::Px(0.),
        padding: UiRect::all(Val::Px(20.)), flex_direction: FlexDirection::Column, row_gap: Val::Px(14.), ..default()
    }, BackgroundColor(PANEL))).id();
    carved_frame(&mut commands, network, 10.);
    commands.spawn((ChildOf(network), Node { width: Val::Percent(100.), ..default() }, retro_caption(&assets, "Servidor", 22.), TextLayout::new(Justify::Center, LineBreak::WordBoundary)));
    commands.spawn((ChildOf(network), Node {
        width: Val::Percent(100.), height: Val::Px(44.), padding: UiRect::all(Val::Px(8.)),
        border: UiRect::all(Val::Px(2.)), overflow: Overflow::clip_x(), ..default()
    }, bevy::input_focus::AutoFocus, bevy::input_focus::tab_navigation::TabIndex(0),
        LineEdit, bevy::text::EditableText::new("127.0.0.1"), CurrentText::new("127.0.0.1"), MainMenuIpLineEdit,
        inset_border(), TextColor(TEXT), BackgroundColor(FIELD),
        Outline { color: Color::srgba(0.7, 0.67, 0.58, 0.6), width: Val::Px(1.), offset: Val::Px(0.) },
        TextLayout::no_wrap(), TextFont { font: assets.load(RETRO_FONT).into(), font_size: FontSize::Px(20.), ..default() }));
    commands.spawn((ChildOf(network), main_menu_button(&assets, MainMenuButton::Join, "Conectar")));
}

fn main_menu_button(assets: &AssetServer, action: MainMenuButton, label: &str) -> impl Bundle {
    button((action, Node {
        width: Val::Percent(100.), min_width: Val::Px(0.), height: Val::Px(44.),
        padding: UiRect::all(Val::Px(5.)), border: UiRect::all(Val::Px(2.)),
        border_radius: panel_radius(),
        justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()
    }), (Text::new(label), TextFont { font: assets.load(RETRO_FONT).into(), font_size: FontSize::Px(20.), ..default() }, TextColor(TEXT), bevy::picking::Pickable::IGNORE, TextLayout::no_wrap()), None)
}
