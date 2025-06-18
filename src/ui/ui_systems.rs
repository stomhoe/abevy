use bevy::{color::palettes::css::{GREY}, input_focus::InputFocus, prelude::*};
use crate::ui::{ui_components::ButtonBackgroundStyle, ui_styles::{BUTTON_BG_HOVERED, BUTTON_BG_NORMAL, BUTTON_BG_PRESSED}};

pub fn update_line_edits_text(
    input_focus: Res<InputFocus>,
    mut outline_query: Query<(Entity, &mut Outline)>,
) {
    if input_focus.is_changed() {
        println!("Input focus changed: {:?}", input_focus.0);
        for (entity, mut outline) in outline_query.iter_mut() {
            if input_focus.0.is_some_and(|active| active == entity) {
                outline.color = Color::WHITE;
            } else {
                outline.color = GREY.into();
            }
        }
    }
}

pub fn button_change_color_on_mouse_action(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&ButtonBackgroundStyle>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, background_style) in &mut interaction_query {
        background_color.0 = match *interaction {
            Interaction::Pressed => {
               if let Some(style) = background_style {
                    style.pressed()
                } else {
                    BUTTON_BG_PRESSED
                }
            },
            Interaction::Hovered => {
                if let Some(style) = background_style {
                    style.hovered()
                } else {
                    BUTTON_BG_HOVERED
                }
            },
            Interaction::None => {
                if let Some(style) = background_style {
                    style.normal()
                } else {
                    BUTTON_BG_NORMAL
                }
            }
        };
    }
}