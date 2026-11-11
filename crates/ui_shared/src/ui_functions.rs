use bevy::{prelude::*};

use crate::*;

pub fn retro_caption(assets: &AssetServer, text: impl Into<String>, size: f32) -> impl Bundle {
    (Text::new(text), TextFont { font: assets.load(TITLE_FONT).into(), font_size: FontSize::Px(size), ..default() }, TextColor(TEXT), TextShadow { offset: Vec2::new(1., 1.), color: Color::BLACK }, bevy::picking::Pickable::IGNORE)
}

/// Blended light and shadow edges, matching AcOLD's overlapping StyleBox panels.
/// The decorative nodes ignore picking so they never cover the controls inside.
pub fn carved_frame(commands: &mut Commands, parent: Entity, thickness: f32) {
    commands.entity(parent).entry::<Node>().and_modify(|mut node| node.border_radius = panel_radius());
    for (vertical, trailing) in [(false, false), (true, false), (false, true), (true, true)] {
        let color = if trailing { Color::srgba(0.025, 0.018, 0.008, 0.68) } else { Color::srgba(0.84, 0.78, 0.65, 0.57) };
        let thickness = thickness * 1.35;
        let mut node = Node { position_type: PositionType::Absolute, ..default() };
        if vertical {
            node.top = Val::Px(0.); node.bottom = Val::Px(0.); node.width = Val::Px(thickness * 0.7);
            if trailing { node.right = Val::Px(0.); } else { node.left = Val::Px(0.); }
        } else {
            node.left = Val::Px(0.); node.right = Val::Px(0.); node.height = Val::Px(thickness);
            if trailing { node.bottom = Val::Px(0.); } else { node.top = Val::Px(0.); }
        }
        let radius = Val::Px(3.).into();
        if vertical {
            if trailing { node.border_radius.top_right = radius; node.border_radius.bottom_right = radius; }
            else { node.border_radius.top_left = radius; node.border_radius.bottom_left = radius; }
        } else if trailing {
            node.border_radius.bottom_left = radius; node.border_radius.bottom_right = radius;
        } else {
            node.border_radius.top_left = radius; node.border_radius.top_right = radius;
        }
        let angle = if vertical { LinearGradient::TO_RIGHT } else { LinearGradient::TO_BOTTOM };
        let (start, end) = if trailing { (Color::NONE, color) } else { (color, Color::NONE) };
        commands.spawn((ChildOf(parent), node, BackgroundGradient::from(LinearGradient::new(angle, vec![ColorStop::new(start, Val::Percent(0.)), ColorStop::new(end, Val::Percent(100.))])), bevy::picking::Pickable::IGNORE));
    }
}

//TODO LA FUENTE
pub fn text_button<T: Into<String>>(base: impl Bundle, text: T, other_children: impl Bundle, style: Option<ButtonBackgroundStyle>,) -> impl Bundle
{

    let implicit_children = (
        TextLayout::new(Justify::Center, LineBreak::WordBoundary),
    );

    button(base, (Text::new(text), other_children, implicit_children), style)
}

pub fn button(
    base: impl Bundle,
    children: impl Bundle,
    style: Option<ButtonBackgroundStyle>,
) -> impl Bundle {
    let style = style.unwrap_or_default();
    (
        base,
        bevy::ui_widgets::Button::default(),
        bevy::picking::hover::Hovered(false),
        BackgroundColor(style.normal()),
        bevel_border(),
        control_sheen(),
        style,
        children![children],
    )
}


pub const fn color_from_triplicate(value: f32) -> Color {
    Color::srgb(value, value, value)
}

// #[derive(Bundle, Clone, Debug)]
// pub struct BorderBundle{
//     pub node: Node,
//     pub border_gradient: BorderGradient,
// }
// impl BorderBundle {
//     pub fn new(
//         node: Node,
//         gradient: LinearGradient,
//     ) -> Self {
//         BorderBundle {
//             node,
//             border_gradient: BorderGradient::from(gradient),
//         }
//     }
// }
// pub fn produce_gradient_border(
//     thickness: f32,
//     stops_tl: Vec<bevy_ui_gradients::ColorStop>,
//     stops_br: Vec<bevy_ui_gradients::ColorStop>,
// ) -> [BorderBundle; 4]
// {
//     let left = BorderBundle::new(
//         Node {
//             width: Val::Px(thickness),
//             height: Val::Percent(100.0),
//             position_type: PositionType::Absolute,
//             left: Val::Px(0.),
//             top: Val::Px(0.),
//             border: UiRect {
//                 left: Val::Px(thickness),
//                 ..default()
//             },
//             ..default()
//         },
//         LinearGradient {
//             angle: LinearGradient::TO_RIGHT,
//             stops: stops_tl.clone(),
//         },
//     );

//     let top = BorderBundle::new(
//         Node {
//             width: Val::Percent(100.0),
//             height: Val::Px(thickness),
//             position_type: PositionType::Absolute,
//             left: Val::Px(0.),
//             top: Val::Px(0.),
//             border: UiRect {
//                 top: Val::Px(thickness),
//                 ..default()
//             },
//             ..default()
//         },
//         LinearGradient {
//             angle: LinearGradient::TO_BOTTOM,
//             stops: stops_tl.clone(),
//         },
//     );

//     let right = BorderBundle::new(
//         Node {
//             width: Val::Px(thickness),
//             height: Val::Percent(100.0),
//             position_type: PositionType::Absolute,
//             right: Val::Px(0.),
//             top: Val::Px(0.),
//             border: UiRect {
//                 right: Val::Px(thickness),
//                 ..default()
//             },
//             ..default()
//         },
//         LinearGradient {
//             angle: LinearGradient::TO_LEFT,
//             stops: stops_br.clone(),
//         },
//     );

//     let bottom = BorderBundle::new(
//         Node {
//             width: Val::Percent(100.0),
//             height: Val::Px(thickness),
//             position_type: PositionType::Absolute,
//             left: Val::Px(0.),
//             bottom: Val::Px(0.),
//             border: UiRect {
//                 bottom: Val::Px(thickness),
//                 ..default()
//             },
//             ..default()
//         },
//         LinearGradient {
//             angle: LinearGradient::TO_TOP,
//             stops: stops_br.clone(),
//         },
//     );

//     [left, top, right, bottom]
// }
