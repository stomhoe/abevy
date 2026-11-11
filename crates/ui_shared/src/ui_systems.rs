use bevy::{input_focus::*, picking::events::PointerPress, prelude::*};
use crate::*;

#[allow(unused_parens, )]
pub fn fit_menu_backgrounds(
    images: Res<Assets<Image>>,
    mut backgrounds: Query<(&ComputedNode, &mut ImageNode, ), (With<MenuBackground>, )>,
) {
    for (node, mut image) in &mut backgrounds {
        let Some(texture) = images.get(&image.image) else { continue; };
        let viewport = node.size();
        let source = texture.size().as_vec2();
        if viewport.min_element() <= 0. || source.min_element() <= 0. { continue; }
        let scale = (viewport.x / source.x).max(viewport.y / source.y);
        let cropped = viewport / scale;
        let origin = (source - cropped) * 0.5;
        let rect = Some(Rect::from_corners(origin, origin + cropped));
        if image.rect != rect { image.rect = rect; }
    }
}

#[allow(unused_parens, )]
pub fn apply_retro_font(
    assets: Res<AssetServer>,
    mut fonts: Query<(&mut TextFont, ), (Changed<TextFont>, )>,
) {
    for (mut font, ) in &mut fonts {
        if font.font == Default::default() {
            font.font = assets.load(RETRO_FONT).into();
        }
    }
}

pub fn focus_line_edit_on_pointer_press(
    press: On<PointerPress>,
    mut input_focus: ResMut<InputFocus>,
    line_edits: Query<(), With<LineEdit>>,
) {
    if press.button != PointerButton::Primary || line_edits.get(press.entity).is_err() {
        return;
    }
    input_focus.set(press.entity, FocusCause::Pressed);
}

/// Keeps text fields focusable when the UI picking stack does not dispatch its
/// normal `AcquireFocus` observer (for example when a parent panel owns focus).
pub fn focus_line_edit_on_press(
    mut input_focus: ResMut<InputFocus>,
    pressed_query: Query<(Entity, Has<bevy::ui::Pressed>, ), (With<LineEdit>, Changed<bevy::ui::Pressed>, )>,
) {
    for (entity, pressed) in pressed_query {
        if pressed {
            input_focus.set(entity, FocusCause::Pressed);
        }
    }
}

#[allow(unused_parens, )]
pub fn update_line_edits_text(
    input_focus: Res<InputFocus>,
    mut outline_query: Query<(Entity, &mut Outline, ), (With<LineEdit>, )>,
) {
    for (entity, mut outline) in outline_query.iter_mut() {
        if input_focus.get().is_some_and(|active| active == entity) {
            outline.color = Color::srgba(0.86, 0.78, 0.63, 0.8);
        } else {
            outline.color = Color::NONE;
        }
    }
}

#[allow(unused_parens, )]
pub fn button_change_color_on_mouse_action(
    mut interaction_query: Query<
        (Has<bevy::ui::Pressed>, &bevy::picking::hover::Hovered, &mut BackgroundColor, Option<&ButtonBackgroundStyle>, ),
        (Or<(Changed<bevy::ui::Pressed>, Changed<bevy::picking::hover::Hovered>)>, With<bevy::ui_widgets::Button>, ),
    >,
) {
    for (pressed, hovered, mut background_color, background_style) in interaction_query.iter_mut() {
        background_color.0 = if pressed {
            background_style.map_or(BUTTON_BG_PRESSED, ButtonBackgroundStyle::pressed)
        } else if hovered.0 {
            background_style.map_or(BUTTON_BG_HOVERED, ButtonBackgroundStyle::hovered)
        } else {
            background_style.map_or(BUTTON_BG_NORMAL, ButtonBackgroundStyle::normal)
        };
    }
}

// Submit the focused single-line field on Enter.
pub fn submit_line_edit(
    focus: Res<InputFocus>, keys: Res<ButtonInput<KeyCode>>,
    fields: Query<&bevy::text::EditableText, With<LineEdit>>,
    mut events: MessageWriter<SubmitText>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        if let Some(entity) = focus.get() {
            if let Ok(field) = fields.get(entity) {
                events.write(SubmitText { entity, text: field.value().to_string() });
            }
        }
    }
}
