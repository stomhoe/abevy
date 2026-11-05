use bevy::{color::palettes::css::{GREY}, input_focus::InputFocus, prelude::*};
use crate::*;

pub fn update_line_edits_text(
    input_focus: Res<InputFocus>,
    mut outline_query: Query<(Entity, &mut Outline)>,
) {
    println!("Input focus changed: {:?}", input_focus.get());
    for (entity, mut outline) in outline_query.iter_mut() {
        if input_focus.get().is_some_and(|active| active == entity) {
            outline.color = Color::WHITE;
        } else {
            outline.color = GREY.into();
        }
    }
}

#[allow(unused_parens, )]
pub fn button_change_color_on_mouse_action(
    mut interaction_query: Query<
        (Has<bevy::ui::Pressed>, Has<bevy::picking::hover::Hovered>, &mut BackgroundColor, Option<&ButtonBackgroundStyle>, ),
        (Or<(Changed<bevy::ui::Pressed>, Changed<bevy::picking::hover::Hovered>)>, With<bevy::ui_widgets::Button>, ),
    >,
) {
    for (pressed, hovered, mut background_color, background_style) in interaction_query.iter_mut() {
        background_color.0 = if pressed {
            background_style.map_or(BUTTON_BG_PRESSED, ButtonBackgroundStyle::pressed)
        } else if hovered {
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
