use bevy::prelude::*;

pub fn direction_arrow(dir: Vec2) -> &'static str {
    if dir == Vec2::ZERO {
        "?"
    } else {
        let angle = dir.y.atan2(dir.x);
        let normalized = ((angle * 4.0 / std::f32::consts::PI + 8.5) as i32 % 8) as usize;
        match normalized {
            0 => "→",
            1 => "↗",
            2 => "↑",
            3 => "↖",
            4 => "←",
            5 => "↙",
            6 => "↓",
            7 => "↘",
            _ => "?",
        }
    }
}